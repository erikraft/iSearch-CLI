# iSearch CLI™

**iSearch CLI™** is a polished terminal browser and search assistant designed for modern developers, power users, and terminal-first workflows.

## What is iSearch CLI™?

iSearch CLI™ delivers a premium terminal experience with a professional interactive user interface, rich search integration, and native support for modern command-line platforms.

- Built for **Windows, macOS, Linux, Android Termux, Docker, SSH sessions, GitHub Codespaces, VS Code terminals, JetBrains terminals, and any ANSI-compatible terminal**
- Designed for **keyboard-first navigation** with optional mouse support when available
- Supports **Unicode icons, 24-bit color, responsive layouts, and polished terminal rendering**
- Offers seamless **search, browsing, and donation workflows entirely inside the terminal**

## Features

- **Interactive TUI:** Rounded borders, responsive layout, color themes, and premium visuals.
- **Dual Engine Browser:** Powered by a super-fast Rust-based Native Engine for offline files/Markdown and an optional headless Chromium Engine for complex JS heavy pages.
- **Favorites (Bookmarks) System:** Organise bookmarks into folders, search by title/URL, import/export to JSON, and manage dynamically entirely inside the terminal.
- **SQLite History Manager:** Persistent SQL-backed browsed history database support. Group by date or domain, search and filter, sort by date or visit counts, import/export history.
- **Private Browsing (Anonymous Mode):** Browsing with absolute privacy. No history is stored, cookies/cache are fully isolated/deleted on exit, temporary in-memory downloads lists are used, and Chromium is launched in strict incognito mode.
- **PIX Donation Screen:** Standard-compliant EMV Co QR Code payloads, high-quality terminal QR rendering, and cross-platform clipboard integration.

## Installation

```bash
git clone https://github.com/erikraft/iSearch-CLI.git
cd iSearch-CLI
cargo run --release
```

For development builds:

```bash
cargo run
```

## Quick Start

Launch the interactive shell:

```bash
cargo run --release
```

Inside the iSearch CLI™ prompt, run commands such as:

```text
iSearch> browse
iSearch> donate
```

## Keyboard Shortcuts & Navigation

### General Browser Controls
- `Esc` / `Q` : Exit the browser or close popups
- `L` : Focus the address bar to enter a URL or search term
- `R` : Reload current page
- `E` : Toggle between NATIVE and CHROMIUM engines
- `T` : Open a new browser tab
- `Tab` : Switch to the next tab
- `W` : Close active tab (or rotate 3D wireframe mesh model)
- `H` : Toggle browser help screen
- `K` : Cycle through visual theme presets (Dracula, Nord, Ocean, Monokai, Light, Default)
- `P` : Download current page / file
- `Up` / `Down` : Scroll page viewport up/down

### Favorites Panel (`O`)
- `O` : Toggle Favorites Panel
- `A` : Add new favorite (brings up Title, URL, and Folder input form)
- `D` / `Delete` : Delete selected favorite
- `/` / `S` : Search favorites by title or URL
- `F` : Filter by folder (cycles through available folders)
- `I` : Import favorites from a JSON file
- `E` / `X` : Export favorites to a JSON file
- `Enter` : Navigate to the selected favorite and close the panel

### History Manager Panel (`Y`)
- `Y` : Toggle History Manager Panel
- `/` / `S` : Search history items by title or URL
- `F` : Filter history by Domain
- `R` : Toggle sorting order (Date vs Visit Count)
- `G` : Toggle grouping mode (None vs Date Grouping vs Domain Grouping)
- `D` / `Delete` : Delete selected history item from database
- `C` / `Backspace` : Clear ALL history from database
- `I` : Import history from JSON (`history_import.json`)
- `E` / `X` : Export history to JSON (`history_export.json`)
- `Enter` : Navigate to the selected history URL and close the panel

### Private / Anonymous Browsing (`V`)
- `V` : Toggle Private Browsing Mode (Anonymous Mode)
  - Displays a clear `🕵️ [PRIVATE]` visual indicator.
  - Deletes all session profiles and isolates cache/cookies.
  - Temporarily holds downloads in an in-memory list which is cleared upon closing.
  - No database history or session logs are recorded.

## Donation Support

iSearch CLI™ includes a professional terminal donation flow powered by PIX.

### Launch the donation screen

```bash
cargo run -- donate
```

From inside the interactive prompt:

```text
iSearch> donate
```

### Donation experience

- Generates a standard-compliant EMV Co QR Code payload
- Renders a high-quality QR code directly in the terminal
- Supports clipboard copy on macOS, Windows, Linux, and Termux
- Performs all formatting and validation locally for secure offline operation

## Configuration

Create or update `config.toml` with your preferences:

```toml
[donation]
pix_key = "11925416678"
currency = "BRL"
default_values = [5, 10, 20, 50, 100]
```

## Privacy & Security

- Strictly offline-first QR generation and validation
- Private/Anonymous mode with Zero persistent trails
- No telemetry or tracking
- Secure defaults for safe terminal browsing

## Author

Original author: **ErikrafT**

MIT License. See [LICENSE](LICENSE) for details.
