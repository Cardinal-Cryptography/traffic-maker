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

impl NewMultisig {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        NewMultisig {
            approving,
            multisig,
            call_hash,
        }
    }
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

impl MultisigApproval {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigApproval {
            approving,
            _timepoint: Default::default(),
            multisig,
            call_hash,
        }
    }
}

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Multisig"]
pub struct MultisigExecuted {
    approving: AccountId,
    #[event_ignore]
    _timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
    #[event_ignore]
    _result: Result<(), sp_runtime::DispatchError>,
}

impl MultisigExecuted {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigExecuted {
            approving,
            _timepoint: Default::default(),
            multisig,
            call_hash,
            _result: Ok(()),
        }
    }
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

impl MultisigCancelled {
    pub fn new(cancelling: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigCancelled {
            cancelling,
            _timepoint: Default::default(),
            multisig,
            call_hash,
        }
    }
}
