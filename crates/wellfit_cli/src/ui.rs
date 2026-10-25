#![forbid(unsafe_code)]
#![allow(dead_code)]

//! UI helpers for the Wellfit CLI: colored output via ANSI escape codes,
//! progress indicators, structured tables, panels, and configuration loading.
//!
//! All output respects the `NO_COLOR`/`CI` environment variables and the
//! `--color`/`--quiet`/`--verbose` flags. No external crates are required.
//!
//! Design principles:
//! - Visual consistency: unified color palette and formatting
//! - Accessibility: respects NO_COLOR, uses icons + text (not color alone)
//! - Responsiveness: adaptive table widths, compact mode for narrow terminals
//! - Ease of use: clear status indicators, helpful error messages

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// ANSI escape code constants.
mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const CLEAR_LINE: &str = "\x1b[2K\r";
    pub const HIDE_CURSOR: &str = "\x1b[?25l";
    pub const SHOW_CURSOR: &str = "\x1b[?25h";
    pub const MOVE_UP: &str = "\x1b[1A";
}

/// Status level for visual indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warning,
    Error,
    Info,
    Pending,
}

impl Status {
    /// Icon for the status (works without color).
    pub fn icon(&self) -> &'static str {
        match self {
            Status::Ok => "✔",
            Status::Warning => "⚠",
            Status::Error => "✗",
            Status::Info => "ℹ",
            Status::Pending => "…",
        }
    }

    /// ANSI color code for the status.
    pub fn color(&self) -> &'static str {
        match self {
            Status::Ok => ansi::GREEN,
            Status::Warning => ansi::YELLOW,
            Status::Error => ansi::RED,
            Status::Info => ansi::BLUE,
            Status::Pending => ansi::CYAN,
        }
    }
}

/// Runtime UI configuration, threaded through the CLI.
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub color: ColorChoice,
    pub quiet: bool,
    pub verbose: bool,
    pub compact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    Always,
    Never,
    Auto,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color: ColorChoice::Auto,
            quiet: false,
            verbose: false,
            compact: false,
        }
    }
}

impl UiConfig {
    /// Load configuration from `wellfit.toml` in the given directory, if present.
    pub fn load(config_dir: &Path) -> Self {
        let config_path = config_dir.join("wellfit.toml");
        if let Ok(text) = fs::read_to_string(&config_path) {
            if let Ok(raw) = parse_toml(&text) {
                let mut cfg = Self::default();
                if let Some(cli) = raw.get("cli") {
                    if let Some(c) = cli.as_str() {
                        cfg.color = match c {
                            "always" => ColorChoice::Always,
                            "never" => ColorChoice::Never,
                            _ => ColorChoice::Auto,
                        };
                    }
                    if let Some(q) = cli.as_bool() {
                        cfg.quiet = q;
                    }
                    if let Some(v) = cli.as_bool() {
                        cfg.verbose = v;
                    }
                    if let Some(c) = cli.as_bool() {
                        cfg.compact = c;
                    }
                }
                return cfg;
            }
        }
        Self::default()
    }

    /// Whether color output is enabled, honouring `NO_COLOR`/`CI`.
    pub fn color_enabled(&self) -> bool {
        match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                if std::env::var("NO_COLOR").is_ok() || std::env::var("CI").as_deref() == Ok("true")
                {
                    return false;
                }
                true
            }
        }
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn is_compact(&self) -> bool {
        self.compact
    }

    /// Wrap text with ANSI color codes if color is enabled.
    pub fn style(&self, text: &str, code: &str) -> String {
        if self.color_enabled() {
            format!("{}{}{}", code, text, ansi::RESET)
        } else {
            text.to_string()
        }
    }

    /// Style text with a status color.
    pub fn status_style(&self, text: &str, status: Status) -> String {
        if self.color_enabled() {
            format!("{}{}{}{}", status.color(), ansi::BOLD, text, ansi::RESET)
        } else {
            format!("{} {}", status.icon(), text)
        }
    }
}

/// Simple TOML value for config parsing.
#[derive(Debug, Clone)]
enum TomlValue {
    Str(String),
    Bool(bool),
    Table(HashMap<String, TomlValue>),
}

impl TomlValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            TomlValue::Str(s) => Some(s),
            _ => None,
        }
    }
    fn as_bool(&self) -> Option<bool> {
        match self {
            TomlValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn get(&self, key: &str) -> Option<&TomlValue> {
        match self {
            TomlValue::Table(map) => map.get(key),
            _ => None,
        }
    }
}

/// Minimal TOML parser sufficient for the wellfit.toml config file.
fn parse_toml(text: &str) -> Result<HashMap<String, TomlValue>, String> {
    let mut root: HashMap<String, TomlValue> = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let section = rest.strip_suffix(']').unwrap_or(rest).trim();
            current_section = Some(section.to_string());
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            let toml_value = if value.starts_with('"') && value.ends_with('"') {
                TomlValue::Str(value[1..value.len() - 1].to_string())
            } else if value == "true" || value == "false" {
                TomlValue::Bool(value == "true")
            } else {
                TomlValue::Str(value.to_string())
            };

            match &current_section {
                Some(section) => {
                    let table = root
                        .entry(section.clone())
                        .or_insert_with(|| TomlValue::Table(HashMap::new()));
                    if let TomlValue::Table(map) = table {
                        map.insert(key.to_string(), toml_value);
                    }
                }
                None => {
                    root.insert(key.to_string(), toml_value);
                }
            }
        }
    }

    Ok(root)
}

// ---------------------------------------------------------------- output helpers

/// Print an informational line (suppressed in quiet mode).
pub fn info(ui: &UiConfig, message: &str) {
    if !ui.is_quiet() {
        println!("{}", message);
    }
}

/// Print an error line to stderr with icon and color.
pub fn error(message: &str) {
    eprintln!("{} ✗ ERROR: {}", ansi::RED, message);
    eprintln!("{}", ansi::RESET);
}

/// Print a warning line to stderr with icon and color.
pub fn warn(ui: &UiConfig, message: &str) {
    if !ui.is_quiet() {
        if ui.color_enabled() {
            eprintln!("{} ⚠ WARN: {}{}", ansi::YELLOW, ansi::BOLD, message);
            eprintln!("{}", ansi::RESET);
        } else {
            eprintln!("⚠ WARN: {message}");
        }
    }
}

/// Print a success line with icon and color.
pub fn ok(ui: &UiConfig, message: &str) {
    if !ui.is_quiet() {
        if ui.color_enabled() {
            println!("{} ✔ {}{}", ansi::GREEN, ansi::BOLD, message);
            println!("{}", ansi::RESET);
        } else {
            println!("✔ {message}");
        }
    }
}

/// Print a verbose diagnostic line (only in verbose mode).
pub fn verbose(ui: &UiConfig, message: &str) {
    if ui.is_verbose() && !ui.is_quiet() {
        if ui.color_enabled() {
            println!("{} debug:{} {}", ansi::DIM, ansi::RESET, message);
        } else {
            println!("debug: {message}");
        }
    }
}

/// Print a section heading with underline.
pub fn heading(ui: &UiConfig, message: &str) {
    if !ui.is_quiet() {
        println!();
        if ui.color_enabled() {
            println!("{}{}{}{}", ansi::CYAN, ansi::BOLD, message, ansi::RESET);
            println!(
                "{}{}{}",
                ansi::DIM,
                "─".repeat(message.chars().count().min(60)),
                ansi::RESET
            );
        } else {
            println!("{}", message);
            println!("{}", "-".repeat(message.chars().count().min(60)));
        }
    }
}

/// Print a sub-heading (smaller, less prominent).
pub fn subheading(ui: &UiConfig, message: &str) {
    if !ui.is_quiet() {
        if ui.color_enabled() {
            println!("{}{}{}{}", ansi::BLUE, ansi::BOLD, message, ansi::RESET);
        } else {
            println!("  {message}");
        }
    }
}

/// Print a key-value pair with aligned key.
pub fn kv(ui: &UiConfig, key: &str, value: &str) {
    if !ui.is_quiet() {
        if ui.color_enabled() {
            println!(
                "  {}{}{:<18}{} {}",
                ansi::DIM,
                ansi::RESET,
                key,
                ansi::RESET,
                value
            );
        } else {
            println!("  {:<18} {}", key, value);
        }
    }
}

/// Print a key-value pair with status color.
pub fn kv_status(ui: &UiConfig, key: &str, value: &str, status: Status) {
    if !ui.is_quiet() {
        let styled_value = ui.status_style(value, status);
        if ui.color_enabled() {
            println!("  {}{:<18}{} {}", ansi::DIM, key, ansi::RESET, styled_value);
        } else {
            println!("  {:<18} {} {}", key, status.icon(), value);
        }
    }
}

/// Print a separator line.
pub fn print_separator(len: usize) {
    let width = len.min(80);
    println!("{}{}{}", ansi::DIM, "─".repeat(width), ansi::RESET);
}

/// Print a table row with optional header styling.
pub fn print_table_row<T: std::fmt::Display>(cells: &[T], header: bool) {
    let cells: Vec<String> = cells.iter().map(|c| c.to_string()).collect();
    let line = cells.join(" │ ");
    if header {
        if true {
            println!("{}{}{}", ansi::BOLD, line, ansi::RESET);
        } else {
            println!("{}", line);
        }
    } else {
        println!("{}", line);
    }
}

/// Print a table with borders.
pub fn print_table(ui: &UiConfig, headers: &[&str], rows: &[Vec<String>], widths: &[usize]) {
    if ui.is_quiet() {
        return;
    }

    // Print header
    let mut header_line = String::new();
    let mut separator = String::new();
    for (i, h) in headers.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(12);
        if ui.color_enabled() {
            header_line.push_str(&format!("{}{:<w$}{} │ ", ansi::BOLD, h, ansi::RESET, w = w));
        } else {
            header_line.push_str(&format!("{:<w$} │ ", h, w = w));
        }
        separator.push_str(&format!("{}{}{}", "─".repeat(w), "┼", "─"));
    }
    println!("{}", header_line.trim_end_matches(" │ "));
    println!("{}", separator.trim_end_matches("─1"));

    // Print rows
    for row in rows {
        let mut row_line = String::new();
        for (i, cell) in row.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(12);
            row_line.push_str(&format!("{:<w$} │ ", cell, w = w));
        }
        println!("{}", row_line.trim_end_matches(" │ "));
    }
}

/// Create a simple spinner for indeterminate work.
pub struct Spinner {
    message: String,
    start: Instant,
    enabled: bool,
    chars: [&'static str; 8],
    finished: bool,
}

impl Spinner {
    pub fn new(message: &str, enabled: bool) -> Self {
        Self {
            message: message.to_string(),
            start: Instant::now(),
            enabled,
            chars: ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"],
            finished: false,
        }
    }

    pub fn tick(&self) {
        if !self.enabled || self.finished {
            return;
        }
        let idx = (self.start.elapsed().as_millis() / 80) as usize % self.chars.len();
        print!("{} {} {}", ansi::CLEAR_LINE, self.chars[idx], self.message);
        let _ = std::io::stdout().flush();
    }

    pub fn finish(mut self, message: Option<&str>) {
        if self.enabled {
            print!("{}", ansi::CLEAR_LINE);
            if let Some(msg) = message {
                if true {
                    println!("{} ✔ {}{}", ansi::GREEN, ansi::BOLD, msg);
                    println!("{}", ansi::RESET);
                } else {
                    println!("✔ {msg}");
                }
            }
        }
        self.finished = true;
    }
}

/// Simple progress bar for known-length work.
pub struct ProgressBar {
    message: String,
    total: u64,
    pos: u64,
    start: Instant,
    enabled: bool,
    width: usize,
}

impl ProgressBar {
    pub fn new(message: &str, len: u64, enabled: bool) -> Self {
        Self {
            message: message.to_string(),
            total: len,
            pos: 0,
            start: Instant::now(),
            enabled,
            width: 40,
        }
    }

    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    pub fn inc(&mut self, delta: u64) {
        self.pos += delta;
        self.render();
    }

    pub fn set_pos(&mut self, pos: u64) {
        self.pos = pos;
        self.render();
    }

    fn render(&self) {
        if !self.enabled {
            return;
        }
        let ratio = if self.total > 0 {
            self.pos as f64 / self.total as f64
        } else {
            0.0
        };
        let filled = (ratio * self.width as f64) as usize;
        let bar: String = std::iter::repeat_n('█', filled)
            .chain(std::iter::repeat_n('░', self.width - filled))
            .collect();
        let elapsed = self.start.elapsed().as_secs_f64();
        let eta = if ratio > 0.0 {
            (elapsed / ratio - elapsed).max(0.0)
        } else {
            0.0
        };
        let pct = (ratio * 100.0) as u8;
        print!(
            "{}{} {} [{}] {}/{} ({}%) ETA {:.0}s{}",
            ansi::CLEAR_LINE,
            ansi::GREEN,
            self.message,
            bar,
            self.pos,
            self.total,
            pct,
            eta,
            ansi::RESET
        );
        let _ = std::io::stdout().flush();
    }

    pub fn finish(self) {
        if self.enabled {
            print!("{}", ansi::CLEAR_LINE);
            println!("{}✔ {} complete{}", ansi::GREEN, self.message, ansi::RESET);
        }
    }
}

/// Print a panel/box with title and content.
pub fn panel(ui: &UiConfig, title: &str, lines: &[&str]) {
    if ui.is_quiet() {
        return;
    }
    let max_len = lines
        .iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(0)
        .max(title.len());
    let width = max_len + 4;

    if ui.color_enabled() {
        println!("{}┌{}┐{}", ansi::CYAN, "─".repeat(width), ansi::RESET);
        println!(
            "{}│ {}{:<width$} │{}",
            ansi::CYAN,
            ansi::BOLD,
            title,
            ansi::RESET,
            width = width - 2
        );
        println!("{}├{}┤{}", ansi::CYAN, "─".repeat(width), ansi::RESET);
        for line in lines {
            println!(
                "{}│ {:<width$} │{}",
                ansi::CYAN,
                line,
                ansi::RESET,
                width = width - 2
            );
        }
        println!("{}└{}┘{}", ansi::CYAN, "─".repeat(width), ansi::RESET);
    } else {
        println!("+{}+", "-".repeat(width));
        println!("| {:<width$} |", title, width = width - 2);
        println!("+{}+", "-".repeat(width));
        for line in lines {
            println!("| {:<width$} |", line, width = width - 2);
        }
        println!("+{}+", "-".repeat(width));
    }
}

/// Print a summary box with pass/warn/fail counts.
pub fn summary_box(ui: &UiConfig, pass: usize, warn: usize, fail: usize, elapsed: f64) {
    if ui.is_quiet() {
        return;
    }
    let lines = [
        format!("Pass:  {}", pass),
        format!("Warn:  {}", warn),
        format!("Fail:  {}", fail),
        format!("Time:  {:.2}s", elapsed),
    ];
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    panel(ui, "Verification Summary", &refs);
}

/// Print a status indicator line.
pub fn status_line(ui: &UiConfig, label: &str, status: Status, detail: &str) {
    if ui.is_quiet() {
        return;
    }
    let icon = status.icon();
    let styled_label = ui.style(label, ansi::BOLD);
    if ui.color_enabled() {
        println!(
            "  {} {} {:<20} {}",
            status.color(),
            icon,
            styled_label,
            detail
        );
        print!("{}", ansi::RESET);
    } else {
        println!("  {} {:<20} {}", icon, label, detail);
    }
}

/// Print a help/command reference.
pub fn print_help(ui: &UiConfig, commands: &[(&str, &str)]) {
    if ui.is_quiet() {
        return;
    }
    heading(ui, "Available Commands");
    for (cmd, desc) in commands {
        if ui.color_enabled() {
            println!(
                "  {}{}{:<12}{} {}",
                ansi::CYAN,
                ansi::BOLD,
                cmd,
                ansi::RESET,
                desc
            );
        } else {
            println!("  {:<12} {}", cmd, desc);
        }
    }
}

/// A small terminal width probe.
pub fn terminal_width() -> usize {
    80
}
