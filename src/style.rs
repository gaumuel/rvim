use crossterm::style::Color;
use syntect::highlighting::Style;

pub const VISUAL_FG: Color = Color::White;
pub const VISUAL_BG: Color = Color::DarkBlue;
pub const CURSORLINE_BG: Color = Color::Rgb { r: 40, g: 40, b: 50 };
pub const TILDE_FG: Color = Color::DarkGrey;
pub const STATUS_FG: Color = Color::Black;
pub const STATUS_BG: Color = Color::White;
pub const THEME_NAME: &str = "base16-ocean.dark";

pub fn syntect_to_crossterm(style: Style) -> Color {
    Color::Rgb {
        r: style.foreground.r,
        g: style.foreground.g,
        b: style.foreground.b,
    }
}
