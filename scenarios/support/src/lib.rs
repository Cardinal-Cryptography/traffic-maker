use std::time::Duration;

use parse_duration::parse;
use serde::{Deserialize, Deserializer};

pub fn parse_interval<'de, D>(deserializer: D) -> core::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}
