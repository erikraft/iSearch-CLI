# iSearch CLI™ Android Termux Installation Guide

Official Name: **iSearch CLI™**
Official Author: **ErikrafT**
Copyright: **Copyright © 2026 ErikrafT**

---

This document describes how to install, run, and update **iSearch CLI™** inside the **Termux** environment on Android devices.

## Automated Installation

You can install **iSearch CLI™** inside Termux automatically using one of the following commands:

### Method 1 (curl)
```bash
curl -fsSL https://download.erikraft.com/install-termux.sh | bash
```
**What it does:**
- Downloads the certified `install-termux.sh` script from the ErikrafT download servers.
- Executes it using the bash shell.
- Performs automatic architecture, internet, and environment detection before applying the download.

### Method 2 (wget)
```bash
wget -qO- https://download.erikraft.com/install-termux.sh | bash
```
**What it does:**
- Similar to curl, it requests the script via wget, piping it directly into bash.
- This serves as a reliable fallback when curl is not pre-installed on your Termux container.

---

## Security and Integrity Considerations

Piping shell scripts from the web is highly convenient, but please review these guidelines to maintain standard security:
1. Ensure the download URL is strictly from the official domain: `https://download.erikraft.com` or raw GitHub under `erikraft/iSearch-CLI`.
2. Inspect the installer script contents before piping:
   ```bash
   curl -fsSL https://download.erikraft.com/install-termux.sh > install.sh
   nano install.sh # View lines
   bash install.sh # Run
   ```
3. Never run these scripts using `root` or `sudo` permissions unless you have audited every line. Termux installs run entirely within user scope.

---

## CPU Architectures Supported
The installer checks for the system architecture of your Android phone or device:
- `aarch64` / `arm64` (Standard modern Android devices)
- `armv7l` / `arm` (Older Android devices)
- `x86_64` (Android Emulators or specialized tablets)

---

## Troubleshooting

### Connection Failures
If the installer exits with `No internet connection detected`, ensure you have granted internet access permissions to Termux in Android App Settings, and check your cellular/Wi-Fi connection.

### Command Not Found
If the installation succeeds but running `isearch` prints `command not found`, ensure `$PREFIX/bin` or `$HOME/bin` is present in your bash PATH variable:
```bash
echo $PATH
```
Normally, Termux initializes this automatically.
