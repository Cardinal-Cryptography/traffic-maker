use reqwest::Response;
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
    pub async fn fetch_all(base_url: String) -> Result<Vec<Scenario>, String> {
        let url = format!("{}/details", base_url);
        let x = reqwest::get(url.clone()).await;
        let y = if x.is_ok() { 1 } else { 2 };

        Ok(vec![Scenario::new(
            format!("Scenario from {}", url),
            y,
            y,
            Duration::from_secs(5),
            Some(Status::Success),
        )])
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
