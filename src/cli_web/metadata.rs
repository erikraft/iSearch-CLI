/// Minimal metadata representation parsed from HTML to detect iSearch CLI support.
#[derive(Debug, Clone)]
pub struct SiteMetadata {
    pub cli: bool,
    pub version: Option<String>,
    pub endpoint: Option<String>,
}

/// Detects presence of `<meta name="iSearch-cli" content="true">` and optional fields.
pub fn detect(html: &str) -> SiteMetadata {
    let mut meta = SiteMetadata {
        cli: false,
        version: None,
        endpoint: None,
    };

    let lower = html.to_lowercase();
    if lower.contains("<meta name=\"isearch-cli\" content=\"true\"")
        || lower.contains("<meta name='isearch-cli' content='true'")
        || lower.contains("name=\"isearch-cli\" content=\"true\"")
    {
        meta.cli = true;
    }

    // version
    if let Some(pos) = lower.find("name=\"isearch-cli-version\"") {
        if let Some(after) = lower.get(pos..) {
            if let Some(cpos) = after.find("content=") {
                let snippet = &after[cpos..];
                if let Some(start) = snippet.find('"') {
                    if let Some(end) = snippet[start + 1..].find('"') {
                        meta.version = Some(snippet[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
    }

    // endpoint
    if let Some(pos) = lower.find("name=\"isearch-cli-endpoint\"") {
        if let Some(after) = lower.get(pos..) {
            if let Some(cpos) = after.find("content=") {
                let snippet = &after[cpos..];
                if let Some(start) = snippet.find('"') {
                    if let Some(end) = snippet[start + 1..].find('"') {
                        meta.endpoint = Some(snippet[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
    }

    meta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_simple_meta() {
        let html = r#"
        <html><head>
        <meta name="iSearch-cli" content="true">
        <meta name="iSearch-cli-version" content="1">
        <meta name="iSearch-cli-endpoint" content="https://drop.erikraft.com/api/cli">
        </head><body></body></html>
        "#;
        let m = detect(html);
        assert!(m.cli);
        assert_eq!(m.version.unwrap(), "1");
        assert_eq!(m.endpoint.unwrap(), "https://drop.erikraft.com/api/cli");
    }

    #[test]
    fn detect_absent() {
        let html = "<html><head></head><body></body></html>";
        let m = detect(html);
        assert!(!m.cli);
        assert!(m.version.is_none());
        assert!(m.endpoint.is_none());
    }
}
