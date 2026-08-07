[![Dependency Integrity Check](https://github.com/erikraft/iSearch-CLI/actions/workflows/dependency-check.yml/badge.svg)](https://github.com/erikraft/iSearch-CLI/actions/workflows/dependency-check.yml)
[![Release Pipeline](https://github.com/erikraft/iSearch-CLI/actions/workflows/release.yml/badge.svg)](https://github.com/erikraft/iSearch-CLI/actions/workflows/release.yml)
```text
  _ ___                  _       ___ _    ___ 
 (_) __| ___ __ _ _ _ __| |_    / __| |  |_ _|
 | \__ \/ -_) _` | '_/ _| ' \  | (__| |__ | | 
 |_|___/\___\__,_|_| \__|_||_|  \___|____|___|
```
                                              

**iSearch CLI™** is a polished terminal browser and search assistant designed for modern developers, power users, and terminal-first workflows.

---

## Project Branding

- **Official Name:** iSearch CLI™
- **Official Author:** ErikrafT
- **Copyright:** Copyright © 2026 ErikrafT

---

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
- **Private Browsing (Anonymous Mode):** Browsing with absolute privacy. No history is stored, cookies/cache are fully isolated/deleted on exit, temporary in-memory downloads lists are used, and Chromium is launched in strict incognito mode (`🕵️‍♂️ [PRIVATE MODE]` indicator).
- **3D Wireframe Rotation Engine:** Interactive 3D wireframe mesh model rendering on ANSI canvases with rotational key controls.
- **ZIP Archive Explorer:** Directly inspect, scroll, and browse directories embedded inside ZIP folders without extraction.
- **PIX Donation Screen:** Standard-compliant EMV Co QR Code payloads, high-quality terminal QR rendering, and cross-platform clipboard integration.
- **Version Check & Self-Updater:** Integrated update checks and robust safe update installation directly inside the application.

---

## Installation

### 💻 Windows
- **Portable:** Download `isearch-windows-x86_64.exe` from GitHub Releases.
- **Installer:** Run `isearch-cli-windows-x64.exe` or `isearch-cli-windows-x64.msi`.

### 🐧 Linux
- **Debian/Ubuntu:** `sudo dpkg -i isearch-linux-x86_64.deb`
- **RedHat/Fedora:** `sudo rpm -i isearch-linux-x86_64.rpm`
- **Portable:** `tar -xzf isearch-linux-x86_64.tar.gz` and place inside your executable path.

### 🍎 macOS
- **Package:** Run `isearch-installer-macos-aarch64.pkg` (Apple Silicon) or `isearch-installer-macos-x86_64.pkg` (Intel).
- **Disk Image:** Drag and drop `isearch.dmg` into Applications.

### 🤖 Android Termux (Automatic Installation)

Install iSearch CLI™ inside Termux automatically using one of the following commands:

```bash
curl -fsSL https://download.erikraft.com/install-termux.sh | bash
```
Or using `wget`:
```bash
wget -qO- https://download.erikraft.com/install-termux.sh | bash
```

See [docs/termux.md](docs/termux.md) for detailed guidelines, security considerations, and manual methods.

### 🛠️ Manual / Compile from Source
If you wish to compile **iSearch CLI™** from source, you can clone and checkout the latest version tag:
```bash
git clone https://github.com/erikraft/iSearch-CLI.git
cd iSearch-CLI
git checkout v1.0.0
cargo build --release
```

---

## Quick Start

Launch the interactive shell:

```bash
isearch
```

Inside the iSearch CLI™ prompt, run commands such as:

```text
iSearch> browse
iSearch> version
iSearch> version --check
iSearch> self-update
iSearch> donate
```

---

## Self Update

To check for newer versions and download the correct package securely, execute:
```bash
isearch self-update
```
The updater utilizes a safe rename-replace method to apply updates, preserving user configurations and rolling back instantly on any failure.

---

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
  - Displays a clear `🕵️‍♂️ [PRIVATE MODE]` visual indicator.
  - Deletes all session profiles and isolates cache/cookies.
  - Temporarily holds downloads in an in-memory list which is cleared upon closing.
  - No database history or session logs are recorded.

---

## Donation Support

iSearch CLI™ includes a professional terminal donation flow powered by PIX.

### Launch the donation screen

```bash
isearch donate
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

---

## Configuration

Create or update `config.toml` with your preferences:

```toml
[donation]
pix_key = "11925416678"
currency = "BRL"
default_values = [5, 10, 20, 50, 100]
```

---

## Privacy & Security

- Strictly offline-first QR generation and validation
- Private/Anonymous mode with Zero persistent trails
- No telemetry or tracking
- Secure defaults for safe terminal browsing

---

## Documentation

Full detailed guides are available in the `docs/` folder and official [GitHub Wiki](https://github.com/erikraft/iSearch-CLI/wiki):
- [docs/installation.md](docs/installation.md) - Deep multi-platform setup instructions.
- [docs/releases.md](docs/releases.md) - Pipeline architecture and verification steps.
- [docs/self-update.md](docs/self-update.md) - Self updating mechanisms.
- [docs/termux.md](docs/termux.md) - Guide specifically tailored for Android Termux.
- [docs/downloads.md](docs/downloads.md) - Centralized download endpoint guidelines.

---

## Author

Original author: **ErikrafT**

MIT License. See [LICENSE](LICENSE) for details.
