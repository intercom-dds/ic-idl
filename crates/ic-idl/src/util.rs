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
use std::io::{self, Result};
use std::path::{Path, PathBuf};

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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Diagnostic(e) => e.fmt(f),
            Error::Parse(e) => e.fmt(f),
            Error::Preproc(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
            Error::Custom(e) => e.fmt(f),
        }
    }
}

/// Recursively iterates a directory and collects and IDL files.
///
/// # Errors
///
/// Returns a vector of I/O errors if any files or directories cannot be read.
pub fn collect_files<'a, I>(paths: I) -> std::result::Result<Vec<PathBuf>, io::Error>
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

    let mut files = HashSet::new();
    for path in paths {
        if std::fs::metadata(path).is_ok_and(|v| v.is_dir()) {
            collect(path, &mut files)?;
        } else {
            files.insert(path.clone());
        }
    }

    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();
    Ok(files)
}

/// Write contents to a file only if it has changed.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be read or written.
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

/// Safely remove a directory and all its contents.
///
/// This function refuses to delete directories containing version control
/// files like `.git` or `.hg` to prevent accidental data loss.
///
/// # Errors
///
/// Returns an error if:
/// - The directory contains blacklisted files (`.git`, `.hg`)
/// - An I/O error occurs while reading or removing the directory
pub fn safe_purge<P>(dir: P) -> std::result::Result<(), Error>
where
    P: AsRef<Path>,
{
    const BLACKLIST: &[&str] = &[".git", ".hg"];

    if let Ok(v) = std::fs::metadata(&dir) {
        if v.is_dir() {
            for entry in std::fs::read_dir(&dir)?.flatten() {
                let file_name = entry.file_name();
                if BLACKLIST.iter().any(|v| file_name.eq_ignore_ascii_case(v)) {
                    return Err(Error::Custom(format!(
                        "cowardly refusing to purge output directory that contains `{}`",
                        file_name.to_string_lossy(),
                    )));
                }
            }
            std::fs::remove_dir_all(&dir)?;
        }
    }
    Ok(())
}
