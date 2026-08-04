use crate::browser::core::{BrowserCore, BrowserError, EngineType, PageContent};
use crate::browser::favorites::{FavoriteItem, FavoritesManager};
use crate::browser::history::{group_by_date, group_by_domain, HistoryItem, HistoryManager};
use crate::browser::native::{render_html_to_lines, render_markdown_to_lines, CssStyle};
use crate::browser::terminal_media::TerminalCapabilities;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
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
    pub downloads: Vec<(String, f32, String)>, // (filename, progress, status)
    pub download_active: bool,

    // New additions
    pub favorites_mgr: FavoritesManager,
    pub history_mgr: HistoryManager,
    pub private_mode: bool,

    // UI Panels
    pub show_favorites: bool,
    pub show_history_mgr: bool,

    // Favorites state
    pub fav_search_buffer: String,
    pub fav_input_mode: bool,
    pub fav_input_field: usize, // 0 = Title, 1 = URL, 2 = Folder, 3 = Search, 4 = ImportPath, 5 = ExportPath
    pub fav_title_input: String,
    pub fav_url_input: String,
    pub fav_folder_input: String,
    pub fav_path_input: String,
    pub fav_selected_idx: usize,
    pub fav_folder_filter: String, // "All" or folder name

    // History state
    pub hist_search_buffer: String,
    pub hist_domain_filter: String,
    pub hist_sort_by: String,  // "date" or "visits"
    pub hist_group_by: String, // "none", "date", or "domain"
    pub hist_selected_idx: usize,
    pub hist_input_mode: bool,
    pub hist_input_field: usize, // 0 = Search, 1 = Domain Filter, 2 = ImportPath, 3 = ExportPath

    // Temporary list for downloads in private mode
    pub temp_downloads: Vec<(String, f32, String)>,
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
            theme: crate::browser::theme::AppTheme::from_preset(
                crate::browser::theme::ThemePreset::Default,
            ),
            theme_preset: crate::browser::theme::ThemePreset::Default,
            downloads: Vec::new(),
            download_active: false,

            favorites_mgr: FavoritesManager::new("favorites.json"),
            history_mgr: HistoryManager::new("history.db"),
            private_mode: false,
            show_favorites: false,
            show_history_mgr: false,

            fav_search_buffer: String::new(),
            fav_input_mode: false,
            fav_input_field: 0,
            fav_title_input: String::new(),
            fav_url_input: String::new(),
            fav_folder_input: String::new(),
            fav_path_input: String::new(),
            fav_selected_idx: 0,
            fav_folder_filter: "All".to_string(),

            hist_search_buffer: String::new(),
            hist_domain_filter: String::new(),
            hist_sort_by: "date".to_string(),
            hist_group_by: "none".to_string(),
            hist_selected_idx: 0,
            hist_input_mode: false,
            hist_input_field: 0,

            temp_downloads: Vec::new(),
        };
        let _ = app.load_current_page();
        app
    }

    pub fn load_current_page(&mut self) -> Result<(), BrowserError> {
        self.scroll_offset = 0;
        let url = self.tabs[self.active_tab_idx].url.clone();

        #[cfg(target_os = "android")]
        {
            self.core.current_engine = EngineType::Native;
        }

        if self.core.current_engine == EngineType::Chromium
            && !self.core.chromium_engine.is_available()
        {
            #[cfg(target_os = "android")]
            {
                self.core.current_engine = EngineType::Native;
            }
            #[cfg(not(target_os = "android"))]
            {
                self.show_installer = true;
                self.installer_step = InstallerStep::Main;
                self.status_message = "Chromium required but not available.".to_string();
                return Err(BrowserError::ChromiumNotAvailable(
                    "Chromium not found".to_string(),
                ));
            }
        }

        self.status_message = format!("Loading {}...", url);

        // Apply incognito flag to chromium engine dynamically
        self.core.chromium_engine.incognito_mode = self.private_mode;

        match self.core.navigate(&url) {
            Ok(content) => {
                let title = match &content {
                    PageContent::Html { title, .. } => title.clone(),
                    PageContent::Markdown { title, .. } => title.clone(),
                    PageContent::Directory { path, .. } => path.to_string_lossy().to_string(),
                    PageContent::FilePreview { path, .. } => path.to_string_lossy().to_string(),
                    PageContent::PdfPreview { title, .. } => title.clone(),
                    PageContent::ArchivePreview { path, .. } => {
                        format!("Archive: {}", path.to_string_lossy())
                    }
                    PageContent::ImagePreview { path, .. } => {
                        format!("Image: {}", path.to_string_lossy())
                    }
                    PageContent::AnsiText { title, .. } => title.clone(),
                    PageContent::Mesh3DPreview { title, .. } => format!("3D Model: {}", title),
                };

                self.tabs[self.active_tab_idx].title = title.clone();
                self.tabs[self.active_tab_idx].content = Some(content);
                self.status_message = "Loaded.".to_string();

                // Save to history if NOT in private mode
                if !self.private_mode {
                    let _ = self.history_mgr.add_visit(&title, &url);
                }
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                Err(e)
            }
        }
    }

    pub fn get_favorites_filtered(&self) -> Vec<FavoriteItem> {
        let items = if self.fav_search_buffer.is_empty() {
            self.favorites_mgr.list.items.clone()
        } else {
            self.favorites_mgr.search(&self.fav_search_buffer)
        };

        if self.fav_folder_filter == "All" {
            items
        } else {
            items
                .into_iter()
                .filter(|item| item.folder == self.fav_folder_filter)
                .collect()
        }
    }

    pub fn get_history_filtered(&self) -> Vec<HistoryItem> {
        self.history_mgr
            .get_all(
                &self.hist_search_buffer,
                &self.hist_domain_filter,
                &self.hist_sort_by,
            )
            .unwrap_or_default()
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
        // Handle download progress
        if app.download_active {
            let active_list = if app.private_mode {
                &mut app.temp_downloads
            } else {
                &mut app.downloads
            };
            for d in active_list.iter_mut() {
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
            if active_list.iter().all(|d| d.1 >= 1.0) {
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
                            InstallerStep::Main => match key.code {
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
                                KeyCode::Char('4')
                                | KeyCode::Esc
                                | KeyCode::Char('q')
                                | KeyCode::Char('Q') => {
                                    app.show_installer = false;
                                    app.core.set_engine(EngineType::Native);
                                    app.status_message = "Using Native Rendering Mode".to_string();
                                    let _ = app.load_current_page();
                                }
                                _ => {}
                            },
                            InstallerStep::DownloadPage | InstallerStep::InstallCommands => {
                                match key.code {
                                    KeyCode::Esc
                                    | KeyCode::Backspace
                                    | KeyCode::Char('b')
                                    | KeyCode::Char('B') => {
                                        app.installer_step = InstallerStep::Main;
                                    }
                                    _ => {}
                                }
                            }
                            InstallerStep::ConfigurePath => match key.code {
                                KeyCode::Esc => {
                                    app.installer_step = InstallerStep::Main;
                                }
                                KeyCode::Enter => {
                                    let path =
                                        std::path::PathBuf::from(app.custom_path_input.trim());
                                    if path.exists() {
                                        app.core.chromium_engine.set_manual_path(path);
                                        app.show_installer = false;
                                        app.installer_status =
                                            "Custom path configured!".to_string();
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
                            },
                        }
                    } else if app.show_favorites {
                        if app.fav_input_mode {
                            match key.code {
                                KeyCode::Esc => {
                                    app.fav_input_mode = false;
                                }
                                KeyCode::Tab => {
                                    // Cycle input fields
                                    if app.fav_input_field == 4 || app.fav_input_field == 5 {
                                        // Import or Export
                                        app.fav_input_mode = false;
                                    } else {
                                        app.fav_input_field = (app.fav_input_field + 1) % 4;
                                    }
                                }
                                KeyCode::Backspace => match app.fav_input_field {
                                    0 => {
                                        app.fav_title_input.pop();
                                    }
                                    1 => {
                                        app.fav_url_input.pop();
                                    }
                                    2 => {
                                        app.fav_folder_input.pop();
                                    }
                                    3 => {
                                        app.fav_search_buffer.pop();
                                    }
                                    4 | 5 => {
                                        app.fav_path_input.pop();
                                    }
                                    _ => {}
                                },
                                KeyCode::Char(c) => match app.fav_input_field {
                                    0 => {
                                        app.fav_title_input.push(c);
                                    }
                                    1 => {
                                        app.fav_url_input.push(c);
                                    }
                                    2 => {
                                        app.fav_folder_input.push(c);
                                    }
                                    3 => {
                                        app.fav_search_buffer.push(c);
                                    }
                                    4 | 5 => {
                                        app.fav_path_input.push(c);
                                    }
                                    _ => {}
                                },
                                KeyCode::Enter => {
                                    match app.fav_input_field {
                                        0..=2 => {
                                            if !app.fav_url_input.trim().is_empty() {
                                                let title = if app.fav_title_input.trim().is_empty()
                                                {
                                                    "Untitled"
                                                } else {
                                                    &app.fav_title_input
                                                };
                                                let _ = app.favorites_mgr.add(
                                                    title,
                                                    &app.fav_url_input,
                                                    &app.fav_folder_input,
                                                );
                                                app.status_message =
                                                    "Favorite added successfully!".to_string();
                                            }
                                            app.fav_input_mode = false;
                                        }
                                        3 => {
                                            app.fav_input_mode = false;
                                        }
                                        4 => {
                                            // Import
                                            match app
                                                .favorites_mgr
                                                .import_from_file(app.fav_path_input.trim())
                                            {
                                                Ok(_) => {
                                                    app.status_message =
                                                        "Favorites imported!".to_string();
                                                }
                                                Err(e) => {
                                                    app.status_message =
                                                        format!("Import failed: {}", e);
                                                }
                                            }
                                            app.fav_input_mode = false;
                                        }
                                        5 => {
                                            // Export
                                            match app
                                                .favorites_mgr
                                                .export_to_file(app.fav_path_input.trim())
                                            {
                                                Ok(_) => {
                                                    app.status_message =
                                                        "Favorites exported!".to_string();
                                                }
                                                Err(e) => {
                                                    app.status_message =
                                                        format!("Export failed: {}", e);
                                                }
                                            }
                                            app.fav_input_mode = false;
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('o') | KeyCode::Char('O') => {
                                    app.show_favorites = false;
                                }
                                KeyCode::Up => {
                                    if app.fav_selected_idx > 0 {
                                        app.fav_selected_idx -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    let items = app.get_favorites_filtered();
                                    if app.fav_selected_idx < items.len().saturating_sub(1) {
                                        app.fav_selected_idx += 1;
                                    }
                                }
                                KeyCode::Char('a') | KeyCode::Char('A') => {
                                    app.fav_input_mode = true;
                                    app.fav_input_field = 0;
                                    app.fav_title_input = String::new();
                                    app.fav_url_input = app.tabs[app.active_tab_idx].url.clone();
                                    app.fav_folder_input = "General".to_string();
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                                    let items = app.get_favorites_filtered();
                                    if !items.is_empty() && app.fav_selected_idx < items.len() {
                                        let selected_url = items[app.fav_selected_idx].url.clone();
                                        let _ = app.favorites_mgr.remove(&selected_url);
                                        app.status_message = "Favorite removed.".to_string();
                                        app.fav_selected_idx =
                                            app.fav_selected_idx.saturating_sub(1);
                                    }
                                }
                                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                                    app.fav_input_mode = true;
                                    app.fav_input_field = 3;
                                    app.fav_search_buffer = String::new();
                                }
                                KeyCode::Char('f') | KeyCode::Char('F') => {
                                    // Cycle folder filter
                                    let folders = app.favorites_mgr.folders();
                                    if let Some(pos) =
                                        folders.iter().position(|f| f == &app.fav_folder_filter)
                                    {
                                        let next_pos = (pos + 1) % folders.len();
                                        app.fav_folder_filter = folders[next_pos].clone();
                                    } else {
                                        app.fav_folder_filter = "All".to_string();
                                    }
                                    app.fav_selected_idx = 0;
                                }
                                KeyCode::Char('i') | KeyCode::Char('I') => {
                                    app.fav_input_mode = true;
                                    app.fav_input_field = 4;
                                    app.fav_path_input = "favorites_import.json".to_string();
                                }
                                KeyCode::Char('x')
                                | KeyCode::Char('X')
                                | KeyCode::Char('e')
                                | KeyCode::Char('E') => {
                                    app.fav_input_mode = true;
                                    app.fav_input_field = 5;
                                    app.fav_path_input = "favorites_export.json".to_string();
                                }
                                KeyCode::Enter => {
                                    let items = app.get_favorites_filtered();
                                    if !items.is_empty() && app.fav_selected_idx < items.len() {
                                        let selected_url = items[app.fav_selected_idx].url.clone();
                                        app.tabs[app.active_tab_idx].url = selected_url;
                                        app.show_favorites = false;
                                        let _ = app.load_current_page();
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if app.show_history_mgr {
                        if app.hist_input_mode {
                            match key.code {
                                KeyCode::Esc => {
                                    app.hist_input_mode = false;
                                }
                                KeyCode::Tab => {
                                    app.hist_input_field = (app.hist_input_field + 1) % 2;
                                }
                                KeyCode::Backspace => match app.hist_input_field {
                                    0 => {
                                        app.hist_search_buffer.pop();
                                    }
                                    1 => {
                                        app.hist_domain_filter.pop();
                                    }
                                    _ => {}
                                },
                                KeyCode::Char(c) => match app.hist_input_field {
                                    0 => {
                                        app.hist_search_buffer.push(c);
                                    }
                                    1 => {
                                        app.hist_domain_filter.push(c);
                                    }
                                    _ => {}
                                },
                                KeyCode::Enter => {
                                    app.hist_input_mode = false;
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    app.show_history_mgr = false;
                                }
                                KeyCode::Up => {
                                    if app.hist_selected_idx > 0 {
                                        app.hist_selected_idx -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    let items = app.get_history_filtered();
                                    if app.hist_selected_idx < items.len().saturating_sub(1) {
                                        app.hist_selected_idx += 1;
                                    }
                                }
                                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                                    app.hist_input_mode = true;
                                    app.hist_input_field = 0;
                                    app.hist_search_buffer = String::new();
                                }
                                KeyCode::Char('f') | KeyCode::Char('F') => {
                                    app.hist_input_mode = true;
                                    app.hist_input_field = 1;
                                    app.hist_domain_filter = String::new();
                                }
                                KeyCode::Char('r') | KeyCode::Char('R') => {
                                    // Toggle sort by
                                    if app.hist_sort_by == "date" {
                                        app.hist_sort_by = "visits".to_string();
                                    } else {
                                        app.hist_sort_by = "date".to_string();
                                    }
                                    app.hist_selected_idx = 0;
                                }
                                KeyCode::Char('g') | KeyCode::Char('G') => {
                                    // Toggle group by
                                    match app.hist_group_by.as_str() {
                                        "none" => app.hist_group_by = "date".to_string(),
                                        "date" => app.hist_group_by = "domain".to_string(),
                                        _ => app.hist_group_by = "none".to_string(),
                                    }
                                    app.hist_selected_idx = 0;
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                                    let items = app.get_history_filtered();
                                    if !items.is_empty() && app.hist_selected_idx < items.len() {
                                        if let Some(id) = items[app.hist_selected_idx].id {
                                            let _ = app.history_mgr.delete_selected(id);
                                            app.status_message =
                                                "History item deleted.".to_string();
                                            app.hist_selected_idx =
                                                app.hist_selected_idx.saturating_sub(1);
                                        }
                                    }
                                }
                                KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Backspace => {
                                    // Delete all
                                    let _ = app.history_mgr.delete_all();
                                    app.status_message = "All history cleared!".to_string();
                                    app.hist_selected_idx = 0;
                                }
                                KeyCode::Char('i') | KeyCode::Char('I') => {
                                    match app.history_mgr.import_from_file("history_import.json") {
                                        Ok(_) => {
                                            app.status_message = "History imported!".to_string();
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Import failed: {}", e);
                                        }
                                    }
                                }
                                KeyCode::Char('x')
                                | KeyCode::Char('X')
                                | KeyCode::Char('e')
                                | KeyCode::Char('E') => {
                                    match app.history_mgr.export_to_file("history_export.json") {
                                        Ok(_) => {
                                            app.status_message =
                                                "History exported to history_export.json!"
                                                    .to_string();
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Export failed: {}", e);
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    let items = app.get_history_filtered();
                                    if !items.is_empty() && app.hist_selected_idx < items.len() {
                                        let selected_url = items[app.hist_selected_idx].url.clone();
                                        app.tabs[app.active_tab_idx].url = selected_url;
                                        app.show_history_mgr = false;
                                        let _ = app.load_current_page();
                                    }
                                }
                                _ => {}
                            }
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
                                let next = match app.core.current_engine {
                                    EngineType::Native => EngineType::Chromium,
                                    EngineType::Chromium => EngineType::Native,
                                };
                                app.core.set_engine(next);
                                app.status_message = format!("Switched engine to {:?}", next);
                                let _ = app.load_current_page();
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                app.tabs.push(BrowserTab::new("https://www.google.com"));
                                app.active_tab_idx = app.tabs.len() - 1;
                                app.address_buffer = "https://www.google.com".to_string();
                                let _ = app.load_current_page();
                            }
                            KeyCode::Tab => {
                                app.active_tab_idx = (app.active_tab_idx + 1) % app.tabs.len();
                                app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                self_render_viewport(app);
                            }
                            KeyCode::Char('w') | KeyCode::Char('W') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_x(0.1);
                                } else {
                                    if app.tabs.len() > 1 {
                                        app.tabs.remove(app.active_tab_idx);
                                        if app.active_tab_idx >= app.tabs.len() {
                                            app.active_tab_idx = app.tabs.len() - 1;
                                        }
                                        app.address_buffer =
                                            app.tabs[app.active_tab_idx].url.clone();
                                        self_render_viewport(app);
                                    }
                                }
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_x(-0.1);
                                }
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_y(-0.1);
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_y(0.1);
                                }
                            }
                            KeyCode::Char('b') | KeyCode::Char('B') => {
                                let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                if current_idx > 0 {
                                    app.tabs[app.active_tab_idx].history_idx -= 1;
                                    let prev_url = app.tabs[app.active_tab_idx].history
                                        [current_idx - 1]
                                        .clone();
                                    app.tabs[app.active_tab_idx].url = prev_url;
                                    let _ = app.load_current_page();
                                } else {
                                    app.status_message = "No back history".to_string();
                                }
                            }
                            KeyCode::Char('f') | KeyCode::Char('F') => {
                                let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                let hist_len = app.tabs[app.active_tab_idx].history.len();
                                if current_idx + 1 < hist_len {
                                    app.tabs[app.active_tab_idx].history_idx += 1;
                                    let next_url = app.tabs[app.active_tab_idx].history
                                        [current_idx + 1]
                                        .clone();
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
                                app.show_favorites = !app.show_favorites;
                                app.show_history_mgr = false;
                                app.scroll_offset = 0;
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.show_history_mgr = !app.show_history_mgr;
                                app.show_favorites = false;
                                app.scroll_offset = 0;
                            }
                            KeyCode::Char('g') | KeyCode::Char('G') => {
                                let current_url = app.tabs[app.active_tab_idx].url.clone();
                                let current_title = app.tabs[app.active_tab_idx].title.clone();
                                let _ =
                                    app.favorites_mgr
                                        .add(&current_title, &current_url, "General");
                                app.status_message = "Added current page to favorites!".to_string();
                            }
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                let filename = app.tabs[app.active_tab_idx]
                                    .url
                                    .split('/')
                                    .next_back()
                                    .unwrap_or("index.html")
                                    .to_string();
                                let clean_filename =
                                    if filename.is_empty() || filename.contains('?') {
                                        "index.html".to_string()
                                    } else {
                                        filename
                                    };
                                if app.private_mode {
                                    app.temp_downloads.push((
                                        clean_filename.clone(),
                                        0.0,
                                        "Initiating private download...".to_string(),
                                    ));
                                } else {
                                    app.downloads.push((
                                        clean_filename.clone(),
                                        0.0,
                                        "Initiating download...".to_string(),
                                    ));
                                }
                                app.download_active = true;
                                app.status_message = format!("Downloading {}...", clean_filename);
                            }
                            KeyCode::Char('k') | KeyCode::Char('K') => {
                                let next_preset = match app.theme_preset {
                                    crate::browser::theme::ThemePreset::Default => {
                                        crate::browser::theme::ThemePreset::Dracula
                                    }
                                    crate::browser::theme::ThemePreset::Dracula => {
                                        crate::browser::theme::ThemePreset::Nord
                                    }
                                    crate::browser::theme::ThemePreset::Nord => {
                                        crate::browser::theme::ThemePreset::Ocean
                                    }
                                    crate::browser::theme::ThemePreset::Ocean => {
                                        crate::browser::theme::ThemePreset::Monokai
                                    }
                                    crate::browser::theme::ThemePreset::Monokai => {
                                        crate::browser::theme::ThemePreset::Light
                                    }
                                    crate::browser::theme::ThemePreset::Light => {
                                        crate::browser::theme::ThemePreset::Default
                                    }
                                };
                                app.theme_preset = next_preset;
                                app.theme =
                                    crate::browser::theme::AppTheme::from_preset(next_preset);
                                app.status_message =
                                    format!("Switched theme to {}", app.theme.name);
                            }
                            KeyCode::Char('v') | KeyCode::Char('V') => {
                                // Toggle Private Mode (Anonymous mode)
                                app.private_mode = !app.private_mode;
                                if app.private_mode {
                                    app.status_message =
                                        "ANONYMOUS BROWSING MODE ACTIVE".to_string();
                                } else {
                                    app.temp_downloads.clear();
                                    app.status_message =
                                        "Returned to standard browsing mode".to_string();
                                }
                                let _ = app.load_current_page();
                            }
                            KeyCode::Up => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_x(0.1);
                                } else if app.scroll_offset > 0 {
                                    app.scroll_offset -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_x(-0.1);
                                } else {
                                    app.scroll_offset += 1;
                                }
                            }
                            KeyCode::Left => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_y(-0.1);
                                } else {
                                    let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                    if current_idx > 0 {
                                        app.tabs[app.active_tab_idx].history_idx -= 1;
                                        let prev_url = app.tabs[app.active_tab_idx].history
                                            [current_idx - 1]
                                            .clone();
                                        app.tabs[app.active_tab_idx].url = prev_url;
                                        let _ = app.load_current_page();
                                    } else {
                                        app.status_message = "No back history".to_string();
                                    }
                                }
                            }
                            KeyCode::Right => {
                                if let Some(PageContent::Mesh3DPreview { mesh, .. }) =
                                    &mut app.tabs[app.active_tab_idx].content
                                {
                                    mesh.rotate_y(0.1);
                                } else {
                                    let current_idx = app.tabs[app.active_tab_idx].history_idx;
                                    let hist_len = app.tabs[app.active_tab_idx].history.len();
                                    if current_idx + 1 < hist_len {
                                        app.tabs[app.active_tab_idx].history_idx += 1;
                                        let next_url = app.tabs[app.active_tab_idx].history
                                            [current_idx + 1]
                                            .clone();
                                        app.tabs[app.active_tab_idx].url = next_url;
                                        let _ = app.load_current_page();
                                    } else {
                                        app.status_message = "No forward history".to_string();
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(PageContent::ArchivePreview { path, files }) =
                                    &app.tabs[app.active_tab_idx].content
                                {
                                    if app.scroll_offset < files.len() {
                                        let selected_file = &files[app.scroll_offset];
                                        let archive_url = format!(
                                            "{}::{}",
                                            path.to_string_lossy(),
                                            selected_file
                                        );
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
                Event::Mouse(mouse_event)
                    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) =>
                {
                    let r = mouse_event.row;
                    let c = mouse_event.column;

                    if r == 1 {
                        if (20..=80).contains(&c) {
                            let click_idx = ((c - 20) / 18) as usize;
                            if click_idx < app.tabs.len() {
                                app.active_tab_idx = click_idx;
                                app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
                                self_render_viewport(app);
                            }
                        }
                    } else if (3..=5).contains(&r) {
                        app.input_mode = true;
                        app.address_buffer = app.tabs[app.active_tab_idx].url.clone();
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

    // Theme and style settings
    let primary_style = if app.private_mode {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD)
    };

    let highlight_style = Style::default()
        .fg(app.theme.highlight)
        .add_modifier(Modifier::BOLD);
    let border_style = if app.private_mode {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(app.theme.border)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Bar (Tabs)
            Constraint::Length(3), // Address Bar
            Constraint::Min(1),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(size);

    // 1. Top bar
    let mut tabs_spans = Vec::new();
    for (i, tab) in app.tabs.iter().enumerate() {
        let style = if i == app.active_tab_idx {
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        tabs_spans.push(Span::styled(
            format!(" [ Tab {}: {} ] ", i + 1, tab.title),
            style,
        ));
    }
    let engine_mode = format!(" Engine: {:?} ", app.core.current_engine);
    let mut top_spans = vec![
        Span::styled(
            if app.private_mode {
                " iSearch CLI™ [PRIVATE] "
            } else {
                " iSearch Browser™ "
            },
            primary_style,
        ),
        Span::raw(" │ "),
    ];
    top_spans.extend(tabs_spans);
    top_spans.push(Span::raw(" │ "));
    top_spans.push(Span::styled(
        engine_mode,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));

    let top_bar = Paragraph::new(Line::from(top_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style),
    );
    f.render_widget(top_bar, chunks[0]);

    // 2. Address bar
    let addr_border_style = if app.input_mode {
        highlight_style
    } else {
        border_style
    };
    let addr_text = if app.input_mode {
        format!("{}█", app.address_buffer)
    } else {
        app.tabs[app.active_tab_idx].url.clone()
    };
    let addr_bar = Paragraph::new(Line::from(vec![
        Span::styled(" URL: ", Style::default().fg(Color::Gray)),
        Span::raw(addr_text),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(addr_border_style),
    );
    f.render_widget(addr_bar, chunks[1]);

    // 3. Main content
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let content_area = content_block.inner(chunks[2]);
    f.render_widget(content_block, chunks[2]);

    if app.show_installer {
        let installer_content = match app.installer_step {
            InstallerStep::Main => "\
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
[4] Continue using Native Rendering Mode"
                .to_string(),
            InstallerStep::DownloadPage => "\
Official Browser Download Pages:\n\n\
- Google Chrome: https://www.google.com/chrome/\n\
- Chromium: https://www.chromium.org/getting-involved/download-chromium/\n\
- Microsoft Edge: https://www.microsoft.com/edge/\n\
- Brave: https://brave.com/\n\
- Vivaldi: https://vivaldi.com/\n\
- Opera: https://www.opera.com/\n\n\
Press [Backspace] or [B] to go back."
                .to_string(),
            InstallerStep::InstallCommands => {
                app.core.chromium_engine.get_guided_install_instructions()
                    + "\n\nPress [Backspace] or [B] to go back."
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

    if app.show_favorites {
        let mut bookmark_content = "\
░█▀▀░█▀█░█░█░█▀█░█▀▄░▀█▀░▀█▀░█▀▀░█▀▀
░█▀▀░█▀█░▀▄▀░█░█░█▀▄░░█░░░█░░█▀▀░▀▀█
░▀░░░▀░▀░░▀░░▀▀▀░▀░▀░▀▀▀░░▀░░▀▀▀░▀▀▀
                 iSearch CLI™ FAVORITES PANEL
\n"
        .to_string();

        bookmark_content.push_str(&format!(
            "  [ Filter Folder: {} ]  [ Search: {} ]\n\n",
            app.fav_folder_filter, app.fav_search_buffer
        ));

        if app.fav_input_mode {
            bookmark_content.push_str("  --- ENTER NEW FAVORITE DETAILS ---\n");
            let fields = [
                format!("  Title : {}", app.fav_title_input),
                format!("  URL   : {}", app.fav_url_input),
                format!("  Folder: {}", app.fav_folder_input),
                format!("  Search: {}", app.fav_search_buffer),
                format!("  Import File Path: {}", app.fav_path_input),
                format!("  Export File Path: {}", app.fav_path_input),
            ];
            for (idx, field) in fields.iter().enumerate() {
                if idx == app.fav_input_field {
                    bookmark_content.push_str(&format!("➔ {} █\n", field));
                } else {
                    bookmark_content.push_str(&format!("  {}\n", field));
                }
            }
            bookmark_content
                .push_str("\n  Press [Tab] to cycle, [Enter] to save/confirm, [Esc] to cancel.\n");
        } else {
            let items = app.get_favorites_filtered();
            for (idx, item) in items.iter().enumerate() {
                let indicator = if idx == app.fav_selected_idx {
                    "➔  "
                } else {
                    "   "
                };
                let line_str = format!(
                    "{} [{}] {} - {}",
                    indicator, item.folder, item.title, item.url
                );
                bookmark_content.push_str(&format!("{}\n", line_str));
            }
            if items.is_empty() {
                bookmark_content.push_str("  No favorites found.\n");
            }
            bookmark_content.push_str("\n  Keyboard Shortcuts:\n");
            bookmark_content.push_str("    [A] Add Favorite  [D / Del] Delete selected  [/ / S] Search  [F] Toggle Folder Filter\n");
            bookmark_content.push_str("    [I] Import JSON  [E / X] Export JSON  [Esc / O] Close panel  [Enter] Go to Favorite\n");
        }

        let bookmarks_p = Paragraph::new(bookmark_content)
            .style(Style::default().fg(app.theme.primary))
            .wrap(Wrap { trim: false });
        f.render_widget(bookmarks_p, content_area);
        return;
    }

    if app.show_history_mgr {
        let mut history_content = "\
░█░█░▀█▀░█▀▀░▀█▀░█▀█░█▀▄░█░█
░█▀█░░█░░▀▀█░░█░░█░█░█▀▄░░█░
░▀░▀░▀▀▀░▀▀▀░░▀░░▀▀▀░▀░▀░░▀░
                iSearch CLI™ HISTORY MANAGER
\n"
        .to_string();

        history_content.push_str(&format!(
            "  [ Search: {} ]  [ Filter Domain: {} ]  [ Sort: {} ]  [ Group: {} ]\n\n",
            app.hist_search_buffer, app.hist_domain_filter, app.hist_sort_by, app.hist_group_by
        ));

        if app.hist_input_mode {
            history_content.push_str("  --- ENTER FILTER DETAILS ---\n");
            let fields = [
                format!("  Search : {}", app.hist_search_buffer),
                format!("  Domain : {}", app.hist_domain_filter),
            ];
            for (idx, field) in fields.iter().enumerate() {
                if idx == app.hist_input_field {
                    history_content.push_str(&format!("➔ {} █\n", field));
                } else {
                    history_content.push_str(&format!("  {}\n", field));
                }
            }
            history_content
                .push_str("\n  Press [Tab] to cycle, [Enter] to submit, [Esc] to cancel.\n");
        } else {
            let items = app.get_history_filtered();
            if items.is_empty() {
                history_content.push_str("  No history recorded yet.\n");
            } else {
                match app.hist_group_by.as_str() {
                    "date" => {
                        let groups = group_by_date(&items);
                        for (date, grp_items) in groups {
                            history_content.push_str(&format!("  📅 Date: {}\n", date));
                            for (orig_idx, item) in grp_items {
                                let indicator = if orig_idx == app.hist_selected_idx {
                                    "➔ "
                                } else {
                                    "  "
                                };
                                history_content.push_str(&format!(
                                    "    {} - [{}] {} ({})\n",
                                    indicator, item.visited_at, item.title, item.url
                                ));
                            }
                        }
                    }
                    "domain" => {
                        let groups = group_by_domain(&items);
                        for (domain, grp_items) in groups {
                            history_content.push_str(&format!("  🌐 Domain: {}\n", domain));
                            for (orig_idx, item) in grp_items {
                                let indicator = if orig_idx == app.hist_selected_idx {
                                    "➔ "
                                } else {
                                    "  "
                                };
                                history_content.push_str(&format!(
                                    "    {} - [{}] {} ({})\n",
                                    indicator, item.visited_at, item.title, item.url
                                ));
                            }
                        }
                    }
                    _ => {
                        // Standard list
                        for (idx, item) in items.iter().enumerate() {
                            let indicator = if idx == app.hist_selected_idx {
                                "➔  "
                            } else {
                                "   "
                            };
                            let line_str = format!(
                                "{} [{}] {} - {} (visits: {})",
                                indicator, item.visited_at, item.title, item.url, item.visit_count
                            );
                            history_content.push_str(&format!("{}\n", line_str));
                        }
                    }
                }
            }

            history_content.push_str("\n  Keyboard Shortcuts:\n");
            history_content.push_str("    [/ / S] Search  [F] Domain Filter  [R] Toggle Sort  [G] Toggle Group (Date/Domain)\n");
            history_content
                .push_str("    [D / Del] Delete selected  [C / Backspace] Clear ALL history\n");
            history_content.push_str(
                "    [I] Import history_import.json  [E / X] Export history_export.json\n",
            );
            history_content.push_str("    [Esc / Y] Close panel  [Enter] Go to History URL\n");
        }

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
  [O]            Toggle Favorites panel\n\
  [Y]            Toggle History Manager panel\n\
  [P]            Download current page/file\n\
  [K]            Cycle theme preset\n\
  [V]            Toggle Private / Anonymous Browsing Mode\n\
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

    // If private mode is toggled but we have no tabs yet, render private browsing ASCII banner!
    if app.private_mode && app.tabs[app.active_tab_idx].content.is_none() {
        display_lines.push(Line::raw(""));
        display_lines.push(Line::from(vec![Span::styled(
            "░█▀█░█▀█░█▀█░█▀█░█░█░█▄█░█▀█░█░█░█▀▀░░░█▀▄░█▀▄░█▀█░█░█░█▀▀░▀█▀░█▀█░█▀▀",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        display_lines.push(Line::from(vec![Span::styled(
            "░█▀█░█░█░█░█░█░█░░█░░█░█░█░█░█░█░▀▀█░░░█▀▄░█▀▄░█░█░█▄█░▀▀█░░█░░█░█░█░█",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        display_lines.push(Line::from(vec![Span::styled(
            "░▀░▀░▀░▀░▀▀▀░▀░▀░░▀░░▀░▀░▀▀▀░▀▀▀░▀▀▀░░░▀▀░░▀░▀░▀▀▀░▀░▀░▀▀▀░▀▀▀░▀░▀░▀▀▀",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        display_lines.push(Line::raw(""));
        display_lines.push(Line::raw(
            "  🕵️ You are now in Private / Anonymous Browsing Mode.",
        ));
        display_lines.push(Line::raw("  - No history is written or recorded."));
        display_lines.push(Line::raw(
            "  - Cookies and caches are isolated and automatically deleted when you close.",
        ));
        display_lines.push(Line::raw(
            "  - No bookmark suggestions or session states are restored.",
        ));
        display_lines.push(Line::raw(
            "  - Downloads are stored in a temporary, in-memory list only.",
        ));
        display_lines.push(Line::raw(""));
        display_lines.push(Line::raw(
            "  Press [L] to search or visit any web page or local file!",
        ));
    } else if let Some(content) = &app.tabs[app.active_tab_idx].content {
        match content {
            PageContent::Html { parsed_nodes, .. } => {
                display_lines = render_html_to_lines(
                    parsed_nodes,
                    content_area.width as usize,
                    CssStyle::default(),
                );
            }
            PageContent::Markdown { raw_md, .. } => {
                display_lines = render_markdown_to_lines(raw_md, content_area.width as usize);
            }
            PageContent::Directory { path, entries } => {
                display_lines.push(Line::from(vec![
                    Span::styled(
                        "Directory Listing: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(path.to_string_lossy().to_string()),
                ]));
                display_lines.push(Line::raw(""));
                for (name, is_dir) in entries {
                    let icon = if *is_dir { "📁 " } else { "📄 " };
                    let name_style = if *is_dir {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
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
            PageContent::PdfPreview {
                metadata,
                text_preview,
                ..
            } => {
                display_lines.push(Line::from(vec![Span::styled(
                    "PDF Document Preview",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
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
                display_lines.push(Line::from(vec![Span::styled(
                    "Archive Contents (ZIP) - Scroll and Press [Enter] to open file:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                display_lines.push(Line::raw(""));
                for (idx, file) in files.iter().enumerate() {
                    let indicator = if idx == app.scroll_offset {
                        "➔ 📦 "
                    } else {
                        "  📦 "
                    };
                    let style = if idx == app.scroll_offset {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
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
                display_lines =
                    mesh.render_to_lines(content_area.width as usize, content_area.height as usize);
            }
            _ => {
                display_lines.push(Line::raw("Preview not supported for this media type."));
            }
        }
    } else {
        display_lines.push(Line::raw(
            "No content loaded. Press [L] and enter a URL to start browsing.",
        ));
    }

    // Scroll handling
    let max_scroll = if display_lines.len() > content_area.height as usize {
        display_lines.len() - content_area.height as usize
    } else {
        0
    };
    let offset = std::cmp::min(app.scroll_offset, max_scroll);
    app.scroll_offset = offset;

    // Render downloads lists (normal and private)
    let active_downloads = if app.private_mode {
        &app.temp_downloads
    } else {
        &app.downloads
    };
    if !active_downloads.is_empty() {
        display_lines.push(Line::raw(""));
        display_lines.push(Line::from(vec![Span::styled(
            if app.private_mode {
                "--- Temporary Private Downloads ---"
            } else {
                "--- Active Downloads ---"
            },
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        )]));
        for d in active_downloads {
            let filled = (d.1 * 10.0) as usize;
            let bar = format!(
                "  [ {}{} ]  {}",
                "█".repeat(filled),
                "░".repeat(10 - filled),
                d.2
            );
            display_lines.push(Line::from(vec![
                Span::styled(
                    format!("📁 File: {}  ", d.0),
                    Style::default().fg(app.theme.text),
                ),
                Span::styled(bar, Style::default().fg(app.theme.success)),
            ]));
        }
    }

    let scrolled_lines: Vec<Line<'_>> = display_lines
        .into_iter()
        .skip(offset)
        .take(content_area.height as usize)
        .collect();
    let viewport_p = Paragraph::new(scrolled_lines);
    f.render_widget(viewport_p, content_area);

    // 4. Status Bar
    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" [Esc/Q: Exit] ", Style::default().fg(Color::Gray)),
        Span::styled(" [L: Go to] ", Style::default().fg(Color::Gray)),
        Span::styled(" [E: Switch Engine] ", Style::default().fg(Color::Gray)),
        Span::styled(" [T: New Tab] ", Style::default().fg(Color::Gray)),
        Span::styled(" [Tab: Next Tab] ", Style::default().fg(Color::Gray)),
        Span::styled(
            " [V: Toggle Private Mode] ",
            Style::default().fg(Color::Gray),
        ),
        Span::styled(" [K: Cycle Theme] ", Style::default().fg(Color::Gray)),
        Span::styled(" [H: Toggle Help] ", Style::default().fg(Color::Gray)),
        Span::raw(" │ Status: "),
        Span::styled(
            app.status_message.clone(),
            Style::default().fg(app.theme.highlight),
        ),
    ]))
    .style(Style::default().bg(Color::Black).fg(app.theme.text));
    f.render_widget(status_bar, chunks[3]);
}
