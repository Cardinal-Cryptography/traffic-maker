use iced::{
    alignment::{Horizontal, Vertical},
    button, Alignment, Button, Column, Element, Length, Row, Text,
};

use crate::{
    data::{Scenario, Status},
    message::Message,
    view::style::{AlephTheme, Color, FontSize, Spacing, WIDE_COLUMN_WIDTH},
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
            .size(FontSize::H2)
            .color(Color::PRIMARY)
            .width(WIDE_COLUMN_WIDTH);
        let title = Row::new()
            .spacing(Spacing::NORMAL)
            .width(Length::Shrink)
            .push(ident)
            .push(Self::status_icon(self.scenario.last_status.clone()));

        let fails = Text::new(format!(
            "failures: {}/{}",
            self.scenario.failures, self.scenario.runs
        ))
        .size(FontSize::CONTENT);
        let interval = Text::new(format!(
            "scheduled every {:?}s",
            self.scenario.interval.as_secs()
        ))
        .size(FontSize::CONTENT);

        let info_view = Column::new()
            .spacing(Spacing::SMALL)
            .push(title)
            .push(fails)
            .push(interval);

        let logs_button = Button::new(
            &mut self.logs_button,
            Text::new(" See logs ").size(FontSize::CONTENT),
        )
        .on_press(Message::GoToLogs(self.scenario.ident.clone()))
        .style(AlephTheme);

        Row::new()
            .padding(Spacing::SMALL)
            .align_items(Alignment::Center)
            .push(info_view)
            .push(logs_button)
            .into()
    }

    // Currently, we have to return lame text, because the combo trunk+iced is not able
    // to work with static data like icons or images. Pathetic.
    fn status_icon<'a>(status: Status) -> Element<'a, Message> {
        match status {
            Status::Success => Text::new("Status: okay").color(Color::GREEN),
            Status::Failure => Text::new("Status: not okay").color(Color::RED),
            Status::NotLaunchedYet => Text::new("Status: not launched yet").color(Color::GRAY),
            Status::Running => Text::new("Status: running").color(Color::GRAY),
        }
        .size(FontSize::H3)
        .vertical_alignment(Vertical::Center)
        .horizontal_alignment(Horizontal::Left)
        .width(WIDE_COLUMN_WIDTH)
        .into()
    }
}
