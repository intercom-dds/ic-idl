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
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use ic_emit::printer::{IterExt, Twine, w};
use ic_emit::{File, case};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    BitmaskTy, ConstTy, Def, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy, Member, Numeric,
    ParamKind, PrimitiveTy, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};

use crate::JavaOptions;

const OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
const PRIME: u64 = 0x0100_0000_01B3;

struct Fnv1a {
    state: u64,
}

impl Fnv1a {
    fn new() -> Self {
        Self {
            state: OFFSET_BASIS,
        }
    }
}

impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(PRIME);
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

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

fn boxed_primitive(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Void => "java.lang.Void",
        PrimitiveTy::Bool => "java.lang.Boolean",
        PrimitiveTy::Char | PrimitiveTy::WChar => "java.lang.Character",
        PrimitiveTy::Int8 | PrimitiveTy::UInt8 => "java.lang.Byte",
        PrimitiveTy::Int16 | PrimitiveTy::UInt16 => "java.lang.Short",
        PrimitiveTy::Int32 | PrimitiveTy::UInt32 => "java.lang.Integer",
        PrimitiveTy::Int64 | PrimitiveTy::UInt64 => "java.lang.Long",
        PrimitiveTy::Float32 => "java.lang.Float",
        PrimitiveTy::Float64 | PrimitiveTy::Float128 => "java.lang.Double",
    }
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

fn escape_java_string(s: &str) -> String {
    let mut result = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x20'..='\x7E' => result.push(ch),
            _ => {
                for code_unit in ch.encode_utf16(&mut [0; 2]) {
                    _ = write!(result, "\\u{code_unit:04x}");
                }
            }
        }
    }
    result.push('"');
    result
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

    fn is_cloneable_adt(&self, ty: &Ty) -> bool {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Adt(def_id) => {
                let def = self.hir.context.base_def_of(*def_id);
                matches!(
                    def.kind,
                    DefKind::Struct(_)
                        | DefKind::Union(_)
                        | DefKind::Valuetype(_)
                        | DefKind::Except(_)
                )
            }
            _ => false,
        }
    }

    fn needs_deep_copy(&self, ty: &Ty) -> bool {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            TyKind::Sequence { .. } | TyKind::Map { .. } => true,
            TyKind::Array { ty, .. } => self.needs_deep_copy(ty),
            _ => self.is_cloneable_adt(ty),
        }
    }

    fn serial_version_uid(&self, def: &Def, members: &[Member]) -> i64 {
        let mut hasher = Fnv1a::new();
        def.ident.name.hash(&mut hasher);
        for member in members {
            member.ident.name.hash(&mut hasher);
            self.hash_type(&member.ty, &mut hasher);
        }
        hasher.finish() as i64
    }

    fn serial_version_uid_union(&self, def: &Def, union_ty: &UnionTy) -> i64 {
        let mut hasher = Fnv1a::new();
        def.ident.name.hash(&mut hasher);
        self.hash_type(&union_ty.disc.ty, &mut hasher);
        for variant in &union_ty.variants {
            variant.ident.name.hash(&mut hasher);
            self.hash_type(&variant.ty, &mut hasher);
        }
        hasher.finish() as i64
    }

    fn hash_type(&self, ty: &Ty, hasher: &mut impl Hasher) {
        let resolved = self.hir.context.resolve_ty(ty);
        std::mem::discriminant(&resolved.kind).hash(hasher);
        match &resolved.kind {
            TyKind::Primitive(prim) => std::mem::discriminant(prim).hash(hasher),
            TyKind::Adt(def_id) => self.java_name(*def_id).hash(hasher),
            TyKind::Array { ty, len, .. } => {
                len.hash(hasher);
                self.hash_type(ty, hasher);
            }
            TyKind::Sequence { ty, .. } => self.hash_type(ty, hasher),
            TyKind::Map { key, elem, .. } => {
                self.hash_type(key, hasher);
                self.hash_type(elem, hasher);
            }
            TyKind::String { .. } => "String".hash(hasher),
            _ => {}
        }
    }

    fn java_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn file_path(&self, def: &Def, suffix: impl Into<Option<&'a str>>) -> PathBuf {
        let mut path = if let Some(package) = self.package(def.id) {
            let pkg_path = package.replace('.', "/");
            PathBuf::from(pkg_path)
        } else {
            PathBuf::new()
        };

        if let Some(suffix) = suffix.into() {
            path.push(format!("{def}{suffix}.java"));
        } else {
            path.push(format!("{def}.java"));
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
            TyKind::Array { ty, .. } => {
                let inner = self.java_type(ty, relative_def);
                format!("{inner}[]")
            }
            TyKind::Sequence { ty, .. } => {
                let inner = self.boxed_java_type(ty, relative_def);
                format!("java.util.List<{inner}>")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.boxed_java_type(key, relative_def);
                let elem_ty = self.boxed_java_type(elem, relative_def);
                format!("java.util.Map<{key_ty}, {elem_ty}>")
            }
            TyKind::Any | TyKind::Fixed => "java.lang.Object".to_string(),
            TyKind::Null => "void".to_string(),
        }
    }

    fn boxed_java_type(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        if let TyKind::Primitive(prim) = &resolved_ty.kind {
            boxed_primitive(*prim).to_string()
        } else {
            self.java_type(ty, relative_def)
        }
    }

    fn raw_java_type(&self, ty: &Ty, relative_def: DefId) -> String {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        match &resolved_ty.kind {
            TyKind::Sequence { .. } => "java.util.List".to_string(),
            TyKind::Map { .. } => "java.util.Map".to_string(),
            TyKind::Array { ty, .. } => {
                let inner = self.raw_java_type(ty, relative_def);
                format!("{inner}[]")
            }
            _ => self.java_type(ty, relative_def),
        }
    }

    fn array_dimensions(&self, ty: &Ty, relative_def: DefId) -> (Vec<usize>, String) {
        let mut dimensions = vec![];
        let mut current_ty = ty.clone();

        loop {
            let resolved = self.hir.context.resolve_ty(&current_ty);
            if let TyKind::Array { ty: inner, len, .. } = resolved.kind {
                dimensions.push(len);
                current_ty = *inner;
            } else {
                let base_type = self.raw_java_type(&resolved, relative_def);
                return (dimensions, base_type);
            }
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
            TyKind::String { .. } => "\"\"".to_string(),
            TyKind::Adt(def_id) => {
                let type_name = self.scoped_name(*def_id, relative_def);
                let def = self.hir.context.definitions.get(*def_id);
                match &def.kind {
                    DefKind::Enum(enum_ty) => {
                        if let Some(&field_id) = enum_ty.fields.first() {
                            let field_name = self.java_name(field_id);
                            format!("{type_name}.{field_name}")
                        } else {
                            "null".to_string()
                        }
                    }
                    _ => format!("new {type_name}()"),
                }
            }
            TyKind::Array { .. } => {
                let (dimensions, base_type) = self.array_dimensions(ty, relative_def);
                let dims = dimensions
                    .iter()
                    .fold(String::new(), |acc, d| acc + &format!("[{d}]"));
                format!("new {base_type}{dims}")
            }
            TyKind::Sequence { .. } => "new java.util.ArrayList<>()".to_string(),
            TyKind::Map { .. } => "new java.util.HashMap<>()".to_string(),
            TyKind::Any | TyKind::Null | TyKind::Fixed => "null".to_string(),
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
            (Some(target_scope), Some(current_scope)) if target_scope == current_scope => {
                type_name.to_string()
            }
            (Some(target_scope), _) => {
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

    fn get_set(&self, name: &str) -> (String, String) {
        let getter = format!("get_{name}");
        let setter = format!("set_{name}");
        if self.options.no_rename {
            (getter, setter)
        } else {
            (case::camel(getter), case::camel(setter))
        }
    }

    fn disc_get_set(&self) -> (String, String) {
        self.get_set("discriminator")
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

    fn emit_file(
        &self,
        def: &Def,
        suffix: impl Into<Option<&'a str>>,
        emit_fn: impl FnOnce(&mut Twine),
    ) -> File {
        let mut w = Twine::new();
        self.emit_header(&mut w);
        self.emit_package(&mut w, def.id);
        emit_fn(&mut w);

        let path = self.file_path(def, suffix);
        File::Generated {
            path,
            source: w.finish(),
        }
    }

    fn emit_def(&self, def_id: DefId, files: &mut Vec<File>) {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Struct(struct_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_struct(w, def, struct_ty));
                files.push(f);
            }
            DefKind::Union(union_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_union(w, def, union_ty));
                files.push(f);
            }
            DefKind::Enum(enum_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_enum(w, def, enum_ty));
                files.push(f);
            }
            DefKind::Except(except_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_except(w, def, except_ty));
                files.push(f);
            }
            DefKind::Bitmask(bitmask_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_bitmask(w, def, bitmask_ty));
                files.push(f);
            }
            DefKind::Const(const_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_const(w, def, const_ty));
                files.push(f);
            }
            DefKind::Interface(interface_ty) => {
                let f = self.emit_file(def, None, |w| self.emit_interface(w, def, interface_ty));
                files.push(f);
            }
            DefKind::Valuetype(value_ty) => {
                let interface = self.emit_file(def, "Abstract", |w| {
                    self.emit_abstract_valuetype(w, def, value_ty);
                });
                files.push(interface);

                let imp = self.emit_file(def, None, |w| self.emit_valuetype(w, def, value_ty));
                files.push(imp);
            }
            DefKind::Module(module_ty) => {
                for &nested_id in &module_ty.definitions {
                    self.emit_def(nested_id, files);
                }
            }
            _ => {}
        }
    }

    fn emit_struct(&self, w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        w!(w, "public class ", def);
        if let Some(parent) = struct_ty.parent {
            let parent = self.java_name(parent);
            w!(w, " extends ", parent);
        }
        w!(w, " implements java.io.Serializable {\n");
        let uid = self.serial_version_uid(def, &struct_ty.members);
        w!(w, "private static final long serialVersionUID = ", uid, "L;\n\n");

        self.emit_default_ctor(w, def.id, &struct_ty.members);
        self.emit_copy_ctor(w, def, def.parent, &struct_ty.members);
        if !struct_ty.members.is_empty() {
            self.emit_arg_ctor(w, def.id, &struct_ty.members);
        }

        w!(w, "@Override\n");
        w!(w, "public ", def, " clone() {\n");
        w!(w, "return new ", def, "(this);\n");
        w!(w, "}\n\n");

        self.emit_accessors(w, def.id, &struct_ty.members);
        self.emit_struct_equals(w, def, &struct_ty.members);
        self.emit_struct_hashcode(w, &struct_ty.members);

        for member in &struct_ty.members {
            let java_type = self.java_type(&member.ty, def.id);
            w!(w, "protected ", java_type, " ", member.ident.name, ";\n");
        }
        w!(w, "}\n");
    }

    fn emit_struct_equals(&self, w: &mut Twine, def: &Def, members: &[Member]) {
        w!(w, "@Override\n");
        w!(w, "public boolean equals(Object obj) {\n");
        w!(w, "if (this == obj) {\n");
        w!(w, "return true;\n");
        w!(w, "}\n");
        w!(w, "if (obj == null || getClass() != obj.getClass()) {\n");
        w!(w, "return false;\n");
        w!(w, "}\n");
        w!(w, def, " other = (", def, ") obj;\n");

        if members.is_empty() {
            w!(w, "return true;\n");
        } else {
            w!(w, "return ");
            for (i, member) in members.iter().enumerate() {
                if i > 0 {
                    w!(w, "\n&& ");
                }
                let resolved = self.hir.context.resolve_ty(&member.ty);
                let name = &member.ident.name;
                match &resolved.kind {
                    TyKind::Primitive(prim) => match prim {
                        PrimitiveTy::Float32 => {
                            w!(w, "Float.compare(", name, ", other.", name, ") == 0");
                        }
                        PrimitiveTy::Float64 | PrimitiveTy::Float128 => {
                            w!(w, "Double.compare(", name, ", other.", name, ") == 0");
                        }
                        _ => {
                            w!(w, name, " == other.", name);
                        }
                    },
                    TyKind::Array { ty: inner, .. } => {
                        let inner_resolved = self.hir.context.resolve_ty(inner);
                        if matches!(inner_resolved.kind, TyKind::Primitive(_)) {
                            w!(w, "java.util.Arrays.equals(", name, ", other.", name, ")");
                        } else {
                            w!(w, "java.util.Arrays.deepEquals(", name, ", other.", name, ")");
                        }
                    }
                    _ => {
                        w!(w, "java.util.Objects.equals(", name, ", other.", name, ")");
                    }
                }
            }
            w!(w, ";\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_struct_hashcode(&self, w: &mut Twine, members: &[Member]) {
        w!(w, "@Override\n");
        w!(w, "public int hashCode() {\n");
        if members.is_empty() {
            w!(w, "return 0;\n");
        } else {
            w!(w, "return java.util.Objects.hash(");
            for (i, member) in members.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let resolved = self.hir.context.resolve_ty(&member.ty);
                let name = &member.ident.name;
                if let TyKind::Array { ty: inner, .. } = &resolved.kind {
                    let inner_resolved = self.hir.context.resolve_ty(inner);
                    if matches!(inner_resolved.kind, TyKind::Primitive(_)) {
                        w!(w, "java.util.Arrays.hashCode(", name, ")");
                    } else {
                        w!(w, "java.util.Arrays.deepHashCode(", name, ")");
                    }
                } else {
                    w!(w, name);
                }
            }
            w!(w, ");\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_default_ctor(&self, w: &mut Twine, def_id: DefId, members: &[Member]) {
        let def = self.hir.context.definitions.get(def_id);
        w!(w, "public ", def.ident.name, "() {\n");

        for member in members {
            let default_val = self.default_value(&member.ty, def_id);
            if !default_val.is_empty() {
                w!(w, "this.", member.ident.name, " = ", default_val, ";\n");
            }
        }

        w!(w, "}\n\n");
    }

    fn emit_copy_ctor(&self, w: &mut Twine, def: &Def, parent: Option<DefId>, members: &[Member]) {
        w!(w, "public ", def.ident.name, "(", def, " other) {\n");
        if parent.is_some() {
            w!(w, "super(other);\n");
        }

        for member in members {
            self.emit_field_copy(w, &member.ident.name, &member.ty, def.id);
        }

        w!(w, "}\n\n");
    }

    fn emit_field_copy(&self, w: &mut Twine, name: &str, ty: &Ty, def_id: DefId) {
        let resolved_ty = self.hir.context.resolve_ty(ty);

        match &resolved_ty.kind {
            TyKind::Sequence { ty: elem, .. } => {
                self.emit_sequence_deep_copy(w, name, elem, def_id);
            }
            TyKind::Array { .. } => {
                self.emit_array_deep_copy(w, def_id, name, ty);
            }
            TyKind::Map { key, elem, .. } => {
                self.emit_map_deep_copy(w, name, key, elem, def_id);
            }
            TyKind::Adt(_) if self.is_cloneable_adt(ty) => {
                w!(w, "this.", name, " = other.", name, ".clone();\n");
            }
            _ => {
                w!(w, "this.", name, " = other.", name, ";\n");
            }
        }
    }

    fn emit_sequence_deep_copy(&self, w: &mut Twine, name: &str, elem_ty: &Ty, def_id: DefId) {
        w!(w, "this.", name, " = new java.util.ArrayList<>(other.", name, ".size());\n");
        w!(w, "for (var _e0 : other.", name, ") {\n");
        let copy_expr = self.emit_value_copy(w, "_e0", elem_ty, def_id, 1);
        w!(w, "this.", name, ".add(", copy_expr, ");\n");
        w!(w, "}\n");
    }

    fn emit_map_deep_copy(
        &self,
        w: &mut Twine,
        name: &str,
        key_ty: &Ty,
        elem_ty: &Ty,
        def_id: DefId,
    ) {
        let key_type = self.boxed_java_type(key_ty, def_id);
        let elem_type = self.boxed_java_type(elem_ty, def_id);
        w!(w, "this.", name, " = new java.util.HashMap<", key_type, ", ", elem_type, ">();\n");
        w!(w, "for (var _entry0 : other.", name, ".entrySet()) {\n");
        let key_copy = self.emit_value_copy(w, "_entry0.getKey()", key_ty, def_id, 1);
        let val_copy = self.emit_value_copy(w, "_entry0.getValue()", elem_ty, def_id, 1);
        w!(w, "this.", name, ".put(", key_copy, ", ", val_copy, ");\n");
        w!(w, "}\n");
    }

    fn emit_value_copy(
        &self,
        w: &mut Twine,
        src: &str,
        ty: &Ty,
        def_id: DefId,
        depth: usize,
    ) -> String {
        let resolved = self.hir.context.resolve_ty(ty);
        match &resolved.kind {
            _ if self.is_cloneable_adt(ty) => format!("{src}.clone()"),
            TyKind::Sequence { ty: inner, .. } => {
                let var = format!("_seq{depth}");
                let elem_type = self.boxed_java_type(inner, def_id);
                w!(w, "var ", &var, " = new java.util.ArrayList<", elem_type, ">();\n");
                let iter_var = format!("_e{depth}");
                w!(w, "for (var ", &iter_var, " : ", src, ") {\n");
                let copy = self.emit_value_copy(w, &iter_var, inner, def_id, depth + 1);
                w!(w, var, ".add(", copy, ");\n");
                w!(w, "}\n");
                var
            }
            TyKind::Map { key, elem, .. } => {
                let var = format!("_map{depth}");
                let key_type = self.boxed_java_type(key, def_id);
                let elem_type = self.boxed_java_type(elem, def_id);
                w!(w, "var ", &var, " = new java.util.HashMap<", key_type, ", ", elem_type, ">();\n");
                let entry_var = format!("_entry{depth}");
                w!(w, "for (var ", &entry_var, " : ", src, ".entrySet()) {\n");
                let key_src = format!("{entry_var}.getKey()");
                let val_src = format!("{entry_var}.getValue()");
                let key_copy = self.emit_value_copy(w, &key_src, key, def_id, depth + 1);
                let val_copy = self.emit_value_copy(w, &val_src, elem, def_id, depth + 1);
                w!(w, var, ".put(", key_copy, ", ", val_copy, ");\n");
                w!(w, "}\n");
                var
            }
            _ => src.to_string(),
        }
    }

    fn emit_array_deep_copy(&self, w: &mut Twine, def_id: DefId, name: &str, ty: &Ty) {
        let (dimensions, base_type) = self.array_dimensions(ty, def_id);
        let innermost_ty = self.get_innermost_array_type(ty);

        if self.needs_deep_copy(&innermost_ty) {
            w!(w, "this.", name, " = new ", &base_type);
            for dim in &dimensions {
                w!(w, "[", dim, "]");
            }
            w!(w, ";\n");
            self.emit_array_deep_clone_loop(w, def_id, name, &dimensions, &innermost_ty);
        } else {
            self.emit_array_shallow_clone(w, name, &base_type, &dimensions);
        }
    }

    fn get_innermost_array_type(&self, ty: &Ty) -> Ty {
        let mut current = ty.clone();
        loop {
            let resolved = self.hir.context.resolve_ty(&current);
            if let TyKind::Array { ty: inner, .. } = resolved.kind {
                current = *inner;
            } else {
                return resolved;
            }
        }
    }

    fn emit_array_deep_clone_loop(
        &self,
        w: &mut Twine,
        def_id: DefId,
        name: &str,
        dimensions: &[usize],
        elem_ty: &Ty,
    ) {
        for (idx, dim) in dimensions.iter().enumerate() {
            let var = format!("_i{idx}");
            w!(w, "for (int ", var, " = 0; ", var, " < ", dim, "; ", var, "++) {\n");
        }

        let mut indices = String::new();
        for idx in 0..dimensions.len() {
            _ = write!(indices, "[_i{idx}]");
        }

        let src = format!("other.{name}{indices}");
        let copy_expr = self.emit_value_copy(w, &src, elem_ty, def_id, 0);
        w!(w, "this.", name, indices, " = ", copy_expr, ";\n");

        for _ in 0..dimensions.len() {
            w!(w, "}\n");
        }
    }

    fn emit_array_shallow_clone(
        &self,
        w: &mut Twine,
        name: &str,
        base_type: &str,
        dimensions: &[usize],
    ) {
        if dimensions.len() == 1 {
            let dim = dimensions[0];
            w!(w, "this.", name, " = java.util.Arrays.copyOf(other.", name, ", ", dim, ");\n");
            return;
        }

        w!(w, "this.", name, " = new ", base_type);
        for (i, dim) in dimensions.iter().enumerate() {
            if i == 0 {
                w!(w, "[", dim, "]");
            } else {
                w!(w, "[]");
            }
        }
        w!(w, ";\n");

        for (idx, dim) in dimensions.iter().enumerate().take(dimensions.len() - 1) {
            let var = format!("_i{idx}");
            w!(w, "for (int ", var, " = 0; ", var, " < ", dim, "; ", var, "++) {\n");
        }

        let mut indices = String::new();
        for idx in 0..dimensions.len() - 1 {
            _ = write!(indices, "[_i{idx}]");
        }

        let last_dim = dimensions[dimensions.len() - 1];
        w!(w, "this.", name, indices, " = java.util.Arrays.copyOf(other.", name, indices, ", ", last_dim, ");\n");

        for _ in 0..dimensions.len() - 1 {
            w!(w, "}\n");
        }
    }

    fn emit_arg_ctor(&self, w: &mut Twine, def_id: DefId, members: &[Member]) {
        let def = self.hir.context.definitions.get(def_id);
        w!(w, "public ", def.ident.name, "(");

        for (i, member) in members.iter().enumerate() {
            if i > 0 {
                w!(w, ",\n");
            } else if members.len() > 1 {
                w!(w, "\n");
            }

            let java_type = self.java_type(&member.ty, def_id);
            w!(w, java_type, " ", member.ident.name);
        }

        w!(w, "\n) {\n");

        for member in members {
            w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_accessors(&self, w: &mut Twine, def_id: DefId, members: &[Member]) {
        for member in members {
            let java_type = self.java_type(&member.ty, def_id);
            let (getter, setter) = self.get_set(&member.ident.name);

            w!(w, "public ", java_type, " ", getter, "() {\n");
            w!(w, "return this.", member.ident.name, ";\n");
            w!(w, "}\n\n");

            w!(w, "public void ", setter, "(",  java_type, " ", member.ident.name, ") {\n");
            w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
            w!(w, "}\n\n");
        }
    }

    fn emit_except(&self, w: &mut Twine, def: &Def, except_ty: &ExceptTy) {
        w!(w, "public class ", def, " extends java.lang.RuntimeException {\n");
        let uid = self.serial_version_uid(def, &except_ty.members);
        w!(w, "private static final long serialVersionUID = ", uid, "L;\n\n");

        self.emit_default_ctor(w, def.id, &except_ty.members);
        self.emit_copy_ctor(w, def, None, &except_ty.members);
        if !except_ty.members.is_empty() {
            self.emit_arg_ctor(w, def.id, &except_ty.members);
        }

        self.emit_accessors(w, def.id, &except_ty.members);

        for member in &except_ty.members {
            let java_type = self.java_type(&member.ty, def.id);
            w!(w, "private ", java_type, " ", member.ident.name, ";\n");
        }
        w!(w, "}\n");
    }

    fn emit_enum(&self, w: &mut Twine, def: &Def, enum_ty: &EnumTy) {
        w!(w, "public enum ", def, " {\n");
        self.emit_enumerators(w, &enum_ty.fields);
        w!(w, "\n");

        w!(w, "private ", def, "(int value) {\n");
        w!(w, "_value = value;\n");
        w!(w, "}\n\n");

        w!(w, "public final int getValue() {\n");
        w!(w, "return _value;\n");
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
        w!(w, "throw new java.lang.IllegalArgumentException(\"invalid ", def, " value: \" + val);\n");
        w!(w, "}\n\n");

        w!(w, "private final int _value;\n");
        w!(w, "}\n");
    }

    fn emit_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let disc_type = self.java_type(&union_ty.disc.ty, def.id);
        let (disc_get, disc_set) = self.disc_get_set();

        w!(w, "public final class ", def, " implements java.io.Serializable {\n");
        let uid = self.serial_version_uid_union(def, union_ty);
        w!(w, "private static final long serialVersionUID = ", uid, "L;\n\n");

        self.emit_union_constructors(w, def, union_ty);

        w!(w, "@Override\n");
        w!(w, "public ", def.ident.name, " clone() {\n");
        w!(w, "return new ", def.ident.name, "(this);\n");
        w!(w, "}\n\n");

        self.emit_union_clear(w, union_ty);
        self.emit_union_accessors(w, def, union_ty);

        w!(w, "public ", disc_type, " ", disc_get, "() {\n");
        w!(w, "return discriminator;\n");
        w!(w, "}\n\n");

        w!(w, "public void ", disc_set, "(", disc_type, " discriminator) {\n");
        w!(w, "if (this.discriminator != discriminator) {\n");
        w!(w, "this.discriminator = discriminator;\n");
        w!(w, "_clear();\n");
        w!(w, "switch (discriminator) {\n");

        for variant in &union_ty.variants {
            self.emit_variant_cases(w, &union_ty.disc.ty, variant, def.id);
            let default_val = self.default_value(&variant.ty, def.id);
            w!(w, "this.", variant.ident.name, " = ", default_val, ";\n");
            w!(w, "break;\n");
        }

        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        self.emit_union_equals(w, def, union_ty);
        self.emit_union_hashcode(w, union_ty);

        w!(w, "private ", disc_type, " discriminator;\n");
        for variant in &union_ty.variants {
            let java_type = self.java_type(&variant.ty, def.id);
            w!(w, "protected ", java_type, " ", variant.ident.name, ";\n");
        }

        w!(w, "}\n");
    }

    fn emit_union_equals(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        w!(w, "@Override\n");
        w!(w, "public boolean equals(Object obj) {\n");
        w!(w, "if (this == obj) {\n");
        w!(w, "return true;\n");
        w!(w, "}\n");
        w!(w, "if (obj == null || getClass() != obj.getClass()) {\n");
        w!(w, "return false;\n");
        w!(w, "}\n");
        w!(w, def, " other = (", def, ") obj;\n");

        let disc_resolved = self.hir.context.resolve_ty(&union_ty.disc.ty);
        if let TyKind::Adt(_) = &disc_resolved.kind {
            w!(w, "if (!java.util.Objects.equals(discriminator, other.discriminator)) {\n");
        } else {
            w!(w, "if (discriminator != other.discriminator) {\n");
        }
        w!(w, "return false;\n");
        w!(w, "}\n");

        w!(w, "switch (discriminator) {\n");
        for variant in &union_ty.variants {
            self.emit_variant_cases(w, &union_ty.disc.ty, variant, def.id);
            let resolved = self.hir.context.resolve_ty(&variant.ty);
            let name = &variant.ident.name;
            match &resolved.kind {
                TyKind::Primitive(prim) => match prim {
                    PrimitiveTy::Float32 => {
                        w!(w, "return Float.compare(", name, ", other.", name, ") == 0;\n");
                    }
                    PrimitiveTy::Float64 | PrimitiveTy::Float128 => {
                        w!(w, "return Double.compare(", name, ", other.", name, ") == 0;\n");
                    }
                    _ => {
                        w!(w, "return ", name, " == other.", name, ";\n");
                    }
                },
                TyKind::Array { ty: inner, .. } => {
                    let inner_resolved = self.hir.context.resolve_ty(inner);
                    if matches!(inner_resolved.kind, TyKind::Primitive(_)) {
                        w!(w, "return java.util.Arrays.equals(", name, ", other.", name, ");\n");
                    } else {
                        w!(w, "return java.util.Arrays.deepEquals(", name, ", other.", name, ");\n");
                    }
                }
                _ => {
                    w!(w, "return java.util.Objects.equals(", name, ", other.", name, ");\n");
                }
            }
        }
        w!(w, "}\n");
        w!(w, "return true;\n");
        w!(w, "}\n\n");
    }

    fn emit_union_hashcode(&self, w: &mut Twine, union_ty: &UnionTy) {
        w!(w, "@Override\n");
        w!(w, "public int hashCode() {\n");
        w!(w, "int result = java.util.Objects.hashCode(discriminator);\n");
        w!(w, "switch (discriminator) {\n");
        for variant in &union_ty.variants {
            self.emit_variant_cases_no_def(w, variant);
            let resolved = self.hir.context.resolve_ty(&variant.ty);
            let name = &variant.ident.name;
            if let TyKind::Array { ty: inner, .. } = &resolved.kind {
                let inner_resolved = self.hir.context.resolve_ty(inner);
                if matches!(inner_resolved.kind, TyKind::Primitive(_)) {
                    w!(w, "result = 31 * result + java.util.Arrays.hashCode(", name, ");\n");
                } else {
                    w!(w, "result = 31 * result + java.util.Arrays.deepHashCode(", name, ");\n");
                }
            } else {
                w!(w, "result = 31 * result + java.util.Objects.hashCode(", name, ");\n");
            }
            w!(w, "break;\n");
        }
        w!(w, "}\n");
        w!(w, "return result;\n");
        w!(w, "}\n\n");
    }

    fn emit_variant_cases_no_def(&self, w: &mut Twine, variant: &Variant) {
        w.dedent();
        for label in &variant.labels {
            if let Some(val) = self.hir.context.integer_value(&label.value) {
                w!(w, "case ", val, ":\n");
            }
        }
        if variant.is_default {
            w!(w, "default:\n");
        }
        w.indent();
    }

    fn emit_union_clear(&self, w: &mut Twine, union_ty: &UnionTy) {
        w!(w, "private void _clear() {\n");
        for variant in &union_ty.variants {
            let resolved = self.hir.context.resolve_ty(&variant.ty);
            let clear_value = match &resolved.kind {
                TyKind::Primitive(prim) => match prim {
                    PrimitiveTy::Bool => "false",
                    PrimitiveTy::Char | PrimitiveTy::WChar => "'\\0'",
                    PrimitiveTy::Float32 => "0.0f",
                    PrimitiveTy::Float64 | PrimitiveTy::Float128 => "0.0",
                    PrimitiveTy::Int64 | PrimitiveTy::UInt64 => "0L",
                    _ => "0",
                },
                _ => "null",
            };
            w!(w, "this.", variant.ident.name, " = ", clear_value, ";\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_union_accessors(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let disc_type = self.java_type(&union_ty.disc.ty, def.id);
        let (_disc_get, disc_set) = self.disc_get_set();

        for variant in &union_ty.variants {
            let java_type = self.java_type(&variant.ty, def.id);
            let (getter, setter) = self.get_set(&variant.ident.name);

            w!(w, "public ", java_type, " ", getter, "() {\n");
            self.emit_variant_discriminator_check(w, &union_ty.disc.ty, variant, union_ty, def.id);
            w!(w, "return ", variant.ident.name, ";\n");
            w!(w, "}\n\n");

            w!(w, "public void ", setter, "(", java_type, " ", variant.ident.name, ") {\n");
            if let Some(first_label) = variant.labels.first() {
                let disc_value = self.format_numeric(&first_label.value, &union_ty.disc.ty, def.id);
                w!(w, "this.discriminator = ", disc_value, ";\n");
            } else if variant.is_default {
                let default_value =
                    self.find_default_discriminator_value(union_ty, &union_ty.disc.ty, def.id);
                w!(w, "this.discriminator = ", default_value, ";\n");
            }
            w!(w, "_clear();\n");
            w!(w, "this.", variant.ident.name, " = ", variant.ident.name, ";\n");
            w!(w, "}\n\n");

            if variant.labels.len() > 1 {
                w!(w, "public void ", setter, "(", java_type, " ", variant.ident.name, ", ", disc_type, " discriminator) {\n");
                w!(w, disc_set, "(discriminator);\n");
                w!(w, "this.", variant.ident.name, " = ", variant.ident.name, ";\n");
                w!(w, "}\n\n");
            }
        }
    }

    fn emit_union_constructors(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let (disc_get, disc_set) = self.disc_get_set();

        w!(w, "public ", def, "() {\n");
        if let Some(first_variant) = union_ty.variants.first() {
            if let Some(first_label) = first_variant.labels.first() {
                let disc_value = self.format_numeric(&first_label.value, &union_ty.disc.ty, def.id);
                w!(w, disc_set, "(", disc_value, ");\n");
            }
        }
        w!(w, "}\n\n");

        w!(w, "public ", def, "(", def, " other) {\n");
        w!(w, disc_set, "(other.", disc_get, "());\n");
        w!(w, "switch (", disc_get, "()) {\n");

        for variant in &union_ty.variants {
            self.emit_variant_cases(w, &union_ty.disc.ty, variant, def.id);
            self.emit_field_copy(w, &variant.ident.name, &variant.ty, def.id);
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

    fn emit_variant_discriminator_check(
        &self,
        w: &mut Twine,
        disc_ty: &Ty,
        variant: &Variant,
        union_ty: &UnionTy,
        def_id: DefId,
    ) {
        if variant.labels.is_empty() && !variant.is_default {
            return;
        }

        let resolved = self.hir.context.resolve_ty(disc_ty);
        let is_enum = matches!(resolved.kind, TyKind::Adt(_));

        let labels: Vec<_> = if variant.is_default {
            union_ty
                .variants
                .iter()
                .filter(|v| !v.is_default)
                .flat_map(|v| &v.labels)
                .collect()
        } else {
            variant.labels.iter().collect()
        };

        if labels.is_empty() {
            return;
        }

        w!(w, "if (");
        for (i, label) in labels.iter().enumerate() {
            if i > 0 {
                if variant.is_default {
                    w!(w, " || ");
                } else {
                    w!(w, " && ");
                }
            }
            let disc_value = self.format_numeric(&label.value, disc_ty, def_id);
            if is_enum {
                if variant.is_default {
                    w!(w, "java.util.Objects.equals(discriminator, ", disc_value, ")");
                } else {
                    w!(w, "!java.util.Objects.equals(discriminator, ", disc_value, ")");
                }
            } else if variant.is_default {
                w!(w, "discriminator == ", disc_value);
            } else {
                w!(w, "discriminator != ", disc_value);
            }
        }
        w!(w, ") {\n");
        w!(w, "throw new IllegalStateException(\"Invalid union access: discriminator is \" + discriminator);\n");
        w!(w, "}\n");
    }

    // TODO: we should do this in the HIR and inject the value there instead
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
            TyKind::Primitive(_) => {
                let mut val = 0i64;
                while used_values.contains(&val) {
                    val += 1;
                }
                format_primitive_value(val, &resolved_ty)
            }
            TyKind::Adt(adt_def_id) => {
                let adt_def = self.hir.context.definitions.get(*adt_def_id);
                if let DefKind::Enum(enum_ty) = &adt_def.kind {
                    for &field_id in &enum_ty.fields {
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

    fn emit_bitmask(&self, w: &mut Twine, def: &Def, bitmask_ty: &BitmaskTy) {
        w!(w, "public final class ", def, " {\n");
        w!(w, "private ", def, "() {}\n\n");

        for &flag_id in &bitmask_ty.flags {
            let flag_def = self.hir.context.definitions.get(flag_id);
            if let DefKind::Const(const_ty) = &flag_def.kind {
                let value = self.hir.context.integer_value(&const_ty.value).unwrap_or(0);
                w!(w, "public static final int ", flag_def.ident.name, " = ", value, ";\n");
            }
        }

        w!(w, "}\n");
    }

    fn emit_proto(&self, w: &mut Twine, def: &Def, proto: &ProtoTy, is_abstract: bool) {
        let proto_ty = self.java_type(&proto.ty, def.id);
        w!(w, "public ");

        if is_abstract {
            w!(w, "abstract ");
        }
        w!(w, proto_ty, " ", proto.ident.name, "(");

        for (i, param) in proto.params.iter().enumerate() {
            let param_ty = if param.kind == ParamKind::In {
                self.java_type(&param.ty, def.id)
            } else {
                format!("Holder<{}>", self.boxed_java_type(&param.ty, def.id))
            };
            w!(w, "\n", param_ty, " ", param.ident.name);

            if i < proto.params.len() - 1 {
                w!(w, ",");
            }
        }

        w!(w, ")");

        if !proto.raises.is_empty() {
            let exceptions = proto
                .raises
                .iter()
                .map(|e| self.scoped_name(*e, def.id))
                .join(", ");

            w!(w, " throws ", exceptions);
        }
    }

    fn emit_interface(&self, w: &mut Twine, def: &Def, interface_ty: &InterfaceTy) {
        w!(w, "public interface ", def, " {\n");

        for proto in &interface_ty.prototypes {
            self.emit_proto(w, def, proto, false);
            w!(w, ";\n");
        }

        w!(w, "}\n");
    }

    fn emit_abstract_valuetype(&self, w: &mut Twine, def: &Def, value_ty: &ValueTy) {
        w!(w, "public abstract class ", def, "Abstract");

        if let Some(extends) = value_ty.parent {
            let name = self.java_name(extends);
            w!(w, " extends ", name);
        }

        if let Some(supports) = value_ty.supports {
            let name = self.java_name(supports);
            w!(w, " implements ", name);
        }
        w!(w, " {\n");

        for proto in &value_ty.prototypes {
            self.emit_proto(w, def, proto, true);
            w!(w, ";\n\n");
        }

        for attr in &value_ty.attributes {
            let attr_ty = self.java_type(&attr.ty, def.id);
            w!(w, "public ", attr_ty, " ", attr.ident.name, ";\n");
        }

        for mem in &value_ty.members {
            let attr_ty = self.java_type(&mem.ty, def.id);
            w!(w, "public ", attr_ty, " ", mem.ident.name, ";\n");
        }

        w!(w, "}\n");
    }

    fn emit_valuetype(&self, w: &mut Twine, def: &Def, value_ty: &ValueTy) {
        w!(w, "public class ", def, " extends ", def, "Abstract {\n");
        w!(w, "public ", def, "() {}\n\n");

        for proto in &value_ty.prototypes {
            w!(w, "@Override\n");
            self.emit_proto(w, def, proto, false);
            w!(w, "\n{\n");
            w!(w, "throw new java.lang.UnsupportedOperationException(\"not implemented\");\n");
            w!(w, "}\n\n");
        }

        w!(w, "}\n");
    }

    fn emit_const(&self, w: &mut Twine, def: &Def, const_ty: &ConstTy) {
        w!(w, "public interface ", def, " {\n");
        let java_type = self.java_type(&const_ty.ty, def.id);
        let value = self.format_numeric(&const_ty.value, &const_ty.ty, def.id);

        w!(w, "public static final ", java_type, " value = ", value, ";\n");
        w!(w, "}\n");
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
            Numeric::String(s) => escape_java_string(s),
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

    pub fn generate(&self) -> Vec<File> {
        let mut result = vec![];
        for &def_id in &self.hir.order {
            self.emit_def(def_id, &mut result);
        }
        result
    }
}
