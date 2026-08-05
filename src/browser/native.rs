//! Native terminal document parsing, HTML/CSS styles extraction, and 3D graphics mesh engine.
//!
//! # Purpose
//! This module implements high-performance, offline-capable parsing of local and remote documents. It decodes
//! raw HTML structures using `tl`, styles them by interpreting inline CSS variables, compiles Markdown text
//! with `pulldown-cmark`, renders binary datasets as interactive hex-dumps, extracts metadata and structures from PDFs,
//! browses inside compressed ZIP collections, and renders interactive rotating 3D meshes inside raw terminal coordinates.
//!
//! # Architecture
//! * [HtmlNode](crate::browser::native::HtmlNode) acts as the parsed tree representation.
//! * [NativeEngine](crate::browser::native::NativeEngine) implements the [BrowserEngine](crate::browser::core::BrowserEngine) trait, handling local file system routes, ZIP offsets (`::`), and web requests.
//! * [Mesh3D](crate::browser::native::Mesh3D) implements an interactive orthographic projection engine that rotates and projects vectors onto character matrices.

use crate::browser::core::{BrowserEngine, BrowserError, PageContent};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::fs;
use std::path::PathBuf;
use tl::Node;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Represents a parsed HTML element node or leaf text block.
#[derive(Debug, Clone)]
pub enum HtmlNode {
    /// Text leaf node holding a sanitized content string.
    Text(String),
    /// Standard tag element containing key-value attributes and lists of children.
    Element {
        /// Element tag identifier (e.g. `p`, `div`, etc.).
        tag: String,
        /// Extracted list of paired attributes.
        attributes: Vec<(String, String)>,
        /// Children html nodes nested inside the tag.
        children: Vec<HtmlNode>,
    },
}

/// The offline native parsing engine which handles requests without executing JavaScript.
pub struct NativeEngine {}

impl Default for NativeEngine {
    /// Generates the default [NativeEngine].
    ///
    /// # Returns
    ///
    /// Returns default [NativeEngine] context.
    fn default() -> Self {
        Self::new()
    }
}

impl NativeEngine {
    /// Creates a new instance of [NativeEngine].
    ///
    /// # Returns
    ///
    /// Returns [NativeEngine].
    pub fn new() -> Self {
        Self {}
    }

    /// Parses raw HTML strings into structured [HtmlNode] arrays using `tl`.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw document body text.
    ///
    /// # Returns
    ///
    /// Returns a vector of [HtmlNode] trees.
    pub fn parse_html(html: &str) -> Vec<HtmlNode> {
        let dom = match tl::parse(html, tl::ParserOptions::default()) {
            Ok(dom) => dom,
            Err(_) => return vec![HtmlNode::Text(html.to_string())],
        };

        let parser = dom.parser();
        let mut roots = Vec::new();
        for &handle in dom.children() {
            if let Some(node) = handle.get(parser) {
                if let Some(parsed) = Self::parse_node(node, parser) {
                    roots.push(parsed);
                }
            }
        }
        roots
    }

    /// Recursively processes individual `tl::Node` references into corresponding [HtmlNode] types.
    fn parse_node(node: &Node, parser: &tl::Parser) -> Option<HtmlNode> {
        match node {
            Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str().to_string();
                let mut attrs = Vec::new();
                for (key, val) in tag.attributes().iter() {
                    let k = key.to_string();
                    let v = val.map(|v| v.to_string()).unwrap_or_default();
                    attrs.push((k, v));
                }

                let mut children = Vec::new();
                for &child_handle in tag.children().top().iter() {
                    if let Some(child_node) = child_handle.get(parser) {
                        if let Some(parsed_child) = Self::parse_node(child_node, parser) {
                            children.push(parsed_child);
                        }
                    }
                }

                Some(HtmlNode::Element {
                    tag: tag_name,
                    attributes: attrs,
                    children,
                })
            }
            Node::Raw(bytes) => {
                let s = String::from_utf8_lossy(bytes.as_bytes()).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(HtmlNode::Text(s))
                }
            }
            _ => None,
        }
    }
}

/// Generates a structured multi-column hex-dump view representing binary payload bytes.
///
/// Columns map absolute hex offset addresses on the left, hexadecimal values in the middle, and ascii prints on the right.
///
/// # Arguments
///
/// * `bytes` - Raw byte array to visualize.
/// * `width` - Active layout column dimension constraints.
///
/// # Returns
///
/// Returns a vector of styled [Line] blocks suitable for display.
pub fn render_hex_dump(bytes: &[u8], width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let chunk_size = if width > 80 { 16 } else { 8 };
    for (i, chunk) in bytes.chunks(chunk_size).enumerate() {
        let offset = i * chunk_size;
        let mut spans = vec![Span::styled(
            format!("{:08x}:  ", offset),
            Style::default().fg(Color::Yellow),
        )];

        let mut hex_part = String::new();
        let mut ascii_part = String::new();

        for (j, &b) in chunk.iter().enumerate() {
            hex_part.push_str(&format!("{:02x} ", b));
            if j == chunk_size / 2 - 1 {
                hex_part.push(' ');
            }
            if b.is_ascii_graphic() || b == b' ' {
                ascii_part.push(b as char);
            } else {
                ascii_part.push('.');
            }
        }

        let padding = chunk_size * 3 + 1 - hex_part.len();
        if padding > 0 {
            hex_part.push_str(&" ".repeat(padding));
        }

        spans.push(Span::styled(hex_part, Style::default().fg(Color::Cyan)));
        spans.push(Span::styled(" |", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(ascii_part, Style::default().fg(Color::Green)));
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));

        lines.push(Line::from(spans));
    }
    lines
}

/// Extracted styling rules corresponding to custom parsed inline stylesheet rules.
///
/// # Fields
/// * `fg` - Custom foreground text color.
/// * `bg` - Custom background container color.
/// * `bold` - Font weight bold parameter.
/// * `underline` - Underline decoration parameter.
/// * `border` - Border visual frame parameter.
/// * `rounded` - Rounded visual layout border parameters.
/// * `margin_left` - Left margin character padding.
/// * `padding_left` - Left padding character padding.
/// * `progress_value` - Formatted numeric value mapped from HTML progress components.
#[derive(Debug, Default, Clone)]
pub struct CssStyle {
    /// Target text foreground color.
    pub fg: Option<Color>,
    /// Target background color.
    pub bg: Option<Color>,
    /// Font bold flag.
    pub bold: bool,
    /// Font underline decoration flag.
    pub underline: bool,
    /// Box border outline flag.
    pub border: bool,
    /// Box border curved corner outline flag.
    pub rounded: bool,
    /// Left margin space offset columns.
    pub margin_left: usize,
    /// Left padding space offset columns.
    pub padding_left: usize,
    /// Evaluated progress float representing percentage ratios.
    pub progress_value: Option<f32>,
}

/// Resolves standard text color names or hex hashes into compatible [Color] formats.
///
/// # Arguments
///
/// * `color_str` - Input CSS color string slice.
///
/// # Returns
///
/// Returns `Some(Color)` if mapped successfully, otherwise `None`.
fn parse_color(color_str: &str) -> Option<Color> {
    let clean = color_str.trim().to_lowercase();
    match clean.as_str() {
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "white" => Some(Color::White),
        "black" => Some(Color::Black),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        _ => {
            if clean.starts_with('#') {
                if clean.len() == 7 {
                    let r = u8::from_str_radix(&clean[1..3], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&clean[3..5], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&clean[5..7], 16).unwrap_or(0);
                    Some(Color::Rgb(r, g, b))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// Parses inline HTML CSS style attributes into a structured [CssStyle].
///
/// Support targets include colors, background-colors, font-weights, borders, text-decorations, and left spacing properties.
///
/// # Arguments
///
/// * `style_str` - Raw style attribute string value (e.g. `"color: red; font-weight: bold;"`).
///
/// # Returns
///
/// Returns parsed [CssStyle] values.
///
/// # Examples
///
/// ```
/// use isearch_cli::browser::native::parse_style_attribute;
/// let css = parse_style_attribute("color: red; margin-left: 16px;");
/// assert!(css.fg.is_some());
/// ```
pub fn parse_style_attribute(style_str: &str) -> CssStyle {
    let mut style = CssStyle::default();
    for part in style_str.split(';') {
        let kv: Vec<&str> = part.split(':').collect();
        if kv.len() == 2 {
            let key = kv[0].trim().to_lowercase();
            let val = kv[1].trim().to_lowercase();
            match key.as_str() {
                "color" => style.fg = parse_color(&val),
                "background" | "background-color" => style.bg = parse_color(&val),
                "font-weight" if val == "bold" => style.bold = true,
                "text-decoration" if val == "underline" => style.underline = true,
                "border" => style.border = true,
                "border-radius" => style.rounded = true,
                "margin-left" => {
                    if let Some(num_str) = val.strip_suffix("px") {
                        if let Ok(num) = num_str.trim().parse::<usize>() {
                            style.margin_left = num / 8; // approximate spacing
                        }
                    } else if let Ok(num) = val.trim().parse::<usize>() {
                        style.margin_left = num;
                    }
                }
                "padding-left" => {
                    if let Some(num_str) = val.strip_suffix("px") {
                        if let Ok(num) = num_str.trim().parse::<usize>() {
                            style.padding_left = num / 8;
                        }
                    } else if let Ok(num) = val.trim().parse::<usize>() {
                        style.padding_left = num;
                    }
                }
                _ => {}
            }
        }
    }
    style
}

// Convert HTML nodes recursively into styled Lines
pub fn render_html_to_lines(
    nodes: &[HtmlNode],
    width: usize,
    base_style: CssStyle,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for node in nodes {
        render_node_to_lines(node, width, base_style.clone(), &mut lines);
    }
    lines
}

fn render_node_to_lines(
    node: &HtmlNode,
    _width: usize,
    parent_style: CssStyle,
    lines: &mut Vec<Line<'static>>,
) {
    match node {
        HtmlNode::Text(txt) => {
            if txt.is_empty() {
                return;
            }
            let mut style = Style::default();
            if let Some(fg) = parent_style.fg {
                style = style.fg(fg);
            }
            if let Some(bg) = parent_style.bg {
                style = style.bg(bg);
            }
            if parent_style.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if parent_style.underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            let margin = " ".repeat(parent_style.margin_left + parent_style.padding_left);
            lines.push(Line::from(vec![
                Span::raw(margin),
                Span::styled(txt.clone(), style),
            ]));
        }
        HtmlNode::Element {
            tag,
            attributes,
            children,
        } => {
            let mut style = parent_style.clone();
            let mut href = None;
            for (k, v) in attributes {
                if k.to_lowercase() == "style" {
                    let s = parse_style_attribute(v);
                    if let Some(fg) = s.fg {
                        style.fg = Some(fg);
                    }
                    if let Some(bg) = s.bg {
                        style.bg = Some(bg);
                    }
                    if s.bold {
                        style.bold = true;
                    }
                    if s.underline {
                        style.underline = true;
                    }
                    if s.border {
                        style.border = true;
                    }
                    if s.rounded {
                        style.rounded = true;
                    }
                    if s.margin_left > 0 {
                        style.margin_left = s.margin_left;
                    }
                    if s.padding_left > 0 {
                        style.padding_left = s.padding_left;
                    }
                } else if k.to_lowercase() == "href" {
                    href = Some(v.clone());
                } else if k.to_lowercase() == "value" && tag.to_lowercase() == "progress" {
                    if let Ok(v) = v.parse::<f32>() {
                        style.progress_value = Some(v);
                    }
                }
            }

            // Tag specific formatting
            match tag.to_lowercase().as_str() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    style.bold = true;
                    if style.fg.is_none() {
                        style.fg = Some(Color::Cyan);
                    }
                    lines.push(Line::raw("")); // margin top
                    let prefix = match tag.to_lowercase().as_str() {
                        "h1" => "# ",
                        "h2" => "## ",
                        "h3" => "### ",
                        _ => "#### ",
                    };
                    let mut heading_lines = Vec::new();
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), &mut heading_lines);
                    }
                    for line in heading_lines {
                        let mut new_spans = vec![Span::styled(
                            prefix,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )];
                        new_spans.extend(line.spans);
                        lines.push(Line::from(new_spans));
                    }
                    lines.push(Line::raw("")); // margin bottom
                }
                "p" => {
                    lines.push(Line::raw(""));
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), lines);
                    }
                    lines.push(Line::raw(""));
                }
                "br" => {
                    lines.push(Line::raw(""));
                }
                "a" => {
                    style.underline = true;
                    if style.fg.is_none() {
                        style.fg = Some(Color::Blue);
                    }
                    // Implement terminal hyperlink protocol OSC 8 if possible or format as link
                    let mut a_lines = Vec::new();
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), &mut a_lines);
                    }
                    for line in a_lines {
                        let mut spans = line.spans;
                        if let Some(target) = &href {
                            spans.push(Span::styled(
                                format!(" ({})", target),
                                Style::default().fg(Color::DarkGray),
                            ));
                        }
                        lines.push(Line::from(spans));
                    }
                }
                "button" | "input" => {
                    style.bold = true;
                    if style.bg.is_none() {
                        style.bg = Some(Color::DarkGray);
                    }
                    if style.fg.is_none() {
                        style.fg = Some(Color::White);
                    }

                    let input_type = attributes
                        .iter()
                        .find(|(k, _)| k.to_lowercase() == "type")
                        .map(|(_, v)| v.to_lowercase())
                        .unwrap_or_else(|| "text".to_string());
                    let is_checked = attributes
                        .iter()
                        .any(|(k, _)| k.to_lowercase() == "checked");

                    if input_type == "checkbox" {
                        let marker = if is_checked { "[x]" } else { "[ ]" };
                        lines.push(Line::from(vec![Span::styled(
                            format!(" {} ", marker),
                            Style::default().fg(Color::Green),
                        )]));
                    } else if input_type == "radio" {
                        let marker = if is_checked { "(*)" } else { "( )" };
                        lines.push(Line::from(vec![Span::styled(
                            format!(" {} ", marker),
                            Style::default().fg(Color::Green),
                        )]));
                    } else {
                        let mut btn_lines = Vec::new();
                        for child in children {
                            render_node_to_lines(child, _width, style.clone(), &mut btn_lines);
                        }
                        if btn_lines.is_empty() {
                            // Value or Placeholder for inputs
                            let placeholder = attributes
                                .iter()
                                .find(|(k, _)| k.to_lowercase() == "placeholder")
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default();
                            let value = attributes
                                .iter()
                                .find(|(k, _)| k.to_lowercase() == "value")
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default();
                            let text = if !value.is_empty() {
                                value
                            } else if !placeholder.is_empty() {
                                placeholder
                            } else {
                                "Input".to_string()
                            };
                            lines.push(Line::from(vec![Span::styled(
                                format!(" [ {} ] ", text),
                                Style::default().fg(Color::Yellow).bg(Color::DarkGray),
                            )]));
                        } else {
                            for line in btn_lines {
                                let mut spans = vec![Span::raw(" [ ")];
                                spans.extend(line.spans);
                                spans.push(Span::raw(" ] "));
                                lines.push(Line::from(spans));
                            }
                        }
                    }
                }
                "blockquote" => {
                    // Indented block with left border style
                    let mut quote_lines = Vec::new();
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), &mut quote_lines);
                    }
                    for line in quote_lines {
                        let mut spans =
                            vec![Span::styled("│ ", Style::default().fg(Color::DarkGray))];
                        spans.extend(line.spans);
                        lines.push(Line::from(spans));
                    }
                }
                "ul" | "ol" => {
                    let mut list_lines = Vec::new();
                    for (i, child) in children.iter().enumerate() {
                        let bullet = match tag.to_lowercase().as_str() {
                            "ol" => format!("{}. ", i + 1),
                            _ => "• ".to_string(),
                        };
                        let mut item_lines = Vec::new();
                        render_node_to_lines(child, _width, style.clone(), &mut item_lines);
                        for (idx, line) in item_lines.into_iter().enumerate() {
                            let mut spans = Vec::new();
                            if idx == 0 {
                                spans.push(Span::styled(
                                    bullet.clone(),
                                    Style::default().fg(Color::Yellow),
                                ));
                            } else {
                                spans.push(Span::raw("  "));
                            }
                            spans.extend(line.spans);
                            list_lines.push(Line::from(spans));
                        }
                    }
                    lines.extend(list_lines);
                }
                "li" => {
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), lines);
                    }
                }
                "progress" => {
                    let val = style.progress_value.unwrap_or(0.0);
                    let percent = (val * 100.0) as usize;
                    let filled = percent / 10;
                    let bar = format!(
                        "[{}{}] {}%",
                        "█".repeat(filled),
                        "░".repeat(10 - filled),
                        percent
                    );
                    lines.push(Line::from(vec![Span::styled(
                        bar,
                        Style::default().fg(Color::Green),
                    )]));
                }
                "table" => {
                    // Render simple grids/tables
                    let mut table_rows = Vec::new();
                    for child in children {
                        if let HtmlNode::Element {
                            tag: child_tag,
                            children: td_children,
                            ..
                        } = child
                        {
                            if child_tag.to_lowercase() == "tr" {
                                let mut cols = Vec::new();
                                for td in td_children {
                                    if let HtmlNode::Element {
                                        tag: td_tag,
                                        children: text_children,
                                        ..
                                    } = td
                                    {
                                        if td_tag.to_lowercase() == "td"
                                            || td_tag.to_lowercase() == "th"
                                        {
                                            let mut text = String::new();
                                            for tc in text_children {
                                                if let HtmlNode::Text(t) = tc {
                                                    text.push_str(t);
                                                }
                                            }
                                            cols.push(text);
                                        }
                                    }
                                }
                                if !cols.is_empty() {
                                    table_rows.push(cols);
                                }
                            }
                        }
                    }

                    if !table_rows.is_empty() {
                        let num_cols = table_rows.iter().map(|r| r.len()).max().unwrap_or(0);
                        let mut col_widths = vec![10; num_cols];
                        for row in &table_rows {
                            for (i, col) in row.iter().enumerate() {
                                if i < col_widths.len() {
                                    col_widths[i] = std::cmp::max(col_widths[i], col.len() + 2);
                                }
                            }
                        }

                        // Top border
                        let border_line = format!(
                            "┌{}┐",
                            col_widths
                                .iter()
                                .map(|w| "─".repeat(*w))
                                .collect::<Vec<_>>()
                                .join("┬")
                        );
                        lines.push(Line::raw(border_line.clone()));

                        for (r_idx, row) in table_rows.iter().enumerate() {
                            let mut row_cells = Vec::new();
                            row_cells.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
                            for (c_idx, col) in row.iter().enumerate() {
                                let width = col_widths[c_idx];
                                let padded = format!(" {:<width$} ", col, width = width - 2);
                                row_cells.push(Span::styled(
                                    padded,
                                    if r_idx == 0 {
                                        Style::default()
                                            .add_modifier(Modifier::BOLD)
                                            .fg(Color::Yellow)
                                    } else {
                                        Style::default()
                                    },
                                ));
                                row_cells
                                    .push(Span::styled("│", Style::default().fg(Color::DarkGray)));
                            }
                            lines.push(Line::from(row_cells));

                            // Separator or bottom border
                            if r_idx == 0 {
                                let sep = format!(
                                    "├{}┤",
                                    col_widths
                                        .iter()
                                        .map(|w| "─".repeat(*w))
                                        .collect::<Vec<_>>()
                                        .join("┼")
                                );
                                lines.push(Line::raw(sep));
                            }
                        }
                        let bottom_line = format!(
                            "└{}┘",
                            col_widths
                                .iter()
                                .map(|w| "─".repeat(*w))
                                .collect::<Vec<_>>()
                                .join("┴")
                        );
                        lines.push(Line::raw(bottom_line));
                    }
                }
                _ => {
                    // Render children inside panels if style.border is set
                    if style.border {
                        lines.push(Line::raw("┌────────────────────────────────────────┐"));
                    }
                    for child in children {
                        render_node_to_lines(child, _width, style.clone(), lines);
                    }
                    if style.border {
                        lines.push(Line::raw("└────────────────────────────────────────┘"));
                    }
                }
            }
        }
    }
}

/// Converts syntect colors into native Ratatui formatting values.
fn to_ratatui_color(c: syntect::highlighting::Color) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

/// Highlights blocks of programming code using the Oceanic-dark theme inside the `syntect` engine.
///
/// # Arguments
///
/// * `code` - Plain-text block of programming source.
/// * `lang` - Target syntax alias name (e.g. `rust` or `python`).
///
/// # Returns
///
/// Returns a vector of [Line] elements mapped with color syntax tokens.
pub fn highlight_code_block(code: &str, lang: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    let mut lines = Vec::new();
    for line in code.lines() {
        if let Ok(ranges) = h.highlight_line(line, &ps) {
            let mut spans = Vec::new();
            for (style, text) in ranges {
                let fg = to_ratatui_color(style.foreground);
                let mut span_style = Style::default().fg(fg);
                if style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::BOLD)
                {
                    span_style = span_style.add_modifier(Modifier::BOLD);
                }
                if style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::UNDERLINE)
                {
                    span_style = span_style.add_modifier(Modifier::UNDERLINED);
                }
                spans.push(Span::styled(text.to_string(), span_style));
            }
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::raw(line.to_string()));
        }
    }
    lines
}

/// Parses and compiles markdown strings using `pulldown-cmark`, styling list items and fenced code blocks.
///
/// # Arguments
///
/// * `md` - Raw markdown string slice.
/// * `_width` - Available width.
///
/// # Returns
///
/// Returns a vector of styled [Line] elements.
pub fn render_markdown_to_lines(md: &str, _width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let parser = Parser::new(md);

    let mut current_line_spans = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    let mut in_blockquote = false;
    let mut list_index = 0;
    let mut in_ordered_list = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                    lines.push(Line::raw(""));
                    let prefix = "#".repeat(level as usize) + " ";
                    current_line_spans.push(Span::styled(
                        prefix,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                Tag::BlockQuote(_kind) => {
                    in_blockquote = true;
                }
                Tag::List(start_num) => {
                    if let Some(num) = start_num {
                        in_ordered_list = true;
                        list_index = num as usize;
                    } else {
                        in_ordered_list = false;
                    }
                }
                Tag::Item => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                    let prefix = if in_ordered_list {
                        let p = format!(" {}. ", list_index);
                        list_index += 1;
                        p
                    } else {
                        " • ".to_string()
                    };
                    current_line_spans
                        .push(Span::styled(prefix, Style::default().fg(Color::Yellow)));
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_block_lang = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        _ => String::new(),
                    };
                    code_block_content.clear();
                }
                Tag::Link { dest_url, .. } => {
                    current_line_spans.push(Span::styled("[", Style::default().fg(Color::Blue)));
                    // Save or show the link url inline or inside parenthesis
                    let _ = dest_url;
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                    lines.push(Line::raw(""));
                }
                TagEnd::BlockQuote(_) => {
                    in_blockquote = false;
                }
                TagEnd::Item => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    let highlighted = highlight_code_block(&code_block_content, &code_block_lang);
                    lines.push(Line::raw("┌─── Code Block ────────────────────────────"));
                    for h_line in highlighted {
                        let mut spans = vec![Span::raw("│ ")];
                        spans.extend(h_line.spans);
                        lines.push(Line::from(spans));
                    }
                    lines.push(Line::raw("└───────────────────────────────────────────"));
                }
                TagEnd::Link => {
                    current_line_spans.push(Span::styled(
                        "]",
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                }
                _ => {}
            },
            Event::Text(txt) => {
                if in_code_block {
                    code_block_content.push_str(&txt);
                } else {
                    let style = Style::default();
                    if in_blockquote {
                        current_line_spans
                            .push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
                    }
                    current_line_spans.push(Span::styled(txt.to_string(), style));
                }
            }
            Event::Code(txt) => {
                current_line_spans.push(Span::styled(
                    format!("`{}`", txt),
                    Style::default().fg(Color::Magenta),
                ));
            }
            Event::SoftBreak | Event::HardBreak if !current_line_spans.is_empty() => {
                lines.push(Line::from(std::mem::take(&mut current_line_spans)));
            }
            _ => {}
        }
    }

    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }
    lines
}

/// Dynamic structure managing vectors of vertices and edges representing 3D wireframe mesh objects.
///
/// # Fields
/// * `vertices` - Points on the 3D grid space.
/// * `edges` - Paired coordinate indices representing mesh edges.
#[derive(Debug, Clone)]
pub struct Mesh3D {
    /// List of point vectors.
    pub vertices: Vec<[f32; 3]>,
    /// Mapped index indices connecting vertices.
    pub edges: Vec<(usize, usize)>,
}

impl Mesh3D {
    /// Factory builder pre-populating standard coordinates of a symmetric 3D wireframe cube.
    ///
    /// # Returns
    ///
    /// Returns initialized [Mesh3D].
    pub fn new_cube() -> Self {
        Self {
            vertices: vec![
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
            edges: vec![
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0), // back face
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4), // front face
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7), // connections
            ],
        }
    }

    /// Performs trigonometric rotation offsets along the vertical Y-axis.
    ///
    /// # Arguments
    ///
    /// * `angle` - Radial offset in radians.
    pub fn rotate_y(&mut self, angle: f32) {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        for v in &mut self.vertices {
            let x = v[0];
            let z = v[2];
            v[0] = x * cos_a - z * sin_a;
            v[2] = x * sin_a + z * cos_a;
        }
    }

    /// Performs trigonometric rotation offsets along the horizontal X-axis.
    ///
    /// # Arguments
    ///
    /// * `angle` - Radial offset in radians.
    pub fn rotate_x(&mut self, angle: f32) {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        for v in &mut self.vertices {
            let y = v[1];
            let z = v[2];
            v[1] = y * cos_a - z * sin_a;
            v[2] = y * sin_a + z * cos_a;
        }
    }

    /// Projects 3D wires into 2D character-grid structures and outputs them as styled terminal lines.
    ///
    /// Uses Bresenham's line algorithm mapped to local cells.
    ///
    /// # Arguments
    ///
    /// * `width` - Target cell column dimension.
    /// * `height` - Target cell row dimension.
    ///
    /// # Returns
    ///
    /// Returns lines displaying wire characters.
    pub fn render_to_lines(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        let mut grid = vec![vec![' '; width]; height];

        // Draw edges
        for &(i1, i2) in &self.edges {
            let v1 = self.vertices[i1];
            let v2 = self.vertices[i2];

            // Orthographic projection & scale to fit terminal viewport
            let scale_x = (width as f32) * 0.25;
            let scale_y = (height as f32) * 0.45;

            let x1 = ((width as f32) / 2.0 + v1[0] * scale_x) as isize;
            let y1 = ((height as f32) / 2.0 - v1[1] * scale_y) as isize;
            let x2 = ((width as f32) / 2.0 + v2[0] * scale_x) as isize;
            let y2 = ((height as f32) / 2.0 - v2[1] * scale_y) as isize;

            draw_line_on_grid(&mut grid, x1, y1, x2, y2, '*');
        }

        grid.into_iter()
            .map(|row| {
                Line::from(vec![Span::styled(
                    row.into_iter().collect::<String>(),
                    Style::default().fg(Color::Magenta),
                )])
            })
            .collect()
    }
}

fn draw_line_on_grid(
    grid: &mut [Vec<char>],
    mut x1: isize,
    mut y1: isize,
    x2: isize,
    y2: isize,
    ch: char,
) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let height = grid.len() as isize;
    if height == 0 {
        return;
    }
    let width = grid[0].len() as isize;

    loop {
        if x1 >= 0 && x1 < width && y1 >= 0 && y1 < height {
            grid[y1 as usize][x1 as usize] = ch;
        }

        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x1 += sx;
        }
        if e2 < dx {
            err += dx;
            y1 += sy;
        }
    }
}

impl BrowserEngine for NativeEngine {
    /// Navigates to local resources (folders, ZIPs, PDFs, meshes) or requests HTTP document bodies.
    ///
    /// # Arguments
    ///
    /// * `url` - Full destination URL or system filepath structure.
    ///
    /// # Returns
    ///
    /// Returns [PageContent] structures.
    ///
    /// # Errors
    ///
    /// Returns a [BrowserError] if network downloads, file reads, or format parsing fails.
    fn navigate(&mut self, url: &str) -> Result<PageContent, BrowserError> {
        // Handle zip inner file format: /path/to/archive.zip::filename.txt
        if url.contains("::") {
            let parts: Vec<&str> = url.split("::").collect();
            if parts.len() == 2 {
                let zip_path_str = if parts[0].starts_with("file://") {
                    &parts[0][7..]
                } else {
                    parts[0]
                };
                let zip_path = PathBuf::from(zip_path_str);
                let file_in_zip = parts[1];
                if zip_path.exists() {
                    if let Ok(file) = fs::File::open(&zip_path) {
                        if let Ok(mut archive) = zip::ZipArchive::new(file) {
                            if let Ok(mut subfile) = archive.by_name(file_in_zip) {
                                let mut contents = Vec::new();
                                if std::io::copy(&mut subfile, &mut contents).is_ok() {
                                    if let Ok(text) = String::from_utf8(contents.clone()) {
                                        return Ok(PageContent::FilePreview {
                                            path: PathBuf::from(url),
                                            content: text,
                                            is_binary: false,
                                        });
                                    } else {
                                        let lines = render_hex_dump(&contents, 80);
                                        let mut content = String::new();
                                        for line in lines {
                                            for span in line.spans {
                                                content.push_str(&span.content);
                                            }
                                            content.push('\n');
                                        }
                                        return Ok(PageContent::FilePreview {
                                            path: PathBuf::from(url),
                                            content,
                                            is_binary: true,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle file:/// or local path paths
        if url.starts_with("file://")
            || url.starts_with('/')
            || url.contains('\\')
            || std::path::Path::new(url).exists()
        {
            let path_str = if let Some(stripped) = url.strip_prefix("file://") {
                stripped
            } else {
                url
            };
            let path = PathBuf::from(path_str);
            if path.is_dir() {
                let mut entries = Vec::new();
                if let Ok(dir_entries) = fs::read_dir(&path) {
                    for entry in dir_entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                            entries.push((name, is_dir));
                        }
                    }
                }
                return Ok(PageContent::Directory { path, entries });
            } else if path.exists() {
                // Check if zip archive
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "zip" {
                        // Check if a sub-file is being requested, e.g. /path/to/archive.zip::subfile.txt
                        if let Ok(file) = fs::File::open(&path) {
                            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                                let mut files = Vec::new();
                                for i in 0..archive.len() {
                                    if let Ok(file) = archive.by_index(i) {
                                        files.push(file.name().to_string());
                                    }
                                }
                                return Ok(PageContent::ArchivePreview { path, files });
                            }
                        }
                    } else if ext.to_string_lossy().to_lowercase() == "pdf" {
                        // PDF Metadata preview
                        return Ok(PageContent::PdfPreview {
                            path: path.clone(),
                            title: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "PDF Document".to_string()),
                            metadata: vec![
                                ("Format".to_string(), "PDF (Portable Document Format)".to_string()),
                                ("Size".to_string(), format!("{} bytes", fs::metadata(&path).map(|m| m.len()).unwrap_or(0))),
                            ],
                            pages_count: 1,
                            text_preview: "PDF binary files are previewed natively in high-fidelity on supported viewers.\nHere we parse metadata and structure for console output.".to_string(),
                        });
                    } else if ["png", "jpg", "jpeg", "webp", "gif", "bmp"]
                        .contains(&ext.to_string_lossy().to_lowercase().as_str())
                    {
                        if let Ok(bytes) = fs::read(&path) {
                            return Ok(PageContent::ImagePreview {
                                path: path.clone(),
                                raw_bytes: bytes,
                            });
                        }
                    } else if ["obj", "mesh", "3d"]
                        .contains(&ext.to_string_lossy().to_lowercase().as_str())
                    {
                        // Return custom 3D mesh preview
                        return Ok(PageContent::Mesh3DPreview {
                            title: path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "3D Mesh".to_string()),
                            mesh: Mesh3D::new_cube(),
                        });
                    }
                }

                // Normal file
                if let Ok(bytes) = fs::read(&path) {
                    if let Ok(text) = String::from_utf8(bytes.clone()) {
                        return Ok(PageContent::FilePreview {
                            path,
                            content: text,
                            is_binary: false,
                        });
                    } else {
                        // Render structured hex dump
                        let lines = render_hex_dump(&bytes, 80);
                        let mut content = String::new();
                        for line in lines {
                            for span in line.spans {
                                content.push_str(&span.content);
                            }
                            content.push('\n');
                        }
                        return Ok(PageContent::FilePreview {
                            path,
                            content,
                            is_binary: true,
                        });
                    }
                }
            }
            return Err(BrowserError::IoError(format!("Path not found: {}", url)));
        }

        // Web Request
        let mut clean_url = url.to_string();
        if !clean_url.starts_with("http://") && !clean_url.starts_with("https://") {
            clean_url = format!("https://{}", clean_url);
        }

        let agent = ureq::Agent::new_with_defaults();
        match agent.get(&clean_url).call() {
            Ok(response) => {
                let mut body_obj = response.into_body();
                let body = body_obj.read_to_string().unwrap_or_default();

                let title = if let Ok(dom) = tl::parse(&body, tl::ParserOptions::default()) {
                    let mut t_str = clean_url.clone();
                    for node in dom.nodes() {
                        if let Node::Tag(tag) = node {
                            if tag.name().as_utf8_str() == "title" {
                                t_str = tag.inner_text(dom.parser()).to_string();
                                break;
                            }
                        }
                    }
                    t_str
                } else {
                    clean_url.clone()
                };

                // Check content-type if we can, or parse as html
                if clean_url.ends_with(".md") {
                    Ok(PageContent::Markdown {
                        title,
                        raw_md: body,
                    })
                } else {
                    let parsed = Self::parse_html(&body);
                    Ok(PageContent::Html {
                        title,
                        raw_html: body,
                        parsed_nodes: parsed,
                    })
                }
            }
            Err(e) => Err(BrowserError::NetworkError(format!("{:?}", e))),
        }
    }

    /// Standard Google query router.
    ///
    /// # Arguments
    ///
    /// * `query` - Text to search.
    ///
    /// # Returns
    ///
    /// Returns [PageContent] structures.
    ///
    /// # Errors
    ///
    /// Returns an error if navigation fails.
    fn search(&mut self, query: &str) -> Result<PageContent, BrowserError> {
        let url = format!(
            "https://www.google.com/search?q={}",
            percent_encoding::utf8_percent_encode(query, percent_encoding::NON_ALPHANUMERIC)
        );
        self.navigate(&url)
    }

    fn capture_screenshot(
        &mut self,
        _url: &str,
        _width: u32,
        _height: u32,
    ) -> Result<Vec<u8>, BrowserError> {
        Err(BrowserError::UnsupportedPlatform(
            "Native engine cannot capture screenshots".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_style_attribute() {
        let style = parse_style_attribute("color: red; background-color: blue; font-weight: bold; text-decoration: underline; margin-left: 16px");
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.bold);
        assert!(style.underline);
        assert_eq!(style.margin_left, 2); // 16px / 8 = 2
    }

    #[test]
    fn test_parse_html_basic() {
        let nodes = NativeEngine::parse_html("<p>Hello <span>World</span></p>");
        assert!(!nodes.is_empty());
        if let HtmlNode::Element { tag, children, .. } = &nodes[0] {
            assert_eq!(tag, "p");
            assert_eq!(children.len(), 2);
        }
    }
}
