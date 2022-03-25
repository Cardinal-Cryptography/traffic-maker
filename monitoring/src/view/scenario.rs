use common::{ScenarioDetails, ScenarioStatus};
use iced::{
    alignment::{Horizontal, Vertical},
    button, Alignment, Button, Column, Element, Length, Row, Text,
};

use crate::{
    message::Message,
    view::style::{AlephTheme, Color, FontSize, Spacing, WIDE_COLUMN_WIDTH},
};

#[derive(Clone)]
pub struct ScenarioView {
    scenario: ScenarioDetails,
    logs_button: button::State,
}

impl ScenarioView {
    pub fn new(scenario: ScenarioDetails) -> Self {
        ScenarioView {
            scenario,
            logs_button: button::State::new(),
        }
    }

    pub fn view_in_list(&mut self) -> Element<Message> {
        let ident = Text::new(self.scenario.ident.0.clone())
            .size(FontSize::H2)
            .color(Color::PRIMARY)
            .width(WIDE_COLUMN_WIDTH);
        let title = Row::new()
            .spacing(Spacing::NORMAL)
            .width(Length::Shrink)
            .push(ident)
            .push(Self::status_icon(self.scenario.last_status));

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
    fn status_icon<'a>(status: ScenarioStatus) -> Element<'a, Message> {
        match status {
            ScenarioStatus::Success => Text::new("Status: okay").color(Color::GREEN),
            ScenarioStatus::Failure => Text::new("Status: not okay").color(Color::RED),
            ScenarioStatus::NotLaunchedYet => {
                Text::new("Status: not launched yet").color(Color::GRAY)
            }
            ScenarioStatus::Running => Text::new("Status: running").color(Color::GRAY),
        }
        .size(FontSize::H3)
        .vertical_alignment(Vertical::Center)
        .horizontal_alignment(Horizontal::Left)
        .width(WIDE_COLUMN_WIDTH)
        .into()
    }
}
