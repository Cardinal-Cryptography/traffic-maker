use aleph_client::KeyPair;
use std::fmt::Debug;

use codec::Decode;
use substrate_api_client::{AccountId, Pair};
use thiserror::Error;

pub use single_event::SingleEventListener;

mod single_event;

#[derive(Debug, Error)]
pub enum ListeningError {
    #[error("⏳❌ Could not subscribe to events.")]
    CannotSubscribe,
    #[error("⏳❌ Expected event has not been emitted.")]
    NoEventSpotted,
}

pub type EventKind = (&'static str, &'static str);

pub trait Event: Clone + Debug + Decode + Send + 'static {
    fn kind(&self) -> EventKind;
    fn matches(&self, other: &Self) -> bool;
}

#[derive(Clone, Debug, Decode, PartialEq, Eq)]
pub struct TransferEvent {
    from: AccountId,
    to: AccountId,
    amount: u128,
}

impl TransferEvent {
    pub fn new(from: &KeyPair, to: &AccountId, amount: u128) -> Self {
        TransferEvent {
            from: AccountId::from(from.public()),
            to: to.clone(),
            amount,
        }
    }
}

impl Event for TransferEvent {
    fn kind(&self) -> EventKind {
        ("Balances", "Transfer")
    }

    fn matches(&self, other: &Self) -> bool {
        self.eq(other)
    }
}
