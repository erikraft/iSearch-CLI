use crate::browser::core::{BrowserCore, EngineType, PageContent, BrowserError};
use crate::browser::native::{render_html_to_lines, render_markdown_to_lines, CssStyle};
use crate::browser::terminal_media::TerminalCapabilities;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap},
    Terminal, Frame,
};
use std::io;

pub struct BrowserTab {
    pub url: String,
    pub title: String,
    pub content: Option<PageContent>,
    pub history: Vec<String>,
    pub history_idx: usize,
}

impl BrowserTab {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            title: "New Tab".to_string(),
            content: None,
            history: vec![url.to_string()],
            history_idx: 0,
        }
    }
}

pub struct BrowserApp {
    pub core: BrowserCore,
    pub tabs: Vec<BrowserTab>,
    pub active_tab_idx: usize,
    pub show_help: bool,
    pub input_mode: bool,
    pub address_buffer: String,
    pub scroll_offset: usize,
    pub status_message: String,
    pub term_caps: TerminalCapabilities,
}

impl BrowserApp {
    pub fn new() -> Self {
        let mut app = Self {
            core: BrowserCore::new(),
            tabs: vec![BrowserTab::new("https://www.google.com")],
            active_tab_idx: 0,
            show_help: false,
            input_mode: false,
            address_buffer: "https://www.google.com".to_string(),
            scroll_offset: 0,
            status_message: "Ready".to_string(),
            term_caps: TerminalCapabilities::detect(),
        };
        // Load initial page
        let _ = app.load_current_page();
        app
    }

    pub fn load_current_page(&mut self) -> Result<(), BrowserError> {
        self.scroll_offset = 0;
        let url = self.tabs[self.active_tab_idx].url.clone();
        self.status_message = format!("Loading {}...", url);

        match self.core.navigate(&url) {
            Ok(content) => {
                let title = match &content {
                    PageContent::Html { title, .. } => title.clone(),
                    PageContent::Markdown { title, .. } => title.clone(),
                    PageContent::Directory { path, .. } => path.to_string_lossy().to_string(),
                    PageContent::FilePreview { path, .. } => path.to_string_lossy().to_string(),
                    PageContent::PdfPreview { title, .. } => title.clone(),
                    PageContent::ArchivePreview { path, .. } => format!("Archive: {}", path.to_string_lossy()),
                    PageContent::ImagePreview { path, .. } => format!("Image: {}", path.to_string_lossy()),
                    PageContent::AnsiText { title, .. } => title.clone(),
                };

                self.tabs[self.active_tab_idx].title = title;
                self.tabs[self.active_tab_idx].content = Some(content);
                self.status_message = format!("Loaded.");
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                Err(e)
            }
        }
    }
}

pub fn run_browser_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = BrowserApp::new();
    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error running browser: {}", err);
    }
    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut BrowserApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    if app.input_mode {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = false;
                            }
                            KeyCode::Enter => {
                                app.input_mode = false;
                                let new_url = app.address_buffer.trim().to_string();
                                if !new_url.is_empty() {
                                    app.tabs[app.active_tab_idx].url = new_url.clone();
                                    let mut hist = app.tabs[app.active_tab_idx].history.clone();
                                    hist.push(new_url);
                                    app.tabs[app.active_tab_idx].history = hist;
                                    app.tabs[app.active_tab_idx].history_idx += 1;
                                    let _ = app.load_current_page();
                                }
                            }
                            KeyCode::Char(c) => {
                                app.address_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                app.address_buffer.pop();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                return Ok(());
                            }
                            KeyCode::Char('l') | KeyCode::Char('L') => {
                                app.input_mode = true;
                                app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                let _ = app.load_current_page();
                            }
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                // Toggle engine
                                let next = match app.core.current_engine {
                                    EngineType::Native => EngineType::Chromium,
                                    EngineType::Chromium => EngineType::Native,
                                };
                                app.core.set_engine(next);
                                app.status_message = format!("Switched engine to {:?}", next);
                                let _ = app.load_current_page();
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                // New tab
                                app.tabs.push(BrowserTab::new("https://www.google.com"));
                                app.active_tab_idx = app.tabs.len() - 1;
                                app.address_buffer = "https://www.google.com".to_string();
                                let _ = app.load_current_page();
                            }
                            KeyCode::Tab => {
                                // Next tab
                                app.active_tab_idx = (app.active_tab_idx + 1) % app.tabs.len();
                                app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                self_render_viewport(app);
                            }
                            KeyCode::Char('w') | KeyCode::Char('W') => {
                                // Close tab
                                if app.tabs.len() > 1 {
                                    app.tabs.remove(app.active_tab_idx);
                                    if app.active_tab_idx >= app.tabs.len() {
                                        app.active_tab_idx = app.tabs.len() - 1;
                                    }
                                    app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                    self_render_viewport(app);
                                }
                            }
                            KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Left => {
                                // Back in history
                                let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                if current_idx > 0 {
                                    app.tabs[app.active_tab_idx].history_idx -= 1;
                                    let prev_url = app.tabs[app.active_tab_idx].history[current_idx - 1].clone();
                                    app.tabs[app.active_tab_idx].url = prev_url;
                                    let _ = app.load_current_page();
                                } else {
                                    app.status_message = "No back history".to_string();
                                }
                            }
                            KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Right => {
                                // Forward in history
                                let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                let hist_len = app.tabs[app.active_tab_idx].history.len();
                                if current_idx + 1 < hist_len {
                                    app.tabs[app.active_tab_idx].history_idx += 1;
                                    let next_url = app.tabs[app.active_tab_idx].history[current_idx + 1].clone();
                                    app.tabs[app.active_tab_idx].url = next_url;
                                    let _ = app.load_current_page();
                                } else {
                                    app.status_message = "No forward history".to_string();
                                }
                            }
                            KeyCode::Char('h') | KeyCode::Char('H') => {
                                app.show_help = !app.show_help;
                            }
                            KeyCode::Up => {
                                if app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                app.scroll_offset += 1;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                        let r = mouse_event.row;
                        let c = mouse_event.column;

                        if r == 1 {
                            // Clicks on Top Bar / Tabs
                            if c >= 20 && c <= 80 {
                                let click_idx = ((c - 20) / 18) as usize;
                                if click_idx < app.tabs.len() {
                                    app.active_tab_idx = click_idx;
                                    app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                    self_render_viewport(app);
                                }
                            }
                        } else if r >= 3 && r <= 5 {
                            // Click on Address bar -> Focus input mode
                            app.input_mode = true;
                            app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn self_render_viewport(app: &mut BrowserApp) {
    app.scroll_offset = 0;
}

pub fn ui(f: &mut Frame, app: &mut BrowserApp) {
    let size = f.area();

    // Theme Colors
    let primary_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let highlight_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let border_style = Style::default().fg(Color::DarkGray);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Bar (Tabs)
            Constraint::Length(3), // Address Bar
            Constraint::Min(1),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(size);

    // 1. Top bar (Tabs)
    let mut tabs_spans = Vec::new();
    for (i, tab) in app.tabs.iter().enumerate() {
        let style = if i == app.active_tab_idx {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        tabs_spans.push(Span::styled(format!(" [ Tab {}: {} ] ", i + 1, tab.title), style));
    }
    let engine_mode = format!(" Engine: {:?} ", app.core.current_engine);
    let mut top_spans = vec![
        Span::styled(" iSearch Browser™ ", primary_style),
        Span::raw(" │ "),
    ];
    top_spans.extend(tabs_spans);
    top_spans.push(Span::raw(" │ "));
    top_spans.push(Span::styled(engine_mode, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));

    let top_bar = Paragraph::new(Line::from(top_spans))
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style));
    f.render_widget(top_bar, chunks[0]);

    // 2. Address bar
    let addr_border_style = if app.input_mode { highlight_style } else { border_style };
    let addr_text = if app.input_mode {
        format!("{}█", app.address_buffer)
    } else {
        app.tabs[app.active_tab_idx].url.clone()
    };
    let addr_bar = Paragraph::new(Line::from(vec![
        Span::styled(" URL: ", Style::default().fg(Color::Gray)),
        Span::raw(addr_text),
    ]))
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(addr_border_style));
    f.render_widget(addr_bar, chunks[1]);

    // 3. Main content viewport
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let content_area = content_block.inner(chunks[2]);
    f.render_widget(content_block, chunks[2]);

    if app.show_help {
        let help_text = "\
iSearch CLI™ Terminal Browser Help\n\n\
Keybindings:\n\
  [Esc / Q]      Exit the browser\n\
  [L]            Focus address bar (type URL or search term)\n\
  [R]            Reload current page\n\
  [E]            Toggle between NATIVE and CHROMIUM engines\n\
  [T]            Open a new tab\n\
  [Tab]          Switch to next tab\n\
  [W]            Close active tab\n\
  [H]            Toggle this help screen\n\
  [Up / Down]    Scroll viewport up/down\n\n\
Under the hood:\n\
  - Native Engine uses highly optimized Rust parsers for super fast, low-memory offline browsing.\n\
  - Chromium Engine runs Chrome/Chromium headlessly for complex Javascript-heavy modern apps (React, Vue, Next.js).";

        let help_p = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        f.render_widget(help_p, content_area);
        return;
    }

    // Render loaded PageContent
    let mut display_lines = Vec::new();
    if let Some(content) = &app.tabs[app.active_tab_idx].content {
        match content {
            PageContent::Html { parsed_nodes, .. } => {
                display_lines = render_html_to_lines(parsed_nodes, content_area.width as usize, CssStyle::default());
            }
            PageContent::Markdown { raw_md, .. } => {
                display_lines = render_markdown_to_lines(raw_md, content_area.width as usize);
            }
            PageContent::Directory { path, entries } => {
                display_lines.push(Line::from(vec![
                    Span::styled("Directory Listing: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(path.to_string_lossy().to_string()),
                ]));
                display_lines.push(Line::raw(""));
                for (name, is_dir) in entries {
                    let icon = if *is_dir { "📁 " } else { "📄 " };
                    let name_style = if *is_dir { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default() };
                    display_lines.push(Line::from(vec![
                        Span::raw(icon),
                        Span::styled(name.clone(), name_style),
                    ]));
                }
            }
            PageContent::FilePreview { content, .. } => {
                for line in content.lines() {
                    display_lines.push(Line::raw(line.to_string()));
                }
            }
            PageContent::PdfPreview { metadata, text_preview, .. } => {
                display_lines.push(Line::from(vec![
                    Span::styled("PDF Document Preview", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                ]));
                display_lines.push(Line::raw(""));
                for (k, v) in metadata {
                    display_lines.push(Line::from(vec![
                        Span::styled(format!("{}: ", k), Style::default().fg(Color::Yellow)),
                        Span::raw(v),
                    ]));
                }
                display_lines.push(Line::raw(""));
                display_lines.push(Line::raw(text_preview.clone()));
            }
            PageContent::ArchivePreview { files, .. } => {
                display_lines.push(Line::from(vec![
                    Span::styled("Archive Contents (ZIP):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                ]));
                display_lines.push(Line::raw(""));
                for file in files {
                    display_lines.push(Line::from(vec![
                        Span::raw("📦 "),
                        Span::styled(file.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
            PageContent::ImagePreview { raw_bytes, .. } => {
                display_lines = crate::browser::terminal_media::render_image_to_lines(
                    raw_bytes,
                    content_area.width as u32,
                    content_area.height as u32,
                    &app.term_caps,
                );
            }
            _ => {
                display_lines.push(Line::raw("Preview not supported for this media type."));
            }
        }
    } else {
        display_lines.push(Line::raw("No content loaded. Press [L] and enter a URL to start browsing."));
    }

    // Scroll handling and rendering
    let max_scroll = if display_lines.len() > content_area.height as usize {
        display_lines.len() - content_area.height as usize
    } else {
        0
    };
    let offset = std::cmp::min(app.scroll_offset, max_scroll);
    app.scroll_offset = offset;

    let scrolled_lines: Vec<Line<'_>> = display_lines.into_iter().skip(offset).take(content_area.height as usize).collect();
    let viewport_p = Paragraph::new(scrolled_lines);
    f.render_widget(viewport_p, content_area);

    // 4. Status Bar
    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" [Esc/Q: Exit] ", Style::default().fg(Color::Gray)),
        Span::styled(" [L: Go to] ", Style::default().fg(Color::Gray)),
        Span::styled(" [E: Switch Engine] ", Style::default().fg(Color::Gray)),
        Span::styled(" [T: New Tab] ", Style::default().fg(Color::Gray)),
        Span::styled(" [Tab: Next Tab] ", Style::default().fg(Color::Gray)),
        Span::styled(" [H: Toggle Help] ", Style::default().fg(Color::Gray)),
        Span::raw(" │ Status: "),
        Span::styled(app.status_message.clone(), Style::default().fg(Color::Yellow)),
    ]))
    .style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(status_bar, chunks[3]);
}
