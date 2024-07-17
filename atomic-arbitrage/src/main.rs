use color_eyre::eyre::{bail, ensure, eyre, Result};
use ekubo::{
    models::{PoolKey, Quote, Quotes, RouteNode},
    Client,
};
use futures::future::join_all;
use starknet::{
    accounts::{Account, Call, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id,
        types::{
            BlockId, BlockTag, Felt, FunctionCall, TransactionReceiptWithBlockInfo,
            TransactionStatus, U256,
        },
        utils::get_selector_from_name,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
    signers::{LocalWallet, SigningKey},
};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::env;
use std::iter;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

struct ArbitrageOpportunity {
    amount: Felt,
    quotes: Quotes,
    profit: Felt,
}

async fn check_arbitrage(
    client: &Client,
    amount: Felt,
    min_profit: Felt,
    token_address: &str,
    max_splits: u8,
    max_hops: u8,
) -> Option<ArbitrageOpportunity> {
    let quotes = client
        .quotes(amount, token_address, token_address, max_splits, max_hops)
        .await
        .map_err(|e| {
            error!("quotes err: {e:#?}");
            e
        })
        .ok()?;
    debug!("quotes for amount {amount}:\n{quotes:#?}");
    let total = quotes.total;
    let extensions: HashSet<Felt> = quotes
        .splits
        .iter()
        .map(|quote| quote.route.iter())
        .flatten()
        .filter_map(|node| {
            let ext = node.pool_key.extension;
            (ext != Felt::ZERO).then_some(ext)
        })
        .collect();
    // We only check arbitrage opporunities in pools without extensions to prevent any front-running or other activities
    // It is possible to extend to pools with some white-listed extensions
    (extensions.is_empty() && total > amount + min_profit && !quotes.splits.is_empty()).then(|| {
        ArbitrageOpportunity {
            amount,
            quotes,
            profit: total - amount,
        }
    })
}

// RouteNode in the ABI
fn node_to_array(node: RouteNode) -> [Felt; 8] {
    let RouteNode {
        pool_key:
            PoolKey {
                token0,
                token1,
                fee,
                tick_spacing,
                extension,
            },
        sqrt_ratio_limit,
        skip_ahead,
    } = node;
    let sqrt_ratio = U256::from(sqrt_ratio_limit);
    let low = Felt::from(sqrt_ratio.low());
    let high = Felt::from(sqrt_ratio.high());
    [
        token0,
        token1,
        fee,
        tick_spacing.into(),
        extension,
        // Note on `sqrt_ratio_limit` is U256 in signatures
        // It means that Starknet expects two Felts (low and high)
        // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/
        // It is ok to operate with it as a Felt as it seems that it is 192 bits - see https://github.com/EkuboProtocol/abis/blob/main/src/types/pool_price.cairo
        low,
        high,
        skip_ahead,
    ]
}

// multihop_swap in the ABI
// Does a multihop swap, where the output/input of each hop is passed as input/output of the next swap
// see also https://github.com/EkuboProtocol/abis/blob/main/src/router_lite.cairo
fn call_data_for_split(split: Quote, token_address: Felt) -> impl Iterator<Item = Felt> {
    let specified_amount = split.specified_amount;
    iter::once(Felt::from(split.route.len()))
        .chain(split.route.into_iter().flat_map(node_to_array))
        .chain(iter::once(token_address))
        .chain(iter::once(specified_amount))
        .chain(iter::once(Felt::ZERO)) // we specify an exact input amount, so it is positive
}

// multi_multihop_swap in the ABI
// Does multiple multihop swaps
// see also https://github.com/EkuboProtocol/abis/blob/main/src/router_lite.cairo
fn call_data_for_multisplit(splits: Vec<Quote>, token_address: Felt) -> impl Iterator<Item = Felt> {
    iter::once(Felt::from(splits.len())).chain(
        splits
            .into_iter()
            .flat_map(move |split| call_data_for_split(split, token_address)),
    )
}

fn get_chain_id(ekubo_url: &str, provider_url: &str) -> Result<Felt> {
    if ekubo_url.contains("sepolia") {
        ensure!(
            provider_url.contains("sepolia"),
            "Ekubo API and RPC provider urls should point to the same chain: Sepolia"
        );
        Ok(chain_id::SEPOLIA)
    } else if ekubo_url.contains("mainnet") {
        ensure!(
            provider_url.contains("mainnet"),
            "Ekubo API and RPC provider urls should point to the same chain: Mainnet"
        );
        Ok(chain_id::MAINNET)
    } else {
        bail!("unsupported chain - verify environment variables")
    }
}

async fn get_account_balance(
    token_contract: Felt,
    account_address: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<U256> {
    let felts = provider
        .call(
            FunctionCall {
                contract_address: token_contract,
                entry_point_selector: get_selector_from_name("balanceOf")?,
                calldata: vec![account_address],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .map_err(|e| eyre!("Error when fetching account balance:\n{e:#?}"))?;
    let low = u128::from_le_bytes(felts[0].to_bytes_le()[0..16].try_into()?);
    let high = u128::from_le_bytes(felts[1].to_bytes_le()[0..16].try_into()?);
    // the unit of data in Cairo is Felt (u252) but ERC20 standard suggests to return u256 from balanceOf
    // So the token contract returns a pair of low (128 bits) and high (128 bits) to construct a u256
    Ok(U256::from_words(low, high))
}

// Wait for the transaction to be accepted
async fn wait_for_transaction(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: Felt,
) -> Result<TransactionReceiptWithBlockInfo> {
    let mut retries = 200;
    let retry_interval = Duration::from_millis(3000);

    while retries >= 0 {
        tokio::time::sleep(retry_interval).await; // sleep before the tx status to give some time for a tx get to the provider node
        let status = provider
            .get_transaction_status(tx_hash)
            .await
            .map_err(|e| eyre!("failed to get tx status: {e:#?}"))?;
        retries -= 1;
        match status {
            TransactionStatus::Received => continue,
            TransactionStatus::Rejected => bail!("transaction is rejected"),
            TransactionStatus::AcceptedOnL2(_) | TransactionStatus::AcceptedOnL1(_) => {
                match provider.get_transaction_receipt(tx_hash).await {
                    Ok(receipt) => return Ok(receipt),
                    // For some nodes even though the transaction has execution status SUCCEEDED finality status ACCEPTED_ON_L2,
                    // get_transaction_receipt returns "Transaction hash not found"
                    // see https://github.com/starknet-io/starknet.js/blob/v6.7.0/src/channel/rpc_0_7.ts#L248
                    Err(_) => continue,
                }
            }
        }
    }
    bail!("maximum retries attempts")
}

#[allow(unreachable_code)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let token_address_hex = env::var("TOKEN_TO_ARBITRAGE")?;
    let token_address = Felt::from_hex(&token_address_hex)?;
    let router_address_hex = env::var("ROUTER_ADDRESS")?;
    let router_address = Felt::from_hex(&router_address_hex)?;
    let url = env::var("EKUBO_URL")?;
    let provider_url = env::var("JSON_RPC_URL")?;
    let explorer_url = env::var("EXPLORER_TX_PREFIX")?;
    info!("starting bot with Ekubo API {url} and RPC {provider_url}");
    let chain_id = get_chain_id(&url, &provider_url)?;

    let client = Client::new(url, "atomic-bot".to_string());
    let rpc_transport = HttpTransport::new(Url::parse(&provider_url)?);
    let provider = JsonRpcClient::new(rpc_transport);
    ensure!(chain_id == provider.chain_id().await?);

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(&env::var(
        "ACCOUNT_PRIVATE_KEY",
    )?)?));
    let account_address = Felt::from_hex(&env::var("ACCOUNT_ADDRESS")?)?;

    let mut account = SingleOwnerAccount::new(
        provider,
        signer,
        account_address,
        chain_id,
        ExecutionEncoding::New, // https://docs.rs/starknet/0.11.0/starknet/accounts/enum.ExecutionEncoding.html#variant.New,
    );
    // otherwise we will get a DuplicateTx error as nonce by default set to the latest block
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let transfer_selector = get_selector_from_name("transfer")?;
    let clear_minimum_selector = get_selector_from_name("clear_minimum")?;
    let multihop_swap_selector = get_selector_from_name("multihop_swap")?;
    let multi_multihop_swap_selector = get_selector_from_name("multi_multihop_swap")?;

    let min_power: u8 = 32.max(env::var("MIN_POWER_OF_2")?.parse()?);
    let max_power: u8 = (min_power + 1).max(65.min(env::var("MAX_POWER_OF_2")?.parse()?));

    let amounts_to_quote: Vec<Felt> = (min_power..max_power)
        .map(|p| Felt::from(2u8).pow(p))
        .collect();

    let max_splits: u8 = env::var("MAX_SPLITS")?.parse()?;
    let max_hops: u8 = env::var("MAX_HOPS")?.parse()?;
    let min_profit = Felt::from_dec_str(&env::var("MIN_PROFIT")?)?;
    let num_top_quotes: usize = env::var("NUM_TOP_QUOTES_TO_ESTIMATE")?.parse()?;
    let check_interval: u64 = env::var("CHECK_INTERVAL_MS")?.parse()?;

    loop {
        let account_balance =
            get_account_balance(token_address, account_address, account.provider()).await?;
        info!("Account balance: {account_balance} WEI");

        let mut opportunities: Vec<ArbitrageOpportunity> = join_all(
            amounts_to_quote
                .iter()
                .filter(|&&amount| U256::from(amount) <= account_balance)
                .map(|&amount| {
                    check_arbitrage(
                        &client,
                        amount,
                        min_profit,
                        &token_address_hex,
                        max_splits,
                        max_hops,
                    )
                }),
        )
        .await
        .into_iter()
        .flatten()
        .collect();
        opportunities.sort_unstable_by_key(|opportunity| Reverse(opportunity.profit));
        let top: Option<(Felt, Felt, [Call; 3])> = opportunities
            .into_iter()
            .take(num_top_quotes)
            .find_map(|opportunity| {
                // transfer takes the second argument (amount) as U256
                // pay the input
                let transfer_call = Call {
                    to: token_address,
                    selector: transfer_selector,
                    calldata: vec![router_address, opportunity.amount, Felt::ZERO],
                };

                // clear_minimum takes the second argument (amount) as U256
                // withdraw the output
                let clear_profits_call = Call {
                    to: router_address,
                    selector: clear_minimum_selector,
                    calldata: vec![token_address, opportunity.amount, Felt::ZERO],
                };
                let profit = opportunity.profit;
                let amount = opportunity.amount;
                let mut splits = opportunity.quotes.splits;

                let call = if splits.len() == 1 {
                    let split = splits.pop()?;
                    if split.route.len() == 1 {
                        error!("unexpected single hop route");
                        return None;
                    }
                    Call {
                        to: router_address,
                        selector: multihop_swap_selector,
                        calldata: call_data_for_split(split, token_address).collect(),
                    }
                } else {
                    Call {
                        to: router_address,
                        selector: multi_multihop_swap_selector,
                        calldata: call_data_for_multisplit(splits, token_address).collect(),
                    }
                };

                Some((profit, amount, [transfer_call, call, clear_profits_call]))
            });
        if let Some((profit, amount, calls)) = top {
            info!("top arbitrage profit: {profit}, amount {amount}");
            info!("Executing top arbitrage:\n{calls:#?}");
            let cost = account.execute_v1(calls.to_vec()).estimate_fee().await?;
            // gas fees in WEI as we use tx v1,
            // see https://docs.rs/starknet/0.11.0/starknet/core/types/struct.FeeEstimate.html
            let total_gas_cost_wei = cost.overall_fee;
            info!("cost etimation:\n{total_gas_cost_wei:#?}");
            // Get a tx receipt, actual fee, link to the explorer with tx
            let limit_fee = total_gas_cost_wei * Felt::TWO;
            info!("cost etimation:\n{total_gas_cost_wei}");
            info!("profit etimation:\n{profit}, limit fee:\n{limit_fee}");
            // We can make this comparison as both the swapped token and limit fee are nominated in ETH
            if profit > limit_fee {
                let tx = account
                    .execute_v1(calls.to_vec())
                    .max_fee(limit_fee)
                    .send()
                    .await
                    .map_err(|e| eyre!("Error while sending arbitrage transaction:\n{e:#?}"))?;
                info!(
                    "sent transaction:\n{explorer_url}{:#x}",
                    tx.transaction_hash
                );
                match wait_for_transaction(account.provider(), tx.transaction_hash).await {
                    Ok(receipt) => info!("Transaction receipt: {receipt:#?}"),
                    Err(e) => error!("Arbitrage transaction failed: {e:#?}"),
                }
            } else {
                info!("Non-profitable opportunity");
            }
        }
        sleep(Duration::from_millis(check_interval)).await;
    }

    Ok(())
}
