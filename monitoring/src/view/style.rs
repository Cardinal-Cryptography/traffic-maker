use iced::{button, container, container::Style, Color as IcedColor, Length};

pub struct Color;

impl Color {
    pub const RED: IcedColor = Self::convert(153, 0, 0);
    pub const GREEN: IcedColor = Self::convert(0, 153, 0);
    pub const GRAY: IcedColor = Self::convert(192, 192, 192);

    pub const BACKGROUND: IcedColor = Self::convert(54, 57, 63);

    const fn convert(r: u8, g: u8, b: u8) -> IcedColor {
        IcedColor::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
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
pub const WIDE_COLUMN_WIDTH: Length = Length::Units(300);

pub struct AlephTheme;

impl container::StyleSheet for AlephTheme {
    fn style(&self) -> Style {
        container::Style {
            // background: BACKGROUND.into(),
            background: BACKGROUND.into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}

// impl button::StyleSheet for AlephTheme {
//     fn active(&self) -> button::Style {
//         todo!()
//     }
//
//     fn hovered(&self) -> button::Style {
//         todo!()
//     }
//
//     fn pressed(&self) -> button::Style {
//         todo!()
//     }
//
//     fn disabled(&self) -> button::Style {
//         todo!()
//     }
// }
// 00CCAB
