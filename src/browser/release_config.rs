//! Centralized configuration for iSearch CLI™ release distribution and update checking.
//! This module holds official branding constants and update URLs.

/// Official branding name of the application.
pub const BRAND_NAME: &str = "iSearch CLI™";

/// Official author of the application.
pub const AUTHOR: &str = "ErikrafT";

/// Official copyright notice of the application.
pub const COPYRIGHT: &str = "Copyright © 2026 ErikrafT";

/// GitHub organization and repository path.
pub const GITHUB_ORG_REPO: &str = "erikraft/iSearch-CLI";

/// Certified future download domain.
pub const DOWNLOAD_DOMAIN: &str = "https://download.erikraft.com";

/// Toggles whether to use the custom download domain or GitHub releases API for checking updates.
pub const USE_DOWNLOAD_DOMAIN_FOR_UPDATES: bool = false;

/// Retrieves the correct URL to query for identifying the latest release version.
pub fn get_latest_release_url() -> String {
    if USE_DOWNLOAD_DOMAIN_FOR_UPDATES {
        format!("{}/releases/latest.json", DOWNLOAD_DOMAIN)
    } else {
        format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_ORG_REPO
        )
    }
}

/// Retrieves the download URL for a specific compiled binary asset filename.
pub fn get_download_url(version: &str, filename: &str) -> String {
    if USE_DOWNLOAD_DOMAIN_FOR_UPDATES {
        format!("{}/releases/{}/{}", DOWNLOAD_DOMAIN, version, filename)
    } else {
        format!(
            "https://github.com/{}/releases/download/v{}/{}",
            GITHUB_ORG_REPO, version, filename
        )
    }
}
