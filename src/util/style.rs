//! Terminal styling: ANSI colors, status glyphs, and spinners.
//!
//! Everything here is **purely cosmetic** and routes through a single gate
//! (`color_enabled` / the spinner's interactivity check) so it never corrupts
//! machine-readable output. Rules:
//!
//! - Color is emitted only when stdout is a TTY and `NO_COLOR` is unset.
//! - Spinners render only in interactive human mode and draw to **stderr**, so
//!   they never touch the bytes a caller pipes from stdout (or parses as JSON).

use std::borrow::Cow;
use std::time::Duration;

use anstyle::{AnsiColor, Color, Style};
use indicatif::{ProgressBar, ProgressStyle};

use crate::util::output;

const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
const CYAN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
const BLUE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Blue)));
const MAGENTA: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Magenta)));
const DIM: Style = Style::new().dimmed();
const BOLD: Style = Style::new().bold();
const ERROR_LABEL: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
    .bold();
const LINK: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Cyan)))
    .underline();

/// True when ANSI styling should be emitted: stdout is a TTY and `NO_COLOR`
/// is unset. Mirrors the gate the rest of the CLI uses for human output.
pub fn color_enabled() -> bool {
    output::is_tty() && !output::no_color()
}

fn paint(style: Style, text: &str) -> String {
    if color_enabled() {
        format!("{}{text}{}", style.render(), style.render_reset())
    } else {
        text.to_string()
    }
}

// --- Named colors ---

pub fn green(text: &str) -> String {
    paint(GREEN, text)
}

pub fn red(text: &str) -> String {
    paint(RED, text)
}

pub fn yellow(text: &str) -> String {
    paint(YELLOW, text)
}

pub fn cyan(text: &str) -> String {
    paint(CYAN, text)
}

pub fn blue(text: &str) -> String {
    paint(BLUE, text)
}

pub fn magenta(text: &str) -> String {
    paint(MAGENTA, text)
}

pub fn dim(text: &str) -> String {
    paint(DIM, text)
}

pub fn bold(text: &str) -> String {
    paint(BOLD, text)
}

/// Bold red label, e.g. the `Error:` prefix on failures.
pub fn error_label(text: &str) -> String {
    paint(ERROR_LABEL, text)
}

/// Cyan underlined text for URLs.
pub fn link(text: &str) -> String {
    paint(LINK, text)
}

// --- Status glyphs (colored, with the symbol baked in) ---

/// Green check mark for success states.
pub fn tick() -> String {
    green("✓")
}

/// Red cross for failure states.
pub fn cross() -> String {
    red("✗")
}

/// Yellow bang for warnings / degraded states.
pub fn bang() -> String {
    yellow("!")
}

/// Dim arrow used to prefix follow-up hints / fixes.
pub fn arrow() -> String {
    dim("→")
}

// --- Interactive prompts ---

/// Theme for `dialoguer` prompts: colorful when color is enabled, plain
/// otherwise. Use as `Input::with_theme(&*style::prompt_theme())`.
pub fn prompt_theme() -> Box<dyn dialoguer::theme::Theme> {
    if color_enabled() {
        Box::new(dialoguer::theme::ColorfulTheme::default())
    } else {
        Box::new(dialoguer::theme::SimpleTheme)
    }
}

// --- Boxes ---

/// Render `content` inside a rounded single-line box, the content bold and the
/// border dim. Lines are prefixed with `indent`. Used to spotlight a value the
/// user needs to copy (e.g. a device login code).
pub fn code_box(content: &str, indent: &str) -> String {
    const PAD: usize = 2;
    let rule = "─".repeat(content.chars().count() + PAD * 2);
    let pad = " ".repeat(PAD);
    let bar = dim("│");
    let top = dim(&format!("╭{rule}╮"));
    let bot = dim(&format!("╰{rule}╯"));
    let mid = format!("{bar}{pad}{}{pad}{bar}", bold(content));
    format!("{indent}{top}\n{indent}{mid}\n{indent}{bot}")
}

// --- Spinners ---

/// A best-effort spinner. In non-interactive contexts (piped stdout, `--json`,
/// non-TTY) it is a no-op so call sites stay uniform. Drawn to stderr.
pub struct Spinner(Option<ProgressBar>);

/// Start a spinner with `message`. No-op (but still a valid handle) when output
/// is not an interactive human terminal.
pub fn spinner(message: impl Into<Cow<'static, str>>) -> Spinner {
    if !output::is_tty() || output::is_json() {
        return Spinner(None);
    }

    let template = if output::no_color() {
        "{spinner} {msg}"
    } else {
        "{spinner:.cyan} {msg}"
    };
    let style = ProgressStyle::with_template(template)
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");

    let pb = ProgressBar::new_spinner();
    pb.set_style(style);
    pb.set_message(message);
    pb.enable_steady_tick(Duration::from_millis(80));
    Spinner(Some(pb))
}

impl Spinner {
    /// Update the spinner's message in place.
    pub fn set_message(&self, message: impl Into<Cow<'static, str>>) {
        if let Some(pb) = &self.0 {
            pb.set_message(message);
        }
    }

    /// Stop the spinner and erase its line. Call this before writing results to
    /// stdout so the cleared line and the output don't interleave.
    pub fn finish_and_clear(self) {
        if let Some(pb) = &self.0 {
            pb.finish_and_clear();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // Safety net for early returns (`?`) so a failed command never leaves a
        // dangling spinner line behind. Idempotent with `finish_and_clear`.
        if let Some(pb) = &self.0 {
            pb.finish_and_clear();
        }
    }
}
