/// Here we have 'copies' of the events from `pallet_multisig`,
/// since need `Event` trait implemented for them. The fields of
/// `Timepoint` or `DispatchResult` are ignored as their values
/// are either non-reproducible or irrelevant.
use aleph_client::BlockNumber;
use codec::Decode;
use pallet_multisig::Timepoint;

use chain_support::{Event, EventKind};

use crate::{AccountId, CallHash};

#[derive(Clone, Debug, Decode, Default, Event)]
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

#[derive(Clone, Debug, Decode, Default, Event)]
#[pallet = "Multisig"]
pub struct MultisigApproval {
    approving: AccountId,
    #[event_ignore]
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigApproval {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigApproval {
            approving,
            multisig,
            call_hash,
            ..Default::default()
        }
    }
}

/// Placeholder for real `DispatchResult`. Without it,
/// we would have to add dependency on some substrate secondary crates.
#[derive(Debug, Decode, Clone)]
enum DispatchResult {
    Ok,
    Err,
}

impl Default for DispatchResult {
    fn default() -> Self {
        DispatchResult::Ok
    }
}

#[derive(Clone, Debug, Decode, Default, Event)]
#[pallet = "Multisig"]
pub struct MultisigExecuted {
    approving: AccountId,
    #[event_ignore]
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
    #[event_ignore]
    result: DispatchResult,
}

impl MultisigExecuted {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigExecuted {
            approving,
            multisig,
            call_hash,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Decode, Default, Event)]
#[pallet = "Multisig"]
pub struct MultisigCancelled {
    cancelling: AccountId,
    #[event_ignore]
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigCancelled {
    pub fn new(cancelling: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigCancelled {
            cancelling,
            multisig,
            call_hash,
            ..Default::default()
        }
    }
}
