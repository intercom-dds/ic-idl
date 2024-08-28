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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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
use std::sync::OnceLock;

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

pub trait Colorize: Display {
    fn fg(&self, color: Color) -> String {
        let c = match color {
            Color::Black => "30m",
            Color::Red => "31m",
            Color::Green => "32m",
            Color::Yellow => "33m",
            Color::Blue => "34m",
            Color::Purple => "35m",
            Color::Cyan => "36m",
            Color::White => "37m",
            Color::Gray => "90m",
            Color::Clear => "0m",
        };
        fmt_ansi(c, self)
    }

    fn bg(&self, color: Color) -> String {
        let c = match color {
            Color::Black => "40m",
            Color::Red => "41m",
            Color::Green => "42m",
            Color::Yellow => "43m",
            Color::Blue => "44m",
            Color::Purple => "45m",
            Color::Cyan => "46m",
            Color::White => "47m",
            Color::Gray => "100m",
            Color::Clear => "49m",
        };
        fmt_ansi(c, self)
    }

    fn black(&self) -> String {
        self.fg(Color::Black)
    }

    fn red(&self) -> String {
        self.fg(Color::Red)
    }

    fn green(&self) -> String {
        self.fg(Color::Green)
    }

    fn yellow(&self) -> String {
        self.fg(Color::Yellow)
    }

    fn blue(&self) -> String {
        self.fg(Color::Blue)
    }

    fn purple(&self) -> String {
        self.fg(Color::Purple)
    }

    fn cyan(&self) -> String {
        self.fg(Color::Cyan)
    }

    fn white(&self) -> String {
        self.fg(Color::White)
    }

    fn gray(&self) -> String {
        self.fg(Color::Gray)
    }

    fn bold(&self) -> String {
        fmt_ansi("1m", self)
    }

    fn clear(&self) -> String {
        self.fg(Color::Clear)
    }
}

impl<T: Display> Colorize for T {}

fn fmt_ansi<T: Display>(code: &str, input: T) -> String {
    if has_colors() {
        format!("\x1b[{code}{input}\x1b[0m")
    } else {
        input.to_string()
    }
}

/// Checks if stdout and stderr are both capable of handling ANSI escape codes.
pub fn has_colors() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(is_terminal)
}

#[cfg(windows)]
fn virtual_term() -> bool {
    extern "C" {
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

fn is_terminal() -> bool {
    use std::io::{self, IsTerminal};

    let is_dumb = if let Ok(v) = std::env::var("TERM") {
        v == "dumb"
    } else {
        false
    };

    #[cfg(windows)]
    if !virtual_term() {
        return false;
    }

    !is_dumb && io::stdin().is_terminal() && io::stdout().is_terminal()
}
