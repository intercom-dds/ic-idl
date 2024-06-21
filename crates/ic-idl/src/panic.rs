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

use std::panic::PanicInfo;
use std::{backtrace, panic};

use ic_cli::color::Colorize;

fn dump_backtrace(info: &PanicInfo) {
    let thread = std::thread::current();
    let thread = thread.name().unwrap_or("unknown");
    let trace = backtrace::Backtrace::force_capture();

    let msg = match info.payload().downcast_ref::<&str>() {
        Some(s) => *s,
        None => info
            .payload()
            .downcast_ref::<String>()
            .map_or("<null>", |s| &**s),
    };

    eprint!("{} ", "error:".red().bold());
    match info.location() {
        Some(loc) => {
            eprintln!(
                "thread '{thread}' panicked at '{msg}', {}:{}",
                loc.file(),
                loc.line(),
            );
        }
        None => {
            eprintln!("thread '{thread}' panicked at '{msg}'");
        }
    }

    if trace.status() == backtrace::BacktraceStatus::Captured {
        eprintln!("{trace:#?}");
    }
    eprintln!("This is a compiler bug. Please report it to the InterCOM DDS team.");
}

pub fn install_hook() {
    // Dump the backtrace if a thread panics
    panic::set_hook(Box::new(dump_backtrace));
}
