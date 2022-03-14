use iced::{Column, Element, Text};

use crate::{
    data::Logs,
    message::Message,
    view::style::{FontSize, Spacing, FULL_SCREEN_TEXT_WIDTH},
};

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
            .fold(
                Column::new().spacing(Spacing::Normal as u16),
                |col, logline| {
                    col.push(
                        Text::new(logline.clone())
                            .size(FontSize::Content as u16)
                            .width(FULL_SCREEN_TEXT_WIDTH),
                    )
                },
            )
            .into()
    }
}
