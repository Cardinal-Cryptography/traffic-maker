use crate::data::{Logs, Scenario};

/// Events driving the logic of the app.
#[derive(Debug, Clone)]
pub enum Message {
    /// Requested scenarios are ready.
    FetchedScenarios(Result<Vec<Scenario>, String>),
    /// Requested logs for a scenario are ready.
    FetchedLogs(Result<Logs, String>),

    /// `Route::Overview` has been selected.
    GoToOverview,
    /// `Route::Logs` has been selected.
    GoToLogs(String),
}
