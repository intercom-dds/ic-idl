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
use ic_hir::hir::{DefId, DefKind};

pub(crate) fn children(hir: &ResolvedGraph, id: DefId) -> &[DefId] {
    match &hir.context.definitions.get(id).kind {
        DefKind::Module(module) => &module.definitions,
        _ => &[],
    }
}

pub(crate) fn siblings_mut(
    hir: &mut ResolvedGraph,
    parent: Option<DefId>,
) -> Option<&mut Vec<DefId>> {
    match parent {
        None => Some(&mut hir.order),
        Some(parent_id) => match &mut hir.context.definitions.get_mut(parent_id).kind {
            DefKind::Module(module) => Some(&mut module.definitions),
            _ => None,
        },
    }
}

pub(crate) fn detach(hir: &mut ResolvedGraph, id: DefId) -> Option<usize> {
    let parent = hir.context.definitions.get(id).parent;
    let siblings = siblings_mut(hir, parent)?;
    let position = siblings.iter().position(|&child| child == id)?;

    let _ = siblings.remove(position);
    Some(position)
}

pub(crate) fn attach(
    hir: &mut ResolvedGraph,
    parent: Option<DefId>,
    id: DefId,
    position: Option<usize>,
) {
    hir.context.definitions.get_mut(id).parent = parent;

    let Some(siblings) = siblings_mut(hir, parent) else {
        return;
    };

    match position {
        Some(index) => {
            siblings.insert(index.min(siblings.len()), id);
        }
        None => siblings.push(id),
    }
}

pub(crate) fn prune_empty(hir: &mut ResolvedGraph, mut module: Option<DefId>) {
    while let Some(id) = module {
        let def = hir.context.definitions.get(id);
        if !matches!(def.kind, DefKind::Module(_)) || !children(hir, id).is_empty() {
            return;
        }

        let parent = def.parent;
        detach(hir, id);
        hir.context.definitions.get_mut(id).parent = None;
        module = parent;
    }
}
