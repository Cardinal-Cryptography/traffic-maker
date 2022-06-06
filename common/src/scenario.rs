use aleph_client::Connection;
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
}

impl Display for ScenarioError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioError::ExecutionFailure => write!(f, "Failure while executing scenario"),
            ScenarioError::CannotSendExtrinsic => write!(f, "Could not send extrinsic"),
        }
    }
}

/// Core trait that every bot should satisfy.
#[async_trait::async_trait]
pub trait Scenario: Send + Sync + 'static {
    /// Runs the scenario and returns whether it succeeded.
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()>;
}

#[async_trait::async_trait]
impl Scenario for Box<dyn Scenario> {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        self.as_mut().play(connection, logger).await
    }
}

/// A wrapper around the actual scenario that manages cross-cutting concerns.
#[async_trait::async_trait]
pub trait ScenarioInstance: Send + Sync + 'static {
    /// Identifier for this instance of the scenario.
    fn ident(&self) -> Ident;
    /// How often should it be run.
    fn interval(&self) -> Duration;
    /// Runs the scenario and returns whether it succeeded.
    async fn play(&mut self) -> AnyResult<()>;
}

pub struct ScenarioContainer<S: Scenario> {
    /// Identifier for this instance of the scenario.
    pub ident: Ident,
    /// How often should it be run.
    pub interval: Duration,
    /// The connection to use for the scenario.
    pub connection: Connection,
    /// The actual scenario to perform.
    pub scenario: S,
}

#[async_trait::async_trait]
impl<S: Scenario> ScenarioInstance for ScenarioContainer<S> {
    fn ident(&self) -> Ident {
        self.ident.clone()
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> AnyResult<()> {
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
    pub fn new<S: ScenarioInstance + ?Sized>(instance: &S) -> Self {
        ScenarioDetails {
            ident: instance.ident(),
            runs: 0,
            failures: 0,
            interval: instance.interval(),
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

    pub fn handle<R: Debug>(&self, result: AnyResult<R>) -> AnyResult<R> {
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
