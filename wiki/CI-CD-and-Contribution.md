# CI/CD & Contribution

This page outlines the GitHub Actions continuous integration pipelines, automated packaging matrices, and how to contribute to the **iSearch CLI™** open-source repository.

---

## 🏗️ Automated CI/CD Pipelines

**iSearch CLI™** incorporates 13 production-grade workflows located under `.github/workflows/` that execute linting, testing, and multi-platform packaging:

1. **`tests.yml`:** Executes unit tests across macOS, Windows, and Linux matrices on every push and pull request.
2. **`fmt.yml`:** Validates code formatting rules via `cargo fmt`.
3. **`clippy.yml`:** Enforces optimal code using strict `cargo clippy` compiler checks.
4. **`rustdoc.yml`:** Validates that 100% of public items are fully documented, enforcing the `#![deny(missing_docs)]` rule in `src/main.rs`.
5. **`security.yml`:** Scans code files for sensitive keys or vulnerable patterns.
6. **`audit.yml`:** Scans dependency crates for any active security advisories.
7. **`dependency-check.yml`:** Validates open-source license compatibility across tree structures.
8. **`publish.yml`:** Performs dry-run package checks for publishing to `crates.io`.
9. **`build-linux.yml`:** Builds Linux binaries.
10. **`build-windows.yml`:** Builds Windows binaries.
11. **`build-macos.yml`:** Builds macOS binaries.
12. **`build-termux.yml`:** Builds Android Termux binaries.
13. **`release.yml`:** Triggered on tag updates (e.g., `v1.0.0`). It packages installers, compiles setup wizards, calculates cryptographic hashes, and drafts GitHub Releases.

---

## 📦 Consistent Release Asset Configurations

When the release pipeline triggers, it publishes pre-compiled binaries and native installers to the official release server (`https://download.erikraft.com`) and GitHub Release assets.

### Filename Specifications:
- **Windows x64:** `isearch-cli-windows-x64.zip`, `isearch-windows-x86_64.exe`
- **Linux x64:** `isearch-cli-linux-x64.tar.gz`, `isearch-linux-x86_64.AppImage`, `isearch-cli-linux-amd64.deb`, `isearch-cli-linux-amd64.rpm`
- **macOS Intel:** `isearch-cli-macos-intel.zip`, `isearch-installer-macos-x86_64.dmg`
- **macOS Apple Silicon:** `isearch-cli-macos-arm64.zip`, `isearch-installer-macos-aarch64.pkg`
- **Android Termux:** `isearch-cli-termux-aarch64.tar.gz`, `isearch-linux-android-aarch64`

*(Legacy duplicate files are retained for backward compatibility with older installation shell scripts.)*

---

## 🤝 Contribution Guidelines

We welcome contributions from the community! To contribute:

1. **Fork the Repository:** Create your own fork of `erikraft/iSearch-CLI`.
2. **Create a Feature Branch:** Ensure your branch has a clear, descriptive name.
3. **Adhere to Code Standards:**
   - **Documentation:** Write standard idiomatic rustdoc tags (`//!` and `///`) for all new modules, structs, enums, functions, and fields. Document parameters, returns, and errors, and include runnable examples where applicable.
   - **No Binary Commitments:** Never commit compiled executables, installers, or compressed assets directly. These are generated on remote runners during release.
   - **Tests:** Add unit tests verifying your changes under appropriate module `tests` modules.
4. **Submit a Pull Request:** Describe your changes in detail and confirm all 13 GitHub Actions workflows pass.
