# iSearch CLI™

**iSearch CLI™** is a polished terminal browser and search assistant designed for modern developers, power users, and terminal-first workflows.

## What is iSearch CLI™?

iSearch CLI™ delivers a premium terminal experience with a professional interactive user interface, rich search integration, and native support for modern command-line platforms.

- Built for **Windows, macOS, Linux, Android Termux, Docker, SSH sessions, GitHub Codespaces, VS Code terminals, JetBrains terminals, and any ANSI-compatible terminal**
- Designed for **keyboard-first navigation** with optional mouse support when available
- Supports **Unicode icons, 24-bit color, responsive layouts, and polished terminal rendering**
- Offers seamless **search, browsing, and donation workflows entirely inside the terminal**

## Features

- Interactive, modern TUI with rounded borders, color themes, and responsive layouts
- Cross-platform support for major desktop and mobile terminal environments
- Native terminal search interface with support for search engine aliases and presets
- Terminal-based PIX donation screen with QR code generation and clipboard integration
- Customizable configuration via `config.toml`
- Secure offline-first operations and privacy-friendly defaults

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
iSearch> search rust terminal browser
iSearch> donate
```

## Donation Support

iSearch CLI™ includes a professional terminal donation flow powered by PIX.

### Launch the donation screen

```bash
isearch donate
# or
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

## Supported Platforms

- macOS
- Windows
- Linux
- Android (Termux)
- Docker containers
- SSH terminals
- GitHub Codespaces
- VS Code terminal
- JetBrains terminal

## Privacy & Security

- Offline-first QR generation and validation
- No telemetry or tracking
- Secure defaults for safe terminal browsing

## Author

Original author: **ErikrafT**

MIT License. See [LICENSE](LICENSE) for details.
