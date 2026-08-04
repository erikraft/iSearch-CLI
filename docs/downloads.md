# iSearch CLI™ Download Server & Infrastructure

Official Name: **iSearch CLI™**
Official Author: **ErikrafT**
Copyright: **Copyright © 2026 ErikrafT**

---

This document outlines the architecture and future roadmap of the **iSearch CLI™** distribution servers.

## Current Setup (GitHub Releases)

At present, releases are distributed directly through GitHub Releases under the official repository `erikraft/iSearch-CLI`.
- Release binaries are built exclusively by GitHub Actions.
- Download URLs are computed by requesting version tags through the GitHub API.

---

## Future Setup (download.erikraft.com)

To provide an enterprise-grade experience, distribution will transition to the future certified domain:
```text
https://download.erikraft.com
```

The codebase, updater commands, and installation scripts are preconfigured to support this transition seamlessly, without requiring extensive refactoring.

### Switching endpoints

In **`src/browser/release_config.rs`**, updating the server address requires changing only a single boolean flag:
```rust
// Toggle this to true if migrating update check from GitHub Releases API to download.erikraft.com
pub const USE_DOWNLOAD_DOMAIN_FOR_UPDATES: bool = true;
```

Similarly, in **`scripts/install-termux.sh`**, updating the variable `USE_CUSTOM_DOMAIN` will redirect the installer to pull directly from the new hosting server:
```bash
USE_CUSTOM_DOMAIN=true
```

---

## Expected Manifest Schema

The update server will host a manifest JSON file at:
```text
https://download.erikraft.com/releases/latest.json
```

Schema of the file:
```json
{
  "tag_name": "v0.1.0",
  "published_at": "2026-01-01T12:00:00Z",
  "assets": [
    {
      "name": "isearch-linux-x86_64",
      "browser_download_url": "https://download.erikraft.com/releases/v0.1.0/isearch-linux-x86_64"
    },
    {
      "name": "isearch-windows-x86_64.exe",
      "browser_download_url": "https://download.erikraft.com/releases/v0.1.0/isearch-windows-x86_64.exe"
    }
  ]
}
```
This schema is completely identical to the GitHub Releases API structure, enabling backward compatibility with older client installations.
