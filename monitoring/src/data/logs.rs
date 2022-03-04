use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
    pub scenario_ident: String,
    pub content: Vec<String>,
}

impl Logs {
    pub async fn fetch(_scenario_ident: String) -> Result<Logs, String> {
        Ok(Logs::new(
            "SimpleTransfer".to_string(),
            vec![
                "log-line1".to_string(),
                "log-line2".to_string(),
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
