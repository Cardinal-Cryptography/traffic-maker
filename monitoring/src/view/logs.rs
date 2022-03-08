use crate::{data::Logs, message::Message};
use iced::{Column, Element, Length, Text};

pub struct LogsView {
    logs: Logs,
}

impl LogsView {
    pub fn new(logs: Logs) -> Self {
        Self { logs }
    }

    pub fn view(&self) -> Element<Message> {
        self.logs
            .content
            .iter()
            .fold(Column::new().spacing(20), |col, logline| {
                col.push(Text::new(logline.clone()).width(Length::Units(600)))
            })
            .into()
    }
}
