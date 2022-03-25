use std::collections::HashMap;

use common::{Ident, ScenarioDetails, ScenarioLogs};

pub async fn fetch_scenarios(base_url: String) -> Result<Vec<ScenarioDetails>, String> {
    inner_fetch_scenarios(base_url)
        .await
        .map_err(|e| format!("{:?}", e))
}

async fn inner_fetch_scenarios(base_url: String) -> reqwest::Result<Vec<ScenarioDetails>> {
    Ok(reqwest::get(format!("{}/details", base_url))
        .await?
        .json::<HashMap<String, ScenarioDetails>>()
        .await?
        .values()
        .cloned()
        .collect())
}

pub async fn fetch_logs(scenario_ident: Ident, base_url: String) -> Result<ScenarioLogs, String> {
    inner_fetch_logs(scenario_ident, base_url)
        .await
        .map_err(|e| format!("{:?}", e))
}

async fn inner_fetch_logs(
    scenario_ident: Ident,
    base_url: String,
) -> reqwest::Result<ScenarioLogs> {
    Ok(
        reqwest::get(format!("{}/logs/{}", base_url, scenario_ident.0))
            .await?
            .json::<ScenarioLogs>()
            .await?,
    )
}
