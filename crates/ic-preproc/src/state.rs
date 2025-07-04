// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

use std::collections::{HashMap, HashSet, VecDeque};

use ic_lexer::token::Token;
use ic_vfs::FileId;

use crate::Span;
use crate::macros::Macro;

/// Preprocessor state containing macro definitions, error state, and token queue
#[derive(Debug, Default)]
pub struct State {
    /// Defined macros
    pub defines: HashMap<String, Macro>,
    /// List of errors encountered during preprocessing
    pub errors: Vec<Error>,
    /// List of warnings encountered during preprocessing
    pub warnings: Vec<Error>,
    /// Queue of tokens to be emitted
    pub queue: VecDeque<Token>,
    /// Set of files we've already parsed.
    /// Used to enable `#pragma once`-like functionality.
    pub parsed_files: HashSet<FileId>,
}

impl State {
    /// Create a new preprocessor state
    #[must_use]
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            queue: VecDeque::new(),
            parsed_files: HashSet::new(),
        }
    }

    /// Check if a macro is defined
    #[must_use]
    pub fn is_defined(&self, name: &str) -> bool {
        self.defines.contains_key(name)
    }

    /// Get a macro definition
    #[must_use]
    pub fn get_macro(&self, name: &str) -> Option<&Macro> {
        self.defines.get(name)
    }

    /// Define a new macro
    pub fn define(&mut self, name: String, macro_def: Macro) {
        self.defines.insert(name, macro_def);
    }

    /// Undefine a macro
    pub fn undefine(&mut self, name: &str) {
        self.defines.remove(name);
    }

    /// Get the list of errors
    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Get the list of warnings
    #[must_use]
    pub fn warnings(&self) -> &[Error] {
        &self.warnings
    }

    /// Add an error
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Mark a file as parsed (for pragma once)
    pub fn mark_parsed(&mut self, file_id: FileId) {
        self.parsed_files.insert(file_id);
    }

    /// Check if a file has been parsed
    #[must_use]
    pub fn is_parsed(&self, file_id: FileId) -> bool {
        self.parsed_files.contains(&file_id)
    }
}

/// Directive types
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Directive {
    If,
    Ifdef,
    Ifndef,
    Elif,
    Else,
    Endif,
    Include,
    Define,
    Undef,
    Line,
    Warning,
    Error,
    Pragma,
}

/// Error types for preprocessing
#[derive(Clone, Debug)]
pub enum Error {
    Note {
        span: Span,
        tokens: Vec<Token>,
    },
    Extraneous {
        directive: Directive,
        span: Span,
        tokens: Vec<Token>,
    },
    Syntax {
        message: &'static str,
        span: Span,
    },
    // TODO: this should be Error::Syntax, but we don't currently record
    // spans of expression
    Expr {
        message: &'static str,
    },
}
