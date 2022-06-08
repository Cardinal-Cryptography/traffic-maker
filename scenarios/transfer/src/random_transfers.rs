use std::{collections::HashMap, time::Duration};

use aleph_client::{
    get_free_balance, substrate_api_client, try_send_xt, AnyConnection, Connection, KeyPair,
    SignedConnection,
};
use anyhow::Result as AnyResult;
use codec::{Compact, Decode};
use rand::{prelude::IteratorRandom, thread_rng, Rng};
use serde::Deserialize;
use substrate_api_client::{
    compose_call, compose_extrinsic, AccountId, GenericAddress, Pair, XtStatus,
};

use chain_support::{do_async, keypair_derived_from_seed, with_event_listening, Event};
use common::{Scenario, ScenarioError, ScenarioLogging};

use crate::try_transfer;

/// We operate on an account pool based on this seed. The final seeds will have
/// a form of `RANDOM_TRANSFER_SEED{i: usize}`.
const RANDOM_TRANSFER_SEED: &str = "//RandomTransfer";

/// We expect that there are as many endowed accounts (of seed phrases: `RANDOM_TRANSFER_SEED{i}`,
/// where `i` is from 0 to this value (exclusively)).
const AVAILABLE_ACCOUNTS: usize = 100;

/// Returns keypair of the common account with index `idx`.
fn compute_keypair(idx: usize) -> KeyPair {
    keypair_derived_from_seed(format!("{}{}", RANDOM_TRANSFER_SEED, idx))
}

#[derive(Debug, Clone, Event, Decode)]
#[pallet = "Utility"]
struct BatchCompleted;

/// Describes which type of traffic is intended. Variants are pretty self-explanatory.
#[derive(Clone, Debug, Deserialize)]
pub enum Direction {
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Describes whether transfers should be submitted as independent extrinsics
/// or in a batch.
#[derive(Clone, Debug, Deserialize)]
pub enum Granularity {
    OneByOne,
    Batched,
}

/// Scenario making traffic through random transfers within the account pool.
///
/// Its specific behavior depends on `direction`:
/// - `OneToMany`: one account is randomly chosen as a sender, other `transfers`
///   accounts are chosen as receivers; then sender transfers `transfer_fraction` of
///   their balances to every receiver
/// - `ManyToOne`: one account is randomly chosen as a receiver, other `transfers`
///   accounts are chosen as senders; then each sender sends `transfer_fraction` of
///   their balances to the receiver
/// - `ManyToMany`: `transfers` random pairs are chosen as (sender, receiver); then every
///   sender sends `transfer_fraction` of their balances to their corresponding receiver
///
/// Depending on `granularity`, transfers are submitted sequentially or in a batch.
#[derive(Clone, Debug, Deserialize)]
pub struct RandomTransfers {
    /// What type of traffic should be made.
    direction: Direction,
    /// How to submit extrinsics.
    granularity: Granularity,
    /// How many transfers should be performed during a single run.
    /// This translates to different settings, depending on the scenario.
    /// E.g. in `OneToMany`, `transfers` will determine how many receivers
    /// are there.
    transfers: usize,
    /// To avoid exhausting one's balances, senders in these scenarios
    /// transfer a constant fraction of their balances. `transfer_fraction`
    /// describes this part in thousandths (passing e.g. 5 will result in
    /// sending 0.5% of available funds).
    transfer_fraction: u16,
}

/// Represents a single sender-receiver pair.
#[derive(Clone)]
struct TransferPair {
    sender: KeyPair,
    sender_id: usize,
    receiver: AccountId,
    receiver_id: usize,
}

impl RandomTransfers {
    /// Returns an iterator over all possible (sender, receiver) pairs
    /// corresponding to `self.direction`.
    fn generate_pairs(&self) -> impl Iterator<Item = (usize, usize)> {
        let range = 0..AVAILABLE_ACCOUNTS;
        // Have to use ugly `Box` with annotation because arms return different `Map<_>` objects
        // (different closures => different types).
        let unfiltered: Box<dyn Iterator<Item = (usize, usize)>> = match self.direction {
            Direction::OneToMany => {
                let sender = thread_rng().gen_range(range.clone());
                Box::new(range.map(move |receiver| (sender, receiver)))
            }
            Direction::ManyToOne => {
                let receiver = thread_rng().gen_range(range.clone());
                Box::new(range.map(move |sender| (sender, receiver)))
            }
            Direction::ManyToMany => Box::new(
                range
                    .clone()
                    .flat_map(move |r| range.clone().map(move |s| (s, r))),
            ),
        };
        unfiltered.filter(|(s, r)| s != r)
    }

    /// Returns a vector of `self.transfers` random (sender, receiver) pairs corresponding
    /// to `self.direction`.
    fn designate_pairs(&self) -> Vec<TransferPair> {
        let possibilities = self.generate_pairs();
        let mut generator = thread_rng();
        let index_pairs = possibilities.choose_multiple(&mut generator, self.transfers);

        let keypairs: HashMap<usize, KeyPair> = index_pairs
            .iter()
            .flat_map(|pair| [pair.0, pair.1])
            .map(|i| (i, compute_keypair(i)))
            .collect();

        index_pairs
            .into_iter()
            .map(|(s, r)| TransferPair {
                sender: keypairs[&s].clone(),
                sender_id: s,
                receiver: AccountId::from(keypairs[&r].public()),
                receiver_id: r,
            })
            .collect()
    }

    /// Computes estimated fraction of `balances` (`self.transfer_fraction`‰).
    fn balances_fraction(&self, balances: u128) -> u128 {
        balances
            .saturating_div(1000)
            .saturating_mul(self.transfer_fraction as u128)
    }

    /// Computes how much money should be transferred from `sender`.
    async fn compute_transfer_value(
        &self,
        connection: &Connection,
        sender: &KeyPair,
    ) -> AnyResult<u128> {
        let sender_account = AccountId::from(sender.public());
        let sender_balances = do_async!(get_free_balance, &connection, &sender_account)?;
        Ok(self.balances_fraction(sender_balances))
    }

    async fn send_sequentially(
        &self,
        connection: &Connection,
        pairs: Vec<TransferPair>,
        logger: &ScenarioLogging,
    ) -> AnyResult<()> {
        for (idx, transfer_pair) in pairs.into_iter().enumerate() {
            let TransferPair {
                sender,
                sender_id,
                receiver,
                receiver_id,
            } = transfer_pair;

            logger.debug(format!(
                "Transferring money from #{} to #{}.",
                sender_id, receiver_id
            ));

            let transfer_value = self.compute_transfer_value(connection, &sender).await?;
            let transfer_result =
                try_transfer(connection, &sender, &receiver, transfer_value).await;
            logger.log_result(transfer_result)?;

            logger.debug(format!(
                "Completed {}/{} transfers.",
                idx + 1,
                self.transfers
            ));
        }
        Ok(())
    }

    async fn send_in_batch(
        &self,
        connection: &Connection,
        pairs: Vec<TransferPair>,
        logger: &ScenarioLogging,
    ) -> AnyResult<()> {
        // `xts` is built in good old imperative way, because it requires async, fallible call
        // for computing transfer value, which is not so nice to be used within `map()`.
        let mut xts = Vec::new();
        for transfer_pair in pairs.clone() {
            let TransferPair {
                sender,
                sender_id,
                receiver,
                receiver_id,
            } = transfer_pair;

            logger.debug(format!(
                "Preparing transfer from #{} to #{}.",
                sender_id, receiver_id
            ));

            let transfer_value = self.compute_transfer_value(connection, &sender).await?;
            let metadata = SignedConnection::from_any_connection(connection, sender)
                .as_connection()
                .metadata;
            xts.push(compose_call!(
                metadata,
                "Balances",
                "transfer",
                GenericAddress::Id(receiver),
                Compact(transfer_value)
            ));
        }

        // `self.connection` may not be signed, but somebody has to pay for submitting
        let connection = SignedConnection::from_any_connection(connection, pairs[0].sender.clone());
        let xt = compose_extrinsic!(connection.as_connection(), "Utility", "batch", xts);

        let batch_result = with_event_listening(
            &connection,
            BatchCompleted {},
            Duration::from_secs(1),
            async {
                try_send_xt(
                    &connection,
                    xt,
                    Some("Sending transfers in batch"),
                    XtStatus::Finalized,
                )
                .map_err(|_| ScenarioError::CannotSendExtrinsic.into())
            },
        )
        .await;

        logger.log_result(batch_result)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario<Connection> for RandomTransfers {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        let pairs = self.designate_pairs();
        match self.granularity {
            Granularity::OneByOne => self.send_sequentially(connection, pairs, logger).await,
            Granularity::Batched => self.send_in_batch(connection, pairs, logger).await,
        }?;

        logger.info("Scenario finished successfully");
        Ok(())
    }
}
