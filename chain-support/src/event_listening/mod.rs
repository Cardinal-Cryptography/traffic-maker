use std::fmt::Debug;

use aleph_client::substrate_api_client;
use codec::Decode;
use substrate_api_client::AccountId;
use thiserror::Error;

pub use event_derive::Event;
pub use single_event::{with_event_listening, SingleEventListener};

#[cfg(test)]
mod macro_tests;
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
/// For these you have to look at Substrate code. Pallet's name should be in PascalCase. The name
/// of the particular event is the name of the corresponding variant of pallet's `Event` enum.
/// Usually, also in PascalCase.
pub type EventKind = (&'static str, &'static str);

/// This trait effectively represents two concepts: identification of an event type and event
/// filtering. When you are expecting some particular event, you have to provide corresponding
/// implementation of `Event`.
///
/// Stream of events will firstly be filtered by `kind()`. Events with identical `EventKind` will be
/// decoded (deserialized) to the corresponding struct and then checked against `matches()` method.
///
/// For a reference, look below at `Transfer`.
pub trait Event: Clone + Debug + Decode + Send + 'static {
    /// Returns corresponding `EventKind`.
    fn kind(&self) -> EventKind;
    /// Decides whether `other` should be considered as the expected event, i.e. whether `self` and
    /// `other` are equivalent.
    fn matches(&self, other: &Self) -> bool;
}

/// Representation of the `Transfer` event from pallet `Balances`.
///
/// For details look at `pallet_balances::Event::Transfer`.
#[derive(Clone, Debug, Event, Decode, PartialEq, Eq)]
#[pallet = "Balances"]
pub struct Transfer {
    from: AccountId,
    to: AccountId,
    amount: u128,
}
