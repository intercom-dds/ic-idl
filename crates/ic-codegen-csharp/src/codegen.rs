// Copyright 2024 KONGSBERG
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

use std::collections::HashMap;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::hir::{
    Ann, Attribute, BitmaskTy, ConstTy, Def, DefFlags, DefId, DefKind, EnumTy, ExceptTy,
    InterfaceTy, ModuleTy, Numeric, ParamKind, PrimitiveTy, ProtoTy, StructTy, Ty, TyKind, UnionTy,
    ValueTy,
};
use ic_hir::{Context, ResolvedGraph};
use ic_vfs::{FileId, SourceMap};

use crate::CSharpOptions;

pub struct CSharpGen<'a> {
    hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    options: CSharpOptions,
}

impl<'a> CSharpGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: CSharpOptions) -> Self {
        Self {
            hir,
            source_map,
            options,
        }
    }

    fn primitive_type(prim: PrimitiveTy) -> &'static str {
        match prim {
            PrimitiveTy::Void => "void",
            PrimitiveTy::Bool => "bool",
            PrimitiveTy::Char | PrimitiveTy::WChar => "char",
            PrimitiveTy::Int8 => "sbyte",
            PrimitiveTy::UInt8 => "byte",
            PrimitiveTy::Int16 => "short",
            PrimitiveTy::UInt16 => "ushort",
            PrimitiveTy::Int32 => "int",
            PrimitiveTy::UInt32 => "uint",
            PrimitiveTy::Int64 => "long",
            PrimitiveTy::UInt64 => "ulong",
            PrimitiveTy::Float32 => "float",
            PrimitiveTy::Float64 => "double",
            PrimitiveTy::Float128 => "decimal",
        }
    }

    /// Format a numeric value without type suffixes
    fn format_numeric_bare(value: &Numeric) -> String {
        match value {
            Numeric::Null => "null".to_string(),
            Numeric::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Numeric::Char(c) => format!("'{}'", c.escape_default()),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::Int64(v) => v.to_string(),
            Numeric::UInt64(v) => v.to_string(),
            Numeric::Float(v) => format!("{v}f"),
            Numeric::Double(v) => format!("{v}d"),
            Numeric::String(s) => format!("\"{}\"", s.escape_default()),
            _ => String::new(),
        }
    }

    fn format_numeric(&self, value: &Numeric, relative_to_def_id: DefId) -> String {
        match value {
            Numeric::Null => "null".to_string(),
            Numeric::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Numeric::Char(c) => format!("'{}'", c.escape_default()),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => format!("{v}U"),
            Numeric::Int64(v) => format!("{v}L"),
            Numeric::UInt64(v) => format!("{v}UL"),
            Numeric::Float(v) => format!("{v}f"),
            Numeric::Double(v) => format!("{v}d"),
            Numeric::String(s) => format!("\"{}\"", s.escape_default()),
            Numeric::Const(def_id) => self.scoped_name(*def_id, relative_to_def_id),
            Numeric::Array { values, .. } => {
                let formatted: Vec<_> = values
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();
                format!("new[] {{ {} }}", formatted.join(", "))
            }
            Numeric::Sequence { values, .. } => {
                let formatted: Vec<_> = values
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();
                format!("new List<> {{ {} }}", formatted.join(", "))
            }
            Numeric::Map { entries, .. } => {
                let formatted: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{{ {}, {} }}",
                            self.format_numeric(k, relative_to_def_id),
                            self.format_numeric(v, relative_to_def_id),
                        )
                    })
                    .collect();
                format!("new Dictionary<,> {{ {} }}", formatted.join(", "))
            }
            Numeric::Struct { ty, fields } => {
                let struct_name = self.scoped_name(*ty, relative_to_def_id);
                let formatted: Vec<_> = fields
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();
                format!("new {}({})", struct_name, formatted.join(", "))
            }
            Numeric::Union { .. } => String::new(),
        }
    }

    fn get_scope(&self, def_id: DefId) -> Option<DefId> {
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

    fn common_scope(&self, def_id1: DefId, def_id2: DefId) -> Option<DefId> {
        let scope1 = self.get_scope(def_id1)?;
        let scope2 = self.get_scope(def_id2)?;

        let mut ancestors1 = Vec::new();
        let mut current = scope1;
        loop {
            ancestors1.push(current);
            let def = self.hir.context.definitions.get(current);
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

        let mut current = scope2;
        loop {
            if ancestors1.contains(&current) {
                return Some(current);
            }
            let def = self.hir.context.definitions.get(current);
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

        None
    }

    fn build_path_from(&self, from_scope: DefId, to_scope: Option<DefId>) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = from_scope;

        loop {
            if Some(current) == to_scope {
                break;
            }

            let def = self.hir.context.definitions.get(current);
            // Use I prefix for interfaces in path
            let name = if matches!(def.kind, DefKind::Interface(_)) {
                format!("I{}", def.ident.name)
            } else {
                def.ident.name.clone()
            };
            path.push(name);

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
        let target_def = self.hir.context.type_of(target_def_id);
        let type_name = &target_def.ident.name;

        // Enum fields and bitmask flags need special handling
        if let Some(parent_id) = target_def.parent {
            let parent_def = self.hir.context.type_of(parent_id);
            match &parent_def.kind {
                DefKind::Enum(_) | DefKind::Bitmask(_) => {
                    let bitmask_name = self.scoped_name(parent_id, relative_to_def_id);
                    return format!("{bitmask_name}.{type_name}");
                }
                _ => {}
            }
        }

        let target_scope = self.get_scope(target_def_id);
        let current_scope = self.get_scope(relative_to_def_id);

        match (target_scope, current_scope) {
            (None, None) => type_name.clone(),
            (None, Some(_)) => format!("global::{type_name}"),
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join(".");
                if pkg_name.is_empty() {
                    type_name.clone()
                } else {
                    format!("{pkg_name}.{type_name}")
                }
            }
            (Some(target_scope), Some(current_scope)) => {
                if target_scope == current_scope {
                    return type_name.clone();
                }

                let common = self.common_scope(target_def_id, relative_to_def_id);
                if common == Some(current_scope) {
                    let relative_path = self.build_path_from(target_scope, common);
                    let pkg_name = relative_path.join(".");
                    format!("{pkg_name}.{type_name}")
                } else {
                    let full_path = self.build_path_from(target_scope, None);
                    let pkg_name = full_path.join(".");
                    if pkg_name.is_empty() {
                        type_name.clone()
                    } else {
                        format!("{pkg_name}.{type_name}")
                    }
                }
            }
        }
    }

    /// Check if a type is a C# value type (struct/primitive) vs reference type.
    /// Value types need `.Value` to unwrap from nullable, reference types use `!`.
    fn is_value_type(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Primitive(_) | TyKind::Fixed => true,
            TyKind::Adt(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                matches!(def.kind, DefKind::Enum(_) | DefKind::Bitmask(_))
            }
            _ => false,
        }
    }

    /// Convert IDL type to C# type string
    fn csharp_type(&self, ty: &Ty, relative_def: DefId) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => Self::primitive_type(*prim).to_string(),
            TyKind::String { .. } => "string".to_string(),
            TyKind::Adt(def_id) => self.scoped_name(*def_id, relative_def),
            TyKind::Sequence { ty, .. } => {
                let inner = self.csharp_type(ty, relative_def);
                format!("IList<{inner}>")
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.csharp_type(key, relative_def);
                let elem_ty = self.csharp_type(elem, relative_def);
                format!("IDictionary<{key_ty}, {elem_ty}>")
            }
            TyKind::Array { ty, .. } => {
                let (base_ty, dims) = Self::count_array_dimensions(ty);
                let base_ty_str = self.csharp_type(&base_ty, relative_def);
                let commas = ",".repeat(dims);
                format!("{base_ty_str}[{commas}]")
            }
            TyKind::Any => "object".to_string(),
            TyKind::Fixed => "decimal".to_string(),
            TyKind::Null => "void".to_string(),
        }
    }

    /// Returns the default initializer for reference types, `None` for value types.
    fn default_initializer(&self, ty: &Ty, relative_def: DefId) -> Option<String> {
        match &ty.kind {
            TyKind::String { .. } => Some("\"\"".to_string()),
            TyKind::Sequence { ty: inner, .. } => {
                let inner_ty = self.csharp_type(inner, relative_def);
                Some(format!("new List<{inner_ty}>()"))
            }
            TyKind::Map { key, elem, .. } => {
                let key_ty = self.csharp_type(key, relative_def);
                let elem_ty = self.csharp_type(elem, relative_def);
                Some(format!("new Dictionary<{key_ty}, {elem_ty}>()"))
            }
            TyKind::Array { .. } => {
                let dims = Self::collect_array_dimensions(ty);
                let (base_ty, _) = Self::count_array_dimensions(ty);
                let base_ty_str = self.csharp_type(&base_ty, relative_def);
                let dim_str = dims
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("new {base_ty_str}[{dim_str}]"))
            }
            TyKind::Any => Some("null!".to_string()),
            TyKind::Adt(def_id) => {
                let def = self.hir.context.type_of(*def_id);
                match &def.kind {
                    DefKind::Enum(enum_ty) => {
                        Some(self.scoped_name(enum_ty.fields[0], relative_def))
                    }
                    DefKind::Bitmask(_) => None,
                    // Valuetypes and interfaces are abstract, can't instantiate
                    DefKind::Valuetype(_) | DefKind::Interface(_) => Some("null!".to_string()),
                    _ => {
                        let type_name = self.scoped_name(*def_id, relative_def);
                        Some(format!("new {type_name}()"))
                    }
                }
            }
            _ => None,
        }
    }

    fn emit_doc_comments(&self, w: &mut Twine, annotations: &[Ann]) {
        for ann in annotations {
            if !is_doc(&self.hir.context, ann) {
                continue;
            }

            for doc in &ann.args {
                if let Some(ty) = &doc.ty
                    && let TyKind::String { .. } = ty.kind
                    && let Some(str) = self.hir.context.string_value(&doc.value)
                {
                    let text = str.trim_end();
                    w!(w, "/// <summary>", text, "</summary>\n");
                }
            }
        }
    }

    fn emit_module(&self, w: &mut Twine, def: &Def, module: &ModuleTy) {
        // Check if this is a synthesized Constants module
        if def.flags.contains(DefFlags::IS_SYNTHESIZED) && def.ident.name == "Constants" {
            self.emit_constants_class(w, def, module);
            return;
        }

        self.emit_doc_comments(w, &def.annotations);

        w!(w, "namespace ", def.ident.name, "\n");
        w!(w, "{\n");
        for &nested_id in &module.definitions {
            self.emit_definition(w, nested_id);
            w!(w, "\n");
        }
        w!(w, "}\n");
    }

    /// Emit a synthesized `Constants` module as a static class containing constants.
    fn emit_constants_class(&self, w: &mut Twine, def: &Def, module: &ModuleTy) {
        self.emit_doc_comments(w, &def.annotations);

        w!(w, "public static class ", def.ident.name, "\n");
        w!(w, "{\n");
        for &nested_id in &module.definitions {
            self.emit_definition(w, nested_id);
            w!(w, "\n");
        }
        w!(w, "}\n\n");
    }

    fn emit_struct(&self, w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        self.emit_doc_comments(w, &def.annotations);

        // Emit class declaration with IEquatable<T>
        w!(w, "public partial class ", def.ident.name);

        // Handle inheritance
        if let Some(parent_id) = struct_ty.parent {
            let parent_name = self.scoped_name(parent_id, def.id);
            w!(w, " : ", parent_name);
        } else {
            w!(w, " : IEquatable<", def.ident.name, ">");
        }

        w!(w, "\n");
        w!(w, "{\n");

        // Emit properties for each member
        for member in &struct_ty.members {
            self.emit_doc_comments(w, &member.annotations);
            self.emit_struct_member(w, def.id, &member.ident.name, &member.ty);
            w!(w, "\n");
        }

        // Emit default constructor
        w!(w, "\n");
        w!(w, "public ", def.ident.name, "()\n");
        w!(w, "{\n");
        w!(w, "}\n");

        // Emit copy constructor
        w!(w, "\n");
        w!(w, "public ", def.ident.name, "(", def.ident.name, " other)\n");
        w!(w, "{\n");
        w!(w, "if (other is null)\n");
        w!(w, "{\n");
        w!(w, "throw new ArgumentNullException(nameof(other));\n");
        w!(w, "}\n");
        if struct_ty.parent.is_some() {
            w!(w, "// Copy base members\n");
        }
        for member in &struct_ty.members {
            let member_name = &member.ident.name;
            if let TyKind::Array { .. } = &member.ty.kind {
                w!(
                    w, "this.", member_name, " = (",
                    self.csharp_type(&member.ty, def.id), ")other.", member_name, ".Clone();\n",
                );
            } else {
                w!(w, "this.", member_name, " = other.", member_name, ";\n");
            }
        }
        w!(w, "}\n");

        // Emit constructor with all fields
        if !struct_ty.members.is_empty() {
            w!(w, "\n");
            w!(w, "public ", def.ident.name, "(");
            for (i, member) in struct_ty.members.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let ty_str = self.csharp_type(&member.ty, def.id);
                w!(w, ty_str, " ", member.ident.name);
            }
            w!(w, ")\n");
            w!(w, "{\n");
            for member in &struct_ty.members {
                w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
            }
            w!(w, "}\n");
        }

        // Emit Equals method
        self.emit_struct_equals(w, def, struct_ty);

        // Emit GetHashCode method
        Self::emit_struct_hashcode(w, struct_ty);

        w!(w, "}\n\n");
    }

    /// Emit a struct member property, with array bounds validation if applicable.
    fn emit_struct_member(&self, w: &mut Twine, def_id: DefId, name: &str, ty: &Ty) {
        if matches!(ty.kind, TyKind::Array { .. }) {
            // Array with bounds - emit backing field + validated property
            let ty_str = self.csharp_type(ty, def_id);
            let backing_field = format!("_{name}");

            // Collect all dimensions for rectangular array initialization
            let dims = Self::collect_array_dimensions(ty);
            let (base_ty, _) = Self::count_array_dimensions(ty);
            let base_ty_str = self.csharp_type(&base_ty, def_id);
            let dims_str = dims
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            // Backing field with initializer
            w!(w, "private ", &ty_str, " ", &backing_field, " = new ", &base_ty_str, "[", &dims_str, "];\n");

            // Property with validation of all dimensions
            w!(w, "public ", &ty_str, " ", name, "\n");
            w!(w, "{\n");
            w!(w, "get => ", &backing_field, ";\n");
            w!(w, "set\n");
            w!(w, "{\n");
            w!(w, "if (value is null");
            for (i, dim) in dims.iter().enumerate() {
                if dims.len() == 1 {
                    w!(w, " || value.Length != ", dim);
                } else {
                    w!(w, " || value.GetLength(", i, ") != ", dim);
                }
            }
            w!(w, ")\n");
            w!(w, "{\n");
            w!(w, "throw new ArgumentOutOfRangeException(nameof(value), \"Array dimensions must match\");\n");
            w!(w, "}\n");
            w!(w, &backing_field, " = value;\n");
            w!(w, "}\n");
            w!(w, "}");
        } else {
            let ty_str = self.csharp_type(ty, def_id);
            w!(w, "public ", ty_str, " ", name, " { get; set; }");
            if let Some(init) = self.default_initializer(ty, def_id) {
                w!(w, " = ", init, ";");
            }
        }
    }

    /// Count array dimensions and return the base (non-array) type.
    /// For `Array { ty: Array { ty: double, len: 3 }, len: 3 }`, returns `(double, 1)`.
    fn count_array_dimensions(ty: &Ty) -> (Ty, usize) {
        match &ty.kind {
            TyKind::Array { ty: inner, .. } => {
                let (base, dims) = Self::count_array_dimensions(inner);
                (base, dims + 1)
            }
            _ => (ty.clone(), 0),
        }
    }

    /// Collect array dimension sizes for rectangular array initialization.
    /// For `Array { ty: Array { ty: double, len: 3 }, len: 2 }`, returns `[2, 3]`.
    fn collect_array_dimensions(ty: &Ty) -> Vec<usize> {
        match &ty.kind {
            TyKind::Array { ty: inner, len, .. } => {
                let mut dims = vec![*len];
                dims.extend(Self::collect_array_dimensions(inner));
                dims
            }
            _ => vec![],
        }
    }

    fn emit_struct_equals(&self, w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        let name = &def.ident.name;

        // IEquatable<T>.Equals
        w!(w, "\n");
        w!(w, "public bool Equals(", name, "? other)\n");
        w!(w, "{\n");
        w!(w, "if (other is null)\n");
        w!(w, "{\n");
        w!(w, "return false;\n");
        w!(w, "}\n");
        w!(w, "if (ReferenceEquals(this, other))\n");
        w!(w, "{\n");
        w!(w, "return true;\n");
        w!(w, "}\n");

        if struct_ty.parent.is_some() {
            w!(w, "if (!base.Equals(other))\n");
            w!(w, "{\n");
            w!(w, "return false;\n");
            w!(w, "}\n");
        }

        for member in &struct_ty.members {
            let member_name = &member.ident.name;
            w!(
                w,
                "if (!EqualityComparer<", self.csharp_type(&member.ty, def.id),
                ">.Default.Equals(", member_name, ", other.", member_name, "))\n",
            );
            w!(w, "{\n");
            w!(w, "return false;\n");
            w!(w, "}\n");
        }

        w!(w, "return true;\n");
        w!(w, "}\n");

        // object.Equals override
        w!(w, "\n");
        w!(w, "public override bool Equals(object? obj)\n");
        w!(w, "{\n");
        w!(w, "return Equals(obj as ", name, ");\n");
        w!(w, "}\n");
    }

    fn emit_struct_hashcode(w: &mut Twine, struct_ty: &StructTy) {
        w!(w, "\n");
        w!(w, "public override int GetHashCode()\n");
        w!(w, "{\n");
        w!(w, "HashCode hash = new HashCode();\n");

        if struct_ty.parent.is_some() {
            w!(w, "hash.Add(base.GetHashCode());\n");
        }

        for member in &struct_ty.members {
            w!(w, "hash.Add(", member.ident.name, ");\n");
        }

        w!(w, "return hash.ToHashCode();\n");
        w!(w, "}\n");
    }

    fn emit_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        self.emit_doc_comments(w, &def.annotations);

        let disc_ty = self.csharp_type(&union_ty.disc.ty, def.id);
        let name = &def.ident.name;

        w!(w, "public partial class ", name, " : IEquatable<", name, ">\n");
        w!(w, "{\n");

        // Discriminator property (read-only per spec)
        w!(w, "public ", disc_ty, " Discriminator { get; private set; }\n");

        // Private field for each variant
        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }
            let ty_str = self.csharp_type(&variant.ty, def.id);
            w!(w, "private ", ty_str, "? _", variant.ident.name, ";\n");
        }

        // Default constructor
        w!(w, "\n");
        w!(w, "public ", name, "()\n");
        w!(w, "{\n");
        w!(w, "}\n");

        // Copy constructor
        w!(w, "\n");
        w!(w, "public ", name, "(", name, " other)\n");
        w!(w, "{\n");
        w!(w, "if (other is null)\n");
        w!(w, "{\n");
        w!(w, "throw new ArgumentNullException(nameof(other));\n");
        w!(w, "}\n");
        w!(w, "Discriminator = other.Discriminator;\n");
        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }
            let var_name = &variant.ident.name;
            w!(w, "_", var_name, " = other._", var_name, ";\n");
        }
        w!(w, "}\n");

        // Property for each variant with getter/setter that manages discriminator
        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }

            w!(w, "\n");
            self.emit_doc_comments(w, &variant.annotations);

            let ty_str = self.csharp_type(&variant.ty, def.id);
            let var_name = &variant.ident.name;

            // Build condition for discriminator check (true when variant is NOT active)
            let inactive_condition = self.build_inactive_condition(variant, union_ty, def.id);

            w!(w, "public ", ty_str, " ", var_name, "\n");
            w!(w, "{\n");

            // Getter with validation (spec: throw InvalidOperationException if not set)
            w!(w, "get\n");
            w!(w, "{\n");
            if !inactive_condition.is_empty() {
                w!(w, "if (", inactive_condition, ")\n");
                w!(w, "{\n");
                w!(
                    w,
                    "throw new InvalidOperationException(\"Member '",
                    var_name,
                    "' is not active for current discriminator value.\");\n",
                );
                w!(w, "}\n");
            }

            if self.is_value_type(&variant.ty) {
                w!(w, "return _", var_name, "!.Value;\n");
            } else {
                w!(w, "return _", var_name, "!;\n");
            }
            w!(w, "}\n");

            // Setter
            w!(w, "set\n");
            w!(w, "{\n");

            // Set discriminator to first label value
            if let Some(label) = variant.labels.first() {
                let label_val = self.format_numeric(&label.value, def.id);
                w!(w, "Discriminator = ", label_val, ";\n");
            }
            w!(w, "_", var_name, " = value;\n");

            // Clear other variants
            for other in &union_ty.variants {
                if other.ident.name != *var_name && !matches!(other.ty.kind, TyKind::Null) {
                    w!(w, "_", other.ident.name, " = default;\n");
                }
            }

            w!(w, "}\n");
            w!(w, "}\n");

            // If variant has multiple labels, emit Set<MemberName> modifier method
            if variant.labels.len() > 1 {
                self.emit_union_set_method(w, def, variant, &disc_ty);
            }
        }

        // Emit Equals method (IEquatable<T>)
        self.emit_union_equals(w, def, union_ty);

        // Emit GetHashCode
        Self::emit_union_hashcode(w, union_ty);

        w!(w, "}\n\n");
    }

    /// Build a condition string that is true when the variant is NOT active.
    /// Used to guard property getters: `if (condition) throw ...`
    fn build_inactive_condition(
        &self,
        variant: &ic_hir::hir::Variant,
        union_ty: &UnionTy,
        relative_def: DefId,
    ) -> String {
        if variant.is_default {
            // Default is inactive when discriminator matches any explicit label
            let other_labels: Vec<String> = union_ty
                .variants
                .iter()
                .filter(|v| !v.is_default)
                .flat_map(|v| &v.labels)
                .map(|label| {
                    let label_val = self.format_numeric(&label.value, relative_def);
                    format!("Discriminator == {label_val}")
                })
                .collect();

            return other_labels.join(" || ");
        }

        // Non-default variant is inactive when discriminator doesn't match any of its labels
        let conditions: Vec<String> = variant
            .labels
            .iter()
            .map(|label| {
                let label_val = self.format_numeric(&label.value, relative_def);
                format!("Discriminator != {label_val}")
            })
            .collect();

        conditions.join(" && ")
    }

    /// Emit Set<MemberName> modifier method for union variants with multiple labels.
    fn emit_union_set_method(
        &self,
        w: &mut Twine,
        def: &Def,
        variant: &ic_hir::hir::Variant,
        disc_ty: &str,
    ) {
        let var_name = &variant.ident.name;
        let ty_str = self.csharp_type(&variant.ty, def.id);

        w!(w, "\n");
        w!(w, "public void Set", var_name, "(", ty_str, " value, ", disc_ty, " discriminator)\n");
        w!(w, "{\n");

        // Validate discriminator is one of the valid labels
        let valid_labels: Vec<String> = variant
            .labels
            .iter()
            .map(|label| self.format_numeric(&label.value, def.id))
            .collect();

        if !valid_labels.is_empty() {
            let conditions: Vec<String> = valid_labels
                .iter()
                .map(|v| format!("discriminator != {v}"))
                .collect();
            w!(w, "if (", conditions.join(" && "), ")\n");
            w!(w, "{\n");
            w!(w, "throw new ArgumentException(\"Invalid discriminator value for member '", var_name, "'.\", nameof(discriminator));\n");
            w!(w, "}\n");
        }

        w!(w, "Discriminator = discriminator;\n");
        w!(w, "_", var_name, " = value;\n");

        w!(w, "}\n");
    }

    fn emit_union_equals(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let name = &def.ident.name;

        w!(w, "\n");
        w!(w, "public bool Equals(", name, "? other)\n");
        w!(w, "{\n");

        w!(w, "if (other is null)\n");
        w!(w, "{\n");
        w!(w, "return false;\n");
        w!(w, "}\n");

        w!(w, "if (ReferenceEquals(this, other))\n");
        w!(w, "{\n");
        w!(w, "return true;\n");
        w!(w, "}\n");

        w!(
            w,
            "if (!EqualityComparer<", self.csharp_type(&union_ty.disc.ty, def.id),
            ">.Default.Equals(Discriminator, other.Discriminator))\n"
        );
        w!(w, "{\n");
        w!(w, "return false;\n");
        w!(w, "}\n");

        // Compare the active variant based on discriminator
        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }
            let var_name = &variant.ident.name;
            let ty_str = self.csharp_type(&variant.ty, def.id);
            w!(w, "if (!EqualityComparer<", ty_str, "?>.Default.Equals(_", var_name, ", other._", var_name, "))\n");
            w!(w, "{\n");
            w!(w, "return false;\n");
            w!(w, "}\n");
        }

        w!(w, "return true;\n");
        w!(w, "}\n");

        w!(w, "\n");
        w!(w, "public override bool Equals(object? obj)\n");
        w!(w, "{\n");
        w!(w, "return Equals(obj as ", name, ");\n");
        w!(w, "}\n");
    }

    fn emit_union_hashcode(w: &mut Twine, union_ty: &UnionTy) {
        w!(w, "\n");
        w!(w, "public override int GetHashCode()\n");
        w!(w, "{\n");
        w!(w, "HashCode hash = new HashCode();\n");
        w!(w, "hash.Add(Discriminator);\n");

        for variant in &union_ty.variants {
            if matches!(variant.ty.kind, TyKind::Null) {
                continue;
            }
            w!(w, "hash.Add(_", variant.ident.name, ");\n");
        }

        w!(w, "return hash.ToHashCode();\n");
        w!(w, "}\n");
    }

    fn emit_enum(&self, w: &mut Twine, def: &Def, enum_ty: &EnumTy) {
        self.emit_doc_comments(w, &def.annotations);

        let underlying_type = Self::primitive_type(enum_ty.ty);
        w!(w, "public enum ", def.ident.name, " : ", underlying_type, "\n");
        w!(w, "{\n");

        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let field_name = &field_def.ident.name;

            if i > 0 {
                w!(w, ",\n");
            }
            self.emit_doc_comments(w, &field_def.annotations);

            if field_def.flags.contains(DefFlags::IS_ENUMERATED) {
                if let DefKind::Const(const_ty) = &field_def.kind {
                    let value_str = Self::format_numeric_bare(&const_ty.value);
                    w!(w, field_name, " = ", value_str);
                } else {
                    w!(w, field_name);
                }
            } else {
                w!(w, field_name);
            }
        }

        w!(w, "\n");
        w!(w, "}\n\n");
    }

    fn emit_interface(&self, w: &mut Twine, def: &Def, interface: &InterfaceTy) {
        self.emit_doc_comments(w, &def.annotations);

        // Use "I" prefix for interface names per C# convention
        let interface_name = format!("I{}", def.ident.name);

        w!(w, "public interface ", interface_name);

        // Handle inheritance
        if !interface.parents.is_empty() {
            w!(w, " : ");
            for (i, &parent_id) in interface.parents.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let parent_name = self.scoped_name(parent_id, def.id);
                w!(w, "I", parent_name);
            }
        }

        w!(w, "\n");
        w!(w, "{\n");

        // Emit nested type definitions
        for &nested_id in &interface.definitions {
            self.emit_definition(w, nested_id);
        }

        // Emit methods (prototypes)
        for proto in &interface.prototypes {
            self.emit_prototype(w, proto, def.id);
        }

        // Emit properties (attributes)
        for attr in &interface.attributes {
            self.emit_attribute(w, attr, def.id);
        }

        w!(w, "}\n\n");
    }

    fn emit_prototype(&self, w: &mut Twine, proto: &ProtoTy, relative_to_def_id: DefId) {
        let ret_ty = self.csharp_type(&proto.ty, relative_to_def_id);
        w!(w, ret_ty, " ", proto.ident.name, "(");

        for (i, param) in proto.params.iter().enumerate() {
            if i > 0 {
                w!(w, ", ");
            }
            match param.kind {
                ParamKind::In => {}
                ParamKind::Out => w!(w, "out "),
                ParamKind::Inout => w!(w, "ref "),
            }
            let param_ty = self.csharp_type(&param.ty, relative_to_def_id);
            w!(w, param_ty, " ", param.ident.name);
        }

        w!(w, ");\n");
    }

    fn emit_attribute(&self, w: &mut Twine, attr: &Attribute, relative_to_def_id: DefId) {
        let ty_str = self.csharp_type(&attr.ty, relative_to_def_id);
        w!(w, ty_str, " ", attr.ident.name, " { get;");

        if !attr.is_readonly {
            w!(w, " set;");
        }

        w!(w, " }\n");
    }

    fn emit_valuetype(&self, w: &mut Twine, def: &Def, valuetype: &ValueTy) {
        self.emit_doc_comments(w, &def.annotations);

        w!(w, "public abstract class ", def.ident.name);

        // Handle inheritance
        if let Some(parent_id) = valuetype.parent {
            let parent_name = self.scoped_name(parent_id, def.id);
            w!(w, " : ", parent_name);

            if let Some(supports_id) = valuetype.supports {
                let supports_name = self.scoped_name(supports_id, def.id);
                w!(w, ", I", supports_name);
            }
        } else if let Some(supports_id) = valuetype.supports {
            let supports_name = self.scoped_name(supports_id, def.id);
            w!(w, " : I", supports_name);
        }

        w!(w, "\n");
        w!(w, "{\n");

        // Emit nested definitions
        for &nested_id in &valuetype.definitions {
            self.emit_definition(w, nested_id);
        }

        // Emit members as properties
        for member in &valuetype.members {
            self.emit_doc_comments(w, &member.annotations);
            let ty_str = self.csharp_type(&member.ty, def.id);

            w!(w, "public ", ty_str, " ", member.ident.name, " { get; set; }");
            if let Some(init) = self.default_initializer(&member.ty, def.id) {
                w!(w, " = ", init, ";");
            }
            w!(w, "\n");
        }

        // Emit abstract methods
        for proto in &valuetype.prototypes {
            w!(w, "\n");
            w!(w, "public abstract ");
            let ret_ty = self.csharp_type(&proto.ty, def.id);
            w!(w, ret_ty, " ", proto.ident.name, "(");

            for (i, param) in proto.params.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                match param.kind {
                    ParamKind::In => {}
                    ParamKind::Out => w!(w, "out "),
                    ParamKind::Inout => w!(w, "ref "),
                }
                let param_ty = self.csharp_type(&param.ty, def.id);
                w!(w, param_ty, " ", param.ident.name);
            }

            w!(w, ");\n");
        }

        // Emit attributes as properties
        for attr in &valuetype.attributes {
            let ty_str = self.csharp_type(&attr.ty, def.id);

            w!(w, "public ", ty_str, " ", attr.ident.name, " { get; set; }");
            if let Some(init) = self.default_initializer(&attr.ty, def.id) {
                w!(w, " = ", init, ";");
            }
            w!(w, "\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_exception(&self, w: &mut Twine, def: &Def, except: &ExceptTy) {
        self.emit_doc_comments(w, &def.annotations);

        // C# exceptions extend `System.Exception`
        w!(w, "public partial class ", def.ident.name, " : Exception\n");
        w!(w, "{\n");

        // Emit properties for each member
        for member in &except.members {
            self.emit_doc_comments(w, &member.annotations);
            let ty_str = self.csharp_type(&member.ty, def.id);

            // Use `new` keyword if member hides an inherited Exception member
            if crate::EXCEPTION_MEMBER_NAMES.contains(&member.ident.name.as_str()) {
                w!(w, "public new ", ty_str, " ", member.ident.name, " { get; set; }");
            } else {
                w!(w, "public ", ty_str, " ", member.ident.name, " { get; set; }");
            }

            if let Some(init) = self.default_initializer(&member.ty, def.id) {
                w!(w, " = ", init, ";");
            }
            w!(w, "\n");
        }

        // Default constructor
        w!(w, "\n");
        w!(w, "public ", def.ident.name, "() : base(\"", def.ident.name, "\")\n");
        w!(w, "{\n");
        w!(w, "}\n");

        // Constructor with all members
        if !except.members.is_empty() {
            w!(w, "\n");
            w!(w, "public ", def.ident.name, "(");
            for (i, member) in except.members.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let ty_str = self.csharp_type(&member.ty, def.id);
                w!(w, ty_str, " ", member.ident.name);
            }
            w!(w, ") : base(\"", def.ident.name, "\")\n");
            w!(w, "{\n");
            for member in &except.members {
                w!(w, "this.", member.ident.name, " = ", member.ident.name, ";\n");
            }
            w!(w, "}\n");
        }

        w!(w, "}\n\n");
    }

    fn emit_const(&self, w: &mut Twine, def: &Def, const_ty: &ConstTy) {
        self.emit_doc_comments(w, &def.annotations);

        let ty_str = self.csharp_type(&const_ty.ty, def.id);
        let value_str = self.format_numeric(&const_ty.value, def.id);

        // Wrap in a class if `--const-classes` is set
        if self.options.const_classes {
            w!(w, "public static class ", def.ident.name, "\n");
            w!(w, "{\n");
        }

        let value_name = if self.options.const_classes {
            "Value"
        } else {
            &def.ident.name
        };

        if is_const_eligible(&const_ty.ty) {
            w!(w, "public const ", ty_str, " ", value_name, " = ", value_str, ";\n");
        } else {
            w!(w, "public static readonly ", ty_str, " ", value_name, " = ", value_str, ";\n");
        }

        if self.options.const_classes {
            w!(w, "}\n\n");
        }
    }

    fn emit_bitmask(&self, w: &mut Twine, def: &Def, bitmask: &BitmaskTy) {
        self.emit_doc_comments(w, &def.annotations);

        let underlying_type = Self::primitive_type(bitmask.ty);
        w!(w, "[Flags]\n");
        w!(w, "public enum ", def.ident.name, " : ", underlying_type, "\n");
        w!(w, "{\n");

        for (i, &flag_id) in bitmask.flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let flag_name = &flag_def.ident.name;

            if i > 0 {
                w!(w, ",\n");
            }

            if flag_def.flags.contains(DefFlags::IS_ENUMERATED)
                && let DefKind::Const(const_ty) = &flag_def.kind
            {
                let value = Self::format_numeric_bare(&const_ty.value);
                w!(w, flag_name, " = ", value);
            } else {
                // Auto-generate power of 2 values
                let value = 1u64 << i;
                w!(w, flag_name, " = ", value);
            }
        }

        w!(w, "\n");
        w!(w, "}\n\n");
    }

    fn emit_definition(&self, w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Module(module) => self.emit_module(w, def, module),
            DefKind::Struct(struct_ty) => self.emit_struct(w, def, struct_ty),
            DefKind::Union(union_ty) => self.emit_union(w, def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(w, def, enum_ty),
            DefKind::Interface(interface) => self.emit_interface(w, def, interface),
            DefKind::Valuetype(valuetype) => self.emit_valuetype(w, def, valuetype),
            DefKind::Except(except) => self.emit_exception(w, def, except),
            DefKind::Const(const_ty) => self.emit_const(w, def, const_ty),
            DefKind::Bitmask(bitmask) => self.emit_bitmask(w, def, bitmask),
            _ => (),
        }
    }

    pub fn generate(&self) -> Vec<File> {
        const IC_VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut files_map: HashMap<FileId, Vec<DefId>> = HashMap::new();

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);
            let file_id = def.ident.span.start.file_id;
            files_map.entry(file_id).or_default().push(def_id);
        }

        let mut result = Vec::new();

        for (file_id, def_ids) in files_map {
            let file_name = self
                .source_map
                .name(file_id)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap();

            let mut w = Twine::new();

            // File header
            w!(w, "// <auto-generated>\n");
            w!(w, "// Generated by ic-idl ", IC_VERSION, "\n");
            w!(w, "// </auto-generated>\n\n");
            w!(w, "#nullable enable\n\n");

            // Common using statements
            w!(w, "using System;\n");
            w!(w, "using System.Collections.Generic;\n\n");

            // Emit all definitions
            for def_id in def_ids {
                self.emit_definition(&mut w, def_id);
            }

            let content = w.finish();
            let output_path = PathBuf::from(format!("{file_name}.cs"));

            result.push(File::Generated {
                path: output_path,
                source: content,
            });
        }

        result
    }
}

/// Check if an annotation is a documentation annotation
fn is_doc(ctx: &Context, ann: &Ann) -> bool {
    if let Some(def_id) = ann.def_id {
        let def = ctx.type_of(def_id);
        if def.flags.contains(DefFlags::IS_BUILTIN) && def.ident.name == "doc" {
            return true;
        }
    }
    false
}

/// Check if a type can be used with `const` in C#
fn is_const_eligible(ty: &Ty) -> bool {
    matches!(ty.kind, TyKind::Primitive(_) | TyKind::String { .. })
}
