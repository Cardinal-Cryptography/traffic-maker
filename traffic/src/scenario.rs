use std::time::Duration;

#[async_trait::async_trait]
pub trait Scenario: Clone + Send + Sync + 'static {
    /// How often should it be run.
    fn interval(&self) -> Duration;

    /// Run the scenario and return whether it succeeded.
    // TODO: make it return Result<>
    async fn play(&self) -> bool;

    /// String identifier for this particular scenario.
    fn ident(&self) -> str;

    /// Whether the first run should occur immediately or after `interval()`.
    fn immediate(&self) -> bool;
}
