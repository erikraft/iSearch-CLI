/// Minimal metadata representation parsed from HTML to detect iSearch CLI support.
#[derive(Debug, Clone)]
pub struct SiteMetadata {
    pub cli: bool,
    pub version: Option<String>,
    pub endpoint: Option<String>,
}

fn meta_content(html: &str, wanted_name: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut offset = 0;
    while let Some(start) = lower[offset..].find("<meta") {
        let abs_start = offset + start;
        let end = lower[abs_start..].find('>').map(|e| abs_start + e)?;
        let tag = &html[abs_start..=end];
        let tag_lower = tag.to_lowercase();
        if tag_lower.contains(&format!("name=\"{}\"", wanted_name.to_lowercase()))
            || tag_lower.contains(&format!("name='{}'", wanted_name.to_lowercase()))
        {
            return extract_attr(tag, "content");
        }
        offset = end + 1;
    }
    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    for quote in ['\"', '\''] {
        let pat = format!("{}={}", attr, quote);
        if let Some(pos) = tag.to_lowercase().find(&pat) {
            let start = pos + pat.len();
            if let Some(end) = tag[start..].find(quote) {
                return Some(tag[start..start + end].to_string());
            }
        }
    }
    None
}

/// Detects presence of `<meta name="iSearch-cli" content="true">` and optional fields.
pub fn detect(html: &str) -> SiteMetadata {
    let cli = meta_content(html, "isearch-cli")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    SiteMetadata {
        cli,
        version: meta_content(html, "isearch-cli-version"),
        endpoint: meta_content(html, "isearch-cli-endpoint"),
    }
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
