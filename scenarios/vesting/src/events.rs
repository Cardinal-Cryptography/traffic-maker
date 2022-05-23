//! Here we have 'copies' of the events from `pallet_vesting`, since we need `Event` trait
//! implemented for them.
use aleph_client::substrate_api_client::{AccountId, Balance};
use codec::Decode;

use chain_support::Event;

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Vesting"]
pub struct VestingUpdated {
    account: AccountId,
    unvested: Balance,
}
