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

use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefFlags, DefId, DefKind, Ident, ModuleTy, Span};
use tracing::{debug, debug_span};

use crate::module_tree::{attach, children, detach, prune_empty, siblings_mut};

#[derive(Debug, Clone)]
pub struct ModuleMap {
    pub from: Vec<String>,
    pub to: Vec<String>,
}

#[must_use]
pub fn transform(mut hir: ResolvedGraph, maps: &[ModuleMap]) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "remap_modules").entered();
    debug!("applying transform");

    for map in maps {
        let matches = find_modules(&hir, &map.from);
        if matches.is_empty() {
            continue;
        }

        let Some((last, chain)) = map.to.split_last() else {
            for module_id in matches {
                hoist(&mut hir, module_id);
            }

            continue;
        };

        let first = hir.context.definitions.get(matches[0]);
        let span = first.span;
        let flags = DefFlags::IS_SYNTHESIZED | (first.flags & DefFlags::IS_INCLUDED);
        let root_position = root_position(&hir, matches[0]);

        let target = synthesize_chain(&mut hir, chain, span, flags, root_position);

        let mut position = root_position;
        for module_id in matches {
            let old_parent = hir.context.definitions.get(module_id).parent;
            detach(&mut hir, module_id);
            prune_empty(&mut hir, old_parent);

            hir.context
                .definitions
                .get_mut(module_id)
                .ident
                .name
                .clone_from(last);

            if target.is_some() {
                attach(&mut hir, target, module_id, None);
            } else {
                attach(&mut hir, None, module_id, position);
                position = position.map(|index| index + 1);
            }
        }
    }

    hir
}

fn find_modules(hir: &ResolvedGraph, path: &[String]) -> Vec<DefId> {
    let mut candidates = hir.order.clone();
    let mut matched = Vec::new();

    for (index, segment) in path.iter().enumerate() {
        matched = candidates
            .iter()
            .copied()
            .filter(|&id| {
                let def = hir.context.definitions.get(id);
                matches!(def.kind, DefKind::Module(_)) && def.ident.name == *segment
            })
            .collect();

        if index + 1 < path.len() {
            candidates = matched
                .iter()
                .flat_map(|&id| children(hir, id).iter().copied())
                .collect();
        }
    }

    matched
}

fn root_position(hir: &ResolvedGraph, id: DefId) -> Option<usize> {
    let mut root = id;
    while let Some(parent) = hir.context.definitions.get(root).parent {
        root = parent;
    }

    hir.order.iter().position(|&def| def == root)
}

fn synthesize_chain(
    hir: &mut ResolvedGraph,
    chain: &[String],
    span: Span,
    flags: DefFlags,
    root_position: Option<usize>,
) -> Option<DefId> {
    let mut parent = None;
    let mut position = root_position;

    for name in chain {
        let ident = Ident {
            name: name.clone(),
            span,
        };

        let id = hir.context.definitions.alloc_with_id(|id| Def {
            id,
            ident,
            parent,
            annotations: vec![],
            span,
            kind: DefKind::Module(ModuleTy {
                definitions: vec![],
            }),
            flags,
        });

        attach(hir, parent, id, position);
        parent = Some(id);
        position = None;
    }

    parent
}

fn hoist(hir: &mut ResolvedGraph, id: DefId) {
    let parent = hir.context.definitions.get(id).parent;

    let Some(siblings) = siblings_mut(hir, parent) else {
        return;
    };

    let position = siblings
        .iter()
        .position(|&sibling| sibling == id)
        .expect("module is listed under its parent");

    let contents = match &mut hir.context.definitions.get_mut(id).kind {
        DefKind::Module(module) => std::mem::take(&mut module.definitions),
        _ => Vec::new(),
    };

    for &child in &contents {
        hir.context.definitions.get_mut(child).parent = parent;
    }

    hir.context.definitions.get_mut(id).parent = None;

    if let Some(siblings) = siblings_mut(hir, parent) {
        siblings.splice(position..=position, contents);
    }
}
