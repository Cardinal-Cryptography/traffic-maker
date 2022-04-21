mod multisig;
use anyhow::Result as AnyResult;
use rand::{random, thread_rng, Rng};
use serde::Deserialize;
use thiserror::Error;

/// Gathers all possible errors from this module.
#[derive(Debug, Error)]
pub enum MultisigError {
    #[error("👪❌ Provided threshold value is too high.")]
    ThresholdTooHigh,
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
            Threshold::Precise(threshold) => {
                if threshold > party_size {
                    Err(MultisigError::ThresholdTooHigh.into())
                } else {
                    Ok(threshold)
                }
            }
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
}

#[derive(Clone, Debug, Deserialize)]
pub enum Strategy {
    Optimal,
    Mess,
    InAdvance,
}

use Action::*;
use Strategy::*;

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
            Mess if random() || !call_submitted => ApproveWithCall,
            _ => ApproveWithHash,
        }
    }
}
