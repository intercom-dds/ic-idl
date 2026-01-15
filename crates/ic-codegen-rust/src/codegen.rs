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

use std::collections::BTreeMap;

use ic_emit::printer::Twine;
use ic_emit::{File, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefFlags, DefId, DefKind, Numeric, ParamKind, PrimitiveTy, Ty, TyKind};

use crate::RustOptions;
use crate::helpers::{
    default_value, is_copy, is_debug, is_eq, is_hash, is_optional, is_ord, is_trivial,
    rust_primitive,
};

struct Module {
    printer: Twine,
    entries: BTreeMap<String, Module>,
}

impl Module {
    fn new() -> Self {
        Self {
            printer: Twine::new(),
            entries: BTreeMap::new(),
        }
    }

    fn find(&mut self, hir: &ResolvedGraph, def: &Def) -> &mut Twine {
        let mut scope = Vec::new();
        let mut current_id = def.parent;

        while let Some(id) = current_id {
            let parent_def = hir.context.definitions.get(id);
            if matches!(parent_def.kind, DefKind::Module(_)) {
                scope.push(parent_def.ident.name.clone());
            }
            current_id = parent_def.parent;
        }

        let mut curr = self;
        for name in scope.iter().rev() {
            curr = curr.entries.entry(name.clone()).or_insert_with(Module::new);
        }
        &mut curr.printer
    }
}

pub(crate) struct RustGen<'a> {
    pub(crate) hir: &'a ResolvedGraph,
    pub(crate) original_hir: &'a ResolvedGraph,
    pub(crate) options: RustOptions,
}

impl<'a> RustGen<'a> {
    pub fn new(
        hir: &'a ResolvedGraph,
        original_hir: &'a ResolvedGraph,
        options: RustOptions,
    ) -> Self {
        Self {
            hir,
            original_hir,
            options,
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut root = Module::new();

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);
            self.recurse_node(&mut root, def);
        }

        let mut files = Vec::new();
        Self::emit_module(root, &mut files, "");
        files
    }

    fn emit_module(module: Module, files: &mut Vec<File>, path: &str) {
        let mut content = String::new();
        content.push_str("// @generated\n\n");

        for (name, child) in module.entries {
            content.push_str("pub mod ");
            content.push_str(&name);
            content.push_str(";\n");
            let child_path = if path.is_empty() {
                name.clone()
            } else {
                format!("{path}/{name}")
            };
            Self::emit_module(child, files, &child_path);
        }

        if !content.is_empty() && !content.ends_with("\n\n") {
            content.push('\n');
        }

        content.push_str(&module.printer.finish());

        let file_path = if path.is_empty() {
            "lib.rs".to_string()
        } else {
            format!("{path}.rs")
        };

        files.push(File::Generated {
            path: file_path.into(),
            source: content,
        });
    }

    pub(crate) fn rust_type(&self, ty: &Ty, ctx: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        if matches!(resolved_ty.kind, TyKind::String { .. })
            && let ctx_def = self.hir.context.definitions.get(ctx)
            && let DefKind::Const(const_ty) = &ctx_def.kind
            && self.is_string_const_literal(&const_ty.ty, &const_ty.value)
        {
            return "&str".to_string();
        }

        match &ty.kind {
            TyKind::Primitive(prim) => rust_primitive(*prim).to_string(),
            TyKind::String { .. } => "::std::string::String".to_string(),
            TyKind::Sequence { ty, .. } => {
                format!("::std::vec::Vec<{}>", self.rust_type(ty, ctx))
            }
            TyKind::Array { ty, len, .. } => {
                format!("[{}; {}]", self.rust_type(ty, ctx), len)
            }
            TyKind::Map { key, elem, .. } => {
                format!(
                    "::std::collections::BTreeMap<{}, {}>",
                    self.rust_type(key, ctx),
                    self.rust_type(elem, ctx)
                )
            }
            TyKind::Adt(def_id) => self.scoped_name(*def_id, ctx),
            TyKind::Any | TyKind::Fixed | TyKind::Null => "()".to_string(),
        }
    }

    fn emit_struct(&self, def: &Def, struct_ty: &ic_hir::hir::StructTy, w: &mut Twine) {
        self.emit_derives(def, w);
        w!(w, "pub struct ", def, " {\n");

        let members = self.struct_members(struct_ty);
        for member in &members {
            let member_ty = self.member_type(&member.ty);
            let field_ty = if is_optional(member) {
                format!("::std::option::Option<{member_ty}>")
            } else {
                member_ty
            };
            w!(w, "pub ", member.ident.name, ": ", field_ty, ",\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_except(&self, def: &Def, except_ty: &ic_hir::hir::ExceptTy, w: &mut Twine) {
        self.emit_derives(def, w);
        w!(w, "pub struct ", def, " {\n");

        for member in &except_ty.members {
            let member_ty = self.member_type(&member.ty);
            let field_ty = if is_optional(member) {
                format!("::std::option::Option<{member_ty}>")
            } else {
                member_ty
            };
            w!(w, "pub ", member.ident.name, ": ", field_ty, ",\n");
        }
        w!(w, "}\n\n");

        w!(w, "pub type ", def, "Result<T> = ::std::result::Result<T, ", def, ">;\n\n");

        w!(w, "impl ::std::fmt::Display for ", def, " {\n");
        w!(w, "fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {\n");
        w!(w, "write!(f, \"", def, "\")\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl ::std::error::Error for ", def, " {}\n\n");
    }

    fn emit_valuetype(&self, def: &Def, value_ty: &ic_hir::hir::ValueTy, w: &mut Twine) {
        self.emit_derives(def, w);
        w!(w, "pub struct ", def, " {\n");

        let members = self.valuetype_members(value_ty);
        for member in &members {
            let member_ty = self.member_type(&member.ty);
            let field_ty = if is_optional(member) {
                format!("::std::option::Option<{member_ty}>")
            } else {
                member_ty
            };
            w!(w, "pub ", member.ident.name, ": ", field_ty, ",\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_interface(&self, def: &Def, interface_ty: &ic_hir::hir::InterfaceTy, w: &mut Twine) {
        w!(w, "pub trait ", def);

        if !interface_ty.parents.is_empty() {
            w!(w, ": ");
            for (i, &parent_id) in interface_ty.parents.iter().enumerate() {
                let parent_def = self.hir.context.definitions.get(parent_id);
                w!(w, parent_def);
                if i + 1 < interface_ty.parents.len() {
                    w!(w, " + ");
                }
            }
        }

        w!(w, " {");
        for proto in &interface_ty.prototypes {
            w!(w, "\n");
            self.emit_prototype(proto, def.id, w);
        }
        w!(w, "}\n\n");
    }

    fn emit_prototype(&self, proto: &ic_hir::hir::ProtoTy, ctx: DefId, w: &mut Twine) {
        w!(w, "fn ", proto.ident.name, "(");
        let multi = proto.params.len() > 1;
        if multi {
            w!(w, "\n");
        }
        w!(w, "&mut self");
        if !proto.params.is_empty() {
            w!(w, ", ");
        }

        for (i, param) in proto.params.iter().enumerate() {
            if multi {
                w!(w, "\n");
            }
            w!(w, param.ident.name, ": ");
            self.emit_param_type(param, ctx, w);
            if i + 1 < proto.params.len() || multi {
                w!(w, ",");
            }
        }

        if multi {
            w!(w, "\n");
        }
        w!(w, ")");

        if !matches!(proto.ty.kind, TyKind::Primitive(PrimitiveTy::Void))
            || !proto.raises.is_empty()
        {
            w!(w, " -> ");
            self.emit_prototype_return_type(&proto.ty, &proto.raises, ctx, w);
        }

        w!(w, ";\n");
    }

    fn emit_param_type(&self, param: &ic_hir::hir::Parameter, ctx: DefId, w: &mut Twine) {
        // TODO: should be is_trivial(def)
        let is_trivial = matches!(param.ty.kind, TyKind::Primitive(_));

        if !is_trivial || matches!(param.kind, ParamKind::Out | ParamKind::Inout) {
            w!(w, "&");
            if matches!(param.kind, ParamKind::Out | ParamKind::Inout) {
                w!(w, "mut ");
            } else {
                if let TyKind::String { .. } = param.ty.kind {
                    w!(w, "str");
                    return;
                }

                if let TyKind::Sequence { ty, .. } = &param.ty.kind {
                    let elem_ty = self.rust_type(ty, ctx);
                    w!(w, "[", elem_ty, "]");
                    return;
                }
            }

            if let TyKind::Adt(def_id) = param.ty.kind {
                let def = self.hir.context.base_def_of(def_id);
                if matches!(def.kind, DefKind::Interface(_)) {
                    w!(w, "dyn ");
                }
            }
        }

        let ty = self.rust_type(&param.ty, ctx);
        w!(w, ty);
    }

    fn emit_prototype_return_type(&self, ty: &Ty, raises: &[DefId], ctx: DefId, w: &mut Twine) {
        if raises.len() > 1 {
            w!(w, "\n\t");
            w!(w, "::std::result::Result<");
            self.emit_prototype_return_type(ty, &[], ctx, w);
            w!(w, ", ::std::boxed::Box<dyn ::std::error::Error>>");
        } else if !raises.is_empty() {
            let except_name = self.scoped_name(raises[0], ctx);
            w!(w, except_name, "Result<");
            self.emit_prototype_return_type(ty, &[], ctx, w);
            w!(w, ">");
        } else if let TyKind::Adt(def_id) = ty.kind {
            let def = self.hir.context.base_def_of(def_id);
            if matches!(def.kind, DefKind::Interface(_)) {
                w!(w, "Box<dyn ", def, ">");
            } else {
                let ty_str = self.rust_type(ty, ctx);
                w!(w, ty_str);
            }
        } else {
            let ty_str = self.rust_type(ty, ctx);
            w!(w, ty_str);
        }
    }

    fn emit_union(&self, def: &Def, union_ty: &ic_hir::hir::UnionTy, w: &mut Twine) {
        self.emit_derives(def, w);
        w!(w, "pub enum ", def, " {\n");

        for variant in &union_ty.variants {
            if variant.labels.is_empty() {
                w!(w, variant.ident.name);
                if !matches!(variant.ty.kind, TyKind::Null) {
                    let member_ty = self.member_type(&variant.ty);
                    w!(w, "(", member_ty, ")");
                }
                w!(w, ",\n");
            } else {
                for label in &variant.labels {
                    let variant_name = self.union_variant_name(variant, label, union_ty);
                    w!(w, variant_name);
                    if !matches!(variant.ty.kind, TyKind::Null) {
                        let member_ty = self.member_type(&variant.ty);
                        w!(w, "(", member_ty, ")");
                    }
                    w!(w, ",\n");
                }
            }
        }
        w!(w, "}\n\n");
    }

    fn emit_union_impl(&self, def: &Def, union_ty: &ic_hir::hir::UnionTy, w: &mut Twine) {
        w!(w, "impl ", def, " {\n");
        if !self.options.must_use {
            w!(w, "#[must_use]\n");
        }

        w!(w, "pub fn new() -> Self {\n");

        let def_variant = union_ty
            .variants
            .iter()
            .find(|v| v.is_default)
            .unwrap_or(&union_ty.variants[0]);

        let variant_name = if def_variant.labels.is_empty() {
            def_variant.ident.name.clone()
        } else {
            let def_label = &def_variant.labels[0];
            self.union_variant_name(def_variant, def_label, union_ty)
        };
        w!(w, "Self::", variant_name);
        if !matches!(def_variant.ty.kind, TyKind::Null) {
            w!(w, "(");
            self.emit_default_value(&def_variant.ty, def.id, w);
            w!(w, ")");
        }
        w!(w, "\n}\n\n");

        let disc_ty = self.rust_type(&union_ty.disc.ty, def.id);
        if !self.options.must_use {
            w!(w, "#[must_use]\n");
        }
        w!(w, "pub const fn disc(&self) -> ", disc_ty, " {\n");
        w!(w, "match self {\n");
        for variant in &union_ty.variants {
            if variant.labels.is_empty() {
                w!(w, "Self::", variant.ident.name);
                if !matches!(variant.ty.kind, TyKind::Null) {
                    w!(w, "(_)");
                }
                w!(w, " => ");
                self.emit_const_default_value(&union_ty.disc.ty, def.id, w);
                w!(w, ",\n");
            } else {
                for label in &variant.labels {
                    let variant_name = self.union_variant_name(variant, label, union_ty);
                    w!(w, "Self::", variant_name);
                    if !matches!(variant.ty.kind, TyKind::Null) {
                        w!(w, "(_)");
                    }
                    w!(w, " => ");
                    self.emit_const_value(&label.value, &union_ty.disc.ty, def.id, w);
                    w!(w, ",\n");
                }
            }
        }
        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl From<", disc_ty, "> for ", def, " {\n");
        w!(w, "fn from(disc: ", disc_ty, ") -> Self {\n");
        w!(w, "match disc {\n");
        for variant in &union_ty.variants {
            if !variant.is_default {
                for label in &variant.labels {
                    self.emit_const_value(&label.value, &union_ty.disc.ty, def.id, w);
                    w!(w, " => Self::", self.union_variant_name(variant, label, union_ty));
                    if !matches!(variant.ty.kind, TyKind::Null) {
                        w!(w, "(");
                        self.emit_default_value(&variant.ty, def.id, w);
                        w!(w, ")");
                    }
                    w!(w, ",\n");
                }
            }
        }
        if union_ty.variants.iter().any(|v| v.is_default) {
            w!(w, "_ => Self::default(),\n");
        }
        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn union_variant_name(
        &self,
        variant: &ic_hir::hir::Variant,
        label: &ic_hir::hir::Label,
        _union_ty: &ic_hir::hir::UnionTy,
    ) -> String {
        if variant.labels.len() <= 1 {
            return variant.ident.name.clone();
        }

        match &label.value {
            Numeric::Const(def_id) => {
                let label_def = self.hir.context.definitions.get(*def_id);
                label_def.ident.name.clone()
            }
            _ => {
                format!(
                    "{}{}",
                    variant.ident.name,
                    Self::format_numeric(&label.value),
                )
            }
        }
    }

    fn emit_enum(&self, def: &Def, enum_ty: &ic_hir::hir::EnumTy, w: &mut Twine) {
        let repr_ty = rust_primitive(enum_ty.ty);

        self.emit_derives(def, w);
        w!(w, "#[repr(", repr_ty, ")]\n");
        w!(w, "pub enum ", def, " {\n");

        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                w!(w, field_def);
                if field_def.flags.contains(DefFlags::IS_ENUMERATED) {
                    let value = Self::format_numeric(&const_ty.value);
                    w!(w, " = ", value);
                }
                w!(w, ",\n");
            }
        }
        w!(w, "}\n\n");
    }

    fn emit_enum_impl(&self, def: &Def, enum_ty: &ic_hir::hir::EnumTy, w: &mut Twine) {
        w!(w, "impl ", def, " {\n");
        if !self.options.must_use {
            w!(w, "#[must_use]\n");
        }
        w!(w, "pub const fn new() -> Self {\n");
        if let Some(&first_id) = enum_ty.fields.first() {
            let first_def = self.hir.context.definitions.get(first_id);
            w!(w, "Self::", first_def, "\n");
        }
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl ::std::str::FromStr for ", def, " {\n");
        w!(w, "type Err = ::intercom_cts::error::UnknownVariant;\n\n");
        w!(w, "fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {\n");
        w!(w, "match s {\n");
        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            let original_name = self.original_name(field_id);
            w!(w, "\"", original_name, "\" => Ok(Self::", field_def, "),\n");
        }
        w!(w, "_ => Err(::intercom_cts::error::UnknownVariant),\n");
        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl ::std::fmt::Display for ", def, " {\n");
        w!(w, "fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {\n");
        w!(w, "match self {\n");
        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            let original_name = self.original_name(field_id);
            w!(w, "Self::", field_def, " => f.write_str(\"", original_name, "\"),\n");
        }
        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_bitmask(&self, def: &Def, bitmask_ty: &ic_hir::hir::BitmaskTy, w: &mut Twine) {
        let element_type = rust_primitive(bitmask_ty.ty);

        w!(w, "::intercom_cts::bitmask! {\n");
        self.emit_derives(def, w);
        w!(w, "pub ", def, ": ", element_type, " {\n");

        for &flag_id in &bitmask_ty.flags {
            let flag_def = self.hir.context.definitions.get(flag_id);
            if let DefKind::Const(const_ty) = &flag_def.kind {
                let value = Self::format_numeric(&const_ty.value);
                w!(w, flag_def, " = ", value, ",\n");
            }
        }

        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_bitmask_impl(&self, def: &Def, w: &mut Twine) {
        w!(w, "impl ", def, " {\n");

        if !self.options.must_use {
            w!(w, "#[must_use]\n");
        }
        w!(w, "pub const fn new() -> Self {\n");
        w!(w, "Self::nil()\n");
        w!(w, "}\n");

        w!(w, "}\n\n");
    }

    fn emit_alias(&self, def: &Def, alias: &ic_hir::hir::AliasTy, w: &mut Twine) {
        let ty = self.rust_type(&alias.ty, def.id);
        // TODO: should be base_type_of, I think?
        if let TyKind::Adt(id) = alias.ty.kind
            && let DefKind::Interface(_) = self.hir.context.type_of(id).kind
        {
            w!(w, "pub use ", ty, " as ", def, ";\n\n");
        } else {
            w!(w, "pub type ", def, " = ", ty, ";\n\n");
        }
    }

    fn emit_const(&self, def: &Def, const_ty: &ic_hir::hir::ConstTy, w: &mut Twine) {
        let is_const_str = self.is_string_const_literal(&const_ty.ty, &const_ty.value);
        let trivial = is_trivial(def) || is_const_str;
        let kind = if trivial { "const" } else { "static" };

        w!(w, "pub ", kind, " " , def, ": ");

        if trivial {
            let ty = self.rust_type(&const_ty.ty, def.id);
            w!(w, ty, " = ");
            self.emit_const_value(&const_ty.value, &const_ty.ty, def.id, w);
        } else {
            let ty = self.rust_type(&const_ty.ty, def.id);
            w!(w, "::std::sync::LazyLock<", ty, "> =\n");
            w!(w, "::std::sync::LazyLock::new(|| ");
            self.emit_const_value(&const_ty.value, &const_ty.ty, def.id, w);
            w!(w, ")");
        }
        w!(w, ";\n\n");
    }

    fn emit_derives(&self, def: &Def, w: &mut Twine) {
        if self.options.must_use {
            w!(w, "#[must_use]\n");
        }

        let mut derives = vec![];
        if is_copy(def) {
            derives.push("Copy");
        }
        derives.push("Clone");

        if is_debug(def) {
            derives.push("Debug");
        }
        if is_eq(def) {
            derives.push("Eq");
        }
        derives.push("PartialEq");

        if is_ord(def) {
            derives.push("Ord");
        }
        derives.push("PartialOrd");

        if is_hash(def) {
            derives.push("Hash");
        }

        w!(w, "#[derive(", derives.join(", "), ")]\n");
    }

    fn emit_struct_impl(&self, def: &Def, members: &[ic_hir::hir::Member], w: &mut Twine) {
        w!(w, "impl ", def, " {\n");
        if !self.options.must_use {
            w!(w, "#[must_use]\n");
        }

        w!(w, "pub fn new() -> Self {\n");
        w!(w, "Self {\n");
        for member in members {
            w!(w, member.ident.name, ": ");
            if is_optional(member) {
                w!(w, "::std::option::Option::None");
            } else {
                let default_val = default_value(member);
                self.emit_const_value(default_val, &member.ty, def.id, w);
            }
            w!(w, ",\n");
        }
        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_default_impl(def: &Def, w: &mut Twine) {
        w!(w, "impl ::std::default::Default for ", def, " {\n");
        w!(w, "fn default() -> Self {\n");
        w!(w, "Self::new()\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn recurse_node(&self, root: &mut Module, def: &Def) {
        let w = root.find(self.hir, def);
        match &def.kind {
            DefKind::Module(module_ty) => {
                for &child_id in &module_ty.definitions {
                    let child_def = self.hir.context.definitions.get(child_id);
                    self.recurse_node(root, child_def);
                }
            }
            DefKind::Struct(struct_ty) => {
                self.emit_struct(def, struct_ty, w);
                let members = self.struct_members(struct_ty);
                self.emit_struct_impl(def, &members, w);
                Self::emit_default_impl(def, w);
                self.emit_type_info(def, w);
                self.emit_member_info(def.id, w);
                Self::emit_marshal_impl(def, &members, w);
                Self::emit_unmarshal_impl(def, &members, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Except(except_ty) => {
                self.emit_except(def, except_ty, w);
                self.emit_struct_impl(def, &except_ty.members, w);
                Self::emit_default_impl(def, w);
                self.emit_type_info(def, w);
                self.emit_member_info(def.id, w);
                Self::emit_marshal_impl(def, &except_ty.members, w);
                Self::emit_unmarshal_impl(def, &except_ty.members, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Valuetype(value_ty) => {
                self.emit_valuetype(def, value_ty, w);
                let members = self.valuetype_members(value_ty);
                self.emit_struct_impl(def, &members, w);
                Self::emit_default_impl(def, w);
                self.emit_type_info(def, w);
                self.emit_member_info(def.id, w);
                Self::emit_marshal_impl(def, &members, w);
                Self::emit_unmarshal_impl(def, &members, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Union(union_ty) => {
                self.emit_union(def, union_ty, w);
                self.emit_union_impl(def, union_ty, w);
                Self::emit_default_impl(def, w);
                self.emit_type_info(def, w);
                self.emit_union_member_info(def, union_ty, w);
                self.emit_union_marshal_impl(def, union_ty, w);
                self.emit_union_unmarshal_impl(def, union_ty, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Enum(enum_ty) => {
                self.emit_enum(def, enum_ty, w);
                self.emit_enum_impl(def, enum_ty, w);
                Self::emit_default_impl(def, w);
                self.emit_enum_type_info(def, enum_ty, w);
                self.emit_enum_member_info(def, enum_ty, w);
                self.emit_enum_marshal_impl(def, enum_ty, w);
                self.emit_enum_unmarshal_impl(def, enum_ty, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Bitmask(bitmask_ty) => {
                self.emit_bitmask(def, bitmask_ty, w);
                self.emit_bitmask_impl(def, w);
                Self::emit_default_impl(def, w);
                self.emit_bitmask_type_info(def, bitmask_ty, w);
                self.emit_bitmask_member_info(def, bitmask_ty, w);
                Self::emit_type_info_close(w);
            }
            DefKind::Alias(alias_ty) => {
                self.emit_alias(def, alias_ty, w);
            }
            DefKind::Const(const_ty) => {
                self.emit_const(def, const_ty, w);
            }
            DefKind::Interface(interface_ty) => {
                self.emit_interface(def, interface_ty, w);
            }
            DefKind::Annotation(_) | DefKind::Bitset(_) | DefKind::Decl(_) => {}
        }
    }
}
