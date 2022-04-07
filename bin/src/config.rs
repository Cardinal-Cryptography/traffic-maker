use serde::Deserialize;

use chain_support::{create_connection, Connection};
use common::Scenario;
use scenarios_transfer::{RoundRobin, RoundRobinProps, SimpleTransfer, SimpleTransferProps};

/// This structure should exactly correspond to `Timetable.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    environment: Environment,
    scenarios: Vec<ScenarioKind>,
}

impl Config {
    pub fn construct_scenarios(&self) -> Vec<impl Scenario> {
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
enum ScenarioKind {
    SimpleTransfer(SimpleTransferProps),
    RoundRobin(RoundRobinProps),
}

impl ScenarioKind {
    pub fn construct_scenario(&self, connection: &Connection) -> Box<dyn Scenario> {
        match self {
            ScenarioKind::SimpleTransfer(props) => {
                Box::new(SimpleTransfer::new(connection, props.clone()))
            }
            ScenarioKind::RoundRobin(props) => Box::new(RoundRobin::new(connection, props.clone())),
        }
    }
}
