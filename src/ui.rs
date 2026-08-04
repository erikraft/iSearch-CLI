//! TUI screen implementation for the premium donation system.
//!
//! # Purpose
//! This module renders interactive terminal screens using `ratatui` where users can
//! select preset values or specify custom amounts to support the developers. It generates
//! live, scannable QR codes representing static PIX payloads.
//!
//! # Architecture & Responsibilities
//! * [DonationApp] acts as the primary state holder containing details of input fields, selection focus, and QR codes.
//! * [run_donation_tui] manages standard terminal setup (raw mode, alternative screen), main loop processing, events, and restore handlers.
//! * [ui] formats blocks, lists, inputs, custom amounts, and QR rendering layouts.
//!
//! # Interactions
//! Uses [crate::config::AppConfig] to parse keys and fallback parameters, [crate::pix::generate_pix_payload] to format raw payments,
//! and [crate::utils::generate_qr_code_for_terminal] to render output block graphics.

use crate::config::AppConfig;
use crate::pix::{generate_pix_payload, validate_amount};
use crate::utils::{copy_to_clipboard, generate_qr_code_for_terminal};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Wrap},
    Terminal, Frame,
};
use std::io;

/// Identifies which element has focus in the Select Amount screen.
#[derive(Debug, Clone, PartialEq)]
enum SelectionFocus {
    /// Focus is currently on the preset amount choices list.
    AmountList,
    /// Focus is currently on the optional text message input field.
    MessageInput,
    /// Focus is currently on the "Generate PIX QR Code" submission action box.
    GenerateButton,
}

/// Identifies the active screen currently being displayed to the user.
#[derive(Debug, Clone, PartialEq)]
enum ActiveScreen {
    /// Screen where users select from preset donation amounts or enter a custom path.
    SelectAmount,
    /// Prompt overlay to key in a custom payment decimal amount.
    EnterCustomAmount,
    /// Terminal-rendered static QR payload and address details screen.
    ShowQRCode {
        /// The validated numeric amount being donated.
        amount: f64,
        /// The optional message input.
        message: String,
        /// The raw PIX static string payload.
        payload: String,
        /// The text representing the rendered QR pattern.
        qr_code_text: String,
        /// Tracks whether copy to clipboard has been executed.
        copied: bool,
        /// Potential warnings or terminal width errors encountered during rendering.
        error_msg: Option<String>,
    },
}

/// Primary interactive application state manager for the donation TUI screen.
///
/// Keeps track of user list states, custom entries, message buffers, and focuses.
///
/// # Fields
/// * `config` - Reference to loaded configuration variables.
/// * `active_screen` - Current layout focus.
/// * `focus` - Targeted field selection focus.
/// * `default_amounts` - Cached list of values derived from configuration TOML.
/// * `amount_list_state` - List index selection state.
/// * `custom_amount` - Populated custom numeric value.
/// * `message_input` - Raw text message to embed into the PIX transaction description.
/// * `custom_amount_buffer` - Temporary keyboard entry workspace for specifying amounts.
/// * `custom_amount_error` - Holds parsing/validation messages.
pub struct DonationApp {
    config: AppConfig,
    active_screen: ActiveScreen,
    focus: SelectionFocus,

    // Amount selection
    default_amounts: Vec<u32>,
    amount_list_state: ListState,
    custom_amount: Option<f64>,

    // Message input
    message_input: String,

    // Custom amount input buffer and error
    custom_amount_buffer: String,
    custom_amount_error: Option<String>,
}

impl DonationApp {
    /// Creates a new instance of [DonationApp] based on configuration settings.
    ///
    /// # Arguments
    ///
    /// * `config` - Application config containing the required PIX key.
    ///
    /// # Returns
    ///
    /// Returns a initialized [DonationApp] structural context.
    pub fn new(config: AppConfig) -> Self {
        let default_values = config.donation.default_values.clone();
        let mut amount_list_state = ListState::default();
        amount_list_state.select(Some(0));

        Self {
            config,
            active_screen: ActiveScreen::SelectAmount,
            focus: SelectionFocus::AmountList,
            default_amounts: default_values,
            amount_list_state,
            custom_amount: None,
            message_input: String::new(),
            custom_amount_buffer: String::new(),
            custom_amount_error: None,
        }
    }

    /// Evaluates the currently selected amount based on preset choices or custom entry values.
    ///
    /// # Returns
    ///
    /// Returns the active numeric `f64` amount on success, or an error if custom input has not been populated.
    ///
    /// # Errors
    ///
    /// Returns an error if the user has highlighted the custom option but hasn't configured a numeric value yet.
    fn get_selected_amount(&self) -> Result<f64, String> {
        let idx = self.amount_list_state.selected().unwrap_or(0);
        if idx < self.default_amounts.len() {
            Ok(self.default_amounts[idx] as f64)
        } else {
            match self.custom_amount {
                Some(amt) => Ok(amt),
                None => Err("Please enter a custom amount first.".to_string()),
            }
        }
    }
}

/// Sets up the terminal backend, initializes raw mode, and launches the donation interactive screen loop.
///
/// Ensures proper restoration of terminal screens and raw mode upon exit or failure.
///
/// # Arguments
///
/// * `config` - Application [AppConfig] with configured details.
///
/// # Returns
///
/// Returns `Ok(())` upon successful termination, or a boxed dynamic error if initialization fails.
///
/// # Errors
///
/// Returns an error if raw mode, alternate screen buffers, or event polling fails.
pub fn run_donation_tui(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DonationApp::new(config);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }
    Ok(())
}

/// Core event-handling and terminal-drawing loop for the interactive donation app.
///
/// Implements keyboard navigation, text field buffer updates, and basic mouse coordinate selection.
///
/// # Arguments
///
/// * `terminal` - Mutable reference to the terminal renderer.
/// * `app` - Mutable reference to the active [DonationApp] state.
///
/// # Returns
///
/// Returns an `io::Result<()>` indicating success or failure.
///
/// # Errors
///
/// Returns an error if standard polling or keyboard reads encounter underlying I/O errors.
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut DonationApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                match &app.active_screen {
                    ActiveScreen::SelectAmount => match key.code {
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Tab => {
                            app.focus = match app.focus {
                                SelectionFocus::AmountList => SelectionFocus::MessageInput,
                                SelectionFocus::MessageInput => SelectionFocus::GenerateButton,
                                SelectionFocus::GenerateButton => SelectionFocus::AmountList,
                            };
                        }
                        KeyCode::Up => {
                            if app.focus == SelectionFocus::AmountList {
                                let current = app.amount_list_state.selected().unwrap_or(0);
                                if current > 0 {
                                    app.amount_list_state.select(Some(current - 1));
                                } else {
                                    app.amount_list_state.select(Some(app.default_amounts.len()));
                                }
                            }
                        }
                        KeyCode::Down => {
                            if app.focus == SelectionFocus::AmountList {
                                let current = app.amount_list_state.selected().unwrap_or(0);
                                if current < app.default_amounts.len() {
                                    app.amount_list_state.select(Some(current + 1));
                                } else {
                                    app.amount_list_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.focus == SelectionFocus::MessageInput {
                                app.message_input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.focus == SelectionFocus::MessageInput {
                                app.message_input.pop();
                            }
                        }
                        KeyCode::Enter => {
                            match app.focus {
                                SelectionFocus::AmountList => {
                                    let idx = app.amount_list_state.selected().unwrap_or(0);
                                    if idx == app.default_amounts.len() {
                                        // Selected Custom Amount
                                        app.active_screen = ActiveScreen::EnterCustomAmount;
                                        app.custom_amount_buffer = app.custom_amount
                                            .map(|a| format!("{:.2}", a))
                                            .unwrap_or_default();
                                        app.custom_amount_error = None;
                                    } else {
                                        app.focus = SelectionFocus::MessageInput;
                                    }
                                }
                                SelectionFocus::MessageInput => {
                                    app.focus = SelectionFocus::GenerateButton;
                                }
                                SelectionFocus::GenerateButton => {
                                    // Generate QR code and proceed
                                    match app.get_selected_amount() {
                                        Ok(amt) => {
                                            let message_opt = if app.message_input.trim().is_empty() {
                                                None
                                            } else {
                                                Some(app.message_input.as_str())
                                            };
                                            match generate_pix_payload(
                                                &app.config.donation.pix_key,
                                                Some(amt),
                                                "Erik Rodrigues Balisa",
                                                "SAO PAULO",
                                                message_opt,
                                            ) {
                                                Ok(payload) => {
                                                    let term_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
                                                    match generate_qr_code_for_terminal(&payload, term_width) {
                                                        Ok((qr_text, _)) => {
                                                            app.active_screen = ActiveScreen::ShowQRCode {
                                                                amount: amt,
                                                                message: app.message_input.clone(),
                                                                payload,
                                                                qr_code_text: qr_text,
                                                                copied: false,
                                                                error_msg: None,
                                                            };
                                                        }
                                                        Err(e) => {
                                                            app.active_screen = ActiveScreen::ShowQRCode {
                                                                amount: amt,
                                                                message: app.message_input.clone(),
                                                                payload,
                                                                qr_code_text: String::new(),
                                                                copied: false,
                                                                error_msg: Some(e),
                                                            };
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    app.custom_amount_error = Some(e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.custom_amount_error = Some(e);
                                            // Focus on Amount List if we got an error trying to get selected amount
                                            app.focus = SelectionFocus::AmountList;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    ActiveScreen::EnterCustomAmount => match key.code {
                        KeyCode::Esc => {
                            app.active_screen = ActiveScreen::SelectAmount;
                        }
                        KeyCode::Char(c) => {
                            if c.is_ascii_digit() || c == '.' || c == ',' {
                                app.custom_amount_buffer.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            app.custom_amount_buffer.pop();
                        }
                        KeyCode::Enter => {
                            match validate_amount(&app.custom_amount_buffer) {
                                Ok(amt) => {
                                    app.custom_amount = Some(amt);
                                    app.custom_amount_error = None;
                                    app.active_screen = ActiveScreen::SelectAmount;
                                    app.focus = SelectionFocus::GenerateButton;
                                }
                                Err(e) => {
                                    app.custom_amount_error = Some(e);
                                }
                            }
                        }
                        _ => {}
                    },
                    ActiveScreen::ShowQRCode { amount, message, payload, qr_code_text, copied, error_msg } => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.active_screen = ActiveScreen::SelectAmount;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Enter => {
                            let mut new_copied = *copied;
                            if copy_to_clipboard(payload).is_ok() {
                                new_copied = true;
                            }
                            app.active_screen = ActiveScreen::ShowQRCode {
                                amount: *amount,
                                message: message.clone(),
                                payload: payload.clone(),
                                qr_code_text: qr_code_text.clone(),
                                copied: new_copied,
                                error_msg: error_msg.clone(),
                            };
                        }
                        _ => {}
                    },
                }
            } else if let Event::Mouse(mouse_event) = event::read()? {
                if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                    // Simple mouse navigation support!
                    // Let's check mouse clicks to focus on specific regions
                    let y = mouse_event.row;

                    if let ActiveScreen::SelectAmount = &app.active_screen {
                        // Estimate where elements are located vertically in terminal coordinate
                        // SelectAmount screen has elements in middle area. Let's do a basic mapping:
                        if (6..=12).contains(&y) {
                            app.focus = SelectionFocus::AmountList;
                            let idx = (y - 6) as usize;
                            if idx <= app.default_amounts.len() {
                                app.amount_list_state.select(Some(idx));
                            }
                        } else if (14..=16).contains(&y) {
                            app.focus = SelectionFocus::MessageInput;
                        } else if (18..=20).contains(&y) {
                            app.focus = SelectionFocus::GenerateButton;
                        }
                    }
                }
            }
        }
    }
}

/// Renders the layout widgets to the frame based on screen context and application focus.
///
/// Implements responsive constraints, checking that the host terminal size meets minimal guidelines.
///
/// # Arguments
///
/// * `f` - Mutable reference to the `Frame` used by `ratatui`.
/// * `app` - Reference to the current [DonationApp] context.
fn ui(f: &mut Frame, app: &mut DonationApp) {
    let size = f.area();

    // Premium UI Theme colors matching both Dark and Light Backgrounds
    let primary_style = Style::default().fg(Color::Cyan);
    let highlight_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let error_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let border_style = Style::default().fg(Color::DarkGray);

    // Dynamic layout responsiveness: check minimum terminal dimensions
    if size.width < 55 || size.height < 18 {
        let warning_p = Paragraph::new("Terminal too small. Please resize terminal to at least 60x20.")
            .style(error_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(warning_p, size);
        return;
    }

    match &app.active_screen {
        ActiveScreen::SelectAmount => {
            // Main donation box layout
            let area = centered_rect(55, 18, size);

            let main_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(Span::styled(" Support iSearch CLI™ ", primary_style.add_modifier(Modifier::BOLD)));
            f.render_widget(main_block, area);

            // Inner content split
            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(2), // Subtitle
                    Constraint::Length(7), // Choices
                    Constraint::Length(3), // Message input
                    Constraint::Length(3), // Button / Error space
                ])
                .split(area);

            // 1. Thank you message
            let subtitle = Paragraph::new("Thank you for supporting the project!\nChoose an amount to support our team:")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White));
            f.render_widget(subtitle, inner_chunks[0]);

            // 2. Amount list
            let mut items = Vec::new();
            for val in &app.default_amounts {
                items.push(ListItem::new(format!("  ○ R$ {}", val)));
            }

            let custom_label = match app.custom_amount {
                Some(amt) => format!("  ○ Custom Amount (R$ {:.2})", amt),
                None => "  ○ Custom Amount".to_string(),
            };
            items.push(ListItem::new(custom_label));

            let list_widget = List::new(items)
                .block(Block::default().borders(Borders::NONE))
                .highlight_symbol("  ● ")
                .highlight_style(highlight_style);

            let mut state = app.amount_list_state.clone();
            f.render_stateful_widget(list_widget, inner_chunks[1], &mut state);

            // 3. Optional message
            let msg_border_style = if app.focus == SelectionFocus::MessageInput {
                highlight_style
            } else {
                Style::default().fg(Color::Gray)
            };

            let msg_box = Paragraph::new(app.message_input.as_str())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(msg_border_style)
                    .title(" Message (Optional) "))
                .style(Style::default().fg(Color::White));
            f.render_widget(msg_box, inner_chunks[2]);

            // 4. Generate button & validation error
            let btn_border_style = if app.focus == SelectionFocus::GenerateButton {
                highlight_style
            } else {
                Style::default().fg(Color::Gray)
            };

            let btn_text = " [ Generate PIX QR Code ] ";
            let btn_p = Paragraph::new(btn_text)
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(btn_border_style))
                .style(if app.focus == SelectionFocus::GenerateButton {
                    highlight_style
                } else {
                    Style::default().fg(Color::White)
                });
            f.render_widget(btn_p, inner_chunks[3]);
        }

        ActiveScreen::EnterCustomAmount => {
            let area = centered_rect(45, 8, size);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(primary_style)
                .title(" Custom Amount ");
            f.render_widget(block, area);

            let inner = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1), // Label
                    Constraint::Length(3), // Input box
                    Constraint::Length(1), // Error
                ])
                .split(area);

            let label = Paragraph::new("Enter amount (R$):").style(Style::default().fg(Color::White));
            f.render_widget(label, inner[0]);

            let input_box = Paragraph::new(app.custom_amount_buffer.as_str())
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(highlight_style))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(input_box, inner[1]);

            if let Some(err) = &app.custom_amount_error {
                let err_p = Paragraph::new(err.as_str()).style(error_style).alignment(Alignment::Center);
                f.render_widget(err_p, inner[2]);
            }
        }

        ActiveScreen::ShowQRCode { amount, message, payload, qr_code_text, copied, error_msg } => {
            // Adjust box size to fit the QR code and recipient details
            let area = centered_rect(64, 23, size);

            let main_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(Span::styled(" PIX Payment Terminal ", primary_style.add_modifier(Modifier::BOLD)));
            f.render_widget(main_block, area);

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Percentage(45), // Left side details
                    Constraint::Percentage(55), // Right side QR code
                ])
                .split(area);

            // 1. Left Column: Details
            let details_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Recipient
                    Constraint::Length(3), // Key
                    Constraint::Length(3), // Amount / Msg
                    Constraint::Length(4), // Copy box
                    Constraint::Length(3), // Copy button
                ])
                .split(chunks[0]);

            let recipient_p = Paragraph::new("Erik Rodrigues Balisa")
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style).title(" Recipient "));
            f.render_widget(recipient_p, details_layout[0]);

            let key_p = Paragraph::new("11925416678")
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style).title(" PIX Key (Phone) "));
            f.render_widget(key_p, details_layout[1]);

            let amount_msg_str = format!("R$ {:.2}\nMsg: {}", amount, if message.is_empty() { "None" } else { message });
            let amt_p = Paragraph::new(amount_msg_str)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(border_style).title(" Donation Details "));
            f.render_widget(amt_p, details_layout[2]);

            // Copy paste display
            let copy_label = if *copied { " Copied! " } else { " PIX Copy & Paste " };
            let copy_box_style = if *copied { highlight_style } else { border_style };
            let truncated_payload = if payload.len() > 30 {
                format!("{}...", &payload[..27])
            } else {
                payload.clone()
            };
            let copy_box_p = Paragraph::new(truncated_payload)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(copy_box_style).title(copy_label));
            f.render_widget(copy_box_p, details_layout[3]);

            // Copy action button
            let btn_p = Paragraph::new(" [C] Copy to clipboard ")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(highlight_style))
                .style(highlight_style);
            f.render_widget(btn_p, details_layout[4]);

            // 2. Right Column: QR Code
            let qr_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // QR Code
                    Constraint::Length(2), // Help line
                ])
                .split(chunks[1]);

            if let Some(err) = error_msg {
                let err_p = Paragraph::new(err.as_str())
                    .style(error_style)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(err_p, qr_layout[0]);
            } else {
                let qr_p = Paragraph::new(qr_code_text.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White));
                f.render_widget(qr_p, qr_layout[0]);
            }

            let help_p = Paragraph::new("Scan using your banking app\nPress [Esc / Q] to return")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(help_p, qr_layout[1]);
        }
    }
}

/// Helper function to center a bounding layout rectangle inside a larger frame with width/height constraint thresholds.
///
/// # Arguments
///
/// * `width_cols` - Minimum preferred character columns.
/// * `height_rows` - Minimum preferred character rows.
/// * `r` - Outer container bounds.
///
/// # Returns
///
/// Returns a centered [Rect].
fn centered_rect(width_cols: u16, height_rows: u16, r: Rect) -> Rect {
    let x = if r.width > width_cols {
        r.x + (r.width - width_cols) / 2
    } else {
        r.x
    };
    let y = if r.height > height_rows {
        r.y + (r.height - height_rows) / 2
    } else {
        r.y
    };
    Rect::new(
        x,
        y,
        std::cmp::min(width_cols, r.width),
        std::cmp::min(height_rows, r.height),
    )
}
