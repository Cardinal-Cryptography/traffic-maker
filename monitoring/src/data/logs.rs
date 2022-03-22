use serde::Deserialize;
use std::collections::HashMap;

/// A desarizable counterpart for a corresponding struct from `bin/stats.rs`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
    pub scenario_ident: String,
    pub content: Vec<String>,
}

impl Logs {
    pub async fn fetch(_scenario_ident: String, base_url: String) -> Result<Logs, String> {
        Self::_fetch(base_url).await.map_err(|e| format!("{:?}", e))
    }

    async fn _fetch(base_url: String) -> reqwest::Result<Logs> {
        Ok(reqwest::get(format!("{}/logs", base_url))
            .await?
            .json::<HashMap<String, Logs>>()
            .await?
            .values()
            .cloned()
            .collect())
        .map(|v: Vec<Logs>| v[0].clone())
    }
}
