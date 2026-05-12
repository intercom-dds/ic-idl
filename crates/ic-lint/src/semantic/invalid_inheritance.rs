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

use ic_diagnostic::{Color, Diag, Label};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{self, DefKind};
use ic_hir::visit::walk_tree;

use crate::{Category, Lint, LintCtx};

pub struct InvalidInheritance<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for InvalidInheritance<'a> {
    fn name() -> &'static str {
        "invalid-inheritance"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Validates that type inheritance follows IDL rules"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}

impl<'a> ic_hir::visit::Visitor<'a> for InvalidInheritance<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a hir::Def, data: &'a hir::StructTy) {
        if let Some(parent) = data.parent {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if !matches!(&parent_def.kind, DefKind::Struct(_)) {
                self.ctx.report(
                    InvalidInheritance::name(),
                    InvalidInheritance::category(),
                    Diag::error(format!(
                        "struct `{}` cannot inherit from {} `{}`",
                        def.ident.name,
                        parent_def.kind.kind_name(),
                        parent_def.ident.name
                    ))
                    .label(
                        Label::new(def.ident.span)
                            .message("structs can only inherit from other structs")
                            .color(Color::Red),
                    ),
                );
            }
        }
    }

    fn visit_interface(&mut self, def: &'a hir::Def, data: &'a hir::InterfaceTy) {
        for parent in &data.parents {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if !matches!(&parent_def.kind, DefKind::Interface(_)) {
                self.ctx.report(
                    InvalidInheritance::name(),
                    InvalidInheritance::category(),
                    Diag::error(format!(
                        "interface `{}` cannot inherit from {} `{}`",
                        def.ident.name,
                        parent_def.kind.kind_name(),
                        parent_def.ident.name
                    ))
                    .label(
                        Label::new(def.ident.span)
                            .message("interfaces can only inherit from other interfaces")
                            .color(Color::Red),
                    ),
                );
            }
        }
    }

    fn visit_valuetype(&mut self, def: &'a hir::Def, data: &'a hir::ValueTy) {
        if let Some(parent) = data.parent {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if !matches!(&parent_def.kind, DefKind::Valuetype(_)) {
                self.ctx.report(
                    InvalidInheritance::name(),
                    InvalidInheritance::category(),
                    Diag::error(format!(
                        "valuetype `{}` cannot inherit from {} `{}`",
                        def.ident.name,
                        parent_def.kind.kind_name(),
                        parent_def.ident.name
                    ))
                    .label(
                        Label::new(def.ident.span)
                            .message("valuetypes can only inherit from other valuetypes")
                            .color(Color::Red),
                    ),
                );
            }
        }

        if let Some(supports) = data.supports {
            let supports_def = self.hir.context.definitions.get(supports.def_id);
            if !matches!(&supports_def.kind, DefKind::Interface(_)) {
                self.ctx.report(
                    InvalidInheritance::name(),
                    InvalidInheritance::category(),
                    Diag::error(format!(
                        "valuetype `{}` cannot support {} `{}`",
                        def.ident.name,
                        supports_def.kind.kind_name(),
                        supports_def.ident.name
                    ))
                    .label(
                        Label::new(def.ident.span)
                            .message("valuetypes can only support interfaces")
                            .color(Color::Red),
                    ),
                );
            }
        }
    }

    fn visit_bitset(&mut self, def: &'a hir::Def, data: &'a hir::BitsetTy) {
        if let Some(parent) = data.parent {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if !matches!(&parent_def.kind, DefKind::Bitset(_)) {
                self.ctx.report(
                    InvalidInheritance::name(),
                    InvalidInheritance::category(),
                    Diag::error(format!(
                        "bitset `{}` cannot inherit from {} `{}`",
                        def.ident.name,
                        parent_def.kind.kind_name(),
                        parent_def.ident.name
                    ))
                    .label(
                        Label::new(def.ident.span)
                            .message("bitsets can only inherit from other bitsets")
                            .color(Color::Red),
                    ),
                );
            }
        }
    }
}
