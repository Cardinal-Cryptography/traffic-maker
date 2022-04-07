use parse_duration::parse;
use std::time::Duration;

use serde::Deserialize;

use chain_support::{create_connection, Connection};
use common::{Ident, Scenario};
use scenario_transfer::SimpleTransferScenario;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    environment: Environment,
    scenarios: Vec<ScenarioConfig>,
}

impl Config {
    pub fn create_scenarios(&self) -> Vec<impl Scenario> {
        let connection = self.environment.new_connection();
        self.scenarios
            .iter()
            .map(|sc| sc.create_scenario(&connection))
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
    pub fn new_connection(&self) -> Connection {
        create_connection(&*self.node)
    }

    pub fn get_expose_host(&self) -> &str {
        self.expose_host.as_str()
    }
}

/// All implemented scenarios should be included here.
#[derive(Debug, Copy, Clone, Deserialize)]
enum ScenarioKind {
    SimpleTransfer,
}

#[derive(Debug, Clone, Deserialize)]
struct ScenarioConfig {
    /// What kind of scenario should be run.
    kind: ScenarioKind,

    /// Unique identifier.
    ident: Ident,

    /// How often should this scenario be launched.
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
}

impl ScenarioConfig {
    pub fn create_scenario(&self, connection: &Connection) -> impl Scenario {
        #[allow(clippy::match_single_binding)]
        match self.kind {
            ScenarioKind::SimpleTransfer => {
                SimpleTransferScenario::new(connection, self.ident.clone(), self.interval)
            }
        }
    }
}

fn parse_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}
