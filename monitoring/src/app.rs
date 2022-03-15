use iced::{executor, window, window::Mode, Application, Color, Command, Element, Length};

use crate::{
    data::{Logs, Scenario},
    message::Message,
    view::{LogsPage, OverviewPage},
};

/// Available dashboards.
#[derive(Clone, Debug)]
enum Route {
    /// See all the launched scenarios with their current status.
    Overview,
    /// See recent logs from particular scenario.
    Logs(String),
}

/// Core struct connecting state, view and update (Elm architecture).
pub struct App {
    /// Currently chosen route.
    current_route: Route,

    /// Fetched scenarios.
    scenarios: Option<Vec<Scenario>>,
    /// Fetched logs for a particular scenario.
    logs: Option<Logs>,

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
                overview_page: None,
                logs_page: None,
            },
            Command::perform(Scenario::fetch_all(), Message::FetchedScenarios),
        )
    }

    fn title(&self) -> String {
        String::from("Traffic maker: monitoring")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::GoToOverview => {
                self.current_route = Route::Overview;
                Command::perform(Scenario::fetch_all(), Message::FetchedScenarios)
            }
            Message::GoToLogs(scenario) => {
                self.current_route = Route::Logs(scenario.clone());
                Command::perform(Logs::fetch(scenario), Message::FetchedLogs)
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

    fn background_color(&self) -> Color {
        Color::from_rgb8(0x36, 0x39, 0x3F)
    }
}
