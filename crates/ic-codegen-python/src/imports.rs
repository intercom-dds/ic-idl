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

use std::collections::{BTreeMap, BTreeSet};

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefFlags, DefId, DefKind};

use crate::py;
use crate::types::needs_decimal;
use crate::writer::PyWriter;

#[derive(Default)]
pub struct Stdlib {
    pub dataclasses: bool,
    pub enum_: bool,
    pub typing: bool,
    pub abc: bool,
    pub decimal: bool,
}

#[derive(Default)]
pub struct Imports {
    pub stdlib: Stdlib,
    pub types: BTreeMap<String, BTreeSet<String>>,
}

impl Imports {
    pub fn emit(&self, w: &mut PyWriter) {
        let mut has_imports = false;

        if self.stdlib.abc {
            py!(w, "import abc as _abc_\n");
            has_imports = true;
        }

        if self.stdlib.dataclasses {
            py!(w, "import dataclasses as _dataclasses_\n");
            has_imports = true;
        }

        if self.stdlib.decimal {
            py!(w, "import decimal as _decimal_\n");
            has_imports = true;
        }

        if self.stdlib.enum_ {
            py!(w, "import enum as _enum_\n");
            has_imports = true;
        }

        if self.stdlib.typing {
            py!(w, "import typing as _typing_\n");
            has_imports = true;
        }

        for (module, names) in &self.types {
            let names_str = names.iter().cloned().collect::<Vec<_>>().join(", ");
            py!(w, "from ", module, " import ", names_str, "\n");
            has_imports = true;
        }

        if has_imports {
            py!(w, "\n");
        }
    }
}

pub fn collect_imports(
    hir: &ResolvedGraph,
    defs: &[DefId],
    current_module: &[String],
    module_path_fn: impl Fn(DefId) -> Vec<String>,
) -> Imports {
    let mut imports = Imports::default();

    for &def_id in defs {
        collect_stdlib_imports(hir, def_id, &mut imports);
        collect_type_imports(hir, def_id, current_module, &module_path_fn, &mut imports);
    }

    imports
}

#[allow(clippy::too_many_lines)]
fn collect_stdlib_imports(hir: &ResolvedGraph, def_id: DefId, imports: &mut Imports) {
    let def = hir.context.definitions.get(def_id);

    match &def.kind {
        DefKind::Struct(struct_ty) => {
            imports.stdlib.dataclasses = true;
            for member in &struct_ty.members {
                if needs_decimal(hir, &member.ty) {
                    imports.stdlib.decimal = true;
                }
            }
        }
        DefKind::Union(union_ty) => {
            imports.stdlib.dataclasses = true;
            if needs_decimal(hir, &union_ty.disc.ty) {
                imports.stdlib.decimal = true;
            }
            for variant in &union_ty.variants {
                if needs_decimal(hir, &variant.ty) {
                    imports.stdlib.decimal = true;
                }
            }
        }
        DefKind::Except(except_ty) => {
            imports.stdlib.dataclasses = true;
            for member in &except_ty.members {
                if needs_decimal(hir, &member.ty) {
                    imports.stdlib.decimal = true;
                }
            }
        }
        DefKind::Enum(_) | DefKind::Bitmask(_) => {
            imports.stdlib.enum_ = true;
        }
        DefKind::Alias(alias_ty) => {
            imports.stdlib.typing = true;
            if needs_decimal(hir, &alias_ty.ty) {
                imports.stdlib.decimal = true;
            }
        }
        DefKind::Const(const_ty) => {
            if needs_decimal(hir, &const_ty.ty) {
                imports.stdlib.decimal = true;
            }
            if let Some(parent) = &def.parent
                && !matches!(hir.context.type_of(*parent).kind, DefKind::Enum(_))
            {
                imports.stdlib.typing = true;
            }
        }
        DefKind::Interface(interface_ty) => {
            imports.stdlib.abc = true;
            for attr in &interface_ty.attributes {
                if needs_decimal(hir, &attr.ty) {
                    imports.stdlib.decimal = true;
                }
            }
            for proto in &interface_ty.prototypes {
                if needs_decimal(hir, &proto.ty) {
                    imports.stdlib.decimal = true;
                }
                for param in &proto.params {
                    if needs_decimal(hir, &param.ty) {
                        imports.stdlib.decimal = true;
                    }
                }
            }
            for &nested_id in &interface_ty.definitions {
                collect_stdlib_imports(hir, nested_id, imports);
            }
        }
        DefKind::Valuetype(value_ty) => {
            imports.stdlib.dataclasses = true;
            if value_ty.parent.is_none()
                && value_ty.supports.is_none()
                && !value_ty.prototypes.is_empty()
            {
                imports.stdlib.abc = true;
            }
            if !value_ty.prototypes.is_empty() || !value_ty.attributes.is_empty() {
                imports.stdlib.abc = true;
            }
            for member in &value_ty.members {
                if needs_decimal(hir, &member.ty) {
                    imports.stdlib.decimal = true;
                }
            }
            for attr in &value_ty.attributes {
                if needs_decimal(hir, &attr.ty) {
                    imports.stdlib.decimal = true;
                }
            }
            for proto in &value_ty.prototypes {
                if needs_decimal(hir, &proto.ty) {
                    imports.stdlib.decimal = true;
                }
                for param in &proto.params {
                    if needs_decimal(hir, &param.ty) {
                        imports.stdlib.decimal = true;
                    }
                }
            }
            for &nested_id in &value_ty.definitions {
                collect_stdlib_imports(hir, nested_id, imports);
            }
        }
        _ => {}
    }
}

fn collect_type_imports(
    hir: &ResolvedGraph,
    def_id: DefId,
    current_module: &[String],
    module_path_fn: &impl Fn(DefId) -> Vec<String>,
    imports: &mut Imports,
) {
    for dep_id in hir.context.deps(def_id) {
        let dep_def = hir.context.definitions.get(dep_id);

        if dep_def.flags.contains(DefFlags::IS_BUILTIN) {
            continue;
        }

        if matches!(
            dep_def.kind,
            DefKind::Module(_) | DefKind::Annotation(_) | DefKind::Const(_)
        ) {
            continue;
        }

        let dep_module = module_path_fn(dep_id);
        if dep_module != current_module {
            let module_str = if dep_module.is_empty() {
                ".".to_string()
            } else {
                format!(".{}", dep_module.join("."))
            };
            imports
                .types
                .entry(module_str)
                .or_default()
                .insert(dep_def.ident.name.clone());
        }
    }
}
