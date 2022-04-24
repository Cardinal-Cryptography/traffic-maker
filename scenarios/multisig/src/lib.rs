// for `do_async!`
#![feature(fn_traits)]

use std::time::Duration;

use aleph_client::{
    account_from_keypair, compute_call_hash,
    substrate_api_client::{AccountId, UncheckedExtrinsicV4},
    Connection, KeyPair, MultisigParty, SignatureAggregation,
};
use anyhow::Result as AnyResult;
use codec::Encode;
use rand::{random, thread_rng, Rng};
use serde::Deserialize;
use thiserror::Error;

use chain_support::{do_async, with_event_listening};
pub use multisig::{Multisig, MultisigConfig};
use Action::*;
use Strategy::*;

use crate::events::{
    MultisigApprovalEvent, MultisigCancelledEvent, MultisigExecutedEvent, NewMultisigEvent,
};

mod events;
mod multisig;

const EVENT_TIMEOUT: Duration = Duration::from_millis(1500);
type CallHash = [u8; 32];

/// Gathers all possible errors from this module.
#[derive(Debug, Error)]
pub enum MultisigError {
    #[error("👪❌ Provided threshold value is too high.")]
    ThresholdTooHigh,
    #[error("👪❌ Threshold should be at least 2.")]
    ThresholdTooLow,
    #[error("👪❌ Aggregation is no longer valid.")]
    InvalidAggregation,
}

#[derive(Clone, Debug, Deserialize)]
pub enum PartySize {
    Small,
    Medium,
    Large,
    Precise(usize),
}

impl PartySize {
    pub fn get(self, upper_bound: usize) -> usize {
        match self {
            PartySize::Small => thread_rng().gen_range(2..6),
            PartySize::Medium => thread_rng().gen_range(6..15),
            PartySize::Large => thread_rng().gen_range(15..=upper_bound),
            PartySize::Precise(size) => size,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum Threshold {
    Random,
    Precise(usize),
}

impl Threshold {
    pub fn get(self, party_size: usize) -> AnyResult<usize> {
        match self {
            Threshold::Random => Ok(thread_rng().gen_range(2..=party_size)),
            Threshold::Precise(threshold) if threshold > party_size => {
                Err(MultisigError::ThresholdTooHigh.into())
            }
            Threshold::Precise(threshold) if threshold < 2 => {
                Err(MultisigError::ThresholdTooLow.into())
            }
            Threshold::Precise(threshold) => Ok(threshold),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    InitiateWithHash,
    InitiateWithCall,
    ApproveWithHash,
    ApproveWithCall,
    Cancel,
}

impl Action {
    pub fn requires_call(&self) -> bool {
        matches!(self, InitiateWithCall | ApproveWithCall)
    }

    pub fn is_cancel(&self) -> bool {
        matches!(self, Cancel)
    }

    pub fn is_initial(&self) -> bool {
        matches!(self, InitiateWithCall | InitiateWithHash)
    }

    fn flatten<R, E: Into<anyhow::Error>>(result: Result<AnyResult<R>, E>) -> AnyResult<R> {
        match result {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn perform<CallDetails: Encode + Clone + Send + 'static>(
        &self,
        connection: &Connection,
        party: MultisigParty,
        sig_agg: Option<SignatureAggregation>,
        call: UncheckedExtrinsicV4<CallDetails>,
        caller: &KeyPair,
        should_finalize: bool,
    ) -> AnyResult<Option<SignatureAggregation>> {
        if !self.is_initial() && sig_agg.is_none() {
            return Err(MultisigError::InvalidAggregation.into());
        }

        let connection = connection.clone().set_signer(caller.clone());
        let caller = account_from_keypair(caller);
        let caller_idx = party.get_member_index(caller.clone())?;
        let call_hash = compute_call_hash(&call);

        match self {
            InitiateWithHash => {
                let event = NewMultisigEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::initiate_aggregation_with_hash,
                        party,
                        &connection,
                        call_hash,
                        caller_idx
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            InitiateWithCall => {
                let event = NewMultisigEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::initiate_aggregation_with_call,
                        party,
                        &connection,
                        call,
                        true.into(),
                        caller_idx
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            ApproveWithHash if should_finalize => {
                let event = MultisigExecutedEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::approve,
                        party,
                        &connection,
                        caller_idx,
                        sig_agg.unwrap()
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            ApproveWithCall if should_finalize => {
                let event = MultisigExecutedEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::approve_with_call,
                        party,
                        &connection,
                        caller_idx,
                        sig_agg.unwrap(),
                        call,
                        true.into()
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            ApproveWithHash => {
                let event = MultisigApprovalEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::approve,
                        party,
                        &connection,
                        caller_idx,
                        sig_agg.unwrap()
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            ApproveWithCall => {
                let event = MultisigApprovalEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::approve_with_call,
                        party,
                        &connection,
                        caller_idx,
                        sig_agg.unwrap(),
                        call,
                        true.into()
                    ))
                })
                .await
                .map(|(sig_agg, _event)| Some(sig_agg))
            }
            Cancel => {
                let event = MultisigCancelledEvent::new(caller, party.get_account(), call_hash);
                with_event_listening(&connection, event, EVENT_TIMEOUT, async {
                    Self::flatten(do_async!(
                        MultisigParty::cancel,
                        party,
                        &connection,
                        caller_idx,
                        sig_agg.unwrap()
                    ))
                })
                .await
                .map(|_| None)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum Strategy {
    Optimal,
    Mess,
    InAdvance,
}

impl Strategy {
    fn initial_action(&self) -> Action {
        match self {
            Optimal => InitiateWithHash,
            InAdvance => InitiateWithCall,
            Mess if random() => InitiateWithHash,
            _ => InitiateWithCall,
        }
    }

    fn middle_action(&self) -> Action {
        match self {
            Optimal | InAdvance => ApproveWithHash,
            Mess if random() => ApproveWithHash,
            _ => ApproveWithCall,
        }
    }

    fn final_action(&self, call_submitted: bool) -> Action {
        match self {
            Optimal => ApproveWithCall,
            InAdvance => ApproveWithHash,
            Mess if !call_submitted || random() => ApproveWithCall,
            _ => ApproveWithHash,
        }
    }
}
