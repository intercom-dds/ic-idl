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
use ic_hir::hir::{Def, DefFlags, DefId, DefKind, Ident, ModuleTy};
use tracing::{debug, debug_span};

/// Moves all root-level definitions into a module based on the file name of the definition
///
/// This is useful for languages like Ada where all types must be decleared inside of a package
///
/// # Panics
///
/// Panics if the file name of the declearing IDL of a root level definition is not a valid `String`.
#[must_use]
pub fn transform(
    mut hir: ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    suffix: &str,
) -> (ResolvedGraph, HashSet<DefId>) {
    let _span = debug_span!("xform", name = "modularize_root").entered();
    debug!("applying transform");

    let mut moved_defs = HashSet::new();
    let mut modules: HashMap<String, DefId> = HashMap::new();
    for def_id in std::mem::take(&mut hir.order) {
        let (name, span) = {
            let def = hir.context.definitions.get(def_id);
            if matches!(def.kind, DefKind::Module(_)) {
                hir.order.push(def_id);
                continue;
            }

            let file_id = def.ident.span.start.file_id;
            let file_path = source_map.name(file_id);
            let mut name = file_path
                .file_stem()
                .expect("IDL file has a file name")
                .to_str()
                .expect("IDL file name is valid UTF-8")
                .replace('.', "_");

            if name.chars().next().is_some_and(char::is_numeric) {
                name = format!("IDL_{name}");
            }

            (name, def.ident.span)
        };

        let module_id = *modules.entry(name.clone()).or_insert_with(|| {
            let name = format!("{name}{suffix}");
            let ident = Ident {
                name: name.clone(),
                span,
            };

            let id = hir.context.definitions.alloc_with_id(|id| Def {
                id,
                span: ident.span,
                ident,
                parent: None,
                annotations: vec![],
                kind: DefKind::Module(ModuleTy {
                    definitions: vec![],
                }),
                flags: DefFlags::IS_SYNTHESIZED,
            });

            hir.order.push(id);
            id
        });

        let def = hir.context.definitions.get_mut(def_id);
        def.parent = Some(module_id);

        let module = hir.context.definitions.get_mut(module_id);
        if let DefKind::Module(module_ty) = &mut module.kind {
            module_ty.definitions.push(def_id);
        }

        moved_defs.insert(def_id);
    }

    (hir, moved_defs)
}
