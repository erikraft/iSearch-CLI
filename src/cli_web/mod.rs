pub mod metadata;
pub mod renderer;
pub mod parser;

use crate::browser;
use metadata::SiteMetadata;

/// Entrypoint para executar o cliente ErikrafT Drop via CLI.
pub fn run_erikraft_drop() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://drop.erikraft.com";

    // Simple HTTP GET using ureq (already used elsewhere in the project)
    let agent = ureq::Agent::new();
    let resp = agent.get(url).call()?;
    let body = resp.into_string()?;

    let meta = metadata::detect(&body);
    if meta.cli {
        // Server advertises CLI support - use native TUI renderer
        renderer::run_drop_tui(&meta)
    } else {
        // Fallback to standard browser TUI
        eprintln!("Site does not advertise iSearch CLI™ support. Falling back to standard browser renderer.");
        browser::ui::run_browser_tui().map_err(|e| e.into())
    }
}
