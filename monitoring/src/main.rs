use iced::{
    button, executor, Align, Application, Button, Clipboard, Column, Command, Element, Row,
    Settings, Text, VerticalAlignment,
};

mod app;
mod data;
mod message;
mod view;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

impl App {
    fn render_posts(posts: &mut Vec<Post>) -> Element<Message> {
        let c = Column::new();
        let posts: Element<_> = posts
            .iter_mut()
            .fold(Column::new().spacing(10), |col, p| {
                col.push(p.view_in_list())
            })
            .into();
        c.push(posts).into()
    }
}

struct Post {
    detail_button: button::State,
    post: data::Post,
}

impl Post {
    fn view(&mut self) -> Element<Message> {
        Column::new()
            .push(Text::new(format!("id: {}", self.post.id)).size(12))
            .push(Text::new(format!("user_id: {}", self.post.user_id)).size(12))
            .push(Text::new(format!("title: {}", self.post.title)).size(12))
            .push(Text::new(self.post.body.to_owned()).size(12))
            .into()
    }

    fn view_in_list(&mut self) -> Element<Message> {
        let r = Row::new().padding(5).spacing(5);
        r.push(
            Column::new().spacing(5).push(
                Text::new(self.post.title.to_owned())
                    .size(12)
                    .vertical_alignment(VerticalAlignment::Center),
            ),
        )
        .push(
            Button::new(&mut self.detail_button, Text::new("Detail").size(12))
                .on_press(Message::GoToDetail(self.post.id)),
        )
        .into()
    }
}
