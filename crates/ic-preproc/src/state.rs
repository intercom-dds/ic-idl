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

use std::collections::VecDeque;
use std::rc::Rc;

use ic_lexer::token::Token;
use ic_vfs::FileId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::Span;
use crate::macros::Macro;

/// Information about where a token was expanded from
#[derive(Debug, Clone)]
pub struct ExpansionInfo {
    /// The span where the macro was invoked
    pub invocation_span: Span,
    /// The name of the macro that was expanded
    pub macro_name: Rc<str>,
}

/// Preprocessor state containing macro definitions, error state, and token queue
#[derive(Debug, Default)]
pub struct State {
    /// Defined macros
    pub defines: FxHashMap<Rc<str>, Rc<Macro>>,

    /// List of errors encountered during preprocessing
    pub errors: Vec<Error>,

    /// List of warnings encountered during preprocessing
    pub warnings: Vec<Error>,

    /// Queue of tokens to be emitted
    pub queue: VecDeque<Token>,

    /// Set of files we've already parsed.
    /// Used to enable `#pragma once`-like functionality.
    pub parsed_files: FxHashSet<FileId>,

    /// Map from token spans to their macro expansion context
    pub expansion_info: FxHashMap<Span, ExpansionInfo>,
}

impl State {
    /// Create a new preprocessor state
    #[must_use]
    pub fn new() -> Self {
        Self {
            defines: FxHashMap::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
            queue: VecDeque::new(),
            parsed_files: FxHashSet::default(),
            expansion_info: FxHashMap::default(),
        }
    }

    /// Check if a macro is defined
    #[must_use]
    pub fn is_defined(&self, name: &str) -> bool {
        self.defines.contains_key(name)
    }

    /// Get a macro definition
    #[must_use]
    pub fn get_macro(&self, name: &str) -> Option<&Rc<Macro>> {
        self.defines.get(name)
    }

    /// Get a macro definition along with its name (to reuse the Rc<str>)
    #[must_use]
    pub fn get_macro_with_name(&self, name: &str) -> Option<(Rc<str>, Rc<Macro>)> {
        self.defines
            .get_key_value(name)
            .map(|(k, v)| (Rc::clone(k), Rc::clone(v)))
    }

    /// Define a new macro
    pub fn define(&mut self, name: Rc<str>, macro_def: Rc<Macro>) {
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

impl std::fmt::Display for Directive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Directive::If => "if",
            Directive::Ifdef => "ifdef",
            Directive::Ifndef => "ifndef",
            Directive::Elif => "elif",
            Directive::Else => "else",
            Directive::Endif => "endif",
            Directive::Include => "include",
            Directive::Define => "define",
            Directive::Undef => "undef",
            Directive::Line => "line",
            Directive::Warning => "warning",
            Directive::Error => "error",
            Directive::Pragma => "pragma",
        };
        write!(f, "{str}")
    }
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
    Expr {
        message: &'static str,
        span: Span,
    },
}

impl Error {
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Note { span, .. }
            | Self::Extraneous { span, .. }
            | Self::Syntax { span, .. }
            | Self::Expr { span, .. } => *span,
        }
    }
}
