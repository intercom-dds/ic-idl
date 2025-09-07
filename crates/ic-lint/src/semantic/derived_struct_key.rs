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

use ic_diagnostic::{Label, error_span};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, StructTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Enforces that derived structs (structs with inheritance) may not define @key fields
pub struct DerivedStructKey<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DerivedStructKey<'a> {
    fn name() -> &'static str {
        "derived-struct-key"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "prevents derived structs from defining @key fields"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DerivedStructKey { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for DerivedStructKey<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, struct_ty: &'a StructTy) {
        // Only check if the struct has a parent (is derived)
        if struct_ty.parent.is_some() {
            // Check each member for @key annotation
            for member in &struct_ty.members {
                for ann in &member.annotations {
                    if ann.ident.name == "key" {
                        Self::report(
                            self.ctx,
                            error_span(
                                format!(
                                    "derived struct '{}' cannot define @key fields",
                                    def.ident.name
                                ),
                                Label::new(ann.ident.span)
                                    .message("@key not allowed in derived struct"),
                            )
                            .note("only base structs can define @key fields")
                            .help("move @key fields to the base struct or remove inheritance"),
                        );
                    }
                }
            }
        }

        // Continue visiting
        ic_hir::visit::walk_struct(self, struct_ty);
    }
}
