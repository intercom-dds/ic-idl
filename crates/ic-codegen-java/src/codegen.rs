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

#![allow(clippy::cast_possible_wrap, clippy::unused_self)]

use std::collections::HashSet;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    BitmaskTy, ConstTy, Def, DefId, DefKind, EnumTy, Numeric, PrimitiveTy, Ty, TyKind, UnionTy,
    Variant,
};

use crate::JavaOptions;

fn primitive_type(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Void => "void",
        PrimitiveTy::Bool => "boolean",
        PrimitiveTy::Char | PrimitiveTy::WChar => "char",
        PrimitiveTy::Int8 | PrimitiveTy::UInt8 => "byte",
        PrimitiveTy::Int16 | PrimitiveTy::UInt16 => "short",
        PrimitiveTy::Int32 | PrimitiveTy::UInt32 => "int",
        PrimitiveTy::Int64 | PrimitiveTy::UInt64 => "long",
        PrimitiveTy::Float32 => "float",
        PrimitiveTy::Float64 | PrimitiveTy::Float128 => "double",
    }
}

fn array_dimensions(ty: &Ty) -> Vec<usize> {
    let mut dims = Vec::new();
    let mut current_ty = ty;

    while let TyKind::Array { ty: inner, len, .. } = &current_ty.kind {
        dims.push(*len);
        current_ty = inner;
    }

    dims
}

fn format_primitive_value(value: i64, ty: &Ty) -> String {
    match &ty.kind {
        TyKind::Primitive(prim) => match prim {
            PrimitiveTy::Int8 | PrimitiveTy::UInt8 => format!("(byte){value}"),
            PrimitiveTy::Int16 | PrimitiveTy::UInt16 => format!("(short){value}"),
            PrimitiveTy::Int64 | PrimitiveTy::UInt64 => format!("{value}L"),
            _ => format!("{value}"),
        },
        _ => format!("{value}"),
    }
}

pub struct JavaGen<'a> {
    hir: &'a ResolvedGraph,
    options: JavaOptions,
}

impl<'a> JavaGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, options: JavaOptions) -> Self {
        Self { hir, options }
    }

    fn is_bitmask(&self, def_id: DefId) -> bool {
        matches!(
            self.hir.context.base_def_of(def_id).kind,
            DefKind::Bitmask(_)
        )
    }

    fn java_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn file_path(&self, def: &Def, suffix: impl Into<Option<&'a str>>) -> PathBuf {
        let type_name = &def.ident.name;
        let mut path = if let Some(package) = self.package(def.id) {
            let pkg_path = package.replace('.', "/");
            PathBuf::from(pkg_path)
        } else {
            PathBuf::new()
        };

        if let Some(suffix) = suffix.into() {
            path.push(format!("{type_name}{suffix}.java"));
        } else {
            path.push(format!("{type_name}.java"));
        }
        path
    }

    fn java_type(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match &resolved_ty.kind {
            TyKind::Primitive(prim) => primitive_type(*prim).to_string(),
            TyKind::String { .. } => "java.lang.String".to_string(),
            TyKind::Adt(def_id) => {
                if self.is_bitmask(*def_id) {
                    "java.util.BitSet".to_string()
                } else {
                    self.scoped_name(*def_id, relative_def)
                }
            }
            TyKind::Sequence { ty, .. } | TyKind::Array { ty, .. } => {
                let inner = self.java_type(ty, relative_def);
                format!("{inner}[]")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.java_type(key, relative_def);
                let elem_ty = self.java_type(elem, relative_def);
                format!("java.util.Map<{key_ty}, {elem_ty}>")
            }
            TyKind::Any => "java.lang.Object".to_string(),
            TyKind::Fixed => "java.math.BigDecimal".to_string(),
            TyKind::Null => "void".to_string(),
        }
    }

    fn default_value(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match &resolved_ty.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => "false".to_string(),
                PrimitiveTy::Char | PrimitiveTy::WChar => "'\\0'".to_string(),
                PrimitiveTy::Int8
                | PrimitiveTy::UInt8
                | PrimitiveTy::Int16
                | PrimitiveTy::UInt16
                | PrimitiveTy::Int32
                | PrimitiveTy::UInt32 => "0".to_string(),
                PrimitiveTy::Int64 | PrimitiveTy::UInt64 => "0L".to_string(),
                PrimitiveTy::Float32 => "0.0f".to_string(),
                PrimitiveTy::Float64 | PrimitiveTy::Float128 => "0.0".to_string(),
                PrimitiveTy::Void => String::new(),
            },
            TyKind::String { .. } => "new String()".to_string(),
            TyKind::Adt(def_id) => {
                let type_name = self.scoped_name(*def_id, relative_def);
                let def = self.hir.context.definitions.get(*def_id);
                match &def.kind {
                    DefKind::Enum(_) => {
                        let first_field = if let DefKind::Enum(enum_ty) = &def.kind {
                            enum_ty.fields.first().copied()
                        } else {
                            None
                        };
                        if let Some(field_id) = first_field {
                            let field_name = self.java_name(field_id);
                            format!("{type_name}.{field_name}")
                        } else {
                            "null".to_string()
                        }
                    }
                    _ => format!("new {type_name}()"),
                }
            }
            TyKind::Sequence { ty, .. } => {
                let inner = self.java_type(ty, relative_def);
                format!("new {inner}[0]")
            }
            TyKind::Array { ty, len, .. } => {
                let inner = self.java_type(ty, relative_def);
                format!("new {inner}[{len}]")
            }
            TyKind::Map { .. } | TyKind::Any | TyKind::Null | TyKind::Fixed => "null".to_string(),
        }
    }

    fn scope_of(&self, def_id: DefId) -> Option<DefId> {
        let def = self.hir.context.definitions.get(def_id);
        let mut current = def.parent?;

        loop {
            let def = self.hir.context.definitions.get(current);
            if matches!(def.kind, DefKind::Module(_) | DefKind::Interface(_)) {
                return Some(current);
            }
            current = def.parent?;
        }
    }

    fn build_path_from(&self, from_scope: DefId, to_scope: Option<DefId>) -> Vec<String> {
        let mut path = Vec::new();
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
                    if matches!(parent_def.kind, DefKind::Module(_) | DefKind::Interface(_)) {
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
        let type_name = self.java_name(target_def_id);
        let target_scope = self.scope_of(target_def_id);
        let current_scope = self.scope_of(relative_to_def_id);

        match (target_scope, current_scope) {
            (None, _) => type_name.to_string(),
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join(".");
                if pkg_name.is_empty() {
                    type_name.to_string()
                } else {
                    format!("{pkg_name}.{type_name}")
                }
            }
            (Some(target_scope), Some(current_scope)) => {
                if target_scope == current_scope {
                    return type_name.to_string();
                }
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

    fn package(&self, def_id: DefId) -> Option<String> {
        let scope = self.scope_of(def_id)?;
        let path = self.build_path_from(scope, None);
        if path.is_empty() {
            None
        } else {
            Some(path.join("."))
        }
    }

    fn emit_header(&self, w: &mut Twine) {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");
        w!(w, "// @generated by ic-idl ", IC_VERSION, "\n\n");
    }

    fn emit_package(&self, w: &mut Twine, def_id: DefId) {
        if let Some(mut package) = self.package(def_id) {
            if let Some(prefix) = &self.options.package_prefix {
                package = format!("{prefix}.{package}");
            }
            w!(w, "package ", package, ";\n\n");
        } else if let Some(prefix) = &self.options.package_prefix {
            w!(w, "package ", prefix, ";\n\n");
        }
    }

    fn emit_def(&self, def_id: DefId, files: &mut Vec<File>) {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Struct(_) | DefKind::Except(_) => {
                files.push(self.emit_struct(def));
                files.push(self.emit_holder(def));
                files.push(self.emit_seq_holder(def));
            }
            DefKind::Union(union_ty) => {
                files.push(self.emit_union(def, union_ty));
                files.push(self.emit_holder(def));
                files.push(self.emit_seq_holder(def));
            }
            DefKind::Enum(enum_ty) => {
                files.push(self.emit_enum(def, enum_ty));
            }
            DefKind::Bitmask(bitmask_ty) => {
                files.push(self.emit_bitmask(def, bitmask_ty));
            }
            DefKind::Const(const_ty) => {
                files.push(self.emit_const(def, const_ty));
            }
            DefKind::Module(module_ty) => {
                for &nested_id in &module_ty.definitions {
                    self.emit_def(nested_id, files);
                }
            }
            DefKind::Interface(interface_ty) => {
                for &nested_id in &interface_ty.definitions {
                    self.emit_def(nested_id, files);
                }
            }
            DefKind::Valuetype(valuetype_ty) => {
                for &nested_id in &valuetype_ty.definitions {
                    self.emit_def(nested_id, files);
                }
            }
            _ => {}
        }
    }

    fn emit_struct(&self, def: &Def) -> File {
        let DefKind::Struct(struct_ty) = &def.kind else {
            unreachable!()
        };

        let mut w = Twine::new();

        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);
        w!(w, "public class ", def.ident.name, " {\n");

        self.emit_default_constructor(&mut w, def.id, &struct_ty.members);
        self.emit_copy_constructor(&mut w, def.id, &struct_ty.members);
        self.emit_all_args_constructor(&mut w, def.id, &struct_ty.members);

        w!(w, "@Override\n");
        w!(w, "public ", def.ident.name, " clone() {\n");
        w!(w, "return new ", def.ident.name, "(this);\n");
        w!(w, "}\n\n");

        for member in &struct_ty.members {
            let java_type = self.java_type(&member.ty, def.id);
            let array_dims = array_dimensions(&member.ty);

            w!(w, "public ", java_type);
            for &dim in &array_dims {
                w!(w, "[", dim, "]");
            }
            w!(w, " ", member.ident.name, ";\n");
        }

        w!(w, "};\n");

        let path = self.file_path(def, None);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_default_constructor(
        &self,
        w: &mut Twine,
        def_id: DefId,
        members: &[ic_hir::hir::Member],
    ) {
        let def = self.hir.context.definitions.get(def_id);
        w!(w, "public ", def.ident.name, " () {\n");

        for member in members {
            let default_val = self.default_value(&member.ty, def_id);
            if !default_val.is_empty() {
                w!(w, "this.", member.ident.name, " = ", default_val, ";\n");

                let resolved_ty = self.hir.context.resolve_ty(&member.ty);
                if let TyKind::Array { .. } = &resolved_ty.kind {
                    self.emit_array_initialization(w, &member.ident.name, &resolved_ty, def_id);
                }
            }
        }

        w!(w, "}\n\n");
    }

    fn emit_array_initialization(&self, w: &mut Twine, field_name: &str, ty: &Ty, def_id: DefId) {
        if let TyKind::Array { ty: inner, len, .. } = &ty.kind {
            let inner_default = self.default_value(inner, def_id);
            if !inner_default.is_empty() {
                w!(w, "for (int _i_ind = 0; _i_ind < ", len, "; _i_ind++) {\n");
                w!(w, "this.", field_name, "[_i_ind] = ", inner_default, ";\n");
                w!(w, "}\n");
            }
        }
    }

    fn emit_copy_constructor(&self, w: &mut Twine, def_id: DefId, members: &[ic_hir::hir::Member]) {
        let def = self.hir.context.definitions.get(def_id);
        w!(w, "public ", def.ident.name, "(", def.ident.name, " other) {\n");

        for member in members {
            let resolved_ty = self.hir.context.resolve_ty(&member.ty);
            match &resolved_ty.kind {
                TyKind::Sequence { ty, .. } => {
                    w!(w, "{\n");
                    w!(w, "int _i_len = other.", member.ident.name, ".length;\n");
                    let inner_type = self.java_type(ty, def_id);
                    w!(w, "if (this.", member.ident.name, " == null || this.", member.ident.name, ".length != _i_len) {\n");
                    w!(w, "this.", member.ident.name, " = new ", inner_type, "[_i_len];\n");
                    w!(w, "}\n");
                    w!(w, "for (int _i_ind = 0; _i_ind < _i_len; _i_ind++) {\n");
                    w!(w, "this.", member.ident.name, "[_i_ind] = other.", member.ident.name, "[_i_ind];\n");
                    w!(w, "}\n");
                    w!(w, "}\n");
                }
                TyKind::Array { len, .. } => {
                    w!(w, "for (int _i_ind = 0; _i_ind < ", len, "; _i_ind++) {\n");
                    w!(w, "this.", member.ident.name, "[_i_ind] = other.", member.ident.name, "[_i_ind];\n");
                    w!(w, "}\n");
                }
                TyKind::Map { key, elem, .. } => {
                    let key_ty = self.java_type(key, def_id);
                    let elem_ty = self.java_type(elem, def_id);
                    w!(w, "this.", member.ident.name, " = new java.util.HashMap< ", key_ty, ", ", elem_ty, " >( other.", member.ident.name, " );\n");
                }
                _ => {
                    w!(w, "this.", member.ident.name, " = other.", member.ident.name, ";\n");
                }
            }
        }

        w!(w, "}\n\n");
    }

    fn emit_all_args_constructor(
        &self,
        w: &mut Twine,
        def_id: DefId,
        members: &[ic_hir::hir::Member],
    ) {
        let def = self.hir.context.definitions.get(def_id);
        w!(w, "public ", def.ident.name, "(");

        for (i, member) in members.iter().enumerate() {
            if i > 0 {
                w!(w, ", ");
            }
            let java_type = self.java_type(&member.ty, def_id);
            w!(w, java_type, " ", member.ident.name);
        }

        w!(w, ") {\n");

        for member in members {
            w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_enum(&self, def: &Def, enum_ty: &EnumTy) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "public enum ", def, " {\n");
        self.emit_enumerators(&mut w, &enum_ty.fields);

        w!(w, "private int _the_ordinal;\n\n");

        w!(w, "private ", def, "(int a_ordinal) {\n");
        w!(w, "_the_ordinal = a_ordinal;\n");
        w!(w, "}\n\n");

        w!(w, "public final int getValue() {\n");
        w!(w, "return _the_ordinal;\n");
        w!(w, "}\n\n");

        w!(w, "public static final ", def, " valueOf(int val) {\n");
        w!(w, "switch (val) {\n");

        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            let field_name = &field_def.ident.name;

            if let DefKind::Const(const_ty) = &field_def.kind {
                if let Some(val) = self.hir.context.integer_value(&const_ty.value) {
                    w.dedent();
                    w!(w, "case ", val, ":\n");
                    w.indent();
                    w!(w, "return ", field_name, ";\n");
                }
            }
        }

        w!(w, "}\n");

        if let Some(&first_field_id) = enum_ty.fields.first() {
            let first_field_name = self.java_name(first_field_id);
            w!(w, "return ", first_field_name, ";\n");
        }

        w!(w, "}\n");
        w!(w, "}\n");

        let path = self.file_path(def, None);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_union(&self, def: &Def, union_ty: &UnionTy) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "public class ", def, " {\n");
        self.emit_union_constructors(&mut w, def, union_ty);

        w!(w, "@Override\n");
        w!(w, "public ", def.ident.name, " clone() {\n");
        w!(w, "return new ", def.ident.name, "(this);\n");
        w!(w, "}\n\n");

        for variant in &union_ty.variants {
            let java_type = self.java_type(&variant.ty, def.id);
            w!(w, "public ", java_type, " ", variant.ident.name, "() {\n");
            w!(w, "return _", variant.ident.name, ";\n");
            w!(w, "}\n\n");

            w!(w, "public void ", variant.ident.name, "(", java_type, " ", variant.ident.name, ") {\n");

            if let Some(first_label) = variant.labels.first() {
                let disc_value = self.format_numeric(&first_label.value, &union_ty.disc.ty, def.id);
                w!(w, "_discriminator = ", disc_value, ";\n");
            } else if variant.is_default {
                let default_value =
                    self.find_default_discriminator_value(union_ty, &union_ty.disc.ty, def.id);
                w!(w, "_discriminator = ", default_value, ";\n");
            }

            w!(w, "this._", variant.ident.name, " = ", variant.ident.name, ";\n");
            w!(w, "}\n\n");
        }

        let disc_type = self.java_type(&union_ty.disc.ty, def.id);
        w!(w, "public ", disc_type, " discriminator() {\n");
        w!(w, "return _discriminator;\n");
        w!(w, "}\n\n");

        w!(w, "public void discriminator(", disc_type, " discriminator) {\n");
        w!(w, "if (_discriminator != discriminator) {\n");
        w!(w, "_discriminator = discriminator;\n");
        w!(w, "switch (discriminator) {\n");

        for variant in &union_ty.variants {
            self.emit_variant_cases(&mut w, &union_ty.disc.ty, variant, def.id);
            let default_val = self.default_value(&variant.ty, def.id);
            w!(w, "this._", variant.ident.name, " = ", default_val, ";\n");
            w!(w, "break;\n");
        }

        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "private ", disc_type, " _discriminator;\n");
        for variant in &union_ty.variants {
            let java_type = self.java_type(&variant.ty, def.id);
            w!(w, "protected ", java_type, " _", variant.ident.name, ";\n");
        }

        w!(w, "\n}\n");

        let path = self.file_path(def, None);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_union_constructors(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        w!(w, "public ", def, "() {\n");
        if let Some(first_variant) = union_ty.variants.first() {
            if let Some(first_label) = first_variant.labels.first() {
                let disc_value = self.format_numeric(&first_label.value, &union_ty.disc.ty, def.id);
                w!(w, "discriminator(", disc_value, ");\n");
            }
        }
        w!(w, "}\n\n");

        w!(w, "public ", def, "(", def, " other) {\n");
        w!(w, "discriminator(other.discriminator());\n");
        w!(w, "switch (discriminator()) {\n");

        for variant in &union_ty.variants {
            self.emit_variant_cases(w, &union_ty.disc.ty, variant, def.id);
            w!(w, "this._", variant.ident.name, " = other._", variant.ident.name, ";\n");
            w!(w, "break;\n");
        }

        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    fn emit_variant_cases(&self, w: &mut Twine, disc_ty: &Ty, variant: &Variant, def_id: DefId) {
        w.dedent();
        for label in &variant.labels {
            let case_val = self.format_numeric(&label.value, disc_ty, def_id);
            w!(w, "case ", case_val, ":\n");
        }
        if variant.is_default {
            w!(w, "default:\n");
        }
        w.indent();
    }

    fn find_default_discriminator_value(
        &self,
        union_ty: &UnionTy,
        disc_ty: &Ty,
        def_id: DefId,
    ) -> String {
        let mut used_values = HashSet::new();
        for variant in &union_ty.variants {
            for label in &variant.labels {
                if let Some(val) = self.hir.context.integer_value(&label.value) {
                    used_values.insert(val);
                }
            }
        }

        let resolved_ty = self.hir.context.resolve_ty(disc_ty);

        match &resolved_ty.kind {
            TyKind::Primitive(prim) => {
                let max = match prim {
                    PrimitiveTy::Int8 => i64::from(i8::MAX),
                    PrimitiveTy::UInt8 => i64::from(u8::MAX),
                    PrimitiveTy::Int16 => i64::from(i16::MAX),
                    PrimitiveTy::UInt16 => i64::from(u16::MAX),
                    PrimitiveTy::Int32 => i64::from(i32::MAX),
                    PrimitiveTy::UInt32 => i64::from(u32::MAX),
                    PrimitiveTy::Char | PrimitiveTy::WChar => 255,
                    _ => 0,
                };

                for val in (0..=max).rev() {
                    if !used_values.contains(&val) {
                        return format_primitive_value(val, &resolved_ty);
                    }
                }

                format_primitive_value(max, &resolved_ty)
            }
            TyKind::Adt(adt_def_id) => {
                let adt_def = self.hir.context.definitions.get(*adt_def_id);
                if let DefKind::Enum(enum_ty) = &adt_def.kind {
                    for &field_id in enum_ty.fields.iter().rev() {
                        let field_def = self.hir.context.definitions.get(field_id);
                        if let DefKind::Const(const_ty) = &field_def.kind {
                            if let Some(val) = self.hir.context.integer_value(&const_ty.value) {
                                if !used_values.contains(&val) {
                                    return self.format_numeric(&const_ty.value, disc_ty, def_id);
                                }
                            }
                        }
                    }
                }
                self.default_value(disc_ty, def_id)
            }
            _ => self.default_value(disc_ty, def_id),
        }
    }

    fn emit_enumerators(&self, w: &mut Twine, fields: &[DefId]) {
        for (i, flag_id) in fields.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            if let DefKind::Const(const_ty) = &flag_def.kind {
                let ordinal = self.hir.context.integer_value(&const_ty.value).unwrap_or(0);
                w!(w, flag_def, "(", ordinal, ")");
                if i < fields.len() - 1 {
                    w!(w, ",\n");
                } else {
                    w!(w, ";\n");
                }
            }
        }
    }

    fn emit_bitmask(&self, def: &Def, bitmask_ty: &BitmaskTy) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "public enum ", def, " {\n");
        self.emit_enumerators(&mut w, &bitmask_ty.flags);
        w!(w, "}\n");

        let path = self.file_path(def, None);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_const(&self, def: &Def, const_ty: &ConstTy) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "public interface ", def, "{ \n");
        let java_type = self.java_type(&const_ty.ty, def.id);
        let value = self.format_numeric(&const_ty.value, &const_ty.ty, def.id);

        w!(w, "public static final ", java_type, " value = ", value, ";\n");
        w!(w, "}\n");

        let path = self.file_path(def, None);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn format_numeric(&self, value: &ic_hir::hir::Numeric, ty: &Ty, def_id: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match value {
            Numeric::Bool(b) => b.to_string(),
            Numeric::Char(c) => format!("'\\u{:04x}'", u32::from(*c)),
            Numeric::Int8(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::UInt8(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::Int16(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::UInt16(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::Int32(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::UInt32(i) => format_primitive_value(i64::from(*i), &resolved_ty),
            Numeric::Int64(i) => format_primitive_value(*i, &resolved_ty),
            Numeric::UInt64(i) => format!("{i}L"),
            Numeric::Float(f) => format!("(float){f:e}"),
            Numeric::Double(f) => format!("(double){f:e}"),
            Numeric::String(s) => {
                format!("\"{s}\"")
            }
            Numeric::Const(const_def_id) => {
                let const_def = self.hir.context.definitions.get(*const_def_id);
                if let Some(parent_id) = const_def.parent {
                    let parent_def = self.hir.context.definitions.get(parent_id);
                    if matches!(parent_def.kind, DefKind::Enum(_)) {
                        let enum_name = self.scoped_name(parent_id, def_id);
                        let const_name = &const_def.ident.name;
                        return format!("{enum_name}.{const_name}");
                    }
                }
                self.scoped_name(*const_def_id, def_id)
            }
            _ => "null".to_string(),
        }
    }

    fn emit_holder(&self, def: &Def) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "final public class ", def, "Holder {\n");
        w!(w, "public ", def, " value;\n\n");
        w!(w, "public ", def, "Holder() {}\n\n");
        w!(w, "public ", def, "Holder(", def, " initial) {\n");
        w!(w, "value = initial;\n");
        w!(w, "}\n");
        w!(w, "}\n");

        let path = self.file_path(def, "Holder");
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_seq_holder(&self, def: &Def) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);

        w!(w, "final public class ", def, "SeqHolder {\n");
        w!(w, "public ", def, "[] value;\n\n");
        w!(w, "public ", def, "SeqHolder() {}\n\n");
        w!(w, "public ", def, "SeqHolder(", def, "[] initial) {\n");
        w!(w, "value = initial;\n");
        w!(w, "}\n");
        w!(w, "}\n");

        let path = self.file_path(def, "SeqHolder");
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut result = Vec::new();
        for &def_id in &self.hir.order {
            self.emit_def(def_id, &mut result);
        }
        result
    }
}
