use std::fs;

use clap::Parser;
use serde::Deserialize;

use crate::{
    cli_config::CliConfig,
    endowment::{perform_endowments, Endowment},
};

mod cli_config;
mod endowment;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    pub endowments: Vec<Endowment>,
}

#[tokio::main]
async fn main() {
    let cli_config: CliConfig = CliConfig::parse();

    let config_content = fs::read_to_string(cli_config.config_file.clone())
        .expect("Config file should exist and be readable");
    let config: Config = toml::from_str(&config_content).expect("Should deserialize");

    perform_endowments(&cli_config, &config.endowments).await;
}
