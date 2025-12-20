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
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    AliasTy, Ann, ConstTy, Def, DefFlags, DefId, DefKind, ExceptTy, InterfaceTy, ModuleTy, Numeric,
    ParamKind, PrimitiveTy, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy,
};

use crate::TypeScriptOptions;

pub struct TsGen<'a> {
    hir: &'a ResolvedGraph,
    options: TypeScriptOptions,
}

impl<'a> TsGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, options: TypeScriptOptions) -> Self {
        Self { hir, options }
    }

    fn ts_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn scope_of(&self, def_id: DefId) -> Option<DefId> {
        let def = self.hir.context.definitions.get(def_id);
        let mut current = def.parent?;

        loop {
            let def = self.hir.context.definitions.get(current);
            if matches!(def.kind, DefKind::Module(_)) {
                return Some(current);
            }
            current = def.parent?;
        }
    }

    fn get_root_module(&self, def_id: DefId) -> Option<DefId> {
        let mut current = def_id;
        loop {
            let def = self.hir.context.definitions.get(current);
            if matches!(def.kind, DefKind::Module(_)) && def.parent.is_none() {
                return Some(current);
            }
            current = def.parent?;
        }
    }

    fn module_ancestors(&self, def_id: DefId) -> Vec<DefId> {
        let mut ancestors = vec![];
        let mut current = Some(def_id);
        while let Some(id) = current {
            let def = self.hir.context.definitions.get(id);
            if matches!(def.kind, DefKind::Module(_)) {
                ancestors.push(id);
            }
            current = def.parent;
        }
        ancestors.reverse();
        ancestors
    }

    fn build_path_from(&self, from_scope: DefId, to_scope: Option<DefId>) -> Vec<String> {
        let mut path = vec![];
        let mut current = from_scope;

        loop {
            if Some(current) == to_scope {
                break;
            }

            let def = self.hir.context.definitions.get(current);
            path.push(def.ident.name.clone());

            match def.parent {
                Some(parent_id) => {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Module(_)) {
                        current = parent_id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        path.reverse();
        path
    }

    fn scoped_name(&self, target_def_id: DefId, relative_to_def_id: DefId) -> String {
        let type_name = self.ts_name(target_def_id);
        let target_scope = self.scope_of(target_def_id);
        let current_scope = self.scope_of(relative_to_def_id);

        match (target_scope, current_scope) {
            (None, None) => type_name.to_string(),
            (None, Some(_)) => format!("types.{type_name}"),
            (Some(target_scope), Some(current_scope)) if target_scope == current_scope => {
                type_name.to_string()
            }
            (Some(target_scope), Some(current_scope)) => {
                // Find common ancestor and build relative path
                let target_ancestors = self.module_ancestors(target_scope);
                let current_ancestors = self.module_ancestors(current_scope);

                // Find the common prefix length
                let common_len = target_ancestors
                    .iter()
                    .zip(current_ancestors.iter())
                    .take_while(|(a, b)| a == b)
                    .count();

                // Build path from the divergence point
                let relative_path: Vec<_> = target_ancestors[common_len..]
                    .iter()
                    .map(|&id| self.hir.context.definitions.get(id).ident.name.clone())
                    .collect();

                if relative_path.is_empty() {
                    // Target is in an ancestor of current scope
                    // We need to reference via the ancestor's name
                    if common_len > 0 && current_ancestors.len() > common_len {
                        // Get the name of the module that contains the target
                        let target_module = self.hir.context.definitions.get(target_scope);
                        format!("{}.{type_name}", target_module.ident.name)
                    } else {
                        type_name.to_string()
                    }
                } else {
                    let pkg_name = relative_path.join(".");
                    format!("{pkg_name}.{type_name}")
                }
            }
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join(".");
                if pkg_name.is_empty() {
                    type_name.to_string()
                } else {
                    format!("{pkg_name}.{type_name}")
                }
            }
        }
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

    fn is_optional(&self, annotations: &[Ann]) -> bool {
        annotations.iter().any(|ann| {
            if let Some(def_id) = ann.def_id {
                let def = self.hir.context.type_of(def_id);
                def.flags.contains(DefFlags::IS_BUILTIN) && def.ident.name == "optional"
            } else {
                false
            }
        })
    }

    fn format_numeric(&self, value: &Numeric, def_id: DefId) -> String {
        match value {
            Numeric::Null | Numeric::Union { .. } => "null".to_string(),
            Numeric::Bool(b) => b.to_string(),
            Numeric::Char(c) => format!("\"{}\"", c.escape_default()),
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
            Numeric::String(s) => format!("\"{}\"", s.escape_default()),
            Numeric::Const(const_def_id) => {
                let const_def = self.hir.context.definitions.get(*const_def_id);
                if let Some(parent_id) = const_def.parent {
                    let parent_def = self.hir.context.definitions.get(parent_id);
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
                // Look up struct definition to get renamed member names
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
            let parent_name = self.scoped_name(parent, def.id);
            w!(w, " extends ", parent_name);
        }
        w!(w, " {\n");

        for member in &struct_ty.members {
            let ty_str = self.ts_type(&member.ty, def.id);
            let optional = if self.is_optional(&member.annotations) {
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
            let member_def = self.hir.context.definitions.get(member_id);
            if let DefKind::Const(const_ty) = &member_def.kind {
                let val = self.format_numeric(&const_ty.value, def.id);
                w!(w, member_def.ident.name, " = ", val, ",\n");
            }
        }

        w!(w, "}\n\n");
    }

    fn emit_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let disc_type = self.ts_type(&union_ty.disc.ty, def.id);

        w!(w, "export type ", def.ident.name, " =\n");

        for (i, variant) in union_ty.variants.iter().enumerate() {
            let variant_type = self.ts_type(&variant.ty, def.id);

            if variant.is_default {
                w!(w, "    | { $discriminator: ", disc_type);
            } else if variant.labels.len() == 1 {
                let label_str = self.format_numeric(&variant.labels[0].value, def.id);
                w!(w, "    | { $discriminator: ", label_str);
            } else {
                let labels: Vec<_> = variant
                    .labels
                    .iter()
                    .map(|l| self.format_numeric(&l.value, def.id))
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

        w!(w, "options?: { cause?: Error }\n");
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
            for (i, &parent_id) in interface_ty.parents.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let parent_name = self.scoped_name(parent_id, def.id);
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
            .filter(|p| p.kind == ParamKind::In || p.kind == ParamKind::Inout)
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
            .filter(|p| p.kind == ParamKind::Out || p.kind == ParamKind::Inout)
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
            let parent_name = self.scoped_name(parent, def.id);
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
            let supports_name = self.scoped_name(supports, def.id);
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
        let ty_str = self.ts_type(&const_ty.ty, def.id);
        let value_str = self.format_numeric(&const_ty.value, def.id);
        w!(w, "export const ", def.ident.name, ": ", ty_str, " = ", value_str, ";\n\n");
    }

    fn emit_non_module_definition(&self, w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);

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
                    def.parent
                        .map(|p| &self.hir.context.definitions.get(p).kind),
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

    fn partition_module_defs(&self, module_ty: &ModuleTy) -> (Vec<DefId>, Vec<DefId>) {
        let mut nested_modules = vec![];
        let mut other_defs = vec![];

        for &def_id in &module_ty.definitions {
            let def = self.hir.context.definitions.get(def_id);
            if matches!(def.kind, DefKind::Module(_)) {
                nested_modules.push(def_id);
            } else {
                other_defs.push(def_id);
            }
        }

        (nested_modules, other_defs)
    }

    fn relative_import_path(&self, from_module: Option<DefId>, to_module: DefId) -> String {
        let from_ancestors = from_module
            .map(|m| self.module_ancestors(m))
            .unwrap_or_default();
        let to_ancestors = self.module_ancestors(to_module);

        let common_len = from_ancestors
            .iter()
            .zip(to_ancestors.iter())
            .take_while(|(a, b)| a == b)
            .count();

        let ups = from_ancestors.len() - common_len;
        let remaining = &to_ancestors[common_len..];

        if ups == 0 && remaining.is_empty() {
            return ".".to_string();
        }

        let mut parts = String::new();
        if ups == 0 {
            parts.push('.');
        } else {
            for i in 0..ups {
                if i > 0 {
                    parts.push('/');
                }
                parts.push_str("..");
            }
        }
        for &id in remaining {
            parts.push('/');
            parts.push_str(&self.hir.context.definitions.get(id).ident.name);
        }

        parts
    }

    fn emit_file(
        &self,
        dir_module: Option<DefId>,
        file_name: &str,
        defs: &[DefId],
        exclude_from_deps: &[DefId],
        re_exports: Option<&[DefId]>,
    ) -> File {
        let mut w = Twine::new();
        Self::emit_header(&mut w);

        if let Some(nested_modules) = re_exports {
            // Import nested modules for use within this file
            for &nested_id in nested_modules {
                let nested_def = self.hir.context.definitions.get(nested_id);
                w!(w, "import * as ", nested_def.ident.name, " from './", nested_def.ident.name, "';\n");
            }
            // Re-export nested modules for external consumers
            for &nested_id in nested_modules {
                let nested_def = self.hir.context.definitions.get(nested_id);
                w!(w, "export * as ", nested_def.ident.name, " from './", nested_def.ident.name, "';\n");
            }
        }

        let referenced = self.collect_deps(defs);
        let mut import_sources: HashSet<Option<DefId>> = referenced
            .iter()
            .map(|&id| self.get_root_module(id))
            .collect();
        for &exclude in exclude_from_deps {
            import_sources.remove(&Some(exclude));
        }
        if file_name == "index.ts" && dir_module.is_none() {
            import_sources.remove(&None);
        }

        let has_re_exports = re_exports.is_some_and(|r| !r.is_empty());
        if !import_sources.is_empty() {
            if has_re_exports {
                w!(w, "\n");
            }
            let ups = dir_module.map_or(0, |m| self.module_ancestors(m).len());
            for &source in &import_sources {
                let (name, import_path) = match source {
                    None => {
                        let path = match ups {
                            0 => ".".to_string(),
                            n => vec![".."; n].join("/"),
                        };
                        ("types".to_string(), path)
                    }
                    Some(module_id) => {
                        let module_def = self.hir.context.definitions.get(module_id);
                        (
                            module_def.ident.name.clone(),
                            self.relative_import_path(dir_module, module_id),
                        )
                    }
                };
                w!(w, "import * as ", name, " from '", import_path, "';\n");
            }
        }

        if (has_re_exports || !import_sources.is_empty()) && !defs.is_empty() {
            w!(w, "\n");
        }

        for &def_id in defs {
            self.emit_non_module_definition(&mut w, def_id);
        }

        let mut path: PathBuf = dir_module
            .map(|m| self.module_ancestors(m))
            .unwrap_or_default()
            .iter()
            .map(|&id| &self.hir.context.definitions.get(id).ident.name)
            .collect();
        path.push(file_name);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn generate_module(&self, module_id: DefId, result: &mut Vec<File>) {
        let def = self.hir.context.definitions.get(module_id);
        let DefKind::Module(module_ty) = &def.kind else {
            return;
        };

        let (nested_modules, non_module_defs) = self.partition_module_defs(module_ty);
        let parent_module = def
            .parent
            .filter(|&p| matches!(self.hir.context.definitions.get(p).kind, DefKind::Module(_)));

        if nested_modules.is_empty() {
            result.push(self.emit_file(
                parent_module,
                &format!("{}.ts", def.ident.name),
                &non_module_defs,
                &[module_id],
                None,
            ));
        } else {
            let mut exclude: Vec<DefId> = vec![module_id];
            exclude.extend(&nested_modules);

            result.push(self.emit_file(
                Some(module_id),
                "index.ts",
                &non_module_defs,
                &exclude,
                Some(&nested_modules),
            ));

            for &nested_id in &nested_modules {
                let nested_def = self.hir.context.definitions.get(nested_id);
                if let DefKind::Module(nested_module_ty) = &nested_def.kind {
                    let (nested_nested, nested_other) =
                        self.partition_module_defs(nested_module_ty);
                    if nested_nested.is_empty() {
                        result.push(self.emit_file(
                            Some(module_id),
                            &format!("{}.ts", nested_def.ident.name),
                            &nested_other,
                            &[nested_id],
                            None,
                        ));
                    } else {
                        self.generate_module(nested_id, result);
                    }
                }
            }
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut result = vec![];

        let mut top_level_defs: Vec<DefId> = vec![];
        let mut top_level_modules: Vec<DefId> = vec![];

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);

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
                "index.ts",
                &top_level_defs,
                &[],
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
