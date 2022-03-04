use crate::{data::Logs, message::Message};
use iced::{button, Align, Button, Column, Element, Text};

pub fn view_logs_page(_scenario_ident: String, _logs: Option<Logs>) -> Element<Message> {
    let home_button = Button::new(&mut button::State::new(), Text::new("Overview"))
        .on_press(Message::GoToOverview);

    Column::new()
        .max_width(800)
        .spacing(10)
        .padding(10)
        .align_items(Align::Center)
        .push(home_button)
        .into()
}

// match self.route {
//     Route::List => {
//         let posts: Element<_> = match self.posts {
//             None => Column::new()
//                 .push(Text::new("loading...".to_owned()).size(15))
//                 .into(),
//             Some(ref mut p) => App::render_posts(p),
//         };
//         col.push(Text::new("Home".to_owned()).size(20))
//             .push(posts)
//             .into()
//     }
//     Route::Detail(id) => {
//         let post: Element<_> = match self.post {
//             None => Column::new()
//                 .push(Text::new("loading...".to_owned()).size(15))
//                 .into(),
//             Some(ref mut p) => p.view(),
//         };
//
//         col.push(Text::new(format!("Post: {}", id)).size(20))
//             .push(post)
//             .into()
//     }
// }
