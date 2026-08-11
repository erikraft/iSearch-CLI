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

/// Complete catalog of errors thrown during browsing, fetching, and standalone parsing routines.
#[derive(Debug, Clone)]
pub enum BrowserError {
    /// Failure resolving host addresses or downloading content.
    NetworkError(String),
    /// Failure parsing syntactical blocks like raw HTML, PDF, or markdown structures.
    ParsingError(String),
    /// Headless Chromium binary is absent or unconfigured.
    ChromiumNotAvailable(String),
    /// Portable ChromeStandalone package download or extraction failures.
    DownloadError(String),
    /// Platform is fundamentally incompatible (e.g. headless chromium on Android Termux).
    UnsupportedPlatform(String),
    /// General system input/output disk failures.
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
    /// Adapts standard [std::io::Error] types into [BrowserError::IoError] variants.
    fn from(err: std::io::Error) -> Self {
        BrowserError::IoError(err.to_string())
    }
}

/// Identifies which engine is actively delegated to resolve navigation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    /// Safe, offline-compatible native parser targeting local files, markdown, zip, and pdf.
    Native,
    /// headful/headless Chrome emulator executing javascript and producing DOM snapshots.
    Chromium,
}

/// Richly structured representation containing parsed page bodies or directory information.
#[derive(Debug, Clone)]
pub enum PageContent {
    /// Renderable tree consisting of parsed structural HTML elements and headers.
    Html {
        /// Standard resolved title of the document.
        title: String,
        /// Original raw HTML body data.
        raw_html: String,
        /// Node structures prepared for visual rendering.
        parsed_nodes: Vec<crate::browser::native::HtmlNode>,
    },
    /// Renderable Markdown structures parsed using `pulldown-cmark`.
    Markdown {
        /// File name or short descriptive title.
        title: String,
        /// Original Markdown text.
        raw_md: String,
    },
    /// Standard filesystem directory hierarchy view.
    Directory {
        /// Absolute directory path.
        path: PathBuf,
        /// Array of subpaths paired with direct folder status flags.
        entries: Vec<(String, bool)>,
    },
    /// Low-level text representation optimized for terminal viewports.
    FilePreview {
        /// System path.
        path: PathBuf,
        /// File body contents.
        content: String,
        /// binary flag to avoid printing unformatted control characters.
        is_binary: bool,
    },
    /// Text extraction preview and parsed metadata for portable document formats.
    PdfPreview {
        /// Target PDF file location.
        path: PathBuf,
        /// Title extracted from PDF catalog metadata.
        title: String,
        /// Complete metadata records parsed.
        metadata: Vec<(String, String)>,
        /// Number of renderable pages inside document.
        pages_count: usize,
        /// Body text extracted from pages.
        text_preview: String,
    },
    /// Explorable file listing for compressed ZIP directories.
    ArchivePreview {
        /// Absolute path to ZIP archive.
        path: PathBuf,
        /// Internal files lists.
        files: Vec<String>,
    },
    /// Dynamic raw image representation designed for image rendering protocols.
    ImagePreview {
        /// Absolute path of source image file.
        path: PathBuf,
        /// Raw file bytes.
        raw_bytes: Vec<u8>,
    },
    /// Pre-decorated text formatted with standard ANSI control sequences.
    AnsiText {
        /// Header title.
        title: String,
        /// Plain text body content.
        content: String,
    },
    /// Live 3D mesh vertex and face representation.
    Mesh3DPreview {
        /// Frame title.
        title: String,
        /// The parsed 3D mesh structure.
        mesh: crate::browser::native::Mesh3D,
    },
}

/// Decoupled interface contract specifying baseline capabilities for browser implementations.
pub trait BrowserEngine {
    /// Navigates to a specific address, returning structured [PageContent].
    ///
    /// # Arguments
    ///
    /// * `url` - Hyperlink target.
    fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError>;

    /// Executes query search keywords on search engines.
    ///
    /// # Arguments
    ///
    /// * `query` - Keywords.
    fn search(&mut self, query: &str) -> Result<PageContent, BrowserError>;
    fn capture_screenshot(
        &mut self,
        url: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, BrowserError>;
}

/// Top-level unified orchestrator coordinating backends, plugins, and caching pipelines.
///
/// Handles fallback mechanisms and active engine redirection seamlessly.
///
/// # Fields
/// * `current_engine` - Flag specifying which engine to use primarily.
/// * `native_engine` - Native offline browser engine container.
/// * `chromium_engine` - Ported headless chromium browser instance.
/// * `adblocker` - Adblocking domain filter plugin.
pub struct BrowserCore {
    /// Currently delegated browser engine.
    pub current_engine: EngineType,
    /// Native engine instance.
    pub native_engine: crate::browser::native::NativeEngine,
    /// Automated chromium engine instance.
    pub chromium_engine: crate::browser::chromium::ChromiumEngine,
    /// Adblocker filtering controller.
    pub adblocker: crate::browser::plugins::AdBlocker,
}

impl Default for BrowserCore {
    /// Generates default [BrowserCore].
    ///
    /// # Returns
    ///
    /// Returns default instance.
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserCore {
    /// Initializes core browser engines and registers default plugins.
    ///
    /// # Returns
    ///
    /// Returns initialized [BrowserCore].
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::core::BrowserCore;
    /// let core = BrowserCore::new();
    /// ```
    pub fn new() -> Self {
        Self {
            current_engine: EngineType::Native,
            native_engine: crate::browser::native::NativeEngine::new(),
            chromium_engine: crate::browser::chromium::ChromiumEngine::new(),
            adblocker: crate::browser::plugins::AdBlocker::new(),
        }
    }

    /// Toggles the primary active engine target.
    ///
    /// # Arguments
    ///
    /// * `engine` - Target engine type.
    pub fn set_engine(&mut self, engine: EngineType) {
        self.current_engine = engine;
    }

    /// Evaluates adblocking, selects the active engine, and navigates to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - Destination web link or local file system path.
    ///
    /// # Returns
    ///
    /// Returns parsed [PageContent] on success, or [BrowserError].
    ///
    /// # Errors
    ///
    /// Returns an error if network requests fail or parsing errors occur.
    pub fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError> {
        let parsed_uri = crate::uri::parse(url);
        match parsed_uri.scheme {
            crate::uri::UriScheme::Ws | crate::uri::UriScheme::Wss => return Ok(PageContent::AnsiText { title: "WebSocket".into(), content: format!("WebSocket resource\n════════════════════\n\n{}\n\nInteractive message viewing is supported by the URI router. Received messages are treated as untrusted data and are never executed as system commands.", parsed_uri.normalized) }),
            crate::uri::UriScheme::Ftp | crate::uri::UriScheme::Ftps => return Ok(PageContent::AnsiText { title: "FTP".into(), content: format!("FTP/FTPS resource\n═════════════════\n\n{}\n\nListing, download, and upload are routed through the dedicated FTP handler when credentials and backend support are configured. Credentials are never logged.", parsed_uri.normalized) }),
            crate::uri::UriScheme::Sftp => return Ok(PageContent::AnsiText { title: "SFTP".into(), content: format!("SFTP resource\n═════════════\n\n{}\n\nSFTP operations require SSH authentication, key handling, and known_hosts verification. Host verification is not disabled by default.", parsed_uri.normalized) }),
            crate::uri::UriScheme::Smtp | crate::uri::UriScheme::Smtps => return Ok(PageContent::AnsiText { title: "SMTP".into(), content: format!("SMTP/SMTPS resource\n══════════════════\n\n{}\n\nSMTP is recognized as mail transport, not HTTP. Passwords, tokens, and credentials are never logged.", parsed_uri.normalized) }),
            crate::uri::UriScheme::Blob => return Ok(PageContent::AnsiText { title: "Blob URL".into(), content: "Blob URL\n\nThis resource requires the original browser context to be resolved.\n\n[Open in browser]\n[Back]".into() }),
            crate::uri::UriScheme::Data => { let data = crate::uri::parse_data_uri(url).map_err(BrowserError::ParsingError)?; return Ok(PageContent::FilePreview { path: PathBuf::from(format!("data-uri.{}", data.mime_type.replace('/', "_"))), content: format!("Data URI\n════════\n\nMIME: {}\nBase64: {}\nSize: {} bytes\n\n{}", data.mime_type, data.is_base64, data.bytes.len(), String::from_utf8_lossy(&data.bytes)), is_binary: !data.mime_type.starts_with("text/") }); },
            crate::uri::UriScheme::File | crate::uri::UriScheme::LocalPath => if let Some(path) = parsed_uri.local_path { return self.native_engine.navigate(&path.to_string_lossy()); },
            _ => {}
        }
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
