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

//! Processing for type items: struct, union, interface, valuetype, native.

use ic_syntax::{InterfaceDef, StructDef, UnionDef, ValuetypeDef};

use super::LoweringContext;
use super::registry::DefKindTag;
use super::type_resolver::TypeResolver;
use super::utils::TyExt;
use crate::hir::{
    Decl, Def, DefFlags, DefId, DefKind, InterfaceTy, Member, StructTy, Ty, TyKind,
    UnionTy, ValueTy, Variant,
};
use crate::scope::ScopeId;

/// Processes type items (struct, union, interface, valuetype, native).
pub struct TypeItemProcessor<'ctx> {
    ctx: &'ctx mut LoweringContext,
    current_scope: ScopeId,
}

impl<'ctx> TypeItemProcessor<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    /// Process a struct definition.
    pub fn process_struct(&mut self, s: &StructDef) {
        // Resolve parent type if present
        let parent = if let Some(ref parent_type) = s.parent {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver
                .resolve_path_type(parent_type)
                .and_then(|ty| {
                    if let Some(parent_id) = ty.as_adt() {
                        Some(parent_id)
                    } else {
                        self.ctx.diagnostics.error(
                            "parent must be a struct type".to_string(),
                            ic_diagnostic::Label::new(super::utils::path_span(parent_type))
                                .message("expected struct type"),
                        );
                        None
                    }
                })
        } else {
            None
        };

        // Create scope and process members
        let (_scope, members) = self.process_members(&s.members);

        // Create the complete struct definition
        let struct_ty = StructTy { parent, members };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: s.ident.clone(),
            parent: None,
            annotations: Vec::new(), // TODO: Convert annotations
            span: (s.ident.span),
            kind: DefKind::Struct(struct_ty),
            flags: DefFlags::nil(),
        });

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &s.ident,
                DefKindTag::Struct,
                def_id,
                &mut self.ctx.diagnostics,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, s.ident.name.clone(), def_id);

            // Record as a top-level type
            self.ctx.order.push(def_id);
        }
    }

    /// Process an interface definition.
    pub fn process_interface(&mut self, i: &InterfaceDef) {
        // Resolve parent interfaces
        let mut parents = Vec::new();

        for parent_path in &i.inherits {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            if let Some(ty) = resolver.resolve_path_type(parent_path) {
                if let Some(parent_id) = ty.as_adt() {
                    parents.push(parent_id);
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be an interface type".to_string(),
                        ic_diagnostic::Label::new(ic_syntax::util::path_span(parent_path))
                            .message("expected interface type"),
                    );
                }
            }
        }

        // Create scope and process members
        let scope = self
            .ctx
            .context
            .scopes
            .create_child_scope(self.current_scope, i.ident.name.clone(), None);

        // TODO: Process interface members (operations, attributes)
        let prototypes = Vec::new();
        let attributes = Vec::new();

        // Create the complete interface definition
        let interface_ty = InterfaceTy {
            parents,
            prototypes,
            attributes,
            is_local: i.local.is_some(),
            // TODO: process members
            definitions: vec![],
        };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: i.ident.clone(),
            parent: None,
            annotations: Vec::new(), // TODO: Convert annotations
            span: (i.ident.span),
            kind: DefKind::Interface(interface_ty),
            flags: DefFlags::nil(),
        });

        // Update the scope's def_id
        self.ctx.context.scopes.get_scope_mut(scope).def_id = Some(def_id);

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &i.ident,
                DefKindTag::Interface,
                def_id,
                &mut self.ctx.diagnostics,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, i.ident.name.clone(), def_id);

            // Record as a top-level type
            self.ctx.order.push(def_id);
        }
    }

    /// Process a union definition.
    pub fn process_union(&mut self, u: &UnionDef) {
        // Resolve discriminator type
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let disc = resolver.resolve_type(&u.disc.ty).unwrap_or_else(|| {
            // Use a default type on error
            Ty {
                span: (ic_syntax::util::ty_span(&u.disc.ty)),
                kind: crate::hir::TyKind::Primitive(crate::hir::PrimitiveTy::Int32),
            }
        });

        // Create scope and process branches
        let scope = self
            .ctx
            .context
            .scopes
            .create_child_scope(self.current_scope, u.ident.name.clone(), None);

        // Process union variants
        let variants = self.process_union_variants(&u.fields);

        // Create the complete union definition
        let union_ty = UnionTy {
            disc,
            variants,
        };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: u.ident.clone(),
            parent: None,
            annotations: Vec::new(), // TODO: Convert annotations
            span: (u.ident.span),
            kind: DefKind::Union(union_ty),
            flags: DefFlags::nil(),
        });

        // Update the scope's def_id
        self.ctx.context.scopes.get_scope_mut(scope).def_id = Some(def_id);

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &u.ident,
                DefKindTag::Union,
                def_id,
                &mut self.ctx.diagnostics,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, u.ident.name.clone(), def_id);

            // Record as a top-level type
            self.ctx.order.push(def_id);
        }
    }

    /// Process a valuetype definition.
    pub fn process_valuetype(&mut self, v: &ValuetypeDef) {
        // Resolve parent type if present
        let parent = if let Some(ref parent_type) = v.inherits {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver
                .resolve_path_type(parent_type)
                .and_then(|ty| ty.as_adt())
        } else {
            None
        };

        // TODO: Process supports interface
        let supports = if let Some(ref supports_type) = v.supports {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver
                .resolve_path_type(supports_type)
                .and_then(|ty| ty.as_adt())
        } else {
            None
        };

        // TODO: Process valuetype elements properly
        let members = Vec::new();

        // Create the complete valuetype definition
        let value_ty = ValueTy {
            parent,
            supports,
            members,
            prototypes: Vec::new(), // TODO: implement valuetype prototypes
            attributes: Vec::new(), // TODO: implement valuetype attributes
            definitions: Vec::new(), // TODO: implement valuetype definitions
        };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: v.ident.clone(),
            parent: None,
            annotations: Vec::new(), // TODO: Convert annotations
            span: (v.ident.span),
            kind: DefKind::Valuetype(value_ty),
            flags: DefFlags::nil(),
        });

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &v.ident,
                DefKindTag::Valuetype,
                def_id,
                &mut self.ctx.diagnostics,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, v.ident.name.clone(), def_id);

            // Record as a top-level type
            self.ctx.order.push(def_id);
        }
    }

    /// Process a forward declaration.
    pub fn process_forward_decl(&mut self, decl: &ic_syntax::Decl) {
        let hir_decl_kind = match decl.kind {
            ic_syntax::DeclKind::Struct => Decl::Struct,
            ic_syntax::DeclKind::Union => Decl::Union,
            ic_syntax::DeclKind::Interface => Decl::Interface,
            ic_syntax::DeclKind::Valuetype => Decl::Valuetype,
            ic_syntax::DeclKind::Native => Decl::Native,
        };

        let def_id = self.create_forward_declaration(&decl.ident, hir_decl_kind);

        // Record as a top-level type
        self.ctx.order.push(def_id);
    }

    /// Create a forward declaration.
    fn create_forward_declaration(&mut self, ident: &ic_syntax::Ident, kind: Decl) -> DefId {
        // Allocate the forward declaration definition
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent: None,
            annotations: Vec::new(),
            span: (ident.span),
            kind: DefKind::Decl(kind),
            flags: DefFlags::IS_INCOMPLETE,
        });

        // Register with the registry
        if let Some(registered_id) = self.ctx.registry.register_forward_decl(
            self.current_scope,
            ident,
            kind,
            def_id,
            &mut self.ctx.diagnostics,
        ) {
            if registered_id != def_id {
                // Return existing forward declaration
                return registered_id;
            }
        }

        // Register in scope
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, ident.name.clone(), def_id);

        def_id
    }

    /// Process members and create a scope for them.
    fn process_members(&mut self, fields: &[ic_syntax::Field]) -> (ScopeId, Vec<Member>) {
        let scope = self.ctx.context.scopes.create_child_scope(self.current_scope, "_members_".to_string(), None);
        let mut members = Vec::new();

        for field in fields {
            // Resolve the type once for this field
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let ty = match resolver.resolve_type(&field.ty) {
                Some(ty) => ty,
                None => continue, // Error already reported
            };

            // Process each declarator in the field
            for decl in &field.names {
                let ident = ic_syntax::Ident {
                    name: ic_syntax::util::decl_name(decl).to_string(),
                    span: ic_syntax::util::decl_span(decl),
                };

                members.push(Member {
                    ident: ident.clone(),
                    ty: ty.clone(),
                    annotations: Vec::new(), // TODO: Convert annotations from field.annotations
                });
            }
        }

        (scope, members)
    }

    /// Process union variants.
    fn process_union_variants(&mut self, fields: &[ic_syntax::UnionField]) -> Vec<Variant> {
        let mut variants = Vec::new();

        for field in fields {
            match &field.field {
                ic_syntax::UnionElement::Member(member) => {
                    // Resolve the type for this variant
                    let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                    let ty = match resolver.resolve_type(&member.ty) {
                        Some(ty) => ty,
                        None => continue, // Error already reported
                    };

                    // Get the name from the declarator
                    let ident = ic_syntax::Ident {
                        name: ic_syntax::util::decl_name(&member.decl).to_string(),
                        span: ic_syntax::util::decl_span(&member.decl),
                    };

                    // Check if this is a default case
                    let is_default = field.labels.iter().any(|label| {
                        matches!(label, ic_syntax::Label::Default(_))
                    });

                    // Process case labels (will be evaluated later)
                    let labels = Vec::new(); // Labels will be evaluated in the evaluation phase

                    variants.push(Variant {
                        annotations: Vec::new(), // TODO: Convert annotations from field.annotations
                        ident,
                        ty,
                        labels,
                        is_default,
                    });
                }
                ic_syntax::UnionElement::Null(null_elem) => {
                    // Generate a synthetic identifier for the null case based on its position
                    let ident = ic_syntax::Ident {
                        name: format!("_null_case_{}", variants.len()),
                        span: null_elem.span,
                    };

                    // Check if this is a default case
                    let is_default = field.labels.iter().any(|label| {
                        matches!(label, ic_syntax::Label::Default(_))
                    });

                    // Process case labels (will be evaluated later)
                    let labels = Vec::new(); // Labels will be evaluated in the evaluation phase

                    // Use a null type for null cases
                    let null_ty = Ty {
                        span: null_elem.span,
                        kind: TyKind::Null,
                    };

                    variants.push(Variant {
                        annotations: Vec::new(), // TODO: Convert annotations
                        ident,
                        ty: null_ty,
                        labels,
                        is_default,
                    });
                }
            }
        }

        variants
    }
}
