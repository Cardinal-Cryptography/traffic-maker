use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::channel::mpsc::UnboundedSender;
use log::{Log, Metadata, Record};

use crate::Ident;

pub type LogLine = (Ident, String);

#[derive(Default, Clone)]
pub struct Logger {
    subscriptions: Arc<Mutex<HashMap<Ident, Vec<UnboundedSender<LogLine>>>>>,
}

impl Logger {
    pub fn subscribe(&self, target: &Ident, sender: UnboundedSender<LogLine>) {
        self.subscriptions
            .lock()
            .expect("Should acquire lock")
            .entry(target.clone())
            .or_insert_with(Vec::new)
            .push(sender);
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
        let target = Ident(record.target().to_string());
        if let Some(senders) = self
            .subscriptions
            .lock()
            .expect("Should acquire lock")
            .get_mut(&target)
        {
            senders.retain(|s| {
                s.unbounded_send((target.clone(), Self::format(record)))
                    .is_ok()
            })
        }
    }

    fn flush(&self) {}
}
