use color_eyre::eyre::Result;
use ekubo::{models::Quotes, Client};
use futures::future::join_all;
use std::env;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

struct ArbitrageOpportunity {
    amount: u128,
    quotes: Quotes,
    profit: u128,
}

async fn check_arbitrage(
    client: &Client,
    amount: u128,
    min_profit: u128,
    token_address: &str,
    max_splits: u8,
    max_hops: u8,
) -> Option<ArbitrageOpportunity> {
    let quotes = client
        .quotes(amount, token_address, token_address, max_splits, max_hops)
        .await
        .ok()?;
    let total: u128 = quotes.total.parse().ok()?;
    let profit = total - amount;
    (profit > min_profit && !quotes.splits.is_empty()).then_some(ArbitrageOpportunity {
        amount,
        quotes,
        profit,
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let client = Client::new(env::var("EKUBO_URL")?, "atomic-bot".to_string());

    let token_address = env::var("TOKEN_TO_ARBITRAGE")?;

    // let response = client
    //     .quotes(36028797018963968, &token_address, &token_address, 2, 3)
    //     .await?;
    // info!("{:#?}", response);

    let min_power: u8 = 32.max(env::var("MIN_POWER_OF_2")?.parse()?);
    let max_power: u8 = (min_power + 1).max(65.min(env::var("MAX_POWER_OF_2")?.parse()?));
    dbg!(min_power, max_power);

    let amounts_to_quote: Vec<u128> = (min_power..max_power)
        .map(|p| 2u128.pow(p.into()))
        .collect();
    dbg!(&amounts_to_quote);
    let max_splits: u8 = env::var("MAX_SPLITS")?.parse()?;
    let max_hops: u8 = env::var("MAX_HOPS")?.parse()?;
    let min_profit: u128 = env::var("MIN_PROFIT")?.parse()?;
    loop {
        let opportunities: Vec<ArbitrageOpportunity> =
            join_all(amounts_to_quote.iter().map(|&amount| {
                check_arbitrage(
                    &client,
                    amount,
                    min_profit,
                    &token_address,
                    max_splits,
                    max_hops,
                )
            }))
            .await
            .into_iter()
            .flatten()
            .collect();
        // TODO sort by profit, limit by NUM_TOP_QUOTES_TO_ESTIMATE
    }

    Ok(())
}
