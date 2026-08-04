//! iSearch CLI™: Premium Terminal Browser & Interactive Utility Suite.
//!
//! # Purpose
//! This crate implements a full-featured terminal web and offline browser with advanced capabilities
//! including history trackers, secure/anonymous profiles, adblocking filters, customizable theme schemes,
//! ZIP exploration, 3D interactive renders, and automated PIX donation systems.
//!
//! # Architecture
//! The CLI operates primarily through an interactive command shell loop.
//! * [config] manages parsing preferences from local config configurations.
//! * [pix] implements static EMV Co PIX payload encoding requirements.
//! * [utils] formats QR terminal graphics and bridges system clipboard APIs.
//! * [ui] builds interactive ratatui screens for donation features.
//! * [browser] manages terminal browser backend layers.

#![deny(missing_docs)]

pub mod config;
pub mod pix;
pub mod utils;
pub mod ui;
pub mod browser;

use std::env;
use std::io::{self, Write};
use config::load_config;
use ui::run_donation_tui;

/// Formats and outputs the global list of interactive commands.
pub fn print_help() {
    println!("iSearch CLI™ - Version 0.1.0");
    println!("Available commands:");
    println!("  browse         - Open the premium multi-engine interactive terminal browser");
    println!("  donate         - Open the premium terminal donation screen");
    println!("  help           - Display this help message");
    println!("  exit / quit    - Exit the interactive CLI");
}

/// Initiates the primary interactive prompt loop when the executable is launched without arguments.
pub fn start_interactive_cli() {
    println!("░▀█▀░█▀▀░█▀▀░█▀█░█▀▄░█▀▀░█░█░░░█▀▀░█░░░▀█▀");
    println!("░░█░░▀▀█░█▀▀░█▀█░█▀▄░█░░░█▀█░░░█░░░█░░░░█░");
    println!("░▀▀▀░▀▀▀░▀▀▀░▀░▀░▀░▀░▀▀▀░▀░▀░░░▀▀▀░▀▀▀░▀▀▀");
    println!("                 iSearch CLI™");
    println!("=================================================");
    println!(" Type 'browse' to surf the web/local files,");
    println!(" type 'donate' to support the project, or");
    println!(" type 'help' to see other commands.");
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
