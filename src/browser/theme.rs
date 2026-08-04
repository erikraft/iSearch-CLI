//! Styling configurations and presets for the terminal browser interface.
//!
//! # Purpose
//! This module defines color layouts and theme structures to allow responsive rendering
//! of the TUI browser interface across various color depth capabilities.
//!
//! # Architecture & Responsibilities
//! * [ThemePreset] lists predefined styles.
//! * [AppTheme] wraps exact `ratatui::style::Color` mappings for frame headers, tabs, borders, and text components.

use serde::Deserialize;
use ratatui::style::Color;

/// Enum representing the available built-in color style themes.
#[derive(Debug, Clone, Deserialize, Copy)]
pub enum ThemePreset {
    /// Standard Cyan/Yellow theme utilizing terminal colors without high-depth RGB requirements.
    Default,
    /// Rich color configuration styled after the popular Dracula workspace colors.
    Dracula,
    /// Cold arctic palette featuring frosty blues and polar night darks.
    Nord,
    /// Deep dark ocean palette with soft blues and rose highlights.
    Ocean,
    /// High-contrast retro pastel colors with pink highlighting on dark gray.
    Monokai,
    /// High-contrast black on white configuration optimized for daytime visibility.
    Light,
}

/// Structured color specification used to draw frames, status bars, and highlighted regions.
///
/// # Fields
/// * `name` - String label of the active color scheme.
/// * `primary` - Accent color for active borders, focused highlights, and primary banners.
/// * `highlight` - Visual highlight color for highlighted items or tabs.
/// * `border` - Color of standard, non-active layout separators.
/// * `background` - Base terminal background color override.
/// * `text` - Standard text color.
/// * `success` - Accent color indicating successful actions.
#[derive(Debug, Clone)]
pub struct AppTheme {
    /// String identifier for this theme.
    pub name: String,
    /// Primary highlight/accent color.
    pub primary: Color,
    /// Highlight color for alerts or selected components.
    pub highlight: Color,
    /// Frame border base color.
    pub border: Color,
    /// Terminal widget background override.
    pub background: Color,
    /// Default body text color.
    pub text: Color,
    /// Color indicating confirmation, download completions, or matches.
    pub success: Color,
}

impl AppTheme {
    /// Factory constructor generating the exact [AppTheme] structural parameters corresponding to a [ThemePreset].
    ///
    /// # Arguments
    ///
    /// * `preset` - The enum specifying the desired style target.
    ///
    /// # Returns
    ///
    /// Returns the fully initialized [AppTheme] ready for rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::theme::{AppTheme, ThemePreset};
    /// let theme = AppTheme::from_preset(ThemePreset::Nord);
    /// assert_eq!(theme.name, "Nord");
    /// ```
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
