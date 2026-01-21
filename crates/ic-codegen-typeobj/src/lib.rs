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

use std::path::PathBuf;

use ic_cli::Command;
use ic_emit::File;
use ic_hir::hir::{Def, DefId, DefKind};
use ic_hir::visit::{Visitor, walk_tree};
use ic_hir::{Context, ResolvedGraph};
use ic_typeobj::TypeObjectCache;
use intercom_cts::json;

#[derive(Command, Copy, Clone, Debug, Default)]
pub struct TypeObjOptions {}

#[must_use]
pub fn codegen_typeobj(hir: &ResolvedGraph, _options: TypeObjOptions) -> Vec<File> {
    let mut visitor = TypeObjVisitor::new(&hir.context);
    walk_tree(&mut visitor, hir);
    visitor.files
}

struct TypeObjVisitor<'a> {
    context: &'a Context,
    cache: TypeObjectCache<'a>,
    files: Vec<File>,
}

impl<'a> TypeObjVisitor<'a> {
    fn new(context: &'a Context) -> Self {
        Self {
            context,
            cache: TypeObjectCache::new(context),
            files: Vec::new(),
        }
    }

    fn module_path(&self, def_id: DefId) -> PathBuf {
        let def = self.context.type_of(def_id);
        let mut parts = vec![];
        let mut current_parent = def.parent;

        while let Some(parent_id) = current_parent {
            let parent_def = self.context.type_of(parent_id);
            if matches!(
                parent_def.kind,
                DefKind::Module(_) | DefKind::Interface(_) | DefKind::Valuetype(_),
            ) {
                parts.push(&parent_def.ident.name);
            }
            current_parent = parent_def.parent;
        }

        parts.reverse();
        PathBuf::from_iter(parts)
    }

    fn file_path(&self, def: &Def) -> PathBuf {
        let mut path = self.module_path(def.id);
        path.push(&def.ident.name);
        path.set_extension("json");
        path
    }
}

fn has_type_obj(def: &Def) -> bool {
    matches!(
        def.kind,
        DefKind::Annotation(_)
            | DefKind::Alias(_)
            | DefKind::Struct(_)
            | DefKind::Union(_)
            | DefKind::Except(_)
            | DefKind::Enum(_)
            | DefKind::Bitmask(_)
            | DefKind::Bitset(_)
    )
}

impl<'a> Visitor<'a> for TypeObjVisitor<'a> {
    fn context(&self) -> &'a Context {
        self.context
    }

    fn visit_def(&mut self, def: &'a Def) {
        if has_type_obj(def) {
            let type_def = ic_typeobj::type_definition(def.id, &mut self.cache);
            let source = json::to_string(&type_def, true).expect("failed to serialize TypeObj");
            let path = self.file_path(def);
            self.files.push(File::Generated { path, source });
        }
        ic_hir::visit::walk_def(self, def);
    }
}
