use iced::{
    button, scrollable, Alignment, Button, Column, Container, Element, Length, Scrollable, Text,
};
use primitives::{Ident, ScenarioLogs};

use crate::{
    message::Message,
    view::{
        logs::LogsView,
        style::{AlephTheme, Color, FontSize, Spacing},
    },
};

pub struct LogsPage {
    scenario: Ident,
    log_view: Option<LogsView>,

    home_button_state: button::State,
    scroll_state: scrollable::State,
}

impl LogsPage {
    pub fn new(scenario: Ident, logs: Option<ScenarioLogs>) -> Self {
        LogsPage {
            scenario,
            log_view: logs.map(LogsView::new),
            home_button_state: button::State::new(),
            scroll_state: scrollable::State::new(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let home_button = Button::new(
            &mut self.home_button_state,
            Text::new(" Go back to overview ").size(FontSize::H3),
        )
        .on_press(Message::GoToOverview)
        .style(AlephTheme);

        let title = Text::new(&self.scenario.0)
            .size(FontSize::H1)
            .color(Color::PRIMARY);

        let content = Column::new()
            .spacing(Spacing::BIG)
            .padding(Spacing::BIG)
            .push(home_button)
            .push(title)
            .push(Self::content(&self.log_view));

        let content = Scrollable::new(&mut self.scroll_state)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .padding(Spacing::BIG)
            .push(content);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(AlephTheme)
            .into()
    }

    fn content(logs_view: &Option<LogsView>) -> Element<Message> {
        match logs_view {
            None => Text::new("No logs available").size(FontSize::H2).into(),
            Some(ref logs_view) => logs_view.view(),
        }
    }
}
