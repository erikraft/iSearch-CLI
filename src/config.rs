//! Configuration management module for iSearch CLI™.
//!
//! # Architecture and Responsibility
//! This module parses and provides configuration settings for the CLI application,
//! specifically for donation-related services (e.g., PIX payment key and default currencies).
//! It attempts to read from local paths and falls back to default values when configuration
//! files are absent or malformed.
//!
//! # Interactions
//! The configuration defined here is primarily used by the [crate::ui::run_donation_tui] function
//! to render donation prompts and generate valid QR payloads using configuration values.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration details specifically for donation functionality inside the terminal.
///
/// Contains information required to generate EMV Co PIX static QR payloads, including the receiver's key,
/// currency code, and standard suggested amounts.
///
/// # Fields
/// * `pix_key` - The recipient's PIX key (often a phone number, email, CPF, or random key).
/// * `currency` - The currency code (e.g., "BRL").
/// * `default_values` - List of standard donation amounts (e.g., `[5, 10, 20, 50, 100]`).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DonationConfig {
    /// The PIX address/key of the receiver (e.g., a phone number, CPF, email, or UUID).
    pub pix_key: String,
    /// The currency of transaction, usually "BRL".
    pub currency: String,
    /// Preset options of donation values shown in the user interface.
    pub default_values: Vec<u32>,
}

/// The main application configuration wrapper for iSearch CLI™.
///
/// Holds sub-configurations such as [DonationConfig].
///
/// # Example
///
/// ```
/// use isearch_cli::config::AppConfig;
/// let config = AppConfig::default();
/// assert_eq!(config.donation.currency, "BRL");
/// ```
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrowserBackendConfig {
    pub selected: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    /// Settings for donation payloads and TUI screen presets.
    pub donation: DonationConfig,
    /// Optional user-selected Chromium-compatible browser backend.
    #[serde(default)]
    pub browser_backend: BrowserBackendConfig,
}

impl Default for AppConfig {
    /// Generates the default fallback [AppConfig].
    ///
    /// # Returns
    ///
    /// Returns an instance of [AppConfig] pre-configured with default developer keys and currencies.
    fn default() -> Self {
        Self {
            donation: DonationConfig {
                pix_key: "11925416678".to_string(),
                currency: "BRL".to_string(),
                default_values: vec![5, 10, 20, 50, 100],
            },
            browser_backend: BrowserBackendConfig { selected: None },
        }
    }
}

/// Loads the application configuration from a list of standard local TOML files.
///
/// Looks for configuration files in the current working directory in the following order:
/// 1. `config.toml`
/// 2. `isearch.toml`
/// 3. `.isearch.toml`
///
/// If none of these files are found, or if they cannot be successfully parsed as valid TOML matching the
/// schema, it falls back to using [AppConfig::default()].
///
/// # Returns
///
/// Returns the parsed [AppConfig] or the default config as a fallback.
///
/// # Example
///
/// ```no_run
/// use isearch_cli::config::load_config;
/// let config = load_config();
/// println!("Current PIX key: {}", config.donation.pix_key);
/// ```
pub fn load_config() -> AppConfig {
    // Try to load from "config.toml" in current working directory first.
    let paths = ["config.toml", "isearch.toml", ".isearch.toml"];
    for path_str in paths.iter() {
        let path = Path::new(path_str);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.donation.pix_key, "11925416678");
        assert_eq!(config.donation.currency, "BRL");
        assert_eq!(config.donation.default_values, vec![5, 10, 20, 50, 100]);
        assert!(config.browser_backend.selected.is_none());
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
[donation]
pix_key = "98765432100"
currency = "USD"
default_values = [1, 2, 3]

[browser_backend]
selected = "Google Chrome"
"#;
        let config: AppConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.donation.pix_key, "98765432100");
        assert_eq!(config.donation.currency, "USD");
        assert_eq!(config.donation.default_values, vec![1, 2, 3]);
        assert_eq!(
            config.browser_backend.selected.as_deref(),
            Some("Google Chrome")
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserBackendCandidate {
    pub name: String,
    pub path: std::path::PathBuf,
}

pub fn detect_browser_backends() -> Vec<BrowserBackendCandidate> {
    let candidates: &[(&str, &[&str])] = &[
        (
            "Google Chrome",
            &["google-chrome", "google-chrome-stable", "chrome"],
        ),
        ("Chromium", &["chromium", "chromium-browser"]),
        ("Microsoft Edge", &["microsoft-edge", "msedge"]),
        ("Brave", &["brave-browser", "brave"]),
        ("Opera", &["opera"]),
        ("Vivaldi", &["vivaldi"]),
    ];
    let mut found = Vec::new();
    for (name, bins) in candidates {
        for bin in *bins {
            if let Some(path) = find_on_path(bin) {
                found.push(BrowserBackendCandidate {
                    name: (*name).into(),
                    path,
                });
                break;
            }
        }
    }
    found
}

fn find_on_path(bin: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(bin))
            .find(|p| p.exists())
    })
}
