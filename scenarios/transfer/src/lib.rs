#![feature(fn_traits)]

use std::time::Duration;

use aleph_client::{substrate_api_client, try_send_xt, Connection, KeyPair};
use anyhow::Result as AnyResult;
use parse_duration::parse;
use serde::de::{Deserialize, Deserializer};
use substrate_api_client::{AccountId, GenericAddress, XtStatus};
use tokio::time::sleep;

use chain_support::{SingleEventListener, TransferEvent};
use common::ScenarioError;
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

async fn loop_transfer(connection: &Connection, target: &AccountId, amount: u128) -> AnyResult<()> {
    for _ in 0..5 {
        let xt = connection.balance_transfer(GenericAddress::Id(target.clone()), amount);
        if try_send_xt(connection, xt, Some("transfer"), XtStatus::Finalized).is_ok() {
            return Ok(());
        }

        sleep(Duration::from_millis(500)).await;
    }
    Err(ScenarioError::CannotSendExtrinsic.into())
}

pub async fn try_transfer(
    connection: &Connection,
    source: &KeyPair,
    target: &AccountId,
    amount: u128,
) -> AnyResult<()> {
    let connection = connection.clone().set_signer(source.clone());
    let expected_event = TransferEvent::new(source, target, amount);
    let sel = SingleEventListener::new(&connection, expected_event).await?;
    let transfer_result = loop_transfer(&connection, target, amount).await;
    sel.expect_event_if_ok(Duration::from_secs(1), transfer_result)
        .await
        .map(|_| ())
}
