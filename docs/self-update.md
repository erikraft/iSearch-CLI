# iSearch CLI™ Self-Update System

Official Name: **iSearch CLI™**
Official Author: **ErikrafT**
Copyright: **Copyright © 2026 ErikrafT**

---

**iSearch CLI™** features a built-in, production-grade self-update command to ensure you always have access to the latest premium features and security improvements.

## Usage

To check for a new version and update automatically, run:
```bash
isearch self-update
```

Or from inside the interactive terminal prompt, type:
```text
iSearch> self-update
```

---

## Technical Architecture

The update system uses a robust and secure rename-replace strategy to replace the running executable without causing locks or process crashes:

1. **Version Fetching:** It queries the centralized endpoint (`api.github.com` or `download.erikraft.com`) to check the tag of the latest release.
2. **Platform and Architecture Detection:** It identifies the running OS and CPU architecture.
3. **Asset Downloading:** It retrieves the exact corresponding binary from the remote release assets list.
4. **Binary Buffering:** The binary data is read and buffered entirely in memory.
5. **Backup & Safety Rollback:**
   - The current binary is renamed to `isearch.bak` (which works even while the process is active).
   - The new binary is written to `isearch.tmp`.
   - On success, `isearch.tmp` replaces the active `isearch` path, and `isearch.bak` is removed.
   - On any write or permission error, the system automatically rolls back by restoring `isearch.bak` to preserve the working state.
6. **Execution Privileges:** Executable permissions are set correctly on Unix/macOS platforms.

## Configuration Preservation

Your user settings and custom configuration files (`config.toml`, `isearch.toml`) are completely untouched and fully preserved. They are stored separately from the binary executable directory to prevent data loss.

---

## Troubleshooting

### Error: Permission Denied
If you receive a permission error, it means **iSearch CLI™** was installed into a system directory (like `/usr/local/bin`) requiring root privileges to modify. Run:
```bash
sudo isearch self-update
```
*Note: This is never required for Android Termux installations, which are fully local.*
