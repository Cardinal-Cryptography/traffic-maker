#![feature(fn_traits)]

use std::time::Duration;

use aleph_client::{try_send_xt, Connection, KeyPair};
use anyhow::Result;
use parse_duration::parse;
use serde::de::{Deserialize, Deserializer};
use substrate_api_client::{AccountId, GenericAddress, XtStatus};
use tokio::time::sleep;

use chain_support::{SingleEventListener, TransferEvent};
use common::ScenarioError;
pub use round_robin::{RoundRobin, RoundRobinConfig};
pub use simple_transfer::{SimpleTransfer, SimpleTransferConfig};

mod round_robin;
mod simple_transfer;

fn parse_interval<'de, D>(deserializer: D) -> core::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse(s).map_err(serde::de::Error::custom)
}

async fn loop_transfer(connection: &Connection, target: &AccountId, amount: u128) -> Result<()> {
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
) -> Result<()> {
    let connection = connection.clone().set_signer(source.clone());
    let expected_event = TransferEvent::new(source, target, amount);
    let sel = SingleEventListener::new(&connection, expected_event).await?;
    match loop_transfer(&connection, target, amount).await {
        Ok(_) => sel
            .expect_event(Duration::from_millis(1000))
            .await
            .map(|_| ()),
        Err(e) => {
            let _ = sel.kill().await;
            Err(e)
        }
    }
}
