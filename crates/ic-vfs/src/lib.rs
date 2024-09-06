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

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_syntax::Span;

/// An ID of a file in the [`SourceMap`].
pub type FileId = Id<FileInfo>;

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

#[must_use]
#[derive(Debug)]
pub struct FileInfo {
    /// Absolute path of the file.
    // TODO: store filename elsewhere?
    pub path: PathBuf,

    /// If we imagine that our preprocessor creates a single file where
    /// all includes have been inlined, this span would represent the span of
    /// the expanded include in the amalgamation.
    pub span: Span,

    /// Contents of the file.
    pub source: Rc<str>,

    /// Populated if this file was included by another file, and if so, the
    /// span of the `#include` directive.
    pub included_from: Option<(FileId, Span)>,

    pub kind: Include,
}

#[derive(Debug, Default)]
pub struct SourceMap {
    sources: Arena<FileInfo>,
    files: BTreeMap<PathBuf, FileId>,
    builtin_count: usize,
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
        let path = std::path::absolute(path)?;
        let src = match self.files.entry(path) {
            Entry::Occupied(id) => {
                let id = *id.get();
                (id, self.source(id))
            }
            Entry::Vacant(v) => {
                let source = Rc::from(std::fs::read_to_string(v.key())?);
                let path = v.key().clone();
                let id = self.insert(path, source, Span::default(), kind);
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
        self.builtin_count += 1;
        self.insert(
            PathBuf::from(name),
            source,
            Span::default(),
            Include::Static,
        )
    }

    /// # Panics
    ///
    /// This may panic if the ID does not exist in the `SourceMap`. This can
    /// only happen if you have mixed up IDs between multiple `SourceMap`
    /// instances.
    pub fn file_info(&self, id: FileId) -> &FileInfo {
        self.sources.get(id).unwrap()
    }

    #[must_use]
    pub fn source(&self, id: FileId) -> Rc<str> {
        self.file_info(id).source.clone()
    }

    #[must_use]
    pub fn source_str(&self, id: FileId) -> &str {
        &self.file_info(id).source
    }

    // #[must_use]
    // pub fn source_of(&self, span: FileId) -> &str {
    //     &self.file_info(id).source[id]
    // }

    #[must_use]
    pub fn span_of_file(&self, id: FileId) -> Span {
        self.file_info(id).span
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
    pub fn line_span(&self, _id: FileId) -> Span {
        todo!()
    }

    fn insert(&mut self, path: PathBuf, source: Rc<str>, span: Span, kind: Include) -> FileId {
        let info = FileInfo {
            path: path.clone(),
            span,
            source,
            included_from: None,
            kind,
        };

        let id = self.sources.alloc(info);
        self.files.insert(path, id);
        id
    }
}
