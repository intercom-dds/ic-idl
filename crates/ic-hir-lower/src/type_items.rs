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

use ic_hir::hir::{
    AliasTy, Attribute, Decl, Def, DefFlags, DefId, DefKind, ExceptTy, InterfaceTy, Label, Member,
    Parameter, PrimitiveTy, ProtoTy, Spanned, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use ic_hir::scope::ScopeId;
use ic_syntax::{AliasDef, ExceptDef, InterfaceDef, StructDef, UnionDef, ValuetypeDef};

use crate::LoweringContext;
use crate::annotation::convert_annotations;
use crate::eval::ConstEvaluator;
use crate::registry::DefKindTag;
use crate::type_resolver::TypeResolver;
use crate::utils::TyExt;
use crate::value_items::resolve_declarator;

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
    /// If valid, marks the parent as having children.
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
            // Mark parent as having children
            let parent_def = self.ctx.context.definitions.get_mut(parent_id);
            parent_def.flags |= DefFlags::HAS_CHILDREN;
            Some(parent_id)
        }
    }

    /// Process a struct definition.
    pub fn process_struct(&mut self, s: &StructDef) -> DefId {
        let annotations = convert_annotations(self.ctx, &s.annotations, self.current_scope);

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: s.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: s.span,
            kind: DefKind::Struct(StructTy {
                parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
                members: Vec::new(),
            }),
            flags: DefFlags::nil(),
        });

        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, s.ident.name.clone(), def_id);

        _ = self.ctx.registry.register_definition(
            self.current_scope,
            &s.ident,
            DefKindTag::Struct,
            def_id,
            &mut self.ctx.diagnostics,
            &self.ctx.context,
        );

        let parent = if let Some(ref parent_type) = s.parent {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver.resolve_path_type(parent_type).and_then(|ty| {
                if let Some(parent_id) = ty.as_adt() {
                    self.validate_parent_inheritance(
                        parent_id,
                        "struct",
                        &s.ident.name,
                        crate::utils::path_span(parent_type),
                    )
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be a struct type".to_string(),
                        ic_diagnostic::Label::new(crate::utils::path_span(parent_type))
                            .message("expected struct type"),
                    );
                    None
                }
            })
        } else {
            None
        };

        let members = self.process_members(&s.members);

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
        let annotations = convert_annotations(self.ctx, &i.annotations, self.current_scope);

        for parent_path in &i.inherits {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            if let Some(ty) = resolver.resolve_path_type(parent_path) {
                if let Some(parent_id) = ty.as_adt() {
                    if let Some(valid_parent_id) = self.validate_parent_inheritance(
                        parent_id,
                        "interface",
                        &i.ident.name,
                        crate::utils::path_span(parent_path),
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

        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            i.ident.name.clone(),
            None,
        );

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: i.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: i.span,
            kind: DefKind::Interface(InterfaceTy {
                parents,
                prototypes: Vec::new(),
                attributes: Vec::new(),
                is_local: i.local.is_some(),
                definitions: Vec::new(),
            }),
            flags: DefFlags::nil(),
        });

        self.ctx.context.scopes.set_scope_def_id(scope, def_id);

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
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                i.ident.name.clone(),
                def_id,
            );
        }

        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();
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
                    let mut builder = crate::builder::HirBuilder::new(self.ctx);
                    builder.current_scope = scope;
                    let item_defs = builder.process_item(item);
                    definitions.extend(item_defs);
                }
            }
        }

        self.current_scope = prev_scope;

        let interface_def = self.ctx.context.definitions.get_mut(def_id);
        if let DefKind::Interface(ref mut interface_ty) = interface_def.kind {
            interface_ty.prototypes = prototypes;
            interface_ty.attributes = attributes;
            interface_ty.definitions = definitions;
        }

        def_id
    }

    /// Process a union definition.
    pub fn process_union(&mut self, u: &UnionDef) -> DefId {
        let annotations = convert_annotations(self.ctx, &u.annotations, self.current_scope);

        // Create as a forward declaration first so self-references work
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: u.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: u.span,
            kind: DefKind::Decl(Decl::Union),
            flags: DefFlags::nil(),
        });

        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, u.ident.name.clone(), def_id);

        self.ctx.registry.register_definition(
            self.current_scope,
            &u.ident,
            DefKindTag::Union,
            def_id,
            &mut self.ctx.diagnostics,
            &self.ctx.context,
        );

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let disc_ty = resolver.resolve_type(&u.disc.ty).unwrap_or_else(|| Ty {
            span: (ic_syntax::util::ty_span(&u.disc.ty)),
            kind: ic_hir::hir::TyKind::Primitive(ic_hir::hir::PrimitiveTy::Int32),
        });

        let disc_annotations =
            convert_annotations(self.ctx, &u.disc.annotations, self.current_scope);

        // Validate discriminator is an enum, integral type, boolean, or char
        let resolved_disc_ty = self.ctx.context.resolve_ty(&disc_ty);
        let is_valid_discriminator = match &resolved_disc_ty.kind {
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

        let _scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            u.ident.name.clone(),
            Some(def_id),
        );

        let variants = self.process_union_variants(&u.fields, &disc_ty);

        let disc = ic_hir::hir::Disc {
            annotations: disc_annotations,
            ty: disc_ty,
        };

        let def = self.ctx.context.definitions.get_mut(def_id);
        def.kind = DefKind::Union(UnionTy { disc, variants });

        def_id
    }

    /// Process a valuetype definition.
    pub fn process_valuetype(&mut self, v: &ValuetypeDef) -> DefId {
        let parent = self.resolve_valuetype_parent(v);
        let supports = self.resolve_valuetype_supports(v);
        let annotations = convert_annotations(self.ctx, &v.annotations, self.current_scope);

        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            v.ident.name.clone(),
            None,
        );

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: v.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: v.span,
            kind: DefKind::Valuetype(ValueTy {
                parent,
                supports,
                prototypes: Vec::new(),
                attributes: Vec::new(),
                members: Vec::new(),
                definitions: Vec::new(),
            }),
            flags: DefFlags::nil(),
        });

        self.ctx.context.scopes.set_scope_def_id(scope, def_id);

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

        let (members, prototypes, attributes, definitions) =
            self.process_valuetype_elements(v, scope);

        let valuetype_def = self.ctx.context.definitions.get_mut(def_id);
        if let DefKind::Valuetype(ref mut value_ty) = valuetype_def.kind {
            value_ty.prototypes = prototypes;
            value_ty.attributes = attributes;
            value_ty.members = members;
            value_ty.definitions = definitions;
        }

        def_id
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
                    crate::utils::path_span(parent_type),
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
                    // Mark the supported interface as having children
                    let def = self.ctx.context.definitions.get_mut(supports_id);
                    def.flags |= DefFlags::HAS_CHILDREN;
                    Some(supports_id)
                } else {
                    self.ctx.diagnostics.error(
                        "supports must be an interface type".to_string(),
                        ic_diagnostic::Label::new(crate::utils::path_span(supports_type))
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
                    let mut builder = crate::builder::HirBuilder::new(self.ctx);
                    builder.current_scope = scope;
                    let item_defs = builder.process_item(item);
                    definitions.extend(item_defs);
                }
            }
        }

        self.current_scope = prev_scope;
        (members, prototypes, attributes, definitions)
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

    fn create_forward_declaration(&mut self, ident: &ic_syntax::Ident, kind: Decl) -> DefId {
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations: Vec::new(),
            span: (ident.span),
            kind: DefKind::Decl(kind),
            flags: DefFlags::IS_INCOMPLETE,
        });

        if let Some(existing_id) = self.ctx.registry.register_forward_decl(
            self.current_scope,
            ident,
            kind,
            def_id,
            &mut self.ctx.diagnostics,
            &self.ctx.context,
        ) && existing_id != def_id
        {
            return existing_id;
        }

        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, ident.name.clone(), def_id);

        def_id
    }

    /// Process members.
    fn process_members(&mut self, fields: &[ic_syntax::Field]) -> Vec<Member> {
        let mut members = Vec::new();

        for field in fields {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(ty) = resolver.resolve_type(&field.ty) else {
                continue;
            };

            let annotations = convert_annotations(self.ctx, &field.annotations, self.current_scope);

            for decl in &field.names {
                let (ident, member_ty) =
                    resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

                members.push(Member {
                    ident,
                    ty: member_ty,
                    annotations: annotations.clone(),
                });
            }
        }

        members
    }

    /// Process union variants.
    fn process_union_variants(
        &mut self,
        fields: &[ic_syntax::UnionField],
        disc: &Ty,
    ) -> Vec<Variant> {
        let mut variants = Vec::new();

        for field in fields {
            let annotations = convert_annotations(self.ctx, &field.annotations, self.current_scope);
            match &field.field {
                ic_syntax::UnionElement::Member(member) => {
                    let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                    let Some(ty) = resolver.resolve_type(&member.ty) else {
                        continue;
                    };

                    let (ident, variant_ty) =
                        resolve_declarator(&member.decl, ty, self.ctx, self.current_scope);

                    let is_default = field
                        .labels
                        .iter()
                        .any(|label| matches!(label, ic_syntax::Label::Default(_)));

                    let mut labels = Vec::new();
                    for label in &field.labels {
                        if let ic_syntax::Label::Case(expr) = label {
                            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
                            if let Some(numeric) = evaluator.eval_union_case_label(expr, disc) {
                                labels.push(Label {
                                    value: numeric,
                                    span: expr.span(),
                                });
                            }
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

                    let mut labels = Vec::new();
                    for label in &field.labels {
                        if let ic_syntax::Label::Case(expr) = label {
                            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
                            if let Some(numeric) = evaluator.eval_union_case_label(expr, disc) {
                                labels.push(Label {
                                    value: numeric,
                                    span: expr.span(),
                                });
                            }
                        }
                    }

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

    fn process_prototype(&mut self, proto: &ic_syntax::Prototype) -> ProtoTy {
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let ty = resolver.resolve_type(&proto.ret).unwrap_or_else(|| Ty {
            span: ic_syntax::util::ty_span(&proto.ret),
            kind: TyKind::Null,
        });

        let params = proto
            .params
            .iter()
            .map(|param| {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                let base_ty = resolver.resolve_type(&param.ty).unwrap_or_else(|| Ty {
                    span: ic_syntax::util::ty_span(&param.ty),
                    kind: TyKind::Primitive(PrimitiveTy::Int32),
                });

                let (ident, param_ty) =
                    resolve_declarator(&param.decl, base_ty, self.ctx, self.current_scope);

                Parameter {
                    ident,
                    ty: param_ty,
                    kind: param.kind.unwrap_or(ic_syntax::ParamKind::In),
                }
            })
            .collect();

        let raises = self.resolve_exception_paths(&proto.raises);

        ProtoTy {
            ident: proto.ident.clone(),
            ty,
            params,
            raises,
        }
    }

    fn process_attributes(&mut self, attr: &ic_syntax::Attribute) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let Some(ty) = resolver.resolve_type(&attr.ty) else {
            return attributes;
        };

        let getraises = self.resolve_exception_paths(&attr.getraises);
        let setraises = self.resolve_exception_paths(&attr.setraises);

        for decl in &attr.decl {
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

    fn resolve_exception_paths(&mut self, paths: &[ic_syntax::Path]) -> Vec<Spanned<DefId>> {
        paths
            .iter()
            .filter_map(|path| {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                resolver.resolve_path_type(path).and_then(|ty| {
                    if let Some(def_id) = ty.as_adt() {
                        let def = self.ctx.context.definitions.get(def_id);
                        if matches!(&def.kind, DefKind::Except(_)) {
                            Some(Spanned {
                                value: def_id,
                                span: crate::utils::path_span(path),
                            })
                        } else {
                            self.ctx.diagnostics.error(
                                format!("'{}' is not an exception type", def.ident.name),
                                ic_diagnostic::Label::new(crate::utils::path_span(path))
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

    pub fn process_alias(&mut self, a: &AliasDef) -> Vec<DefId> {
        let mut def_ids = Vec::new();
        let annotations = convert_annotations(self.ctx, &a.annotations, self.current_scope);

        for decl in &a.decl {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(base_ty) = resolver.resolve_type(&a.ty) else {
                continue;
            };

            let (ident, ty) = resolve_declarator(decl, base_ty, self.ctx, self.current_scope);
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

            if self
                .ctx
                .registry
                .register_definition(
                    self.current_scope,
                    &ident,
                    DefKindTag::Alias,
                    def_id,
                    &mut self.ctx.diagnostics,
                    &self.ctx.context,
                )
                .is_some()
            {
                self.ctx.context.scopes.add_definition(
                    self.current_scope,
                    ident.name.clone(),
                    def_id,
                );

                def_ids.push(def_id);
            }
        }

        def_ids
    }

    pub fn process_exception(&mut self, e: &ExceptDef) -> DefId {
        let annotations = convert_annotations(self.ctx, &e.annotations, self.current_scope);
        let members = self.process_members(&e.members);
        let except_ty = ExceptTy { members };

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: e.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: e.span,
            kind: DefKind::Except(except_ty),
            flags: DefFlags::nil(),
        });

        // Register in scope so self-references work
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, e.ident.name.clone(), def_id);

        def_id
    }

    fn process_value_members(&mut self, members: &ic_syntax::ValueMember) -> Vec<Member> {
        let mut result = Vec::new();

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let Some(ty) = resolver.resolve_type(&members.ty) else {
            return result;
        };

        for decl in &members.decl {
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
