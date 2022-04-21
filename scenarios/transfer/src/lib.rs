#![feature(fn_traits)]

use std::time::Duration;

use parse_duration::parse;
use serde::de::{Deserialize, Deserializer};

pub use random_transfers::{Direction, Granularity, RandomTransfers, RandomTransfersConfig};
pub use round_robin::{RoundRobin, RoundRobinConfig};
pub use simple_transfer::{SimpleTransfer, SimpleTransferConfig};

mod random_transfers;
mod round_robin;
mod simple_transfer;

fn parse_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}
