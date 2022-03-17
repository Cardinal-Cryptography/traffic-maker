use std::time::Duration;

use clap::Parser;

use chain_support::create_connection;
use scenario_transfer::SimpleTransferScenario;
use traffic::run_schedule;

use crate::config::Config;

mod config;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config: Config = Config::parse();
    let connection = create_connection(&config.node);

    // TODO: read from some config scenarios to launch together with parameters
    let scenarios = vec![SimpleTransferScenario::new(
        &connection,
        Duration::from_secs(5),
    )];

    run_schedule(scenarios).await;
}
