use iced::Color;
pub use logs_page::LogsPage;
pub use overview_page::OverviewPage;

mod logs;
mod logs_page;
mod overview_page;
mod scenario;

const RED: Color = Color::from_rgb(153f32 / 255.0, 0f32 / 255.0, 0f32 / 255.0);

const GREEN: Color = Color::from_rgb(0f32 / 255.0, 153f32 / 255.0, 0f32 / 255.0);

const GRAY: Color = Color::from_rgb(192f32 / 255.0, 192f32 / 255.0, 192f32 / 255.0);
