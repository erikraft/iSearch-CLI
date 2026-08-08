use crate::branding::gradient_spans;
use crate::browser::terminal_media::TerminalCapabilities;
use crossterm::execute;
use crossterm::event::EnableMouseCapture;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, Paragraph}, Terminal};
use std::io::{self, Write};
use super::metadata::SiteMetadata;
use crate::cli_web::parser::CliDocument;

/// Simple TUI renderer for ErikrafT Drop client inside iSearch CLI™.
pub fn run_drop_tui(meta: &SiteMetadata, document: &CliDocument) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let mut top_spans = gradient_spans(" iSearch CLI™ ");
            top_spans.push(Span::raw(" │ "));
            top_spans.push(Span::styled("ErikrafT Drop", Style::default().add_modifier(Modifier::BOLD)));
            let top = Paragraph::new(Line::from(top_spans)).block(
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded),
            );
            f.render_widget(top, chunks[0]);

            let mut body_lines = vec![
                Line::from(Span::styled("ErikrafT Drop CLI", Style::default().add_modifier(Modifier::BOLD)));
                Line::from(Span::raw("")),
            ];

            let info_lines = vec![
                format!("Site CLI version: {}", meta.version.as_deref().unwrap_or("unknown")),
                format!("Server API version: {}", document.server_version.as_deref().unwrap_or("unknown")),
                format!("Client type: {}", document.client_type.as_deref().unwrap_or("isearch-cli")),
                format!("Signaling server: {}", document.signaling_server.as_deref().unwrap_or("local")),
                format!("WebSocket: {}", document.ws_url.as_deref().unwrap_or("pending")),
                format!("Features: {}", if document.features.is_empty() { "basic CLI".to_string() } else { document.features.join(", ") }),
            ];

            for line_text in info_lines {
                body_lines.push(Line::from(Span::raw(line_text)));
            }

            if let Some(message) = &document.message {
                body_lines.push(Line::from(Span::raw("")));
                body_lines.push(Line::from(Span::styled("Message:", Style::default().add_modifier(Modifier::BOLD))));
                body_lines.push(Line::from(Span::raw(message)));
            }

            let body_paragraph = Paragraph::new(body_lines)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
            f.render_widget(body_paragraph, chunks[1]);

            let footer = Paragraph::new(Line::from(vec![
                Span::styled("[Q/Esc] Exit", Style::default().fg(Color::Gray)),
                Span::raw(" │ "),
                Span::raw("Real CLI mode enabled. Connect files with ErikrafT Drop over the same network."),
            ]));
            f.render_widget(footer, chunks[2]);
        })?;

        use crossterm::event::{self, Event, KeyCode};
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
