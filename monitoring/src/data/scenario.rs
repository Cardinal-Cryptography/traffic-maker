use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use serde::Deserialize;

/// Result of a single scenario run.
#[derive(Debug, Clone, Deserialize)]
pub enum Status {
    Success,
    Failure,
}

/// The struct representing a running bot.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    pub ident: String,
    /// How many times the scenario has been already run.
    pub runs: u32,
    /// How many times the scenario has failed.
    pub failures: u32,
    /// How often the scenario is run. Corresponds to `fn interval(&self)` from `Scenario` trait.
    pub interval: Duration,
    /// Most recent status.
    pub last_status: Option<Status>,
}

impl Scenario {
    pub async fn fetch_all() -> Result<Vec<Scenario>, String> {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed);

        Ok(vec![
            Scenario::new(
                "SimpleTransfer".to_string(),
                COUNTER.load(Ordering::Relaxed) + 10,
                COUNTER.load(Ordering::Relaxed) + 2,
                Duration::from_secs(5),
                Some(Status::Success),
            ),
            Scenario::new(
                "ComplexTransfer".to_string(),
                4,
                1,
                Duration::from_millis(500000),
                Some(Status::Failure),
            ),
            Scenario::new(
                "ActuallyNoTransfer".to_string(),
                0,
                0,
                Duration::from_millis(500000000),
                None,
            ),
        ])
    }

    fn new(
        ident: String,
        runs: u32,
        failures: u32,
        interval: Duration,
        last_status: Option<Status>,
    ) -> Self {
        Scenario {
            ident,
            runs,
            failures,
            interval,
            last_status,
        }
    }
}
