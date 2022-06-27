use parse_duration::parse;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;

pub use scenario::{
    Scenario, ScenarioDetails, ScenarioError, ScenarioLogging, ScenarioLogs, ScenarioStatus,
    ScheduledScenario,
};

mod scenario;

/// A wrapper type for scenario identification.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ident(pub String);

impl From<String> for Ident {
    fn from(inner: String) -> Self {
        Ident(inner)
    }
}

impl From<&str> for Ident {
    fn from(inner: &str) -> Self {
        Ident(inner.to_string())
    }
}

/// Utility parser method for `Duration` struct.
pub fn parse_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}
