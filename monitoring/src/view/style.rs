use iced::{Color, Length};

pub const RED: Color = Color::from_rgb(153f32 / 255.0, 0f32 / 255.0, 0f32 / 255.0);
pub const GREEN: Color = Color::from_rgb(0f32 / 255.0, 153f32 / 255.0, 0f32 / 255.0);
pub const GRAY: Color = Color::from_rgb(192f32 / 255.0, 192f32 / 255.0, 192f32 / 255.0);

#[repr(u16)]
pub enum FontSize {
    H1 = 40,
    H2 = 30,
    H3 = 25,
    Content = 20,
}

#[repr(u16)]
pub enum Spacing {
    Big = 50,
    Normal = 20,
    Small = 10,
}

pub const FULL_SCREEN_TEXT_WIDTH: Length = Length::Units(600);
pub const WIDE_COLUMN_WIDTH: Length = Length::Units(300);
