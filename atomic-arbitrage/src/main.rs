use color_eyre::eyre::Result;
use ekubo::{
    models::{PoolKey, Quote, Quotes, RouteNode},
    Client,
};
use futures::future::join_all;
use starknet::{
    accounts::Call,
    core::{types::Felt, utils::get_selector_from_name},
};
use std::cmp::Reverse;
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
    (total > amount + min_profit && !quotes.splits.is_empty()).then(|| ArbitrageOpportunity {
        amount,
        quotes,
        profit: total - amount,
    })
}

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
    [
        token0,
        token1,
        fee,
        tick_spacing.into(),
        extension,
        Felt::from(sqrt_ratio_limit.to_bigint() % Felt::from(2u8).pow(128u8).to_bigint()),
        Felt::from(sqrt_ratio_limit.to_bigint() >> 128),
        skip_ahead,
    ]
}

fn call_data_for_split(split: Quote, token_address: Felt) -> impl Iterator<Item = Felt> {
    let specified_amount = split.specified_amount;
    iter::once(Felt::from(split.route.len()))
        .chain(split.route.into_iter().flat_map(node_to_array))
        .chain(iter::once(token_address))
        .chain(iter::once(specified_amount))
        .chain(iter::once(Felt::ZERO))
}

fn call_data_for_multisplit(splits: Vec<Quote>, token_address: Felt) -> impl Iterator<Item = Felt> {
    iter::once(Felt::from(splits.len())).chain(
        splits
            .into_iter()
            .flat_map(move |split| call_data_for_split(split, token_address)),
    )
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

    let client = Client::new(env::var("EKUBO_URL")?, "atomic-bot".to_string());

    let token_address_hex = env::var("TOKEN_TO_ARBITRAGE")?;
    let token_address = Felt::from_hex(&token_address_hex)?;
    let router_address_hex = env::var("ROUTER_ADDRESS")?;
    let router_address = Felt::from_hex(&router_address_hex)?;
    let transfer_selector = get_selector_from_name("transfer")?;
    let clear_minimum_selector = get_selector_from_name("clear_minimum")?;
    let multihop_swap_selector = get_selector_from_name("multihop_swap")?;

    let min_power: u8 = 32.max(env::var("MIN_POWER_OF_2")?.parse()?);
    let max_power: u8 = (min_power + 1).max(65.min(env::var("MAX_POWER_OF_2")?.parse()?));
    dbg!(min_power, max_power);

    let amounts_to_quote: Vec<Felt> = (min_power..max_power)
        .map(|p| Felt::from(2u8).pow(p))
        .collect();
    dbg!(&amounts_to_quote);

    let max_splits: u8 = env::var("MAX_SPLITS")?.parse()?;
    let max_hops: u8 = env::var("MAX_HOPS")?.parse()?;
    let min_profit = Felt::from_dec_str(&env::var("MIN_PROFIT")?)?;
    let num_top_quotes: usize = env::var("NUM_TOP_QUOTES_TO_ESTIMATE")?.parse()?;
    let check_interval: u64 = env::var("CHECK_INTERVAL_MS")?.parse()?;
    loop {
        let mut opportunities: Vec<ArbitrageOpportunity> =
            join_all(amounts_to_quote.iter().map(|&amount| {
                check_arbitrage(
                    &client,
                    amount,
                    min_profit,
                    &token_address_hex,
                    max_splits,
                    max_hops,
                )
            }))
            .await
            .into_iter()
            .flatten()
            .collect();
        opportunities.sort_unstable_by_key(|opportunity| Reverse(opportunity.profit));
        let calls: Option<[Call; 3]> =
            opportunities
                .into_iter()
                .take(num_top_quotes)
                .find_map(|opportunity| {
                    let transfer_call = Call {
                        to: token_address,
                        selector: transfer_selector,
                        calldata: vec![router_address, opportunity.amount, Felt::ZERO],
                    };

                    let clear_profits_call = Call {
                        to: router_address,
                        selector: clear_minimum_selector,
                        calldata: vec![token_address, opportunity.amount, Felt::ZERO],
                    };

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
                            selector: multihop_swap_selector,
                            calldata: call_data_for_multisplit(splits, token_address).collect(),
                        }
                    };

                    Some([transfer_call, call, clear_profits_call])
                });
        match calls {
            Some(calls) => {
                info!("Executing top arbitrage:\n{calls:#?}")
            }
            None => info!("No arbitrage found"),
        }
        sleep(Duration::from_millis(check_interval)).await;
    }

    Ok(())
}
