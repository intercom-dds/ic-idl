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

use std::collections::HashMap;
use std::path::PathBuf;

use ic_emit::File;
use ic_emit::printer::{Twine, w};
use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    AliasTy, Ann, AnnArg, AnnotationTy, Attribute, BitmaskTy, BitsetTy, ConstTy, Decl, Def,
    DefFlags, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy, ModuleTy, Numeric, ParamKind, ProtoTy,
    StructTy, Ty, TyKind, UnionTy, ValueTy,
};
use ic_hir_analysis::annotation::doc;
use ic_vfs::{FileId, SourceMap};

use crate::IdlOptions;
use crate::deps::collect_def_dependencies;

type Path = Vec<String>;

const ANNOTATION_INLINE_MAX_WIDTH: usize = 72;

pub struct IdlGen<'a> {
    hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    options: IdlOptions,
}

impl<'a> IdlGen<'a> {
    pub fn new(hir: &'a ResolvedGraph, source_map: &'a SourceMap, options: IdlOptions) -> Self {
        Self {
            hir,
            source_map,
            options,
        }
    }

    fn idl_name(&self, def_id: DefId) -> &str {
        &self.hir.context.definitions.get(def_id).ident.name
    }

    fn format_numeric(&self, value: &Numeric, relative_to_def_id: DefId) -> String {
        match value {
            Numeric::Null => "null".to_string(),
            Numeric::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Numeric::Char(c) => format!("'{}'", c.escape_default()),
            Numeric::WChar(c) => format!("L'{}'", c.escape_default()),
            Numeric::Int8(v) => v.to_string(),
            Numeric::UInt8(v) => v.to_string(),
            Numeric::Int16(v) => v.to_string(),
            Numeric::UInt16(v) => v.to_string(),
            Numeric::Int32(v) => v.to_string(),
            Numeric::UInt32(v) => v.to_string(),
            Numeric::Int64(v) => v.to_string(),
            Numeric::UInt64(v) => v.to_string(),
            Numeric::Float(v) => v.to_string(),
            Numeric::Double(v) => v.to_string(),
            Numeric::String(s) => format!("\"{}\"", s.escape_default()),
            Numeric::WString(s) => format!("L\"{}\"", s.escape_default()),
            Numeric::Const(def_id) => self.scoped_name(*def_id, relative_to_def_id),
            Numeric::Array { values, .. } => {
                let formatted: Vec<_> = values
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();

                format!("{{{}}}", formatted.join(", "))
            }
            Numeric::Sequence { values, .. } => {
                let formatted: Vec<_> = values
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();

                format!("{{{}}}", formatted.join(", "))
            }
            Numeric::Map { entries, .. } => {
                let formatted: Vec<_> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{{{}, {}}}",
                            self.format_numeric(k, relative_to_def_id),
                            self.format_numeric(v, relative_to_def_id),
                        )
                    })
                    .collect();

                format!("{{{}}}", formatted.join(", "))
            }
            Numeric::Struct { fields, .. } => {
                let formatted: Vec<_> = fields
                    .iter()
                    .map(|v| self.format_numeric(v, relative_to_def_id))
                    .collect();

                format!("{{{}}}", formatted.join(", "))
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

    fn build_path_from(&self, from_scope: DefId, to_scope: Option<DefId>) -> Path {
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
        let type_name = self.idl_name(target_def_id);

        let target_scope = self.get_scope(target_def_id);
        let current_scope = self.get_scope(relative_to_def_id);

        match (target_scope, current_scope) {
            (None, _) => type_name.to_string(),
            (Some(target_scope), None) => {
                let full_path = self.build_path_from(target_scope, None);
                let pkg_name = full_path.join("::");
                if pkg_name.is_empty() {
                    type_name.to_string()
                } else {
                    format!("{pkg_name}::{type_name}")
                }
            }
            (Some(target_scope), Some(current_scope)) => {
                if target_scope == current_scope {
                    return type_name.to_string();
                }

                let common = self.common_scope(target_def_id, relative_to_def_id);
                if common == Some(current_scope) {
                    let relative_path = self.build_path_from(target_scope, common);
                    let pkg_name = relative_path.join("::");
                    format!("{pkg_name}::{type_name}")
                } else {
                    let full_path = self.build_path_from(target_scope, None);
                    let pkg_name = full_path.join("::");
                    if pkg_name.is_empty() {
                        type_name.to_string()
                    } else {
                        format!("{pkg_name}::{type_name}")
                    }
                }
            }
        }
    }

    fn idl_type(&self, ty: &Ty, relative_def: DefId) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => {
                if self.options.idl_legacy {
                    prim.legacy_name().to_string()
                } else {
                    prim.name().to_string()
                }
            }
            TyKind::String { wide, bound, .. } => {
                let base = if *wide { "wstring" } else { "string" };
                if let Some(b) = bound {
                    format!("{base}<{b}>")
                } else {
                    base.to_string()
                }
            }
            TyKind::Adt(def_id) => self.scoped_name(*def_id, relative_def),
            TyKind::Sequence { ty, bound, .. } => {
                let inner = self.idl_type(ty, relative_def);
                if let Some(b) = bound {
                    format!("sequence<{inner}, {b}>")
                } else {
                    format!("sequence<{inner}>")
                }
            }
            TyKind::Map {
                key, elem, bound, ..
            } => {
                let key_ty = self.idl_type(key, relative_def);
                let elem_ty = self.idl_type(elem, relative_def);
                if let Some(b) = bound {
                    format!("map<{key_ty}, {elem_ty}, {b}>")
                } else {
                    format!("map<{key_ty}, {elem_ty}>")
                }
            }
            TyKind::Array { ty, .. } => self.idl_type(ty, relative_def),
            TyKind::Any => "any".to_string(),
            TyKind::Fixed => "fixed".to_string(),
            TyKind::Null => "null".to_string(),
        }
    }

    fn emit_parents<'ctx>(
        &self,
        w: &mut Twine,
        parents: impl IntoIterator<Item = &'ctx ic_hir::hir::Spanned<DefId>>,
        relative_to_def_id: DefId,
    ) {
        let mut parents_iter = parents.into_iter().map(|p| p.def_id);
        if let Some(first) = parents_iter.next() {
            w!(w, " : ");
            let first_name = self.scoped_name(first, relative_to_def_id);
            w!(w, first_name);

            for parent in parents_iter {
                w!(w, ", ");
                let parent_name = self.scoped_name(parent, relative_to_def_id);
                w!(w, parent_name);
            }
        }
    }

    fn emit_module(&self, w: &mut Twine, def: &Def, module: &ModuleTy) {
        w!(w, "module ", def.ident.name, " {");
        for &nested_id in &module.definitions {
            w!(w, "\n");
            self.emit_definition(w, nested_id);
        }
        w!(w, "}; // module ", def.ident.name, "\n\n");
    }

    fn emit_annotations(&self, w: &mut Twine, annotations: &[Ann], relative_to_def_id: DefId) {
        // Emit doc annotations first
        for annotation in annotations {
            let Some(text) = doc(&self.hir.context, annotation) else {
                continue;
            };

            for line in text.trim_end().lines() {
                w!(w, "/// ", line, "\n");
            }
        }

        // And then non-doc annotations
        for ann in annotations {
            if doc(&self.hir.context, ann).is_some() {
                continue;
            }
            let non_default_args = filter_non_default_args(ann, self.hir);

            w!(w, "@", ann.ident.name);
            if !non_default_args.is_empty() {
                let named = non_default_args.len() > 1;
                let args: Vec<_> = non_default_args
                    .iter()
                    .map(|arg| {
                        let val_str = self.format_numeric(&arg.value, relative_to_def_id);
                        if named {
                            format!("{} = {val_str}", arg.ident.name)
                        } else {
                            val_str
                        }
                    })
                    .collect();

                if named
                    && inline_annotation_width(&ann.ident.name, &args) > ANNOTATION_INLINE_MAX_WIDTH
                {
                    w!(w, "(\n", args.join(",\n"), "\n)");
                } else {
                    w!(w, "(", args.join(", "), ")");
                }
            }
            w!(w, "\n");
        }
    }

    fn emit_struct(&self, w: &mut Twine, def: &Def, struct_ty: &StructTy) {
        w!(w, "struct ", def.ident.name);
        self.emit_parents(w, &struct_ty.parent, def.id);
        w!(w, " {");
        for member in &struct_ty.members {
            w!(w, "\n");
            self.emit_annotations(w, &member.annotations, def.id);

            let ty_str = self.idl_type(&member.ty, def.id);
            let array_bounds = format_member_name(&member.ty);
            w!(w, ty_str, " ", member.ident.name, array_bounds, ";");
        }
        w!(w, "\n};\n");
    }

    fn emit_union(&self, w: &mut Twine, def: &Def, union_ty: &UnionTy) {
        let discriminator = self.idl_type(&union_ty.disc.ty, def.id);
        w!(w, "union ", def.ident.name, " switch (");
        self.emit_annotations(w, &union_ty.disc.annotations, def.id);
        w!(w, discriminator, ") {");

        for variant in &union_ty.variants {
            let ty_str = self.idl_type(&variant.ty, def.id);
            w.dedent();
            if variant.is_default {
                if !matches!(variant.ty.kind, TyKind::Null) {
                    w!(w, "\ndefault:");
                }
            } else {
                for label in &variant.labels {
                    let label_str = self.format_numeric(&label.value, def.id);
                    w!(w, "\ncase ", label_str, ":");
                }
            }
            w.indent();

            w!(w, "\n");
            self.emit_annotations(w, &variant.annotations, def.id);

            if !matches!(variant.ty.kind, TyKind::Null) {
                w!(w, ty_str, " ", variant.ident.name, ";");
            }
        }
        w!(w, "\n};\n");
    }

    fn emit_enum(&self, w: &mut Twine, def: &Def, enum_ty: &EnumTy) {
        w!(w, "enum ", def.ident.name, " {\n");

        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            let field_name = &field_def.ident.name;
            self.emit_annotations(w, &field_def.annotations, field_id);

            if field_def.flags.contains(DefFlags::IS_ENUMERATED) {
                if let DefKind::Const(const_ty) = &field_def.kind {
                    let value_str = self.format_numeric(&const_ty.value, def.id);
                    w!(w, "@value(", value_str, ") ", field_name);
                } else {
                    w!(w, field_name);
                }
            } else {
                w!(w, field_name);
            }

            if i < enum_ty.fields.len() - 1 {
                w!(w, ",\n");
            }
        }
        w!(w, "\n};\n");
    }

    fn emit_attribute(&self, w: &mut Twine, attr: &Attribute, relative_to_def_id: DefId) {
        if attr.is_readonly {
            w!(w, "readonly ");
        }
        w!(w, "attribute ");
        let ty_str = self.idl_type(&attr.ty, relative_to_def_id);
        w!(w, ty_str, " ", attr.ident.name);

        if !attr.getraises.is_empty() {
            w!(w, " getraises (");
            for (i, exc) in attr.getraises.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let exc_name = self.scoped_name(exc.def_id, relative_to_def_id);
                w!(w, exc_name);
            }
            w!(w, ")");
        }

        if !attr.setraises.is_empty() {
            w!(w, " setraises (");
            for (i, exc) in attr.setraises.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let exc_name = self.scoped_name(exc.def_id, relative_to_def_id);
                w!(w, exc_name);
            }
            w!(w, ")");
        }

        w!(w, ";");
    }

    fn emit_prototype(&self, w: &mut Twine, proto: &ProtoTy, relative_to_def_id: DefId) {
        let ret_ty = self.idl_type(&proto.ty, relative_to_def_id);
        w!(w, ret_ty, " ", proto.ident.name, "(");
        for (i, param) in proto.params.iter().enumerate() {
            if i > 0 {
                w!(w, ", ");
            }
            match param.kind {
                ParamKind::In => w!(w, "in "),
                ParamKind::Out => w!(w, "out "),
                ParamKind::InOut => w!(w, "inout "),
            }
            let param_ty = self.idl_type(&param.ty, relative_to_def_id);
            w!(w, param_ty, " ", param.ident.name);
        }
        w!(w, ")");

        if !proto.raises.is_empty() {
            w!(w, " raises (");
            for (i, exc) in proto.raises.iter().enumerate() {
                if i > 0 {
                    w!(w, ", ");
                }
                let exc_name = self.scoped_name(exc.def_id, relative_to_def_id);
                w!(w, exc_name);
            }
            w!(w, ")");
        }

        w!(w, ";");
    }

    fn emit_interface(&self, w: &mut Twine, def: &Def, interface: &InterfaceTy) {
        if interface.is_local {
            w!(w, "local ");
        }
        w!(w, "interface ", def.ident.name);
        self.emit_parents(w, &interface.parents, def.id);
        w!(w, " {");

        for &nested_id in &interface.definitions {
            w!(w, "\n");
            self.emit_definition(w, nested_id);
        }

        for (i, proto) in interface.prototypes.iter().enumerate() {
            w!(w, "\n");
            self.emit_prototype(w, proto, def.id);
            if i < interface.prototypes.len() - 1 {
                w!(w, "\n");
            }
        }

        for attr in &interface.attributes {
            w!(w, "\n");
            self.emit_attribute(w, attr, def.id);
        }

        w!(w, "\n};\n");
    }

    fn emit_valuetype(&self, w: &mut Twine, def: &Def, valuetype: &ValueTy) {
        w!(w, "valuetype ", def.ident.name);
        self.emit_parents(w, &valuetype.parent, def.id);

        if let Some(supports) = valuetype.supports {
            let supports_name = self.scoped_name(supports.def_id, def.id);
            w!(w, " supports ", supports_name);
        }

        w!(w, " {");

        for &nested_id in &valuetype.definitions {
            w!(w, "\n");
            self.emit_definition(w, nested_id);
        }

        for (i, proto) in valuetype.prototypes.iter().enumerate() {
            w!(w, "\n");
            self.emit_prototype(w, proto, def.id);
            if i < valuetype.prototypes.len() - 1 {
                w!(w, "\n");
            }
        }

        for member in &valuetype.members {
            w!(w, "\n");
            self.emit_annotations(w, &member.annotations, def.id);
            let ty_str = self.idl_type(&member.ty, def.id);
            let array_bounds = format_member_name(&member.ty);
            w!(w, "public ", ty_str, " ", member.ident.name, array_bounds, ";");
        }

        w!(w, "\n};\n");
    }

    fn emit_exception(&self, w: &mut Twine, def: &Def, except: &ExceptTy) {
        w!(w, "exception ", def.ident.name, " {");
        for member in &except.members {
            w!(w, "\n");
            self.emit_annotations(w, &member.annotations, def.id);
            let ty_str = self.idl_type(&member.ty, def.id);
            let array_bounds = format_member_name(&member.ty);
            w!(w, ty_str, " ", member.ident.name, array_bounds, ";");
        }
        w!(w, "\n};\n\n");
    }

    fn emit_alias(&self, w: &mut Twine, def: &Def, alias: &AliasTy) {
        let ty_str = self.idl_type(&alias.ty, def.id);
        let array_bounds = format_member_name(&alias.ty);
        w!(w, "typedef ", ty_str, " ", def.ident.name, array_bounds, ";\n");
    }

    fn emit_const(&self, w: &mut Twine, def: &Def, const_ty: &ConstTy) {
        let ty_str = self.idl_type(&const_ty.ty, def.id);
        let value_str = self.format_numeric(&const_ty.value, def.id);
        w!(w, "const ", ty_str, " ", def.ident.name, " = ", value_str, ";\n");
    }

    fn emit_bitmask(&self, w: &mut Twine, def: &Def, bitmask: &BitmaskTy) {
        w!(w, "bitmask ", def.ident.name, " {");
        for (i, &flag_id) in bitmask.flags.iter().enumerate() {
            let flag_def = self.hir.context.definitions.get(flag_id);
            let flag_name = &flag_def.ident.name;

            w!(w, "\n");
            if flag_def.flags.contains(DefFlags::IS_ENUMERATED)
                && let DefKind::Const(const_ty) = &flag_def.kind
            {
                let val = self.hir.context.unsigned_value(&const_ty.value);
                let position = val.trailing_zeros();
                w!(w, "@position(", position, ") ");
            }

            w!(w, flag_name);
            if i < bitmask.flags.len() - 1 {
                w!(w, ",");
            }
        }
        w!(w, "\n};\n");
    }

    fn emit_bitset(&self, w: &mut Twine, def: &Def, bitset: &BitsetTy) {
        w!(w, "bitset ", def.ident.name);
        self.emit_parents(w, &bitset.parent, def.id);
        w!(w, " {");
        for field in &bitset.fields {
            w!(w, "\n");
            self.emit_annotations(w, &field.annotations, def.id);
            let ty = self.idl_type(&field.ty, def.id);
            w!(w, "bitfield<", field.size, ", ", ty, "> ", field.ident.name, ";");
        }
        w!(w, "\n};\n");
    }

    fn emit_annotation(&self, w: &mut Twine, def: &Def, annotation: &AnnotationTy) {
        w!(w, "@annotation ", def.ident.name, " {");
        for param in &annotation.params {
            let ty_str = self.idl_type(&param.ty, def.id);
            w!(w, "\n", ty_str, " ", param.ident.name);
            if let Some(default_val) = &param.default {
                let val_str = self.format_numeric(default_val, def.id);
                w!(w, " default ", val_str);
            }
            w!(w, ";");
        }
        w!(w, "\n};\n\n");
    }

    fn emit_decl(w: &mut Twine, def: &Def, decl: Decl) {
        match decl {
            Decl::Struct => w!(w, "struct ", def.ident.name, ";\n"),
            Decl::Union => w!(w, "union ", def.ident.name, ";\n"),
            Decl::Interface => w!(w, "interface ", def.ident.name, ";\n"),
            Decl::Valuetype => w!(w, "valuetype ", def.ident.name, ";\n"),
            Decl::Native => w!(w, "native ", def.ident.name, ";\n"),
        }
    }

    fn emit_definition(&self, w: &mut Twine, def_id: DefId) {
        let def = self.hir.context.definitions.get(def_id);
        self.emit_annotations(w, &def.annotations, def.id);

        match &def.kind {
            DefKind::Module(module) => self.emit_module(w, def, module),
            DefKind::Struct(struct_ty) => self.emit_struct(w, def, struct_ty),
            DefKind::Union(union_ty) => self.emit_union(w, def, union_ty),
            DefKind::Enum(enum_ty) => self.emit_enum(w, def, enum_ty),
            DefKind::Interface(interface) => self.emit_interface(w, def, interface),
            DefKind::Valuetype(valuetype) => self.emit_valuetype(w, def, valuetype),
            DefKind::Except(except) => self.emit_exception(w, def, except),
            DefKind::Alias(alias) => self.emit_alias(w, def, alias),
            DefKind::Const(const_ty) => self.emit_const(w, def, const_ty),
            DefKind::Bitmask(bitmask) => self.emit_bitmask(w, def, bitmask),
            DefKind::Bitset(bitset) => self.emit_bitset(w, def, bitset),
            DefKind::Annotation(annotation) => self.emit_annotation(w, def, annotation),
            DefKind::Decl(decl) => Self::emit_decl(w, def, *decl),
        }
    }

    pub fn generate_type(&self, def_id: DefId) -> String {
        let mut w = Twine::new();
        self.emit_definition(&mut w, def_id);
        w.finish()
    }

    pub fn generate(&self) -> Vec<File> {
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
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap();

            let mut w = Twine::new();
            if self.options.idl_legacy {
                let guard = file_name.to_uppercase().replace(['.', '-'], "_");
                w!(w, "#ifndef ", guard, "\n");
                w!(w, "#define ", guard, "\n");
            } else {
                w!(w, "#pragma once\n");
            }

            let mut dependencies = std::collections::HashSet::new();
            for &def_id in &def_ids {
                collect_def_dependencies(self.hir, def_id, file_id, &mut dependencies);
            }

            let mut deps_vec: Vec<FileId> = dependencies.into_iter().collect();
            deps_vec.sort();

            for &dep_file_id in &deps_vec {
                let dep_file = self
                    .source_map
                    .name(dep_file_id)
                    .with_extension("idl")
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .to_string();

                w!(w, "\n#include \"", dep_file, "\"");
            }

            if !deps_vec.is_empty() {
                w!(w, "\n");
            }

            w!(w, "\n");

            for def_id in def_ids {
                self.emit_definition(&mut w, def_id);
            }

            if self.options.idl_legacy {
                w!(w, "\n#endif\n");
            }

            let content = w.finish();
            result.push(File::Generated {
                path: PathBuf::from(file_name),
                source: content,
            });
        }
        result
    }
}

fn inline_annotation_width(name: &str, args: &[String]) -> usize {
    let separators = args.len().saturating_sub(1) * ", ".len();
    let payload: usize = args.iter().map(String::len).sum();
    "@".len() + name.len() + "(".len() + payload + separators + ")".len()
}

fn filter_non_default_args<'a>(ann: &'a Ann, hir: &ResolvedGraph) -> Vec<&'a AnnArg> {
    ann.args
        .iter()
        .filter(|arg| {
            if let Some(def_id) = ann.def_id
                && let ann_def = hir.context.definitions.get(def_id)
                && let DefKind::Annotation(ann_ty) = &ann_def.kind
            {
                for param in &ann_ty.params {
                    if param.ident.name == arg.ident.name
                        && let Some(default_val) = &param.default
                    {
                        return &arg.value != default_val;
                    }
                }
            }
            true
        })
        .collect()
}

fn format_member_name(ty: &Ty) -> String {
    let mut result = String::new();
    let mut current_ty = ty;

    while let TyKind::Array {
        ty: inner_ty, len, ..
    } = &current_ty.kind
    {
        result.push('[');
        result.push_str(&len.to_string());
        result.push(']');
        current_ty = inner_ty;
    }
    result
}
