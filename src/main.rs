pub mod config;
pub mod pix;
pub mod utils;
pub mod ui;
pub mod browser;

use std::env;
use std::io::{self, Write};
use config::load_config;
use ui::run_donation_tui;

fn print_help() {
    println!("iSearch CLI™ - Version 0.1.0");
    println!("Available commands:");
    println!("  browse         - Open the premium multi-engine interactive terminal browser");
    println!("  donate         - Open the premium terminal donation screen");
    println!("  help           - Display this help message");
    println!("  exit / quit    - Exit the interactive CLI");
}

fn start_interactive_cli() {
    println!("╭────────────────────────────────────────────╮");
    println!("│             iSearch CLI™ Terminal          │");
    println!("├────────────────────────────────────────────┤");
    println!("│ Type 'browse' to surf the web/local files, │");
    println!("│ type 'donate' to support the project, or   │");
    println!("│ type 'help' to see other commands.         │");
    println!("╰────────────────────────────────────────────╯");

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
                } else if trimmed == "browse" || trimmed == "isearch browse" || trimmed.starts_with("browse ") {
                    if let Err(e) = browser::ui::run_browser_tui() {
                        eprintln!("Error launching browser: {}", e);
                    }
                } else if trimmed.is_empty() {
                    // Do nothing
                } else {
                    println!("Unknown command: '{}'. Type 'help', 'browse' or 'donate'.", trimmed);
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
        // Handle "isearch donate" or "donate"
        let joined_args = args[1..].join(" ");
        if joined_args == "donate" || joined_args == "isearch donate" {
            let config = load_config();
            if let Err(e) = run_donation_tui(config) {
                eprintln!("Error launching donation screen: {}", e);
                std::process::exit(1);
            }
        } else if joined_args == "browse" || joined_args == "isearch browse" || joined_args.starts_with("browse ") {
            if let Err(e) = browser::ui::run_browser_tui() {
                eprintln!("Error launching browser: {}", e);
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
