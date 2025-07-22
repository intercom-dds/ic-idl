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

use ic_alloc::insensitive::CaseMap;
use ic_cli::color::Colorize;
use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, EnumTy, ExceptTy, InterfaceTy, StructTy, Ty, TyKind, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Tracks consistent capitalization of identifiers across the codebase.
/// Warns when the same identifier is referenced with different capitalization.
pub struct CaseConsistency<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
    case_map: CaseMap<String>,
}

impl<'a> Lint<'a> for CaseConsistency<'a> {
    fn name() -> &'static str {
        "case_consistency"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Inconsistent capitalization of identifiers"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = CaseConsistency {
            ctx,
            hir,
            case_map: CaseMap::new(),
        };

        // First pass: collect all definitions
        visitor.collect_definitions();

        // Second pass: check references
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl CaseConsistency<'_> {
    /// Collect all type definitions with their canonical capitalization
    fn collect_definitions(&mut self) {
        for (_def_id, def) in &self.hir.context.definitions {
            // Store the fully qualified name -> canonical name mapping
            self.case_map
                .insert(def.ident.name.clone(), def.ident.name.clone());
        }
    }

    /// Check a type reference for consistent capitalization
    fn check_type(&self, ty: &Ty) {
        match &ty.kind {
            TyKind::Adt(def_id) => {
                // Extract the reference text from the span
                let reference_text = self.ctx.slice(ty.span);
                let def = self.hir.context.definitions.get(*def_id);

                // Check if the reference matches the canonical name
                if let Some(canonical_name) = self.case_map.get(&def.ident.name) {
                    if canonical_name != reference_text
                        && canonical_name.eq_ignore_ascii_case(reference_text)
                    {
                        Self::report(
                            self.ctx,
                            ic_diagnostic::warn_span(
                                format!(
                                    "inconsistent capitalization: `{}` should be `{}`",
                                    reference_text.yellow(),
                                    canonical_name.yellow()
                                ),
                                Label::new(ty.span).message("used here"),
                            )
                            .note(format!("the canonical name is `{canonical_name}`")),
                        );
                    }
                }
            }
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => self.check_type(ty),
            TyKind::Map { key, elem, .. } => {
                self.check_type(key);
                self.check_type(elem);
            }
            _ => {} // Other type kinds don't have references
        }
    }
}

impl<'a> Visitor<'a> for CaseConsistency<'a> {
    fn visit_struct(&mut self, _def: &'a Def, struct_ty: &'a StructTy) {
        // Check parent reference if any
        if let Some(parent_id) = struct_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
            // For parent references, we don't have a span of the reference itself,
            // so we can't check consistency here
            _ = parent_def;
        }

        // Check member types
        for member in &struct_ty.members {
            self.check_type(&member.ty);
        }

        ic_hir::visit::walk_struct(self, struct_ty);
    }

    fn visit_except(&mut self, _def: &'a Def, except_ty: &'a ExceptTy) {
        // Check member types
        for member in &except_ty.members {
            self.check_type(&member.ty);
        }

        ic_hir::visit::walk_except(self, except_ty);
    }

    fn visit_enum(&mut self, _def: &'a Def, enum_ty: &'a EnumTy) {
        // Check underlying type
        self.check_type(&enum_ty.ty);

        ic_hir::visit::walk_enum(self, enum_ty);
    }

    fn visit_union(&mut self, _def: &'a Def, union_ty: &'a UnionTy) {
        // Check discriminator type
        self.check_type(&union_ty.disc);

        // Check variant types
        for variant in &union_ty.variants {
            self.check_type(&variant.ty);
        }

        ic_hir::visit::walk_union(self, union_ty);
    }

    fn visit_interface(&mut self, def: &'a Def, interface: &'a InterfaceTy) {
        // Check parent interfaces - but we don't have spans for these references
        if let DefKind::Interface(_) = &def.kind {
            // Parent interface references are stored as DefIds, not as type references with spans
        }

        // Check method return and parameter types
        for proto in &interface.prototypes {
            // Check return type (proto.ty is the return type)
            self.check_type(&proto.ty);

            // Check parameter types
            for param in &proto.params {
                self.check_type(&param.ty);
            }
        }

        ic_hir::visit::walk_interface(self, def, interface);
    }
}
