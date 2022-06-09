use std::sync::{Arc, Mutex};

use futures::{
    channel::{mpsc, mpsc::UnboundedReceiver},
    StreamExt,
};
use log::{error, LevelFilter};
use tokio::task::JoinHandle;

use common::{Ident, ScheduledScenario};

use crate::logger::{LogLine, Logger};

/// Abstraction for registering events (hook for stats).
pub trait EventListener: Send + Sync + Clone {
    fn register_scenario<C: Send + Sync + 'static>(&mut self, scenario: &ScheduledScenario<C>);
    fn report_success(&mut self, scenario_ident: Ident);
    fn report_launch(&mut self, scenario_ident: Ident);
    fn report_failure(&mut self, scenario_ident: Ident);
    fn report_logs(&mut self, scenario_ident: Ident, log: String);
}

impl<EL: EventListener> EventListener for Arc<Mutex<EL>> {
    fn register_scenario<C: Send + Sync + 'static>(&mut self, scenario: &ScheduledScenario<C>) {
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
pub async fn run_schedule<C: Send + Sync + 'static, EL: 'static + EventListener>(
    scenarios: Vec<ScheduledScenario<C>>,
    event_listener: EL,
) {
    let logger = setup_logging();
    let (report_logs, receive_logs) = mpsc::unbounded();

    forward_logging(receive_logs, event_listener.clone());

    let handles = scenarios
        .into_iter()
        .map(|s| {
            logger.subscribe(s.ident(), report_logs.clone());
            tokio::spawn(schedule_scenario(s, event_listener.clone()))
        })
        .collect::<Vec<JoinHandle<_>>>();

    for handle in handles {
        let _ = handle.await;
        error!("Should never stop scheduling scenario")
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

fn forward_logging<EL: 'static + EventListener>(
    mut receive_logs: UnboundedReceiver<LogLine>,
    mut logs_listener: EL,
) {
    tokio::spawn(async move {
        loop {
            let (ident, log) = receive_logs
                .next()
                .await
                .expect("There should be some logging on");
            logs_listener.report_logs(ident, log);
        }
    });
}

async fn schedule_scenario<C: Send + Sync + 'static, EL: 'static + EventListener>(
    mut scenario: ScheduledScenario<C>,
    mut event_listener: EL,
) -> impl Send {
    event_listener.register_scenario(&scenario);

    let id = scenario.ident();
    let mut interval = tokio::time::interval(scenario.interval());

    interval.tick().await; // this one is immediate
    loop {
        interval.tick().await;

        event_listener.report_launch(id.clone());
        match scenario.play().await {
            Ok(()) => event_listener.report_success(id.clone()),
            Err(_) => event_listener.report_failure(id.clone()),
        }
    }
}
