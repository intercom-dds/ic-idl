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

//! Phase 2: Type resolution.
//!
//! This phase walks the AST again and:
//! - Resolves all type references (Path -> DefId)
//! - Fills in struct members, union variants, etc.
//! - Resolves inheritance relationships
//! - Does NOT evaluate constant expressions yet

use std::collections::HashMap;

use ic_diagnostic::{Diag, Label, error_span};
use ic_syntax::{Item, Path, Span};

use crate::{Context, hir::*};
use super::collect::NameMap;

/// Resolves type references in the HIR.
pub struct TypeResolver<'a> {
    ctx: &'a mut Context,
    name_map: &'a NameMap,
    errors: Vec<Diag>,
    /// Maps AST items to their DefIds for easy lookup.
    item_map: HashMap<ItemKey, DefId>,
}

/// Key for looking up items by their AST identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ItemKey {
    name: String,
    kind: &'static str,
}

impl<'a> TypeResolver<'a> {
    fn new(ctx: &'a mut Context, name_map: &'a NameMap) -> Self {
        Self {
            ctx,
            name_map,
            errors: Vec::new(),
            item_map: HashMap::new(),
        }
    }
    
    /// Resolves a path to a DefId.
    fn resolve_path(&mut self, path: &Path) -> Option<DefId> {
        let qualified = path_to_string(path);
        
        match self.name_map.get(&qualified) {
            Some(&id) => Some(id),
            None => {
                // Check if it's a primitive type
                if path.segments.len() == 1 && path.leading_colons.is_none() {
                    return None;  // Might be a primitive, handled in resolve_type
                }
                
                self.errors.push(error_span(
                    format!("unresolved type `{}`", qualified),
                    Label::new(ic_syntax::util::path_span(path)).message("unknown type"),
                ));
                None
            }
        }
    }
    
    /// Resolves an AST type to a HIR type.
    fn resolve_type(&mut self, ty: &ic_syntax::Type) -> Ty {
        use ic_syntax::Type;
        
        match ty {
            Type::Any(v) => Ty {
                kind: TyKind::Any,
                span: v.span,
            },
            Type::Fixed(v) => Ty {
                kind: TyKind::Fixed,
                span: v.span,
            },
            Type::Sequence(v) => Ty {
                kind: TyKind::Sequence {
                    ty: Box::new(self.resolve_type(&v.ty)),
                    bound: None,  // Will be filled in evaluation phase
                },
                span: v.span,
            },
            Type::String(v) => Ty {
                kind: TyKind::String {
                    wide: v.wide,
                    bound: None,  // Will be filled in evaluation phase
                },
                span: v.span,
            },
            Type::Map(v) => Ty {
                kind: TyKind::Map {
                    key: Box::new(self.resolve_type(&v.key)),
                    elem: Box::new(self.resolve_type(&v.value)),
                    bound: None,  // Will be filled in evaluation phase
                },
                span: v.span,
            },
            Type::Path(v) => {
                // Check for primitive types first
                if v.segments.len() == 1 && v.leading_colons.is_none() {
                    if let Some(prim) = resolve_primitive(&v.segments[0].name) {
                        return Ty {
                            kind: TyKind::Primitive(prim),
                            span: ic_syntax::util::path_span(v),
                        };
                    }
                }
                
                // Otherwise resolve as user-defined type
                let kind = self.resolve_path(v)
                    .map(TyKind::Adt)
                    .unwrap_or(TyKind::Any);
                
                Ty {
                    kind,
                    span: ic_syntax::util::path_span(v),
                }
            }
        }
    }
    
    /// Resolves a declarator into (name, type).
    fn resolve_declarator(
        &mut self,
        decl: &ic_syntax::Declarator,
        base_ty: Ty,
    ) -> (Ident, Ty) {
        match decl {
            ic_syntax::Declarator::Simple(name) => {
                (Ident { name: name.clone(), span: Span::default() }, base_ty)
            },
            ic_syntax::Declarator::Array(arr) => {
                // Build array type from innermost to outermost
                let mut ty = base_ty;
                for _ in &arr.bounds {
                    ty = Ty {
                        span: ty.span,
                        kind: TyKind::Array {
                            ty: Box::new(ty.clone()),
                            len: 0,  // Will be filled in evaluation phase
                        },
                    };
                }
                (arr.ident.clone(), ty)
            }
        }
    }
    
    /// Resolves struct members.
    fn resolve_struct_members(&mut self, def: &ic_syntax::StructDef) -> Vec<Member> {
        let mut members = Vec::new();
        
        for field in &def.members {
            let base_ty = self.resolve_type(&field.ty);
            
            for decl in &field.names {
                let (ident, ty) = self.resolve_declarator(decl, base_ty.clone());
                members.push(Member {
                    ident,
                    ty,
                    annotations: Vec::new(),  // TODO: Resolve annotations
                });
            }
        }
        
        members
    }
    
    /// Resolves a struct definition.
    fn resolve_struct(&mut self, id: DefId, def: &ic_syntax::StructDef) {
        let parent = def.parent.as_ref().and_then(|p| self.resolve_path(p));
        let members = self.resolve_struct_members(def);
        
        // Update the definition
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.remove(DefFlags::IS_INCOMPLETE);
        
        if let DefKind::Struct(struct_ty) = &mut hir_def.kind {
            struct_ty.parent = parent;
            struct_ty.members = members;
        }
    }
    
    /// Resolves union variants.
    fn resolve_union(&mut self, id: DefId, def: &ic_syntax::UnionDef) {
        let disc = self.resolve_type(&def.disc.ty);
        let mut variants = Vec::new();
        
        for field in &def.fields {
            use ic_syntax::UnionElement;
            
            let variant = match &field.field {
                UnionElement::Member(m) => {
                    let base_ty = self.resolve_type(&m.ty);
                    let (ident, ty) = self.resolve_declarator(&m.decl, base_ty);
                    
                    Variant {
                        annotations: Vec::new(),  // TODO
                        ident,
                        ty,
                        labels: Vec::new(),  // Will be filled in evaluation phase
                        is_default: field.labels.iter().any(|l| matches!(l, ic_syntax::Label::Default(_))),
                    }
                },
                UnionElement::Null(n) => {
                    Variant {
                        annotations: Vec::new(),
                        ident: Ident { name: "null".to_string(), span: n.span },
                        ty: Ty { kind: TyKind::Any, span: n.span },
                        labels: Vec::new(),  // Will be filled in evaluation phase
                        is_default: field.labels.iter().any(|l| matches!(l, ic_syntax::Label::Default(_))),
                    }
                }
            };
            
            variants.push(variant);
        }
        
        // Update the definition
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.remove(DefFlags::IS_INCOMPLETE);
        
        if let DefKind::Union(union_ty) = &mut hir_def.kind {
            union_ty.disc = disc;
            union_ty.variants = variants;
        }
    }
    
    /// Resolves an exception definition.
    fn resolve_exception(&mut self, id: DefId, def: &ic_syntax::ExceptDef) {
        let members = self.resolve_struct_members(&ic_syntax::StructDef {
            ident: def.ident.clone(),
            parent: None,
            members: def.members.clone(),
            annotations: def.annotations.clone(),
            span: def.span,
        });
        
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.remove(DefFlags::IS_INCOMPLETE);
        
        if let DefKind::Except(except_ty) = &mut hir_def.kind {
            except_ty.members = members;
        }
    }
    
    /// Resolves an alias definition.
    fn resolve_alias(&mut self, id: DefId, def: &ic_syntax::AliasDef, decl_idx: usize) {
        let base_ty = self.resolve_type(&def.ty);
        let (_, ty) = self.resolve_declarator(&def.decl[decl_idx], base_ty);
        
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.remove(DefFlags::IS_INCOMPLETE);
        
        if let DefKind::Alias(alias_ty) = &mut hir_def.kind {
            alias_ty.ty = ty;
        }
    }
    
    /// Resolves an interface definition.
    fn resolve_interface(&mut self, id: DefId, def: &ic_syntax::InterfaceDef) {
        let parents = def.inherits.iter()
            .filter_map(|p| self.resolve_path(p))
            .collect();
        
        let mut prototypes = Vec::new();
        
        for member in &def.members {
            if let ic_syntax::InterfaceMember::Proto(proto) = member {
                let ret_ty = self.resolve_type(&proto.ret);
                let mut params = Vec::new();
                
                for param in &proto.params {
                    let param_ty = self.resolve_type(&param.ty);
                    let (ident, ty) = self.resolve_declarator(&param.decl, param_ty);
                    
                    params.push(Parameter {
                        ident,
                        ty,
                        kind: param.kind.unwrap_or(ParamKind::In),
                    });
                }
                
                prototypes.push(ProtoTy {
                    ident: proto.ident.clone(),
                    ty: ret_ty,
                    params,
                });
            }
        }
        
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.remove(DefFlags::IS_INCOMPLETE);
        
        if let DefKind::Interface(interface) = &mut hir_def.kind {
            interface.parents = parents;
            interface.prototypes = prototypes;
        }
    }
    
    /// Builds a mapping from AST items to their DefIds.
    fn build_item_map(&mut self, items: &[Item]) {
        for item in items {
            let key = match item {
                Item::StructValue(v) => ItemKey { name: v.ident.name.clone(), kind: "struct" },
                Item::UnionValue(v) => ItemKey { name: v.ident.name.clone(), kind: "union" },
                Item::EnumValue(v) => ItemKey { name: v.ident.name.clone(), kind: "enum" },
                Item::ExceptionValue(v) => ItemKey { name: v.ident.name.clone(), kind: "exception" },
                Item::BitmaskValue(v) => ItemKey { name: v.ident.name.clone(), kind: "bitmask" },
                Item::InterfaceValue(v) => ItemKey { name: v.ident.name.clone(), kind: "interface" },
                Item::ModuleValue(v) => ItemKey { name: v.ident.name.clone(), kind: "module" },
                Item::AnnotationValue(v) => ItemKey { name: v.ident.name.clone(), kind: "annotation" },
                _ => continue,
            };
            
            // Look up in global scope for now (TODO: handle nested scopes)
            if let Some(&id) = self.name_map.get(&key.name) {
                self.item_map.insert(key, id);
            }
        }
    }
    
    /// Resolves all type references in the HIR.
    fn resolve_all(&mut self, items: &[Item]) {
        // First pass: build item map
        self.build_item_map(items);
        
        // Second pass: resolve each item
        for item in items {
            match item {
                Item::StructValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey { 
                        name: v.ident.name.clone(), 
                        kind: "struct" 
                    }) {
                        self.resolve_struct(id, v);
                    }
                },
                Item::UnionValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey { 
                        name: v.ident.name.clone(), 
                        kind: "union" 
                    }) {
                        self.resolve_union(id, v);
                    }
                },
                Item::ExceptionValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey { 
                        name: v.ident.name.clone(), 
                        kind: "exception" 
                    }) {
                        self.resolve_exception(id, v);
                    }
                },
                Item::InterfaceValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey { 
                        name: v.ident.name.clone(), 
                        kind: "interface" 
                    }) {
                        self.resolve_interface(id, v);
                    }
                },
                Item::AliasValue(v) => {
                    // Handle multiple declarators
                    for (idx, decl) in v.decl.iter().enumerate() {
                        let name = match decl {
                            ic_syntax::Declarator::Simple(n) => n.clone(),
                            ic_syntax::Declarator::Array(a) => a.ident.name.clone(),
                        };
                        
                        if let Some(&id) = self.name_map.get(&name) {
                            self.resolve_alias(id, v, idx);
                        }
                    }
                },
                // TODO: Handle other item types
                _ => {},
            }
        }
    }
}

/// Converts a path to its string representation.
fn path_to_string(path: &Path) -> String {
    let segments = path.segments.iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    
    if path.leading_colons.is_some() {
        format!("::{}", segments)
    } else {
        segments
    }
}

/// Resolves a primitive type name.
fn resolve_primitive(name: &str) -> Option<PrimitiveTy> {
    match name {
        "void" => Some(PrimitiveTy::Void),
        "boolean" => Some(PrimitiveTy::Bool),
        "char" => Some(PrimitiveTy::Char),
        "wchar" => Some(PrimitiveTy::WChar),
        "int8" => Some(PrimitiveTy::Int8),
        "octet" | "uint8" => Some(PrimitiveTy::UInt8),
        "int16" => Some(PrimitiveTy::Int16),
        "uint16" => Some(PrimitiveTy::UInt16),
        "int32" => Some(PrimitiveTy::Int32),
        "uint32" => Some(PrimitiveTy::UInt32),
        "int64" => Some(PrimitiveTy::Int64),
        "uint64" => Some(PrimitiveTy::UInt64),
        "float" => Some(PrimitiveTy::Float32),
        "double" => Some(PrimitiveTy::Float64),
        "long double" => Some(PrimitiveTy::Float128),
        _ => None,
    }
}

/// Resolves all type references in the HIR.
pub fn resolve_types(
    ctx: &mut Context,
    name_map: &NameMap,
    items: &[Item],
) -> Vec<Diag> {
    let mut resolver = TypeResolver::new(ctx, name_map);
    resolver.resolve_all(items);
    resolver.errors
}