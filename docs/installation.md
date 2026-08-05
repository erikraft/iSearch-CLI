# iSearch CLI™ Installation Guide

Official Name: **iSearch CLI™**
Official Author: **ErikrafT**
Copyright: **Copyright © 2026 ErikrafT**

---

This document provides a comprehensive setup guide for installing and running **iSearch CLI™** on all major operating systems.

## Supported Platforms

iSearch CLI™ runs natively on:
- **Windows** (x86_64, portable, installers, .msi)
- **Linux** (x86_64, aarch64, armv7, .deb, .rpm, .AppImage)
- **macOS** (Intel x86_64 and Apple Silicon aarch64, .pkg, .dmg)
- **Android Termux** (Aarch64, arm, x86_64)

---

## 💻 Windows Installation

### Method 1: Portable Binary
1. Go to the [Releases Page](https://github.com/erikraft/iSearch-CLI/releases).
2. Download `isearch-windows-x86_64.exe`.
3. Add the directory containing the file to your system's `PATH` environment variable.
4. Run `isearch` in Command Prompt, PowerShell, or Windows Terminal.

### Method 2: Windows Installer (.exe)
1. Download `isearch-installer-x86_64.exe` from GitHub Releases.
2. Double-click the installer and follow the on-screen instructions.
3. The installer will automatically configure shortcuts and environment variables.

### Method 3: MSI Installer (.msi)
For enterprise enrollment and automated deployment, download `isearch-installer-x86_64.msi` and run:
```powershell
msiexec /i isearch-installer-x86_64.msi /quiet
```

---

## 🐧 Linux Installation

### Method 1: Debian/Ubuntu (.deb)
```bash
sudo dpkg -i isearch-linux-x86_64.deb
```

### Method 2: RedHat/Fedora (.rpm)
```bash
sudo rpm -i isearch-linux-x86_64.rpm
```

### Method 3: AppImage
1. Download `isearch-linux-x86_64.AppImage`.
2. Make it executable:
   ```bash
   chmod +x isearch-linux-x86_64.AppImage
   ```
3. Execute:
   ```bash
   ./isearch-linux-x86_64.AppImage
   ```

### Method 4: Portable tar.gz
```bash
tar -xzf isearch-linux-x86_64.tar.gz
sudo mv isearch-cli /usr/local/bin/isearch
sudo chmod +x /usr/local/bin/isearch
```

---

## 🍎 macOS Installation

### Method 1: macOS Installer (.pkg)
1. Download `isearch-installer-macos-aarch64.pkg` (Apple Silicon) or `isearch-installer-macos-x86_64.pkg` (Intel).
2. Double-click to run and follow the prompts.

### Method 2: DMG Package (.dmg)
1. Open the `.dmg` file.
2. Drag **iSearch CLI™** to your `/Applications` directory.

### Method 3: Portable Binary
```bash
tar -xzf isearch-macos-aarch64.tar.gz
sudo mv isearch-cli /usr/local/bin/isearch
chmod +x /usr/local/bin/isearch
```

---

## 🤖 Android Termux Installation

See [docs/termux.md](termux.md) for detailed guidelines.

### Automatic Method 1 (curl)
```bash
curl -fsSL https://download.erikraft.com/install-termux.sh | bash
```

### Automatic Method 2 (wget)
```bash
wget -qO- https://download.erikraft.com/install-termux.sh | bash
```

---

## 🛠️ Manual Installation Alternative

If you wish to compile **iSearch CLI™** from source:

1. Clone the repository:
   ```bash
   git clone https://github.com/erikraft/iSearch-CLI.git
   cd iSearch-CLI
   git checkout v1.0.0
   ```
2. Build the executable in release mode:
   ```bash
   cargo build --release
   ```
3. Move the binary into your standard path:
   ```bash
   sudo mv target/release/isearch-cli /usr/local/bin/isearch
   ```

---

## 🗑️ Removal Instructions

### Windows
- Open Settings -> Apps -> Installed Apps, locate **iSearch CLI™**, and click **Uninstall**.

### Linux / macOS
- Delete the executable and any configuration folder:
  ```bash
  rm -f /usr/local/bin/isearch
  rm -rf ~/.config/isearch
  ```

### Termux
- Run:
  ```bash
  rm -f $PREFIX/bin/isearch
  ```
