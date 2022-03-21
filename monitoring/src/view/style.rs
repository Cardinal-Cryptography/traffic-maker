use iced::{button, container, container::Style, Color as IcedColor, Length, Vector};

macro_rules! convert {
    ($r: expr, $g: expr, $b: expr) => {
        IcedColor::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
}

pub struct Color;
impl Color {
    pub const WHITE: IcedColor = convert!(255, 255, 255);
    pub const RED: IcedColor = convert!(153, 0, 0);
    pub const GREEN: IcedColor = convert!(0, 153, 0);
    pub const GRAY: IcedColor = convert!(192, 192, 192);
    pub const ANTHRACITE: IcedColor = convert!(17, 27, 36);

    pub const BACKGROUND: IcedColor = convert!(54, 57, 60);
    pub const PRIMARY: IcedColor = convert!(0, 234, 199);
}

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
pub const WIDE_COLUMN_WIDTH: Length = Length::Units(400);

pub struct AlephTheme;

impl container::StyleSheet for AlephTheme {
    fn style(&self) -> Style {
        container::Style {
            background: Color::BACKGROUND.into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}

impl button::StyleSheet for AlephTheme {
    fn active(&self) -> button::Style {
        button::Style {
            background: Color::PRIMARY.into(),
            text_color: Color::ANTHRACITE,
            border_radius: 5f32,
            shadow_offset: Vector::new(1.0, 1.0),
            ..button::Style::default()
        }
    }
}
