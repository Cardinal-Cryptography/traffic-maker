use aleph_client::substrate_api_client::{AccountId, Balance};
use codec::Decode;

use chain_support::Event;

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Vesting"]
pub struct VestingUpdated {
    account: AccountId,
    unvested: Balance,
}

impl VestingUpdated {
    pub fn new(account: AccountId, unvested: Balance) -> Self {
        Self { account, unvested }
    }
}
