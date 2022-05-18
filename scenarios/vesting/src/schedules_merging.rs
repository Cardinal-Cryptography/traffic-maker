use std::time::Duration;

use aleph_client::{
    account_from_keypair, get_schedules, merge_schedules,
    substrate_api_client::{AccountId, Balance},
    vested_transfer, BlockNumber, Connection, KeyPair, VestingSchedule,
};
use anyhow::{ensure, Result as AnyResult};
use rand::random;
use serde::Deserialize;
use thiserror::Error;

use chain_support::{do_async, keypair_derived_from_seed, with_event_listening};
use common::{Ident, Scenario, ScenarioLogging};
use scenarios_support::parse_interval;

use crate::events::VestingUpdated;

/// We operate on an account pool based on this seed. The final seeds will have a form of
/// `ACCOUNT_SEED{i: usize}`.
const ACCOUNT_SEED: &str = "//VestingSchedulesMerging";

/// We expect that there are as many endowed accounts (of seed phrases: `ACCOUNT_SEED{i}`, where
/// `i` is from 0 to this value (exclusively)).
const AVAILABLE_ACCOUNTS: usize = 50;

/// Returns keypair of the common account with index `idx`.
fn compute_keypair(idx: usize) -> KeyPair {
    keypair_derived_from_seed(format!("{}{}", ACCOUNT_SEED, idx))
}

/// Returns keypair of some random account from common pool with index `idx`.
fn get_random_keypair() -> KeyPair {
    compute_keypair(random::<usize>() % AVAILABLE_ACCOUNTS)
}

/// Possible errors from this module.
#[derive(Debug, Error)]
pub enum SchedulesMergingError {
    #[error("🦺❌ This scenario does not make sense when `MAX_VESTING_SCHEDULES` is less than 2.")]
    LimitTooLow,
    #[error("🦺❌ Couldn't reach `MAX_VESTING_SCHEDULES` for `{0:?}`.")]
    ReachingLimitFailure(AccountId),
    #[error("🦺❌ Account `{0:?}` has already `MAX_VESTING_SCHEDULES` active schedules.")]
    LimitAlreadyReached(AccountId),
    #[error(
        "🦺❌ Account `{account:?}` has {num_of_schedules} active schedules, which should \
        correspond to at least {lowerbound} locked balance, but actually there is {locked}."
    )]
    UnexpectedLockedBalances {
        locked: Balance,
        num_of_schedules: usize,
        lowerbound: Balance,
        account: AccountId,
    },
    #[error("🦺❌ Couldn't merge all active schedules for the account `{0:?}`.")]
    MergingFailure(AccountId),
}

/// Configuration for `SchedulesMerging` scenario.
#[derive(Clone, Debug, Deserialize)]
pub struct SchedulesMergingConfig {
    /// Unique string identifier for the scenario.
    ident: Ident,
    /// Periodicity of launching.
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
}

/// Scenario that performs merging vesting schedules. This happens as follows:
///  1. We choose a random receiver account.
///  2. We perform `MAX_VESTING_SCHEDULES` vested transfers to receiver so that no other vested
///     transfer can succeed.
///  3. Receiver merges all current schedules, exposing itself for further transfers.
#[derive(Clone)]
pub struct SchedulesMerging {
    ident: Ident,
    interval: Duration,
    schedules_limit: usize,
    transfer_value: Balance,
    connection: Connection,
}

impl SchedulesMerging {
    pub fn new(connection: &Connection, config: SchedulesMergingConfig) -> AnyResult<Self> {
        let schedules_limit = connection
            .get_constant::<u32>("Vesting", "MaxVestingSchedules")
            .expect("Constant Vesting::MaxVestingSchedules should be present");
        ensure!(schedules_limit >= 2, SchedulesMergingError::LimitTooLow);

        let transfer_value = connection
            .get_constant::<Balance>("Vesting", "MinVestedTransfer")
            .expect("Constant Vesting::MinVestedTransfer should be present");

        Ok(Self {
            ident: config.ident,
            interval: config.interval,
            schedules_limit: schedules_limit as usize,
            transfer_value,
            connection: connection.clone(),
        })
    }

    fn get_common_schedule(&self) -> VestingSchedule {
        VestingSchedule::new(self.transfer_value, 1u128, BlockNumber::MAX)
    }

    async fn transfer(&self, receiver: &AccountId, sender_idx: usize) -> AnyResult<()> {
        let sender = compute_keypair(sender_idx);
        let connection = self.connection.clone().set_signer(sender);
        let schedule = self.get_common_schedule();
        do_async!(vested_transfer, connection, receiver, schedule)?
    }

    fn get_vesting_info(&self, receiver: &AccountId) -> AnyResult<(usize, Balance)> {
        let schedules = self
            .connection
            .get_storage_map::<AccountId, Vec<VestingSchedule>>(
                "Vesting",
                "Vesting",
                receiver.clone(),
                None,
            )?;

        match schedules {
            Some(schedules) => {
                let num_of_schedules = schedules.len();
                let locked = schedules
                    .iter()
                    .fold(0u128, |acc, schedule| acc + schedule.locked());
                Ok((num_of_schedules, locked))
            }
            // meh
            None => Ok((0, 0)),
        }
    }

    async fn reach_limit(&self, receiver: &AccountId) -> AnyResult<Balance> {
        self.info(format!(
            "Start making vested transfers to {:?} in order to reach vesting schedules limit",
            receiver,
        ));

        let (num_of_schedules, locked) = self.get_vesting_info(receiver)?;
        ensure!(
            num_of_schedules < self.schedules_limit,
            SchedulesMergingError::LimitAlreadyReached(receiver.clone())
        );
        let locked_lowerbound = self.transfer_value * (num_of_schedules as u128);
        ensure!(
            locked >= locked_lowerbound,
            SchedulesMergingError::UnexpectedLockedBalances {
                locked,
                lowerbound: locked_lowerbound,
                num_of_schedules,
                account: receiver.clone()
            }
        );

        for i in num_of_schedules..self.schedules_limit {
            let expected_locked_after =
                locked + self.transfer_value * (1 + i - num_of_schedules) as u128;
            let expected_event = VestingUpdated::new(receiver.clone(), expected_locked_after);

            with_event_listening(
                &self.connection,
                expected_event,
                Duration::from_secs(1),
                async { self.transfer(receiver, i).await },
            )
            .await
            .map(|_| ())?;

            self.debug(format!(
                "Reaching limit: {}/{}",
                i + 1,
                self.schedules_limit
            ));
        }

        let (num_of_schedules, locked) = self.get_vesting_info(receiver)?;
        ensure!(
            num_of_schedules == self.schedules_limit,
            SchedulesMergingError::ReachingLimitFailure(receiver.clone())
        );

        self.info(format!(
            "Reached maximum number of vesting schedules for {:?}",
            receiver,
        ));
        Ok(locked)
    }

    async fn merge_schedules(&self, receiver: &KeyPair, total_locked: Balance) -> AnyResult<()> {
        let receiver_account = account_from_keypair(receiver);
        self.info(format!(
            "Start merging schedules for {:?}",
            receiver_account.clone()
        ));

        let expected_event = VestingUpdated::new(receiver_account.clone(), total_locked);
        let timeout = Duration::from_secs(1);

        let connection = self.connection.clone().set_signer(receiver.clone());
        for i in 1..self.schedules_limit {
            with_event_listening(&self.connection, expected_event.clone(), timeout, async {
                match do_async!(merge_schedules, connection, 0, 1) {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(e) => Err(e.into()),
                }
            })
            .await
            .map(|_| ())?;

            self.debug(format!(
                "Merged schedules: {}/{}",
                i + 1,
                self.schedules_limit
            ));
        }

        let (num_of_schedules, _) = self.get_vesting_info(&receiver_account)?;
        ensure!(
            num_of_schedules == 1,
            SchedulesMergingError::MergingFailure(receiver_account.clone())
        );

        self.info(format!("Merged all schedules for {:?}", receiver_account));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario for SchedulesMerging {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> AnyResult<()> {
        self.info("Starting scenario");

        let receiver = get_random_keypair();
        let receiver_account = account_from_keypair(&receiver);

        let locked = self.handle(self.reach_limit(&receiver_account).await)?;
        self.handle(self.merge_schedules(&receiver, locked).await)?;

        self.info("Successfully finished scenario");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
