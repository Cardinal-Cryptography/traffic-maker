use std::sync::{Arc, Mutex};

use futures::{
    channel::{mpsc, mpsc::UnboundedSender},
    StreamExt,
};

use crate::scenario::Scenario;

/// Abstraction for registering events (hook for stats).
pub trait EventListener: Send + Clone {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S);
    fn report_success(&mut self, scenario_ident: String);
    fn report_failure(&mut self, scenario_ident: String);
    fn report_logs(&mut self, scenario_ident: String, logs: Vec<String>);
}

impl<EL: EventListener> EventListener for Arc<Mutex<EL>> {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S) {
        self.lock().unwrap().register_scenario(scenario)
    }

    fn report_success(&mut self, scenario_ident: String) {
        self.lock().unwrap().report_success(scenario_ident)
    }

    fn report_failure(&mut self, scenario_ident: String) {
        self.lock().unwrap().report_failure(scenario_ident)
    }

    fn report_logs(&mut self, scenario_ident: String, logs: Vec<String>) {
        self.lock().unwrap().report_logs(scenario_ident, logs)
    }
}

/// Firstly schedules all the scenarios according to their declared intervals. Then, in a loop,
/// waits for the next ready scenario and launches it.
pub async fn run_schedule<EL: 'static + EventListener>(
    scenarios: Vec<impl Scenario>,
    event_listener: EL,
) {
    let (report_ready, mut receive_ready) = mpsc::unbounded();
    let mut event_listener = event_listener;

    for scenario in scenarios {
        event_listener.register_scenario(&scenario);
        tokio::spawn(schedule_scenario(scenario, report_ready.clone()));
    }

    loop {
        let mut scenario = receive_ready
            .next()
            .await
            .expect("There should be at least one scenario scheduled");

        let mut event_listener = event_listener.clone();
        tokio::spawn(async move {
            let id = scenario.ident().to_string();
            match scenario.play().await {
                true => event_listener.report_success(id),
                false => event_listener.report_failure(id),
            }
        });
    }
}

/// After every period of `scenario.interval()` reports readiness through the channel.
/// Additionally, if `scenario.immediate() == true`, reports its readiness immediately.
async fn schedule_scenario<S: Scenario>(
    scenario: S,
    report_ready: UnboundedSender<S>,
) -> impl Send {
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
