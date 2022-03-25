use iced::{executor, Application, Command, Element};

use primitives::{Ident, ScenarioDetails, ScenarioLogs};

use crate::{
    data::{fetch_logs, fetch_scenarios},
    message::Message,
    view::{LogsPage, OverviewPage},
};

/// Available dashboards.
#[derive(Clone, Debug)]
enum Route {
    /// See all the launched scenarios with their current status.
    Overview,
    /// See recent logs from particular scenario.
    Logs(Ident),
}

/// Core struct connecting state, view and update (Elm architecture).
pub struct App {
    /// Currently chosen route.
    current_route: Route,

    /// Base URL where stats from backend are exposed. Don't forget about protocol prefix
    /// (like `http://`), even if using localhost.
    stats_base_url: String,
    /// Fetched scenarios.
    scenarios: Option<Vec<ScenarioDetails>>,
    /// Fetched logs for a particular scenario.
    logs: Option<ScenarioLogs>,

    /// Overview page view.
    ///
    /// When creating views, iced often operates on object references: e.g. to display a list
    /// of scenario views, we need to keep every single view somewhere. Thus, the `App` object
    /// has to keep (perhaps indirectly) these views. However, for enhanced encapsulation, we store
    /// an `OverviewPage` object, which allows for better separation.
    overview_page: Option<OverviewPage>,
    /// Particular scenario logs page view.
    logs_page: Option<LogsPage>,
}

#[derive(Clone, Debug, Default)]
pub struct Flags {
    pub stats_base_url: String,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Flags) -> (App, Command<Message>) {
        (
            App {
                current_route: Route::Overview,
                stats_base_url: flags.stats_base_url.clone(),
                scenarios: None,
                logs: None,
                overview_page: None,
                logs_page: None,
            },
            Command::perform(
                fetch_scenarios(flags.stats_base_url),
                Message::FetchedScenarios,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Traffic maker: monitoring")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::GoToOverview => {
                self.current_route = Route::Overview;
                Command::perform(
                    fetch_scenarios(self.stats_base_url.clone()),
                    Message::FetchedScenarios,
                )
            }
            Message::GoToLogs(scenario) => {
                self.current_route = Route::Logs(scenario.clone());
                Command::perform(
                    fetch_logs(scenario, self.stats_base_url.clone()),
                    Message::FetchedLogs,
                )
            }
            Message::FetchedScenarios(result) => {
                self.scenarios = result.ok();
                Command::none()
            }
            Message::FetchedLogs(result) => {
                self.logs = result.ok();
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self.current_route {
            Route::Overview => {
                self.overview_page = Some(OverviewPage::new(self.scenarios.clone()));
                self.overview_page.as_mut().unwrap().view()
            }
            Route::Logs(ref scenario) => {
                self.logs_page = Some(LogsPage::new(scenario.clone(), self.logs.clone()));
                self.logs_page.as_mut().unwrap().view()
            }
        }
    }
}
