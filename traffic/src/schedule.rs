use futures::{
    channel::{mpsc, mpsc::UnboundedSender},
    StreamExt,
};

use crate::scenario::Scenario;

pub async fn run_schedule(scenarios: Vec<impl Scenario>) {
    let (report_ready, mut receive_ready) = mpsc::unbounded();

    for scenario in scenarios {
        tokio::spawn(schedule_scenario(scenario, report_ready.clone()));
    }

    loop {
        let scenario = receive_ready
            .next()
            .await
            .expect("There should be at least one scenario scheduled");

        tokio::spawn(async move {
            scenario.play().await;
        });
    }
}

async fn schedule_scenario<S: Scenario>(
    scenario: S,
    report_ready: UnboundedSender<S>,
) -> impl Send {
    async move {
        let mut interval = tokio::time::interval(scenario.interval());

        interval.tick().await;
        if scenario.immediate() {
            report_ready
                .unbounded_send(scenario.clone())
                .expect("Should be able to report readiness");
        }

        loop {
            interval.tick().await;
            report_ready
                .unbounded_send(scenario.clone())
                .expect("Should be able to report readiness");
        }
    }
}
