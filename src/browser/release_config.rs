pub const BRAND_NAME: &str = "iSearch CLI™";
pub const AUTHOR: &str = "ErikrafT";
pub const COPYRIGHT: &str = "Copyright © 2026 ErikrafT";

// Update endpoints
pub const GITHUB_ORG_REPO: &str = "erikraft/iSearch-CLI";
pub const DOWNLOAD_DOMAIN: &str = "https://download.erikraft.com";

// Toggle this to true if migrating update check from GitHub Releases API to download.erikraft.com
pub const USE_DOWNLOAD_DOMAIN_FOR_UPDATES: bool = false;

pub fn get_latest_release_url() -> String {
    if USE_DOWNLOAD_DOMAIN_FOR_UPDATES {
        format!("{}/releases/latest.json", DOWNLOAD_DOMAIN)
    } else {
        format!("https://api.github.com/repos/{}/releases/latest", GITHUB_ORG_REPO)
    }
}

pub fn get_download_url(version: &str, filename: &str) -> String {
    if USE_DOWNLOAD_DOMAIN_FOR_UPDATES {
        format!("{}/releases/{}/{}", DOWNLOAD_DOMAIN, version, filename)
    } else {
        format!("https://github.com/{}/releases/download/v{}/{}", GITHUB_ORG_REPO, version, filename)
    }
}
