use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::channel::mpsc::UnboundedSender;
use log::{Log, Metadata, Record};

use crate::Ident;

pub type LogLine = (Ident, String);

#[derive(Default, Clone)]
pub struct Logger {
    subscriptions: Arc<RwLock<HashMap<Ident, Vec<UnboundedSender<LogLine>>>>>,
}

impl Logger {
    pub fn subscribe(&self, target: &Ident, sender: UnboundedSender<LogLine>) {
        self.subscriptions
            .write()
            .expect("Should acquire write lock")
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
            .read()
            .expect("Should acquire read lock")
            .get(&target)
        {
            senders.iter().for_each(|s| {
                s.unbounded_send((target.clone(), Self::format(record)))
                    .expect("Should manage to send")
            })
        }
    }

    fn flush(&self) {}
}
