use crate::browser::terminal_media::{ColorSupport, TerminalCapabilities};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use std::io::{stdout, IsTerminal};
use std::sync::OnceLock;

const GRADIENT_STOPS: [(u8, u8, u8); 4] = [
    (0x42, 0x85, 0xF4),
    (0xDB, 0x44, 0x37),
    (0xF4, 0xB4, 0x00),
    (0x0F, 0x9D, 0x58),
];

const BRAND_TEXT: &str = "iSearch™";
const BRAND_CLI_TEXT: &str = "iSearch CLI™";

static CACHED_ISEARCH: OnceLock<String> = OnceLock::new();
static CACHED_ISEARCH_CLI: OnceLock<String> = OnceLock::new();
static CACHED_ASCII_LOGO: OnceLock<String> = OnceLock::new();

fn stdout_supports_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }

    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    stdout().is_terminal()
}

fn effective_color_support() -> ColorSupport {
    if !stdout_supports_color() {
        return ColorSupport::None;
    }

    TerminalCapabilities::detect().color_support
}

fn gradient_color_at(index: usize, total_chars: usize) -> (u8, u8, u8) {
    if total_chars <= 1 {
        return GRADIENT_STOPS[0];
    }

    let position = index as f32 / (total_chars - 1) as f32;
    let segment_count = GRADIENT_STOPS.len() - 1;
    let segment = (position * segment_count as f32).floor() as usize;
    let segment = segment.min(segment_count - 1);
    let segment_start = segment as f32 / segment_count as f32;
    let segment_end = (segment + 1) as f32 / segment_count as f32;
    let segment_position = (position - segment_start) / (segment_end - segment_start);

    let (sr, sg, sb) = GRADIENT_STOPS[segment];
    let (er, eg, eb) = GRADIENT_STOPS[segment + 1];

    let r = sr as f32 + (er as f32 - sr as f32) * segment_position;
    let g = sg as f32 + (eg as f32 - sg as f32) * segment_position;
    let b = sb as f32 + (eb as f32 - sb as f32) * segment_position;

    (r.round() as u8, g.round() as u8, b.round() as u8)
}

fn rgb_to_ansi256_code((r, g, b): (u8, u8, u8)) -> u8 {
    let r = ((r as f32 / 255.0) * 5.0).round() as u8;
    let g = ((g as f32 / 255.0) * 5.0).round() as u8;
    let b = ((b as f32 / 255.0) * 5.0).round() as u8;
    16 + 36 * r + 6 * g + b
}

fn rgb_to_basic_ansi_code((r, g, b): (u8, u8, u8)) -> u8 {
    let candidates = [
        ((0x42, 0x85, 0xF4), 34),
        ((0xDB, 0x44, 0x37), 31),
        ((0xF4, 0xB4, 0x00), 33),
        ((0x0F, 0x9D, 0x58), 32),
    ];

    let mut best = candidates[0];
    let mut best_score = u32::MAX;

    for candidate in candidates {
        let dr = r as i32 - candidate.0 .0;
        let dg = g as i32 - candidate.0 .1;
        let db = b as i32 - candidate.0 .2;
        let score = (dr * dr + dg * dg + db * db) as u32;
        if score < best_score {
            best_score = score;
            best = candidate;
        }
    }

    best.1
}

fn ansi_code_for(color_support: ColorSupport, rgb: (u8, u8, u8)) -> String {
    match color_support {
        ColorSupport::TrueColor => format!("\x1b[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2),
        ColorSupport::Ansi256 => {
            let code = rgb_to_ansi256_code(rgb);
            format!("\x1b[38;5;{}m", code)
        }
        ColorSupport::Ansi16 => {
            let code = rgb_to_basic_ansi_code(rgb);
            format!("\x1b[{}m", code)
        }
        ColorSupport::None => String::new(),
    }
}

/// Returns the branded `iSearch™` string, using terminal colors when available.
pub fn isearch() -> &'static str {
    CACHED_ISEARCH.get_or_init(|| gradient(BRAND_TEXT)).as_str()
}

/// Returns the branded `iSearch CLI™` string, using terminal colors when available.
pub fn isearch_cli() -> &'static str {
    CACHED_ISEARCH_CLI
        .get_or_init(|| gradient(BRAND_CLI_TEXT))
        .as_str()
}

/// Returns the ASCII art logo for the application, with colors applied when supported.
pub fn ascii_logo() -> &'static str {
    CACHED_ASCII_LOGO.get_or_init(|| {
        let banner = r#"
                                                                                                                        
    ██       ▄▄▄▄                                            ▄▄                     ▄▄▄▄   ▄▄         ▄▄▄▄▄▄  ▄▄▄ ▄▄ ▄▄ 
    ▀▀     ▄█▀▀▀▀█                                           ██                   ██▀▀▀▀█  ██         ▀▀██▀▀   █  █▀▄▀█ 
  ████     ██▄        ▄████▄    ▄█████▄   ██▄████   ▄█████▄  ██▄████▄            ██▀       ██           ██     █  █ ▀ █ 
    ██      ▀████▄   ██▄▄▄▄██   ▀ ▄▄▄██   ██▀      ██▀    ▀  ██▀   ██            ██        ██           ██              
    ██          ▀██  ██▀▀▀▀▀▀  ▄██▀▀▀██   ██       ██        ██    ██            ██▄       ██           ██              
 ▄▄▄██▄▄▄  █▄▄▄▄▄█▀  ▀██▄▄▄▄█  ██▄▄▄███   ██       ▀██▄▄▄▄█  ██    ██             ██▄▄▄▄█  ██▄▄▄▄▄▄   ▄▄██▄▄            
 ▀▀▀▀▀▀▀▀   ▀▀▀▀▀      ▀▀▀▀▀    ▀▀▀▀ ▀▀   ▀▀         ▀▀▀▀▀   ▀▀    ▀▀               ▀▀▀▀   ▀▀▀▀▀▀▀▀   ▀▀▀▀▀▀            
                                                                                                                        
                                                                                                                        
"#;
        gradient(banner)
    }).as_str()
}

/// Applies the official brand gradient to arbitrary text.
pub fn gradient(text: &str) -> String {
    let color_support = effective_color_support();
    if color_support == ColorSupport::None {
        return text.to_string();
    }

    let total_chars = text.chars().count();
    let mut output = String::with_capacity(text.len() * 16);

    for (index, character) in text.chars().enumerate() {
        let rgb = gradient_color_at(index, total_chars);
        output.push_str(&ansi_code_for(color_support, rgb));
        output.push(character);
    }

    output.push_str("\x1b[0m");
    output
}

/// Returns a themed style for terminal UI headings and branded titles.
pub fn brand_style() -> Style {
    Style::default()
        .fg(Color::Rgb(
            GRADIENT_STOPS[0].0,
            GRADIENT_STOPS[0].1,
            GRADIENT_STOPS[0].2,
        ))
        .add_modifier(Modifier::BOLD)
}

/// Builds a vector of gradient spans for Ratatui terminals.
pub fn gradient_spans(text: &str) -> Vec<Span<'static>> {
    let total_chars = text.chars().count();
    if total_chars == 0 {
        return vec![Span::raw(text.to_string())];
    }

    let mut spans = Vec::with_capacity(total_chars);
    for (index, character) in text.chars().enumerate() {
        let (r, g, b) = gradient_color_at(index, total_chars);
        spans.push(Span::styled(
            character.to_string(),
            Style::default()
                .fg(Color::Rgb(r, g, b))
                .add_modifier(Modifier::BOLD),
        ));
    }
    spans
}
