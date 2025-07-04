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

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

pub use ic_lexer::token::{Kind, Token};
pub use ic_vfs::Span;
use ic_vfs::{FileId, Include, SourceMap};
pub use state::{Error, State};
pub use processor::TokenIter;

mod directives;
mod expression;
mod macros;
mod processor;
mod state;
mod time;

const RECURSION_DEPTH: usize = 200;

#[must_use]
#[derive(Clone, Debug)]
pub struct ProcArgs {
    include_dirs: HashSet<PathBuf>,
    defines: HashMap<String, Option<String>>,
    skip_comments: bool,
    recursion_depth: usize,
}

impl Default for ProcArgs {
    fn default() -> Self {
        Self {
            include_dirs: HashSet::default(),
            defines: HashMap::default(),
            skip_comments: false,
            recursion_depth: RECURSION_DEPTH,
        }
    }
}

impl ProcArgs {
    pub fn include<S>(mut self, dir: S) -> Self
    where
        S: Into<PathBuf>,
    {
        self.include_dirs.insert(dir.into());
        self
    }

    pub fn includes<I, S>(mut self, dirs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<PathBuf>,
    {
        self.include_dirs.extend(dirs.into_iter().map(Into::into));
        self
    }

    pub fn define<K>(mut self, arg: K, val: Option<String>) -> Self
    where
        K: Into<String>,
    {
        self.defines.insert(arg.into(), val);
        self
    }

    pub fn defines<I, K, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, Option<V>)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.defines
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v.map(Into::into))));
        self
    }

    pub fn recursion_depth(mut self, depth: usize) -> Self {
        self.recursion_depth = depth;
        self
    }

    pub fn skip_comments(mut self, strip: bool) -> Self {
        self.skip_comments = strip;
        self
    }

    #[must_use]
    pub fn get_skip_comments(&self) -> bool {
        self.skip_comments
    }
}

#[derive(Debug)]
pub enum ProcError {
    /// An error occurred when opening included files or writing to the
    /// provided output sink.
    Io(io::Error),

    /// The input contained invalid UTF-8 characters.
    Encoding(FromUtf8Error),

    /// The nested include depth was reached, likely due to recursive includes
    /// without header guards.
    DepthLimit(usize),

    /// An included file was not found.
    NotFound(String),
}

impl std::error::Error for ProcError {}

impl std::fmt::Display for ProcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcError::Io(e) => write!(f, "{}", e.to_string().to_lowercase()),
            ProcError::Encoding(e) => write!(f, "{e}"),
            ProcError::DepthLimit(v) => {
                write!(f, "`#include` recursion depth limit of {v} was reached")
            }
            ProcError::NotFound(e) => write!(f, "'{e}' file not found"),
        }
    }
}

/// Processes the specified file and returns an iterator of the preprocessed
/// tokens. Any macro definitions it encounters will be expanded in-place.
pub fn preprocess(file_id: FileId, args: ProcArgs, vfs: &mut SourceMap) -> TokenIter<'_, State> {
    let state = State::default();
    processor::preprocess(file_id, args, state, vfs)
}

/// Processes the specified file and returns an iterator of the preprocessed
/// tokens. Any macro definitions it encounters will be expanded in-place.
///
/// Unlike [`preprocess`], this mutably borrows a [`State`] and uses that as
/// the preprocessors internal state. This can be useful for effectively
/// re-using a preprocessor for subsequent runs without resetting or clearing
/// the state.
pub fn with_state<'a>(
    file_id: FileId,
    args: ProcArgs,
    state: &'a mut State,
    vfs: &'a mut SourceMap,
) -> TokenIter<'a, &'a mut State> {
    processor::preprocess(file_id, args, state, vfs)
}

/// Parses the given file and returns a string of the preprocessed contents.
///
/// # Errors
///
/// Returns an error if there was an error opening the specified `path`.
pub fn to_string<P>(path: P, args: ProcArgs) -> std::io::Result<(String, Vec<Error>)>
where
    P: AsRef<Path>,
{
    let mut vfs = SourceMap::default();
    let (file_id, _) = vfs.open(path, Include::Static)?;
    let mut state = State::new();
    Ok(processor::to_string(file_id, args, &mut state, &mut vfs))
}
