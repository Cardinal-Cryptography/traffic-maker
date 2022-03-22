use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use log::warn;
use serde::Serialize;

use traffic::{EventListener, Ident, Scenario};

use crate::data_export::DataExporter;

/// Current status of the scheduled scenario.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Status {
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
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioDetails {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    ident: Ident,
    /// How many times the scenario has been already run.
    runs: u32,
    /// How many times the scenario has failed.
    failures: u32,
    /// How often the scenario is run. Corresponds to `fn interval(&self)` from `Scenario` trait.
    interval: Duration,
    /// Scenario status.
    last_status: Status,
}

impl ScenarioDetails {
    pub fn new<S: Scenario>(scenario: S) -> Self {
        ScenarioDetails {
            ident: scenario.ident(),
            runs: 0,
            failures: 0,
            interval: scenario.interval(),
            last_status: Status::NotLaunchedYet,
        }
    }
}

/// The struct representing a collection of logs for a single bot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioLogs {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    scenario_ident: Ident,
    /// List of the recent log lines.
    content: Vec<String>,
}

impl ScenarioLogs {
    pub fn new(scenario_ident: Ident) -> Self {
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
    details: HashMap<Ident, ScenarioDetails>,
    logs: HashMap<Ident, ScenarioLogs>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            details: HashMap::new(),
            logs: HashMap::new(),
        }
    }

    fn update_storage<V, A: FnOnce(&mut V)>(
        storage: &mut HashMap<Ident, V>,
        scenario_ident: Ident,
        update_action: A,
    ) {
        match storage.entry(scenario_ident.clone()) {
            Entry::Vacant(_) => {
                warn!(target: "stats", "Scenario {:?} has not been registered yet", scenario_ident)
            }
            Entry::Occupied(ref mut entry) => update_action(entry.get_mut()),
        }
    }

    fn update_status(&mut self, scenario_ident: Ident, status: Status) {
        Self::update_storage(&mut self.details, scenario_ident, |details| {
            match status {
                Status::Running | Status::NotLaunchedYet => {}
                Status::Success => details.runs += 1,
                Status::Failure => {
                    details.runs += 1;
                    details.failures += 1;
                }
            }
            details.last_status = status;
        })
    }
}

impl DataExporter for Stats {
    fn export_details(&self) -> String {
        serde_json::to_string(&self.details).expect("Details should be serializable")
    }

    fn export_logs(&self, scenario_ident: Ident) -> String {
        if let Some(logs) = self.logs.get(&scenario_ident) {
            serde_json::to_string(logs).expect("Logs should be serializable")
        } else {
            "".to_string()
        }
    }
}

impl EventListener for Stats {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S) {
        let id = scenario.ident();
        let already_registered = self
            .details
            .insert(id.clone(), ScenarioDetails::new(scenario.clone()))
            .is_some();

        if already_registered {
            warn!(target: "stats", "Scenario {:?} has already been registered", id);
        } else {
            self.logs.insert(id.clone(), ScenarioLogs::new(id));
        }
    }

    fn report_success(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, Status::Success)
    }

    fn report_launch(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, Status::Running)
    }

    fn report_failure(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, Status::Failure)
    }

    fn report_logs(&mut self, scenario_ident: Ident, logs: Vec<String>) {
        Self::update_storage(&mut self.logs, scenario_ident, |all_logs| {
            let mut logs = logs;
            // TODO: make `content` a bounded container
            all_logs.content.append(&mut logs);
        });
    }
}
