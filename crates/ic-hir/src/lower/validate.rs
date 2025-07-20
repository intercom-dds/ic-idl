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

use ic_diagnostic::{Diag, Label, error_span};

use crate::Context;
use crate::hir::{
    Decl, Def, DefFlags, DefId, DefKind, InterfaceTy, PrimitiveTy, StructTy, Ty, TyKind, UnionTy,
};

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
                // Don't validate_type here - it would cause infinite recursion
                // The type will be validated separately in validate_all
            }
            TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
                self.validate_type_ref(ty);
            }
            TyKind::Map { key, elem, .. } => {
                self.validate_type_ref(key);
                self.validate_type_ref(elem);
            }
            _ => {}
        }
    }

    /// Validates a struct definition.
    fn validate_struct(&mut self, id: DefId, struct_ty: &StructTy) {
        let (def_name, def_span) = {
            let def = self.get_def(id);
            (def.ident.name.clone(), def.span)
        };
        // Check parent inheritance
        if let Some(parent_id) = struct_ty.parent {
            let parent = self.get_def(parent_id);
            match &parent.kind {
                DefKind::Struct(_) => {
                    // Valid inheritance - parent will be validated separately
                }
                DefKind::Decl(Decl::Struct) => {
                    // Parent is forward-declared but not yet defined
                    self.errors.push(
                        error_span(
                            format!(
                                "struct `{}` cannot inherit from incomplete type `{}`",
                                def_name, parent.ident.name
                            ),
                            Label::new(def_span).message("invalid inheritance"),
                        )
                        .label(
                            Label::new(parent.ident.span)
                                .message("forward declaration here, but no definition found"),
                        ),
                    );
                }
                _ => {
                    self.errors.push(error_span(
                        format!(
                            "struct `{}` cannot inherit from non-struct type `{}`",
                            def_name, parent.ident.name
                        ),
                        Label::new(def_span).message("invalid inheritance"),
                    ));
                }
            }
        }

        // Validate members
        for member in &struct_ty.members {
            self.validate_type_ref(&member.ty);
        }
    }

    /// Validates a union definition.
    #[allow(clippy::match_same_arms)]
    fn validate_union(&mut self, id: DefId, union_ty: &UnionTy) {
        let (def_name, _def_span) = {
            let def = self.get_def(id);
            (def.ident.name.clone(), def.span)
        };
        // Validate discriminator type
        self.validate_type_ref(&union_ty.disc);

        // Check that discriminator is an appropriate type
        match &union_ty.disc.kind {
            TyKind::Primitive(
                PrimitiveTy::Bool
                | PrimitiveTy::Char
                | PrimitiveTy::WChar
                | PrimitiveTy::Int8
                | PrimitiveTy::UInt8
                | PrimitiveTy::Int16
                | PrimitiveTy::UInt16
                | PrimitiveTy::Int32
                | PrimitiveTy::UInt32
                | PrimitiveTy::Int64
                | PrimitiveTy::UInt64,
            ) => {
                // Valid discriminator types
            }
            TyKind::Primitive(_) => {
                self.errors.push(error_span(
                    format!("invalid discriminator type for union `{def_name}`"),
                    Label::new(union_ty.disc.span)
                        .message("discriminator must be an integral type"),
                ));
            }
            TyKind::Adt(id) => {
                let disc_def = self.get_def(*id);
                if !matches!(disc_def.kind, DefKind::Enum(_)) {
                    self.errors.push(error_span(
                        format!("invalid discriminator type for union `{def_name}`"),
                        Label::new(union_ty.disc.span)
                            .message("discriminator must be an enum or integral type"),
                    ));
                }
            }
            _ => {
                self.errors.push(error_span(
                    format!("invalid discriminator type for union `{def_name}`"),
                    Label::new(union_ty.disc.span)
                        .message("discriminator must be an integral type"),
                ));
            }
        }

        // Validate variants
        for variant in &union_ty.variants {
            self.validate_type_ref(&variant.ty);

            // Check case labels
            // TODO: Implement duplicate case value checking that handles float values
            // For now, skip this validation since Numeric contains float types which don't implement Eq/Hash
        }
    }

    /// Validates an interface definition.
    fn validate_interface(&mut self, id: DefId, interface: &InterfaceTy) {
        let (def_name, def_span) = {
            let def = self.get_def(id);
            (def.ident.name.clone(), def.span)
        };
        // Validate parent interfaces
        for &parent_id in &interface.parents {
            let parent = self.get_def(parent_id);
            match &parent.kind {
                DefKind::Interface(_) => {
                    // Valid inheritance - parent will be validated separately
                }
                DefKind::Decl(Decl::Interface) => {
                    // Parent is forward-declared but not yet defined
                    self.errors.push(
                        error_span(
                            format!(
                                "interface `{}` cannot inherit from incomplete type `{}`",
                                def_name, parent.ident.name
                            ),
                            Label::new(def_span).message("invalid inheritance"),
                        )
                        .label(
                            Label::new(parent.ident.span)
                                .message("forward declaration here, but no definition found"),
                        ),
                    );
                }
                _ => {
                    self.errors.push(error_span(
                        format!(
                            "interface `{}` cannot inherit from non-interface type `{}`",
                            def_name, parent.ident.name
                        ),
                        Label::new(def_span).message("invalid inheritance"),
                    ));
                }
            }
        }

        // Validate prototypes
        let mut method_names = HashSet::new();
        for proto in &interface.prototypes {
            if !method_names.insert(proto.ident.name.as_str()) {
                self.errors.push(error_span(
                    format!(
                        "duplicate method `{}` in interface `{}`",
                        proto.ident.name, def_name
                    ),
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

    /// Validates circular dependencies.
    fn check_circular(&mut self, id: DefId) {
        if self.visiting.contains(&id) {
            let def = self.get_def(id);
            self.errors.push(error_span(
                format!("circular dependency detected for type `{}`", def.ident.name),
                Label::new(def.span).message("type is part of a circular dependency"),
            ));
            return;
        }

        self.visiting.insert(id);

        // Check dependencies based on type
        let parents_to_check = {
            let def = self.get_def(id);
            match &def.kind {
                DefKind::Struct(s) => s.parent.into_iter().collect::<Vec<_>>(),
                DefKind::Interface(i) => i.parents.clone(),
                _ => Vec::new(),
            }
        };

        for parent in parents_to_check {
            self.check_circular(parent);
        }

        self.visiting.remove(&id);
    }

    /// Main validation entry point for a type.
    fn validate_type(&mut self, id: DefId) {
        if self.validated.contains(&id) {
            return;
        }

        // Check for circular dependencies
        self.check_circular(id);

        // Validate based on type
        let (def_kind, _def_name) = {
            let def = self.get_def(id);
            (def.kind.clone(), def.ident.name.clone())
        };
        match &def_kind {
            DefKind::Struct(s) => self.validate_struct(id, s),
            DefKind::Union(u) => self.validate_union(id, u),
            DefKind::Interface(i) => self.validate_interface(id, i),
            DefKind::Except(e) => {
                // Exceptions are like structs without inheritance
                let struct_ty = StructTy {
                    parent: None,
                    members: e.members.clone(),
                };
                self.validate_struct(id, &struct_ty);
            }
            DefKind::Alias(a) => {
                self.validate_type_ref(&a.ty);
            }
            DefKind::Module(m) => {
                // Validate all module members
                for &child_id in &m.definitions {
                    self.validate_type(child_id);
                }
            }
            DefKind::Annotation(a) => {
                // Validate annotation members
                for member in &a.members {
                    self.validate_type_ref(&member.ty);
                }
                for &child_id in &a.types {
                    self.validate_type(child_id);
                }
            }
            DefKind::Const(c) => {
                self.validate_type_ref(&c.ty);
                // TODO: Validate that constant value matches type
            }
            DefKind::Enum(_)
            | DefKind::Bitmask(_)
            | DefKind::Bitset(_)
            | DefKind::Decl(_)
            | DefKind::Valuetype(_) => {
                // Duplicate checks moved to ic-lint for Enum and Bitmask
                // Forward declarations are checked for completion elsewhere
                // Valuetype validation handled elsewhere
            }
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

        // Third pass: validate forward declarations match definitions
        self.validate_forward_declarations(order);
    }

    /// Validates that forward declarations match their definitions.
    #[allow(clippy::too_many_lines)]
    fn validate_forward_declarations(&mut self, order: &[DefId]) {
        // Group definitions by name AND parent scope
        let mut definitions_by_name_and_scope: HashMap<(String, Option<DefId>), Vec<DefId>> =
            HashMap::new();

        for &id in order {
            let def = self.get_def(id);
            definitions_by_name_and_scope
                .entry((def.ident.name.clone(), def.parent))
                .or_default()
                .push(id);
        }

        // Check each group of definitions with the same name in the same scope
        for ((name, _parent), ids) in definitions_by_name_and_scope {
            if ids.len() < 2 {
                continue; // No duplicates to check
            }

            // Find the actual definition (non-Decl) if any
            let mut actual_def: Option<(DefId, &str)> = None;
            let mut forward_decls: Vec<DefId> = Vec::new();

            for &id in &ids {
                let def = self.get_def(id);
                match &def.kind {
                    DefKind::Decl(_decl_type) => {
                        forward_decls.push(id);
                    }
                    DefKind::Struct(_) => {
                        if let Some((first_id, _)) = actual_def {
                            let first_def = self.get_def(first_id);
                            self.errors.push(
                                error_span(
                                    format!("multiple definitions of struct `{name}`"),
                                    Label::new(def.span).message("redefined here"),
                                )
                                .label(Label::new(first_def.span).message("first defined here")),
                            );
                        }
                        actual_def = Some((id, "struct"));
                    }
                    DefKind::Union(_) => {
                        if let Some((first_id, _)) = actual_def {
                            let first_def = self.get_def(first_id);
                            self.errors.push(
                                error_span(
                                    format!("multiple definitions of union `{name}`"),
                                    Label::new(def.span).message("redefined here"),
                                )
                                .label(Label::new(first_def.span).message("first defined here")),
                            );
                        }
                        actual_def = Some((id, "union"));
                    }
                    DefKind::Interface(_) => {
                        if let Some((first_id, _)) = actual_def {
                            let first_def = self.get_def(first_id);
                            self.errors.push(
                                error_span(
                                    format!("multiple definitions of interface `{name}`"),
                                    Label::new(def.span).message("redefined here"),
                                )
                                .label(Label::new(first_def.span).message("first defined here")),
                            );
                        }
                        actual_def = Some((id, "interface"));
                    }
                    DefKind::Valuetype(_) => {
                        if let Some((first_id, _)) = actual_def {
                            let first_def = self.get_def(first_id);
                            self.errors.push(
                                error_span(
                                    format!("multiple definitions of valuetype `{name}`"),
                                    Label::new(def.span).message("redefined here"),
                                )
                                .label(Label::new(first_def.span).message("first defined here")),
                            );
                        }
                        actual_def = Some((id, "valuetype"));
                    }
                    _ => {} // Other types don't have forward declarations
                }
            }

            // Check that all forward declarations match the actual definition
            if let Some((def_id, def_type)) = actual_def {
                for &decl_id in &forward_decls {
                    let decl_def = self.get_def(decl_id);
                    if let DefKind::Decl(decl_type) = &decl_def.kind {
                        let decl_type_str = match decl_type {
                            Decl::Struct => "struct",
                            Decl::Union => "union",
                            Decl::Native => "native",
                            Decl::Interface => "interface",
                            Decl::Valuetype => "valuetype",
                        };

                        if decl_type_str != def_type {
                            let actual_def = self.get_def(def_id);
                            self.errors.push(
                                error_span(
                                    format!(
                                        "forward declaration of `{name}` as {decl_type_str} \
                                         conflicts with {def_type} definition"
                                    ),
                                    Label::new(decl_def.span).message("forward declared here"),
                                )
                                .label(
                                    Label::new(actual_def.span)
                                        .message(format!("defined as {def_type} here")),
                                ),
                            );
                        }
                    }
                }
            } else if !forward_decls.is_empty() {
                // We have forward declarations but no definition
                // This is handled by validate_complete already
            }
        }
    }
}

/// Validates the constructed HIR.
pub fn validate_hir(ctx: &Context, order: &[DefId]) -> Vec<Diag> {
    let mut validator = Validator::new(ctx);
    validator.validate_all(order);
    validator.errors
}
