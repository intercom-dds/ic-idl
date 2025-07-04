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

use ic_lexer::token::Token;

use crate::Span;

/// Macro definition
#[derive(Debug, Clone)]
pub enum Macro {
    /// Function-like macro with parameters
    Function {
        span: Span,
        args: Vec<Token>,
        def: Vec<Token>,
        variadic: bool,
    },
    /// Object-like macro (simple replacement)
    Object { span: Span, def: Vec<Token> },
}

impl Macro {
    /// Get the span where this macro was defined
    pub fn span(&self) -> Span {
        match self {
            Macro::Function { span, .. } | Macro::Object { span, .. } => *span,
        }
    }

    /// Get the definition tokens
    pub fn definition(&self) -> &[Token] {
        match self {
            Macro::Function { def, .. } | Macro::Object { def, .. } => def,
        }
    }

    /// Check if this is a function-like macro
    pub fn is_function(&self) -> bool {
        matches!(self, Macro::Function { .. })
    }

    /// Check if this is a variadic macro
    pub fn is_variadic(&self) -> bool {
        match self {
            Macro::Function { variadic, .. } => *variadic,
            Macro::Object { .. } => false,
        }
    }

    /// Get the parameter list for function-like macros
    pub fn parameters(&self) -> Option<&[Token]> {
        match self {
            Macro::Function { args, .. } => Some(args),
            Macro::Object { .. } => None,
        }
    }
}
