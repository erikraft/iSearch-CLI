pub mod metadata;
pub mod parser;
pub mod renderer;

use crate::browser;

fn build_cli_user_agent() -> String {
    format!(
        "iSearchCLI/{} ({}/{})",
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
    let resp = agent
        .get(url)
        .header("User-Agent", &build_cli_user_agent())
        .call()?;
    Ok(resp.into_body().read_to_string()?)
}

fn fetch_cli_api(
    agent: &ureq::Agent,
    endpoint: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let resp = agent
        .get(endpoint)
        .header("Accept", "application/json")
        .header("User-Agent", &build_cli_user_agent())
        .call()?;
    Ok(resp.into_body().read_to_string()?)
}

/// Opens the actual ErikrafT Drop website in the terminal browser.
///
/// The API metadata endpoint is checked for compatibility, but the site itself is
/// rendered by the existing browser engine. Chromium's current implementation
/// returns a JavaScript-rendered DOM snapshot; interactive WebRTC/file workflows
/// are not yet guaranteed to work in a terminal renderer.
pub fn run_erikraft_drop() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://drop.erikraft.com";
    let agent = ureq::Agent::new_with_defaults();
    let page_url = format!("{}?client_type=isearch-cli", url);

    match fetch_page(&agent, &page_url) {
        Ok(body) => {
            let meta = metadata::detect(&body);
            if meta.cli {
                let endpoint_path = meta.endpoint.as_deref().unwrap_or("/api/cli");
                let endpoint_url = normalize_endpoint(url, endpoint_path);
                let endpoint_url = if endpoint_url.contains('?') {
                    format!("{}&client_type=isearch-cli", endpoint_url)
                } else {
                    format!("{}?client_type=isearch-cli", endpoint_url)
                };

                match fetch_cli_api(&agent, &endpoint_url)
                    .and_then(|json| parser::parse_cli_json(&json).map(|_| ()))
                {
                    Ok(()) => {}
                    Err(error) => eprintln!(
                        "Drop API metadata could not be validated (opening website anyway): {}",
                        error
                    ),
                }
            } else {
                eprintln!(
                    "Drop does not advertise the CLI metadata contract; opening the website renderer anyway."
                );
            }
        }
        Err(error) => eprintln!(
            "Could not preflight Drop metadata (opening website anyway): {}",
            error
        ),
    }

    let chromium = crate::browser::chromium::ChromiumEngine::new();
    if chromium.is_available() {
        eprintln!(
            "Opening ErikrafT Drop with Chromium rendering. The current terminal renderer may not support interactive WebRTC/file controls."
        );
        return browser::ui::run_browser_tui_with_url_and_engine(
            url,
            crate::browser::core::EngineType::Chromium,
        );
    }

    eprintln!(
        "Chromium is unavailable. Opening the site with native rendering; JavaScript-driven Drop features and transfers may not work."
    );
    browser::ui::run_browser_tui_with_url(url)
}
