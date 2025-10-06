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

use std::collections::HashSet;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind, Ty, TyKind};
use ic_vfs::FileId;

pub fn collect_type_dependencies(
    hir: &ResolvedGraph,
    ty: &Ty,
    current_file: FileId,
    deps: &mut HashSet<FileId>,
) {
    match &ty.kind {
        TyKind::Adt(def_id) => {
            let def = hir.context.definitions.get(*def_id);
            let dep_file = def.ident.span.start.file_id;
            if dep_file != current_file {
                deps.insert(dep_file);
            }
        }
        TyKind::Sequence { ty, .. } | TyKind::Array { ty, .. } => {
            collect_type_dependencies(hir, ty, current_file, deps);
        }
        TyKind::Map { key, elem, .. } => {
            collect_type_dependencies(hir, key, current_file, deps);
            collect_type_dependencies(hir, elem, current_file, deps);
        }
        _ => {}
    }
}

pub fn collect_def_dependencies(
    hir: &ResolvedGraph,
    def_id: DefId,
    current_file: FileId,
    deps: &mut HashSet<FileId>,
) {
    let def = hir.context.definitions.get(def_id);

    match &def.kind {
        DefKind::Module(module_ty) => {
            for &child_def_id in &module_ty.definitions {
                collect_def_dependencies(hir, child_def_id, current_file, deps);
            }
        }
        DefKind::Struct(struct_ty) => {
            if let Some(parent) = struct_ty.parent {
                let parent_def = hir.context.definitions.get(parent);
                let parent_file = parent_def.ident.span.start.file_id;
                if parent_file != current_file {
                    deps.insert(parent_file);
                }
            }
            for member in &struct_ty.members {
                collect_type_dependencies(hir, &member.ty, current_file, deps);
            }
        }
        DefKind::Union(union_ty) => {
            collect_type_dependencies(hir, &union_ty.disc.ty, current_file, deps);
            for variant in &union_ty.variants {
                collect_type_dependencies(hir, &variant.ty, current_file, deps);
            }
        }
        DefKind::Interface(interface) => {
            for &parent in &interface.parents {
                let parent_def = hir.context.definitions.get(parent);
                let parent_file = parent_def.ident.span.start.file_id;
                if parent_file != current_file {
                    deps.insert(parent_file);
                }
            }
            for attr in &interface.attributes {
                collect_type_dependencies(hir, &attr.ty, current_file, deps);
            }
            for proto in &interface.prototypes {
                collect_type_dependencies(hir, &proto.ty, current_file, deps);
                for param in &proto.params {
                    collect_type_dependencies(hir, &param.ty, current_file, deps);
                }
            }
        }
        DefKind::Valuetype(valuetype) => {
            if let Some(parent) = valuetype.parent {
                let parent_def = hir.context.definitions.get(parent);
                let parent_file = parent_def.ident.span.start.file_id;
                if parent_file != current_file {
                    deps.insert(parent_file);
                }
            }
            if let Some(supports) = valuetype.supports {
                let supports_def = hir.context.definitions.get(supports);
                let supports_file = supports_def.ident.span.start.file_id;
                if supports_file != current_file {
                    deps.insert(supports_file);
                }
            }
            for member in &valuetype.members {
                collect_type_dependencies(hir, &member.ty, current_file, deps);
            }
        }
        DefKind::Except(except) => {
            for member in &except.members {
                collect_type_dependencies(hir, &member.ty, current_file, deps);
            }
        }
        DefKind::Alias(alias) => {
            collect_type_dependencies(hir, &alias.ty, current_file, deps);
        }
        DefKind::Const(const_ty) => {
            collect_type_dependencies(hir, &const_ty.ty, current_file, deps);
        }
        DefKind::Bitset(bitset) => {
            if let Some(parent) = bitset.parent {
                let parent_def = hir.context.definitions.get(parent);
                let parent_file = parent_def.ident.span.start.file_id;
                if parent_file != current_file {
                    deps.insert(parent_file);
                }
            }
        }
        _ => (),
    }
}
