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
use ic_hir::hir::{Def, DefId, DefKind, InterfaceTy, StructTy};
use ic_hir::visit::{Visitor, walk_tree};

use crate::{Category, Lint, LintCtx};

/// Lint that checks for circular inheritance in structs and interfaces.
pub struct CircularInheritance<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
    /// Track types currently being visited to detect cycles
    visiting: HashSet<DefId>,
}

impl<'a> Lint<'a> for CircularInheritance<'a> {
    fn name() -> &'static str {
        "circular_inheritance"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Circular inheritance in structs or interfaces"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut lint = CircularInheritance {
            ctx,
            hir,
            visiting: HashSet::new(),
        };
        walk_tree(&mut lint, hir);
    }
}

impl CircularInheritance<'_> {
    /// Check for circular inheritance starting from the given type
    fn check_circular_inheritance(&mut self, id: DefId, path: &mut Vec<(DefId, String)>) -> bool {
        // If we're already visiting this type, we found a cycle
        if self.visiting.contains(&id) {
            return true;
        }

        // Mark as visiting
        self.visiting.insert(id);
        let def_name = self.hir.context.definitions.get(id).ident.name.clone();
        path.push((id, def_name));

        let mut found_cycle = false;

        // Get parent types to check
        let parents = {
            let def = self.hir.context.definitions.get(id);
            match &def.kind {
                DefKind::Struct(s) => s.parent.map(|p| vec![p]).unwrap_or_default(),
                DefKind::Interface(i) => i.parents.clone(),
                _ => Vec::new(),
            }
        };

        // Check each parent for cycles
        for parent in parents {
            if self.check_circular_inheritance(parent, path) {
                found_cycle = true;

                // Report the cycle when we're back at the type that completes the cycle
                if path.iter().any(|(type_id, _)| *type_id == parent) {
                    let def = self.hir.context.definitions.get(id);
                    let parent_def = self.hir.context.definitions.get(parent);

                    // Build the cycle path for the error message
                    let cycle_start = path
                        .iter()
                        .position(|(type_id, _)| *type_id == parent)
                        .unwrap();
                    let cycle_path: Vec<_> = path[cycle_start..]
                        .iter()
                        .map(|(_, name)| name.as_str())
                        .collect();

                    let diag = ic_diagnostic::error_span(
                        format!(
                            "{} `{}` has circular inheritance",
                            if matches!(def.kind, DefKind::Struct(_)) {
                                "struct"
                            } else {
                                "interface"
                            },
                            def.ident.name
                        ),
                        Label::new(def.span).message("type is part of a circular dependency"),
                    )
                    .note(format!(
                        "inheritance cycle: {} -> {}",
                        cycle_path.join(" -> "),
                        parent_def.ident.name
                    ));

                    Self::report(self.ctx, diag);
                }
            }
        }

        // Clean up
        path.pop();
        self.visiting.remove(&id);

        found_cycle
    }
}

impl<'a> Visitor<'a> for CircularInheritance<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_struct(&mut self, def: &'a Def, _struct_ty: &'a StructTy) {
        let mut path = Vec::new();
        self.check_circular_inheritance(def.id, &mut path);
    }

    fn visit_interface(&mut self, def: &'a Def, _interface_ty: &'a InterfaceTy) {
        let mut path = Vec::new();
        self.check_circular_inheritance(def.id, &mut path);
    }
}
