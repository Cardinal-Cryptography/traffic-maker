use serde::Deserialize;

use chain_support::{create_connection, Connection};
use common::Scenario;
use scenarios_multisig::{Multisig, MultisigConfig};
use scenarios_transfer::{
    RandomTransfers, RandomTransfersConfig, RoundRobin, RoundRobinConfig, SimpleTransfer,
    SimpleTransferConfig,
};

/// This struct combines both the execution environment (including hosts and chain address), as well
/// as the scenario configurations.
///
/// It should be read from `Timetable.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    environment: Environment,
    scenarios: Vec<ScenarioConfig>,
}

impl Config {
    pub fn construct_scenarios(&self) -> Vec<Box<dyn Scenario>> {
        let connection = self.environment.get_new_connection();
        self.scenarios
            .iter()
            .map(|sc| sc.construct_scenario(&connection))
            .collect()
    }

    pub fn get_expose_host(&self) -> &str {
        self.environment.get_expose_host()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Environment {
    /// WS endpoint address of the node to connect to
    node: String,

    /// Where to expose stats
    expose_host: String,
}

impl Environment {
    pub fn get_new_connection(&self) -> Connection {
        create_connection(&*self.node)
    }

    pub fn get_expose_host(&self) -> &str {
        self.expose_host.as_str()
    }
}

/// All implemented scenarios should be included here.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
enum ScenarioConfig {
    SimpleTransfer(SimpleTransferConfig),
    RoundRobin(RoundRobinConfig),
    RandomTransfers(RandomTransfersConfig),
    Multisig(MultisigConfig),
}

impl ScenarioConfig {
    pub fn construct_scenario(&self, connection: &Connection) -> Box<dyn Scenario> {
        match self {
            ScenarioConfig::SimpleTransfer(config) => {
                Box::new(SimpleTransfer::new(connection, config.clone()))
            }
            ScenarioConfig::RoundRobin(config) => {
                Box::new(RoundRobin::new(connection, config.clone()))
            }
            ScenarioConfig::RandomTransfers(config) => {
                Box::new(RandomTransfers::new(connection, config.clone()))
            }
            ScenarioConfig::Multisig(config) => Box::new(Multisig::new(connection, config.clone())),
        }
    }
}
