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

use ic_diagnostic::Label;
use ic_hir::hir::{Def, DefId, DefKind, InterfaceTy, ProtoTy};
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph};

use crate::{Category, Lint, LintCtx};

/// Lint that checks for duplicate method names in interface inheritance chains.
/// This is an error because it creates ambiguity and would fail in most target languages.
pub struct DuplicateMethods<'a> {
    ctx: &'a LintCtx<'a>,
    hir_ctx: &'a Context,
}

impl<'a> Lint<'a> for DuplicateMethods<'a> {
    fn name() -> &'static str {
        "duplicate_methods"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = DuplicateMethods {
            ctx,
            hir_ctx: &hir.context,
        };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> DuplicateMethods<'a> {
    /// Collect all methods from an interface and its parents
    fn collect_methods_with_sources(
        &self,
        interface_id: DefId,
        visited: &mut HashSet<DefId>,
    ) -> HashMap<String, Vec<(DefId, &'a ProtoTy)>> {
        let mut methods = HashMap::new();

        // Check if we've already visited this interface to prevent infinite loops
        if visited.contains(&interface_id) {
            // Already visited this interface (handles circular inheritance)
            return methods;
        }

        visited.insert(interface_id);

        let def = self.hir_ctx.definitions.get(interface_id);
        match &def.kind {
            DefKind::Interface(interface) => {
                // Add methods from this interface
                for proto in &interface.prototypes {
                    methods
                        .entry(proto.ident.name.clone())
                        .or_insert_with(Vec::new)
                        .push((interface_id, proto));
                }

                // Recursively collect from parent interfaces
                for &parent_id in &interface.parents {
                    let parent_methods = self.collect_methods_with_sources(parent_id, visited);
                    for (name, sources) in parent_methods {
                        methods.entry(name).or_insert_with(Vec::new).extend(sources);
                    }
                }
            }
            _ => {
                // Not an interface, return empty methods
                return methods;
            }
        }

        methods
    }

    /// Check if two method signatures are compatible (same parameters and return type)
    fn methods_compatible(_method1: &ProtoTy, _method2: &ProtoTy) -> bool {
        // For now, always return false to treat all duplicate names as incompatible
        // This ensures we catch all duplicate method names as errors
        // TODO: Implement proper type comparison when TyKind implements PartialEq
        false
    }
}

impl<'a> Visitor<'a> for DuplicateMethods<'a> {
    fn visit_interface(&mut self, def: &'a Def, interface: &'a InterfaceTy) {
        // Only process interfaces, not other types
        if !matches!(def.kind, DefKind::Interface(_)) {
            return;
        }

        let mut visited = HashSet::new();
        let methods = self.collect_methods_with_sources(def.id, &mut visited);

        // Check for duplicates
        for (method_name, sources) in &methods {
            if sources.len() > 1 {
                // We have duplicate method names - check if they're compatible
                let first_method = sources[0].1;
                let all_compatible = sources[1..]
                    .iter()
                    .all(|(_, method)| Self::methods_compatible(first_method, method));

                if !all_compatible {
                    // Incompatible duplicate methods - this is an error
                    let current_interface_has_method = sources.iter().any(|(id, _)| *id == def.id);

                    if current_interface_has_method {
                        // This interface defines a method that conflicts with inherited ones
                        let (_, current_method) =
                            sources.iter().find(|(id, _)| *id == def.id).unwrap();

                        let diag = ic_diagnostic::error_span(
                            format!(
                                "interface `{}` defines method `{}` which conflicts with \
                                 inherited method",
                                def.ident.name, method_name
                            ),
                            Label::new(current_method.ident.span)
                                .message("conflicting method definition"),
                        );

                        // Add labels for inherited methods
                        let mut diag = diag;
                        for (source_id, source_method) in sources {
                            if *source_id != def.id {
                                let source_def = self.hir_ctx.definitions.get(*source_id);
                                diag = diag.label(Label::new(source_method.ident.span).message(
                                    format!("inherited from `{}`", source_def.ident.name),
                                ));
                            }
                        }

                        Self::report(self.ctx, diag);
                    } else {
                        // This interface inherits conflicting methods from multiple parents
                        let diag = ic_diagnostic::error_span(
                            format!(
                                "interface `{}` inherits conflicting definitions of method `{}`",
                                def.ident.name, method_name
                            ),
                            Label::new(def.ident.span)
                                .message("interface with conflicting inherited methods"),
                        );

                        // Add labels for all conflicting methods
                        let mut diag = diag;
                        for (source_id, source_method) in sources {
                            let source_def = self.hir_ctx.definitions.get(*source_id);
                            diag = diag.label(
                                Label::new(source_method.ident.span)
                                    .message(format!("defined in `{}`", source_def.ident.name)),
                            );
                        }

                        Self::report(self.ctx, diag);
                    }
                } else if sources.len() > 1 {
                    // Compatible duplicate methods - still worth a warning
                    // Check if this interface is redefining an inherited method
                    let current_interface_has_method = sources.iter().any(|(id, _)| *id == def.id);

                    if current_interface_has_method {
                        let (_, current_method) =
                            sources.iter().find(|(id, _)| *id == def.id).unwrap();

                        if let Some(mut diag) = self.ctx.diag_span(
                            Self::name(),
                            Self::category(),
                            format!(
                                "interface `{}` redefines inherited method `{}`",
                                def.ident.name, method_name
                            ),
                            Label::new(current_method.ident.span).message("method redefinition"),
                        ) {
                            // Show where it was inherited from
                            for (source_id, source_method) in sources {
                                if *source_id != def.id {
                                    let source_def = self.hir_ctx.definitions.get(*source_id);
                                    diag =
                                        diag.label(Label::new(source_method.ident.span).message(
                                            format!("inherited from `{}`", source_def.ident.name),
                                        ));
                                }
                            }

                            diag = diag.note(
                                "while this redefinition is compatible, it may cause confusion",
                            );

                            Self::report(self.ctx, diag);
                        }
                    }
                }
            }
        }

        // Continue visiting
        ic_hir::visit::walk_interface(self, def, interface);
    }
}
