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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerStep {
    Main,
    DownloadPage,
    InstallCommands,
    ConfigurePath,
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
    pub show_installer: bool,
    pub installer_step: InstallerStep,
    pub custom_path_input: String,
    pub installer_status: String,
    pub theme: crate::browser::theme::AppTheme,
    pub theme_preset: crate::browser::theme::ThemePreset,
    pub bookmarks: Vec<String>,
    pub history_list: Vec<String>,
    pub downloads: Vec<(String, f32, String)>, // (filename, progress, status)
    pub show_bookmarks: bool,
    pub show_history: bool,
    pub download_active: bool,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self::new()
    }
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
            show_installer: false,
            installer_step: InstallerStep::Main,
            custom_path_input: String::new(),
            installer_status: "Ready".to_string(),
            theme: crate::browser::theme::AppTheme::from_preset(crate::browser::theme::ThemePreset::Default),
            theme_preset: crate::browser::theme::ThemePreset::Default,
            bookmarks: vec!["https://www.rust-lang.org".to_string(), "https://news.ycombinator.com".to_string()],
            history_list: vec!["https://www.google.com".to_string()],
            downloads: Vec::new(),
            show_bookmarks: false,
            show_history: false,
            download_active: false,
        };
        // Load initial page
        let _ = app.load_current_page();
        app
    }

    pub fn load_current_page(&mut self) -> Result<(), BrowserError> {
        self.scroll_offset = 0;
        let url = self.tabs[self.active_tab_idx].url.clone();

        // Android (Termux) target - force native engine and never show Chromium error
        #[cfg(target_os = "android")]
        {
            self.core.current_engine = EngineType::Native;
        }

        // Check if Chromium is required but not available
        if self.core.current_engine == EngineType::Chromium && !self.core.chromium_engine.is_available() {
            #[cfg(target_os = "android")]
            {
                // Never show Chromium errors/assistant on Android
                self.core.current_engine = EngineType::Native;
            }
            #[cfg(not(target_os = "android"))]
            {
                self.show_installer = true;
                self.installer_step = InstallerStep::Main;
                self.status_message = "Chromium required but not available.".to_string();
                return Err(BrowserError::ChromiumNotAvailable("Chromium not found".to_string()));
            }
        }

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
                    PageContent::Mesh3DPreview { title, .. } => format!("3D Model: {}", title),
                };

                self.tabs[self.active_tab_idx].title = title;
                self.tabs[self.active_tab_idx].content = Some(content);
                self.status_message = "Loaded.".to_string();
                if !self.history_list.contains(&url) {
                    self.history_list.push(url);
                }
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
        if app.download_active {
            for d in &mut app.downloads {
                if d.1 < 1.0 {
                    d.1 += 0.2;
                    d.2 = format!("Downloading... {:.0}%", d.1 * 100.0);
                    if d.1 >= 1.0 {
                        d.1 = 1.0;
                        d.2 = "Finished".to_string();
                        app.status_message = format!("Downloaded successfully: {}", d.0);
                    }
                }
            }
            if app.downloads.iter().all(|d| d.1 >= 1.0) {
                app.download_active = false;
            }
        }

        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    if app.show_installer {
                        match app.installer_step {
                            InstallerStep::Main => {
                                match key.code {
                                    KeyCode::Char('1') => {
                                        app.installer_step = InstallerStep::DownloadPage;
                                    }
                                    KeyCode::Char('2') => {
                                        app.installer_step = InstallerStep::InstallCommands;
                                    }
                                    KeyCode::Char('3') => {
                                        app.installer_step = InstallerStep::ConfigurePath;
                                        app.custom_path_input = String::new();
                                    }
                                    KeyCode::Char('4') | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        app.show_installer = false;
                                        app.core.set_engine(EngineType::Native);
                                        app.status_message = "Using Native Rendering Mode".to_string();
                                        let _ = app.load_current_page();
                                    }
                                    _ => {}
                                }
                            }
                            InstallerStep::DownloadPage | InstallerStep::InstallCommands => {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('b') | KeyCode::Char('B') => {
                                        app.installer_step = InstallerStep::Main;
                                    }
                                    _ => {}
                                }
                            }
                            InstallerStep::ConfigurePath => {
                                match key.code {
                                    KeyCode::Esc => {
                                        app.installer_step = InstallerStep::Main;
                                    }
                                    KeyCode::Enter => {
                                        let path = std::path::PathBuf::from(app.custom_path_input.trim());
                                        if path.exists() {
                                            app.core.chromium_engine.set_manual_path(path);
                                            app.show_installer = false;
                                            app.installer_status = "Custom path configured!".to_string();
                                            let _ = app.load_current_page();
                                        } else {
                                            app.installer_status = "Path does not exist!".to_string();
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        app.custom_path_input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        app.custom_path_input.pop();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if app.show_bookmarks {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('o') | KeyCode::Char('O') => {
                                app.show_bookmarks = false;
                            }
                            KeyCode::Up => {
                                if app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.scroll_offset < app.bookmarks.len().saturating_sub(1) {
                                    app.scroll_offset += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if !app.bookmarks.is_empty() {
                                    let selected_url = app.bookmarks[app.scroll_offset.min(app.bookmarks.len() - 1)].clone();
                                    app.tabs[app.active_tab_idx].url = selected_url;
                                    app.show_bookmarks = false;
                                    let _ = app.load_current_page();
                                }
                            }
                            _ => {}
                        }
                    } else if app.show_history {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.show_history = false;
                            }
                            KeyCode::Up => {
                                if app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.scroll_offset < app.history_list.len().saturating_sub(1) {
                                    app.scroll_offset += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if !app.history_list.is_empty() {
                                    let selected_url = app.history_list[app.scroll_offset.min(app.history_list.len() - 1)].clone();
                                    app.tabs[app.active_tab_idx].url = selected_url;
                                    app.show_history = false;
                                    let _ = app.load_current_page();
                                }
                            }
                            _ => {}
                        }
                    } else if app.input_mode {
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
                                if app.download_active {
                                    app.download_active = false;
                                } else {
                                    return Ok(());
                                }
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
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_x(0.1);
                                } else {
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
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_x(-0.1);
                                }
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_y(-0.1);
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_y(0.1);
                                }
                            }
                            KeyCode::Char('b') | KeyCode::Char('B') => {
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
                            KeyCode::Char('f') | KeyCode::Char('F') => {
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
                            KeyCode::Char('o') | KeyCode::Char('O') => {
                                app.show_bookmarks = !app.show_bookmarks;
                                app.show_history = false;
                                app.scroll_offset = 0;
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.show_history = !app.show_history;
                                app.show_bookmarks = false;
                                app.scroll_offset = 0;
                            }
                            KeyCode::Char('g') | KeyCode::Char('G') => {
                                let current_url = app.tabs[app.active_tab_idx].url.clone();
                                if !app.bookmarks.contains(&current_url) {
                                    app.bookmarks.push(current_url);
                                    app.status_message = "Bookmarked page!".to_string();
                                } else {
                                    app.status_message = "Already bookmarked".to_string();
                                }
                            }
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                // Trigger simulated/mock download with progress
                                let filename = app.tabs[app.active_tab_idx].url.split('/').next_back().unwrap_or("index.html").to_string();
                                let clean_filename = if filename.is_empty() || filename.contains('?') { "index.html".to_string() } else { filename };
                                app.downloads.push((clean_filename.clone(), 0.0, "Initiating download...".to_string()));
                                app.download_active = true;
                                app.status_message = format!("Downloading {}...", clean_filename);
                            }
                            KeyCode::Char('k') | KeyCode::Char('K') => {
                                // Cycle themes
                                let next_preset = match app.theme_preset {
                                    crate::browser::theme::ThemePreset::Default => crate::browser::theme::ThemePreset::Dracula,
                                    crate::browser::theme::ThemePreset::Dracula => crate::browser::theme::ThemePreset::Nord,
                                    crate::browser::theme::ThemePreset::Nord => crate::browser::theme::ThemePreset::Ocean,
                                    crate::browser::theme::ThemePreset::Ocean => crate::browser::theme::ThemePreset::Monokai,
                                    crate::browser::theme::ThemePreset::Monokai => crate::browser::theme::ThemePreset::Light,
                                    crate::browser::theme::ThemePreset::Light => crate::browser::theme::ThemePreset::Default,
                                };
                                app.theme_preset = next_preset;
                                app.theme = crate::browser::theme::AppTheme::from_preset(next_preset);
                                app.status_message = format!("Switched theme to {}", app.theme.name);
                            }
                            KeyCode::Up => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_x(0.1);
                                } else if app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_x(-0.1);
                                } else {
                                    app.scroll_offset += 1;
                                }
                            }
                            KeyCode::Left => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_y(-0.1);
                                } else {
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
                            }
                            KeyCode::Right => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) = &mut app.tabs[app.active_tab_idx].content {
                                    mesh.rotate_y(0.1);
                                } else {
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
                            }
                            KeyCode::Enter => {
                                // If inside zip archive browser, we can select a file and navigate to it!
                                if let Some(PageContent::ArchivePreview { path, files }) = &app.tabs[app.active_tab_idx].content {
                                    if app.scroll_offset < files.len() {
                                        let selected_file = &files[app.scroll_offset];
                                        let archive_url = format!("{}::{}", path.to_string_lossy(), selected_file);
                                        app.tabs[app.active_tab_idx].url = archive_url.clone();
                                        let mut hist = app.tabs[app.active_tab_idx].history.clone();
                                        hist.push(archive_url);
                                        app.tabs[app.active_tab_idx].history = hist;
                                        app.tabs[app.active_tab_idx].history_idx += 1;
                                        let _ = app.load_current_page();
                                    }
                                }
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
                            if (20..=80).contains(&c) {
                                let click_idx = ((c - 20) / 18) as usize;
                                if click_idx < app.tabs.len() {
                                    app.active_tab_idx = click_idx;
                                    app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                    self_render_viewport(app);
                                }
                            }
                        } else if (3..=5).contains(&r) {
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

    // Theme Colors from Theme System
    let primary_style = Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD);
    let highlight_style = Style::default().fg(app.theme.highlight).add_modifier(Modifier::BOLD);
    let border_style = Style::default().fg(app.theme.border);

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

    if app.show_installer {
        let installer_content = match app.installer_step {
            InstallerStep::Main => {
                "\
Chromium-compatible browser not found.\n\n\
Modern JavaScript websites require a Chromium-compatible rendering backend.\n\n\
Supported browsers:\n\
✓ Google Chrome\n\
✓ Chromium\n\
✓ Microsoft Edge\n\
✓ Brave\n\
✓ Vivaldi\n\
✓ Opera\n\
✓ Ungoogled Chromium\n\n\
Choose:\n\
[1] Open the official download page\n\
[2] Show installation commands (when available)\n\
[3] Configure an existing executable\n\
[4] Continue using Native Rendering Mode".to_string()
            }
            InstallerStep::DownloadPage => {
                "\
Official Browser Download Pages:\n\n\
- Google Chrome: https://www.google.com/chrome/\n\
- Chromium: https://www.chromium.org/getting-involved/download-chromium/\n\
- Microsoft Edge: https://www.microsoft.com/edge/\n\
- Brave: https://brave.com/\n\
- Vivaldi: https://vivaldi.com/\n\
- Opera: https://www.opera.com/\n\n\
Press [Backspace] or [B] to go back.".to_string()
            }
            InstallerStep::InstallCommands => {
                app.core.chromium_engine.get_guided_install_instructions() + "\n\nPress [Backspace] or [B] to go back."
            }
            InstallerStep::ConfigurePath => {
                format!(
                    "Configure existing browser executable path:\n\n\
                     Type/paste absolute path and press [Enter]:\n\n\
                     > {}\n\n\
                     Status: {}\n\n\
                     Press [Esc] to go back.",
                    app.custom_path_input, app.installer_status
                )
            }
        };

        let installer_p = Paragraph::new(installer_content)
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: false });
        f.render_widget(installer_p, content_area);
        return;
    }

    if app.show_bookmarks {
        let mut bookmark_content = "iSearch CLI™ Bookmarks:\n\n\
                                    Scroll with [Up/Down] and Press [Enter] to go to page:\n\n".to_string();
        for (idx, b) in app.bookmarks.iter().enumerate() {
            let indicator = if idx == app.scroll_offset { "➔  " } else { "   " };
            let style_str = if idx == app.scroll_offset { format!("[ {} ]", b) } else { b.clone() };
            bookmark_content.push_str(&format!("{}{}\n", indicator, style_str));
        }
        if app.bookmarks.is_empty() {
            bookmark_content.push_str("No bookmarks saved. Press [G] on any web page to bookmark it!");
        }
        bookmark_content.push_str("\n\nPress [Esc] or [O] to exit Bookmarks panel.");

        let bookmarks_p = Paragraph::new(bookmark_content)
            .style(Style::default().fg(app.theme.primary))
            .wrap(Wrap { trim: false });
        f.render_widget(bookmarks_p, content_area);
        return;
    }

    if app.show_history {
        let mut history_content = "iSearch CLI™ Persistent History:\n\n\
                                   Scroll with [Up/Down] and Press [Enter] to go to page:\n\n".to_string();
        for (idx, h) in app.history_list.iter().enumerate() {
            let indicator = if idx == app.scroll_offset { "➔  " } else { "   " };
            let style_str = if idx == app.scroll_offset { format!("[ {} ]", h) } else { h.clone() };
            history_content.push_str(&format!("{}{}\n", indicator, style_str));
        }
        if app.history_list.is_empty() {
            history_content.push_str("No history recorded yet.");
        }
        history_content.push_str("\n\nPress [Esc] or [Y] to exit History panel.");

        let history_p = Paragraph::new(history_content)
            .style(Style::default().fg(app.theme.primary))
            .wrap(Wrap { trim: false });
        f.render_widget(history_p, content_area);
        return;
    }

    if app.show_help {
        let help_text = "\
iSearch CLI™ Terminal Browser Help\n\n\
Keybindings:\n\
  [Esc / Q]      Exit the browser / close download popups\n\
  [L]            Focus address bar (type URL or search term)\n\
  [R]            Reload current page\n\
  [E]            Toggle between NATIVE and CHROMIUM engines\n\
  [T]            Open a new tab\n\
  [Tab]          Switch to next tab\n\
  [W]            Close active tab\n\
  [G]            Bookmark current page\n\
  [O]            Toggle Bookmarks panel\n\
  [Y]            Toggle History panel\n\
  [P]            Download current page/file\n\
  [K]            Cycle theme preset\n\
  [H]            Toggle this help screen\n\
  [Up / Down]    Scroll viewport up/down\n\n\
Under the hood:\n\
  - Native Engine uses highly optimized Rust parsers for super fast, low-memory offline browsing.\n\
  - Chromium Engine runs Chrome/Chromium headlessly for complex Javascript-heavy modern apps (React, Vue, Next.js).";

        let help_p = Paragraph::new(help_text)
            .style(Style::default().fg(app.theme.text))
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
                    Span::styled("Archive Contents (ZIP) - Scroll and Press [Enter] to open file:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                ]));
                display_lines.push(Line::raw(""));
                for (idx, file) in files.iter().enumerate() {
                    let indicator = if idx == app.scroll_offset { "➔ 📦 " } else { "  📦 " };
                    let style = if idx == app.scroll_offset {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    display_lines.push(Line::from(vec![
                        Span::raw(indicator),
                        Span::styled(file.clone(), style),
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
            PageContent::Mesh3DPreview { mesh, .. } => {
                display_lines = mesh.render_to_lines(content_area.width as usize, content_area.height as usize);
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

    if !app.downloads.is_empty() {
        display_lines.push(Line::raw(""));
        display_lines.push(Line::from(vec![
            Span::styled("--- Active Downloads ---", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))
        ]));
        for d in &app.downloads {
            let filled = (d.1 * 10.0) as usize;
            let bar = format!("  [ {}{} ]  {}", "█".repeat(filled), "░".repeat(10 - filled), d.2);
            display_lines.push(Line::from(vec![
                Span::styled(format!("📁 File: {}  ", d.0), Style::default().fg(app.theme.text)),
                Span::styled(bar, Style::default().fg(app.theme.success)),
            ]));
        }
    }

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
        Span::styled(" [K: Cycle Theme] ", Style::default().fg(Color::Gray)),
        Span::styled(" [H: Toggle Help] ", Style::default().fg(Color::Gray)),
        Span::raw(" │ Status: "),
        Span::styled(app.status_message.clone(), Style::default().fg(app.theme.highlight)),
    ]))
    .style(Style::default().bg(Color::Black).fg(app.theme.text));
    f.render_widget(status_bar, chunks[3]);
}
