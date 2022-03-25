use primitives::Ident;
use serde::Deserialize;

/// A deserializable counterpart for a corresponding struct from `bin/stats.rs`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
    pub scenario_ident: Ident,
    pub content: Vec<String>,
}

impl Logs {
    pub async fn fetch(scenario_ident: Ident, base_url: String) -> Result<Logs, String> {
        Self::inner_fetch(scenario_ident, base_url)
            .await
            .map_err(|e| format!("{:?}", e))
    }

    async fn inner_fetch(scenario_ident: Ident, base_url: String) -> reqwest::Result<Logs> {
        Ok(
            reqwest::get(format!("{}/logs/{}", base_url, scenario_ident.0))
                .await?
                .json::<Logs>()
                .await?,
        )
    }
}
