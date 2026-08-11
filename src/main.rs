pub mod branding;
pub mod browser;
pub mod cli_web;
pub mod config;
pub mod pix;
pub mod ui;
pub mod uri;
pub mod utils;

use branding::{ascii_logo, isearch_cli};
use config::load_config;
use std::env;
use std::io::{self, Write};
use ui::run_donation_tui;

/// Formats and outputs the global list of interactive commands.
pub fn print_help() {
    println!("{} - Version 0.1.0", isearch_cli());
    println!("Available commands:");
    println!("  browse         - Open the premium multi-engine interactive terminal browser");
    println!("  open <URI>     - Open a URI or local file through the centralized URI router");
    println!("  erikraft-drop  - Open the ErikrafT Drop CLI experience");
    println!("  donate         - Open the premium terminal donation screen");
    println!("  version        - Display version information");
    println!("  version --check- Check for updates online");
    println!("  self-update    - Upgrade to the latest premium release automatically");
    println!("  help           - Display this help message");
    println!("  exit / quit    - Exit the interactive CLI");
}

/// Initiates the primary interactive prompt loop when the executable is launched without arguments.
pub fn start_interactive_cli() {
    println!("{}", ascii_logo());
    println!("=================================================");
    println!(" Type 'browse' to surf the web/local files,");
    println!(" Type 'donate' to support the project, or");
    println!(" Type 'help' to see other commands.");
    println!("=================================================");

    let stdin = io::stdin();
    loop {
        print!("iSearch> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            break;
        }

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye!");
                    break;
                } else if trimmed == "help" {
                    print_help();
                } else if trimmed == "donate" || trimmed == "isearch donate" {
                    let config = load_config();
                    if let Err(e) = run_donation_tui(config) {
                        eprintln!("Error launching donation screen: {}", e);
                    }
                } else if trimmed == "browse"
                    || trimmed == "isearch browse"
                    || trimmed.starts_with("browse ")
                {
                    if let Err(e) = browser::ui::run_browser_tui() {
                        eprintln!("Error launching browser: {}", e);
                    }
                } else if trimmed.starts_with("open ") || trimmed.starts_with("isearch open ") {
                    let target = trimmed
                        .strip_prefix("isearch open ")
                        .or_else(|| trimmed.strip_prefix("open "))
                        .unwrap_or("");
                    if let Err(e) = browser::ui::run_browser_tui_with_url(target) {
                        eprintln!("Error opening resource: {}", e);
                    }
                } else if looks_like_resource(trimmed) {
                    if let Err(e) = browser::ui::run_browser_tui_with_url(trimmed) {
                        eprintln!("Error opening resource: {}", e);
                    }
                } else if trimmed == "erikraft-drop" || trimmed == "isearch erikraft-drop" {
                    if let Err(e) = cli_web::run_erikraft_drop() {
                        eprintln!("Error launching ErikrafT Drop client: {}", e);
                    }
                } else if trimmed == "version" || trimmed == "isearch version" {
                    browser::updater::print_version(false);
                } else if trimmed == "version --check" || trimmed == "isearch version --check" {
                    browser::updater::print_version(true);
                } else if trimmed == "self-update" || trimmed == "isearch self-update" {
                    if let Err(e) = browser::updater::perform_self_update() {
                        eprintln!("Error performing self-update: {}", e);
                    }
                } else if trimmed.is_empty() {
                    // Do nothing
                } else {
                    println!("Unknown command: '{}'. Type 'help', 'browse', 'version', 'self-update' or 'donate'.", trimmed);
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse commands
    if args.len() > 1 {
        let joined_args = args[1..].join(" ");
        if joined_args == "donate" || joined_args == "isearch donate" {
            let config = load_config();
            if let Err(e) = run_donation_tui(config) {
                eprintln!("Error launching donation screen: {}", e);
                std::process::exit(1);
            }
        } else if joined_args == "browse"
            || joined_args == "isearch browse"
            || joined_args.starts_with("browse ")
        {
            if let Err(e) = browser::ui::run_browser_tui() {
                eprintln!("Error launching browser: {}", e);
                std::process::exit(1);
            }
        } else if joined_args.starts_with("open ") || joined_args.starts_with("isearch open ") {
            let target = joined_args
                .strip_prefix("isearch open ")
                .or_else(|| joined_args.strip_prefix("open "))
                .unwrap_or("");
            if let Err(e) = browser::ui::run_browser_tui_with_url(target) {
                eprintln!("Error opening resource: {}", e);
                std::process::exit(1);
            }
        } else if looks_like_resource(&joined_args) {
            if let Err(e) = browser::ui::run_browser_tui_with_url(&joined_args) {
                eprintln!("Error opening resource: {}", e);
                std::process::exit(1);
            }
        } else if joined_args == "erikraft-drop" || joined_args == "isearch erikraft-drop" {
            if let Err(e) = cli_web::run_erikraft_drop() {
                eprintln!("Error launching ErikrafT Drop client: {}", e);
                std::process::exit(1);
            }
        } else if joined_args == "version" || joined_args == "isearch version" {
            browser::updater::print_version(false);
        } else if joined_args == "version --check"
            || joined_args == "isearch version --check"
            || joined_args == "--check"
        {
            browser::updater::print_version(true);
        } else if joined_args == "self-update" || joined_args == "isearch self-update" {
            if let Err(e) = browser::updater::perform_self_update() {
                eprintln!("Error performing self-update: {}", e);
                std::process::exit(1);
            }
        } else if joined_args == "help" || joined_args == "--help" || joined_args == "-h" {
            print_help();
        } else {
            eprintln!("Unknown arguments: '{}'", joined_args);
            print_help();
            std::process::exit(1);
        }
    } else {
        // No arguments - start interactive CLI
        start_interactive_cli();
    }
}

fn looks_like_resource(input: &str) -> bool {
    matches!(
        uri::parse(input).scheme,
        uri::UriScheme::Http
            | uri::UriScheme::Https
            | uri::UriScheme::Ws
            | uri::UriScheme::Wss
            | uri::UriScheme::Ftp
            | uri::UriScheme::Ftps
            | uri::UriScheme::Sftp
            | uri::UriScheme::Smtp
            | uri::UriScheme::Smtps
            | uri::UriScheme::Data
            | uri::UriScheme::Blob
            | uri::UriScheme::File
            | uri::UriScheme::LocalPath
    )
}
