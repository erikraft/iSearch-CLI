use tl::{Node, ParserOptions};

/// Minimal metadata representation parsed from HTML to detect iSearch CLI support.
#[derive(Debug, Clone)]
pub struct SiteMetadata {
    pub cli: bool,
    pub version: Option<String>,
    pub endpoint: Option<String>,
}

fn attribute_value<'a>(tag: &tl::Tag<'a>, key: &str) -> Option<String> {
    for (attr_key, attr_value) in tag.attributes().iter() {
        if attr_key.as_utf8_str().eq_ignore_ascii_case(key) {
            return attr_value.map(|value| value.to_string());
        }
    }
    None
}

/// Detects presence of `<meta name="iSearch-cli" content="true">` and optional fields.
pub fn detect(html: &str) -> SiteMetadata {
    let mut meta = SiteMetadata {
        cli: false,
        version: None,
        endpoint: None,
    };

    if let Ok(dom) = tl::parse(html, ParserOptions::default()) {
        let parser = dom.parser();
        for node_handle in dom.nodes() {
            if let Some(node) = node_handle.get(parser) {
                if let Node::Tag(tag) = node {
                    if tag.name().as_utf8_str().eq_ignore_ascii_case("meta") {
                        if let Some(name) = attribute_value(tag, "name") {
                            let name_lower = name.to_lowercase();
                            let content = attribute_value(tag, "content");

                            match name_lower.as_str() {
                                "isearch-cli" => {
                                    meta.cli = content
                                        .as_deref()
                                        .map(|value| value.eq_ignore_ascii_case("true"))
                                        .unwrap_or(false);
                                }
                                "isearch-cli-version" => {
                                    meta.version = content;
                                }
                                "isearch-cli-endpoint" => {
                                    meta.endpoint = content;
                                }
                                _ => {}
                            }
                        }
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
