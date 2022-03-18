use serde::Deserialize;

/// The struct representing a collection of logs for a single bot.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
    /// Unique identifier of the scenario. Corresponds to `fn ident(&self)` from `Scenario` trait.
    pub scenario_ident: String,
    /// List of the recent log lines.
    pub content: Vec<String>,
}

impl Logs {
    pub async fn fetch(_scenario_ident: String) -> Result<Logs, String> {
        Ok(Logs::new(
            "SimpleTransfer".to_string(),
            vec![
                "log-line1".to_string(),
                "Reeeaaallly looooong line from the logs for the chosen scenario. The title of \
                this scenario is presented above. And there are two log lines more."
                    .to_string(),
                "log-line3".to_string(),
            ],
        ))
    }

    fn new(scenario_ident: String, content: Vec<String>) -> Self {
        Logs {
            scenario_ident,
            content,
        }
    }
}
