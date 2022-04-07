use std::{error::Error, fmt::Debug, time::Duration};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::Ident;

pub enum ScenarioError {
    ExecutionFailure,
    CannotSendExtrinsic,
}

impl<E: Error> From<E> for ScenarioError {
    fn from(_: E) -> Self {
        ScenarioError::ExecutionFailure
    }
}

/// Core trait that every bot should satisfy.
#[async_trait::async_trait]
pub trait Scenario: Send + 'static {
    /// How often should it be run.
    fn interval(&self) -> Duration;

    /// Runs the scenario and returns whether it succeeded.
    async fn play(&mut self) -> Result<(), ScenarioError>;

    /// Identifier for this particular scenario.
    fn ident(&self) -> Ident;
}

#[async_trait::async_trait]
impl Scenario for Box<dyn Scenario> {
    fn interval(&self) -> Duration {
        (**self).interval()
    }

    async fn play(&mut self) -> Result<(), ScenarioError> {
        (**self).play().await
    }

    fn ident(&self) -> Ident {
        (**self).ident()
    }
}

/// Current status of the scheduled scenario.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioStatus {
    /// Scenario is scheduled for its first run.
    NotLaunchedYet,
    /// Last run was successful. The scenario is scheduled.
    Success,
    /// Last run failed. The scenario is scheduled.
    Failure,
    /// The scenario is running now.
    Running,
}

/// The struct representing a running bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioDetails {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    pub ident: Ident,
    /// How many times the scenario has been already run.
    pub runs: u32,
    /// How many times the scenario has failed.
    pub failures: u32,
    /// How often the scenario is run. Corresponds to `fn interval(&self)` from `Scenario` trait.
    pub interval: Duration,
    /// Scenario status.
    pub last_status: ScenarioStatus,
}

impl ScenarioDetails {
    pub fn new<S: Scenario>(scenario: &S) -> Self {
        ScenarioDetails {
            ident: scenario.ident(),
            runs: 0,
            failures: 0,
            interval: scenario.interval(),
            last_status: ScenarioStatus::NotLaunchedYet,
        }
    }
}

/// The struct representing a collection of logs for a single bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioLogs {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    pub scenario_ident: Ident,
    /// List of the recent log lines.
    pub content: Vec<String>,
}

impl ScenarioLogs {
    pub fn new(scenario_ident: Ident) -> Self {
        ScenarioLogs {
            scenario_ident,
            content: Vec::new(),
        }
    }
}

pub trait ScenarioLogging {
    fn trace<M: AsRef<str> + Debug>(&self, message: M);
    fn debug<M: AsRef<str> + Debug>(&self, message: M);
    fn info<M: AsRef<str> + Debug>(&self, message: M);
    fn warn<M: AsRef<str> + Debug>(&self, message: M);
    fn error<M: AsRef<str> + Debug>(&self, message: M);

    fn handle<R: Debug>(&self, result: Result<R, ScenarioError>) -> Result<R, ScenarioError> {
        match result {
            Err(ScenarioError::CannotSendExtrinsic) => {
                self.error("Could not send extrinsic");
            }
            Err(_) => {
                self.error("Encountered scenario error");
            }
            Ok(ref result) => {
                self.trace(format!("Successfully obtained {:?}", result));
            }
        };
        result
    }
}

impl<S: Scenario> ScenarioLogging for S {
    fn trace<M: AsRef<str> + Debug>(&self, message: M) {
        trace!(target: self.ident().0.as_str(), "{:?}", message)
    }

    fn debug<M: AsRef<str> + Debug>(&self, message: M) {
        debug!(target: self.ident().0.as_str(), "{:?}", message)
    }

    fn info<M: AsRef<str> + Debug>(&self, message: M) {
        info!(target: self.ident().0.as_str(), "{:?}", message)
    }

    fn warn<M: AsRef<str> + Debug>(&self, message: M) {
        warn!(target: self.ident().0.as_str(), "{:?}", message)
    }

    fn error<M: AsRef<str> + Debug>(&self, message: M) {
        error!(target: self.ident().0.as_str(), "{:?}", message)
    }
}
