use serde::{Deserialize, Deserializer};
use std::time::Duration;

use chain_support::{create_connection, Connection};
use common::{Ident, Scenario, ScheduledScenario};
use parse_duration::parse;
use scenarios_transfer::{RandomTransfers, SimpleTransfer};

/// This struct combines both the execution environment (including hosts and chain address), as well
/// as the scenario configurations.
///
/// It should be read from `Timetable.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    environment: Environment,
    scenarios: Vec<ScenarioInstanceConfig>,
}

impl Config {
    pub fn construct_scenarios(&self) -> Vec<ScheduledScenario<Connection>> {
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
    SimpleTransfer(SimpleTransfer),
    RandomTransfers(RandomTransfers),
}

impl ScenarioConfig {
    fn to_scenario(&self) -> Box<dyn Scenario<Connection>> {
        use ScenarioConfig::*;

        match self.clone() {
            SimpleTransfer(s) => Box::new(s),
            RandomTransfers(s) => Box::new(s),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ScenarioInstanceConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    #[serde(rename = "scenario")]
    scenario_config: ScenarioConfig,
}

impl ScenarioInstanceConfig {
    pub fn construct_scenario(&self, connection: &Connection) -> ScheduledScenario<Connection> {
        ScheduledScenario::new(
            self.ident.clone(),
            self.interval,
            connection.clone(),
            self.scenario_config.to_scenario(),
        )
    }
}

/// Utility parser method for `Duration` struct.
fn parse_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}
