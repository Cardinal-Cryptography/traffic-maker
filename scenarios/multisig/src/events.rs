use aleph_client::BlockNumber;
use codec::Decode;
use pallet_multisig::Timepoint;

use chain_support::{Event, EventKind};

use crate::{AccountId, CallHash};

#[derive(Debug, Decode, Clone, Eq, PartialEq)]
pub struct NewMultisigEvent {
    approving: AccountId,
    multisig: AccountId,
    call_hash: CallHash,
}

impl NewMultisigEvent {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        NewMultisigEvent {
            approving,
            multisig,
            call_hash,
        }
    }
}

impl Event for NewMultisigEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "NewMultisig")
    }

    fn matches(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[allow(dead_code)]
#[derive(Debug, Decode, Clone)]
pub struct MultisigApprovalEvent {
    approving: AccountId,
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigApprovalEvent {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigApprovalEvent {
            approving,
            multisig,
            timepoint: Default::default(),
            call_hash,
        }
    }
}

impl Event for MultisigApprovalEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "MultisigApproval")
    }

    fn matches(&self, other: &Self) -> bool {
        self.approving == other.approving
            && self.multisig == other.multisig
            && self.call_hash == other.call_hash
    }
}

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

#[allow(dead_code)]
#[derive(Debug, Decode, Clone)]
pub struct MultisigExecutedEvent {
    approving: AccountId,
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
    result: DispatchResult,
}

impl MultisigExecutedEvent {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigExecutedEvent {
            approving,
            timepoint: Default::default(),
            multisig,
            call_hash,
            result: Default::default(),
        }
    }
}

impl Event for MultisigExecutedEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "MultisigExecuted")
    }

    fn matches(&self, other: &Self) -> bool {
        self.approving == other.approving
            && self.multisig == other.multisig
            && self.call_hash == other.call_hash
    }
}

#[allow(dead_code)]
#[derive(Debug, Decode, Clone)]
pub struct MultisigCancelledEvent {
    cancelling: AccountId,
    timepoint: Timepoint<BlockNumber>,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigCancelledEvent {
    pub fn new(cancelling: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigCancelledEvent {
            cancelling,
            timepoint: Default::default(),
            multisig,
            call_hash,
        }
    }
}

impl Event for MultisigCancelledEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "MultisigCancelled")
    }

    fn matches(&self, other: &Self) -> bool {
        self.cancelling == other.cancelling
            && self.multisig == other.multisig
            && self.call_hash == other.call_hash
    }
}
