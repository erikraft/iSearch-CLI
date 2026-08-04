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
