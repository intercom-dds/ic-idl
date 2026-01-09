// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

//! Lint that warns about unused `#include` directives.
//!
//! An include is considered "unused" if no items from that file (or any files
//! it transitively includes) are referenced in the HIR.

use std::collections::{HashMap, HashSet};

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, DefFlags, InterfaceTy, StructTy, TyKind, ValueTy};
use ic_hir::visit::Visitor;
use ic_vfs::{FileId, IncludeInfo};

use crate::{Category, Lint, LintCtx};

/// Lint that warns about unused `#include` directives.
///
/// An include is considered unused if the included file (and any files it
/// transitively includes) do not contribute any definitions to the final
/// compilation unit.
pub struct UnusedInclude;

impl<'a> Lint<'a> for UnusedInclude {
    fn name() -> &'static str {
        "unused-include"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Included files that are not used"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let includes = ctx.includes();
        if includes.is_empty() {
            return;
        }

        // Build include hierarchy: which files are included by which
        let include_tree = build_include_tree(includes);

        // Collect all FileIds whose definitions are referenced
        let used_files = collect_used_files(hir);

        for include in includes {
            if !is_include_used(include.included_file, &include_tree, &used_files) {
                let diag = ctx
                    .diag_span(
                        Self::name(),
                        Self::category(),
                        format!("unused include `{}`", include.included_as),
                        Label::new(include.path_span).message("unused include"),
                    )
                    .help("remove this include if it is not needed");
                Self::report(ctx, diag);
            }
        }
    }
}

/// Builds a map from each file to the files it includes (directly).
fn build_include_tree(includes: &[IncludeInfo]) -> HashMap<FileId, Vec<FileId>> {
    let mut tree: HashMap<FileId, Vec<FileId>> = HashMap::new();
    for inc in includes {
        tree.entry(inc.including_file)
            .or_default()
            .push(inc.included_file);
    }
    tree
}

/// Collects all `FileId`s whose definitions are referenced in the HIR.
///
/// A file is considered "used" if any type defined in that file is referenced
/// by code in other files. This is different from just checking if a file
/// has definitions - we check if those definitions are actually used.
fn collect_used_files(hir: &ResolvedGraph) -> HashSet<FileId> {
    let mut collector = UsedFileCollector {
        used_files: HashSet::new(),
        hir,
    };
    ic_hir::visit::walk_tree(&mut collector, hir);
    collector.used_files
}

/// Checks if an included file (or any of its transitive includes) is used.
fn is_include_used(
    file_id: FileId,
    include_tree: &HashMap<FileId, Vec<FileId>>,
    used_files: &HashSet<FileId>,
) -> bool {
    // Use a worklist algorithm to check all transitive includes
    let mut visited = HashSet::new();
    let mut worklist = vec![file_id];

    while let Some(current) = worklist.pop() {
        if !visited.insert(current) {
            continue;
        }

        // If this file is used, the include is considered used
        if used_files.contains(&current) {
            return true;
        }

        // Add files that this file includes to the worklist
        if let Some(children) = include_tree.get(&current) {
            worklist.extend(children.iter().copied());
        }
    }

    false
}

/// Visitor that collects all `FileId`s that are "used" in the HIR.
///
/// A file is "used" when a type defined in that file is referenced.
struct UsedFileCollector<'a> {
    /// Files that are marked as used
    used_files: HashSet<FileId>,
    hir: &'a ResolvedGraph,
}

impl<'a> Visitor<'a> for UsedFileCollector<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_def(&mut self, def: &'a ic_hir::hir::Def) {
        // Skip builtins - they are not from user files
        if def.flags.contains(DefFlags::IS_BUILTIN) {
            return;
        }

        // Continue visiting nested definitions to find type references
        ic_hir::visit::walk_def(self, def);
    }

    fn visit_ty(&mut self, ty: &'a ic_hir::hir::Ty) {
        // When we reference a type, mark the file it came from as used
        if let TyKind::Adt(def_id) = &ty.kind {
            let def = self.hir.context.type_of(*def_id);
            if !def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(def.span.start.file_id);
            }
        }

        ic_hir::visit::walk_ty(self, ty);
    }

    fn visit_annotation(&mut self, ann: &'a Ann) {
        // When an annotation is applied, mark the file containing its definition as used
        if let Some(def_id) = ann.def_id {
            let def = self.hir.context.type_of(def_id);
            if !def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(def.span.start.file_id);
            }
        }

        ic_hir::visit::walk_annotation(self, ann);
    }

    fn visit_struct(&mut self, _def: &'a ic_hir::hir::Def, data: &'a StructTy) {
        // Mark the parent struct's file as used if inheriting
        if let Some(parent_id) = data.parent {
            let parent_def = self.hir.context.type_of(parent_id);
            if !parent_def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(parent_def.span.start.file_id);
            }
        }

        ic_hir::visit::walk_struct(self, data);
    }

    fn visit_interface(&mut self, def: &'a ic_hir::hir::Def, data: &'a InterfaceTy) {
        // Mark parent interfaces' files as used
        for &parent_id in &data.parents {
            let parent_def = self.hir.context.type_of(parent_id);
            if !parent_def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(parent_def.span.start.file_id);
            }
        }

        ic_hir::visit::walk_interface(self, def, data);
    }

    fn visit_valuetype(&mut self, def: &'a ic_hir::hir::Def, data: &'a ValueTy) {
        // Mark the parent valuetype's file as used if inheriting
        if let Some(parent_id) = data.parent {
            let parent_def = self.hir.context.type_of(parent_id);
            if !parent_def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(parent_def.span.start.file_id);
            }
        }

        // Mark the supported interface's file as used if present
        if let Some(supports_id) = data.supports {
            let supports_def = self.hir.context.type_of(supports_id);
            if !supports_def.flags.contains(DefFlags::IS_BUILTIN) {
                self.used_files.insert(supports_def.span.start.file_id);
            }
        }

        ic_hir::visit::walk_valuetype(self, def, data);
    }
}
