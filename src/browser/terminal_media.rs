use base64::Engine;
use image::{DynamicImage, GenericImageView};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalGraphicsProtocol {
    Kitty,
    Sixel,
    ITerm2,
    HalfBlocks,
    Braille,
    Ascii,
}

pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub graphics_protocol: TerminalGraphicsProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    TrueColor, // 24-bit
    Ansi256,
    Ansi16,
    None,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let color_support = Self::detect_color_support();
        let graphics_protocol = Self::detect_graphics_protocol();
        Self {
            color_support,
            graphics_protocol,
        }
    }

    fn detect_color_support() -> ColorSupport {
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorSupport::TrueColor;
            }
        }

        let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();
        if term_program == "apple_terminal" {
            return ColorSupport::Ansi256;
        }
        if term_program == "vscode" || term_program == "ghostty" || term_program == "wezterm" {
            return ColorSupport::TrueColor;
        }

        if let Ok(term) = env::var("TERM") {
            if term.contains("truecolor")
                || term.contains("24bit")
                || term.contains("kitty")
                || term.contains("alacritty")
                || term.contains("ghostty")
            {
                return ColorSupport::TrueColor;
            }
            if term.contains("256color") {
                return ColorSupport::Ansi256;
            }
            if term.contains("color") || term == "xterm" || term == "screen" {
                return ColorSupport::Ansi16;
            }
        }

        ColorSupport::Ansi16 // fallback default
    }

    fn detect_graphics_protocol() -> TerminalGraphicsProtocol {
        let term = env::var("TERM").unwrap_or_default().to_lowercase();
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();

        // 1. Kitty protocol detection
        if term.contains("kitty") || term_program.contains("kitty") {
            return TerminalGraphicsProtocol::Kitty;
        }

        // 2. iTerm2 image protocol detection (iTerm2, WezTerm, Ghostty all support this)
        if term_program.contains("iterm")
            || term_program.contains("wezterm")
            || term_program.contains("ghostty")
        {
            return TerminalGraphicsProtocol::ITerm2;
        }

        // 3. Sixel detection
        if term.contains("mlterm") || term.contains("foot") || term.contains("sixel") {
            return TerminalGraphicsProtocol::Sixel;
        }

        // Default high-fidelity Unicode blocks is the most portable high-density fallback!
        TerminalGraphicsProtocol::HalfBlocks
    }
}

pub fn detect_graphics_support() -> String {
    let caps = TerminalCapabilities::detect();
    format!("{:?}", caps.graphics_protocol)
}

// Render an image into terminal Lines based on protocol capability
pub fn render_image_to_lines(
    image_bytes: &[u8],
    term_width: u32,
    term_height: u32,
    caps: &TerminalCapabilities,
) -> Vec<Line<'static>> {
    let img = match image::load_from_memory(image_bytes) {
        Ok(i) => i,
        Err(_) => return vec![Line::raw("Failed to load image preview")],
    };

    match caps.graphics_protocol {
        TerminalGraphicsProtocol::ITerm2 => {
            // Render iTerm2 Inline Image
            let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
            let esc = format!(
                "\x1b]1337;File=inline=1;width={}px;height={}px;size={}:{}\\x07",
                term_width * 8,
                term_height * 16,
                image_bytes.len(),
                b64
            );
            vec![Line::from(vec![Span::raw(esc)])]
        }
        TerminalGraphicsProtocol::Kitty => {
            // Render Kitty base64 chunked image
            let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
            let esc = format!("\x1b_Gf=100,a=T,t=d;{}\x1b\\", b64);
            vec![Line::from(vec![Span::raw(esc)])]
        }
        TerminalGraphicsProtocol::Sixel => {
            // Sixel generation or half blocks fallback (half blocks is extremely fast and portable)
            render_half_blocks(&img, term_width, term_height)
        }
        TerminalGraphicsProtocol::HalfBlocks => render_half_blocks(&img, term_width, term_height),
        TerminalGraphicsProtocol::Braille => render_braille(&img, term_width, term_height),
        TerminalGraphicsProtocol::Ascii => render_ascii(&img, term_width, term_height),
    }
}

// Unicode Half-Blocks Renderer (true-color 24-bit representation on terminal)
fn render_half_blocks(img: &DynamicImage, target_w: u32, target_h: u32) -> Vec<Line<'static>> {
    let resized = img.resize_exact(target_w, target_h * 2, image::imageops::FilterType::Nearest);
    let mut lines = Vec::new();

    for y in (0..resized.height()).step_by(2) {
        let mut spans = Vec::new();
        for x in 0..resized.width() {
            let p_top = resized.get_pixel(x, y);
            let p_bottom = if y + 1 < resized.height() {
                resized.get_pixel(x, y + 1)
            } else {
                p_top
            };

            let fg = Color::Rgb(p_top[0], p_top[1], p_top[2]);
            let bg = Color::Rgb(p_bottom[0], p_bottom[1], p_bottom[2]);

            // "▀" represents the top pixel, background represents the bottom pixel
            spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn render_braille(img: &DynamicImage, target_w: u32, target_h: u32) -> Vec<Line<'static>> {
    let resized = img.grayscale().resize_exact(
        target_w * 2,
        target_h * 4,
        image::imageops::FilterType::Nearest,
    );
    let mut lines = Vec::new();

    // Braille offset is 0x2800
    for y in (0..resized.height()).step_by(4) {
        let mut line_str = String::new();
        for x in (0..resized.width()).step_by(2) {
            let mut val = 0u8;
            // Map 2x4 subgrid to braille bits
            if y < resized.height() && x < resized.width() && resized.get_pixel(x, y)[0] > 127 {
                val |= 1;
            }
            if y + 1 < resized.height()
                && x < resized.width()
                && resized.get_pixel(x, y + 1)[0] > 127
            {
                val |= 2;
            }
            if y + 2 < resized.height()
                && x < resized.width()
                && resized.get_pixel(x, y + 2)[0] > 127
            {
                val |= 4;
            }
            if y < resized.height()
                && x + 1 < resized.width()
                && resized.get_pixel(x + 1, y)[0] > 127
            {
                val |= 8;
            }
            if y + 1 < resized.height()
                && x + 1 < resized.width()
                && resized.get_pixel(x + 1, y + 1)[0] > 127
            {
                val |= 16;
            }
            if y + 2 < resized.height()
                && x + 1 < resized.width()
                && resized.get_pixel(x + 1, y + 2)[0] > 127
            {
                val |= 32;
            }
            if y + 3 < resized.height()
                && x < resized.width()
                && resized.get_pixel(x, y + 3)[0] > 127
            {
                val |= 64;
            }
            if y + 3 < resized.height()
                && x + 1 < resized.width()
                && resized.get_pixel(x + 1, y + 3)[0] > 127
            {
                val |= 128;
            }

            let char_val = std::char::from_u32(0x2800 + val as u32).unwrap_or(' ');
            line_str.push(char_val);
        }
        lines.push(Line::raw(line_str));
    }

    lines
}

fn render_ascii(img: &DynamicImage, target_w: u32, target_h: u32) -> Vec<Line<'static>> {
    let resized =
        img.grayscale()
            .resize_exact(target_w, target_h, image::imageops::FilterType::Nearest);
    let chars = b" .:-=+*#%@";
    let mut lines = Vec::new();

    for y in 0..resized.height() {
        let mut row = String::new();
        for x in 0..resized.width() {
            let luma = resized.get_pixel(x, y)[0];
            let idx = (luma as usize * (chars.len() - 1)) / 255;
            row.push(chars[idx] as char);
        }
        lines.push(Line::raw(row));
    }

    lines
}
