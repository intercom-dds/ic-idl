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

use std::fmt::Display;
use std::sync::{OnceLock, RwLock};

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

        let mut codes = Vec::new();

        if let Some(color) = self.style.fg {
            codes.push(match color {
                Color::Black => "30",
                Color::Red => "31",
                Color::Green => "32",
                Color::Yellow => "33",
                Color::Blue => "34",
                Color::Purple => "35",
                Color::Cyan => "36",
                Color::White => "37",
                Color::Gray => "90",
                Color::Clear => "0",
            });
        }

        if let Some(color) = self.style.bg {
            codes.push(match color {
                Color::Black => "40",
                Color::Red => "41",
                Color::Green => "42",
                Color::Yellow => "43",
                Color::Blue => "44",
                Color::Purple => "45",
                Color::Cyan => "46",
                Color::White => "47",
                Color::Gray => "100",
                Color::Clear => "49",
            });
        }

        if self.style.bold {
            codes.push("1");
        }

        if codes.is_empty() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "\x1b[{}m{}\x1b[0m", codes.join(";"), self.value)
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
    // Check NO_COLOR first (it takes precedence)
    if std::env::var("NO_COLOR").is_ok() {
        return ColorMode::Never;
    }

    // Check FORCE_COLOR
    if let Ok(force) = std::env::var("FORCE_COLOR") {
        if force != "0" && !force.is_empty() {
            return ColorMode::Always;
        }
    }

    // Otherwise, auto-detect
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

#[cfg(windows)]
fn virtual_term() -> bool {
    unsafe extern "C" {
        fn GetStdHandle(handle: u32) -> isize;
        fn GetConsoleMode(handle: isize, lp_mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: isize, dw_mode: u32) -> i32;
    }

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 4;
    const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4;
    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;

    let enable_virt = |handle| unsafe {
        let handle = GetStdHandle(handle);
        if handle == 0 || handle == -1 {
            return false;
        }

        let mut dw_mode: u32 = 0;
        GetConsoleMode(handle, std::ptr::addr_of_mut!(dw_mode));
        dw_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        SetConsoleMode(handle, dw_mode) != 0
    };
    enable_virt(STD_OUTPUT_HANDLE) && enable_virt(STD_ERROR_HANDLE)
}

#[allow(clippy::needless_pass_by_value)]
fn is_terminal_impl<W: std::io::IsTerminal>(stream: W) -> bool {
    let is_dumb = if let Ok(v) = std::env::var("TERM") {
        v == "dumb"
    } else {
        false
    };

    #[cfg(windows)]
    if !virtual_term() {
        return false;
    }

    !is_dumb && stream.is_terminal()
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
        assert!(always.contains("31"));
        assert!(always.contains('1'));
    }

    #[test]
    fn test_should_colorize() {
        // Test explicit modes
        assert!(should_colorize(ColorMode::Always, std::io::stdout()));
        assert!(!should_colorize(ColorMode::Never, std::io::stdout()));
        // Auto mode result depends on terminal detection
    }
}
