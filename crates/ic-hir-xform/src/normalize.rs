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

use std::collections::{HashMap, HashSet};

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, TyKind};

/// Validates that the HIR is consistent and that all relationships are
/// correct. This is meant as a last-effort check to catch bad transformations.
///
/// # Panics
///
/// Panics if the HIR is found to be in an inconsistent state.
pub fn normalize(hir: &ResolvedGraph) {
    let _span = tracing::debug_span!("xform", name = "normalize").entered();
    tracing::debug!("validating HIR");

    if let Err(errors) = validate_hir(hir) {
        panic!("HIR validation failed:\n{}", errors.join("\n"));
    }
}

fn validate_hir(hir: &ResolvedGraph) -> Result<(), Vec<String>> {
    let mut errors = vec![];
    validate_parent_relationships(hir, &mut errors);
    validate_definition_lists(hir, &mut errors);
    validate_scope_relationships(hir, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_parent_relationships(hir: &ResolvedGraph, errors: &mut Vec<String>) {
    let mut containment: HashMap<DefId, HashSet<DefId>> = HashMap::new();
    for (def_id, def) in &hir.context.definitions {
        let children: &[DefId] = match &def.kind {
            DefKind::Module(m) => &m.definitions,
            DefKind::Interface(i) => &i.definitions,
            DefKind::Valuetype(v) => &v.definitions,
            DefKind::Annotation(a) => &a.types,
            DefKind::Enum(e) => &e.fields,
            _ => continue,
        };
        containment.insert(def_id, children.iter().copied().collect());
    }

    for (def_id, def) in &hir.context.definitions {
        if let Some(parent_id) = def.parent {
            let parent_def = hir.context.definitions.get(parent_id);

            let is_bitmask_flag = if let DefKind::Const(c) = &def.kind {
                if let TyKind::Adt(type_id) = &c.ty.kind {
                    matches!(
                        &hir.context.definitions.get(*type_id).kind,
                        DefKind::Bitmask(_)
                    )
                } else {
                    false
                }
            } else {
                false
            };

            let is_contained = if is_bitmask_flag {
                true
            } else {
                containment
                    .get(&parent_id)
                    .is_some_and(|set| set.contains(&def_id))
                    || matches!(parent_def.kind, DefKind::Bitmask(_))
                    || !matches!(
                        parent_def.kind,
                        DefKind::Module(_)
                            | DefKind::Interface(_)
                            | DefKind::Valuetype(_)
                            | DefKind::Annotation(_)
                            | DefKind::Enum(_)
                    )
            };

            if !is_contained {
                errors.push(format!(
                    "Definition '{}' claims parent '{}' but parent doesn't contain it",
                    def.ident.name, parent_def.ident.name
                ));
            }
        }
    }
}

fn validate_definition_lists(hir: &ResolvedGraph, errors: &mut Vec<String>) {
    for (def_id, def) in &hir.context.definitions {
        let children = match &def.kind {
            DefKind::Module(m) => &m.definitions[..],
            DefKind::Interface(i) => &i.definitions[..],
            DefKind::Valuetype(v) => &v.definitions[..],
            _ => continue,
        };

        for &child_id in children {
            let child_def = hir.context.definitions.get(child_id);
            if child_def.parent != Some(def_id) {
                errors.push(format!(
                    "{} '{}' contains '{}' but child has different parent",
                    def.kind.kind_name(),
                    def.ident.name,
                    child_def.ident.name
                ));
            }
        }
    }
}

fn validate_scope_relationships(hir: &ResolvedGraph, errors: &mut Vec<String>) {
    for scope in &hir.context.scopes.scopes {
        if let Some(def_id) = scope.def_id {
            let def = hir.context.definitions.get(def_id);
            if !matches!(
                def.kind,
                DefKind::Module(_)
                    | DefKind::Interface(_)
                    | DefKind::Valuetype(_)
                    | DefKind::Union(_)
                    | DefKind::Enum(_)
                    | DefKind::Bitmask(_)
                    | DefKind::Bitset(_)
                    | DefKind::Annotation(_)
            ) {
                errors.push(format!(
                    "'{}' ({}) has a scope but should not",
                    def.ident.name,
                    def.kind.kind_name(),
                ));
            }
        }
    }
}
