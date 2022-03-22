use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{Log, Metadata, Record};

use crate::Ident;

#[derive(Default, Clone)]
pub struct Logger {
    logs: Arc<Mutex<HashMap<Ident, Vec<String>>>>,
}

impl Logger {
    pub fn claim_logs(&self, target: &Ident) -> Vec<String> {
        self.logs
            .lock()
            .expect("Should acquire lock")
            .remove(target)
            .unwrap_or_default()
    }

    fn format(record: &Record) -> String {
        format!(
            "{}  {}  {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        )
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.logs
            .lock()
            .expect("Should acquire lock")
            .entry(Ident(record.target().to_string()))
            .or_insert_with(Vec::new)
            .push(Self::format(record));
    }

    fn flush(&self) {}
}
