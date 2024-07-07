use color_eyre::eyre::Result;
use ekubo::Client;

use std::env;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let client = Client::new(env::var("EKUBO_URL")?, "atomic-bot".to_string());

    let token = env::var("TOKEN_TO_ARBITRAGE")?;
    info!("token {}", token);

    let response = client.quote("36028797018963968", &token, &token).await?;
    info!("{:?}", response);

    Ok(())
}
