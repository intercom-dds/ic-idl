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
use std::path::PathBuf;

use ic_emit::File;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use ic_vfs::{FileId, SourceMap};
use intercom_cts::json::{self, Value, value};

use crate::JsonSchemaOptions;

pub struct JsonSchemaGen<'a> {
    hir: &'a ResolvedGraph,
    source_map: &'a SourceMap,
    options: JsonSchemaOptions,
}

impl<'a> JsonSchemaGen<'a> {
    pub fn new(
        hir: &'a ResolvedGraph,
        source_map: &'a SourceMap,
        options: JsonSchemaOptions,
    ) -> Self {
        Self {
            hir,
            source_map,
            options,
        }
    }

    pub fn generate(&self) -> Vec<File> {
        let mut files = Vec::new();
        let mut file_map: BTreeMap<FileId, Vec<DefId>> = BTreeMap::new();

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);
            if !matches!(
                def.kind,
                DefKind::Struct(_)
                    | DefKind::Union(_)
                    | DefKind::Enum(_)
                    | DefKind::Alias(_)
                    | DefKind::Bitmask(_)
            ) {
                continue;
            }

            let file_id = self.file_id(def_id);
            file_map.entry(file_id).or_default().push(def_id);
        }

        for (file_id, def_ids) in file_map {
            if let Some(file) = self.generate_file(file_id, &def_ids) {
                files.push(file);
            }
        }

        files
    }

    fn file_id(&self, def_id: DefId) -> FileId {
        let def = self.hir.context.definitions.get(def_id);
        def.span.start.file_id
    }

    fn make_path(&self, def_id: DefId) -> String {
        let mut path = Vec::new();
        let mut current = Some(def_id);

        while let Some(id) = current {
            let def = self.hir.context.definitions.get(id);
            if matches!(def.kind, DefKind::Module(_)) {
                path.push(def.ident.name.clone());
            }
            current = def.parent;
        }

        path.reverse();
        path.join("/")
    }

    fn qualified_name(&self, def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(def_id);
        let path = self.make_path(def_id);
        if path.is_empty() {
            def.ident.name.clone()
        } else {
            format!("{}.{}", path.replace('/', "."), def.ident.name)
        }
    }

    fn join_uri(base: &str, path: &str) -> String {
        let mut base = base.to_string();
        if !base.is_empty() && !base.ends_with('/') {
            base.push('/');
        }
        // Normalize path separators and strip leading slash
        let path = path.replace('\\', "/");
        let path_clean = path.trim_start_matches('/');
        format!("{base}{path_clean}")
    }

    fn output_path(&self, file_id: FileId) -> PathBuf {
        self.source_map.included_as(file_id).with_extension("json")
    }

    fn schema_uri(&self, file_id: FileId) -> String {
        let filename = self.output_path(file_id);
        let filename_str = filename.to_string_lossy();
        let default_uri = "file:///".to_string();
        let base = self
            .options
            .schema_base_uri
            .as_ref()
            .unwrap_or(&default_uri);
        Self::join_uri(base, &filename_str)
    }

    fn make_reference(&self, def_id: DefId, current_file_id: FileId) -> String {
        let target_file_id = self.file_id(def_id);
        let key = self.qualified_name(def_id);

        if target_file_id == current_file_id {
            format!("#/$defs/{key}")
        } else {
            let full_uri = self.schema_uri(target_file_id);
            format!("{full_uri}#/$defs/{key}")
        }
    }

    fn json_type(&self, ty: &Ty) -> &str {
        match &ty.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool => "boolean",
                PrimitiveTy::Char | PrimitiveTy::WChar => "string",
                PrimitiveTy::Int8
                | PrimitiveTy::UInt8
                | PrimitiveTy::Int16
                | PrimitiveTy::UInt16
                | PrimitiveTy::Int32
                | PrimitiveTy::UInt32
                | PrimitiveTy::Int64
                | PrimitiveTy::UInt64 => "integer",
                PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128 => "number",
                PrimitiveTy::Void => "null",
            },
            TyKind::String { .. } => "string",
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                match &def.kind {
                    DefKind::Enum(_) => "string",
                    DefKind::Alias(alias) => self.json_type(&alias.ty),
                    _ => "object",
                }
            }
            _ => "object",
        }
    }

    fn format_numeric(&self, value: &Numeric) -> Value {
        match value {
            Numeric::Bool(b) => Value::Bool(*b),
            Numeric::Char(c) => Value::String(c.to_string()),
            Numeric::Int8(v) => Value::Number((*v).into()),
            Numeric::UInt8(v) => Value::Number((*v).into()),
            Numeric::Int16(v) => Value::Number((*v).into()),
            Numeric::UInt16(v) => Value::Number((*v).into()),
            Numeric::Int32(v) => Value::Number((*v).into()),
            Numeric::UInt32(v) => Value::Number((*v).into()),
            Numeric::Int64(v) => Value::Number((*v).into()),
            Numeric::UInt64(v) => Value::Number((*v).into()),
            Numeric::Float(v) => Value::Number((*v).into()),
            Numeric::Double(v) => Value::Number((*v).into()),
            Numeric::String(s) => Value::String(s.clone()),
            Numeric::Const(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                if let DefKind::Const(const_ty) = &def.kind {
                    self.format_numeric(&const_ty.value)
                } else {
                    Value::String(def.ident.name.clone())
                }
            }
            Numeric::Null
            | Numeric::Array { .. }
            | Numeric::Sequence { .. }
            | Numeric::Map { .. }
            | Numeric::Struct { .. }
            | Numeric::Union { .. } => Value::Null,
        }
    }

    fn doc_comments(def: &Def) -> Option<String> {
        let docs: Vec<String> = def
            .annotations
            .iter()
            .filter(|ann| ann.ident.name == "doc" || ann.ident.name == "documentation")
            .filter_map(|ann| {
                ann.args.first().and_then(|arg| {
                    if let Numeric::String(s) = &arg.value {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    fn apply_bounds(&self, annotations: &[ic_hir::hir::Ann], obj: &mut BTreeMap<String, Value>) {
        for ann in annotations {
            match ann.ident.name.as_str() {
                "min" => {
                    if let Some(arg) = ann.args.first() {
                        obj.insert("minimum".to_string(), self.format_numeric(&arg.value));
                    }
                }
                "max" => {
                    if let Some(arg) = ann.args.first() {
                        obj.insert("maximum".to_string(), self.format_numeric(&arg.value));
                    }
                }
                "range" => {
                    for arg in &ann.args {
                        if arg.ident.name == "min" {
                            obj.insert("minimum".to_string(), self.format_numeric(&arg.value));
                        } else if arg.ident.name == "max" {
                            obj.insert("maximum".to_string(), self.format_numeric(&arg.value));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn generate_preamble(def: &Def) -> BTreeMap<String, Value> {
        let mut obj = BTreeMap::new();
        obj.insert("title".to_string(), Value::String(def.ident.name.clone()));

        if let Some(desc) = Self::doc_comments(def) {
            obj.insert("description".to_string(), Value::String(desc));
        }
        obj
    }

    fn generate_type_schema(&self, ty: &Ty, current_file_id: FileId) -> Value {
        match &ty.kind {
            TyKind::String { bound, .. } => {
                let mut obj = value!({ "type": "string" });
                if let Some(b) = bound {
                    if let Value::Object(ref mut map) = obj {
                        map.insert("maxLength".to_string(), Value::Number((*b).into()));
                    }
                }
                obj
            }
            TyKind::Array {
                ty: elem_ty, len, ..
            } => {
                let items_schema = self.generate_type_schema(elem_ty, current_file_id);
                let mut map = BTreeMap::new();
                map.insert("type".to_string(), Value::String("array".to_string()));
                map.insert("items".to_string(), items_schema);
                map.insert("minItems".to_string(), Value::Number((*len).into()));
                map.insert("maxItems".to_string(), Value::Number((*len).into()));
                Value::Object(map)
            }
            TyKind::Sequence {
                ty: elem_ty, bound, ..
            } => {
                let items_schema = self.generate_type_schema(elem_ty, current_file_id);
                let mut obj = value!({
                    "type": "array",
                    "items": items_schema
                });
                if let Some(b) = bound {
                    if let Value::Object(ref mut map) = obj {
                        map.insert("maxItems".to_string(), Value::Number((*b).into()));
                    }
                }
                obj
            }
            TyKind::Map {
                elem: elem_ty,
                bound,
                ..
            } => {
                let additional_properties = self.generate_type_schema(elem_ty, current_file_id);
                let mut map = BTreeMap::new();
                map.insert("type".to_string(), Value::String("object".to_string()));
                map.insert("additionalProperties".to_string(), additional_properties);

                if let Some(b) = bound {
                    map.insert("maxProperties".to_string(), Value::Number((*b).into()));
                }
                Value::Object(map)
            }
            TyKind::Adt(def_id) => {
                let ref_url = self.make_reference(*def_id, current_file_id);
                value!({ "$ref": ref_url })
            }
            TyKind::Any => Value::Object(BTreeMap::new()),
            _ => {
                let ty_name = self.json_type(ty);
                value!({ "type": ty_name })
            }
        }
    }

    fn generate_struct(
        &self,
        def: &Def,
        struct_ty: &ic_hir::hir::StructTy,
        current_file_id: FileId,
    ) -> Value {
        let mut obj = Self::generate_preamble(def);
        obj.insert("type".to_string(), Value::String("object".to_string()));

        if let Some(parent_id) = struct_ty.parent {
            let ref_url = self.make_reference(parent_id, current_file_id);
            obj.insert("allOf".to_string(), value!([{ "$ref": ref_url }]));
        }

        let mut properties = BTreeMap::new();
        let mut required = Vec::new();

        let is_final_struct = def.annotations.iter().any(|a| a.ident.name == "final");

        for member in &struct_ty.members {
            let mut member_obj = self.generate_type_schema(&member.ty, current_file_id);

            let is_optional = member
                .annotations
                .iter()
                .any(|a| a.ident.name == "optional");
            if is_final_struct && !is_optional {
                required.push(Value::String(member.ident.name.clone()));
            }

            if let Value::Object(ref mut map) = member_obj {
                self.apply_bounds(&member.annotations, map);
            }

            properties.insert(member.ident.name.clone(), member_obj);
        }

        obj.insert("properties".to_string(), Value::Object(properties));
        if !required.is_empty() {
            obj.insert("required".to_string(), Value::Array(required));
        }

        if is_final_struct {
            obj.insert("additionalProperties".to_string(), Value::Bool(false));
        }

        Value::Object(obj)
    }

    fn generate_union(
        &self,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        current_file_id: FileId,
    ) -> Value {
        let mut obj = Self::generate_preamble(def);
        obj.insert("type".to_string(), Value::String("object".to_string()));

        let mut explicit_discriminators = Vec::new();
        for variant in &union_ty.variants {
            for label in &variant.labels {
                explicit_discriminators.push(self.format_numeric(&label.value));
            }
        }

        let one_of: Vec<Value> = union_ty
            .variants
            .iter()
            .map(|variant| {
                let discriminator = if let Some(label) = variant.labels.first() {
                    let const_val = self.format_numeric(&label.value);
                    let ty_name = self.json_type(&union_ty.disc.ty);
                    value!({
                        "const": const_val,
                        "type": ty_name
                    })
                } else {
                    let ty_name = self.json_type(&union_ty.disc.ty);
                    value!({
                        "type": ty_name,
                        "not": { "enum": explicit_discriminators }
                    })
                };

                let mut variant_schema = BTreeMap::new();

                if matches!(variant.ty.kind, TyKind::Null) {
                    variant_schema.insert(
                        "properties".to_string(),
                        value!({ "$discriminator": discriminator }),
                    );
                    variant_schema.insert("required".to_string(), value!(["$discriminator"]));
                } else {
                    let mut value_obj = self.generate_type_schema(&variant.ty, current_file_id);

                    if let Value::Object(ref mut map) = value_obj {
                        self.apply_bounds(&variant.annotations, map);
                    }

                    let variant_name = &variant.ident.name;
                    variant_schema.insert(
                        "properties".to_string(),
                        value!({
                            "$discriminator": discriminator,
                            variant_name: value_obj
                        }),
                    );
                    variant_schema.insert(
                        "required".to_string(),
                        value!(["$discriminator", variant_name]),
                    );
                }

                Value::Object(variant_schema)
            })
            .collect();

        obj.insert("oneOf".to_string(), Value::Array(one_of));

        Value::Object(obj)
    }

    fn generate_enum(&self, def: &Def, enum_ty: &ic_hir::hir::EnumTy) -> Value {
        let mut obj = Self::generate_preamble(def);

        let variants: Vec<Value> = enum_ty
            .fields
            .iter()
            .map(|&field_id| {
                let field_def = self.hir.context.definitions.get(field_id);
                Value::String(field_def.ident.name.clone())
            })
            .collect();

        obj.insert("enum".to_string(), Value::Array(variants));

        Value::Object(obj)
    }

    fn generate_bitmask(def: &Def) -> Value {
        let mut obj = Self::generate_preamble(def);
        obj.insert(
            "oneOf".to_string(),
            value!([
                { "type": "integer", "minimum": 0 },
                { "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*(\\|[A-Za-z_][A-Za-z0-9_]*)*$" }
            ]),
        );
        Value::Object(obj)
    }

    fn generate_typedef(
        &self,
        def: &Def,
        typedef: &ic_hir::hir::AliasTy,
        current_file_id: FileId,
    ) -> Value {
        let mut obj = Self::generate_preamble(def);

        let type_schema = self.generate_type_schema(&typedef.ty, current_file_id);
        if let Value::Object(map) = type_schema {
            obj.extend(map);
        }

        self.apply_bounds(&def.annotations, &mut obj);

        Value::Object(obj)
    }

    fn generate_file(&self, file_id: FileId, def_ids: &[DefId]) -> Option<File> {
        let mut defs_map = BTreeMap::new();

        for &def_id in def_ids {
            let def = self.hir.context.definitions.get(def_id);
            let schema = match &def.kind {
                DefKind::Struct(s) => self.generate_struct(def, s, file_id),
                DefKind::Union(u) => self.generate_union(def, u, file_id),
                DefKind::Enum(e) => self.generate_enum(def, e),
                DefKind::Bitmask(_) => Self::generate_bitmask(def),
                DefKind::Alias(t) => self.generate_typedef(def, t, file_id),
                _ => continue,
            };
            let key = self.qualified_name(def_id);
            defs_map.insert(key, schema);
        }

        if defs_map.is_empty() {
            return None;
        }

        let mut root = BTreeMap::new();
        root.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2019-09/schema#".to_string()),
        );

        // Calculate $id for the file
        let full_id = self.schema_uri(file_id);
        root.insert("$id".to_string(), Value::String(full_id));
        root.insert("$defs".to_string(), Value::Object(defs_map));
        let source = json::to_string(&root, true).ok()?;

        Some(File::Generated {
            path: self.output_path(file_id),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_uri_basic() {
        assert_eq!(
            JsonSchemaGen::join_uri("https://example.com", "schema.json"),
            "https://example.com/schema.json"
        );
    }

    #[test]
    fn join_uri_trailing_slash() {
        assert_eq!(
            JsonSchemaGen::join_uri("https://example.com/", "schema.json"),
            "https://example.com/schema.json"
        );
    }

    #[test]
    fn join_uri_leading_slash_in_path() {
        assert_eq!(
            JsonSchemaGen::join_uri("https://example.com", "/schema.json"),
            "https://example.com/schema.json"
        );
    }

    #[test]
    fn join_uri_both_slashes() {
        assert_eq!(
            JsonSchemaGen::join_uri("https://example.com/", "/schema.json"),
            "https://example.com/schema.json"
        );
    }

    #[test]
    fn join_uri_empty_base() {
        assert_eq!(JsonSchemaGen::join_uri("", "schema.json"), "schema.json");
    }

    #[test]
    fn join_uri_empty_base_leading_slash() {
        assert_eq!(JsonSchemaGen::join_uri("", "/schema.json"), "schema.json");
    }

    #[test]
    fn join_uri_backslashes() {
        assert_eq!(
            JsonSchemaGen::join_uri("file:///", "path\\to\\schema.json"),
            "file:///path/to/schema.json"
        );
    }

    #[test]
    fn join_uri_file_scheme() {
        assert_eq!(
            JsonSchemaGen::join_uri("file:///", "schema.json"),
            "file:///schema.json"
        );
    }
}
