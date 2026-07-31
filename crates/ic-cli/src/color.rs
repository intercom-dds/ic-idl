// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::borrow::Cow;
use std::fmt::Display;
use std::sync::{OnceLock, RwLock};

use crate::terminal;

/// Controls when colors should be used
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Always use colors
    Always,

    /// Never use colors
    Never,

    /// Automatically detect based on terminal capabilities
    #[default]
    Auto,
}

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Gray,
    Clear,
    Rgb(u8, u8, u8),
}

/// A colorized string that respects the color mode
pub struct Colored<T> {
    value: T,
    style: Style,
    mode: ColorMode,
}

#[derive(Clone, Debug, Default)]
struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    dim: bool,
}

impl<T: Display> Display for Colored<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let effective_mode = effective_color_mode(self.mode);
        let use_color = match effective_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => has_colors(),
        };

        if !use_color {
            return write!(f, "{}", self.value);
        }

        let mut codes = vec![];
        if let Some(color) = self.style.fg {
            codes.push(match color {
                Color::Black => Cow::Borrowed("38;5;0"),
                Color::Red => Cow::Borrowed("38;5;1"),
                Color::Green => Cow::Borrowed("38;5;2"),
                Color::Yellow => Cow::Borrowed("38;5;3"),
                Color::Blue => Cow::Borrowed("38;5;4"),
                Color::Purple => Cow::Borrowed("38;5;5"),
                Color::Cyan => Cow::Borrowed("38;5;6"),
                Color::White => Cow::Borrowed("38;5;7"),
                Color::Gray => Cow::Borrowed("38;5;8"),
                Color::Clear => Cow::Borrowed("39"),
                Color::Rgb(r, g, b) => Cow::Owned(format!("38;2;{r};{g};{b}")),
            });
        }

        if let Some(color) = self.style.bg {
            codes.push(match color {
                Color::Black => Cow::Borrowed("40"),
                Color::Red => Cow::Borrowed("41"),
                Color::Green => Cow::Borrowed("42"),
                Color::Yellow => Cow::Borrowed("43"),
                Color::Blue => Cow::Borrowed("44"),
                Color::Purple => Cow::Borrowed("45"),
                Color::Cyan => Cow::Borrowed("46"),
                Color::White => Cow::Borrowed("47"),
                Color::Gray => Cow::Borrowed("100"),
                Color::Clear => Cow::Borrowed("49"),
                Color::Rgb(r, g, b) => Cow::Owned(format!("48;2;{r};{g};{b}")),
            });
        }

        if self.style.bold {
            codes.push(Cow::Borrowed("1"));
        }

        if self.style.dim {
            codes.push(Cow::Borrowed("2"));
        }

        if codes.is_empty() {
            write!(f, "{}", self.value)
        } else {
            const CLEAR: &str = "\x1b[0m";
            let code = format!("\x1b[{}m", codes.join(";"));
            let input = self.value.to_string();
            let restore = format!("{CLEAR}{code}");
            let result = input.replace(CLEAR, &restore);
            write!(f, "{code}{result}{CLEAR}")
        }
    }
}

/// Extension trait for adding color methods to any Display type
pub trait Colorize: Display + Clone {
    #[must_use]
    fn fg(&self, color: Color) -> Colored<Self> {
        Colored {
            value: self.clone(),
            style: Style {
                fg: Some(color),
                ..Default::default()
            },
            mode: ColorMode::Auto,
        }
    }

    #[must_use]
    fn bg(&self, color: Color) -> Colored<Self> {
        Colored {
            value: self.clone(),
            style: Style {
                bg: Some(color),
                ..Default::default()
            },
            mode: ColorMode::Auto,
        }
    }

    #[must_use]
    fn bold(&self) -> Colored<Self> {
        Colored {
            value: self.clone(),
            style: Style {
                bold: true,
                ..Default::default()
            },
            mode: ColorMode::Auto,
        }
    }

    #[must_use]
    fn dim(&self) -> Colored<Self> {
        Colored {
            value: self.clone(),
            style: Style {
                dim: true,
                ..Default::default()
            },
            mode: ColorMode::Auto,
        }
    }

    #[must_use]
    fn red(&self) -> Colored<Self> {
        self.fg(Color::Red)
    }

    #[must_use]
    fn green(&self) -> Colored<Self> {
        self.fg(Color::Green)
    }

    #[must_use]
    fn yellow(&self) -> Colored<Self> {
        self.fg(Color::Yellow)
    }

    #[must_use]
    fn blue(&self) -> Colored<Self> {
        self.fg(Color::Blue)
    }

    #[must_use]
    fn purple(&self) -> Colored<Self> {
        self.fg(Color::Purple)
    }

    #[must_use]
    fn cyan(&self) -> Colored<Self> {
        self.fg(Color::Cyan)
    }

    #[must_use]
    fn white(&self) -> Colored<Self> {
        self.fg(Color::White)
    }

    #[must_use]
    fn gray(&self) -> Colored<Self> {
        self.fg(Color::Gray)
    }

    #[must_use]
    fn black(&self) -> Colored<Self> {
        self.fg(Color::Black)
    }
}

impl<T: Display + Clone> Colorize for T {}

static COLOR_OVERRIDE: RwLock<ColorMode> = RwLock::new(ColorMode::Auto);

pub fn set_color_override(mode: ColorMode) {
    if let Ok(mut override_mode) = COLOR_OVERRIDE.write() {
        *override_mode = mode;
    }
}

fn effective_color_mode(mode: ColorMode) -> ColorMode {
    if let Ok(override_mode) = COLOR_OVERRIDE.read()
        && matches!(*override_mode, ColorMode::Never | ColorMode::Always)
    {
        *override_mode
    } else {
        mode
    }
}

impl<T> Colored<T> {
    #[must_use]
    pub fn mode(mut self, mode: ColorMode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.style.fg = Some(color);
        self
    }

    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.style.bg = Some(color);
        self
    }

    #[must_use]
    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }

    #[must_use]
    pub fn dim(mut self) -> Self {
        self.style.dim = true;
        self
    }

    #[must_use]
    pub fn red(self) -> Self {
        self.fg(Color::Red)
    }

    #[must_use]
    pub fn green(self) -> Self {
        self.fg(Color::Green)
    }

    #[must_use]
    pub fn yellow(self) -> Self {
        self.fg(Color::Yellow)
    }

    #[must_use]
    pub fn blue(self) -> Self {
        self.fg(Color::Blue)
    }

    #[must_use]
    pub fn purple(self) -> Self {
        self.fg(Color::Purple)
    }

    #[must_use]
    pub fn cyan(self) -> Self {
        self.fg(Color::Cyan)
    }

    #[must_use]
    pub fn white(self) -> Self {
        self.fg(Color::White)
    }

    #[must_use]
    pub fn gray(self) -> Self {
        self.fg(Color::Gray)
    }

    #[must_use]
    pub fn black(self) -> Self {
        self.fg(Color::Black)
    }
}

/// Checks if stdout and stderr are both capable of handling ANSI escape codes.
pub fn has_colors() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| is_terminal_impl(std::io::stdout()) && is_terminal_impl(std::io::stderr()))
}

/// Check if a specific stream supports colors.
pub fn supports_color<W: std::io::IsTerminal>(stream: W) -> bool {
    is_terminal_impl(stream)
}

/// Determine the appropriate color mode for a given stream.
///
/// This respects common environment variables:
/// - `NO_COLOR`: If set (to any value), colors are disabled
/// - `FORCE_COLOR`: If set to a non-zero value, colors are forced on
pub fn detect_color_mode<W: std::io::IsTerminal>(stream: W) -> ColorMode {
    if std::env::var("NO_COLOR").is_ok() {
        return ColorMode::Never;
    }

    if let Ok(force) = std::env::var("FORCE_COLOR")
        && force != "0"
        && !force.is_empty()
    {
        return ColorMode::Always;
    }

    if supports_color(stream) {
        ColorMode::Auto
    } else {
        ColorMode::Never
    }
}

/// Check if colors should be used based on the given mode and stream.
pub fn should_colorize<W: std::io::IsTerminal>(mode: ColorMode, stream: W) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => supports_color(stream),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn is_terminal_impl<W: std::io::IsTerminal>(stream: W) -> bool {
    let is_dumb = if let Ok(v) = std::env::var("TERM") {
        v.eq_ignore_ascii_case("dumb")
    } else {
        false
    };

    let no_color = std::env::var_os("NO_COLOR").is_some();
    if !terminal::enable_ansi_colors() {
        return false;
    }

    !is_dumb && !no_color && stream.is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_modes() {
        let text = "Hello";

        let never = text.red().bold().mode(ColorMode::Never).to_string();
        assert_eq!(never, "Hello");

        let always = text.red().bold().mode(ColorMode::Always).to_string();
        assert!(always.contains("\x1b["));
        assert!(always.contains("38"));
        assert!(always.contains('1'));
    }

    #[test]
    fn test_should_colorize() {
        assert!(should_colorize(ColorMode::Always, std::io::stdout()));
        assert!(!should_colorize(ColorMode::Never, std::io::stdout()));
    }

    #[test]
    fn test_color_stacking() {
        let inner = "world".blue().mode(ColorMode::Always).to_string();
        let outer = format!("hello {inner} foo!")
            .red()
            .mode(ColorMode::Always)
            .to_string();

        // Should be: red, "hello ", blue, "world", reset, red restored, " foo!", reset
        assert_eq!(
            outer,
            "\x1b[38;5;1mhello \x1b[38;5;4mworld\x1b[0m\x1b[38;5;1m foo!\x1b[0m"
        );
    }
}
