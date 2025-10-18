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

use ic_cli::color::Colorize as _;

const COMMIT_HASH: &str = concat!("#", env!("COMMIT_HASH"));

const COMMIT_DATE: &str = env!("COMMIT_DATE");

const BUILD_TARGET: &str = env!("BUILD_TARGET");

const BUILD_PROFILE: &str = env!("BUILD_PROFILE");

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Version {
    install_dir: String,
}

fn exe_dir() -> String {
    fn inner() -> Option<String> {
        std::env::current_exe()
            .ok()?
            .parent()?
            .to_str()
            .map(ToString::to_string)
    }
    inner().unwrap_or_else(|| "unknown".to_string())
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {PKG_VERSION} ({} {COMMIT_DATE})",
            PKG_NAME.green(),
            COMMIT_HASH.yellow(),
        )?;
        writeln!(f, "target: {BUILD_TARGET}")?;
        writeln!(f, "install: {}", self.install_dir)?;
        write!(f, "build type: {BUILD_PROFILE}")
    }
}

pub fn version() -> Version {
    Version {
        install_dir: exe_dir(),
    }
}
