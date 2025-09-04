// Copyright 2025 KONGSBERG
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

//! Builder pattern for creating HIR definitions.

use ic_alloc::arena::Id;
use ic_syntax::{Ident, Span};

use crate::hir::{Ann, Def, DefFlags, DefId, DefKind};

/// Builder for creating HIR definitions with a fluent interface.
pub struct DefBuilder {
    id: DefId,
    ident: Ident,
    parent: Option<DefId>,
    annotations: Vec<Ann>,
    span: Span,
    kind: Option<DefKind>,
    flags: DefFlags,
}

impl DefBuilder {
    /// Creates a new definition builder.
    pub fn new(ident: Ident) -> Self {
        Self {
            id: Id::_do_not_use(), // Will be set by registry
            ident,
            parent: None,
            annotations: Vec::new(),
            span: Span::default(),
            kind: None,
            flags: DefFlags::default(),
        }
    }

    /// Sets the span.
    pub fn span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Sets the parent definition ID.
    pub fn parent(mut self, parent: Option<DefId>) -> Self {
        self.parent = parent;
        self
    }

    /// Sets the annotations.
    pub fn annotations(mut self, annotations: Vec<Ann>) -> Self {
        self.annotations = annotations;
        self
    }

    /// Sets the definition kind.
    pub fn kind(mut self, kind: DefKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Adds a flag to the definition.
    pub fn flag(mut self, flag: DefFlags) -> Self {
        self.flags |= flag;
        self
    }

    /// Sets all flags for the definition.
    pub fn flags(mut self, flags: DefFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Marks the definition as incomplete (forward declaration).
    pub fn incomplete(mut self) -> Self {
        self.flags |= DefFlags::IS_INCOMPLETE;
        self
    }

    /// Marks the definition as builtin.
    pub fn builtin(mut self) -> Self {
        self.flags |= DefFlags::IS_BUILTIN;
        self
    }

    /// Builds the definition.
    ///
    /// # Panics
    /// Panics if no kind was set.
    pub fn build(self) -> Def {
        Def {
            id: self.id,
            ident: self.ident,
            parent: self.parent,
            annotations: self.annotations,
            span: self.span,
            kind: self.kind.expect("Definition kind must be set"),
            flags: self.flags,
        }
    }

    /// Builds the definition with a specific ID.
    pub fn build_with_id(mut self, id: DefId) -> Def {
        self.id = id;
        self.build()
    }
}
