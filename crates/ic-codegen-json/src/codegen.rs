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
use ic_hir::hir::{
    AliasTy, Ann, BitmaskTy, ConstTy, Def, DefFlags, DefId, DefKind, EnumTy, ExceptTy, InterfaceTy,
    Member, ModuleTy, Numeric, PrimitiveTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use ic_vfs::{FileId, SourceMap};
use intercom_cts::json::{self, Value};

pub struct JsonGen<'a> {
    hir: &'a ResolvedGraph,
}

impl<'a> JsonGen<'a> {
    pub fn new(hir: &'a ResolvedGraph) -> Self {
        Self { hir }
    }

    pub fn generate(&self, source_map: &SourceMap) -> Vec<File> {
        let mut files = Vec::new();
        let mut file_map: BTreeMap<FileId, Vec<DefId>> = BTreeMap::new();

        for &def_id in &self.hir.order {
            let def = self.hir.context.definitions.get(def_id);
            let file_id = def.span.start.file_id;
            file_map.entry(file_id).or_default().push(def_id);
        }

        for (file_id, def_ids) in file_map {
            if let Some(file) = self.generate_file(source_map, file_id, &def_ids) {
                files.push(file);
            }
        }

        files
    }

    pub fn generate_def(&self, root: DefId) -> String {
        let mut definitions = BTreeMap::new();

        for &def_id in &self.hir.order {
            self.emit_def(def_id, &mut definitions);
        }

        let mut top = BTreeMap::new();
        top.insert(
            "root_type".to_string(),
            Value::String(self.hir.context.qualified_name(root)),
        );
        top.insert("definitions".to_string(), Value::Object(definitions));

        json::to_string(&Value::Object(top), false)
            .expect("serializing an in-memory json value cannot fail")
    }

    fn generate_file(
        &self,
        source_map: &SourceMap,
        file_id: FileId,
        def_ids: &[DefId],
    ) -> Option<File> {
        let mut obj = BTreeMap::new();

        for &def_id in def_ids {
            self.emit_def(def_id, &mut obj);
        }

        if obj.is_empty() {
            return None;
        }

        let source = json::to_string(&Value::Object(obj), true).ok()?;
        let file_path = source_map.name(file_id);
        let file_name = file_path.file_name()?.to_str()?.replace(".idl", ".json");

        Some(File::Generated {
            path: file_name.into(),
            source,
        })
    }

    fn emit_def(&self, def_id: DefId, obj: &mut BTreeMap<String, Value>) {
        let def = self.hir.context.definitions.get(def_id);

        match &def.kind {
            DefKind::Module(m) => self.emit_module(def, m, obj),
            DefKind::Struct(s) => self.emit_struct(def, s, obj),
            DefKind::Union(u) => self.emit_union(def, u, obj),
            DefKind::Enum(e) => self.emit_enum(def, e, obj),
            DefKind::Bitmask(b) => self.emit_bitmask(def, b, obj),
            DefKind::Alias(a) => self.emit_alias(def, a, obj),
            DefKind::Const(c) => self.emit_const(def, c, obj),
            DefKind::Interface(i) => self.emit_interface(def, i, obj),
            DefKind::Valuetype(v) => self.emit_valuetype(def, v, obj),
            DefKind::Except(e) => self.emit_except(def, e, obj),
            DefKind::Annotation(a) => self.emit_annotation_def(def, a, obj),
            DefKind::Decl(d) => Self::emit_forward_decl(def, *d, obj),
            DefKind::Bitset(_) => {}
        }
    }

    fn emit_module(&self, def: &Def, module: &ModuleTy, obj: &mut BTreeMap<String, Value>) {
        let mut module_obj = BTreeMap::new();
        module_obj.insert("kind".to_string(), Value::String("module".to_string()));
        self.emit_annotations(&def.annotations, &mut module_obj);

        for &child_id in &module.definitions {
            self.emit_def(child_id, &mut module_obj);
        }

        obj.insert(def.ident.name.clone(), Value::Object(module_obj));
    }

    fn emit_struct(&self, def: &Def, struct_ty: &StructTy, obj: &mut BTreeMap<String, Value>) {
        let mut struct_obj = BTreeMap::new();
        struct_obj.insert("kind".to_string(), Value::String("struct".to_string()));
        self.emit_annotations(&def.annotations, &mut struct_obj);

        if let Some(parent) = struct_ty.parent {
            let mut base_obj = BTreeMap::new();
            Self::emit_type_info(
                self.hir.context.definitions.get(parent.def_id),
                &mut base_obj,
            );
            struct_obj.insert("base_type".to_string(), Value::Object(base_obj));
        }

        let members: Vec<Value> = struct_ty
            .members
            .iter()
            .map(|m| self.emit_member(m))
            .collect();
        struct_obj.insert("members".to_string(), Value::Array(members));

        obj.insert(def.ident.name.clone(), Value::Object(struct_obj));
    }

    fn emit_union(&self, def: &Def, union_ty: &UnionTy, obj: &mut BTreeMap<String, Value>) {
        let mut union_obj = BTreeMap::new();
        union_obj.insert("kind".to_string(), Value::String("union".to_string()));
        self.emit_annotations(&def.annotations, &mut union_obj);

        let mut disc_obj = BTreeMap::new();
        self.emit_annotations(&union_ty.disc.annotations, &mut disc_obj);
        self.emit_type_ref(&union_ty.disc.ty, &mut disc_obj);
        union_obj.insert("discriminator".to_string(), Value::Object(disc_obj));

        let cases: Vec<Value> = union_ty
            .variants
            .iter()
            .map(|v| self.emit_union_case(v))
            .collect();
        union_obj.insert("cases".to_string(), Value::Array(cases));

        obj.insert(def.ident.name.clone(), Value::Object(union_obj));
    }

    fn emit_union_case(&self, variant: &Variant) -> Value {
        let mut case_obj = BTreeMap::new();

        if variant.labels.is_empty() && variant.is_default {
            case_obj.insert("case".to_string(), Value::String("default".to_string()));
        } else if variant.labels.len() == 1 {
            case_obj.insert(
                "case".to_string(),
                self.format_numeric(&variant.labels[0].value),
            );
        } else {
            let labels: Vec<Value> = variant
                .labels
                .iter()
                .map(|l| self.format_numeric(&l.value))
                .collect();
            case_obj.insert("case".to_string(), Value::Array(labels));
        }

        case_obj.insert(
            "name".to_string(),
            Value::String(variant.ident.name.clone()),
        );
        self.emit_annotations(&variant.annotations, &mut case_obj);
        self.emit_type_ref(&variant.ty, &mut case_obj);

        Value::Object(case_obj)
    }

    fn emit_enum(&self, def: &Def, enum_ty: &EnumTy, obj: &mut BTreeMap<String, Value>) {
        let mut enum_obj = BTreeMap::new();
        enum_obj.insert("kind".to_string(), Value::String("enum".to_string()));
        self.emit_annotations(&def.annotations, &mut enum_obj);

        let enumerators: Vec<Value> = enum_ty
            .fields
            .iter()
            .map(|&field_id| {
                let field_def = self.hir.context.definitions.get(field_id);
                let mut enumerator = BTreeMap::new();
                enumerator.insert(
                    "name".to_string(),
                    Value::String(field_def.ident.name.clone()),
                );

                if field_def.flags.contains(DefFlags::IS_ENUMERATED)
                    && let DefKind::Const(c) = &field_def.kind
                {
                    enumerator.insert("value".to_string(), self.format_numeric(&c.value));
                }

                Value::Object(enumerator)
            })
            .collect();

        enum_obj.insert("enumerators".to_string(), Value::Array(enumerators));
        obj.insert(def.ident.name.clone(), Value::Object(enum_obj));
    }

    fn emit_bitmask(&self, def: &Def, bitmask: &BitmaskTy, obj: &mut BTreeMap<String, Value>) {
        let mut bitmask_obj = BTreeMap::new();
        bitmask_obj.insert("kind".to_string(), Value::String("bitmask".to_string()));
        self.emit_annotations(&def.annotations, &mut bitmask_obj);

        let flags: Vec<Value> = bitmask
            .flags
            .iter()
            .enumerate()
            .map(|(pos, &flag_id)| {
                let flag_def = self.hir.context.definitions.get(flag_id);
                let mut flag_obj = BTreeMap::new();
                flag_obj.insert(
                    "name".to_string(),
                    Value::String(flag_def.ident.name.clone()),
                );

                if let DefKind::Const(c) = &flag_def.kind {
                    flag_obj.insert("position".to_string(), self.format_numeric(&c.value));
                } else {
                    flag_obj.insert("position".to_string(), Value::Number(pos.into()));
                }

                Value::Object(flag_obj)
            })
            .collect();

        bitmask_obj.insert("flags".to_string(), Value::Array(flags));
        obj.insert(def.ident.name.clone(), Value::Object(bitmask_obj));
    }

    fn emit_alias(&self, def: &Def, alias: &AliasTy, obj: &mut BTreeMap<String, Value>) {
        let mut alias_obj = BTreeMap::new();
        self.emit_annotations(&def.annotations, &mut alias_obj);
        alias_obj.insert("kind".to_string(), Value::String("typedef".to_string()));

        let mut type_obj = BTreeMap::new();
        self.emit_type_ref(&alias.ty, &mut type_obj);
        alias_obj.insert("type".to_string(), Value::Object(type_obj));

        obj.insert(def.ident.name.clone(), Value::Object(alias_obj));
    }

    fn emit_const(&self, def: &Def, const_ty: &ConstTy, obj: &mut BTreeMap<String, Value>) {
        let mut const_obj = BTreeMap::new();
        self.emit_annotations(&def.annotations, &mut const_obj);
        const_obj.insert("kind".to_string(), Value::String("const".to_string()));

        let mut type_obj = BTreeMap::new();
        self.emit_type_ref(&const_ty.ty, &mut type_obj);
        const_obj.insert("type".to_string(), Value::Object(type_obj));

        const_obj.insert("value".to_string(), self.format_numeric(&const_ty.value));

        obj.insert(def.ident.name.clone(), Value::Object(const_obj));
    }

    fn emit_interface(
        &self,
        def: &Def,
        interface: &InterfaceTy,
        obj: &mut BTreeMap<String, Value>,
    ) {
        let mut interface_obj = BTreeMap::new();
        interface_obj.insert("kind".to_string(), Value::String("interface".to_string()));
        self.emit_annotations(&def.annotations, &mut interface_obj);

        if !interface.parents.is_empty() {
            if interface.parents.len() > 1 {
                let parents: Vec<Value> = interface
                    .parents
                    .iter()
                    .map(|parent| {
                        let mut parent_obj = BTreeMap::new();
                        Self::emit_type_info(
                            self.hir.context.definitions.get(parent.def_id),
                            &mut parent_obj,
                        );
                        Value::Object(parent_obj)
                    })
                    .collect();
                interface_obj.insert("base_type".to_string(), Value::Array(parents));
            } else {
                let mut parent_obj = BTreeMap::new();
                Self::emit_type_info(
                    self.hir
                        .context
                        .definitions
                        .get(interface.parents[0].def_id),
                    &mut parent_obj,
                );
                interface_obj.insert("base_type".to_string(), Value::Object(parent_obj));
            }
        }

        let attributes: Vec<Value> = interface
            .attributes
            .iter()
            .map(|attr| {
                let mut attr_obj = BTreeMap::new();
                attr_obj.insert("name".to_string(), Value::String(attr.ident.name.clone()));
                self.emit_type_ref(&attr.ty, &mut attr_obj);
                if attr.is_readonly {
                    attr_obj.insert("readonly".to_string(), Value::Bool(true));
                }
                Value::Object(attr_obj)
            })
            .collect();
        interface_obj.insert("attributes".to_string(), Value::Array(attributes));

        obj.insert(def.ident.name.clone(), Value::Object(interface_obj));
    }

    fn emit_valuetype(&self, def: &Def, valuetype: &ValueTy, obj: &mut BTreeMap<String, Value>) {
        let mut valuetype_obj = BTreeMap::new();
        valuetype_obj.insert("kind".to_string(), Value::String("valuetype".to_string()));
        self.emit_annotations(&def.annotations, &mut valuetype_obj);

        if let Some(parent) = valuetype.parent {
            let mut base_obj = BTreeMap::new();
            Self::emit_type_info(
                self.hir.context.definitions.get(parent.def_id),
                &mut base_obj,
            );
            valuetype_obj.insert("base_type".to_string(), Value::Object(base_obj));
        }

        if let Some(supports) = valuetype.supports {
            let mut supports_obj = BTreeMap::new();
            Self::emit_type_info(
                self.hir.context.definitions.get(supports.def_id),
                &mut supports_obj,
            );
            valuetype_obj.insert("supports".to_string(), Value::Object(supports_obj));
        }

        let attributes: Vec<Value> = valuetype
            .members
            .iter()
            .map(|m| self.emit_member(m))
            .collect();
        valuetype_obj.insert("attributes".to_string(), Value::Array(attributes));

        valuetype_obj.insert("methods".to_string(), Value::Array(vec![]));

        obj.insert(def.ident.name.clone(), Value::Object(valuetype_obj));
    }

    fn emit_except(&self, def: &Def, except: &ExceptTy, obj: &mut BTreeMap<String, Value>) {
        let mut except_obj = BTreeMap::new();
        except_obj.insert("kind".to_string(), Value::String("exception".to_string()));
        self.emit_annotations(&def.annotations, &mut except_obj);

        let members: Vec<Value> = except.members.iter().map(|m| self.emit_member(m)).collect();
        except_obj.insert("members".to_string(), Value::Array(members));

        obj.insert(def.ident.name.clone(), Value::Object(except_obj));
    }

    fn emit_annotation_def(
        &self,
        def: &Def,
        _annotation: &ic_hir::hir::AnnotationTy,
        obj: &mut BTreeMap<String, Value>,
    ) {
        let mut ann_obj = BTreeMap::new();
        self.emit_annotations(&def.annotations, &mut ann_obj);
        ann_obj.insert("kind".to_string(), Value::String("annotation".to_string()));

        obj.insert(def.ident.name.clone(), Value::Object(ann_obj));
    }

    fn emit_forward_decl(def: &Def, decl: ic_hir::hir::Decl, obj: &mut BTreeMap<String, Value>) {
        use ic_hir::hir::Decl;

        let kind = match decl {
            Decl::Struct => "struct",
            Decl::Union => "union",
            Decl::Interface => "interface",
            Decl::Valuetype => "valuetype",
            Decl::Native => return,
        };

        let mut decl_obj = BTreeMap::new();
        decl_obj.insert("kind".to_string(), Value::String(kind.to_string()));

        obj.insert(def.ident.name.clone(), Value::Object(decl_obj));
    }

    fn emit_member(&self, member: &Member) -> Value {
        let mut member_obj = BTreeMap::new();
        member_obj.insert("name".to_string(), Value::String(member.ident.name.clone()));
        self.emit_annotations(&member.annotations, &mut member_obj);
        self.emit_type_ref(&member.ty, &mut member_obj);
        Value::Object(member_obj)
    }

    fn emit_annotations(&self, annotations: &[Ann], obj: &mut BTreeMap<String, Value>) {
        if annotations.is_empty() {
            return;
        }

        let mut ann_obj = BTreeMap::new();
        for ann in annotations {
            let scoped_name = if let Some(def_id) = ann.def_id {
                self.make_scoped_name(def_id)
            } else {
                ann.ident.name.clone()
            };

            if ann.args.is_empty() {
                ann_obj.insert(scoped_name, Value::Object(BTreeMap::new()));
            } else if ann.args.len() == 1 {
                ann_obj.insert(scoped_name, self.format_numeric(&ann.args[0].value));
            } else {
                let mut args_obj = BTreeMap::new();
                for arg in &ann.args {
                    args_obj.insert(arg.ident.name.clone(), self.format_numeric(&arg.value));
                }
                ann_obj.insert(scoped_name, Value::Object(args_obj));
            }
        }
        obj.insert("annotations".to_string(), Value::Object(ann_obj));
    }

    fn emit_type_ref(&self, ty: &Ty, obj: &mut BTreeMap<String, Value>) {
        match &ty.kind {
            TyKind::Primitive(prim) => {
                obj.insert(
                    "kind".to_string(),
                    Value::String(type_name(*prim).to_string()),
                );
            }
            TyKind::String { wide, bound, .. } => {
                let kind = if *wide { "wstring" } else { "string" };
                obj.insert("kind".to_string(), Value::String(kind.to_string()));
                if let Some(b) = bound {
                    obj.insert("string_max_length".to_string(), Value::Number((*b).into()));
                }
            }
            TyKind::Sequence { ty, bound, .. } => {
                obj.insert("kind".to_string(), Value::String("sequence".to_string()));
                obj.insert("type".to_string(), Value::String(self.type_to_string(ty)));
                if let Some(b) = bound {
                    obj.insert(
                        "sequence_max_length".to_string(),
                        Value::Number((*b).into()),
                    );
                }
            }
            TyKind::Map {
                key, elem, bound, ..
            } => {
                obj.insert("kind".to_string(), Value::String("map".to_string()));
                obj.insert(
                    "key_type".to_string(),
                    Value::String(self.type_to_string(key)),
                );
                obj.insert(
                    "value_type".to_string(),
                    Value::String(self.type_to_string(elem)),
                );
                if let Some(b) = bound {
                    obj.insert("map_max_length".to_string(), Value::Number((*b).into()));
                }
            }
            TyKind::Array { ty, len, .. } => {
                obj.insert("kind".to_string(), Value::String("array".to_string()));
                obj.insert("type".to_string(), Value::String(self.type_to_string(ty)));
                obj.insert(
                    "array_dimensions".to_string(),
                    Value::Array(vec![Value::Number((*len).into())]),
                );
            }
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                Self::emit_type_info(def, obj);
            }
            _ => {}
        }
    }

    fn emit_type_info(def: &Def, obj: &mut BTreeMap<String, Value>) {
        let kind = match &def.kind {
            DefKind::Struct(_) => "struct",
            DefKind::Union(_) => "union",
            DefKind::Enum(_) => "enum",
            DefKind::Bitmask(_) => "bitmask",
            DefKind::Bitset(_) => "bitset",
            DefKind::Alias(_) => "typedef",
            DefKind::Except(_) => "exception",
            DefKind::Valuetype(_) => "valuetype",
            DefKind::Interface(_) => "interface",
            _ => return,
        };

        obj.insert("kind".to_string(), Value::String(kind.to_string()));
        obj.insert("type".to_string(), Value::String(def.ident.name.clone()));
    }

    fn type_to_string(&self, ty: &Ty) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => type_name(*prim).to_string(),
            TyKind::String { wide, .. } => {
                if *wide {
                    "wstring".to_string()
                } else {
                    "string".to_string()
                }
            }
            TyKind::Adt(def_id) => self.make_scoped_name(*def_id),
            _ => "unknown".to_string(),
        }
    }

    fn make_scoped_name(&self, def_id: DefId) -> String {
        let mut parts = Vec::new();
        let mut current = Some(def_id);

        while let Some(id) = current {
            let def = self.hir.context.definitions.get(id);
            parts.push(def.ident.name.clone());
            current = def.parent;
        }

        parts.reverse();
        parts.join("::")
    }

    fn format_numeric(&self, value: &Numeric) -> Value {
        match value {
            Numeric::Bool(b) => Value::Bool(*b),
            Numeric::Char(value) | Numeric::WChar(value) => Value::String(value.to_string()),
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
            Numeric::String(value) | Numeric::WString(value) => Value::String(value.clone()),
            Numeric::Const(def_id) => Value::String(self.make_scoped_name(*def_id)),
            Numeric::Null
            | Numeric::Array { .. }
            | Numeric::Sequence { .. }
            | Numeric::Map { .. }
            | Numeric::Struct { .. }
            | Numeric::Union { .. } => Value::Null,
        }
    }
}

fn type_name(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Bool => "boolean",
        PrimitiveTy::Char => "char",
        PrimitiveTy::WChar => "wchar",
        PrimitiveTy::Int8 => "int8",
        PrimitiveTy::UInt8 => "uint8",
        PrimitiveTy::Int16 => "int16",
        PrimitiveTy::UInt16 => "uint16",
        PrimitiveTy::Int32 => "int32",
        PrimitiveTy::UInt32 => "uint32",
        PrimitiveTy::Int64 => "int64",
        PrimitiveTy::UInt64 => "uint64",
        PrimitiveTy::Float32 => "float32",
        PrimitiveTy::Float64 => "float64",
        PrimitiveTy::Float128 => "float128",
        PrimitiveTy::Void => "void",
    }
}
