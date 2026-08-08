use crate::branding::gradient_spans;
use crate::browser::terminal_media::TerminalCapabilities;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::EnableMouseCapture;
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::Line, widgets::{Block, BorderType, Borders, Paragraph}, Terminal};
use std::io::{self, Write};
use super::metadata::SiteMetadata;

/// Simple TUI renderer for ErikrafT Drop client inside iSearch CLI™.
pub fn run_drop_tui(meta: &SiteMetadata) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Minimal UI loop: render a static screen until user presses q or Esc
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(size);

            let mut top_spans = gradient_spans(" iSearch CLI™ ");
            top_spans.push(ratatui::text::Span::raw(" │ "));
            top_spans.push(ratatui::text::Span::raw("ErikrafT Drop"));
            let top = Paragraph::new(Line::from(top_spans)).block(
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded),
            );
            f.render_widget(top, chunks[0]);

            // Main content: devices and actions
            let body = "\nConnected devices:\n\n  >_  Chrome\n  >_  iSearch CLI™\n\nFiles:\n\n  [ Send File ]   [ Receive File ]\n\nUse Up/Down and Enter to interact (Q or Esc to quit).";
            let body_p = Paragraph::new(body)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
            f.render_widget(body_p, chunks[1]);

            let status = Paragraph::new(Line::from(vec![
                ratatui::text::Span::styled("[Q/Esc] Exit", Style::default().fg(Color::Gray)),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(format!("CLI Support: {}", meta.version.clone().unwrap_or_else(|| "n/a".to_string()))),
            ]));
            f.render_widget(status, chunks[2]);
        })?;

        // Simple blocking read from stdin for a single byte - non-ideal but sufficient for scaffold
        // Wait for user key to exit
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
