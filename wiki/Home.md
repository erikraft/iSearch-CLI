# iSearch CLI™ Official Wiki

Welcome to the official **iSearch CLI™** Wiki! This documentation hub provides a comprehensive overview of our interactive terminal-based web browser and developer utility.

---

## 📖 Table of Contents

- [Home](Home)
- [Architecture & Technologies](Architecture-and-Technologies)
- [Installation Guide](Installation-Guide)
- [Usage & Examples](Usage-and-Examples)
- [Updates & Troubleshooting](Updates-and-Troubleshooting)
- [CI/CD & Contribution](CI-CD-and-Contribution)

---

## 🌟 Overview

**iSearch CLI™** is a premium command-line web browser and search assistant designed with modern developer workflows, system administrators, and keyboard-first power users in mind. Built entirely in **Rust** and powered by **Ratatui** for visual terminal rendering, iSearch CLI™ bridges the gap between terminal environments and standard web navigation.

- **Fast & Responsive:** Sub-millisecond execution times and zero-cost abstractions.
- **ANSI-Compatible Visuals:** Features 24-bit color palettes, rounded borders, and clean terminal rendering.
- **Dual-Engine Browser:** Leverages a lightweight native Rust engine for file-parsing, PDFs, Markdown, and ZIP archives, and an optional headless Chromium engine for complex JavaScript pages.

---

## 🚀 Key Features

### 💻 Dual-Engine Architecture
- **Rust-Native Engine:** Renders offline documentation, static HTML/CSS files, local directories, Markdown text, and embedded PDFs in pure text layout instantly.
- **Headless Chromium Backend:** Automates modern web page rendering (React, Vue, etc.) entirely within the CLI by executing headlessly and capturing clean parsed text layouts.

### 🕵️‍♂️ 100% Private / Anonymous Browsing Mode
- Enabled instantly with a single keyboard shortcut (`V`).
- Isolates cookies, caching, and state profiles under temporary directories with the `--incognito` flag.
- Zero tracking or persistence: does not record or persist history in the SQLite database.
- Stores session downloads strictly in-memory (RAM) and clears them immediately upon exiting.

### 🗄️ SQLite History Database & Bookmarks (Favorites)
- **SQL-Backed History:** Organizes browsed pages by Domain or Date group. Allows filtering, sorting, and full keyboard-first traversal.
- **Favorites System:** Organizes bookmarks into custom folders, search by titles/URLs, and import/export from/to JSON payloads.

### 📈 Built-in TUI Visual Enhancements
- **3D Viewer Mesh Rotation Engine:** Interactive 3D wireframe mesh model rendering on ANSI canvases with rotational key controls.
- **ZIP Archive Explorer:** Directly inspect, scroll, and browse directories embedded inside ZIP folders without extraction.
- **Visual Media Protocols:** Render graphics using Sixel, Kitty, and iTerm2 protocols with robust Braille and ASCII fallbacks.

---

## ⚙️ System Requirements & Support

| Operating System | Architectures Supported | Minimum Dependencies |
| :--- | :--- | :--- |
| **Linux** | `x86_64`, `aarch64`, `armv7` | Standard libc, SQLite3, `pkg-config` (for manual compilation) |
| **Windows** | `x86_64`, `aarch64` | Windows 10/11, Command Prompt, PowerShell, or Windows Terminal |
| **macOS** | `x86_64` (Intel), `aarch64` (M-series) | macOS Catalina (10.15) or later |
| **Android Termux**| `aarch64`, `arm`, `x86_64` | Android 7.0+, Termux Terminal Emulator |

---

## 🔒 Branding and Intellectual Property

- **Official Project Name:** iSearch CLI™
- **Official Author:** ErikrafT
- **Official Copyright:** Copyright © 2026 ErikrafT
- **Official Distribution Server:** `https://download.erikraft.com`
- **License:** Licensed under the MIT License.
