use iced::{Application, Result, Settings};

use crate::app::App;

mod app;
mod data;
mod message;
mod view;

pub fn main() -> Result {
    App::run(Settings::default())
}
