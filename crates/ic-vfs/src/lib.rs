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

//! Virtual File System for the IDL compiler.
//!
//! This crate provides a source file management system that tracks all files
//! loaded by the compiler, maintains their contents in memory, and provides
//! span-based location tracking for error reporting.
//!
//! # Key Components
//!
//! - [`SourceMap`]: The main file registry that manages all loaded files
//! - [`FileId`]: Unique identifier for a file in the source map
//! - [`Span`]: A range within a file, used for error reporting
//! - [`Location`]: Line and column information for a position in a file
//!
//! # Example
//!
//! ```ignore
//! use ic_vfs::{SourceMap, Include};
//!
//! let mut source_map = SourceMap::default();
//! let (file_id, _) = source_map.open("example.idl", Include::Static)?;
//! let span = source_map.span_of(file_id);
//! let location = source_map.location(span.start());
//! ```

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{fmt, io};

use ic_alloc::arena::{Arena, Id};
use tracing::{debug, trace};

mod span;
pub use span::{Location, Span};

/// Unicode byte-order mark (U+FEFF).
pub const UTF8_BOM: char = '\u{feff}';

/// An ID of a file in the [`SourceMap`].
pub type FileId = Id<FileInfo>;

#[derive(Copy, Clone, Debug)]
pub struct InvalidUtf8 {
    pub span: Span,
    pub encoding: Encoding,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16,
    Utf32,
    Nul,
}

impl fmt::Display for InvalidUtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.encoding {
            Encoding::Utf8 => f.write_str("file contains invalid UTF-8"),
            Encoding::Utf16 => f.write_str("file is UTF-16 encoded; only UTF-8 is supported"),
            Encoding::Utf32 => f.write_str("file is UTF-32 encoded; only UTF-8 is supported"),
            Encoding::Nul => f.write_str("file contains NUL bytes; it may be UTF-16 encoded"),
        }
    }
}

impl std::error::Error for InvalidUtf8 {}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    InvalidUtf8(InvalidUtf8),
}

impl From<io::Error> for LoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::InvalidUtf8(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidUtf8(error) => Some(error),
        }
    }
}

/// The type of file inclusion, affecting how paths are resolved.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Include {
    /// System include, e.g. <foo.idl>
    System,

    /// Local include, e.g. "foo.idl"
    Local,

    /// A "static" file is one that was openened in other contexts than
    /// `#include` directories in the preprocessor, for example if the file was
    /// specified in the command-line interface of an application.
    Static,
}

/// Information about an `#include` directive.
///
/// This tracks which file was included, from where, and the span of the
/// path for diagnostic purposes.
#[derive(Clone, Debug)]
pub struct IncludeInfo {
    /// The span of the include path (e.g., `"foo.idl"` or `<foo.idl>`).
    pub path_span: Span,

    /// The path as written in the include directive (e.g., `foo.idl`).
    pub included_as: String,

    /// The `FileId` of the file that was included.
    pub included_file: FileId,

    /// The `FileId` of the file containing the `#include` directive.
    pub including_file: FileId,

    /// The kind of include (system vs local).
    pub kind: Include,
}

/// Information about a file loaded into the source map.
#[must_use]
#[derive(Debug)]
pub struct FileInfo {
    /// Absolute path of the file.
    pub path: PathBuf,

    /// The file name written exactly as it was first included.
    pub included_as: PathBuf,

    /// Contents of the file.
    pub source: Rc<str>,

    pub kind: Include,

    invalid_utf8: Option<InvalidUtf8>,
}

/// A registry of all source files loaded by the compiler.
///
/// The `SourceMap` maintains file contents in memory and provides utilities
/// for mapping between file positions, spans, and line/column locations.
/// It deduplicates files by their absolute path to avoid loading the same
/// file multiple times.
#[derive(Debug, Default)]
pub struct SourceMap {
    sources: Arena<FileInfo>,
    files: BTreeMap<PathBuf, FileId>,
    builtin_count: usize,
    /// Tracks all `#include` directives processed.
    includes: Vec<IncludeInfo>,
}

impl SourceMap {
    /// # Errors
    ///
    /// Returns an error if `path` does not exist, if we do not have permission
    /// to read the file, or if the contents of the file are not valid UTF-8.
    pub fn open<P: AsRef<Path>>(
        &mut self,
        path: P,
        kind: Include,
    ) -> Result<(FileId, Rc<str>), LoadError> {
        let abs = std::path::absolute(&path)?;
        let id = match self.files.entry(abs) {
            Entry::Occupied(id) => {
                trace!(path = %path.as_ref().display(), "file already loaded (cache hit)");
                *id.get()
            }
            Entry::Vacant(v) => {
                debug!(path = %v.key().display(), "opening file");
                let bytes = std::fs::read(v.key())?;
                if u32::try_from(bytes.len()).is_err() {
                    let error = io::Error::new(io::ErrorKind::InvalidData, "file exceeds 4 GiB");
                    return Err(error.into());
                }

                let (source, invalid_utf8) = match String::from_utf8(bytes) {
                    Ok(source) => (source, None),
                    Err(error) => {
                        let replacement = char::REPLACEMENT_CHARACTER.len_utf8();
                        let (encoding, start, end) = match error.as_bytes() {
                            [0xFF, 0xFE, 0, 0, ..] | [0, 0, 0xFE, 0xFF, ..] => {
                                (Encoding::Utf32, 0, 2 * replacement + 2)
                            }
                            [0xFF, 0xFE, ..] | [0xFE, 0xFF, ..] => {
                                (Encoding::Utf16, 0, 2 * replacement)
                            }
                            _ => {
                                let start = error.utf8_error().valid_up_to();
                                (Encoding::Utf8, start, start + replacement)
                            }
                        };

                        let source = String::from_utf8_lossy(error.as_bytes()).into_owned();
                        (source, Some((start, end, encoding)))
                    }
                };

                let nul = source
                    .find('\0')
                    .map(|start| (start, start + 1, Encoding::Nul));
                let invalid_utf8 = [invalid_utf8, nul]
                    .into_iter()
                    .flatten()
                    .min_by_key(|&(start, _, _)| start);

                let included_as = path.as_ref().to_path_buf();
                let path = v.key().clone();
                let id = self.insert(path, included_as, Rc::from(source), kind);

                if let Some((start, end, encoding)) = invalid_utf8 {
                    let span = Span {
                        start: Location::new(u32::try_from(start).unwrap_or(u32::MAX), id),
                        end: Location::new(u32::try_from(end).unwrap_or(u32::MAX), id),
                    };
                    self.sources[id].invalid_utf8 = Some(InvalidUtf8 { span, encoding });
                }

                id
            }
        };

        let info = self.file_info(id);
        match info.invalid_utf8 {
            Some(error) => Err(LoadError::InvalidUtf8(error)),
            None => Ok((id, info.source.clone())),
        }
    }

    /// Creates a virtual file that contains the given sources.
    pub fn embed(&mut self, src: &str) -> FileId {
        let name = format!("<builtin-{}", self.builtin_count);
        self.embed_with_name(&name, src)
    }

    /// Creates a virtual file that contains the given sources.
    pub fn embed_with_name(&mut self, name: &str, src: impl Into<Rc<str>>) -> FileId {
        let source = src.into();
        let name = PathBuf::from(name);
        self.builtin_count += 1;
        self.insert(name.clone(), name, source, Include::Static)
    }

    /// # Panics
    ///
    /// This may panic if the ID does not exist in the `SourceMap`. This can
    /// only happen if you have mixed up IDs between multiple `SourceMap`
    /// instances.
    pub fn file_info(&self, id: FileId) -> &FileInfo {
        self.sources.get(id)
    }

    #[must_use]
    pub fn included_as(&self, id: FileId) -> &Path {
        &self.file_info(id).included_as
    }

    #[must_use]
    pub fn source(&self, id: FileId) -> Rc<str> {
        self.file_info(id).source.clone()
    }

    #[must_use]
    pub fn source_str(&self, id: FileId) -> &str {
        &self.file_info(id).source
    }

    /// Returns the name of the specified file.
    #[must_use]
    pub fn name(&self, id: FileId) -> &Path {
        &self.file_info(id).path
    }

    /// Returns the absolute path of the specified file.
    #[must_use]
    pub fn path(&self, id: FileId) -> &Path {
        &self.file_info(id).path
    }

    #[must_use]
    pub fn files(&self) -> &BTreeMap<PathBuf, FileId> {
        &self.files
    }

    /// Records an `#include` directive.
    ///
    /// This is called by the preprocessor when it processes an `#include`
    /// directive, allowing lints to track which files were included and from
    /// where.
    pub fn record_include(&mut self, info: IncludeInfo) {
        trace!(
            included_as = %info.included_as,
            kind = ?info.kind,
            "recording include directive"
        );
        self.includes.push(info);
    }

    /// Returns all recorded `#include` directives.
    #[must_use]
    pub fn includes(&self) -> &[IncludeInfo] {
        &self.includes
    }

    fn insert(
        &mut self,
        path: PathBuf,
        included_as: PathBuf,
        source: Rc<str>,
        kind: Include,
    ) -> FileId {
        let source_len = source.len();
        let info = FileInfo {
            path: path.clone(),
            included_as,
            source,
            kind,
            invalid_utf8: None,
        };

        let id = self.sources.alloc(info);
        trace!(
            path = %path.display(),
            size = source_len,
            file_id = ?id,
            "registered file in source map",
        );
        self.files.insert(path, id);
        id
    }
}
