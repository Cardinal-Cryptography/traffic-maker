use crate::{
    data::{Scenario, Status},
    message::Message,
};

use crate::view::{GRAY, GREEN, RED};
use iced::{
    alignment::{Horizontal, Vertical},
    button, Alignment, Button, Column, Element, Length, Row, Text,
};

#[derive(Clone)]
pub struct ScenarioView {
    scenario: Scenario,
    logs_button: button::State,
}

impl ScenarioView {
    pub fn new(scenario: Scenario) -> Self {
        ScenarioView {
            scenario,
            logs_button: button::State::new(),
        }
    }

    pub fn view_in_list(&mut self) -> Element<Message> {
        let ident = Text::new(self.scenario.ident.clone())
            .size(30)
            .width(Length::Units(300));
        let title = Row::new()
            .spacing(15)
            .width(Length::Shrink)
            .push(ident)
            .push(Self::status_icon(self.scenario.last_status.clone()));

        let fails = Text::new(format!(
            "failures: {}/{}",
            self.scenario.failures, self.scenario.runs
        ))
        .size(20);
        let interval = Text::new(format!(
            "scheduled every {:?}s",
            self.scenario.interval.as_secs()
        ))
        .size(20);

        let info_view = Column::new()
            .spacing(10)
            .push(title)
            .push(fails)
            .push(interval);

        let logs_button = Button::new(&mut self.logs_button, Text::new(" See logs ").size(20))
            .on_press(Message::GoToLogs(self.scenario.ident.clone()));

        Row::new()
            .padding(10)
            .align_items(Alignment::Center)
            .push(info_view)
            .push(logs_button)
            .into()
    }

    // Currently, we have to return lame text, because the combo trunk+iced is not able
    // to work with static data like icons or images. Pathetic.
    fn status_icon<'a>(status: Option<Status>) -> Element<'a, Message> {
        let status_size = 25;
        match status {
            None => Text::new("Status: unknown").color(GRAY),
            Some(Status::Success) => Text::new("Status: okay").color(GREEN),
            Some(Status::Failure) => Text::new("Status: not okay").color(RED),
        }
        .size(status_size)
        .vertical_alignment(Vertical::Center)
        .horizontal_alignment(Horizontal::Left)
        .width(Length::Units(300))
        .into()
    }
}
