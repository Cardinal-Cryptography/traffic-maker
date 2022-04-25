// for `do_async!`
#![feature(fn_traits)]

use std::{fmt::Debug, time::Duration};

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

/// How long are we willing to wait for a particular event.
const EVENT_TIMEOUT: Duration = Duration::from_millis(3000);

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

/// Way to express desired multisig party size. The final value is obtainable
/// through consuming getter `get(upper_bound: usize)`. Use:
/// - `Small` for a random size between 2 and 5 members
/// - `Medium` for a random size between 6 and 14 members
/// - `Large` for a random size between 15 and `upper_bound`
/// - `Precise(s)` for a fixed size of `s` members
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

/// Way to express desired threshold. The final value is obtainable
/// through consuming getter `get(party_size: usize)`. Use:
/// - `Random`: for a random threshold between 2 and `party_size`
/// - `Precise(t)`: for a fixed threshold of `t`; beware that when `t` is less than 2
///   or greater than `party_size`, the getter will return suitable error `Err(_)`
///
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

/// Describes what should be done.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    InitiateWithHash,
    InitiateWithCall,
    ApproveWithHash,
    ApproveWithCall,
    Cancel,
}

impl Action {
    /// Checks whether the action carries a whole call (not just its hash).
    pub fn requires_call(&self) -> bool {
        matches!(self, InitiateWithCall | ApproveWithCall)
    }

    /// Checks whether action is `Cancel`.
    pub fn is_cancel(&self) -> bool {
        matches!(self, Cancel)
    }

    /// Checks whether this is an initiating action.
    pub fn is_initial(&self) -> bool {
        matches!(self, InitiateWithCall | InitiateWithHash)
    }

    /// Auxiliary function flattening result of result. Highly useful for results
    /// being returned from within `do_async!`.
    fn flatten<R, E: Into<anyhow::Error>>(result: Result<AnyResult<R>, E>) -> AnyResult<R> {
        match result {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.into()),
        }
    }

    /// Effectively performs the semantics behind `Action`. Calls corresponding methods
    /// of `party`.
    ///
    /// Unfortunately, `party` has to be passed by value here, as `MultisigParty` does
    /// not implement `Clone` trait, and passing a reference would require `'static` lifetime
    /// from the calling code.
    ///
    /// `sig_agg` should be `None` iff `self.is_initial()`.
    ///
    /// `should_finalize` is a flag indicating whether this approval should result in
    /// executing `call`.
    ///
    /// Note: if the action is `InitiateWithCall` or `ApproveWithCall`, `call` will be stored
    /// (unless this is the final approval). In other words, the pallet call flag `store_call`
    /// is always set to `true`.
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

        // The schema is common for all the cases. Firstly, we build an `event` we expect
        // to confirm the action. Then we call corresponding method from `MultisigParty`
        // wrapped with `with_event_listening`. This part is done asynchronously and
        // non-blocking due to `do_async!`.
        // When succeeded, we return new `SignatureAggregation` (in case of `Cancel`,
        // this will be unchanged `sig_agg`).
        //
        // As for similar code: since `event` object is different, we cannot easily (without
        // some ugly boxing) extract common schema.
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
                        #[allow(clippy::useless_conversion)]
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
                        // Here and below, we have to either put a variable or exchange
                        // bool literal / keyword with some expression.
                        // It originates from the fact that in macro definition we cannot
                        // distinguish between keywords (`true`) and values.
                        #[allow(clippy::useless_conversion)]
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
                        #[allow(clippy::useless_conversion)]
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

/// Describes how the signature aggregation should be carried:
/// - `Optimal`: everyone except the last one reports their approval only with hash
///   (this includes the initiator); only the last member passes call within their extrinsic;
/// - `Mess`: everyone randomly reports approval only with hash or with a call; it is guaranteed
///   that at least one approval comes with the call
/// - `InAdvance`: the initiator saves the call; everyone else approves with only hash
///
/// Here by `last` we meant the member who realizes threshold.
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
