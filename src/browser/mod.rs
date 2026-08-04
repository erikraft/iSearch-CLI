//! Premium terminal browser engine module of iSearch CLI™.
//!
//! # Architecture & Purpose
//! The browser module is designed as a modular sub-system supporting multiple engines:
//! * A lightweight, fast offline/Native engine ([native]) that parses files, Markdown, PDFs, and ZIPs.
//! * An automated portable headless browser backend ([chromium]) targeting complex JS web apps.
//!
//! # Interactions
//! All client interfaces communicate through the unified API layer [BrowserCore] in the [core] module,
//! utilizing [theme] styling, [plugins] (like AdBlocking), [favorites] collections, [history] SQL databases,
//! and [terminal_media] imaging.

pub mod core;
pub mod native;
pub mod chromium;
pub mod terminal_media;
pub mod ui;
pub mod theme;
pub mod plugins;
pub mod favorites;
pub mod history;

pub use core::{BrowserCore, EngineType, PageContent, BrowserError};
