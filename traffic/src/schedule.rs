use futures::{
    channel::{mpsc, mpsc::UnboundedSender},
    StreamExt,
};

use crate::scenario::Scenario;

/// Firstly schedules all the scenarios according to their declared intervals. Then, in a loop,
/// waits for the next ready scenario and launches it.
pub async fn run_schedule(scenarios: Vec<impl Scenario>) {
    let (report_ready, mut receive_ready) = mpsc::unbounded();

    for scenario in scenarios {
        tokio::spawn(schedule_scenario(scenario, report_ready.clone()));
    }

    loop {
        let mut scenario = receive_ready
            .next()
            .await
            .expect("There should be at least one scenario scheduled");

        tokio::spawn(async move {
            scenario.play().await;
        });
    }
}

/// After every period of `scenario.interval()` reports readiness through the channel.
async fn schedule_scenario<S: Scenario>(
    scenario: S,
    report_ready: UnboundedSender<S>,
) -> impl Send {
    let mut interval = tokio::time::interval(scenario.interval());

    interval.tick().await;
    loop {
        interval.tick().await;
        report_ready
            .unbounded_send(scenario.clone())
            .expect("Should be able to report readiness");
    }
}
