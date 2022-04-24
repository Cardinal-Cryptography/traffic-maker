use codec::Decode;

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

#[derive(Debug, Decode, Clone, Eq, PartialEq)]
pub struct MultisigApprovalEvent {
    approving: AccountId,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigApprovalEvent {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigApprovalEvent {
            approving,
            multisig,
            call_hash,
        }
    }
}

impl Event for MultisigApprovalEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "MultisigApproval")
    }

    fn matches(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[derive(Debug, Decode, Clone, Eq, PartialEq)]
pub struct MultisigExecutedEvent {
    approving: AccountId,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigExecutedEvent {
    pub fn new(approving: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigExecutedEvent {
            approving,
            multisig,
            call_hash,
        }
    }
}

impl Event for MultisigExecutedEvent {
    fn kind(&self) -> EventKind {
        ("Multisig", "MultisigExecuted")
    }

    fn matches(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[derive(Debug, Decode, Clone, Eq, PartialEq)]
pub struct MultisigCancelledEvent {
    cancelling: AccountId,
    multisig: AccountId,
    call_hash: CallHash,
}

impl MultisigCancelledEvent {
    pub fn new(cancelling: AccountId, multisig: AccountId, call_hash: CallHash) -> Self {
        MultisigCancelledEvent {
            cancelling,
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
        self.eq(other)
    }
}
