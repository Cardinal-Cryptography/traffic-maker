use std::collections::{hash_map::Entry, HashMap};

use log::warn;

use common::{Ident, Scenario, ScenarioDetails, ScenarioLogs, ScenarioStatus};
use traffic::EventListener;

use crate::data_export::DataExporter;

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

    fn update_status(&mut self, scenario_ident: Ident, status: ScenarioStatus) {
        Self::update_storage(&mut self.details, scenario_ident, |details| {
            match status {
                ScenarioStatus::Running | ScenarioStatus::NotLaunchedYet => {}
                ScenarioStatus::Success => details.runs += 1,
                ScenarioStatus::Failure => {
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
    fn register_scenario<S: Scenario + ?Sized>(&mut self, scenario: &S) {
        let id = scenario.ident();
        let already_registered = self
            .details
            .insert(id.clone(), ScenarioDetails::new(scenario))
            .is_some();

        if already_registered {
            warn!(target: "stats", "Scenario {:?} has already been registered", id);
        } else {
            self.logs.insert(id.clone(), ScenarioLogs::new(id));
        }
    }

    fn report_success(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, ScenarioStatus::Success)
    }

    fn report_launch(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, ScenarioStatus::Running)
    }

    fn report_failure(&mut self, scenario_ident: Ident) {
        self.update_status(scenario_ident, ScenarioStatus::Failure)
    }

    fn report_logs(&mut self, scenario_ident: Ident, log: String) {
        Self::update_storage(&mut self.logs, scenario_ident, |all_logs| {
            // TODO: make `content` a bounded container
            all_logs.content.push(log);
        });
    }
}
