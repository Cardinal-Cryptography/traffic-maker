use iced::{button, scrollable, Alignment, Button, Column, Element, Length, Scrollable, Text};

use crate::{
    data::Logs,
    message::Message,
    view::{
        logs::LogsView,
        style::{FontSize, Spacing},
    },
};

pub struct LogsPage {
    scenario: String,
    log_view: Option<LogsView>,

    home_button_state: button::State,
    scroll_state: scrollable::State,
}

impl LogsPage {
    pub fn new(scenario: String, logs: Option<Logs>) -> Self {
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
            Text::new(" Go back to overview ").size(FontSize::H3 as u16),
        )
        .on_press(Message::GoToOverview);

        let title = Text::new(&self.scenario).size(FontSize::H1 as u16);

        let content = Column::new()
            .spacing(Spacing::Big as u16)
            .padding(Spacing::Big as u16)
            .push(home_button)
            .push(title)
            .push(Self::content(&self.log_view));

        Scrollable::new(&mut self.scroll_state)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .padding(Spacing::Big as u16)
            .push(content)
            .into()
    }

    fn content(logs_view: &Option<LogsView>) -> Element<Message> {
        match logs_view {
            None => Text::new("No logs available")
                .size(FontSize::H2 as u16)
                .into(),
            Some(ref logs_view) => logs_view.view(),
        }
    }
}
