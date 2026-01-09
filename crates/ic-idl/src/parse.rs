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

use std::collections::HashMap;
use std::path::Path;

use ic_preproc::{ExpansionInfo, ProcArgs};
use ic_syntax::{AnnotationAppl, Item, Span};
use ic_vfs::{FileId, Include, SourceMap};
use tracing::{debug, debug_span};

use crate::util::Error;

/// Result of parsing an IDL file with preprocessing.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ParseResult {
    pub tree: Vec<Item>,
    pub errors: Vec<Error>,
    pub orphaned_annotations: Vec<AnnotationAppl>,
    pub preproc_warnings: Vec<ic_preproc::Error>,
    pub expansion_info: HashMap<Span, ExpansionInfo>,
}

/// Parse a file from a path with preprocessing.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be opened.
pub fn from_path(path: &Path, args: ProcArgs, vfs: &mut SourceMap) -> std::io::Result<ParseResult> {
    let (file_id, _) = vfs.open(path, Include::Static)?;
    Ok(from_file(file_id, args, vfs))
}

/// Parse a file that's already in the source map with preprocessing.
#[must_use]
pub fn from_file(file_id: FileId, args: ProcArgs, vfs: &mut SourceMap) -> ParseResult {
    let _span = debug_span!("parse_file", ?file_id).entered();
    let mut state = ic_preproc::State::new();
    let tokens: Vec<_> = ic_preproc::with_state(file_id, args, &mut state, vfs).collect();
    let parsed = ic_parse::from_iter(tokens, vfs);
    debug!(
        items = parsed.tree.len(),
        errors = parsed.errors.len(),
        "parsed",
    );

    let mut errors: Vec<Error> = parsed.errors.into_iter().map(Into::into).collect();
    errors.extend(state.errors().iter().cloned().map(Into::into));

    ParseResult {
        tree: parsed.tree,
        errors,
        orphaned_annotations: parsed.orphaned_annotations,
        preproc_warnings: state.warnings().to_vec(),
        expansion_info: state.expansion_info.into_iter().collect(),
    }
}

/// Run preprocessor only and return the preprocessed source.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be opened.
#[allow(dead_code, clippy::print_stderr)]
pub fn preprocess_only(path: &Path, args: ProcArgs) -> std::io::Result<String> {
    let (output, errors) = ic_preproc::to_string(path, args)?;
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{error:?}");
        }
        std::process::exit(1);
    }

    Ok(output)
}
