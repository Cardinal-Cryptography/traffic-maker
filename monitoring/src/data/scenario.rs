use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

/// `Status` and `Scenario` are deserializable counterparts of the corresponding
/// structs from `bin/stats.rs`.
#[derive(Debug, Clone, Deserialize)]
pub enum Status {
    NotLaunchedYet,
    Success,
    Failure,
    Running,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub ident: String,
    pub runs: u32,
    pub failures: u32,
    pub interval: Duration,
    pub last_status: Status,
}

impl Scenario {
    pub async fn fetch_all(base_url: String) -> Result<Vec<Scenario>, String> {
        Self::_fetch_all(base_url)
            .await
            .map_err(|e| format!("{:?}", e))
    }

    async fn _fetch_all(base_url: String) -> reqwest::Result<Vec<Scenario>> {
        Ok(reqwest::get(format!("{}/details", base_url))
            .await?
            .json::<HashMap<String, Scenario>>()
            .await?
            .values()
            .cloned()
            .collect())
    }
}
