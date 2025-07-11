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
use std::fs::DirEntry;
use std::io::{self, Result};
use std::path::{Path, PathBuf};

use ic_cli::color::Colorize;

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
        eprintln!("{} {}", "warning:".purple().bold(), format!($($arg)*));
    }}
}

#[derive(Debug)]
#[allow(unused)]
pub enum Error {
    Diagnostic(Box<ic_diagnostic::Diag>),
    Parse(Box<ic_parse::Error>),
    Preproc(ic_preproc::ProcError),
    Io(std::io::Error),
    Custom(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ic_diagnostic::Diag> for Error {
    fn from(value: ic_diagnostic::Diag) -> Self {
        Self::Diagnostic(Box::new(value))
    }
}

impl From<ic_parse::Error> for Error {
    fn from(value: ic_parse::Error) -> Self {
        Self::Parse(Box::new(value))
    }
}

/// Recursively iterates a directory and collects and IDL files.
pub fn collect_files<'a, I>(paths: I) -> std::result::Result<Vec<PathBuf>, Vec<io::Error>>
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    fn collect(p: &Path, files: &mut HashSet<PathBuf>) -> Result<()> {
        let meta = std::fs::metadata(p)?;
        if meta.is_dir() {
            let iter = std::fs::read_dir(p)?;
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

    let mut errors = vec![];
    let mut files = HashSet::new();
    for path in paths {
        if std::fs::metadata(path).is_ok_and(|v| v.is_dir()) {
            if let Err(e) = collect(path, &mut files) {
                errors.push(e);
            }
        } else {
            files.insert(path.clone());
        }
    }

    if errors.is_empty() {
        let mut files: Vec<_> = files.into_iter().collect();
        files.sort();
        Ok(files)
    } else {
        Err(errors)
    }
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

pub fn safe_purge<P>(dir: P) -> std::result::Result<(), Error>
where
    P: AsRef<Path>,
{
    const BLACKLIST: &[&str] = &[".git", ".hg"];

    let filter = |entry: DirEntry| {
        BLACKLIST
            .iter()
            .find(|v| entry.file_name().eq_ignore_ascii_case(v))
    };

    if let Ok(v) = std::fs::metadata(&dir) {
        if v.is_dir() {
            if let Some(deny) = std::fs::read_dir(&dir)?.flatten().find_map(filter) {
                Err(Error::Custom(format!(
                    "cowardly refusing to purge output directory that contains `{}`",
                    deny.yellow(),
                )))?;
            }
            std::fs::remove_dir_all(&dir)?;
        }
    }
    Ok(())
}
