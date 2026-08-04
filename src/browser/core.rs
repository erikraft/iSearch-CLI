//! Core interfaces, common traits, types, and unified routing logic for the browser.
//!
//! # Purpose
//! This module binds other backend submodules together by exposing the unified interface [BrowserCore],
//! engine selection [EngineType], rendering schemas [PageContent], and comprehensive browser error types [BrowserError].
//!
//! # Architecture
//! * [BrowserCore] acts as the main system orchestrator, maintaining instances of [crate::browser::native::NativeEngine], [crate::browser::chromium::ChromiumEngine], and [crate::browser::plugins::AdBlocker].
//! * [BrowserEngine](crate::browser::core::BrowserEngine) declares the standard navigation and screenshot capture contract implemented by backends.
//! * [PageContent] describes a richly structured abstract representation of pages, folders, zip files, mesh models, and images.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum BrowserError {
    NetworkError(String),
    ParsingError(String),
    ChromiumNotAvailable(String),
    DownloadError(String),
    UnsupportedPlatform(String),
    IoError(String),
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserError::NetworkError(s) => write!(f, "Network error: {}", s),
            BrowserError::ParsingError(s) => write!(f, "Parsing error: {}", s),
            BrowserError::ChromiumNotAvailable(s) => write!(f, "Chromium not available: {}", s),
            BrowserError::DownloadError(s) => write!(f, "Download error: {}", s),
            BrowserError::UnsupportedPlatform(s) => write!(f, "Unsupported platform: {}", s),
            BrowserError::IoError(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl std::error::Error for BrowserError {}

impl From<std::io::Error> for BrowserError {
    fn from(err: std::io::Error) -> Self {
        BrowserError::IoError(err.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    Native,
    Chromium,
}

#[derive(Debug, Clone)]
pub enum PageContent {
    Html {
        title: String,
        raw_html: String,
        parsed_nodes: Vec<crate::browser::native::HtmlNode>,
    },
    Markdown {
        title: String,
        raw_md: String,
    },
    Directory {
        path: PathBuf,
        entries: Vec<(String, bool)>, // (name, is_dir)
    },
    FilePreview {
        path: PathBuf,
        content: String,
        is_binary: bool,
    },
    PdfPreview {
        path: PathBuf,
        title: String,
        metadata: Vec<(String, String)>,
        pages_count: usize,
        text_preview: String,
    },
    ArchivePreview {
        path: PathBuf,
        files: Vec<String>,
    },
    ImagePreview {
        path: PathBuf,
        raw_bytes: Vec<u8>,
    },
    AnsiText {
        title: String,
        content: String,
    },
    Mesh3DPreview {
        title: String,
        mesh: crate::browser::native::Mesh3D,
    },
}

pub trait BrowserEngine {
    fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError>;
    fn search(&mut self, query: &str) -> Result<PageContent, BrowserError>;
    fn capture_screenshot(
        &mut self,
        url: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, BrowserError>;
}

pub struct BrowserCore {
    pub current_engine: EngineType,
    pub native_engine: crate::browser::native::NativeEngine,
    pub chromium_engine: crate::browser::chromium::ChromiumEngine,
    pub adblocker: crate::browser::plugins::AdBlocker,
}

impl Default for BrowserCore {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserCore {
    pub fn new() -> Self {
        Self {
            current_engine: EngineType::Native,
            native_engine: crate::browser::native::NativeEngine::new(),
            chromium_engine: crate::browser::chromium::ChromiumEngine::new(),
            adblocker: crate::browser::plugins::AdBlocker::new(),
        }
    }

    pub fn set_engine(&mut self, engine: EngineType) {
        self.current_engine = engine;
    }

    pub fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError> {
        if self.adblocker.is_blocked(url) {
            return Ok(PageContent::AnsiText {
                title: "Ad Blocked".to_string(),
                content: format!(
                    "The request to '{}' was blocked by the integrated iSearch AdBlocker plugin.",
                    url
                ),
            });
        }
        match self.current_engine {
            EngineType::Native => self.native_engine.navigate(url),
            EngineType::Chromium => {
                if !self.chromium_engine.is_available() {
                    // Fall back automatically if Chromium is not available
                    return self.native_engine.navigate(url);
                }
                self.chromium_engine.navigate(url)
            }
        }
    }

    pub fn search(
        &mut self,
        query: &str,
        alias: Option<&str>,
    ) -> Result<PageContent, BrowserError> {
        let engine = alias.unwrap_or("google");
        let url = match engine {
            "google" | "g" => format!(
                "https://www.google.com/search?q={}",
                percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
            ),
            "duckduckgo" | "ddg" => format!(
                "https://duckduckgo.com/html/?q={}",
                percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
            ),
            "bing" | "b" => format!(
                "https://www.bing.com/search?q={}",
                percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
            ),
            _ => format!(
                "https://www.google.com/search?q={}",
                percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
            ),
        };
        self.navigate(&url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_core_init() {
        let mut core = BrowserCore::new();
        assert_eq!(core.current_engine, EngineType::Native);
        core.set_engine(EngineType::Chromium);
        assert_eq!(core.current_engine, EngineType::Chromium);
    }
}
