use parse_duration::parse;
use std::time::Duration;

use serde::Deserialize;

use chain_support::{create_connection, Connection, Protocol};
use common::{Ident, Scenario};
use scenario_transfer::SimpleTransferScenario;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    environment: Environment,
    scenarios: Vec<ScenarioConfig>,
}

impl Config {
    pub fn get_scenarios(&self) -> Vec<impl Scenario> {
        let connection = self.environment.new_connection();
        self.scenarios
            .iter()
            .map(|sc| sc.get_scenario(&connection))
            .collect()
    }

    pub fn get_expose_host(&self) -> &str {
        self.environment.expose_host()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Environment {
    /// WS endpoint address of the node to connect to
    node: String,

    /// Protocol to be used for connecting to node (`ws` or `wss`).
    protocol: String,

    /// Where to expose stats
    expose_host: String,
}

impl Environment {
    pub fn new_connection(&self) -> Connection {
        let protocol = match self.protocol.as_str() {
            "wss" => Protocol::WSS,
            _ => Protocol::WS,
        };
        create_connection(&*self.node, protocol)
    }

    pub fn expose_host(&self) -> &str {
        self.expose_host.as_str()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ScenarioConfig {
    /// Identifier corresponding to the struct name.
    kind: String,

    /// Unique identifier.
    ident: Ident,

    /// How often should this scenario be launched.
    interval: String,
}

impl ScenarioConfig {
    pub fn get_scenario(&self, connection: &Connection) -> impl Scenario {
        match self.kind.as_str() {
            "SimpleTransfer" => {
                SimpleTransferScenario::new(connection, self.ident.clone(), self.interval())
            }
            _ => panic!("Unknown scenario"),
        }
    }

    fn interval(&self) -> Duration {
        parse(self.interval.as_str()).expect("Interval should be parsable")
    }
}
