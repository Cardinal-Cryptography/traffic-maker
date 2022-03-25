use primitives::Ident;
use std::sync::{Arc, Mutex};

/// Exposing scenario data.
pub trait DataExporter {
    fn export_details(&self) -> String;
    fn export_logs(&self, scenario_ident: Ident) -> String;
}

impl<DE: DataExporter> DataExporter for Arc<Mutex<DE>> {
    fn export_details(&self) -> String {
        self.lock().unwrap().export_details()
    }

    fn export_logs(&self, scenario_ident: Ident) -> String {
        self.lock().unwrap().export_logs(scenario_ident)
    }
}
