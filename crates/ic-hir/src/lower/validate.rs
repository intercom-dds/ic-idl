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

//! Phase 4: Validation.
//!
//! This phase performs semantic validation on the fully-constructed HIR:
//! - Type consistency checks
//! - Circular dependency detection
//! - Inheritance validation
//! - Declaration/definition matching
//! - Member name uniqueness
//! - Union discriminator validation

use std::collections::{HashMap, HashSet};

use ic_diagnostic::{Diag, Label, error_span, warning_span};

use crate::{Context, hir::*};

/// Validates the HIR for semantic correctness.
pub struct Validator<'a> {
    ctx: &'a Context,
    errors: Vec<Diag>,
    /// Tracks visited types for cycle detection.
    visiting: HashSet<DefId>,
    /// Tracks completed types to avoid re-validation.
    validated: HashSet<DefId>,
}

impl<'a> Validator<'a> {
    fn new(ctx: &'a Context) -> Self {
        Self {
            ctx,
            errors: Vec::new(),
            visiting: HashSet::new(),
            validated: HashSet::new(),
        }
    }
    
    /// Gets a definition by ID.
    fn get_def(&self, id: DefId) -> &Def {
        self.ctx.definitions.get(id)
    }
    
    /// Validates that a type is complete (not just declared).
    fn validate_complete(&mut self, id: DefId) {
        let def = self.get_def(id);
        
        if def.flags.contains(DefFlags::IS_INCOMPLETE) {
            self.errors.push(error_span(
                format!("type `{}` is declared but not defined", def.ident.name),
                Label::new(def.span).message("declared here"),
            ));
        }
    }
    
    /// Validates type references (ensures they exist and are complete).
    fn validate_type_ref(&mut self, ty: &Ty) {
        match &ty.kind {
            TyKind::Adt(id) => {
                self.validate_complete(*id);
                self.validate_type(*id);
            },
            TyKind::Array { ty, len } => {
                if *len == 0 {
                    self.errors.push(error_span(
                        "array size must be greater than zero",
                        Label::new(ty.span).message("invalid array size"),
                    ));
                }
                self.validate_type_ref(ty);
            },
            TyKind::Sequence { ty, bound } => {
                if let Some(b) = bound {
                    if *b == 0 {
                        self.errors.push(warning_span(
                            "sequence bound of zero is unusual",
                            Label::new(ty.span).message("consider using a positive bound"),
                        ));
                    }
                }
                self.validate_type_ref(ty);
            },
            TyKind::Map { key, elem, .. } => {
                self.validate_type_ref(key);
                self.validate_type_ref(elem);
                // TODO: Validate that key type is valid for maps
            },
            TyKind::String { bound, .. } => {
                if let Some(b) = bound {
                    if *b == 0 {
                        self.errors.push(warning_span(
                            "string bound of zero is unusual",
                            Label::new(ty.span).message("consider using a positive bound"),
                        ));
                    }
                }
            },
            _ => {},
        }
    }
    
    /// Validates a struct definition.
    fn validate_struct(&mut self, id: DefId, def: &Def, struct_ty: &StructTy) {
        // Check parent inheritance
        if let Some(parent_id) = struct_ty.parent {
            let parent = self.get_def(parent_id);
            match &parent.kind {
                DefKind::Struct(_) => {
                    // Valid inheritance
                    self.validate_type(parent_id);
                },
                _ => {
                    self.errors.push(error_span(
                        format!("struct `{}` cannot inherit from non-struct type `{}`", 
                                def.ident.name, parent.ident.name),
                        Label::new(def.span).message("invalid inheritance"),
                    ));
                }
            }
        }
        
        // Validate members
        let mut member_names = HashSet::new();
        for member in &struct_ty.members {
            if !member_names.insert(&member.ident.name) {
                self.errors.push(error_span(
                    format!("duplicate member `{}` in struct `{}`", 
                            member.ident.name, def.ident.name),
                    Label::new(member.ident.span).message("duplicate member"),
                ));
            }
            
            self.validate_type_ref(&member.ty);
        }
    }
    
    /// Validates a union definition.
    fn validate_union(&mut self, id: DefId, def: &Def, union_ty: &UnionTy) {
        // Validate discriminator type
        self.validate_type_ref(&union_ty.disc);
        
        // Check that discriminator is an appropriate type
        match &union_ty.disc.kind {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::Bool | 
                PrimitiveTy::Char | PrimitiveTy::WChar |
                PrimitiveTy::Int8 | PrimitiveTy::UInt8 |
                PrimitiveTy::Int16 | PrimitiveTy::UInt16 |
                PrimitiveTy::Int32 | PrimitiveTy::UInt32 |
                PrimitiveTy::Int64 | PrimitiveTy::UInt64 => {
                    // Valid discriminator types
                },
                _ => {
                    self.errors.push(error_span(
                        format!("invalid discriminator type for union `{}`", def.ident.name),
                        Label::new(union_ty.disc.span).message("discriminator must be an integral type"),
                    ));
                }
            },
            TyKind::Adt(id) => {
                let disc_def = self.get_def(*id);
                if !matches!(disc_def.kind, DefKind::Enum(_)) {
                    self.errors.push(error_span(
                        format!("invalid discriminator type for union `{}`", def.ident.name),
                        Label::new(union_ty.disc.span).message("discriminator must be an enum or integral type"),
                    ));
                }
            },
            _ => {
                self.errors.push(error_span(
                    format!("invalid discriminator type for union `{}`", def.ident.name),
                    Label::new(union_ty.disc.span).message("discriminator must be an integral type"),
                ));
            }
        }
        
        // Validate variants
        let mut variant_names = HashSet::new();
        let mut case_values = HashSet::new();
        let mut has_default = false;
        
        for variant in &union_ty.variants {
            if !variant_names.insert(&variant.ident.name) {
                self.errors.push(error_span(
                    format!("duplicate variant `{}` in union `{}`", 
                            variant.ident.name, def.ident.name),
                    Label::new(variant.ident.span).message("duplicate variant"),
                ));
            }
            
            self.validate_type_ref(&variant.ty);
            
            // Check case labels
            for label in &variant.labels {
                if !case_values.insert(label) {
                    self.errors.push(error_span(
                        format!("duplicate case value in union `{}`", def.ident.name),
                        Label::new(variant.ident.span).message("case value already used"),
                    ));
                }
            }
            
            if variant.is_default {
                if has_default {
                    self.errors.push(error_span(
                        format!("multiple default cases in union `{}`", def.ident.name),
                        Label::new(variant.ident.span).message("default case already defined"),
                    ));
                }
                has_default = true;
            }
        }
    }
    
    /// Validates an interface definition.
    fn validate_interface(&mut self, id: DefId, def: &Def, interface: &InterfaceTy) {
        // Validate parent interfaces
        for &parent_id in &interface.parents {
            let parent = self.get_def(parent_id);
            match &parent.kind {
                DefKind::Interface(_) => {
                    self.validate_type(parent_id);
                },
                _ => {
                    self.errors.push(error_span(
                        format!("interface `{}` cannot inherit from non-interface type `{}`", 
                                def.ident.name, parent.ident.name),
                        Label::new(def.span).message("invalid inheritance"),
                    ));
                }
            }
        }
        
        // Validate prototypes
        let mut method_names = HashSet::new();
        for proto in &interface.prototypes {
            if !method_names.insert(&proto.ident.name) {
                self.errors.push(error_span(
                    format!("duplicate method `{}` in interface `{}`", 
                            proto.ident.name, def.ident.name),
                    Label::new(proto.ident.span).message("duplicate method"),
                ));
            }
            
            self.validate_type_ref(&proto.ty);
            
            for param in &proto.params {
                self.validate_type_ref(&param.ty);
            }
        }
        
        // Validate nested definitions
        for &child_id in &interface.definitions {
            self.validate_type(child_id);
        }
    }
    
    /// Validates an enum definition.
    fn validate_enum(&mut self, id: DefId, def: &Def, enum_ty: &EnumTy) {
        let mut field_names = HashSet::new();
        let mut field_values = HashSet::new();
        
        for field in &enum_ty.fields {
            if !field_names.insert(&field.ident.name) {
                self.errors.push(error_span(
                    format!("duplicate field `{}` in enum `{}`", 
                            field.ident.name, def.ident.name),
                    Label::new(field.ident.span).message("duplicate field"),
                ));
            }
            
            if !field_values.insert(field.value) {
                self.errors.push(error_span(
                    format!("duplicate value {} in enum `{}`", 
                            field.value, def.ident.name),
                    Label::new(field.ident.span).message("value already used"),
                ));
            }
        }
    }
    
    /// Validates circular dependencies.
    fn check_circular(&mut self, id: DefId, def: &Def) {
        if self.visiting.contains(&id) {
            self.errors.push(error_span(
                format!("circular dependency detected for type `{}`", def.ident.name),
                Label::new(def.span).message("type is part of a circular dependency"),
            ));
            return;
        }
        
        self.visiting.insert(id);
        
        // Check dependencies based on type
        match &def.kind {
            DefKind::Struct(s) => {
                if let Some(parent) = s.parent {
                    if let Some(parent_def) = self.ctx.definitions.try_get(parent) {
                        self.check_circular(parent, parent_def);
                    }
                }
            },
            DefKind::Interface(i) => {
                for &parent in &i.parents {
                    if let Some(parent_def) = self.ctx.definitions.try_get(parent) {
                        self.check_circular(parent, parent_def);
                    }
                }
            },
            _ => {},
        }
        
        self.visiting.remove(&id);
    }
    
    /// Main validation entry point for a type.
    fn validate_type(&mut self, id: DefId) {
        if self.validated.contains(&id) {
            return;
        }
        
        let def = self.get_def(id);
        
        // Check for circular dependencies
        self.check_circular(id, def);
        
        // Validate based on type
        match &def.kind {
            DefKind::Struct(s) => self.validate_struct(id, def, s),
            DefKind::Union(u) => self.validate_union(id, def, u),
            DefKind::Interface(i) => self.validate_interface(id, def, i),
            DefKind::Enum(e) => self.validate_enum(id, def, e),
            DefKind::Except(e) => {
                // Exceptions are like structs without inheritance
                let struct_ty = StructTy {
                    parent: None,
                    members: e.members.clone(),
                };
                self.validate_struct(id, def, &struct_ty);
            },
            DefKind::Alias(a) => {
                self.validate_type_ref(&a.ty);
            },
            DefKind::Module(m) => {
                // Validate all module members
                for &child_id in &m.definitions {
                    self.validate_type(child_id);
                }
            },
            DefKind::Annotation(a) => {
                // Validate annotation members
                for member in &a.members {
                    self.validate_type_ref(&member.ty);
                }
                for &child_id in &a.types {
                    self.validate_type(child_id);
                }
            },
            DefKind::Const(c) => {
                self.validate_type_ref(&c.ty);
                // TODO: Validate that constant value matches type
            },
            DefKind::Bitmask(b) => {
                // Validate bitmask flags
                let mut flag_names = HashSet::new();
                let mut flag_values = HashSet::new();
                
                for flag in &b.flags {
                    if !flag_names.insert(&flag.ident.name) {
                        self.errors.push(error_span(
                            format!("duplicate flag `{}` in bitmask `{}`", 
                                    flag.ident.name, def.ident.name),
                            Label::new(flag.ident.span).message("duplicate flag"),
                        ));
                    }
                    
                    if !flag_values.insert(flag.value) {
                        self.errors.push(error_span(
                            format!("duplicate value {} in bitmask `{}`", 
                                    flag.value, def.ident.name),
                            Label::new(flag.ident.span).message("value already used"),
                        ));
                    }
                }
            },
            DefKind::Decl(_) => {
                // Forward declarations are checked for completion elsewhere
            },
            _ => {},
        }
        
        self.validated.insert(id);
    }
    
    /// Validates all types in the HIR.
    fn validate_all(&mut self, order: &[DefId]) {
        // First pass: validate all forward declarations are defined
        for &id in order {
            self.validate_complete(id);
        }
        
        // Second pass: validate each type
        for &id in order {
            self.validate_type(id);
        }
    }
}

/// Validates the constructed HIR.
pub fn validate_hir(ctx: &Context, order: &[DefId]) -> Vec<Diag> {
    let mut validator = Validator::new(ctx);
    validator.validate_all(order);
    validator.errors
}