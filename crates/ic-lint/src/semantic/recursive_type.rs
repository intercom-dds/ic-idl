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

use std::collections::HashSet;

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, Def, DefFlags, DefId, ExceptTy, StructTy, Ty, TyKind, UnionTy};
use ic_hir::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Lint that checks for recursive types without proper indirection.
///
/// A recursive type must have indirection through either:
/// 1. Being the element type of a sequence or map
/// 2. Having @shared or @external annotation on the recursive field
pub struct RecursiveType<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
    /// Track types currently being checked to detect recursion
    checking: HashSet<DefId>,
}

impl<'a> Lint<'a> for RecursiveType<'a> {
    fn name() -> &'static str {
        "recursive_type"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when recursive types lack proper indirection"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut lint = RecursiveType {
            ctx,
            hir,
            checking: HashSet::new(),
        };
        walk_tree(&mut lint, hir);
    }
}

impl RecursiveType<'_> {
    /// Check if a type reference is properly indirected
    fn is_indirected(&self, ty: &Ty, annotations: &[Ann]) -> bool {
        match &ty.kind {
            // Sequences and maps provide indirection
            TyKind::Sequence { .. } | TyKind::Map { .. } => true,
            // Check if the field itself has @shared or @external
            _ => annotations.iter().any(|ann| {
                if let Some(def_id) = ann.def_id {
                    let def = self.hir.context.type_of(def_id);
                    // Check if it's a builtin annotation with the right name
                    def.flags.contains(DefFlags::IS_BUILTIN)
                        && (def.ident.name == "shared" || def.ident.name == "external")
                } else {
                    false
                }
            }),
        }
    }

    /// Check if a type contains a recursive reference to the given `type_id`
    fn check_type_recursion(
        &mut self,
        ty: &Ty,
        containing_type: DefId,
        member_annotations: &[Ann],
        member_span: ic_syntax::Span,
    ) {
        match &ty.kind {
            TyKind::Adt(id) => {
                if *id == containing_type {
                    // Found direct recursion - check if it's properly indirected
                    if !self.is_indirected(ty, member_annotations) {
                        let def = self.hir.context.definitions.get(containing_type);
                        Self::report(
                            self.ctx,
                            ic_diagnostic::error_span(
                                format!(
                                    "type `{}` contains itself without indirection",
                                    def.ident.name
                                ),
                                Label::new(member_span).message("recursive member here"),
                            )
                            .note(
                                "recursive types must use indirection through sequences, maps, or \
                                 @external annotations",
                            ),
                        );
                    }
                }
            }
            // Arrays of self are also problematic
            TyKind::Array { ty, .. } => {
                if let TyKind::Adt(id) = &ty.kind {
                    if *id == containing_type {
                        let def = self.hir.context.definitions.get(containing_type);
                        Self::report(
                            self.ctx,
                            ic_diagnostic::error_span(
                                format!(
                                    "type `{}` contains itself without indirection",
                                    def.ident.name
                                ),
                                Label::new(member_span).message("recursive member here"),
                            )
                            .note(
                                "recursive types must use indirection through sequences, maps, or \
                                 @external annotations",
                            ),
                        );
                    }
                }
            }
            // Maps provide indirection, so no need to check
            _ => {}
        }
    }

    fn check_struct(&mut self, def: &Def, struct_ty: &StructTy) {
        if !self.checking.insert(def.id) {
            return; // Already checking this type
        }

        for member in &struct_ty.members {
            self.check_type_recursion(&member.ty, def.id, &member.annotations, member.ident.span);
        }

        self.checking.remove(&def.id);
    }

    fn check_except(&mut self, def: &Def, except_ty: &ExceptTy) {
        if !self.checking.insert(def.id) {
            return; // Already checking this type
        }

        for member in &except_ty.members {
            self.check_type_recursion(&member.ty, def.id, &member.annotations, member.ident.span);
        }

        self.checking.remove(&def.id);
    }
}

impl<'a> Visitor<'a> for RecursiveType<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, struct_ty: &'a StructTy) {
        self.check_struct(def, struct_ty);
        ic_hir::visit::walk_struct(self, struct_ty);
    }

    fn visit_union(&mut self, _def: &'a Def, union_ty: &'a UnionTy) {
        ic_hir::visit::walk_union(self, union_ty);
    }

    fn visit_except(&mut self, def: &'a Def, except_ty: &'a ExceptTy) {
        self.check_except(def, except_ty);
        ic_hir::visit::walk_except(self, except_ty);
    }
}
