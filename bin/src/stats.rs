use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use log::warn;
use serde::Serialize;

use crate::data_export::DataExporter;
use traffic::{EventListener, Scenario};

/// Result of a single scenario run.
#[derive(Debug, Clone, Serialize)]
pub enum Status {
    Success,
    Failure,
}

/// The struct representing a running bot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioDetails {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    ident: String,
    /// How many times the scenario has been already run.
    runs: u32,
    /// How many times the scenario has failed.
    failures: u32,
    /// How often the scenario is run. Corresponds to `fn interval(&self)` from `Scenario` trait.
    interval: Duration,
    /// Most recent status.
    last_status: Option<Status>,
}

impl ScenarioDetails {
    pub fn new<S: Scenario>(scenario: S) -> Self {
        ScenarioDetails {
            ident: scenario.ident().to_string(),
            runs: 0,
            failures: 0,
            interval: scenario.interval(),
            last_status: None,
        }
    }
}

/// The struct representing a collection of logs for a single bot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioLogs {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    scenario_ident: String,
    /// List of the recent log lines.
    content: Vec<String>,
}

impl ScenarioLogs {
    pub fn new(scenario_ident: String) -> Self {
        ScenarioLogs {
            scenario_ident,
            content: Vec::new(),
        }
    }
}

/// A single struct realizing two important concepts: data exposure (`DataExporter` trait)
/// and event registration (`EventListener` trait). It is the way in which the scheduler
/// communicates with the outer world.
#[derive(Clone)]
pub struct Stats {
    details: HashMap<String, ScenarioDetails>,
    logs: HashMap<String, ScenarioLogs>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            details: HashMap::new(),
            logs: HashMap::new(),
        }
    }
}

impl DataExporter for Stats {
    fn export_details(&self) -> String {
        serde_json::to_string(&self.details).expect("Details should be serializable")
    }

    fn export_logs(&self) -> String {
        serde_json::to_string(&self.logs).expect("Logs should be serializable")
    }
}

impl EventListener for Stats {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S) {
        let id = scenario.ident().to_string();
        let already_registered = self
            .details
            .insert(id.clone(), ScenarioDetails::new(scenario.clone()))
            .is_some();

        if already_registered {
            warn!(target: "stats", "Scenario {} has already been registered", id);
        } else {
            self.logs.insert(id.clone(), ScenarioLogs::new(id));
        }
    }

    fn report_success(&mut self, scenario_ident: String) {
        match self.details.entry(scenario_ident.clone()) {
            Entry::Vacant(_) => {
                warn!(target: "stats", "Scenario {} has not been registered yet", scenario_ident)
            }
            Entry::Occupied(ref mut entry) => {
                let mut details = entry.get_mut();
                details.runs += 1;
                details.last_status = Some(Status::Success);
            }
        }
    }

    fn report_failure(&mut self, scenario_ident: String) {
        match self.details.entry(scenario_ident.clone()) {
            Entry::Vacant(_) => {
                warn!(target: "stats", "Scenario {} has not been registered yet", scenario_ident)
            }
            Entry::Occupied(ref mut entry) => {
                let mut details = entry.get_mut();
                details.runs += 1;
                details.failures += 1;
                details.last_status = Some(Status::Failure);
            }
        }
    }

    fn report_logs(&mut self, scenario_ident: String, logs: Vec<String>) {
        match self.logs.entry(scenario_ident.clone()) {
            Entry::Vacant(_) => {
                warn!(target: "stats", "Scenario {} has not been registered yet", scenario_ident)
            }
            Entry::Occupied(ref mut entry) => {
                let all_logs = entry.get_mut();
                let mut logs = logs;
                // TODO: make `content` a bounded container
                all_logs.content.append(&mut logs);
            }
        }
    }
}
