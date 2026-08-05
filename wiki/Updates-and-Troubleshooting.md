# Updates & Troubleshooting

This page describes the technical updater system, troubleshooting workflows, and frequently asked questions (FAQ).

---

## 🔄 Self-Update Infrastructure

iSearch CLI™ contains an integrated, secure, production-ready subcommand to update the client:
```bash
isearch self-update
```

### Technical Workflow
The updater uses a robust, fault-tolerant **rename-replace** backup strategy to handle active process files securely across macOS, Windows, Linux, and Android Termux targets:

1. **Version Fetching:** Queries the centralized endpoint (`https://download.erikraft.com/releases/latest.json` or fallback GitHub APIs) to identify the latest release tag.
2. **Platform & Architecture Analysis:** Automatically detects system CPU targets.
3. **Download Buffering:** Downloads the optimized binary and buffers it entirely in-memory to prevent corrupt incomplete files.
4. **Renaming (Live Swap):**
   - Renames the current running executable `isearch` to `isearch.bak` (which is permitted by OS handles even while active).
   - Writes the new binary to `isearch.tmp`.
5. **Rollback Safety:**
   - Moves `isearch.tmp` to `isearch`.
   - On success, it purges the backup file `isearch.bak`.
   - On failure, it immediately restores the active `isearch.bak` binary to its original location to ensure zero service disruption.
6. **Execution Rights:** Sets proper execution permissions (`0o755` on Unix systems) on the new binary.

---

## 🔧 Troubleshooting Guide

### 1. Error: Permission Denied during update
* **Cause:** iSearch CLI™ was installed into a system folder (like `/usr/local/bin`) requiring root access for write operations.
* **Solution:** Execute the self-update command using `sudo`:
  ```bash
  sudo isearch self-update
  ```
  *(Note: This is never required for Android Termux setups, which run entirely within isolated local user spaces.)*

### 2. Chromium required but not available
* **Cause:** The optional Headless Chromium engine requires an external compatible browser executable to function, which could not be found automatically.
* **Solution:**
  1. Toggle to Native Rendering mode by pressing **`E`** inside the browser.
  2. Press **`3`** in the Chromium Assistant screen to type an absolute path to a Chromium binary (e.g., Chrome, Edge, Brave, etc.) manually on your local filesystem.

### 3. Termux Command Not Found after install
* **Cause:** The Termux PREFIX executable path is missing from your system environmental `$PATH` variable.
* **Solution:** Check your path by running `echo $PATH`. If `$PREFIX/bin` is missing, add it to your `~/.bashrc` file:
  ```bash
  export PATH="$PREFIX/bin:$PATH"
  ```

---

## ❓ FAQ (Frequently Asked Questions)

#### Q: Where are configuration files stored?
**A:** Depending on your operating system, config files are located at:
- **Linux/macOS/Termux:** `~/.config/isearch/`
- **Windows:** `%APPDATA%\isearch\`

#### Q: Does Private Mode store anything on disk?
**A:** Absolutely not. When Private Mode (`V`) is active, history logs are completely skipped, cookies/cache are fully sandboxed in temporary paths and purged immediately upon exiting, and download items are held strictly in memory (RAM).

#### Q: How can I change the default visual theme?
**A:** Use the keyboard shortcut **`K`** inside the browser to cycle through presets, or add theme overrides to your `config.toml`.

#### Q: Are local ZIP archives supported?
**A:** Yes. Enter the absolute path of any `.zip` archive (e.g., `/home/user/archive.zip`) in the URL bar. You can navigate folders, scroll the contents, and preview files immediately.
