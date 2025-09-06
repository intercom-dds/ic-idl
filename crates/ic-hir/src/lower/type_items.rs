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

use ic_syntax::{AliasDef, ExceptDef, InterfaceDef, StructDef, UnionDef, ValuetypeDef};

use super::LoweringContext;
use super::eval::ConstEvaluator;
use super::registry::DefKindTag;
use super::type_resolver::TypeResolver;
use super::utils::TyExt;
use super::value_items::resolve_declarator;
use crate::hir::{
    AliasTy, Ann, Attribute, Decl, Def, DefFlags, DefId, DefKind, ExceptTy, InterfaceTy, Label,
    Member, Parameter, PrimitiveTy, ProtoTy, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use crate::scope::ScopeId;

/// Processes type items (struct, union, interface, valuetype, native).
pub struct TypeItemProcessor<'ctx> {
    pub(super) ctx: &'ctx mut LoweringContext,
    pub(super) current_scope: ScopeId,
}

impl<'ctx> TypeItemProcessor<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    /// Check if a parent type is valid for inheritance (not a forward declaration).
    /// Returns `Some(parent_id)` if valid, None if invalid (error already reported).
    fn validate_parent_inheritance(
        &mut self,
        parent_id: DefId,
        child_kind: &str,
        child_name: &str,
        inheritance_span: ic_syntax::Span,
    ) -> Option<DefId> {
        let parent_def = self.ctx.context.definitions.get(parent_id);
        if matches!(&parent_def.kind, DefKind::Decl(_)) {
            use ic_diagnostic::{Label, error_span};
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!(
                        "{child_kind} `{child_name}` cannot inherit from incomplete type `{}`",
                        parent_def.ident.name
                    ),
                    Label::new(inheritance_span).message("invalid inheritance"),
                )
                .label(
                    Label::new(parent_def.ident.span)
                        .message("parent type is only forward declared"),
                ),
            );
            None
        } else {
            Some(parent_id)
        }
    }

    /// Process a struct definition.
    pub fn process_struct(&mut self, s: &StructDef) -> DefId {
        let annotations = self.convert_annotations(&s.annotations, self.current_scope);

        // Create a placeholder definition first
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: s.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: (s.ident.span),
            kind: DefKind::Struct(StructTy {
                parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
                members: Vec::new(),
            }), // Placeholder
            flags: DefFlags::nil(),
        });

        // Register in scope immediately so self-references work
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, s.ident.name.clone(), def_id);

        // Register with the registry
        self.ctx.registry.register_definition(
            self.current_scope,
            &s.ident,
            DefKindTag::Struct,
            def_id,
            &mut self.ctx.diagnostics,
            &self.ctx.context,
        );

        // Now resolve parent type if present
        let parent = if let Some(ref parent_type) = s.parent {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver.resolve_path_type(parent_type).and_then(|ty| {
                if let Some(parent_id) = ty.as_adt() {
                    self.validate_parent_inheritance(
                        parent_id,
                        "struct",
                        &s.ident.name,
                        super::utils::path_span(parent_type),
                    )
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

        // Create scope and process members (struct is now in scope)
        let (_scope, members) = self.process_members(&s.members);

        // Update the definition with the actual struct data
        let def = self.ctx.context.definitions.get_mut(def_id);
        if let DefKind::Struct(struct_ty) = &mut def.kind {
            struct_ty.parent = parent;
            struct_ty.members = members;
        }

        def_id
    }

    /// Process an interface definition.
    pub fn process_interface(&mut self, i: &InterfaceDef) -> DefId {
        let mut parents = Vec::new();
        let mut definitions = Vec::new();
        let annotations = self.convert_annotations(&i.annotations, self.current_scope);

        for parent_path in &i.inherits {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            if let Some(ty) = resolver.resolve_path_type(parent_path) {
                if let Some(parent_id) = ty.as_adt() {
                    // Validate that parent is not an incomplete type
                    if let Some(valid_parent_id) = self.validate_parent_inheritance(
                        parent_id,
                        "interface",
                        &i.ident.name,
                        super::utils::path_span(parent_path),
                    ) {
                        parents.push(valid_parent_id);
                    }
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
        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            i.ident.name.clone(),
            None,
        );

        // Process interface members
        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();

        // Save current scope and switch to interface scope
        let prev_scope = self.current_scope;
        self.current_scope = scope;

        for member in &i.members {
            match member {
                ic_syntax::InterfaceMember::Proto(proto) => {
                    prototypes.push(self.process_prototype(proto));
                }
                ic_syntax::InterfaceMember::Attr(attr) => {
                    attributes.extend(self.process_attributes(attr));
                }
                ic_syntax::InterfaceMember::Item(item) => {
                    // Process nested type definition
                    let mut builder = super::builder::HirBuilder::new(self.ctx);
                    builder.current_scope = scope;
                    let item_defs = builder.process_item(item);
                    definitions.extend(item_defs);
                }
            }
        }

        // Restore previous scope
        self.current_scope = prev_scope;

        // Create the complete interface definition
        let interface_ty = InterfaceTy {
            parents,
            prototypes,
            attributes,
            is_local: i.local.is_some(),
            definitions,
        };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: i.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
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
                &self.ctx.context,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                i.ident.name.clone(),
                def_id,
            );
        }

        def_id
    }

    /// Process a union definition.
    pub fn process_union(&mut self, u: &UnionDef) -> DefId {
        let annotations = self.convert_annotations(&u.annotations, self.current_scope);

        // Create a placeholder union declaration first
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: u.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: (u.ident.span),
            kind: DefKind::Decl(Decl::Union),
            flags: DefFlags::nil(),
        });

        // Register in scope immediately so self-references work
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, u.ident.name.clone(), def_id);

        // Register with the registry
        self.ctx.registry.register_definition(
            self.current_scope,
            &u.ident,
            DefKindTag::Union,
            def_id,
            &mut self.ctx.diagnostics,
            &self.ctx.context,
        );

        // Now resolve discriminator type
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let disc = resolver.resolve_type(&u.disc.ty).unwrap_or_else(|| {
            // Use a default type on error
            Ty {
                span: (ic_syntax::util::ty_span(&u.disc.ty)),
                kind: crate::hir::TyKind::Primitive(crate::hir::PrimitiveTy::Int32),
            }
        });

        // Validate that discriminator is an enum, integral type, boolean, or char
        let is_valid_discriminator = match &disc.kind {
            TyKind::Primitive(p) => matches!(
                p,
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
                    | PrimitiveTy::UInt64
            ),
            TyKind::Adt(def_id) => {
                let def = self.ctx.context.definitions.get(*def_id);
                matches!(&def.kind, DefKind::Enum(_))
            }
            _ => false,
        };

        if !is_valid_discriminator {
            self.ctx.diagnostics.error(
                format!("invalid discriminator type for union `{}`", u.ident.name),
                ic_diagnostic::Label::new(ic_syntax::util::ty_span(&u.disc.ty))
                    .message("discriminator must be an enum or integral type"),
            );
        }

        // Create scope for union members
        let _scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            u.ident.name.clone(),
            Some(def_id),
        );

        // Process union variants (union is now in scope)
        let variants = self.process_union_variants(&u.fields, &disc);

        // Update the definition with the actual union data
        let def = self.ctx.context.definitions.get_mut(def_id);
        def.kind = DefKind::Union(UnionTy { disc, variants });

        def_id
    }

    /// Process a valuetype definition.
    pub fn process_valuetype(&mut self, v: &ValuetypeDef) -> DefId {
        // Resolve parent and supports
        let parent = self.resolve_valuetype_parent(v);
        let supports = self.resolve_valuetype_supports(v);
        let annotations = self.convert_annotations(&v.annotations, self.current_scope);

        // Create scope for valuetype members
        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            v.ident.name.clone(),
            None,
        );

        // Process valuetype elements
        let (members, prototypes, attributes, definitions) =
            self.process_valuetype_elements(v, scope);

        // Create the valuetype definition
        self.create_valuetype_definition(
            v,
            parent,
            supports,
            annotations,
            (members, prototypes, attributes),
            definitions,
        )
    }

    /// Resolve valuetype parent.
    fn resolve_valuetype_parent(&mut self, v: &ValuetypeDef) -> Option<DefId> {
        let parent_type = v.inherits.as_ref()?;

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        resolver
            .resolve_path_type(parent_type)
            .and_then(|ty| ty.as_adt())
            .and_then(|parent_id| {
                self.validate_parent_inheritance(
                    parent_id,
                    "valuetype",
                    &v.ident.name,
                    super::utils::path_span(parent_type),
                )
            })
    }

    /// Resolve valuetype supports interface.
    fn resolve_valuetype_supports(&mut self, v: &ValuetypeDef) -> Option<DefId> {
        let supports_type = v.supports.as_ref()?;

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        resolver.resolve_path_type(supports_type).and_then(|ty| {
            if let Some(supports_id) = ty.as_adt() {
                // Verify it's an interface type
                let def = self.ctx.context.definitions.get(supports_id);
                if matches!(&def.kind, DefKind::Interface(_)) {
                    Some(supports_id)
                } else {
                    self.ctx.diagnostics.error(
                        "supports must be an interface type".to_string(),
                        ic_diagnostic::Label::new(super::utils::path_span(supports_type))
                            .message("expected interface type"),
                    );
                    None
                }
            } else {
                None
            }
        })
    }

    /// Process valuetype elements.
    fn process_valuetype_elements(
        &mut self,
        v: &ValuetypeDef,
        scope: ScopeId,
    ) -> (Vec<Member>, Vec<ProtoTy>, Vec<Attribute>, Vec<DefId>) {
        let mut members = Vec::new();
        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();
        let mut definitions = Vec::new();

        // Save current scope and switch to valuetype scope
        let prev_scope = self.current_scope;
        self.current_scope = scope;

        for element in &v.elements {
            match element {
                ic_syntax::ValueElement::State(member) => {
                    members.extend(self.process_value_members(member));
                }
                ic_syntax::ValueElement::Proto(proto) => {
                    prototypes.push(self.process_prototype(proto));
                }
                ic_syntax::ValueElement::Attr(attr) => {
                    attributes.extend(self.process_attributes(attr));
                }
                ic_syntax::ValueElement::Item(item) => {
                    let mut builder = super::builder::HirBuilder::new(self.ctx);
                    builder.current_scope = scope;
                    let item_defs = builder.process_item(item);
                    definitions.extend(item_defs);
                }
            }
        }

        // Restore previous scope
        self.current_scope = prev_scope;

        (members, prototypes, attributes, definitions)
    }

    /// Create valuetype definition.
    fn create_valuetype_definition(
        &mut self,
        v: &ValuetypeDef,
        parent: Option<DefId>,
        supports: Option<DefId>,
        annotations: Vec<Ann>,
        elements: (Vec<Member>, Vec<ProtoTy>, Vec<Attribute>),
        definitions: Vec<DefId>,
    ) -> DefId {
        let (members, prototypes, attributes) = elements;
        let value_ty = ValueTy {
            parent,
            supports,
            prototypes,
            attributes,
            members,
            definitions,
        };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: v.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
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
                &self.ctx.context,
            )
            .is_some()
        {
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                v.ident.name.clone(),
                def_id,
            );
        }

        def_id
    }

    /// Process a forward declaration.
    pub fn process_forward_decl(&mut self, decl: &ic_syntax::Decl) -> DefId {
        let hir_decl_kind = match decl.kind {
            ic_syntax::DeclKind::Struct => Decl::Struct,
            ic_syntax::DeclKind::Union => Decl::Union,
            ic_syntax::DeclKind::Interface => Decl::Interface,
            ic_syntax::DeclKind::Valuetype => Decl::Valuetype,
            ic_syntax::DeclKind::Native => Decl::Native,
        };

        self.create_forward_declaration(&decl.ident, hir_decl_kind)
    }

    /// Create a forward declaration.
    fn create_forward_declaration(&mut self, ident: &ic_syntax::Ident, kind: Decl) -> DefId {
        // Allocate the forward declaration definition
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
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
            &self.ctx.context,
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
        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            "_members_".to_string(),
            None,
        );
        let mut members = Vec::new();

        for field in fields {
            // Resolve the type once for this field
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(ty) = resolver.resolve_type(&field.ty) else {
                continue; // Error already reported
            };

            // Convert field annotations
            let annotations = self.convert_annotations(&field.annotations, self.current_scope);

            // Process each declarator in the field
            for decl in &field.names {
                // Use resolve_declarator to handle array types properly
                let (ident, member_ty) =
                    resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

                members.push(Member {
                    ident,
                    ty: member_ty,
                    annotations: annotations.clone(),
                });
            }
        }

        (scope, members)
    }

    /// Process union variants.
    fn process_union_variants(
        &mut self,
        fields: &[ic_syntax::UnionField],
        disc: &Ty,
    ) -> Vec<Variant> {
        let mut variants = Vec::new();

        for field in fields {
            let annotations = self.convert_annotations(&field.annotations, self.current_scope);
            match &field.field {
                ic_syntax::UnionElement::Member(member) => {
                    // Resolve the type for this variant
                    let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                    let Some(ty) = resolver.resolve_type(&member.ty) else {
                        continue; // Error already reported
                    };

                    // Use resolve_declarator to handle array types properly
                    let (ident, variant_ty) =
                        resolve_declarator(&member.decl, ty, self.ctx, self.current_scope);

                    // Check if this is a default case
                    let is_default = field
                        .labels
                        .iter()
                        .any(|label| matches!(label, ic_syntax::Label::Default(_)));

                    // Process and evaluate case labels
                    let mut labels = Vec::new();
                    for label in &field.labels {
                        if let ic_syntax::Label::Case(expr) = label {
                            // Create an evaluator and evaluate the expression
                            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
                            if let Some(numeric) = evaluator.eval_union_case_label(expr, disc) {
                                labels.push(Label {
                                    value: numeric,
                                    span: expr.span(),
                                });
                            }
                            // If evaluation fails, error was already reported
                        }
                    }

                    variants.push(Variant {
                        annotations,
                        ident,
                        ty: variant_ty,
                        labels,
                        is_default,
                    });
                }
                ic_syntax::UnionElement::Null(null_elem) => {
                    let ident = ic_syntax::Ident {
                        name: format!("__null_case_{}", variants.len()),
                        span: null_elem.span,
                    };

                    let is_default = field
                        .labels
                        .iter()
                        .any(|label| matches!(label, ic_syntax::Label::Default(_)));

                    // Process and evaluate case labels
                    let mut labels = Vec::new();
                    for label in &field.labels {
                        if let ic_syntax::Label::Case(expr) = label {
                            // Create an evaluator and evaluate the expression
                            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
                            if let Some(numeric) = evaluator.eval_union_case_label(expr, disc) {
                                labels.push(Label {
                                    value: numeric,
                                    span: expr.span(),
                                });
                            }
                            // If evaluation fails, error was already reported
                        }
                    }

                    // Use a null type for null cases
                    let null_ty = Ty {
                        span: null_elem.span,
                        kind: TyKind::Null,
                    };

                    variants.push(Variant {
                        annotations: annotations.clone(),
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

    /// Process a prototype (interface method).
    fn process_prototype(&mut self, proto: &ic_syntax::Prototype) -> ProtoTy {
        // Resolve return type
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let ty = resolver.resolve_type(&proto.ret).unwrap_or_else(|| {
            // Default to void on error
            Ty {
                span: ic_syntax::util::ty_span(&proto.ret),
                kind: TyKind::Null,
            }
        });

        // Process parameters
        let params = proto
            .params
            .iter()
            .map(|param| {
                let ident = ic_syntax::Ident {
                    name: ic_syntax::util::decl_name(&param.decl).to_string(),
                    span: ic_syntax::util::decl_span(&param.decl),
                };

                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                let param_ty = resolver.resolve_type(&param.ty).unwrap_or_else(|| {
                    // Default type on error
                    Ty {
                        span: ic_syntax::util::ty_span(&param.ty),
                        kind: TyKind::Primitive(PrimitiveTy::Int32),
                    }
                });

                Parameter {
                    ident,
                    ty: param_ty,
                    kind: param.kind.unwrap_or(ic_syntax::ParamKind::In),
                }
            })
            .collect();

        ProtoTy {
            ident: proto.ident.clone(),
            ty,
            params,
        }
    }

    /// Process attributes (can have multiple declarators).
    fn process_attributes(&mut self, attr: &ic_syntax::Attribute) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        // Resolve the attribute type
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let Some(ty) = resolver.resolve_type(&attr.ty) else {
            return attributes; // Error already reported
        };

        // Process raises clauses (for exceptions)
        let getraises = self.resolve_exception_paths(&attr.getraises);
        let setraises = self.resolve_exception_paths(&attr.setraises);

        // Process each declarator
        for decl in &attr.decl {
            // Use resolve_declarator to handle array types properly
            let (ident, attr_ty) =
                resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

            attributes.push(Attribute {
                ident,
                ty: attr_ty,
                is_readonly: attr.readonly.is_some(),
                getraises: getraises.clone(),
                setraises: setraises.clone(),
            });
        }

        attributes
    }

    /// Resolve exception paths to `DefIds`.
    fn resolve_exception_paths(&mut self, paths: &[ic_syntax::Path]) -> Vec<DefId> {
        paths
            .iter()
            .filter_map(|path| {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                resolver.resolve_path_type(path).and_then(|ty| {
                    if let Some(def_id) = ty.as_adt() {
                        // Verify it's an exception type
                        let def = self.ctx.context.definitions.get(def_id);
                        if matches!(&def.kind, DefKind::Except(_)) {
                            Some(def_id)
                        } else {
                            self.ctx.diagnostics.error(
                                format!("'{}' is not an exception type", def.ident.name),
                                ic_diagnostic::Label::new(super::utils::path_span(path))
                                    .message("not an exception"),
                            );
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Process a type alias definition.
    pub fn process_alias(&mut self, a: &AliasDef) -> Vec<DefId> {
        let mut def_ids = Vec::new();
        let annotations = self.convert_annotations(&a.annotations, self.current_scope);

        // Type aliases can have multiple declarators, process each one
        for decl in &a.decl {
            let ident = ic_syntax::Ident {
                name: ic_syntax::util::decl_name(decl).to_string(),
                span: ic_syntax::util::decl_span(decl),
            };

            // Resolve the aliased type
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(ty) = resolver.resolve_type(&a.ty) else {
                continue; // Error already reported
            };

            // Create the alias definition
            let alias_ty = AliasTy { ty };

            let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
                id,
                ident: ident.clone(),
                parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
                annotations: annotations.clone(),
                span: ic_syntax::util::decl_span(decl),
                kind: DefKind::Alias(alias_ty),
                flags: DefFlags::nil(),
            });

            // Type aliases don't use the registry (they're not forward-declarable)
            // Just register in the scope
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, ident.name.clone(), def_id);

            def_ids.push(def_id);
        }

        def_ids
    }

    /// Process an exception definition.
    pub fn process_exception(&mut self, e: &ExceptDef) -> DefId {
        let annotations = self.convert_annotations(&e.annotations, self.current_scope);

        // Process exception members (similar to struct members)
        let (_scope, members) = self.process_members(&e.members);

        // Create the exception definition
        let except_ty = ExceptTy { members };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: e.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: e.ident.span,
            kind: DefKind::Except(except_ty),
            flags: DefFlags::nil(),
        });

        // Exceptions are not forward-declarable, just register in the scope
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, e.ident.name.clone(), def_id);

        def_id
    }

    /// Process valuetype state members.
    fn process_value_members(&mut self, members: &ic_syntax::ValueMember) -> Vec<Member> {
        let mut result = Vec::new();

        // Process visibility (public/private)
        // Note: visibility is not stored in HIR, but could be added if needed
        let _ = members.is_public;

        // Resolve the type
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let Some(ty) = resolver.resolve_type(&members.ty) else {
            return result; // Error already reported
        };

        // Process each declarator
        for decl in &members.decl {
            // Use resolve_declarator to handle array types properly
            let (ident, member_ty) =
                resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

            result.push(Member {
                ident,
                ty: member_ty,
                annotations: Vec::new(), // TODO: Convert annotations
            });
        }

        result
    }
}
