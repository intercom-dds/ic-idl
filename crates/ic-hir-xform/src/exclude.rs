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
use ic_hir::hir::{Def, DefId};
use tracing::{debug, debug_span};

use crate::module_tree::detach;

#[must_use]
pub fn transform(mut hir: ResolvedGraph, paths: &[Vec<String>]) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "exclude").entered();
    debug!("applying transform");

    let excluded: Vec<DefId> = hir
        .context
        .definitions
        .iter()
        .filter(|(_, def)| paths.iter().any(|path| matches_path(&hir, def, path)))
        .map(|(id, _)| id)
        .collect();

    for id in excluded {
        if detach(&mut hir, id).is_none() {
            let name = &hir.context.definitions.get(id).ident.name;
            debug!("skipping {name} with a non-module parent");
        }
    }

    hir
}

fn matches_path(hir: &ResolvedGraph, def: &Def, path: &[String]) -> bool {
    let mut segments = path.iter().rev();
    if segments.next() != Some(&def.ident.name) {
        return false;
    }

    let mut parent = def.parent;
    for segment in segments {
        let Some(id) = parent else {
            return false;
        };

        let parent_def = hir.context.definitions.get(id);
        if parent_def.ident.name != *segment {
            return false;
        }

        parent = parent_def.parent;
    }

    parent.is_none()
}
