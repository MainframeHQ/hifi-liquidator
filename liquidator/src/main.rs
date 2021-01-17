use anyhow::{Context, Result as AnyhowResult};
use ethers::prelude::*;
use gumdrop::Options;
use hifi_liquidator::sentinel::Sentinel;
use hifi_liquidator_structs::{Config, Opts};
use std::{convert::TryFrom, fs::OpenOptions, sync::Arc, time::Duration};
use tracing::info;
use tracing_subscriber::{filter::EnvFilter, fmt::Subscriber};

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let opts = Opts::parse_args_default_or_exit();

    if opts.url.starts_with("http") {
        let provider = Provider::<Http>::try_from(opts.url.clone())?;
        run(opts, provider).await?;
    } else {
        let ws = Ws::connect(opts.url.clone()).await?;
        let provider = Provider::new(ws);
        run(opts, provider).await?;
    }

    Ok(())
}

async fn run<P: JsonRpcClient + 'static>(opts: Opts, provider: Provider<P>) -> AnyhowResult<()> {
    info!("Starting Hifi liquidator");
    let provider = provider.interval(Duration::from_millis(opts.interval));
    let wallet: LocalWallet = std::fs::read_to_string(&opts.private_key)
        .with_context(|| format!("Could not read private key: {:?}", &opts.private_key))?
        .parse()
        .with_context(|| "Could not parse private key")?;
    let address = wallet.address();
    let signer_middleware = SignerMiddleware::new(provider, wallet);
    let nonce_manager_middleware = NonceManagerMiddleware::new(signer_middleware, address);
    let client = Arc::new(nonce_manager_middleware);

    info!("Profits will be sent to {:?}", address);
    info!("Node: {}", opts.url);
    info!("Persistent data will be stored in: {:?}", opts.db_file);

    let config: Config = serde_json::from_reader(std::fs::File::open(opts.config)?)?;
    info!("BalanceSheet: {:?}", config.balance_sheet);
    info!("FyTokens: {:?}", config.fy_tokens);
    info!("HifiFlashSwap {:?}", config.hifi_flash_swap);
    info!("Multicall: {:?}", config.multicall);
    info!("UniswapV2Pair: {:?}", config.uniswap_v2_pair);

    let db_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&opts.db_file)
        .unwrap();
    let state = serde_json::from_reader(&db_file).unwrap_or_default();

    let mut sentinel = Sentinel::new(
        config.balance_sheet,
        client,
        config.fy_tokens,
        config.hifi_flash_swap,
        config.multicall,
        opts.min_profit,
        state,
        config.uniswap_v2_pair,
    )
    .await?;

    sentinel.run(opts.db_file, opts.start_block).await?;

    Ok(())
}
