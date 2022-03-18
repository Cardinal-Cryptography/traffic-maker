use crate::app::App;
use iced::{Application, Result, Settings};

mod app;
mod data;
mod message;
mod view;

pub fn main() -> Result {
    App::run(Settings::default())
}
