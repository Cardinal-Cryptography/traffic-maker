use std::sync::{Arc, Mutex};

use futures::{
    channel::{mpsc, mpsc::UnboundedSender},
    StreamExt,
};
use log::LevelFilter;

use primitives::Ident;

use crate::{logger::Logger, scenario::Scenario};

/// Abstraction for registering events (hook for stats).
pub trait EventListener: Send + Clone {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S);
    fn report_success(&mut self, scenario_ident: Ident);
    fn report_launch(&mut self, scenario_ident: Ident);
    fn report_failure(&mut self, scenario_ident: Ident);
    fn report_logs(&mut self, scenario_ident: Ident, log: String);
}

impl<EL: EventListener> EventListener for Arc<Mutex<EL>> {
    fn register_scenario<S: Scenario>(&mut self, scenario: &S) {
        self.lock().unwrap().register_scenario(scenario)
    }

    fn report_success(&mut self, scenario_ident: Ident) {
        self.lock().unwrap().report_success(scenario_ident)
    }

    fn report_launch(&mut self, scenario_ident: Ident) {
        self.lock().unwrap().report_launch(scenario_ident)
    }

    fn report_failure(&mut self, scenario_ident: Ident) {
        self.lock().unwrap().report_failure(scenario_ident)
    }

    fn report_logs(&mut self, scenario_ident: Ident, log: String) {
        self.lock().unwrap().report_logs(scenario_ident, log)
    }
}

/// Firstly schedules all the scenarios according to their declared intervals. Then, in a loop,
/// waits for the next ready scenario and launches it.
pub async fn run_schedule<EL: 'static + EventListener>(
    scenarios: Vec<impl Scenario>,
    event_listener: EL,
) {
    let logger = setup_logging();
    let (report_logs, mut receive_logs) = mpsc::unbounded();
    let (report_ready, mut receive_ready) = mpsc::unbounded();
    let mut event_listener = event_listener;

    for scenario in scenarios {
        event_listener.register_scenario(&scenario);
        logger.subscribe(&scenario.ident(), report_logs.clone());
        tokio::spawn(schedule_scenario(scenario, report_ready.clone()));
    }

    let mut logs_listener = event_listener.clone();
    tokio::spawn(async move {
        loop {
            let (ident, log) = receive_logs
                .next()
                .await
                .expect("There should be some logging on");
            logs_listener.report_logs(ident, log);
        }
    });

    loop {
        let mut scenario = receive_ready
            .next()
            .await
            .expect("There should be at least one scenario scheduled");

        let mut event_listener = event_listener.clone();
        tokio::spawn(async move {
            let id = scenario.ident();
            event_listener.report_launch(id.clone());
            match scenario.play().await {
                true => event_listener.report_success(id.clone()),
                false => event_listener.report_failure(id.clone()),
            }
        });
    }
}

fn setup_logging() -> Logger {
    let logger = Logger::default();
    if log::set_boxed_logger(Box::new(logger.clone())).is_ok() {
        let level = match option_env!("MAX_LOG_LEVEL")
            .unwrap_or("DEBUG")
            .to_lowercase()
            .as_str()
        {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Debug,
        };
        log::set_max_level(level);
        logger
    } else {
        panic!("Cannot setup logger")
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
