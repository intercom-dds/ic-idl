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

//! Validation for inheritance relationships in HIR.

use ic_diagnostic::{Diag, Label, error_span};
use ic_syntax::Path;

use crate::Context;
use crate::hir::{DefFlags, DefId, DefKind};

/// Validates parent-child relationships in inheritance.
pub struct ParentValidator<'a> {
    ctx: &'a Context,
    errors: &'a mut Vec<Diag>,
}

impl<'a> ParentValidator<'a> {
    /// Creates a new parent validator.
    pub fn new(ctx: &'a Context, errors: &'a mut Vec<Diag>) -> Self {
        Self { ctx, errors }
    }

    /// Validates that a parent is suitable for struct inheritance.
    pub fn validate_struct_parent(
        &mut self,
        parent_id: DefId,
        child_name: &str,
        child_span: ic_syntax::Span,
    ) -> Option<DefId> {
        let parent_def = self.ctx.definitions.get(parent_id);

        // Check if parent is incomplete (forward declaration)
        if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
            self.errors.push(
                error_span(
                    format!(
                        "struct `{}` cannot inherit from incomplete type `{}`",
                        child_name, parent_def.ident.name
                    ),
                    Label::new(child_span).message("inherits from incomplete type here"),
                )
                .label(
                    Label::new(parent_def.ident.span)
                        .message("parent type is only forward declared"),
                ),
            );
            return None;
        }

        // Check if parent is a struct
        match &parent_def.kind {
            DefKind::Struct(_) => Some(parent_id),
            DefKind::Decl(_) => {
                // This shouldn't happen if IS_INCOMPLETE flag is set correctly
                self.errors.push(
                    error_span(
                        format!(
                            "struct `{}` cannot inherit from forward declaration `{}`",
                            child_name, parent_def.ident.name
                        ),
                        Label::new(child_span).message("inherits here"),
                    )
                    .label(Label::new(parent_def.ident.span).message("forward declaration here")),
                );
                None
            }
            _ => {
                self.errors.push(
                    error_span(
                        format!(
                            "struct `{}` inherits from non-struct type `{}`",
                            child_name, parent_def.ident.name
                        ),
                        Label::new(child_span).message("inherits here"),
                    )
                    .label(
                        Label::new(parent_def.ident.span)
                            .message(format!("{} defined here", parent_def.kind.kind_name())),
                    ),
                );
                None
            }
        }
    }

    /// Validates parent interfaces for interface inheritance.
    pub fn validate_interface_parents(
        &mut self,
        parent_ids: &[DefId],
        child_name: &str,
        child_span: ic_syntax::Span,
    ) -> Vec<DefId> {
        let mut valid_parents = Vec::new();

        for &parent_id in parent_ids {
            let parent_def = self.ctx.definitions.get(parent_id);

            // Check if parent is incomplete
            if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                self.errors.push(
                    error_span(
                        format!(
                            "interface `{}` cannot inherit from incomplete type `{}`",
                            child_name, parent_def.ident.name
                        ),
                        Label::new(child_span).message("inherits from incomplete type here"),
                    )
                    .label(
                        Label::new(parent_def.ident.span)
                            .message("parent type is only forward declared"),
                    ),
                );
                continue;
            }

            // Check if parent is an interface
            match &parent_def.kind {
                DefKind::Interface(_) => {
                    valid_parents.push(parent_id);
                }
                _ => {
                    self.errors.push(
                        error_span(
                            format!(
                                "interface `{}` inherits from non-interface type `{}`",
                                child_name, parent_def.ident.name
                            ),
                            Label::new(child_span).message("inherits here"),
                        )
                        .label(
                            Label::new(parent_def.ident.span)
                                .message(format!("{} defined here", parent_def.kind.kind_name())),
                        ),
                    );
                }
            }
        }

        valid_parents
    }

    /// Reports an error for undefined parent type.
    pub fn report_undefined_parent(
        &mut self,
        child_type: &str,
        child_name: &str,
        parent_path: &Path,
    ) {
        self.errors.push(error_span(
            format!(
                "{} `{}` inherits from type that is not defined",
                child_type, child_name
            ),
            Label::new(ic_syntax::util::path_span(parent_path)).message("undefined type"),
        ));
    }
}
