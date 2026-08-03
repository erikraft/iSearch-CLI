use qrcode::{QrCode, Color};
use std::process::{Command, Stdio};
use std::io::Write;

/// Detects if the current environment is Termux.
pub fn is_termux() -> bool {
    std::env::var("TERMUX_VERSION").is_ok() || std::path::Path::new("/data/data/com.termux").exists()
}

/// Runs a command with input via stdin and returns if it succeeded.
fn run_command_with_input(cmd: &str, args: &[&str], input: &str) -> Result<(), String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command {} exited with non-zero status", cmd))
    }
}

/// Copies text to the clipboard using native APIs (via arboard) or system fallbacks.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // 1. Try native clipboard via arboard first
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if clipboard.set_text(text.to_string()).is_ok() {
            return Ok(());
        }
    }

    // 2. Platform-specific fallbacks
    if is_termux() {
        if run_command_with_input("termux-clipboard-set", &[], text).is_ok() {
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    {
        if run_command_with_input("pbcopy", &[], text).is_ok() {
            return Ok(());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try PowerShell clipboard command
        if run_command_with_input("powershell", &["-Command", "Set-Clipboard", "-Value", &format!("'{}'", text.replace("'", "''"))], "").is_ok() {
            return Ok(());
        }
        // Try clip.exe
        if run_command_with_input("clip", &[], text).is_ok() {
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try wl-copy (Wayland)
        if run_command_with_input("wl-copy", &[], text).is_ok() {
            return Ok(());
        }
        // Try xclip (X11)
        if run_command_with_input("xclip", &["-selection", "clipboard"], text).is_ok() {
            return Ok(());
        }
        // Try xsel (X11)
        if run_command_with_input("xsel", &["--clipboard", "--input"], text).is_ok() {
            return Ok(());
        }
    }

    Err("Clipboard fallback failed. Please copy the code manually.".to_string())
}

/// Renders a QR code using high-quality Unicode half-block characters.
/// This groups 2 vertical modules into 1 character, making it compact and highly scannable.
pub fn render_qr_half_blocks(code: &QrCode) -> String {
    let width = code.width();
    let mut qr_str = String::new();
    let qz = 2; // Quiet zone size
    let total_width = width + 2 * qz;
    let total_height = width + 2 * qz;

    for y_pair in 0..((total_height + 1) / 2) {
        let y_top = y_pair * 2;
        let y_bottom = y_top + 1;

        let mut line = String::new();
        for x in 0..total_width {
            let top_is_light = if y_top < qz || y_top >= width + qz || x < qz || x >= width + qz {
                true
            } else {
                code[(x - qz, y_top - qz)] == Color::Light
            };

            let bottom_is_light = if y_bottom < qz || y_bottom >= width + qz || x < qz || x >= width + qz {
                true
            } else {
                code[(x - qz, y_bottom - qz)] == Color::Light
            };

            let ch = match (top_is_light, bottom_is_light) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            line.push(ch);
        }
        qr_str.push_str(&line);
        qr_str.push('\n');
    }

    qr_str
}

/// Renders a QR code using pure ASCII characters (# for dark, space for light) with aspect ratio formatting.
pub fn render_qr_pure_ascii(code: &QrCode) -> String {
    let width = code.width();
    let mut qr_str = String::new();
    let qz = 2;
    let total_width = width + 2 * qz;

    for y in 0..(width + 2 * qz) {
        let mut line = String::new();
        for x in 0..total_width {
            let is_light = if y < qz || y >= width + qz || x < qz || x >= width + qz {
                true
            } else {
                code[(x - qz, y - qz)] == Color::Light
            };

            if is_light {
                line.push_str("##");
            } else {
                line.push_str("  ");
            }
        }
        qr_str.push_str(&line);
        qr_str.push('\n');
    }
    qr_str
}

/// Generates the QR Code for the terminal.
/// Automatically detects terminal width and handles fallback if terminal is too narrow.
pub fn generate_qr_code_for_terminal(payload: &str, term_width: u16) -> Result<(String, bool), String> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|e| format!("Failed to generate QR Code: {}", e))?;

    let qr_width = code.width() as u16;
    let required_width_half_blocks = qr_width + 4; // Width with 2-module quiet zone
    let required_width_ascii = (qr_width + 4) * 2; // Aspect ratio corrected ASCII uses 2 chars per module

    if term_width < required_width_half_blocks {
        return Err(format!(
            "Terminal is too narrow ({} columns). Please resize your terminal to at least {} columns to view the QR Code.",
            term_width, required_width_half_blocks
        ));
    }

    // Prefer high-quality Unicode half-blocks, otherwise fallback to ASCII if width permits or requested.
    if term_width >= required_width_half_blocks {
        Ok((render_qr_half_blocks(&code), true))
    } else if term_width >= required_width_ascii {
        Ok((render_qr_pure_ascii(&code), false))
    } else {
        Err("Terminal is too narrow to display QR Code safely.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_rendering_half_blocks() {
        let code = QrCode::new(b"test").unwrap();
        let rendered = render_qr_half_blocks(&code);
        assert!(!rendered.is_empty());
        assert!(rendered.contains('█') || rendered.contains('▄') || rendered.contains('▀'));
    }

    #[test]
    fn test_qr_rendering_ascii() {
        let code = QrCode::new(b"test").unwrap();
        let rendered = render_qr_pure_ascii(&code);
        assert!(!rendered.is_empty());
        assert!(rendered.contains("##") || rendered.contains("  "));
    }

    #[test]
    fn test_generate_for_terminal() {
        let payload = "https://example.com";
        // Wide enough
        let res = generate_qr_code_for_terminal(payload, 80);
        assert!(res.is_ok());
        let (rendered, is_unicode) = res.unwrap();
        assert!(!rendered.is_empty());
        assert!(is_unicode);

        // Too narrow
        let narrow_res = generate_qr_code_for_terminal(payload, 5);
        assert!(narrow_res.is_err());
    }
}
