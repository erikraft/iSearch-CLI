# Architecture & Technologies

This page outlines the technical design, directory architecture, crate dependencies, and modular layout of **iSearch CLI™**.

---

## 🛠️ Architecture Overview

**iSearch-CLI** is constructed as a modular Rust application designed around the **Ratatui** library for immediate-mode Terminal User Interface (TUI) layout management and event handling.

```text
                        ┌────────────────────────┐
                        │      isearch-cli       │
                        └───────────┬────────────┘
                                    │
                  ┌─────────────────┴─────────────────┐
                  ▼                                   ▼
        ┌───────────────────┐               ┌───────────────────┐
        │   Native Engine   │               │  Chromium Engine  │
        └─────────┬─────────┘               └─────────┬─────────┘
                  │                                   │
      ┌───────────┼───────────┐                       │
      ▼           ▼           ▼                       ▼
 📁 Markdown   📄 PDFs    📦 ZIPs               🌐 Headless Chrome
  (pulldown)    (pdf)     (zip-rs)              (Chrome for Testing)
```

### Modular Layout

The codebase inside the `src/` directory is partitioned into five fundamental modules:

1. **`main.rs`** - The application entry point. Handles CLI argument parsing, executes immediate subcommands (like `self-update`, `version`, or `donate`), or boots into the primary interactive prompt loop.
2. **`config.rs`** - Manages parsing of TOML user configurations (`config.toml`). Holds default structures for donation presets, currencies, and local parameters.
3. **`pix.rs`** - Core business logic for EMV Co standard-compliant PIX QR code generation. Computes local CRC16 checks offline for maximum security and integrity.
4. **`utils.rs`** - Handles clipboard integrations (macOS, Windows, Linux x11/wayland) and rendering of QR codes using ASCII half-blocks.
5. **`ui.rs`** - The rendering subsystem for immediate CLI operations (like the donation screen).
6. **`browser/`** - A comprehensive subdirectory module handling the Dual-Engine TUI Browser workspace:
   - `core.rs`: Main router that determines which engine to delegate tasks to based on file system extensions, URLs, or active toggles.
   - `native.rs`: Rust native document parsers. Uses custom rendering wrappers to map Markdown, PDFs, directories, Zip paths, images, and text to crossterm text lines.
   - `chromium.rs`: Automated interface wrapper for headless browser interactions.
   - `favorites.rs`: Flat-file favorites and folders parser with JSON import/export mechanisms.
   - `history.rs`: SQLite-backed browsing database manager.
   - `theme.rs`: Immediate color configuration mappings (Dracula, Nord, Monokai, Ocean, etc.).
   - `updater.rs`: Implements the self-updating rename-replace lifecycle.
   - `release_config.rs`: Centralized release endpoints (targets `https://download.erikraft.com`).
   - `terminal_media.rs`: Probes terminal graphic protocols (Sixel, Kitty, iTerm2).
   - `ui.rs`: The TUI rendering layout loop and keyboard event capturer.

---

## 📦 Key Technologies & Crates

The app utilizes a robust selection of highly optimized, production-ready Rust packages:

* **TUI Layout & Rendering:**
  - `ratatui` (v0.29): Interactive terminal layouts, text rendering, borders, and UI states.
  - `crossterm` (v0.28): Raw mode initialization, keyboard, mouse capture, and color definitions.

* **Database & File Formats:**
  - `rusqlite` (v0.31): SQLite integration with the `bundled` feature for zero runtime configuration.
  - `serde` & `serde_json` (v1.0): Deserializing release payloads, JSON backup importing, and config tracking.
  - `toml` (v0.8): Local profile configuration parsing.
  - `zip` (v8.6): Interactive browsing of compressed structures.

* **Parsers & Protocols:**
  - `tl` (v0.7): Extremely fast HTML5 and CSS selector parsing.
  - `pulldown-cmark` (v0.13): Strict CommonMark-compliant Markdown compilation.
  - `syntect` (v5.3): High-fidelity terminal syntax highlighting.
  - `qrcode` (v0.14): Standard-compliant QR matrix translation.

* **Networking & Updates:**
  - `ureq` (v3.3): Lightweight, safe synchronous HTTP client supporting TLS integrations.

* **Platform Utilities:**
  - `arboard` (v3.4): Cross-platform native clipboard access (excluded on Android Targets).

---

## 📂 Repository Directory Layout

```text
iSearch-CLI/
├── .cargo/               # Target configs for cross-compiling SQLite
├── .github/              # Automation and release workflows (13 in total)
├── docs/                 # Offline documentation guides
│   ├── downloads.md
│   ├── installation.md
│   ├── releases.md
│   ├── self-update.md
│   └── termux.md
├── scripts/              # Distribution, installer, and web portal assets
│   ├── css/
│   ├── images/
│   ├── js/
│   ├── donation.html     # Translated donation portal web screen
│   ├── install-termux.sh # Certified mobile installer script
│   ├── installer.nsi     # NSIS Windows installer script
│   └── installer.wxs     # WiX Toolset Windows installer XML
├── src/                  # Rust Source files
│   ├── browser/          # Comprehensive terminal web browser engine
│   ├── config.rs
│   ├── main.rs
│   ├── pix.rs
│   ├── ui.rs
│   └── utils.rs
├── Cargo.toml            # Project manifest definitions
└── LICENSE               # MIT License
```
