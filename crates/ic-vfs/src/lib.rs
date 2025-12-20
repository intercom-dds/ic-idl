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
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};

mod span;
pub use span::{Location, Span};

/// An ID of a file in the [`SourceMap`].
pub type FileId = Id<FileInfo>;

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
    ) -> io::Result<(FileId, Rc<str>)> {
        let abs = std::path::absolute(&path)?;
        let src = match self.files.entry(abs) {
            Entry::Occupied(id) => {
                let id = *id.get();
                (id, self.source(id))
            }
            Entry::Vacant(v) => {
                let source = Rc::from(std::fs::read_to_string(v.key())?);
                let included_as = path.as_ref().to_path_buf();
                let path = v.key().clone();
                let id = self.insert(path, included_as, source, kind);
                (id, self.source(id))
            }
        };
        Ok(src)
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
        let info = FileInfo {
            path: path.clone(),
            included_as,
            source,
            kind,
        };

        let id = self.sources.alloc(info);
        self.files.insert(path, id);
        id
    }
}
