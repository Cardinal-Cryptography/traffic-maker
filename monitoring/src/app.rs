use iced::{
    button, executor, Align, Application, Button, Clipboard, Column, Command, Element, Text,
};

use crate::{
    data::{Logs, Scenario},
    message::Message,
    view::{view_logs_page, view_overview_page},
};

/// Dashboards
#[derive(Clone, Debug)]
enum Route {
    /// See all the launched scenarios with their current status.
    Overview,
    /// See recent logs from particular scenario.
    Logs(String),
}

/// Core struct connecting state, view and update (Elm architecture).
struct App {
    /// Currently chosen route.
    current_route: Route,

    /// Fetched scenarios.
    scenarios: Option<Vec<Scenario>>,
    /// Fetched logs for a particular scenario.
    logs: Option<Logs>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App {
                current_route: Route::Overview,
                scenarios: None,
                logs: None,
            },
            Command::perform(Scenario::fetch_all(), Message::FetchedScenarios),
        )
    }

    fn title(&self) -> String {
        String::from("Traffic maker: monitoring")
    }

    fn update(&mut self, message: Message, _c: &mut Clipboard) -> Command<Message> {
        match message {
            Message::GoToOverview => {
                self.current_route = Route::Overview;
                Command::perform(Scenario::fetch_all(), Message::FetchedScenarios)
            }
            Message::GoToLogs(scenario) => {
                self.current_route = Route::Logs(scenario.clone());
                Command::perform(Logs::fetch(scenario), Message::FetchedLogs())
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
            Route::Overview => view_overview_page(self.scenarios.clone()),
            Route::Logs(ref scenario) => view_logs_page(scenario.clone(), self.logs.clone()),
        }
    }
}
