use common::ScenarioLogs;
use iced::{Column, Element, Text};

use crate::{
    message::Message,
    view::style::{FontSize, Spacing, FULL_SCREEN_TEXT_WIDTH},
};

pub struct LogsView {
    logs: ScenarioLogs,
}

impl LogsView {
    pub fn new(logs: ScenarioLogs) -> Self {
        Self { logs }
    }

    pub fn view(&self) -> Element<Message> {
        self.logs
            .content
            .iter()
            .fold(Column::new().spacing(Spacing::NORMAL), |col, logline| {
                col.push(
                    Text::new(logline.clone())
                        .size(FontSize::CONTENT)
                        .width(FULL_SCREEN_TEXT_WIDTH),
                )
            })
            .into()
    }
}
