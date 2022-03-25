use std::time::Duration;

use primitives::Ident;

#[async_trait::async_trait]
pub trait Scenario: Clone + Send + Sync + 'static {
    /// How often should it be run.
    fn interval(&self) -> Duration;

    /// Runs the scenario and returns whether it succeeded.
    // TODO: make it return Result<>
    async fn play(&mut self) -> bool;

    /// Identifier for this particular scenario.
    fn ident(&self) -> Ident;
}
