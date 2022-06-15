use iced::{Application, Result, Settings};

use crate::app::{App, Flags};

mod app;
mod data;
mod message;
mod view;

pub const DEFAULT_STATS_BASE_URL: &str = "http://localhost:8040";

pub fn main() -> Result {
    // We need to use macro to resolve this in compilation time, as during execution
    // we won't have access to the env.
    let stats_base_url = option_env!("STATS_BASE_URL")
        .unwrap_or(DEFAULT_STATS_BASE_URL)
        .to_string();

    App::run(Settings {
        flags: Flags { stats_base_url },
        ..Settings::default()
    })
}
