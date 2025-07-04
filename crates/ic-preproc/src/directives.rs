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

use ic_lexer::token::{Kind, Token};
use ic_vfs::Include;

use crate::state::{Directive, Error};
use crate::macros::Macro;
use crate::Span;

/// A small state machine for keeping track of the current state of `if`
/// statements and their expressions.
#[derive(Debug)]
pub struct IfState {
    pub state: IfKind,
    pub evaluated: bool,
    pub defined: Span,
}

#[derive(Debug)]
pub enum IfKind {
    If { result: bool },
    Elif { result: bool },
    Else,
}

impl IfState {
    pub fn new_if(result: bool, defined: Span) -> Self {
        Self {
            state: IfKind::If { result },
            evaluated: false,
            defined,
        }
    }

    pub fn eval_elif(&mut self, result: bool) -> Result<(), Error> {
        let was_true = match self.state {
            IfKind::If { result } | IfKind::Elif { result } => result,
            IfKind::Else => {
                self.evaluated = true;
                Err(Error::Expr {
                    message: "#elif after #else",
                })?
            }
        };

        self.state = IfKind::Elif { result };
        self.evaluated = self.evaluated || was_true;
        Ok(())
    }

    pub fn eval_else(&mut self) -> Result<(), Error> {
        let was_true = match self.state {
            IfKind::If { result } | IfKind::Elif { result } => result,
            IfKind::Else => {
                self.evaluated = true;
                Err(Error::Expr {
                    message: "#else after #else",
                })?
            }
        };

        self.state = IfKind::Else;
        self.evaluated = self.evaluated || was_true;
        Ok(())
    }

    /// Check if this `if` state is "active", that is if the current
    /// processor should be emitting tokens.
    pub fn is_active(&self) -> bool {
        match self.state {
            IfKind::If { result } => result,
            IfKind::Elif { result } => !self.evaluated && result,
            IfKind::Else => !self.evaluated,
        }
    }
}

/// Trait for directive handlers
pub trait DirectiveHandler {
    /// Handle #include directive
    fn dir_include(&mut self, span: Span);
    
    /// Handle #define directive
    fn dir_define(&mut self) -> Option<()>;
    
    /// Handle #undef directive
    fn dir_undef(&mut self);
    
    /// Handle #if directive
    fn dir_if(&mut self, span: Span);
    
    /// Handle #ifdef directive
    fn dir_ifdef(&mut self, span: Span);
    
    /// Handle #ifndef directive
    fn dir_ifndef(&mut self, span: Span);
    
    /// Handle #elif directive
    fn dir_elif(&mut self, span: Span);
    
    /// Handle #else directive
    fn dir_else(&mut self, span: Span);
    
    /// Handle #endif directive
    fn dir_endif(&mut self, span: Span);
    
    /// Handle #pragma directive
    fn dir_pragma(&mut self, span: Span);
    
    /// Handle #error directive
    fn dir_error(&mut self, span: Span);
    
    /// Handle #warning directive
    fn dir_warning(&mut self, span: Span);
    
    /// Handle #line directive
    fn dir_line(&mut self, span: Span);
}