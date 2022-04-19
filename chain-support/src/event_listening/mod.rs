use std::fmt::Debug;

use aleph_client::{account_from_keypair, substrate_api_client, KeyPair};
use codec::Decode;
use substrate_api_client::AccountId;
use thiserror::Error;

pub use single_event::SingleEventListener;

mod single_event;

/// Gathers all possible errors from this module.
#[derive(Debug, Error)]
pub enum ListeningError {
    #[error("⏳❌ Could not subscribe to events.")]
    CannotSubscribe,
    #[error("⏳❌ Expected event has not been emitted.")]
    NoEventSpotted,
}

/// Every event is identified by two coordinates: pallet name and event name,
/// e.g. `("Balances", "Transfer")`.
///
/// For these you have to look at Substrate code. Pallet's name should be in PascalCase.
/// The name of the particular event is the name of the corresponding variant of pallet's
/// `Event` enum. Usually, also in PascalCase.
pub type EventKind = (&'static str, &'static str);

/// This trait effectively represents two concepts: identification of an event type and
/// event filtering. When you are expecting some particular event, you have to provide
/// corresponding implementation of `Event`.
/// Stream of events will firstly be filtered by `kind()`. Events with identical
/// `EventKind` will be decoded (deserialized) to the corresponding struct and then
/// checked against `matches()` method.
///
/// For a reference, look below at `TransferEvent`.
pub trait Event: Clone + Debug + Decode + Send + 'static {
    /// Returns corresponding `EventKind`.
    fn kind(&self) -> EventKind;
    /// Decides whether `other` should be considered as the expected event,
    /// i.e. whether `self` and `other` are equivalent.
    fn matches(&self, other: &Self) -> bool;
}

/// Blanket implementation for events like `pallet_utility::BatchCompleted` with no fields.
#[derive(Clone, Debug, Decode)]
pub struct BareEvent {
    #[codec(skip)]
    kind: EventKind,
}

impl Event for BareEvent {
    fn kind(&self) -> EventKind {
        self.kind
    }

    fn matches(&self, _: &Self) -> bool {
        true
    }
}

impl From<EventKind> for BareEvent {
    fn from(kind: EventKind) -> Self {
        Self { kind }
    }
}

/// Representation of the `Transfer` event from pallet `Balances`. For details
/// look at `pallet_balances::Event::Transfer`.
#[derive(Clone, Debug, Decode, PartialEq, Eq)]
pub struct TransferEvent {
    from: AccountId,
    to: AccountId,
    amount: u128,
}

impl TransferEvent {
    pub fn new(from: &KeyPair, to: &AccountId, amount: u128) -> Self {
        TransferEvent {
            from: account_from_keypair(from),
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
