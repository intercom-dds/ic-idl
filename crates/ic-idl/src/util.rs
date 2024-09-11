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

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("ic-idl: {} {}", "error:".red().bold(), format!($($arg)*));
    }}
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use ic_cli::color::Colorize as _;
        eprintln!("{} {}", "warning:".yellow().bold(), format!($($arg)*));
    }}
}

/// Recursively iterates a directory and collects and IDL files.
pub fn collect_files<'a, I>(paths: I) -> Result<HashSet<PathBuf>>
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    fn collect(p: &Path, files: &mut HashSet<PathBuf>) -> Result<()> {
        let meta = std::fs::metadata(p)?;
        if meta.is_dir() {
            let iter = match std::fs::read_dir(p) {
                Ok(v) => v,
                Err(e) => bail!("couldn't open {}: {e}", p.display()),
            };

            for file in iter.flatten() {
                collect(&file.path(), files)?;
            }
        } else if let Some(ext) = p.extension() {
            if ext.eq_ignore_ascii_case("idl") {
                files.insert(p.to_owned());
            }
        }
        Ok(())
    }

    let mut files = HashSet::new();
    for path in paths {
        if std::fs::metadata(path).map_or(false, |v| v.is_dir()) {
            collect(path, &mut files)?;
        } else {
            files.insert(path.clone());
        }
    }
    Ok(files)
}

pub fn write_if_changed<P>(path: P, contents: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let changed = std::fs::read_to_string(path).map_or(true, |v| v != contents);
    if changed {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(path, contents)?;
    }
    Ok(())
}
