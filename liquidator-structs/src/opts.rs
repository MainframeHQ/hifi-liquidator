use std::path::PathBuf;

use ethers::types::{U256, U64};
use gumdrop::Options;

#[derive(Debug, Options, Clone)]
pub struct Opts {
    pub help: bool,

    #[options(help = "Path to json file with the contract addresses")]
    pub config: PathBuf,

    #[options(help = "Database file to be used for persistence", default = "data.json")]
    pub db_file: PathBuf,

    #[options(help = "Polling interval in milliseconds", default = "1000")]
    pub interval: u64,

    #[options(help = "Minimum desired profit per liquidation", default = "0")]
    pub min_profit: U256,

    #[options(help = "Block at which to begin monitoring")]
    pub start_block: Option<U64>,

    #[options(help = "Ethereum node endpoint (HTTP or WS)", default = "http://localhost:8545")]
    pub url: String,
}
