//! Here we have 'copies' of the events from `pallet_multisig`, since need `Event` trait implemented
//! for them. The fields of `Timepoint` or `DispatchResult` are ignored as their values are either
//! non-reproducible or irrelevant.

use aleph_client::BlockNumber;
use codec::Decode;
use pallet_multisig::Timepoint;

use chain_support::Event;

use crate::{AccountId, CallHash};

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Multisig"]
pub struct NewMultisig {
    approving: AccountId,
    multisig: AccountId,
    call_hash: CallHash,
}

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Multisig"]
pub struct MultisigApproval {
    approving: AccountId,
    #[event_ignore]
    _timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Multisig"]
pub struct MultisigExecuted {
    approving: AccountId,
    #[event_ignore]
    _timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
    #[event_ignore = "Ok(())"]
    _result: Result<(), sp_runtime::DispatchError>,
}

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Multisig"]
pub struct MultisigCancelled {
    cancelling: AccountId,
    #[event_ignore]
    _timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}
