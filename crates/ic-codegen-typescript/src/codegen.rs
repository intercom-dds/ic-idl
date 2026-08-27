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
use std::fmt::Write;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    AliasTy, ConstTy, Def, DefId, DefKind, ExceptTy, InterfaceTy, Numeric, ParamKind, PrimitiveTy,
    ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy,
};
use ic_hir_analysis::annotation::is_optional;

use crate::TypeScriptOptions;
use crate::imports::{self, FileImports, ImportMap};

pub struct TsGen<'a> {
    hir: &'a ResolvedGraph,
    options: TypeScriptOptions,
    imports: ImportMap,
}

impl<'a> TsGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, options: TypeScriptOptions) -> Self {
        let mut generator = Self {
            hir,
            options,
            imports: ImportMap::default(),
        };
        generator.imports =
            imports::collect(hir, &|defs| generator.collect_deps(defs), &|def_id| {
                generator.is_type_only(def_id)
            });
        generator
    }

    fn ts_name(&self, def_id: DefId) -> &str {
        &self.hir.context.type_of(def_id).ident.name
    }

    fn is_type_only(&self, def_id: DefId) -> bool {
        let def = self.hir.context.type_of(def_id);
        match &def.kind {
            DefKind::Struct(_)
            | DefKind::Union(_)
            | DefKind::Except(_)
            | DefKind::Interface(_)
            | DefKind::Valuetype(_)
            | DefKind::Alias(_) => true,
            DefKind::Module(module) => module
                .definitions
                .iter()
                .all(|&child_id| self.is_type_only(child_id)),
            _ => false,
        }
    }

    fn scoped_name(&self, target_def_id: DefId, relative_to_def_id: DefId) -> String {
        let type_name = self.ts_name(target_def_id);
        let target_scope = imports::scope_of(self.hir, target_def_id);
        let file_module = imports::scope_of(self.hir, relative_to_def_id);

        if target_scope == file_module {
            return type_name.to_string();
        }

        let ancestors = target_scope
            .map(|scope| imports::module_ancestors(self.hir, scope))
            .unwrap_or_default();
        let name_of = |&id: &DefId| self.hir.context.type_of(id).ident.name.clone();

        let mut path: Vec<String> = vec![];
        match file_module {
            None => path.extend(ancestors.iter().map(name_of)),
            Some(module_id) => {
                if let Some(index) = ancestors.iter().position(|&id| id == module_id) {
                    path.extend(ancestors[index + 1..].iter().map(name_of));
                } else {
                    let (target, rest) =
                        imports::import_target(self.hir, file_module, target_scope);
                    if let Some(binding) = self.imports.binding(file_module, target) {
                        path.push(binding.to_string());
                    }
                    path.extend(rest.iter().map(name_of));
                }
            }
        }

        path.push(type_name.to_string());
        path.join(".")
    }

    fn collect_deps(&self, def_ids: &[DefId]) -> HashSet<DefId> {
        def_ids
            .iter()
            .flat_map(|&id| self.hir.context.deps(id))
            .collect()
    }

    fn primitive_type(&self, prim: PrimitiveTy) -> &'static str {
        match prim {
            PrimitiveTy::Void => "void",
            PrimitiveTy::Bool => "boolean",
            PrimitiveTy::Char | PrimitiveTy::WChar => "string",
            PrimitiveTy::Int8
            | PrimitiveTy::UInt8
            | PrimitiveTy::Int16
            | PrimitiveTy::UInt16
            | PrimitiveTy::Int32
            | PrimitiveTy::UInt32
            | PrimitiveTy::Float32
            | PrimitiveTy::Float64
            | PrimitiveTy::Float128 => "number",
            PrimitiveTy::Int64 | PrimitiveTy::UInt64 => {
                if self.options.use_bigint {
                    "bigint"
                } else {
                    "number | string"
                }
            }
        }
    }

    fn ts_type(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match &resolved_ty.kind {
            TyKind::Primitive(prim) => self.primitive_type(*prim).to_string(),
            TyKind::String { .. } => "string".to_string(),
            TyKind::Adt(def_id) => self.scoped_name(*def_id, relative_def),
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                let inner = self.ts_type(ty, relative_def);
                if inner.contains('|') {
                    format!("({inner})[]")
                } else {
                    format!("{inner}[]")
                }
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.ts_map_key_type(key, relative_def);
                let elem_ty = self.ts_type(elem, relative_def);
                format!("Record<{key_ty}, {elem_ty}>")
            }
            TyKind::Any => "unknown".to_string(),
            TyKind::Fixed => "number".to_string(),
            TyKind::Null => "void".to_string(),
        }
    }

    /// Returns the TypeScript type for a map key.
    ///
    /// JavaScript object keys are always strings at runtime, so non-string keys
    /// need to use template literal types to maintain type safety:
    /// - `string` → `string` (as-is)
    /// - `boolean` → `` `${boolean}` ``
    /// - numeric types → `` `${number}` `` or `` `${bigint}` ``
    /// - enum types → `` `${EnumType}` ``
    fn ts_map_key_type(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match &resolved_ty.kind {
            // String types can be used directly as keys
            TyKind::String { .. } | TyKind::Primitive(PrimitiveTy::Char | PrimitiveTy::WChar) => {
                "string".to_string()
            }

            // Boolean needs template literal
            TyKind::Primitive(PrimitiveTy::Bool) => "`${boolean}`".to_string(),

            // Numeric types need template literal
            TyKind::Fixed
            | TyKind::Primitive(
                PrimitiveTy::Int8
                | PrimitiveTy::UInt8
                | PrimitiveTy::Int16
                | PrimitiveTy::UInt16
                | PrimitiveTy::Int32
                | PrimitiveTy::UInt32
                | PrimitiveTy::Float32
                | PrimitiveTy::Float64
                | PrimitiveTy::Float128,
            ) => "`${number}`".to_string(),

            // 64-bit integers: bigint or number depending on options
            TyKind::Primitive(PrimitiveTy::Int64 | PrimitiveTy::UInt64) => {
                if self.options.use_bigint {
                    "`${bigint}`".to_string()
                } else {
                    "`${number | string}`".to_string()
                }
            }

            // ADT types: check if it's an enum or bitmask
            TyKind::Adt(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                match &def.kind {
                    // Enums/bitmasks: DDS-JSON prefers string names as keys
                    DefKind::Enum(_) | DefKind::Bitmask(_) => {
                        let name = self.scoped_name(*def_id, relative_def);
                        format!("keyof typeof {name}")
                    }
                    // Structs/unions as keys are serialized to strings (e.g., JSON)
                    DefKind::Struct(_) | DefKind::Union(_) | DefKind::Valuetype(_) => {
                        "string".to_string()
                    }
                    // Aliases: recurse through the alias
                    DefKind::Alias(alias) => self.ts_map_key_type(&alias.ty, relative_def),
                    // Other ADT types - use string as fallback
                    _ => "string".to_string(),
                }
            }

            // Other types - use regular type (will likely be a TS error)
            _ => self.ts_type(ty, relative_def),
        }
    }

    fn format_numeric(&self, value: &Numeric, def_id: DefId) -> String {
        match value {
            Numeric::Null | Numeric::Union { .. } => "null".to_string(),
            Numeric::Bool(b) => b.to_string(),
            Numeric::Char(c) | Numeric::WChar(c) => escape_char(*c),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::Int64(v) => {
                if self.options.use_bigint {
                    format!("{v}n")
                } else {
                    const MAX_SAFE: i64 = 9_007_199_254_740_991;
                    const MIN_SAFE: i64 = -9_007_199_254_740_991;
                    if *v >= MIN_SAFE && *v <= MAX_SAFE {
                        v.to_string()
                    } else {
                        format!("\"{v}\"")
                    }
                }
            }
            Numeric::UInt64(v) => {
                if self.options.use_bigint {
                    format!("{v}n")
                } else {
                    const MAX_SAFE: u64 = 9_007_199_254_740_991;
                    if *v <= MAX_SAFE {
                        v.to_string()
                    } else {
                        format!("\"{v}\"")
                    }
                }
            }
            Numeric::Float(v) => v.to_string(),
            Numeric::Double(v) => v.to_string(),
            Numeric::String(s) | Numeric::WString(s) => escape_str(s),
            Numeric::Const(const_def_id) => {
                let const_def = self.hir.context.type_of(*const_def_id);
                if let Some(parent_id) = const_def.parent {
                    let parent_def = self.hir.context.type_of(parent_id);
                    if matches!(parent_def.kind, DefKind::Enum(_) | DefKind::Bitmask(_)) {
                        let enum_name = self.scoped_name(parent_id, def_id);
                        let const_name = &const_def.ident.name;
                        return format!("{enum_name}.{const_name}");
                    }
                }
                self.scoped_name(*const_def_id, def_id)
            }
            Numeric::Array { values, .. } | Numeric::Sequence { values, .. } => {
                let formatted: Vec<_> = values
                    .iter()
                    .map(|v| self.format_numeric(v, def_id))
                    .collect();
                format!("[{}]", formatted.join(", "))
            }
            Numeric::Map { entries, .. } => {
                let formatted: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "[{}]: {}",
                            self.format_numeric(k, def_id),
                            self.format_numeric(v, def_id),
                        )
                    })
                    .collect();
                format!("{{ {} }}", formatted.join(", "))
            }
            Numeric::Struct { ty, fields } => {
                let struct_def = self.hir.context.type_of(*ty);
                let members = if let DefKind::Struct(s) = &struct_def.kind {
                    &s.members
                } else {
                    return "{}".to_string();
                };

                let formatted: Vec<_> = fields
                    .iter()
                    .zip(members.iter())
                    .map(|(v, member)| {
                        format!("{}: {}", member.ident.name, self.format_numeric(v, def_id))
                    })
                    .collect();
                format!("{{ {} }}", formatted.join(", "))
            }
        }
    }

    fn emit_struct(&self, w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        w!(w, "export interface ", def.ident.name);
        if let Some(parent) = struct_ty.parent {
            let parent_name = self.scoped_name(parent.def_id, def.id);
            w!(w, " extends ", parent_name);
        }
        w!(w, " {\n");

        for member in &struct_ty.members {
            let ty_str = self.ts_type(&member.ty, def.id);
            let optional = if is_optional(&self.hir.context, member) {
                "?"
            } else {
                ""
            };
            w!(w, member.ident.name, optional, ": ", ty_str, ";\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_enum_like(&self, w: &mut Twine, def: &Def, members: &[DefId]) {
        w!(w, "export enum ", def.ident.name, " {\n");

        for &member_id in members {
            let member_def = self.hir.context.type_of(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = self.format_numeric(&const_ty.value, def.id);
                w!(w, member_def.ident.name, " = ", val, ",\n");
            }
        }

        w!(w, "}\n\n");
    }

    fn numeric_typeof(&self, value: &Numeric, def_id: DefId) -> String {
        let formatted = self.format_numeric(value, def_id);
        if matches!(value, Numeric::Const(_)) {
            format!("typeof {formatted}")
        } else {
            formatted
        }
    }

    fn emit_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let disc_type = self.ts_type(&union_ty.disc.ty, def.id);

        w!(w, "export type ", def.ident.name, " =\n");

        for (i, variant) in union_ty.variants.iter().enumerate() {
            let variant_type = self.ts_type(&variant.ty, def.id);

            if variant.is_default {
                w!(w, "    | { $discriminator: ", disc_type);
            } else {
                let labels: Vec<_> = variant
                    .labels
                    .iter()
                    .map(|l| self.numeric_typeof(&l.value, def.id))
                    .collect();
                w!(w, "    | { $discriminator: ", labels.join(" | "));
            }

            if !matches!(variant.ty.kind, TyKind::Null) {
                w!(w, "; ", variant.ident.name, ": ", variant_type);
            }

            w!(w, " }");

            if i < union_ty.variants.len() - 1 {
                w!(w, "\n");
            } else {
                w!(w, ";\n\n");
            }
        }
    }

    fn emit_exception(&self, w: &mut Twine, def: &Def, except_ty: &ExceptTy) {
        w!(w, "export class ", def.ident.name, " extends Error {\n");

        for member in &except_ty.members {
            let ty_str = self.ts_type(&member.ty, def.id);
            w!(w, member.ident.name, ": ", ty_str, ";\n");
        }

        w!(w, "\nconstructor(\n");

        for member in &except_ty.members {
            let ty_str = self.ts_type(&member.ty, def.id);
            w!(w, member.ident.name, ": ", ty_str, ",\n");
        }

        w!(w, "_options?: { cause?: Error }\n");
        w!(w, ") {\n");
        w!(w, "super('", def.ident.name, "');\n");
        w!(w, "this.name = '", def.ident.name, "';\n");

        for member in &except_ty.members {
            w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
        }

        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_interface(&self, w: &mut Twine, def: &Def, interface_ty: &InterfaceTy) {
        w!(w, "export interface ", def.ident.name);

        if !interface_ty.parents.is_empty() {
            w!(w, " extends ");
            for (i, parent) in interface_ty.parents.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let parent_name = self.scoped_name(parent.def_id, def.id);
                w!(w, parent_name);
            }
        }

        w!(w, " {\n");

        for proto in &interface_ty.prototypes {
            self.emit_prototype(w, def, proto);
        }

        for attr in &interface_ty.attributes {
            let ty_str = self.ts_type(&attr.ty, def.id);
            if attr.is_readonly {
                w!(w, "readonly ");
            }
            w!(w, attr.ident.name, ": ", ty_str, ";\n");
        }

        w!(w, "}\n\n");

        for &nested_id in &interface_ty.definitions {
            self.emit_non_module_definition(w, nested_id);
        }
    }

    fn emit_prototype(&self, w: &mut Twine, def: &Def, proto: &ProtoTy) {
        let ret_ty = self.ts_type(&proto.ty, def.id);

        w!(w, proto.ident.name, "(");

        let in_params: Vec<_> = proto
            .params
            .iter()
            .filter(|p| p.kind == ParamKind::In || p.kind == ParamKind::InOut)
            .collect();

        for (i, param) in in_params.iter().enumerate() {
            if i > 0 {
                w!(w, ", ");
            }
            let param_ty = self.ts_type(&param.ty, def.id);
            w!(w, param.ident.name, ": ", param_ty);
        }

        w!(w, "): ");

        let out_params: Vec<_> = proto
            .params
            .iter()
            .filter(|p| p.kind == ParamKind::Out || p.kind == ParamKind::InOut)
            .collect();

        let has_return = !matches!(proto.ty.kind, TyKind::Primitive(PrimitiveTy::Void));

        if out_params.is_empty() {
            w!(w, ret_ty);
        } else if !has_return && out_params.len() == 1 {
            let param_ty = self.ts_type(&out_params[0].ty, def.id);
            w!(w, param_ty);
        } else {
            w!(w, "{ ");
            if has_return {
                w!(w, "$return: ", ret_ty);
                if !out_params.is_empty() {
                    w!(w, "; ");
                }
            }
            for (i, param) in out_params.iter().enumerate() {
                if i > 0 {
                    w!(w, "; ");
                }
                let param_ty = self.ts_type(&param.ty, def.id);
                w!(w, param.ident.name, ": ", param_ty);
            }
            w!(w, " }");
        }

        w!(w, ";\n");
    }

    fn emit_valuetype(&self, w: &mut Twine, def: &Def, value_ty: &ValueTy) {
        w!(w, "export interface ", def.ident.name, "Data");
        if let Some(parent) = value_ty.parent {
            let parent_id = self.hir.context.base_id_of(parent.def_id);
            let parent_name = self.scoped_name(parent_id, def.id);
            w!(w, " extends ", parent_name, "Data");
        }
        w!(w, " {\n");

        for member in &value_ty.members {
            let ty_str = self.ts_type(&member.ty, def.id);
            w!(w, member.ident.name, ": ", ty_str, ";\n");
        }

        for attr in &value_ty.attributes {
            let ty_str = self.ts_type(&attr.ty, def.id);
            w!(w, attr.ident.name, ": ", ty_str, ";\n");
        }

        w!(w, "}\n\n");

        w!(w, "export interface ", def.ident.name, " extends ", def.ident.name, "Data");
        if let Some(supports) = value_ty.supports {
            let supports_name = self.scoped_name(supports.def_id, def.id);
            w!(w, ", ", supports_name);
        }
        w!(w, " {\n");

        for proto in &value_ty.prototypes {
            self.emit_prototype(w, def, proto);
        }

        w!(w, "}\n\n");
    }

    fn emit_alias(&self, w: &mut Twine, def: &Def, alias: &AliasTy) {
        let ty_str = self.ts_type(&alias.ty, def.id);
        w!(w, "export type ", def.ident.name, " = ", ty_str, ";\n\n");
    }

    fn emit_const(&self, w: &mut Twine, def: &Def, const_ty: &ConstTy) {
        let value_str = self.format_numeric(&const_ty.value, def.id);
        w!(w, "export const ", def.ident.name, " = ", value_str);
        if !matches!(const_ty.value, Numeric::Const(_)) {
            w!(w, " as const");
        }
        w!(w, ";\n\n");
    }

    fn emit_non_module_definition(&self, w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.type_of(def_id);

        match &def.kind {
            DefKind::Struct(struct_ty) => self.emit_struct(w, def, struct_ty),
            DefKind::Union(union_ty) => self.emit_union(w, def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum_like(w, def, &enum_ty.fields),
            DefKind::Interface(interface_ty) => self.emit_interface(w, def, interface_ty),
            DefKind::Valuetype(valuetype) => self.emit_valuetype(w, def, valuetype),
            DefKind::Except(except_ty) => self.emit_exception(w, def, except_ty),
            DefKind::Alias(alias) => self.emit_alias(w, def, alias),
            DefKind::Const(const_ty) => {
                let is_enum_member = matches!(
                    def.parent.map(|p| &self.hir.context.type_of(p).kind),
                    Some(DefKind::Enum(_) | DefKind::Bitmask(_))
                );
                if !is_enum_member {
                    self.emit_const(w, def, const_ty);
                }
            }
            DefKind::Bitmask(bitmask_ty) => self.emit_enum_like(w, def, &bitmask_ty.flags),
            DefKind::Module(_) | DefKind::Bitset(_) | DefKind::Annotation(_) | DefKind::Decl(_) => {
            }
        }
    }

    fn emit_re_exports(
        &self,
        w: &mut Twine,
        re_exports: &[DefId],
        referenced: &[DefId],
        dir_module: Option<DefId>,
    ) {
        let is_in_module = |def_id: DefId, module_id: DefId| -> bool {
            imports::module_ancestors(self.hir, def_id).contains(&module_id)
        };

        for &nested_id in re_exports {
            let nested_def = self.hir.context.type_of(nested_id);
            let nested_path = imports::relative_path(self.hir, dir_module, nested_id);
            let refs_in_module: Vec<DefId> = referenced
                .iter()
                .copied()
                .filter(|&ref_id| is_in_module(ref_id, nested_id))
                .collect();

            if refs_in_module.is_empty() {
                w!(w, "export * as ", nested_def, " from \"", nested_path, "\";\n");
            } else {
                let all_types = self.is_type_only(nested_id)
                    && refs_in_module.iter().all(|&id| self.is_type_only(id));

                if all_types {
                    w!(w, "import type * as ", nested_def, " from \"", nested_path, "\";\n");
                    w!(w, "export type { ", nested_def, " };\n");
                } else {
                    w!(w, "import * as ", nested_def, " from \"", nested_path, "\";\n");
                    w!(w, "export { ", nested_def, " };\n");
                }
            }
        }
    }

    fn emit_imports(w: &mut Twine, imports: &FileImports) {
        for import in imports.values() {
            if import.type_only {
                w!(w, "import type * as ", import.binding, " from \"", import.path, "\";\n");
            } else {
                w!(w, "import * as ", import.binding, " from \"", import.path, "\";\n");
            }
        }
    }

    fn emit_file(
        &self,
        file_module: Option<DefId>,
        file_stem: &str,
        defs: &[DefId],
        re_exports: Option<&[DefId]>,
    ) -> File {
        let mut w = Twine::with_indent(2);
        Self::emit_header(&mut w);

        let dir_module = imports::dir_module_of(self.hir, file_module);
        let referenced: Vec<DefId> = self.collect_deps(defs).into_iter().collect();

        if let Some(nested_modules) = re_exports {
            self.emit_re_exports(&mut w, nested_modules, &referenced, dir_module);
        }

        let empty = FileImports::new();
        let imports = self.imports.of(file_module).unwrap_or(&empty);

        let has_re_exports = re_exports.is_some_and(|r| !r.is_empty());
        if !imports.is_empty() {
            if has_re_exports {
                w!(w, "\n");
            }
            Self::emit_imports(&mut w, imports);
        }

        if (has_re_exports || !imports.is_empty()) && !defs.is_empty() {
            w!(w, "\n");
        }

        for &def_id in defs {
            self.emit_non_module_definition(&mut w, def_id);
        }

        if defs.is_empty() && !has_re_exports && imports.is_empty() {
            w!(w, "export {}\n");
        }

        let mut path: PathBuf = dir_module
            .map(|m| imports::module_ancestors(self.hir, m))
            .unwrap_or_default()
            .iter()
            .map(|&id| imports::module_file_stem(self.hir, id))
            .collect();

        path.push(file_stem);
        path.set_extension("ts");
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn generate_module(&self, module_id: DefId, result: &mut Vec<File>) {
        let def = self.hir.context.type_of(module_id);
        let DefKind::Module(module_ty) = &def.kind else {
            return;
        };

        let (nested_modules, non_module_defs) = imports::partition_module_defs(self.hir, module_ty);

        if nested_modules.is_empty() {
            result.push(self.emit_file(
                Some(module_id),
                &imports::module_file_stem(self.hir, module_id),
                &non_module_defs,
                None,
            ));
        } else {
            result.push(self.emit_file(
                Some(module_id),
                imports::BARREL_STEM,
                &non_module_defs,
                Some(&nested_modules),
            ));

            for &nested_id in &nested_modules {
                self.generate_module(nested_id, result);
            }
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut result = vec![];

        let mut top_level_defs: Vec<DefId> = vec![];
        let mut top_level_modules: Vec<DefId> = vec![];

        for &def_id in &self.hir.order {
            let def = self.hir.context.type_of(def_id);

            if matches!(def.kind, DefKind::Module(_)) {
                if def.parent.is_none() {
                    top_level_modules.push(def_id);
                }
            } else if def.parent.is_none() {
                top_level_defs.push(def_id);
            }
        }

        for module_id in &top_level_modules {
            self.generate_module(*module_id, &mut result);
        }

        if !top_level_defs.is_empty() || !top_level_modules.is_empty() {
            result.push(self.emit_file(
                None,
                imports::BARREL_STEM,
                &top_level_defs,
                Some(&top_level_modules),
            ));
        }

        result
    }

    fn emit_header(w: &mut Twine) {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");
        w!(w, "// @generated by ic-idl ", IC_VERSION, "\n\n");
    }
}

fn write_escaped_char<W: Write>(w: &mut W, c: char) -> std::fmt::Result {
    match c {
        '\0' => w.write_str("\\0"),
        '\x08' => w.write_str("\\b"),
        '\t' => w.write_str("\\t"),
        '\n' => w.write_str("\\n"),
        '\x0B' => w.write_str("\\v"),
        '\x0C' => w.write_str("\\f"),
        '\r' => w.write_str("\\r"),
        '"' => w.write_str("\\\""),
        '\'' => w.write_str("\\'"),
        '\\' => w.write_str("\\\\"),
        c if c.is_ascii_graphic() || c == ' ' => w.write_char(c),
        c if c.is_ascii() => write!(w, "\\x{:02X}", c as u32),
        c => {
            let code = c as u32;
            if code <= 0xFFFF {
                write!(w, "\\u{code:04X}")
            } else {
                // ES6+ unicode code point escape
                write!(w, "\\u{{{code:X}}}")
            }
        }
    }
}

fn escape_char(c: char) -> String {
    let mut result = String::new();
    result.push('\'');
    _ = write_escaped_char(&mut result, c);
    result.push('\'');
    result
}

fn escape_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    result.push('"');
    for c in s.chars() {
        _ = write_escaped_char(&mut result, c);
    }
    result.push('"');
    result
}
