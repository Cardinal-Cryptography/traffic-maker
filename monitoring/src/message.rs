use primitives::{Ident, ScenarioDetails, ScenarioLogs};

/// Events driving the logic of the app.
#[derive(Debug, Clone)]
pub enum Message {
    /// Requested scenarios are ready.
    FetchedScenarios(Result<Vec<ScenarioDetails>, String>),
    /// Requested logs for a scenario are ready.
    FetchedLogs(Result<ScenarioLogs, String>),

    /// `Route::Overview` has been selected.
    GoToOverview,
    /// `Route::Logs` has been selected.
    GoToLogs(Ident),
}
