use anyhow::Result as AnyResult;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter},
    time::Duration,
};
use thiserror::Error;

use crate::Ident;

#[derive(Debug, Error)]
pub enum ScenarioError {
    ExecutionFailure,
    CannotSendExtrinsic,
    BadConfig,
}

impl Display for ScenarioError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioError::ExecutionFailure => write!(f, "Failure while executing scenario"),
            ScenarioError::CannotSendExtrinsic => write!(f, "Could not send extrinsic"),
            ScenarioError::BadConfig => write!(f, "Bad config values"),
        }
    }
}

/// Core trait that every bot should satisfy.
#[async_trait::async_trait]
pub trait Scenario<C>: Send + Sync + 'static {
    /// Runs the scenario and returns whether it succeeded.
    async fn play(&mut self, connection: &C, logger: &ScenarioLogging) -> AnyResult<()>;
}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Scenario<C> for Box<dyn Scenario<C>> {
    async fn play(&mut self, connection: &C, logger: &ScenarioLogging) -> AnyResult<()> {
        self.as_mut().play(connection, logger).await
    }
}

pub struct ScheduledScenario<C> {
    /// Identifier for this instance of the scenario.
    ident: Ident,
    /// How often should it be run.
    interval: Duration,
    /// The connection to use for the scenario.
    connection: C,
    /// The actual scenario to perform.
    scenario: Box<dyn Scenario<C>>,
}

impl<C: Send + Sync + 'static> ScheduledScenario<C> {
    pub fn new(
        ident: Ident,
        interval: Duration,
        connection: C,
        scenario: impl Scenario<C>,
    ) -> ScheduledScenario<C> {
        ScheduledScenario {
            ident,
            interval,
            connection,
            scenario: Box::new(scenario),
        }
    }

    pub fn ident(&self) -> Ident {
        self.ident.clone()
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub async fn play(&mut self) -> AnyResult<()> {
        self.scenario
            .play(
                &self.connection,
                &ScenarioLogging {
                    ident: self.ident.clone(),
                },
            )
            .await
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
    pub fn new<C: Send + Sync + 'static>(scenario: &ScheduledScenario<C>) -> Self {
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

pub struct ScenarioLogging {
    ident: Ident,
}

impl ScenarioLogging {
    pub fn trace<M: Debug>(&self, message: M) {
        trace!(target: self.ident.0.as_str(), "{:?}", message)
    }

    pub fn debug<M: Debug>(&self, message: M) {
        debug!(target: self.ident.0.as_str(), "{:?}", message)
    }

    pub fn info<M: Debug>(&self, message: M) {
        info!(target: self.ident.0.as_str(), "{:?}", message)
    }

    pub fn warn<M: Debug>(&self, message: M) {
        warn!(target: self.ident.0.as_str(), "{:?}", message)
    }

    pub fn error<M: Debug>(&self, message: M) {
        error!(target: self.ident.0.as_str(), "{:?}", message)
    }

    pub fn log_result<R: Debug>(&self, result: AnyResult<R>) -> AnyResult<R> {
        match &result {
            Err(e) => {
                self.error(format!("Encountered error: {}", e));
            }
            Ok(result) => {
                self.trace(format!("Successfully obtained {:?}", result));
            }
        };
        result
    }
}
