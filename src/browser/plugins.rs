//! Browser plugin framework and utility integrations.
//!
//! # Purpose
//! This module implements auxiliary browser features that hook into rendering flows,
//! such as domain-filtering AdBlocker lists to speed up loading and reduce clutter.
//!
//! # Architecture
//! The module houses plugins like [AdBlocker](crate::browser::plugins::AdBlocker) which check URLs before loading them.

/// A lightweight, customizable domain-filtering AdBlocker.
///
/// Holds a static list of common analytical tracking and advertising domains.
/// Prevents requests to these domains to enhance privacy and decrease rendering latency.
///
/// # Fields
/// * `blocked_domains` - List of domain patterns used to filter outgoing URLs.
pub struct AdBlocker {
    /// Vector of string patterns to scan for in target request URLs.
    pub blocked_domains: Vec<String>,
}

impl AdBlocker {
    /// Creates a new instance of [AdBlocker] pre-populated with standard blocklists.
    ///
    /// # Returns
    ///
    /// Returns an initialized [AdBlocker] with default analytics and tracking rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::plugins::AdBlocker;
    /// let adblock = AdBlocker::new();
    /// assert!(adblock.is_blocked("https://doubleclick.net/ad"));
    /// ```
    pub fn new() -> Self {
        Self {
            blocked_domains: vec![
                "doubleclick.net".to_string(),
                "google-analytics.com".to_string(),
                "googlesyndication.com".to_string(),
                "adservice.google.com".to_string(),
                "adnxs.com".to_string(),
                "amazon-adsystem.com".to_string(),
            ],
        }
    }

    /// Evaluates if a specified URL matches any domain pattern inside the blocklist.
    ///
    /// Matches are case-insensitive and check if the domain string is contained in the URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The destination URL string slice to inspect.
    ///
    /// # Returns
    ///
    /// Returns `true` if the URL contains a blocked domain, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use isearch_cli::browser::plugins::AdBlocker;
    /// let adblock = AdBlocker::new();
    /// assert!(adblock.is_blocked("http://adservice.google.com/feed"));
    /// assert!(!adblock.is_blocked("https://rust-lang.org"));
    /// ```
    pub fn is_blocked(&self, url: &str) -> bool {
        let clean = url.to_lowercase();
        for domain in &self.blocked_domains {
            if clean.contains(domain) {
                return true;
            }
        }
        false
    }
}

impl Default for AdBlocker {
    /// Generates the default instance of [AdBlocker].
    ///
    /// # Returns
    ///
    /// Returns default [AdBlocker].
    fn default() -> Self {
        Self::new()
    }
}
