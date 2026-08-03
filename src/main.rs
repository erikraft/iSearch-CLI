pub mod config;
pub mod pix;
pub mod utils;
pub mod ui;

use std::env;
use std::io::{self, Write};
use config::load_config;
use ui::run_donation_tui;

fn print_help() {
    println!("iSearch CLI™ - Version 0.1.0");
    println!("Available commands:");
    println!("  donate         - Open the premium terminal donation screen");
    println!("  help           - Display this help message");
    println!("  exit / quit    - Exit the interactive CLI");
}

fn start_interactive_cli() {
    println!("╭────────────────────────────────────────────╮");
    println!("│             iSearch CLI™ Terminal          │");
    println!("├────────────────────────────────────────────┤");
    println!("│ Type 'donate' to support the project, or   │");
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
                match trimmed {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "help" => {
                        print_help();
                    }
                    "donate" | "isearch donate" => {
                        let config = load_config();
                        if let Err(e) = run_donation_tui(config) {
                            eprintln!("Error launching donation screen: {}", e);
                        }
                    }
                    "" => {}
                    _ => {
                        println!("Unknown command: '{}'. Type 'help' or 'donate'.", trimmed);
                    }
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
            return;
        } else if joined_args == "help" || joined_args == "--help" || joined_args == "-h" {
            print_help();
            return;
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
