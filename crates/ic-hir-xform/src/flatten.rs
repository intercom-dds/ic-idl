// Copyright 2026 KONGSBERG
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
use ic_hir::hir::{Decl, Def, DefFlags, DefId, DefKind};

pub struct FlattenResult {
    pub hir: ResolvedGraph,
    pub moved_defs: HashSet<DefId>,
}

fn qualified_name(hir: &ResolvedGraph, def_id: DefId, separator: &str) -> String {
    let def = hir.context.type_of(def_id);
    let mut parts = vec![def.ident.name.clone()];
    let mut current = def.parent;

    if let Some(parent_id) = current {
        let parent = hir.context.type_of(parent_id);
        if matches!(parent.kind, DefKind::Enum(_) | DefKind::Bitmask(_)) {
            current = parent.parent;
        }
    }

    while let Some(parent_id) = current {
        let parent = hir.context.type_of(parent_id);
        if matches!(
            parent.kind,
            DefKind::Module(_) | DefKind::Interface(_) | DefKind::Valuetype(_)
        ) {
            parts.push(parent.ident.name.clone());
        }
        current = parent.parent;
    }

    parts.reverse();
    parts.join(separator)
}

fn flatten_order(
    hir: &ResolvedGraph,
    def_ids: &[DefId],
    declarations: &HashMap<DefId, DefId>,
    order: &mut Vec<DefId>,
    seen: &mut HashSet<DefId>,
) {
    for &def_id in def_ids {
        if !seen.insert(def_id) {
            continue;
        }

        match &hir.context.type_of(def_id).kind {
            DefKind::Module(module) => {
                flatten_order(hir, &module.definitions, declarations, order, seen);
            }
            DefKind::Interface(interface) => {
                if let Some(&declaration) = declarations.get(&def_id)
                    && seen.insert(declaration)
                {
                    order.push(declaration);
                }

                flatten_order(hir, &interface.definitions, declarations, order, seen);
                order.push(def_id);
            }
            DefKind::Valuetype(valuetype) => {
                if let Some(&declaration) = declarations.get(&def_id)
                    && seen.insert(declaration)
                {
                    order.push(declaration);
                }

                flatten_order(hir, &valuetype.definitions, declarations, order, seen);
                order.push(def_id);
            }
            _ => order.push(def_id),
        }
    }
}

#[must_use]
pub fn transform(mut hir: ResolvedGraph, separator: &str) -> FlattenResult {
    let containing_types: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(def_id, def)| {
            if def.flags.contains(DefFlags::IS_BUILTIN) {
                return None;
            }

            let decl = match def.kind {
                DefKind::Interface(_) => Decl::Interface,
                DefKind::Valuetype(_) => Decl::Valuetype,
                _ => return None,
            };
            Some((def_id, def.ident.clone(), def.parent, decl))
        })
        .collect();

    let declarations: HashMap<_, _> = containing_types
        .into_iter()
        .map(|(def_id, ident, parent, decl)| {
            let decl_id = hir.context.definitions.alloc_with_id(|id| Def {
                id,
                span: ident.span,
                ident,
                parent,
                annotations: vec![],
                kind: DefKind::Decl(decl),
                flags: DefFlags::IS_INCOMPLETE | DefFlags::IS_SYNTHESIZED,
            });
            (def_id, decl_id)
        })
        .collect();

    let mut moved_defs = HashSet::new();
    let renames: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(def_id, def)| {
            if def.flags.contains(DefFlags::IS_BUILTIN) || matches!(def.kind, DefKind::Module(_)) {
                return None;
            }

            let qualified = qualified_name(&hir, def_id, separator);
            (qualified != def.ident.name).then_some((def_id, qualified))
        })
        .collect();

    for (def_id, name) in renames {
        hir.context.definitions.get_mut(def_id).ident.name = name;
        moved_defs.insert(def_id);
    }

    let mut order = vec![];
    let mut seen = HashSet::new();
    flatten_order(&hir, &hir.order, &declarations, &mut order, &mut seen);
    hir.order = order;

    for (_, def) in &mut hir.context.definitions {
        match &mut def.kind {
            DefKind::Module(module) => module.definitions.clear(),
            DefKind::Interface(interface) => interface.definitions.clear(),
            DefKind::Valuetype(valuetype) => valuetype.definitions.clear(),
            _ => {}
        }
    }

    FlattenResult { hir, moved_defs }
}
