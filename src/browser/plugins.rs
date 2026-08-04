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
    pub blocked_domains: Vec<String>,
}

impl AdBlocker {
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
    fn default() -> Self {
        Self::new()
    }
}
