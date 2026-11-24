use qrcode::{Color, QrCode};
use std::io::Write;
use std::process::{Command, Stdio};

fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = data.len() as u64 * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            let offset = i * 4;
            w[i] = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut digest = [0u8; 32];
    for (i, value) in h.iter().enumerate() {
        digest[i * 4..i * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }
    digest
}

pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn checksum_for_asset(manifest: &str, asset_name: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let checksum = fields.next()?;
        let name = fields.next()?.trim_start_matches('*');
        (name == asset_name && checksum.len() == 64).then(|| checksum.to_ascii_lowercase())
    })
}

/// Detects if the current running environment is Termux on Android.
///
/// Looks up standard environment variables and local paths used by Termux.
///
/// # Returns
///
/// Returns `true` if the environment is determined to be Termux, otherwise `false`.
///
/// # Examples
///
/// ```
/// use isearch_cli::utils::is_termux;
/// let termux = is_termux();
/// ```
pub fn is_termux() -> bool {
    std::env::var("TERMUX_VERSION").is_ok()
        || std::path::Path::new("/data/data/com.termux").exists()
}

/// Runs an OS command with standard input piped into it.
///
/// # Arguments
///
/// * `cmd` - The system command name or path to execute.
/// * `args` - Arguments passed to the command.
/// * `input` - The string contents to write into the spawned process's stdin.
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` containing an error description.
///
/// # Errors
///
/// Returns an error if spawning the process fails or if writing to standard input fails.
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
    // 1. Try native clipboard via arboard first if feature is enabled
    #[cfg(feature = "clipboard")]
    {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(text.to_string()).is_ok() {
                return Ok(());
            }
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
    #[cfg(feature = "clipboard")]
    {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                return Ok(text);
            }
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

/// Renders a QR code into a terminal-compatible string using ASCII characters (`##` for dark, spaces for light).
///
/// This handles terminals that lack proper Unicode rendering capabilities.
///
/// # Arguments
///
/// * `code` - Reference to the generated [QrCode] instance.
///
/// # Returns
///
/// Returns the rendered ASCII string.
///
/// # Examples
///
/// ```
/// use qrcode::QrCode;
/// use isearch_cli::utils::render_qr_pure_ascii;
/// let code = QrCode::new(b"Hello").unwrap();
/// let rendered = render_qr_pure_ascii(&code);
/// ```
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
    fn test_sha256_and_manifest_parsing() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        let manifest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  isearch-linux-x86_64\n";
        assert_eq!(
            checksum_for_asset(manifest, "isearch-linux-x86_64").as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

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
