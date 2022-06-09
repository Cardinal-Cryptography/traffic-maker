#![feature(fn_traits)]

use std::time::Duration;

use aleph_client::{
    account_from_keypair, substrate_api_client, try_send_xt, AnyConnection, KeyPair,
    SignedConnection,
};
use anyhow::Result as AnyResult;
use substrate_api_client::{AccountId, GenericAddress, XtStatus};
use tokio::time::sleep;

use chain_support::{with_event_listening, TransferEvent};
use common::ScenarioError;
pub use random_transfers::{Direction, Granularity, RandomTransfers};
pub use round_robin::RoundRobin;
pub use simple_transfer::SimpleTransfer;

mod random_transfers;
mod round_robin;
mod simple_transfer;

async fn loop_transfer(
    connection: &SignedConnection,
    target: &AccountId,
    amount: u128,
) -> AnyResult<()> {
    for _ in 0..5 {
        let xt = connection
            .as_connection()
            .balance_transfer(GenericAddress::Id(target.clone()), amount);
        if try_send_xt(connection, xt, Some("transfer"), XtStatus::Finalized).is_ok() {
            return Ok(());
        }

        sleep(Duration::from_millis(500)).await;
    }
    Err(ScenarioError::CannotSendExtrinsic.into())
}

pub async fn try_transfer<C: AnyConnection>(
    connection: &C,
    source: &KeyPair,
    target: &AccountId,
    amount: u128,
) -> AnyResult<()> {
    let connection = SignedConnection::from_any_connection(connection, source.clone());
    let expected_event =
        TransferEvent::from_relevant_fields(account_from_keypair(source), target.clone(), amount);

    with_event_listening(&connection, expected_event, Duration::from_secs(1), async {
        loop_transfer(&connection, target, amount).await
    })
    .await
    .map(|_| ())
}
