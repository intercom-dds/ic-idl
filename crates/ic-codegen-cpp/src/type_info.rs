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

#![allow(clippy::cast_possible_wrap)]

use ic_emit::printer::{Twine, w};
use ic_hir::hir::{Def, DefId, DefKind, Member, PrimitiveTy, Ty, TyKind, UnionTy};

use crate::codegen::CppGen;

struct TypeInfo {
    name: String,
    kind: String,
    flags: String,
    bit_size: usize,
    offset: usize,
    max_length: usize,
    default_value: Option<String>,
    min_value: Option<String>,
    max_value: Option<String>,
    key_type: Option<String>,
    element_type: Option<String>,
    member_count: usize,
    members: Option<String>,
}

impl TypeInfo {
    fn emit(&self, w: &mut Twine) {
        w!(w, "{\n");
        w!(w, "\"", self.name, "\", ", self.kind, ", ");
        w!(w, self.flags, ", ", self.bit_size, ", ", self.offset, ", ", self.max_length, ", ");
        w!(w, self.default_value.as_deref().unwrap_or("nullptr"), ", ");
        w!(w, self.min_value.as_deref().unwrap_or("nullptr"), ", ");
        w!(w, self.max_value.as_deref().unwrap_or("nullptr"), ", ");
        w!(w, self.key_type.as_deref().unwrap_or("nullptr"), ", ");
        w!(w, self.element_type.as_deref().unwrap_or("nullptr"), ", ");
        w!(w, self.member_count, ", ", self.members.as_deref().unwrap_or("nullptr"));
        w!(w, "\n}");
    }
}

fn is_key(member: &Member) -> bool {
    member.annotations.iter().any(|a| a.ident.name == "key")
}

fn is_optional(member: &Member) -> bool {
    CppGen::is_optional(member)
}

fn is_shared(member: &Member) -> bool {
    member
        .annotations
        .iter()
        .any(|a| a.ident.name == "shared" || a.ident.name == "external")
}

fn is_must_understand(member: &Member) -> bool {
    member
        .annotations
        .iter()
        .any(|a| a.ident.name == "must_understand")
}

fn is_nested(def: &Def) -> bool {
    def.annotations.iter().any(|a| a.ident.name == "nested")
}

fn default_value_of(def: &Def, ctx: &ic_hir::Context) -> i64 {
    match &def.kind {
        DefKind::Enum(e) => {
            if let Some(&first_field_id) = e.fields.first() {
                let field_def = ctx.definitions.get(first_field_id);
                if let DefKind::Const(c) = &field_def.kind {
                    return ctx.integer_value(&c.value);
                }
            }
            0
        }
        _ => 0,
    }
}

fn extensibility(def: &Def) -> &'static str {
    let is_final = def.annotations.iter().any(|a| a.ident.name == "final");
    let is_mutable = def.annotations.iter().any(|a| a.ident.name == "mutable");

    if is_final {
        "::ic_cts::dcps::xtypes::IS_FINAL"
    } else if is_mutable {
        "::ic_cts::dcps::xtypes::IS_MUTABLE"
    } else {
        "::ic_cts::dcps::xtypes::IS_APPENDABLE"
    }
}

fn is_autoid_hash(ctx: &ic_hir::Context, def: &Def) -> bool {
    def.annotations.iter().any(|a| {
        a.ident.name == "autoid"
            && a.args.first().is_some_and(|arg| {
                if let ic_hir::hir::Numeric::Const(def_id) = &arg.value {
                    ctx.type_of(*def_id).ident.name == "HASH"
                } else {
                    false
                }
            })
    })
}

fn primitive_bit_size(ty: PrimitiveTy) -> usize {
    match ty {
        PrimitiveTy::Void => 0,
        PrimitiveTy::Bool | PrimitiveTy::Int8 | PrimitiveTy::UInt8 | PrimitiveTy::Char => 8,
        PrimitiveTy::Int16 | PrimitiveTy::UInt16 | PrimitiveTy::WChar => 16,
        PrimitiveTy::Int32 | PrimitiveTy::UInt32 | PrimitiveTy::Float32 => 32,
        PrimitiveTy::Int64 | PrimitiveTy::UInt64 | PrimitiveTy::Float64 => 64,
        PrimitiveTy::Float128 => 128,
    }
}

fn primitive_type_info(ty: PrimitiveTy) -> &'static str {
    match ty {
        PrimitiveTy::Void => "nullptr",
        PrimitiveTy::Bool => "::ic_cts::BOOLEAN_TYPE_INFO",
        PrimitiveTy::Int8 => "::ic_cts::INT8_TYPE_INFO",
        PrimitiveTy::UInt8 => "::ic_cts::UINT8_TYPE_INFO",
        PrimitiveTy::Int16 => "::ic_cts::SHORT_TYPE_INFO",
        PrimitiveTy::UInt16 => "::ic_cts::USHORT_TYPE_INFO",
        PrimitiveTy::Int32 => "::ic_cts::LONG_TYPE_INFO",
        PrimitiveTy::UInt32 => "::ic_cts::ULONG_TYPE_INFO",
        PrimitiveTy::Int64 => "::ic_cts::LONGLONG_TYPE_INFO",
        PrimitiveTy::UInt64 => "::ic_cts::ULONGLONG_TYPE_INFO",
        PrimitiveTy::Float32 => "::ic_cts::FLOAT_TYPE_INFO",
        PrimitiveTy::Float64 => "::ic_cts::DOUBLE_TYPE_INFO",
        PrimitiveTy::Float128 => "::ic_cts::LONG_DOUBLE_TYPE_INFO",
        PrimitiveTy::Char => "::ic_cts::CHAR_TYPE_INFO",
        PrimitiveTy::WChar => "::ic_cts::CHAR16_TYPE_INFO",
    }
}

fn type_kind_name(def_kind: &DefKind) -> &'static str {
    match def_kind {
        DefKind::Struct(_) => "::ic_cts::dcps::xtypes::TK_STRUCTURE",
        DefKind::Union(_) => "::ic_cts::dcps::xtypes::TK_UNION",
        DefKind::Enum(_) => "::ic_cts::dcps::xtypes::TK_ENUM",
        DefKind::Bitmask(_) => "::ic_cts::dcps::xtypes::TK_BITMASK",
        _ => "",
    }
}

fn ty_kind_name(ty: &TyKind) -> &'static str {
    match ty {
        TyKind::String { wide: false, .. } => "::ic_cts::dcps::xtypes::TK_STRING8",
        TyKind::String { wide: true, .. } => "::ic_cts::dcps::xtypes::TK_STRING16",
        TyKind::Sequence { .. } => "::ic_cts::dcps::xtypes::TK_SEQUENCE",
        TyKind::Array { .. } => "::ic_cts::dcps::xtypes::TK_ARRAY",
        TyKind::Primitive(p) => match p {
            PrimitiveTy::Void => "::ic_cts::dcps::xtypes::TK_NONE",
            PrimitiveTy::Bool => "::ic_cts::dcps::xtypes::TK_BOOLEAN",
            PrimitiveTy::Int8 => "::ic_cts::dcps::xtypes::TK_INT8",
            PrimitiveTy::UInt8 => "::ic_cts::dcps::xtypes::TK_UINT8",
            PrimitiveTy::Int16 => "::ic_cts::dcps::xtypes::TK_INT16",
            PrimitiveTy::UInt16 => "::ic_cts::dcps::xtypes::TK_UINT16",
            PrimitiveTy::Int32 => "::ic_cts::dcps::xtypes::TK_INT32",
            PrimitiveTy::UInt32 => "::ic_cts::dcps::xtypes::TK_UINT32",
            PrimitiveTy::Int64 => "::ic_cts::dcps::xtypes::TK_INT64",
            PrimitiveTy::UInt64 => "::ic_cts::dcps::xtypes::TK_UINT64",
            PrimitiveTy::Float32 => "::ic_cts::dcps::xtypes::TK_FLOAT32",
            PrimitiveTy::Float64 => "::ic_cts::dcps::xtypes::TK_FLOAT64",
            PrimitiveTy::Float128 => "::ic_cts::dcps::xtypes::TK_FLOAT128",
            PrimitiveTy::Char => "::ic_cts::dcps::xtypes::TK_CHAR8",
            PrimitiveTy::WChar => "::ic_cts::dcps::xtypes::TK_CHAR16",
        },
        _ => "",
    }
}

fn string_type_name(wide: bool) -> &'static str {
    if wide { "wstring" } else { "string" }
}

fn string_element_type_info(wide: bool) -> String {
    let char_type = if wide { "CHAR16" } else { "CHAR" };
    format!("&::ic_cts::{char_type}_TYPE_INFO")
}

fn add_flag(flag: &mut String, value: &str) {
    if flag.is_empty() || flag == "0" {
        *flag = format!("uint32_t({value})");
    } else {
        flag.push_str(" | uint32_t(");
        flag.push_str(value);
        flag.push(')');
    }
}

fn member_flags(member: &Member, has_key: bool) -> String {
    let mut flag = String::new();

    if is_key(member) {
        add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_KEY");
    }
    if is_optional(member) {
        add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_OPTIONAL");
    }
    if is_shared(member) {
        add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_EXTERNAL");
    }
    if is_must_understand(member) {
        add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_MUST_UNDERSTAND");
    }
    if !has_key {
        add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_IMPLICIT_KEY");
    }

    if flag.is_empty() {
        "0".to_string()
    } else {
        flag
    }
}

fn type_flags(ctx: &ic_hir::Context, def: &Def) -> String {
    let mut flag = String::new();
    match &def.kind {
        DefKind::Struct(_) | DefKind::Union(_) | DefKind::Except(_) => {
            add_flag(&mut flag, extensibility(def));

            if is_nested(def) {
                add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_NESTED");
            }
            if is_autoid_hash(ctx, def) {
                add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_AUTOID_HASH");
            }
        }
        _ => {}
    }

    if flag.is_empty() {
        "0".to_string()
    } else {
        flag
    }
}

fn emit_member_info(
    w: &mut Twine,
    index: usize,
    name: &str,
    flags: &str,
    case_labels: &str,
    type_info: &str,
    default_value: &str,
) {
    w!(w, "{ ", index, ", \"", name, "\", ", flags, ", ");
    w!(w, case_labels, ", ", type_info, ", ", default_value, " },\n");
}

impl CppGen<'_> {
    fn mangled_name(&self, def_id: DefId) -> String {
        self.scoped_name(def_id, None).replace("::", "__")
    }

    fn type_name(&self, ty: &Ty) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Primitive(p) => crate::codegen::cpp_primitive(*p).to_string(),
            TyKind::String { wide, .. } => string_type_name(*wide).to_string(),
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                def.ident.name.clone()
            }
            TyKind::Sequence { ty, bound, .. } => {
                let elem = self.type_name(ty);
                if let Some(b) = bound {
                    format!("sequence<{elem},{b}>")
                } else {
                    format!("sequence<{elem}>")
                }
            }
            TyKind::Array { ty, len, .. } => {
                let elem = self.type_name(ty);
                format!("{elem}[{len}]")
            }
            _ => "unknown".to_string(),
        }
    }

    fn type_info_ref(&self, ty: &Ty) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Adt(def_id) => {
                let type_name = self.scoped_name(*def_id, None);
                format!("&::ic_cts::TypeTraits<{type_name}>::type_info")
            }
            TyKind::Primitive(p) => format!("&{}", primitive_type_info(*p)),
            _ => "nullptr".to_string(),
        }
    }

    fn emit_nested_type_info(
        &self,
        w: &mut Twine,
        ty: &Ty,
        var_name: &str,
        parent: &str,
    ) -> String {
        let resolved = self.hir.context.resolve_ty(ty);

        match &resolved.kind {
            TyKind::Primitive(p) => format!("&{}", primitive_type_info(*p)),
            TyKind::Adt(_) => self.type_info_ref(ty),
            TyKind::String { .. } | TyKind::Sequence { .. } | TyKind::Array { .. } => {
                self.emit_type_info(w, ty, var_name, Some(parent))
            }
            _ => "nullptr".to_string(),
        }
    }

    fn emit_type_info(
        &self,
        w: &mut Twine,
        ty: &Ty,
        var_name: &str,
        parent: Option<&str>,
    ) -> String {
        let resolved = self.hir.context.resolve_ty(ty);

        match &resolved.kind {
            TyKind::Primitive(p) => format!("&{}", primitive_type_info(*p)),
            TyKind::Adt(_) => self.type_info_ref(ty),
            TyKind::String { wide, bound, .. } => {
                let base_name = string_type_name(*wide);
                let name = base_name.to_string();

                let info = TypeInfo {
                    name,
                    kind: ty_kind_name(&resolved.kind).to_string(),
                    flags: "0".to_string(),
                    bit_size: 32,
                    offset: 0,
                    max_length: bound.unwrap_or(0),
                    default_value: None,
                    min_value: None,
                    max_value: None,
                    key_type: None,
                    element_type: Some(string_element_type_info(*wide)),
                    member_count: 0,
                    members: None,
                };

                w!(w, "static ::ic_cts::TypeInfo ", var_name, " = ");
                info.emit(w);
                w!(w, ";\n");
                format!("&{var_name}")
            }
            TyKind::Sequence {
                ty: elem_ty, bound, ..
            } => {
                let elem_var = format!("{var_name}_element");
                let elem_info = self.emit_type_info(w, elem_ty, &elem_var, None);

                let base_name = self.type_name(ty);
                let name = if let Some(p) = parent {
                    format!("{p}::{base_name}")
                } else {
                    base_name
                };
                let info = TypeInfo {
                    name,
                    kind: ty_kind_name(&resolved.kind).to_string(),
                    flags: "0".to_string(),
                    bit_size: 32,
                    offset: 0,
                    max_length: bound.unwrap_or(0),
                    default_value: None,
                    min_value: None,
                    max_value: None,
                    key_type: None,
                    element_type: Some(elem_info),
                    member_count: 0,
                    members: None,
                };

                w!(w, "static ::ic_cts::TypeInfo ", var_name, " = ");
                info.emit(w);
                w!(w, ";\n");
                format!("&{var_name}")
            }
            TyKind::Array {
                ty: elem_ty, len, ..
            } => {
                let elem_var = format!("{var_name}_dim_element");
                let elem_info = self.emit_type_info(w, elem_ty, &elem_var, None);

                let base_name = self.type_name(ty);
                let name = if let Some(p) = parent {
                    format!("{p}::{base_name}")
                } else {
                    base_name
                };
                let info = TypeInfo {
                    name,
                    kind: ty_kind_name(&resolved.kind).to_string(),
                    flags: "0".to_string(),
                    bit_size: 0,
                    offset: 0,
                    max_length: *len,
                    default_value: None,
                    min_value: None,
                    max_value: None,
                    key_type: None,
                    element_type: Some(elem_info),
                    member_count: 0,
                    members: None,
                };

                w!(w, "static ::ic_cts::TypeInfo ", var_name, " = ");
                info.emit(w);
                w!(w, ";\n");
                format!("&{var_name}")
            }
            _ => "nullptr".to_string(),
        }
    }

    // Emit struct members with cleaner structure
    fn emit_struct_members(&self, w: &mut Twine, def: &Def, members: &[Member]) {
        if members.is_empty() {
            return;
        }

        let mangled_name = self.mangled_name(def.id);
        let scoped_name = self.scoped_name(def.id, None);
        let has_key = members.iter().any(is_key);

        let mut type_infos = Vec::new();
        for (i, member) in members.iter().enumerate() {
            let var_name = format!("{mangled_name}_type_info_{i}");
            let type_info = self.emit_nested_type_info(w, &member.ty, &var_name, &scoped_name);
            type_infos.push(type_info);
        }

        w!(w, "static ::ic_cts::MemberInfo ", mangled_name, "_members[", members.len(), "] = {\n");
        for (i, member) in members.iter().enumerate() {
            let flags = member_flags(member, has_key);
            emit_member_info(
                w,
                i,
                &member.ident.name,
                &flags,
                "::ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS",
                &type_infos[i],
                "nullptr",
            );
        }
        w!(w, "};\n\n");
    }

    fn emit_union_members(&self, w: &mut Twine, def: &Def, union: &UnionTy) {
        if union.variants.is_empty() {
            return;
        }

        let mangled_name = self.mangled_name(def.id);
        let mut type_infos = Vec::new();
        let mut case_label_names = Vec::new();

        for (i, variant) in union.variants.iter().enumerate() {
            let var_idx = i + 1;
            let var_name = format!("{mangled_name}_type_info_{var_idx}");
            let type_info = self.emit_type_info(w, &variant.ty, &var_name, None);
            type_infos.push(type_info);

            let labels: Vec<i64> = variant
                .labels
                .iter()
                .map(|label| self.hir.context.integer_value(&label.value))
                .collect();

            if labels.is_empty() {
                case_label_names.push("::ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS".to_string());
            } else {
                let label_name = format!("{mangled_name}_labels_{var_idx}");
                w!(w, "static const int32_t ", label_name, "[] = { ", labels.len());
                for label in &labels {
                    w!(w, ", ", label);
                }
                w!(w, " };\n");
                case_label_names.push(label_name);
            }
        }

        let disc_type_info = self.type_info_ref(&union.disc.ty);
        let total_members = union.variants.len() + 1;
        w!(w, "static ::ic_cts::MemberInfo ", mangled_name, "_members[", total_members, "] = {\n");

        let mut disc_flags = String::new();
        add_flag(&mut disc_flags, "::ic_cts::dcps::xtypes::IS_DISCRIMINATOR");
        add_flag(&mut disc_flags, "::ic_cts::dcps::xtypes::IS_IMPLICIT_KEY");

        emit_member_info(
            w,
            0,
            "_d",
            &disc_flags,
            "::ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS",
            &disc_type_info,
            "nullptr",
        );

        for (i, variant) in union.variants.iter().enumerate() {
            let mut flag = String::new();
            add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_IMPLICIT_KEY");
            if variant.is_default {
                add_flag(&mut flag, "::ic_cts::dcps::xtypes::IS_DEFAULT");
            }

            emit_member_info(
                w,
                i + 1,
                &variant.ident.name,
                &flag,
                &case_label_names[i],
                &type_infos[i],
                "nullptr",
            );
        }
        w!(w, " };\n\n");
    }

    fn emit_enum_members(
        &self,
        w: &mut Twine,
        def: &Def,
        fields: &[DefId],
        element_ty: PrimitiveTy,
    ) {
        let name = self.mangled_name(def.id);
        let mut min_val = i64::MAX;
        let mut max_val = i64::MIN;

        for &field_id in fields {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(c) = &field_def.kind {
                let val = self.hir.context.integer_value(&c.value);
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        let type_name = crate::codegen::cpp_primitive(element_ty);
        w!(w, "static ", type_name, " ", name, "_min = ", min_val, ";\n");
        w!(w, "static ", type_name, " ", name, "_max = ", max_val, ";\n");
        w!(w, "static ", type_name, " ", name, "_default = ", default_value_of(def, &self.hir.context), ";\n\n");

        w!(w, "static ::ic_cts::MemberInfo ", name, "_members[", fields.len(), "] = {\n");
        for (i, &field_id) in fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let member_id = if let DefKind::Const(c) = &field_def.kind {
                self.hir.context.integer_value(&c.value)
            } else {
                i as i64
            };

            if member_id >= 0 {
                w!(w, "{ ", member_id, ", \"", field_def.ident.name,
                   "\", 0, nullptr, nullptr, nullptr },");
            } else {
                w!(w, "{ static_cast<uint32_t>(", member_id, "), \"",
                   field_def.ident.name, "\", 0, nullptr, nullptr, nullptr },");
            }
        }
        w!(w, "\n};\n\n");
    }

    fn emit_bitmask_members(
        &self,
        w: &mut Twine,
        def: &Def,
        flags: &[DefId],
        element_ty: PrimitiveTy,
    ) {
        let name = self.mangled_name(def.id);

        let mut max_val: u64 = 0;
        for &flag_id in flags {
            let flag_def = self.hir.context.definitions.get(flag_id);
            if let DefKind::Const(c) = &flag_def.kind {
                let val = self.hir.context.unsigned_value(&c.value);
                max_val |= val;
            }
        }

        let type_name = crate::codegen::cpp_primitive(element_ty);
        w!(w, "static ", type_name, " ", name, "_max = ", max_val, ";\n");
        w!(w, "static ", type_name, " ", name, "_default = ", default_value_of(def, &self.hir.context), ";\n\n");

        w!(w, "static ::ic_cts::MemberInfo ", name, "_members[", flags.len(), "] = {\n");
        for (i, &flag_id) in flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);

            let member_id = if let DefKind::Const(c) = &flag_def.kind {
                let val = self.hir.context.unsigned_value(&c.value);
                // For bitmasks, the member ID is the bit position
                if val > 0 {
                    i64::from(val.trailing_zeros())
                } else {
                    0
                }
            } else {
                i as i64
            };

            w!(w, "{ ", member_id, ", \"", flag_def.ident.name,
               "\", 0, nullptr, nullptr, nullptr },");
        }
        w!(w, " };\n\n");
    }

    pub(crate) fn emit_member_info(&self, w: &mut Twine, def: &Def) {
        match &def.kind {
            DefKind::Struct(_) => {
                let members = self.collect_all_members(def.id);
                self.emit_struct_members(w, def, &members);
            }
            DefKind::Union(u) => self.emit_union_members(w, def, u),
            DefKind::Enum(e) => self.emit_enum_members(w, def, &e.fields, e.ty),
            DefKind::Bitmask(b) => self.emit_bitmask_members(w, def, &b.flags, b.ty),
            _ => {}
        }
    }

    pub(crate) fn emit_type_info_definition(&self, w: &mut Twine, def: &Def) {
        let def_kind = match &def.kind {
            DefKind::Struct(_) | DefKind::Union(_) | DefKind::Enum(_) | DefKind::Bitmask(_) => {
                &def.kind
            }
            _ => return,
        };

        let scoped_name = self.scoped_name(def.id, None);

        let kind = type_kind_name(def_kind).to_string();

        let flags = match def_kind {
            DefKind::Struct(_) | DefKind::Union(_) => type_flags(&self.hir.context, def),
            _ => "0".to_string(),
        };

        let bit_size = match def_kind {
            DefKind::Enum(e) => primitive_bit_size(e.ty),
            DefKind::Bitmask(b) => primitive_bit_size(b.ty),
            _ => 0,
        };

        let member_count = match def_kind {
            DefKind::Struct(_) => self.collect_all_members(def.id).len(),
            DefKind::Union(u) => u.variants.len() + 1,
            DefKind::Enum(e) => e.fields.len(),
            DefKind::Bitmask(b) => b.flags.len(),
            _ => 0,
        };

        let mangled_name = self.mangled_name(def.id);
        let members = if member_count > 0 {
            Some(format!("{mangled_name}_members"))
        } else {
            None
        };

        let (default_value, min_value, max_value, element_type) = match def_kind {
            DefKind::Enum(e) => {
                let prim_info = primitive_type_info(e.ty);
                (
                    Some(format!("&{mangled_name}_default")),
                    Some(format!("&{mangled_name}_min")),
                    Some(format!("&{mangled_name}_max")),
                    Some(format!("&{prim_info}")),
                )
            }
            DefKind::Bitmask(b) => {
                let prim_info = primitive_type_info(b.ty);
                (
                    Some(format!("&{mangled_name}_default")),
                    None,
                    Some(format!("&{mangled_name}_max")),
                    Some(format!("&{prim_info}")),
                )
            }
            _ => (None, None, None, None),
        };

        let info = TypeInfo {
            name: scoped_name.clone(),
            kind: kind.clone(),
            flags,
            bit_size,
            offset: 0,
            max_length: 0,
            default_value,
            min_value,
            max_value,
            key_type: None,
            element_type,
            member_count,
            members,
        };

        w!(w, "const ::ic_cts::TypeInfo ic_cts::TypeTraits<", scoped_name, ">::type_info = ");
        info.emit(w);
        w!(w, ";\n");
    }
}
