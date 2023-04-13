// for `do_async!`
#![feature(fn_traits)]

use std::{fmt::Debug, time::Duration};

use aleph_client::{
    account_from_keypair,
    pallets::multisig::{
        compute_call_hash, Call, Context, ContextAfterUse, MultisigContextualApi, MultisigParty,
        Ongoing, DEFAULT_MAX_WEIGHT,
    },
    KeyPair, SignedConnection, TxStatus,
    TxStatus::Finalized,
};
use anyhow::Result as AnyResult;
use codec::Encode;
pub use multisig::Multisig;
use rand::{random, thread_rng, Rng};
use serde::Deserialize;
use thiserror::Error;
use Action::*;
use Strategy::*;

mod multisig;

/// Gathers all possible errors from this module.
#[derive(Debug, Error)]
pub enum MultisigError {
    #[error("👪❌ Provided threshold value is too high.")]
    ThresholdTooHigh,
    #[error("👪❌ Threshold should be at least 2.")]
    ThresholdTooLow,
    #[error("👪❌ Party size should be less than {0}.")]
    SizeTooHigh(usize),
    #[error("👪❌ Action requires valid context parameter.")]
    MissingContext,
    #[error("👪❌ Signature aggregation has been closed or finalized unexpectedly.")]
    UnexpectedAggregationClosing,
}

/// Way to express desired multisig party size. The final value is obtainable through consuming
/// getter `get(upper_bound: usize)`.
///
/// Use:
/// - `Small` for a random size between 2 and 5 members
/// - `Medium` for a random size between 6 and 14 members
/// - `Large` for a random size between 15 and `upper_bound`
/// - `Precise(s)` for a fixed size of `s` members; `s` should not be greater than `upper_bound`
#[derive(Clone, Debug, Deserialize)]
pub enum PartySize {
    Small,
    Medium,
    Large,
    Precise(usize),
}

impl PartySize {
    pub fn get(self, upper_bound: usize) -> AnyResult<usize> {
        match self {
            PartySize::Small => Ok(thread_rng().gen_range(2..6)),
            PartySize::Medium => Ok(thread_rng().gen_range(6..15)),
            PartySize::Large => Ok(thread_rng().gen_range(15..=upper_bound)),
            PartySize::Precise(size) => {
                if size <= upper_bound {
                    Ok(size)
                } else {
                    Err(MultisigError::SizeTooHigh(upper_bound).into())
                }
            }
        }
    }
}

/// Way to express desired threshold. The final value is obtainable through consuming getter
/// `get(party_size: usize)`.
///
/// Use:
/// - `Random`: for a random threshold between 2 and `party_size`
/// - `Precise(t)`: for a fixed threshold of `t`; beware that when `t` is less than 2 or greater
///   than `party_size`, the getter will return suitable error `Err(_)`
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

    /// Performs the semantics behind `Action`. Calls corresponding methods of
    /// `MultisigContextualApi`.
    ///
    /// `context` should be `None` iff `self.is_initial()`.
    ///
    /// `should_finalize` is a flag indicating whether this approval should result in executing
    /// `call`.
    pub async fn perform(
        &self,
        connection: &SignedConnection,
        party: &MultisigParty,
        call: Call,
        should_finalize: bool,
        context: Option<Context<Ongoing>>,
    ) -> AnyResult<ContextAfterUse> {
        if !self.is_initial() && context.is_none() {
            return Err(MultisigError::MissingContext.into());
        }
        let call_hash = compute_call_hash(&call);

        match self {
            InitiateWithHash => {
                let (_, context) =
                    connection.initiate(party, &DEFAULT_MAX_WEIGHT, call_hash, Finalized)?;
                Ok(ContextAfterUse::Ongoing(context))
            }
            InitiateWithCall => {
                let (_, context) =
                    connection.initiate_with_call(party, &DEFAULT_MAX_WEIGHT, call, Finalized)?;
                Ok(ContextAfterUse::Ongoing(context))
            }
            // ApproveWithHash if should_finalize => {
            //     let (_, context) = connection.approve(context.unwrap(), Finalized)?;
            //     match context {
            //         ContextAfterUse::Ongoing(_) => Err(MultisigError::)
            //     }
            // }
            // ApproveWithCall if should_finalize => {
            //     let event = MultisigExecutedEvent::from_relevant_fields(
            //         caller,
            //         party.get_account(),
            //         call_hash,
            //     );
            // }
            ApproveWithHash => {
                let (_, context) = connection.approve(context.unwrap(), Finalized)?;
                match context {
                    ContextAfterUse::Ongoing(_) => Ok(context),
                    ContextAfterUse::Closed(_) => {
                        Err(MultisigError::UnexpectedAggregationClosing.into())
                    }
                }
            }
            ApproveWithCall => {
                let (_, context) =
                    connection.approve_with_call(context.unwrap(), Some(call), Finalized)?;
                match context {
                    ContextAfterUse::Ongoing(_) => Ok(context),
                    ContextAfterUse::Closed(_) => {
                        Err(MultisigError::UnexpectedAggregationClosing.into())
                    }
                }
            }
            Cancel => {
                let (_, context) = connection.cancel(context.unwrap(), Finalized)?;
                Ok(ContextAfterUse::Closed(context))
            }
        }
    }
}

/// Describes how the signature aggregation should be carried out:
/// - `Optimal`: everyone except the last one reports their approval only with hash (this includes
///   the initiator); only the last member passes call within their extrinsic;
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
            Mess => InitiateWithCall,
        }
    }

    fn middle_action(&self) -> Action {
        match self {
            Optimal | InAdvance => ApproveWithHash,
            Mess if random() => ApproveWithHash,
            Mess => ApproveWithCall,
        }
    }

    fn final_action(&self, call_submitted: bool) -> Action {
        match self {
            Optimal => ApproveWithCall,
            InAdvance => ApproveWithHash,
            Mess if !call_submitted || random() => ApproveWithCall,
            Mess => ApproveWithHash,
        }
    }
}
