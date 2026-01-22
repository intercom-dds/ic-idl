// Copyright 2026 KONGSBERG
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

#[cfg(unix)]
mod imp {
    use std::os::unix::io::AsRawFd;

    #[must_use]
    pub fn terminal_width() -> Option<u16> {
        // SAFETY: libc::winsize is a POD struct, zeroing is valid initialization
        let mut size: libc::winsize = unsafe { std::mem::zeroed() };
        let fd = std::io::stderr().as_raw_fd();

        // SAFETY: `ioctl` with `TIOCGWINSZ` is safe when passed a valid fd and
        // `Winsize` pointer
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut size) } == 0 && size.ws_col > 0 {
            Some(size.ws_col)
        } else {
            None
        }
    }

    #[must_use]
    pub fn enable_ansi_colors() -> bool {
        true
    }
}

#[cfg(windows)]
mod imp {
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
    use winapi::um::wincon::{
        CONSOLE_SCREEN_BUFFER_INFO, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode,
        GetConsoleScreenBufferInfo, SetConsoleMode,
    };

    #[must_use]
    pub fn terminal_width() -> Option<u16> {
        // SAFETY: GetStdHandle with STD_ERROR_HANDLE is always safe
        let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return None;
        }

        // SAFETY: CONSOLE_SCREEN_BUFFER_INFO is POD, zeroing is valid
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };

        // SAFETY: GetConsoleScreenBufferInfo is safe with valid handle
        if unsafe { GetConsoleScreenBufferInfo(handle, &mut info) } != 0 {
            let width = info.srWindow.Right - info.srWindow.Left + 1;
            if width > 0 {
                return Some(width as u16);
            }
        }
        None
    }

    #[must_use]
    pub fn enable_ansi_colors() -> bool {
        fn enable_for_handle(handle_id: u32) -> bool {
            // SAFETY: `GetStdHandle` is safe with standard handle constants
            let handle = unsafe { GetStdHandle(handle_id) };
            if handle == INVALID_HANDLE_VALUE || handle.is_null() {
                return false;
            }

            let mut mode: u32 = 0;

            // SAFETY: `GetConsoleMode` is safe with valid handle
            if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
                return false;
            }

            mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;

            // SAFETY: `SetConsoleMode` is safe with valid handle
            unsafe { SetConsoleMode(handle, mode) != 0 }
        }

        enable_for_handle(STD_OUTPUT_HANDLE) && enable_for_handle(STD_ERROR_HANDLE)
    }
}

#[cfg(not(any(unix, windows)))]
mod imp {
    #[must_use]
    pub fn terminal_width() -> Option<u16> {
        None
    }

    #[must_use]
    pub fn enable_ansi_colors() -> bool {
        true
    }
}

pub use imp::{enable_ansi_colors, terminal_width};
