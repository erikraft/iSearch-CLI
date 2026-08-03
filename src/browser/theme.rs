use serde::Deserialize;
use ratatui::style::Color;

#[derive(Debug, Clone, Deserialize, Copy)]
pub enum ThemePreset {
    Default,
    Dracula,
    Nord,
    Ocean,
    Monokai,
    Light,
}

#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub primary: Color,
    pub highlight: Color,
    pub border: Color,
    pub background: Color,
    pub text: Color,
    pub success: Color,
}

impl AppTheme {
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Default => Self {
                name: "Default".to_string(),
                primary: Color::Cyan,
                highlight: Color::Yellow,
                border: Color::DarkGray,
                background: Color::Reset,
                text: Color::White,
                success: Color::Green,
            },
            ThemePreset::Dracula => Self {
                name: "Dracula".to_string(),
                primary: Color::Rgb(189, 147, 249),   // Purple
                highlight: Color::Rgb(255, 121, 198), // Pink
                border: Color::Rgb(98, 114, 164),     // Comment
                background: Color::Rgb(40, 42, 54),   // Dark background
                text: Color::Rgb(248, 248, 242),      // Foreground
                success: Color::Rgb(80, 250, 123),    // Green
            },
            ThemePreset::Nord => Self {
                name: "Nord".to_string(),
                primary: Color::Rgb(129, 161, 193),   // Frost blue
                highlight: Color::Rgb(235, 203, 139), // Yellow
                border: Color::Rgb(76, 86, 106),      // Polar night
                background: Color::Rgb(46, 52, 64),   // Polar night dark
                text: Color::Rgb(236, 239, 244),      // Snow storm
                success: Color::Rgb(163, 190, 140),   // Green
            },
            ThemePreset::Ocean => Self {
                name: "Ocean".to_string(),
                primary: Color::Rgb(102, 153, 204),
                highlight: Color::Rgb(236, 92, 116),
                border: Color::Rgb(52, 61, 70),
                background: Color::Rgb(27, 30, 39),
                text: Color::Rgb(192, 197, 206),
                success: Color::Rgb(153, 199, 148),
            },
            ThemePreset::Monokai => Self {
                name: "Monokai".to_string(),
                primary: Color::Rgb(102, 217, 239),   // Cyan
                highlight: Color::Rgb(249, 38, 114),  // Pink
                border: Color::Rgb(117, 113, 94),     // Gray
                background: Color::Rgb(39, 40, 34),    // Dark gray
                text: Color::Rgb(248, 248, 242),      // White
                success: Color::Rgb(166, 226, 46),    // Green
            },
            ThemePreset::Light => Self {
                name: "Light".to_string(),
                primary: Color::Blue,
                highlight: Color::Magenta,
                border: Color::Gray,
                background: Color::White,
                text: Color::Black,
                success: Color::Green,
            },
        }
    }
}
