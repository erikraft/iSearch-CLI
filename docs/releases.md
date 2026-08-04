# iSearch CLI™ Releases & CI Pipeline Documentation

Official Name: **iSearch CLI™**
Official Author: **ErikrafT**
Copyright: **Copyright © 2026 ErikrafT**

---

This document describes the automated release and continuous distribution pipeline for **iSearch CLI™**.

## Automated CI/CD Workflows

All builds, packages, and installers are compiled and generated automatically on remote servers using **GitHub Actions**. Binary files are never stored within the Git repository itself.

The release system includes 13 production-grade workflows located in `.github/workflows/`:

1. **`build-linux.yml`** - Builds and archives standard Linux bin assets.
2. **`build-windows.yml`** - Compiles Windows binaries and prepares installers.
3. **`build-macos.yml`** - Compiles macOS Intel and Apple Silicon targets.
4. **`build-termux.yml`** - Generates Android Termux-compatible executables.
5. **`tests.yml`** - Runs unit tests across Linux, Windows, and macOS matrices.
6. **`fmt.yml`** - Enforces standard Rust style guidelines.
7. **`clippy.yml`** - Ensures clean, optimal code using cargo clippy lints.
8. **`rustdoc.yml`** - Validates code documentation structure and generation.
9. **`security.yml`** - Scans the codebase for security flaws.
10. **`audit.yml`** - Audits dependencies for published security disclosures.
11. **`dependency-check.yml`** - Runs dependency compliance and integrity tools.
12. **`publish.yml`** - Performs dry-run checks for publishing to crates.io.
13. **`release.yml`** - Orchestrates tag/release dispatch events, packages installers, calculates SHA256 checksums, and creates Github Releases.

---

## Release Triggers

The Release pipeline is automatically executed whenever:
1. A Git tag matching `v*` is pushed (e.g. `v1.0.0`).
2. A GitHub Release is manually published.
3. A manual workflow dispatch is triggered by a repository administrator.

---

## Checksum Verification

To ensure binary integrity, GitHub Actions generates checksum files `SHA256SUMS` and `checksums.txt` during release orchestration.

### Verification Instructions

To verify your downloaded binary on Unix-based systems:
```bash
sha256sum --check SHA256SUMS
```

On Windows (PowerShell):
```powershell
Get-FileHash -Algorithm SHA256 .\isearch-windows-x86_64.exe
```
Compare the resulting hash against the values published in `checksums.txt`.
