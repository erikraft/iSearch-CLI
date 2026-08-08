pub mod metadata;
pub mod renderer;
pub mod parser;

use crate::browser;
use metadata::SiteMetadata;
use parser::CliDocument;

fn build_cli_user_agent() -> String {
    format!(
        "iSearch-CLI/{} ({}/{})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

fn normalize_endpoint(base: &str, endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return endpoint.to_string();
    }

    if endpoint.starts_with('/') {
        return format!("{}{}", base.trim_end_matches('/'), endpoint);
    }

    format!("{}/{}", base.trim_end_matches('/'), endpoint)
}

fn fetch_page(agent: &ureq::Agent, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = agent.get(url).call()?;
    Ok(resp.into_string()?)
}

fn fetch_cli_api(agent: &ureq::Agent, endpoint: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = agent
        .get(endpoint)
        .set("Accept", "application/json")
        .call()?;
    Ok(resp.into_string()?)
}

/// Entrypoint para executar o cliente ErikrafT Drop via CLI.
pub fn run_erikraft_drop() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://drop.erikraft.com";
    let user_agent = build_cli_user_agent();
    let agent = ureq::AgentBuilder::new().user_agent(&user_agent).build();
    let page_url = format!("{}?client_type=isearch-cli", url);

    let body = fetch_page(&agent, &page_url)?;
    let meta = metadata::detect(&body);

    if !meta.cli {
        eprintln!(
            "Site does not advertise iSearch CLI™ support. Falling back to standard browser renderer."
        );
        return browser::ui::run_browser_tui().map_err(|e| e.into());
    }

    let endpoint_path = meta.endpoint.as_deref().unwrap_or("/api/cli");
    let endpoint_url = normalize_endpoint(url, endpoint_path);
    let endpoint_url = if endpoint_url.contains('?') {
        format!("{}&client_type=isearch-cli", endpoint_url)
    } else {
        format!("{}?client_type=isearch-cli", endpoint_url)
    };

    let json = fetch_cli_api(&agent, &endpoint_url)?;
    let document = parser::parse_cli_json(&json)?;
    renderer::run_drop_tui(&meta, &document)
}
