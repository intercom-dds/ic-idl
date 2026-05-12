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

use std::collections::{HashMap, HashSet};

use ic_alloc::insensitive::CaseString;
use ic_cli::color::Colorize as _;
use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    BitmaskTy, Def, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy, ProtoTy, StructTy, UnionTy,
};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Unified lint that checks for duplicate names in all IDL constructs.
/// Uses case-insensitive comparison as per IDL specification.
pub struct DuplicateName<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for DuplicateName<'a> {
    fn name() -> &'static str {
        "duplicate-name"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Duplicate names in IDL definitions"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateName { ctx, hir };
        ic_hir::visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> DuplicateName<'a> {
    /// Report a duplicate name with appropriate context
    fn report_duplicate(
        &self,
        name: &str,
        construct_type: &str,
        parent_name: &str,
        span: ic_syntax::Span,
        first_span: ic_syntax::Span,
    ) {
        Self::report(
            self.ctx,
            ic_diagnostic::error_span(
                format!(
                    "duplicate {} `{}` in {} '{}'",
                    construct_type,
                    name.yellow(),
                    construct_type,
                    parent_name
                ),
                Label::new(span).message(format!("duplicate {construct_type}")),
            )
            .label(Label::new(first_span).message("first defined here"))
            .note("names are case-insensitive"),
        );
    }

    /// Check a list of named items for duplicates
    fn check_names<'b, T, F>(
        &self,
        items: &'b [T],
        get_ident: F,
        construct_type: &str,
        parent_name: &str,
    ) where
        F: Fn(&'b T) -> &'b ic_hir::hir::Ident,
    {
        let mut seen = HashMap::new();
        for item in items {
            let ident = get_ident(item);
            let name_lower = CaseString::new(ident.name.as_str());

            if let Some(&first_span) = seen.get(&name_lower) {
                // Found a duplicate
                Self::report(
                    self.ctx,
                    ic_diagnostic::error_span(
                        format!(
                            "duplicate {} `{}` in {} '{}'",
                            construct_type,
                            ident.name.yellow(),
                            construct_type,
                            parent_name
                        ),
                        Label::new(ident.span).message(format!("duplicate {construct_type}")),
                    )
                    .label(Label::new(first_span).message("first defined here"))
                    .note("names are case-insensitive"),
                );
            } else {
                seen.insert(name_lower, ident.span);
            }
        }
    }

    /// Collect all methods from an interface and its parents
    fn collect_methods_with_sources(
        &self,
        interface_id: DefId,
        visited: &mut HashSet<DefId>,
    ) -> HashMap<CaseString, Vec<(DefId, &'a ProtoTy)>> {
        let mut methods = HashMap::new();

        if visited.contains(&interface_id) {
            return methods;
        }
        visited.insert(interface_id);

        let def = self.hir.context.definitions.get(interface_id);
        if let DefKind::Interface(interface) = &def.kind {
            // Add methods from this interface
            for proto in &interface.prototypes {
                methods
                    .entry(CaseString::new(proto.ident.name.as_str()))
                    .or_insert_with(Vec::new)
                    .push((interface_id, proto));
            }

            // Recursively collect from parent interfaces
            for parent in &interface.parents {
                let parent_methods = self.collect_methods_with_sources(parent.def_id, visited);
                for (name, sources) in parent_methods {
                    methods.entry(name).or_insert_with(Vec::new).extend(sources);
                }
            }
        }

        methods
    }
}

impl<'a> Visitor<'a> for DuplicateName<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, struct_ty: &'a StructTy) {
        let mut seen: HashMap<CaseString, ic_syntax::Span> = HashMap::new();

        // Check members from all parent structs first
        let mut parent = struct_ty.parent;
        let mut visited_parents = HashSet::new();

        while let Some(parent_ref) = parent {
            if !visited_parents.insert(parent_ref.def_id) {
                break;
            }

            let parent_def = self.hir.context.definitions.get(parent_ref.def_id);
            if let DefKind::Struct(parent_struct) = &parent_def.kind {
                for member in &parent_struct.members {
                    seen.insert(
                        CaseString::new(member.ident.name.as_str()),
                        member.ident.span,
                    );
                }
                parent = parent_struct.parent;
            } else {
                break;
            }
        }

        // Check this struct's own members
        for member in &struct_ty.members {
            let name = CaseString::new(member.ident.name.as_str());
            if let Some(&first_span) = seen.get(&name) {
                self.report_duplicate(
                    &member.ident.name,
                    "member",
                    &def.ident.name,
                    member.ident.span,
                    first_span,
                );
            } else {
                seen.insert(name, member.ident.span);
            }
        }

        ic_hir::visit::walk_struct(self, struct_ty);
    }

    fn visit_except(&mut self, def: &'a Def, except_ty: &'a ExceptTy) {
        self.check_names(&except_ty.members, |m| &m.ident, "member", &def.ident.name);
        ic_hir::visit::walk_except(self, except_ty);
    }

    fn visit_enum(&mut self, def: &'a Def, enum_ty: &'a EnumTy) {
        // Skip built-in types
        if def.ident.name.starts_with("intercom::") {
            return;
        }

        // Check for duplicate field names
        let field_defs: Vec<&Def> = enum_ty
            .fields
            .iter()
            .map(|&id| self.context().definitions.get(id))
            .collect();
        self.check_names(&field_defs, |f| &f.ident, "field", &def.ident.name);

        ic_hir::visit::walk_enum(self, enum_ty);
    }

    fn visit_bitmask(&mut self, def: &'a Def, bitmask_ty: &'a BitmaskTy) {
        // Get flag definitions from DefIds
        let flag_defs: Vec<&Def> = bitmask_ty
            .flags
            .iter()
            .map(|&id| self.context().definitions.get(id))
            .collect();
        self.check_names(&flag_defs, |f| &f.ident, "flag", &def.ident.name);
        ic_hir::visit::walk_bitmask(self, bitmask_ty);
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        self.check_names(&union_ty.variants, |v| &v.ident, "variant", &def.ident.name);
        ic_hir::visit::walk_union(self, union_ty);
    }

    fn visit_interface(&mut self, def: &'a Def, interface: &'a InterfaceTy) {
        if !matches!(def.kind, DefKind::Interface(_)) {
            return;
        }

        // First check for duplicate methods within the same interface
        self.check_names(
            &interface.prototypes,
            |p| &p.ident,
            "method",
            &def.ident.name,
        );

        // Then, collect inherited methods only (exclude methods from current interface)
        let mut visited = HashSet::new();
        visited.insert(def.id); // Mark current interface as visited to exclude its methods

        let mut inherited_methods = HashMap::new();

        // Collect methods from parent interfaces only
        for parent in &interface.parents {
            let parent_methods = self.collect_methods_with_sources(parent.def_id, &mut visited);
            for (name, sources) in parent_methods {
                inherited_methods
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .extend(sources);
            }
        }

        // Now check current interface's methods against inherited ones
        for proto in &interface.prototypes {
            let method_name = CaseString::new(proto.ident.name.as_str());

            if let Some(inherited_sources) = inherited_methods.get(&method_name) {
                // This method conflicts with an inherited method
                let diag = ic_diagnostic::error_span(
                    format!(
                        "interface `{}` defines method `{}` which conflicts with inherited method",
                        def.ident.name,
                        proto.ident.name.yellow()
                    ),
                    Label::new(proto.ident.span).message("conflicting method definition"),
                );

                let mut diag = diag;
                for (source_id, source_method) in inherited_sources {
                    let source_def = self.hir.context.definitions.get(source_id);
                    diag = diag.label(
                        Label::new(source_method.ident.span)
                            .message(format!("inherited from `{}`", source_def.ident.name)),
                    );
                }

                Self::report(self.ctx, diag.note("method names are case-insensitive"));
            }
        }

        // Check for conflicting inherited methods from multiple parents
        for sources in inherited_methods.values() {
            if sources.len() > 1 {
                let first_method = sources[0].1;
                let diag = ic_diagnostic::error_span(
                    format!(
                        "interface `{}` inherits conflicting definitions of method `{}`",
                        def.ident.name,
                        first_method.ident.name.yellow()
                    ),
                    Label::new(def.ident.span)
                        .message("interface with conflicting inherited methods"),
                );

                let mut diag = diag;
                for (source_id, source_method) in sources {
                    let source_def = self.hir.context.definitions.get(source_id);
                    diag = diag.label(
                        Label::new(source_method.ident.span)
                            .message(format!("defined in `{}`", source_def.ident.name)),
                    );
                }

                Self::report(self.ctx, diag.note("method names are case-insensitive"));
            }
        }

        ic_hir::visit::walk_interface(self, def, interface);
    }

    fn visit_proto(&mut self, proto: &'a ProtoTy) {
        // Check for duplicate parameter names
        let mut seen: HashMap<CaseString, ic_syntax::Span> = HashMap::new();

        for param in &proto.params {
            let name = CaseString::new(param.ident.name.as_str());
            if let Some(&first_span) = seen.get(&name) {
                self.report_duplicate(
                    &param.ident.name,
                    "parameter",
                    &proto.ident.name,
                    param.ident.span,
                    first_span,
                );
            } else {
                seen.insert(name, param.ident.span);
            }
        }

        ic_hir::visit::walk_proto(self, proto);
    }
}
