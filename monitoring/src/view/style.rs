use iced::{Color, Length};

pub const RED: Color = Color::from_rgb(153f32 / 255.0, 0f32 / 255.0, 0f32 / 255.0);
pub const GREEN: Color = Color::from_rgb(0f32 / 255.0, 153f32 / 255.0, 0f32 / 255.0);
pub const GRAY: Color = Color::from_rgb(192f32 / 255.0, 192f32 / 255.0, 192f32 / 255.0);

pub struct FontSize;
impl FontSize {
    pub const H1: u16 = 40;
    pub const H2: u16 = 30;
    pub const H3: u16 = 25;
    pub const CONTENT: u16 = 20;
}

pub struct Spacing;
impl Spacing {
    pub const BIG: u16 = 50;
    pub const NORMAL: u16 = 20;
    pub const SMALL: u16 = 10;
}

pub const FULL_SCREEN_TEXT_WIDTH: Length = Length::Units(600);
pub const WIDE_COLUMN_WIDTH: Length = Length::Units(300);
