use qrcode::{Color, QrCode};
use std::io::Write;
use std::process::{Command, Stdio};

/// Detects if the current environment is Termux.
pub fn is_termux() -> bool {
    std::env::var("TERMUX_VERSION").is_ok()
        || std::path::Path::new("/data/data/com.termux").exists()
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
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command {} exited with non-zero status", cmd))
    }
}

/// Runs an OS command and returns its standard output.
///
/// # Arguments
///
/// * `cmd` - The system command name or path to execute.
/// * `args` - Arguments passed to the command.
///
/// # Returns
///
/// Returns the standard output on success, or `Err(String)` containing an error description.
#[cfg(target_os = "android")]
fn run_command_get_stdout(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!("Command {} exited with non-zero status", cmd))
    }
}

/// Copies the provided text slice to the user's system clipboard.
///
/// This function attempts multiple methods:
/// 1. Primary: Native APIs using the `arboard` crate.
/// 2. Secondary: Fallbacks using CLI commands depending on the platform (e.g., `pbcopy` on macOS,
///    PowerShell commands or `clip.exe` on Windows, `wl-copy`/`xclip`/`xsel` on Linux, and `termux-clipboard-set` on Android/Termux).
///
/// # Arguments
///
/// * `text` - The string slice that will be placed onto the system clipboard.
///
/// # Returns
///
/// Returns `Ok(())` if successfully copied, otherwise `Err(String)` with an error.
///
/// # Errors
///
/// Returns an error if all clipboard copy attempts and fallback tools fail.
///
/// # Examples
///
/// ```no_run
/// use isearch_cli::utils::copy_to_clipboard;
/// copy_to_clipboard("test-text-to-clipboard").unwrap();
/// ```
#[cfg(not(target_os = "android"))]
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // 1. Try native clipboard via arboard first
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if clipboard.set_text(text.to_string()).is_ok() {
            return Ok(());
        }
    }

    // 2. Platform-specific fallbacks
    if is_termux() && run_command_with_input("termux-clipboard-set", &[], text).is_ok() {
        return Ok(());
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
        if run_command_with_input(
            "powershell",
            &[
                "-Command",
                "Set-Clipboard",
                "-Value",
                &format!("'{}'", text.replace("'", "''")),
            ],
            "",
        )
        .is_ok()
        {
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

/// Copies the provided text slice to the user's system clipboard on Android.
///
/// On Android, this utilizes the Termux `termux-clipboard-set` command if available.
///
/// # Arguments
///
/// * `text` - The string slice that will be placed onto the system clipboard.
///
/// # Returns
///
/// Returns `Ok(())` if successfully copied, otherwise `Err(String)` with an error.
///
/// # Errors
///
/// Returns an error if the platform is not Termux or if the copy tool fails.
///
/// # Examples
///
/// ```no_run
/// use isearch_cli::utils::copy_to_clipboard;
/// copy_to_clipboard("test-text-to-clipboard").unwrap();
/// ```
#[cfg(target_os = "android")]
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    if is_termux() && run_command_with_input("termux-clipboard-set", &[], text).is_ok() {
        return Ok(());
    }
    Err("Clipboard copy is not supported on Android without Termux clipboard tools.".to_string())
}

/// Reads the current text from the user's system clipboard.
///
/// # Returns
///
/// Returns `Ok(String)` containing the clipboard text if successfully retrieved, otherwise `Err(String)`.
///
/// # Errors
///
/// Returns an error if reading from the clipboard fails or if the clipboard backend is unavailable.
///
/// # Examples
///
/// ```no_run
/// use isearch_cli::utils::get_from_clipboard;
/// let text = get_from_clipboard().unwrap();
/// ```
#[cfg(not(target_os = "android"))]
pub fn get_from_clipboard() -> Result<String, String> {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            return Ok(text);
        }
    }
    Err("Failed to read from clipboard.".to_string())
}

/// Reads the current text from the user's system clipboard on Android.
///
/// On Android, this utilizes the Termux `termux-clipboard-get` command if available.
///
/// # Returns
///
/// Returns `Ok(String)` containing the clipboard text if successfully retrieved, otherwise `Err(String)`.
///
/// # Errors
///
/// Returns an error if the platform is not Termux or if the get tool fails.
///
/// # Examples
///
/// ```no_run
/// use isearch_cli::utils::get_from_clipboard;
/// let text = get_from_clipboard().unwrap();
/// ```
#[cfg(target_os = "android")]
pub fn get_from_clipboard() -> Result<String, String> {
    if is_termux() {
        run_command_get_stdout("termux-clipboard-get", &[])
    } else {
        Err("Clipboard is not available on Android outside of Termux.".to_string())
    }
}

/// Renders a QR code into a terminal-compatible string using Unicode half-block characters.
///
/// This groups 2 vertical modules into 1 character block, making the rendered QR code much more
/// compact and highly scannable under standard terminal font ratios.
///
/// # Arguments
///
/// * `code` - Reference to the generated [QrCode] instance.
///
/// # Returns
///
/// Returns the rendered multi-line string containing half-block characters representing the QR code.
///
/// # Examples
///
/// ```
/// use qrcode::QrCode;
/// use isearch_cli::utils::render_qr_half_blocks;
/// let code = QrCode::new(b"Hello").unwrap();
/// let rendered = render_qr_half_blocks(&code);
/// ```
pub fn render_qr_half_blocks(code: &QrCode) -> String {
    let width = code.width();
    let mut qr_str = String::new();
    let qz = 2; // Quiet zone size
    let total_width = width + 2 * qz;
    let total_height = width + 2 * qz;

    for y_pair in 0..total_height.div_ceil(2) {
        let y_top = y_pair * 2;
        let y_bottom = y_top + 1;

        let mut line = String::new();
        for x in 0..total_width {
            let top_is_light = if y_top < qz || y_top >= width + qz || x < qz || x >= width + qz {
                true
            } else {
                code[(x - qz, y_top - qz)] == Color::Light
            };

            let bottom_is_light =
                if y_bottom < qz || y_bottom >= width + qz || x < qz || x >= width + qz {
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
pub fn generate_qr_code_for_terminal(
    payload: &str,
    term_width: u16,
) -> Result<(String, bool), String> {
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
