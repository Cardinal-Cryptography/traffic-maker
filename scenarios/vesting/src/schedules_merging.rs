use aleph_client::{
    account_from_keypair, api,
    pallet_vesting::vesting_info::VestingInfo,
    pallets::vesting::{VestingApi, VestingUserApi},
    AccountId, Connection, KeyPair, SignedConnection, TxStatus,
};
use anyhow::{ensure, Result as AnyResult};
use api::constants;
use chain_support::{keypair_derived_from_seed, Balance};
use common::{Scenario, ScenarioLogging};
use rand::random;
use thiserror::Error;

/// We operate on an account pool based on this seed. The final seeds will have a form of
/// `ACCOUNT_SEED{i: usize}`.
const ACCOUNT_SEED: &str = "//VestingSchedulesMerging";

/// We expect that there are as many endowed accounts (of seed phrases: `ACCOUNT_SEED{i}`, where
/// `i` is from 0 to this value (exclusively)).
///
/// This should not be less than `MAX_VESTING_SCHEDULES` constant (of pallet vesting).
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
        "🦺❌ Account `{account:?}` has {num_of_schedules} active schedules, which should sum to \
        {expected} locked balance, but actually there is {locked}."
    )]
    UnexpectedLockedBalances {
        locked: Balance,
        num_of_schedules: usize,
        expected: Balance,
        account: AccountId,
    },
    #[error("🦺❌ Couldn't merge all active schedules for the account `{0:?}`.")]
    MergingFailureNumber(AccountId),
    #[error(
        "🦺❌ Merging schedules for the account `{account:?}` has led to changing overall locked \
        balance amount from {locked_before_merging} to {locked_after_merging}"
    )]
    MergingFailureLocked {
        locked_before_merging: Balance,
        locked_after_merging: Balance,
        account: AccountId,
    },
}

/// Scenario that performs merging vesting schedules. This happens as follows:
///  1. We choose a random receiver account.
///  2. We perform at most `MaxVestingSchedules` vested transfers to receiver so that no other
///     vested transfer can succeed. If receiver already had some schedules, we just meet the limit.
///  3. Receiver merges all current schedules, exposing itself for further transfers.
#[derive(Clone)]
pub struct SchedulesMerging {
    /// Corresponds to `MaxVestingSchedules` constant.
    schedules_limit: usize,
    /// Corresponds to `MinVestedTransfer` constant.
    transfer_value: Balance,
}

impl SchedulesMerging {
    /// Constructs new `SchedulesMerging` object.
    ///
    /// Fails if `MaxVestingSchedules` is less than 2.
    pub fn new(connection: &Connection) -> AnyResult<Self> {
        let schedules_limit: u32 = connection
            .client
            .constants()
            .at(&constants().vesting().max_vesting_schedules())
            .unwrap();

        ensure!(schedules_limit >= 2, SchedulesMergingError::LimitTooLow);

        let transfer_value: Balance = connection
            .client
            .constants()
            .at(&constants().vesting().min_vested_transfer())
            .unwrap();

        Ok(Self {
            schedules_limit: schedules_limit as usize,
            transfer_value,
        })
    }

    /// Performs vested transfer from `ACCOUNT_SEED{sender_idx}` to `receiver`.
    async fn transfer(
        &self,
        connection: &Connection,
        receiver: &AccountId,
        sender_idx: usize,
    ) -> AnyResult<()> {
        let sender = compute_keypair(sender_idx);
        SignedConnection::from_connection(connection.clone(), sender)
            .vested_transfer(
                receiver.clone(),
                VestingInfo {
                    locked: self.transfer_value,
                    per_block: 1,
                    starting_block: u32::MAX,
                },
                TxStatus::Finalized,
            )
            .await
            .map(|_| ())
    }

    /// Reads how many vesting schedules `receiver` has and how much balance there is in summary.
    async fn get_vesting_info(
        &self,
        connection: &Connection,
        receiver: &AccountId,
    ) -> (usize, Balance) {
        let schedules = connection.get_vesting(receiver.clone(), None).await;
        let num_of_schedules = schedules.len();
        let locked = schedules
            .iter()
            .fold(0u128, |acc, schedule| acc + schedule.locked);
        (num_of_schedules, locked)
    }

    /// Performs as many vested transfers to `receiver` as it is needed to meet limit of
    /// `self.schedules_limit` active vesting schedules.
    ///
    /// Returns the amount of all locked tokens at the end.
    async fn reach_limit(
        &self,
        connection: &Connection,
        receiver: &AccountId,
        logger: &ScenarioLogging,
    ) -> AnyResult<Balance> {
        logger.info(format!(
            "Start making vested transfers to {:?} in order to reach vesting schedules limit",
            receiver,
        ));

        let (num_of_schedules, locked_before) = self.get_vesting_info(connection, receiver).await;
        ensure!(
            num_of_schedules < self.schedules_limit,
            SchedulesMergingError::LimitAlreadyReached(receiver.clone())
        );

        for i in num_of_schedules..self.schedules_limit {
            self.transfer(connection, receiver, i).await?;

            logger.debug(format!(
                "Reaching limit: {}/{}",
                i + 1,
                self.schedules_limit
            ));
        }

        let (num_of_schedules, locked_after) = self.get_vesting_info(connection, receiver).await;
        ensure!(
            num_of_schedules == self.schedules_limit,
            SchedulesMergingError::ReachingLimitFailure(receiver.clone())
        );
        let new_locked =
            ((self.schedules_limit - num_of_schedules) as Balance) * self.transfer_value;
        ensure!(
            locked_before + new_locked == locked_after,
            SchedulesMergingError::UnexpectedLockedBalances {
                num_of_schedules,
                locked: locked_after,
                expected: locked_before + new_locked,
                account: receiver.clone()
            }
        );

        logger.info(format!(
            "Reached maximum number of vesting schedules for {:?}",
            receiver,
        ));
        Ok(locked_after)
    }

    /// Merges all active vesting schedules for `receiver` into a single one.
    async fn merge_schedules(
        &self,
        connection: &Connection,
        receiver: KeyPair,
        logger: &ScenarioLogging,
    ) -> AnyResult<()> {
        let receiver_account = account_from_keypair(receiver.signer());
        logger.info(format!(
            "Start merging schedules for {:?}",
            receiver_account.clone()
        ));

        let connection = SignedConnection::from_connection(connection.clone(), receiver);
        for i in 1..self.schedules_limit {
            connection
                .merge_schedules(0, 1, TxStatus::Finalized)
                .await?;

            logger.debug(format!(
                "Merged schedules: {}/{}",
                i + 1,
                self.schedules_limit
            ));
        }

        logger.info(format!("Merged all schedules for {:?}", receiver_account));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario<Connection> for SchedulesMerging {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        logger.info("Starting scenario");

        let receiver = get_random_keypair();
        let receiver_account = account_from_keypair(receiver.signer());

        let locked_before_merging = logger.log_result(
            self.reach_limit(connection, &receiver_account, logger)
                .await,
        )?;
        logger.log_result(self.merge_schedules(connection, receiver, logger).await)?;

        let (num_of_schedules, locked_after_merging) =
            self.get_vesting_info(connection, &receiver_account).await;
        ensure!(
            num_of_schedules == 1,
            SchedulesMergingError::MergingFailureNumber(receiver_account.clone())
        );
        ensure!(
            locked_before_merging == locked_after_merging,
            SchedulesMergingError::MergingFailureLocked {
                account: receiver_account,
                locked_before_merging,
                locked_after_merging,
            }
        );

        logger.info("Successfully finished scenario");
        Ok(())
    }
}
