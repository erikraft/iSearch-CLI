//! Premium multi-engine interactive terminal browser module for iSearch CLI™.
//! Supports Native and headless Chromium backends.

/// Core types and interfaces for the browser system.
pub mod core;

/// The Rust-based native files/HTML/Markdown rendering engine.
pub mod native;

/// Headless Chromium engine integration.
pub mod chromium;

/// Probing and rendering mechanisms for Sixel, Kitty, and Sixel terminal media.
pub mod terminal_media;

/// Responsive TUI dashboard and user-input fields.
pub mod ui;

/// Custom color themes (Dracula, Nord, Ocean, Monokai, Light, Default).
pub mod theme;

/// Built-in and customizable browser plugins (e.g. AdBlocker).
pub mod plugins;

/// Favorites and bookmarks manager.
pub mod favorites;

/// Persistent browser visit log sqlite history database manager.
pub mod history;

/// Centralized configuration details for releases and updates.
pub mod release_config;

/// Auto update checking and self-updater logic.
pub mod updater;

pub use core::{BrowserCore, BrowserError, EngineType, PageContent};
