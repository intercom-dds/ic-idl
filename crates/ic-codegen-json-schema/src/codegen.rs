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

use ic_emit::File;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};
use ic_vfs::{FileId, SourceMap};
use intercom_cts::json::{self, Value, value};

use crate::JsonSchemaOptions;

pub struct JsonSchemaGen<'a> {
    hir: &'a ResolvedGraph,
    _source_map: &'a SourceMap,
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
            _source_map: source_map,
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
                DefKind::Struct(_) | DefKind::Union(_) | DefKind::Enum(_) | DefKind::Alias(_)
            ) {
                continue;
            }

            let file_id = self.get_file_id(def_id);
            file_map.entry(file_id).or_default().push(def_id);
        }

        for (file_id, def_ids) in file_map {
            for def_id in def_ids {
                if let Some(file) = self.generate_file(file_id, def_id) {
                    files.push(file);
                }
            }
        }

        files
    }

    fn get_file_id(&self, def_id: DefId) -> FileId {
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

    fn make_reference(&self, def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(def_id);
        let path = self.make_path(def_id);
        let name = &def.ident.name;

        let default_uri = "file:///".to_string();
        let mut base = self
            .options
            .schema_base_uri
            .as_ref()
            .unwrap_or(&default_uri)
            .clone();

        if !base.is_empty() && !base.ends_with('/') {
            base.push('/');
        }

        if path.is_empty() {
            format!("{base}{name}.json")
        } else {
            format!("{base}{path}/{name}.json")
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
                if matches!(def.kind, DefKind::Enum(_)) {
                    "string"
                } else {
                    "object"
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
                Value::String(def.ident.name.clone())
            }
            _ => Value::Null,
        }
    }

    fn get_doc_comments(def: &Def) -> Option<String> {
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

    fn generate_preamble(&self, def: &Def) -> BTreeMap<String, Value> {
        let mut obj = BTreeMap::new();

        obj.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2019-09/schema#".to_string()),
        );

        let reference = self.make_reference(def.id);
        obj.insert("$id".to_string(), Value::String(reference));
        obj.insert("title".to_string(), Value::String(def.ident.name.clone()));

        if let Some(desc) = Self::get_doc_comments(def) {
            obj.insert("description".to_string(), Value::String(desc));
        }

        obj
    }

    fn generate_struct(&self, def: &Def, struct_ty: &ic_hir::hir::StructTy) -> Value {
        let mut obj = self.generate_preamble(def);

        if let Some(parent_id) = struct_ty.parent {
            let ref_url = self.make_reference(parent_id);
            obj.insert("allOf".to_string(), value!([{ "$ref": ref_url }]));
        }

        let mut properties = BTreeMap::new();
        let mut required = Vec::new();

        let is_final_struct = def.annotations.iter().any(|a| a.ident.name == "final");

        for member in &struct_ty.members {
            let mut member_obj = match &member.ty.kind {
                TyKind::Primitive(_) | TyKind::String { .. } => {
                    let ty_name = self.json_type(&member.ty);
                    value!({ "type": ty_name })
                }
                TyKind::Sequence { ty, .. } => {
                    if let TyKind::Adt(def_id) = &ty.kind {
                        let ref_url = self.make_reference(*def_id);
                        value!({
                            "type": "array",
                            "items": { "$ref": ref_url }
                        })
                    } else {
                        let ty_name = self.json_type(ty);
                        value!({
                            "type": "array",
                            "items": { "type": ty_name }
                        })
                    }
                }
                TyKind::Map { elem, .. } => {
                    if let TyKind::Adt(def_id) = &elem.kind {
                        let ref_url = self.make_reference(*def_id);
                        value!({
                            "type": "object",
                            "additionalProperties": { "$ref": ref_url }
                        })
                    } else {
                        let ty_name = self.json_type(elem);
                        value!({
                            "type": "object",
                            "additionalProperties": {
                                "type": ty_name,
                            }
                        })
                    }
                }
                TyKind::Adt(def_id) => {
                    let ref_url = self.make_reference(*def_id);
                    value!({ "$ref": ref_url })
                }
                _ => {
                    value!({ "type": "object" })
                }
            };

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
        obj.insert("required".to_string(), Value::Array(required));

        Value::Object(obj)
    }

    fn generate_union(&self, def: &Def, union_ty: &ic_hir::hir::UnionTy) -> Value {
        let mut obj = self.generate_preamble(def);
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
            .filter_map(|variant| {
                if matches!(variant.ty.kind, TyKind::Null) {
                    return None;
                }

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

                let mut value_obj = match &variant.ty.kind {
                    TyKind::Primitive(_) | TyKind::String { .. } => {
                        let ty_name = self.json_type(&variant.ty);
                        value!({ "type": ty_name })
                    }
                    TyKind::Adt(def_id) => {
                        let ref_url = self.make_reference(*def_id);
                        value!({ "$ref": ref_url })
                    }
                    _ => {
                        value!({ "type": "object" })
                    }
                };

                if let Value::Object(ref mut map) = value_obj {
                    self.apply_bounds(&variant.annotations, map);
                }

                let variant_name = &variant.ident.name;
                let mut variant_schema = BTreeMap::new();
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
                Some(Value::Object(variant_schema))
            })
            .collect();

        obj.insert("oneOf".to_string(), Value::Array(one_of));

        Value::Object(obj)
    }

    fn generate_enum(&self, def: &Def, enum_ty: &ic_hir::hir::EnumTy) -> Value {
        let mut obj = self.generate_preamble(def);

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

    fn generate_typedef(&self, def: &Def, typedef: &ic_hir::hir::AliasTy) -> Value {
        let mut obj = self.generate_preamble(def);
        obj.insert(
            "type".to_string(),
            Value::String(self.json_type(&typedef.ty).to_string()),
        );

        self.apply_bounds(&def.annotations, &mut obj);

        Value::Object(obj)
    }

    fn generate_file(&self, _file_id: FileId, def_id: DefId) -> Option<File> {
        let def = self.hir.context.definitions.get(def_id);
        let value = match &def.kind {
            DefKind::Struct(s) => self.generate_struct(def, s),
            DefKind::Union(u) => self.generate_union(def, u),
            DefKind::Enum(e) => self.generate_enum(def, e),
            DefKind::Alias(t) => self.generate_typedef(def, t),
            _ => return None,
        };

        let source = json::to_string(&value, true).ok()?;
        let path = self.make_path(def_id);
        let file_name = if path.is_empty() {
            format!("{}.json", def.ident.name)
        } else {
            format!("{}/{}.json", path, def.ident.name)
        };

        Some(File::Generated {
            path: file_name.into(),
            source,
        })
    }
}
