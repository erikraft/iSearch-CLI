use std::env;
use std::fs;
use serde::Deserialize;
use crate::browser::release_config::{
    BRAND_NAME, AUTHOR, COPYRIGHT, get_latest_release_url, get_download_url
};

#[derive(Debug, Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub published_at: String,
    pub assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug)]
pub struct VersionInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_date: String,
    pub platform: String,
    pub arch: String,
    pub update_available: bool,
}

pub fn get_platform_info() -> (String, String) {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();
    (os, arch)
}

pub fn check_for_updates() -> Result<VersionInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let (platform, arch) = get_platform_info();

    let url = get_latest_release_url();
    let agent = ureq::Agent::new_with_defaults();

    // Set User-Agent as required by Github API
    let response = agent.get(&url)
        .header("User-Agent", "isearch-cli-update-agent")
        .call()
        .map_err(|e| format!("Failed to check updates: {:?}", e))?;

    let mut body_reader = response.into_body();
    let body_str = body_reader.read_to_string()
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

pub fn print_version(check: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    let (platform, arch) = get_platform_info();

    println!("░▀█▀░█▀▀░█▀▀░█▀█░█▀▄░█▀▀░█░█░░░█▀▀░█░░░▀█▀");
    println!("░░█░░▀▀█░█▀▀░█▀█░█▀▄░█░░░█▀█░░░█░░░█░░░░█░");
    println!("░▀▀▀░▀▀▀░▀▀▀░▀░▀░▀░▀░▀▀▀░▀░▀░░░▀▀▀░▀▀▀░▀▀▀");
    println!("                 {} ", BRAND_NAME);
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
                    println!("\n🚀 A new version is available! (v{} -> v{})", info.current_version, info.latest_version);
                    println!("To update automatically, run:");
                    println!("  isearch self-update");
                } else {
                    println!("\n✨ You are on the latest version of {}!", BRAND_NAME);
                }
            }
            Err(e) => {
                eprintln!("⚠️ Error checking for updates: {}", e);
            }
        }
        println!("=================================================");
    }
}

pub fn perform_self_update() -> Result<(), String> {
    println!("Checking for latest release...");
    let info = check_for_updates()?;
    if !info.update_available {
        println!("✨ Already up to date (v{})!", info.current_version);
        return Ok(());
    }

    println!("🚀 New version found: v{} (Current: v{})", info.latest_version, info.current_version);

    let current_exe = env::current_exe()
        .map_err(|e| format!("Failed to identify current executable path: {}", e))?;

    let (platform, arch) = get_platform_info();

    // Construct the expected asset name based on platform/arch
    // e.g. isearch-x86_64-unknown-linux-gnu or similar
    // For simplicity and alignment with Github Actions, we can target standard asset names:
    // Linux: isearch-linux-x86_64 or isearch-linux-aarch64
    // Windows: isearch-windows-x86_64.exe
    // macOS: isearch-macos-x86_64 or isearch-macos-aarch64
    let asset_name = match (platform.as_str(), arch.as_str()) {
        ("linux", "x86_64") => "isearch-linux-x86_64",
        ("linux", "aarch64") => "isearch-linux-aarch64",
        ("linux", "arm") => "isearch-linux-arm",
        ("windows", "x86_64") => "isearch-windows-x86_64.exe",
        ("macos", "x86_64") => "isearch-macos-x86_64",
        ("macos", "aarch64") => "isearch-macos-aarch64",
        // Default fallbacks
        (os, ar) => {
            return Err(format!("Unsupported architecture/OS combination for automatic updates: {}-{}", os, ar));
        }
    };

    println!("Downloading asset: {}...", asset_name);
    let download_url = get_download_url(&info.latest_version, asset_name);

    let agent = ureq::Agent::new_with_defaults();
    let response = agent.get(&download_url)
        .call()
        .map_err(|e| format!("Failed to download update binary from {}: {:?}", download_url, e))?;

    let mut reader = response.into_body();
    let binary_data = reader.read_to_vec()
        .map_err(|e| format!("Failed to read binary data: {}", e))?;

    if binary_data.is_empty() {
        return Err("Downloaded binary data is empty!".to_string());
    }

    // Prepare path for backup and temporary new executable to prevent corruption / write locks
    let mut backup_exe = current_exe.clone();
    backup_exe.set_extension("bak");

    let mut temp_exe = current_exe.clone();
    temp_exe.set_extension("tmp");

    println!("Applying update safely...");

    // Write the new binary to the temp path
    fs::write(&temp_exe, &binary_data)
        .map_err(|e| format!("Failed to write temporary binary: {}", e))?;

    // Make the temp binary executable on non-Windows platforms
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&temp_exe) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = fs::set_permissions(&temp_exe, permissions);
        }
    }

    // Backup current executable by renaming
    if current_exe.exists() {
        if let Err(e) = fs::rename(&current_exe, &backup_exe) {
            let _ = fs::remove_file(&temp_exe);
            return Err(format!("Failed to backup current executable: {}", e));
        }
    }

    // Rename the new temporary binary to target path
    if let Err(e) = fs::rename(&temp_exe, &current_exe) {
        // Rollback on failure!
        if backup_exe.exists() {
            let _ = fs::rename(&backup_exe, &current_exe);
        }
        let _ = fs::remove_file(&temp_exe);
        return Err(format!("Failed to install new executable (rolled back): {}", e));
    }

    // Clean up backup file
    if backup_exe.exists() {
        let _ = fs::remove_file(&backup_exe);
    }

    println!("✨ Update to v{} completed successfully!", info.latest_version);
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
