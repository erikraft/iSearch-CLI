//! Automated version checking, update checking, and safe self-updating logic for iSearch CLI™.

use crate::branding::{ascii_logo, isearch_cli};
use crate::browser::release_config::{get_download_url, get_latest_release_url, AUTHOR, COPYRIGHT};
use serde::Deserialize;
use std::env;
use std::fs;

/// Represents an individual asset metadata structure inside a GitHub Release.
#[derive(Debug, Deserialize)]
pub struct GithubReleaseAsset {
    /// The filename of the compiled release asset.
    pub name: String,
    /// The direct browser download URL of the asset.
    pub browser_download_url: String,
}

/// Represents the JSON payload schema returned by the GitHub Release API or update manifest.
#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    /// The Git tag name corresponding to this release (e.g., `v1.2.3`).
    pub tag_name: String,
    /// The UTC ISO timestamp representing when this release was published.
    pub published_at: String,
    /// List of compiled target binary assets included in this release.
    pub assets: Vec<GithubReleaseAsset>,
}

/// Holds parsed release information comparing the current installed version to the latest remote version.
#[derive(Debug)]
pub struct VersionInfo {
    /// The current semver version of the locally running executable.
    pub current_version: String,
    /// The latest semver version found on the remote release server.
    pub latest_version: String,
    /// The publication date of the latest remote release.
    pub release_date: String,
    /// The operating system platform of the local system.
    pub platform: String,
    /// The CPU architecture of the local system.
    pub arch: String,
    /// Indicates whether a newer remote version is available.
    pub update_available: bool,
}

/// Detects and returns the operating system and CPU architecture of the current machine.
pub fn get_platform_info() -> (String, String) {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();
    (os, arch)
}

/// Query the release manifest server to check for any available application updates.
///
/// # Errors
/// Returns an error if the network request fails, or if parsing the metadata fails.
pub fn check_for_updates() -> Result<VersionInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let (platform, arch) = get_platform_info();

    let url = get_latest_release_url();
    let agent = ureq::Agent::new_with_defaults();

    // Set User-Agent as required by Github API
    let response = agent
        .get(&url)
        .header("User-Agent", "isearch-cli-update-agent")
        .call()
        .map_err(|e| format!("Failed to check updates: {:?}", e))?;

    let mut body_reader = response.into_body();
    let body_str = body_reader
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {:?}", e))?;

    let release: GithubRelease = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse release JSON: {:?}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    // Semver check (simple comparison)
    let update_available = latest_version != current_version;

    Ok(VersionInfo {
        current_version,
        latest_version,
        release_date: release.published_at,
        platform,
        arch,
        update_available,
    })
}

/// Prints formatted version information, optionally checking for online updates.
pub fn print_version(check: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    let (platform, arch) = get_platform_info();

    println!("{}", ascii_logo());
    println!("=================================================");
    println!("Author:          {}", AUTHOR);
    println!("Copyright:       {}", COPYRIGHT);
    println!("Current Version: v{}", current_version);
    println!("Platform:        {}", platform);
    println!("Architecture:    {}", arch);
    println!("=================================================");

    if check {
        println!("Checking for updates...");
        match check_for_updates() {
            Ok(info) => {
                println!("Latest Version:  v{}", info.latest_version);
                println!("Release Date:    {}", info.release_date);
                if info.update_available {
                    println!(
                        "\n🚀 A new version is available! (v{} -> v{})",
                        info.current_version, info.latest_version
                    );
                    println!("To update automatically, run:");
                    println!("  isearch self-update");
                } else {
                    println!("\n✨ You are on the latest version of {}!", isearch_cli());
                }
            }
            Err(e) => {
                eprintln!("⚠️ Error checking for updates: {}", e);
            }
        }
        println!("=================================================");
    }
}

/// Automatically downloads and safely installs the latest version of the binary.
///
/// Uses a safe rename-replace backup strategy to support live replacement on Unix,
/// macOS, Windows, and Termux platforms.
///
/// # Errors
/// Returns an error if identifying, downloading, backing up, or applying the update binary fails.
pub fn perform_self_update() -> Result<(), String> {
    println!("Checking for latest release...");
    let info = check_for_updates()?;
    if !info.update_available {
        println!("✨ Already up to date (v{})!", info.current_version);
        return Ok(());
    }

    println!(
        "🚀 New version found: v{} (Current: v{})",
        info.latest_version, info.current_version
    );

    let current_exe = env::current_exe()
        .map_err(|e| format!("Failed to identify current executable path: {}", e))?;

    let (platform, arch) = get_platform_info();

    // Construct expected asset name matching CI runner matrices
    let asset_name = match (platform.as_str(), arch.as_str()) {
        ("linux", "x86_64") => "isearch-linux-x86_64",
        ("linux", "aarch64") => "isearch-linux-aarch64",
        ("linux", "arm") => "isearch-linux-arm",
        ("windows", "x86_64") => "isearch-windows-x86_64.exe",
        ("macos", "x86_64") => "isearch-macos-x86_64",
        ("macos", "aarch64") => "isearch-macos-aarch64",
        (os, ar) => {
            return Err(format!(
                "Unsupported architecture/OS combination for automatic updates: {}-{}",
                os, ar
            ));
        }
    };

    println!("Downloading asset: {}...", asset_name);
    let download_url = get_download_url(&info.latest_version, asset_name);

    let agent = ureq::Agent::new_with_defaults();
    let response = agent.get(&download_url).call().map_err(|e| {
        format!(
            "Failed to download update binary from {}: {:?}",
            download_url, e
        )
    })?;

    let mut reader = response.into_body();
    let binary_data = reader
        .read_to_vec()
        .map_err(|e| format!("Failed to read binary data: {}", e))?;

    if binary_data.is_empty() {
        return Err("Downloaded binary data is empty!".to_string());
    }

    // Backup current executable path to prevent corruption/write locks
    let mut backup_exe = current_exe.clone();
    backup_exe.set_extension("bak");

    let mut temp_exe = current_exe.clone();
    temp_exe.set_extension("tmp");

    println!("Applying update safely...");

    // Write new binary data to temporary path
    fs::write(&temp_exe, &binary_data)
        .map_err(|e| format!("Failed to write temporary binary: {}", e))?;

    // Make temporary binary executable on Unix targets
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&temp_exe) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = fs::set_permissions(&temp_exe, permissions);
        }
    }

    // Rename current binary to backup path
    if current_exe.exists() {
        if let Err(e) = fs::rename(&current_exe, &backup_exe) {
            let _ = fs::remove_file(&temp_exe);
            return Err(format!("Failed to backup current executable: {}", e));
        }
    }

    // Rename temp binary to target current executable path
    if let Err(e) = fs::rename(&temp_exe, &current_exe) {
        if backup_exe.exists() {
            let _ = fs::rename(&backup_exe, &current_exe);
        }
        let _ = fs::remove_file(&temp_exe);
        return Err(format!(
            "Failed to install new executable (rolled back): {}",
            e
        ));
    }

    // Clean up backup file on success
    if backup_exe.exists() {
        let _ = fs::remove_file(&backup_exe);
    }

    println!(
        "✨ Update to v{} completed successfully!",
        info.latest_version
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_info() {
        let (os, arch) = get_platform_info();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_github_release_serialization() {
        let json_data = r#"{
            "tag_name": "v1.2.3",
            "published_at": "2026-01-01T00:00:00Z",
            "assets": [
                {
                    "name": "isearch-linux-x86_64",
                    "browser_download_url": "https://github.com/erikraft/iSearch-CLI/releases/download/v1.2.3/isearch-linux-x86_64"
                }
            ]
        }"#;

        let release: GithubRelease = serde_json::from_str(json_data).unwrap();
        assert_eq!(release.tag_name, "v1.2.3");
        assert_eq!(release.published_at, "2026-01-01T00:00:00Z");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "isearch-linux-x86_64");
    }
}
