# Installation Guide

This guide provides step-by-step instructions for installing and running **iSearch CLI™** on all supported platforms.

---

## 💻 Windows Installation

We offer three flexible options for installing iSearch CLI™ on Windows platforms (fully compatible with Command Prompt, PowerShell, and Windows Terminal):

### Method 1: Portable Binary
1. Download `isearch-windows-x86_64.exe` from [GitHub Releases](https://github.com/erikraft/iSearch-CLI/releases).
2. Save it to a directory of your choice (e.g., `C:\Tools\iSearch`).
3. Add the directory to your system's User `PATH` using PowerShell:
   ```powershell
   [System.Environment]::SetEnvironmentVariable('Path', $env:Path + ';C:\Tools\iSearch', 'User')
   ```
4. Restart your terminal and run: `isearch`

### Method 2: Graphical Setup Installer (.exe)
1. Download `isearch-cli-windows-x64.exe` from GitHub Releases.
2. Double-click the file and follow the interactive wizard. It automatically registers environmental paths, writes standard register metadata, and sets up uninstall parameters.

### Method 3: Enterprise MSI Installer (.msi)
For headless corporate deployment or quiet automation, execute the MSI installer through PowerShell:
```powershell
msiexec /i isearch-cli-windows-x64.msi /quiet
```

---

## 🐧 Linux Installation

Deploy using the native package manager appropriate for your Linux distribution:

### Debian / Ubuntu (`.deb`)
```bash
sudo dpkg -i isearch-linux-x86_64.deb
```

### RedHat / Fedora / CentOS (`.rpm`)
```bash
sudo rpm -i isearch-linux-x86_64.rpm
```

### AppImage Package
1. Download `isearch-linux-x86_64.AppImage`.
2. Grant executable permissions:
   ```bash
   chmod +x isearch-linux-x86_64.AppImage
   ```
3. Execute the package directly:
   ```bash
   ./isearch-linux-x86_64.AppImage
   ```

### Standalone Tarball (`.tar.gz`)
```bash
tar -xzf isearch-linux-x86_64.tar.gz
sudo mv isearch-cli /usr/local/bin/isearch
sudo chmod +x /usr/local/bin/isearch
```

---

## 🍎 macOS Installation

### Method 1: macOS Installer (`.pkg`)
1. Download the installer matching your processor architecture:
   - **Apple Silicon (M1/M2/M3):** `isearch-installer-macos-aarch64.pkg`
   - **Intel Core/Xeon:** `isearch-installer-macos-x86_64.pkg`
2. Run the PKG file and complete the standard macOS wizard setup.

### Method 2: Disk Image (`.dmg`)
1. Open the `.dmg` file.
2. Drag and drop **iSearch CLI™** to your system `/Applications` folder.

### Method 3: Tarball Terminal Extraction
```bash
tar -xzf isearch-macos-aarch64.tar.gz
sudo mv isearch-cli /usr/local/bin/isearch
chmod +x /usr/local/bin/isearch
```

---

## 🤖 Android Termux Installation

iSearch CLI™ is highly optimized for Android mobile terminals. We provide a automated installer script that handles platform analysis, connection vetting, and architecture detection.

### Automated Method 1 (CURL)
```bash
curl -fsSL https://download.erikraft.com/install-termux.sh | bash
```

### Automated Method 2 (WGET)
```bash
wget -qO- https://download.erikraft.com/install-termux.sh | bash
```

### Manual Termux Setup
If you choose not to run the online installer, execute these manual steps within Termux:
```bash
mkdir -p $PREFIX/bin
curl -fsSL -o $PREFIX/bin/isearch https://github.com/erikraft/iSearch-CLI/releases/latest/download/isearch-linux-aarch64
chmod +x $PREFIX/bin/isearch
```

---

## 🛠️ Manual Compilation from Source

If you prefer building and compiling the software yourself, ensure you have a working Rust toolchain and the required system dependencies:

### 1. Install Dependencies
* **Linux (Debian/Ubuntu):**
  ```bash
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config libsqlite3-dev
  ```

### 2. Clone and Compile
```bash
git clone https://github.com/erikraft/iSearch-CLI.git
cd iSearch-CLI
git checkout v1.0.0 # Standard release tag format
cargo build --release
```

### 3. Relocate Compiled Binary
Move the resulting executable to your standard local bin path:
```bash
sudo mv target/release/isearch-cli /usr/local/bin/isearch
```
