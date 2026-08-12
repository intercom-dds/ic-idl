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
    AliasTy, Attribute, Decl, DefFlags, DefId, DefKind, ExceptTy, InterfaceTy, Label, Member,
    Parameter, PrimitiveTy, ProtoTy, Spanned, StructTy, Ty, TyKind, UnionTy, ValueTy, Variant,
};
use ic_hir::scope::ScopeId;
use ic_syntax::{AliasDef, ExceptDef, InterfaceDef, StructDef, UnionDef, ValuetypeDef};

use crate::annotation::convert_annotations;
use crate::eval::ConstEvaluator;
use crate::registry::DefKindTag;
use crate::resolve::{TypeResolver, resolve_declarator};
use crate::utils::TyExt;
use crate::{LoweringContext, define};

/// Processes type items (struct, union, interface, valuetype, native).
pub struct TypeItemProcessor<'ctx> {
    pub(super) ctx: &'ctx mut LoweringContext,
    pub(super) current_scope: ScopeId,
}

/// Validates that `parent_id` is complete, not a forward declaration, for
/// an inheritance-like relationship, and marks it `HAS_CHILDREN` on
/// success. `relationship` is a verb phrase for the diagnostic message,
/// e.g. `"inherit from"` or `"support"`.
pub(super) fn validate_parent_inheritance(
    ctx: &mut LoweringContext,
    parent_id: DefId,
    child_kind: &str,
    child_name: &str,
    relationship: &str,
    inheritance_span: ic_syntax::Span,
) -> Option<DefId> {
    let parent_def = ctx.context.definitions.get(parent_id);
    if matches!(&parent_def.kind, DefKind::Decl(_)) {
        use ic_diagnostic::{Label, error_span};
        ctx.diagnostics.errors.push(
            error_span(
                format!(
                    "{child_kind} `{child_name}` cannot {relationship} incomplete type `{}`",
                    parent_def.ident.name
                ),
                Label::new(inheritance_span).message("invalid inheritance"),
            )
            .label(
                Label::new(parent_def.ident.span).message("parent type is only forward declared"),
            ),
        );
        None
    } else {
        // Mark parent as having children
        let parent_def = ctx.context.definitions.get_mut(parent_id);
        parent_def.flags |= DefFlags::HAS_CHILDREN;
        Some(parent_id)
    }
}

impl<'ctx> TypeItemProcessor<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    /// Process a struct definition.
    pub fn process_struct(&mut self, s: &StructDef) -> DefId {
        let def_id = define::define(
            self.ctx,
            self.current_scope,
            &s.name,
            s.meta.span,
            &s.meta.annotations,
            DefKindTag::Struct,
            |_| {
                DefKind::Struct(StructTy {
                    parent: None,
                    members: Vec::new(),
                })
            },
        );

        let parent = if let Some(ref parent_type) = s.parent {
            let path_span = crate::utils::path_span(parent_type);
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver.resolve_path_type(parent_type).and_then(|ty| {
                if let Some(parent_id) = ty.as_adt() {
                    validate_parent_inheritance(
                        self.ctx,
                        parent_id,
                        "struct",
                        &s.name.name,
                        "inherit from",
                        path_span,
                    )
                    .map(|value| Spanned {
                        def_id: value,
                        span: path_span,
                    })
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be a struct type".to_string(),
                        ic_diagnostic::Label::new(path_span).message("expected struct type"),
                    );
                    None
                }
            })
        } else {
            None
        };

        let members = self.process_members(&s.fields);
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

        for parent_path in &i.inherits {
            let path_span = crate::utils::path_span(parent_path);
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            if let Some(ty) = resolver.resolve_path_type(parent_path) {
                if let Some(parent_id) = ty.as_adt() {
                    if let Some(value) = validate_parent_inheritance(
                        self.ctx,
                        parent_id,
                        "interface",
                        &i.name.name,
                        "inherit from",
                        path_span,
                    ) {
                        parents.push(Spanned {
                            def_id: value,
                            span: path_span,
                        });
                    }
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be an interface type".to_string(),
                        ic_diagnostic::Label::new(path_span).message("expected interface type"),
                    );
                }
            }
        }

        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            i.name.name.clone(),
            None,
        );

        let def_id = define::define(
            self.ctx,
            self.current_scope,
            &i.name,
            i.meta.span,
            &i.meta.annotations,
            DefKindTag::Interface,
            |_| {
                DefKind::Interface(InterfaceTy {
                    parents,
                    prototypes: Vec::new(),
                    attributes: Vec::new(),
                    is_local: matches!(i.kind, ic_syntax::InterfaceKind::Local(_)),
                    definitions: Vec::new(),
                })
            },
        );

        self.ctx.context.scopes.set_scope_def_id(scope, def_id);

        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();
        let prev_scope = self.current_scope;
        self.current_scope = scope;

        for member in &i.members {
            match member {
                ic_syntax::InterfaceMember::Proto(proto) => {
                    prototypes.push(self.process_prototype(proto));
                }
                ic_syntax::InterfaceMember::Attribute(attr) => {
                    attributes.extend(self.process_attributes(attr));
                }
                ic_syntax::InterfaceMember::Item(item) => {
                    definitions.extend(crate::builder::process_nested_item(self.ctx, scope, item));
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
        // Create as a forward declaration first so self-references work
        let def_id = define::define(
            self.ctx,
            self.current_scope,
            &u.name,
            u.meta.span,
            &u.meta.annotations,
            DefKindTag::Union,
            |_| DefKind::Decl(Decl::Union),
        );

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let disc_ty = resolver.resolve_type(&u.disc.ty).unwrap_or_else(|| Ty {
            span: (ic_syntax::util::ty_span(&u.disc.ty)),
            kind: ic_hir::hir::TyKind::Primitive(ic_hir::hir::PrimitiveTy::Int32),
        });

        let disc_annotations =
            convert_annotations(self.ctx, &u.disc.meta.annotations, self.current_scope);

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
                format!("invalid discriminator type for union `{}`", u.name.name),
                ic_diagnostic::Label::new(ic_syntax::util::ty_span(&u.disc.ty))
                    .message("discriminator must be an enum or integral type"),
            );
        }

        let _scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            u.name.name.clone(),
            Some(def_id),
        );

        let variants = self.process_union_variants(&u.cases, &disc_ty);

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

        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            v.name.name.clone(),
            None,
        );

        let def_id = define::define(
            self.ctx,
            self.current_scope,
            &v.name,
            v.meta.span,
            &v.meta.annotations,
            DefKindTag::Valuetype,
            |_| {
                DefKind::Valuetype(ValueTy {
                    parent,
                    supports,
                    prototypes: Vec::new(),
                    attributes: Vec::new(),
                    members: Vec::new(),
                    definitions: Vec::new(),
                })
            },
        );

        self.ctx.context.scopes.set_scope_def_id(scope, def_id);

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
    fn resolve_valuetype_parent(&mut self, v: &ValuetypeDef) -> Option<Spanned<DefId>> {
        let parent_type = v.inherits.as_ref()?;
        let path_span = crate::utils::path_span(parent_type);

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        resolver
            .resolve_path_type(parent_type)
            .and_then(|ty| ty.as_adt())
            .and_then(|parent_id| {
                validate_parent_inheritance(
                    self.ctx,
                    parent_id,
                    "valuetype",
                    &v.name.name,
                    "inherit from",
                    path_span,
                )
            })
            .map(|value| Spanned {
                def_id: value,
                span: path_span,
            })
    }

    /// Resolve valuetype supports interface.
    fn resolve_valuetype_supports(&mut self, v: &ValuetypeDef) -> Option<Spanned<DefId>> {
        let supports_type = v.supports.first()?;
        let path_span = crate::utils::path_span(supports_type);

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        resolver.resolve_path_type(supports_type).and_then(|ty| {
            if let Some(supports_id) = ty.as_adt() {
                validate_parent_inheritance(
                    self.ctx,
                    supports_id,
                    "valuetype",
                    &v.name.name,
                    "support",
                    path_span,
                )
                .map(|value| Spanned {
                    def_id: value,
                    span: path_span,
                })
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

        for element in &v.members {
            match element {
                ic_syntax::ValueMember::State(member) => {
                    members.extend(self.process_value_members(member));
                }
                ic_syntax::ValueMember::Proto(proto) => {
                    prototypes.push(self.process_prototype(proto));
                }
                ic_syntax::ValueMember::Attribute(attr) => {
                    attributes.extend(self.process_attributes(attr));
                }
                ic_syntax::ValueMember::Item(item) => {
                    definitions.extend(crate::builder::process_nested_item(self.ctx, scope, item));
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

        self.create_forward_declaration(&decl.name, hir_decl_kind)
    }

    fn create_forward_declaration(&mut self, ident: &ic_syntax::Ident, kind: Decl) -> DefId {
        define::declare_forward(self.ctx, self.current_scope, ident, kind)
    }

    /// Process members.
    fn process_members(&mut self, fields: &[ic_syntax::Field]) -> Vec<Member> {
        let mut members = Vec::new();

        for field in fields {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(ty) = resolver.resolve_type(&field.ty) else {
                continue;
            };

            let annotations =
                convert_annotations(self.ctx, &field.meta.annotations, self.current_scope);

            for decl in &field.declarators {
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
        fields: &[ic_syntax::UnionCase],
        disc: &Ty,
    ) -> Vec<Variant> {
        let mut variants = vec![];

        for field in fields {
            let annotations =
                convert_annotations(self.ctx, &field.meta.annotations, self.current_scope);

            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(ty) = resolver.resolve_type(&field.ty) else {
                continue;
            };

            let (ident, variant_ty) =
                resolve_declarator(&field.declarator, ty, self.ctx, self.current_scope);

            let is_default = field
                .labels
                .iter()
                .any(|label| matches!(label, ic_syntax::Label::Default(_)));

            let mut labels = vec![];
            for label in &field.labels {
                if let ic_syntax::Label::Value(expr) = label {
                    let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
                    if let Some(numeric) = evaluator.eval_union_case_label(expr, disc) {
                        labels.push(Label {
                            value: numeric,
                            span: expr.span,
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

        variants
    }

    fn process_prototype(&mut self, proto: &ic_syntax::Proto) -> ProtoTy {
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let ty = resolver
            .resolve_type(&proto.return_type)
            .unwrap_or_else(|| Ty {
                span: ic_syntax::util::ty_span(&proto.return_type),
                kind: TyKind::Null,
            });

        let params = proto
            .parameters
            .iter()
            .map(|param| {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                let base_ty = resolver.resolve_type(&param.ty).unwrap_or_else(|| Ty {
                    span: ic_syntax::util::ty_span(&param.ty),
                    kind: TyKind::Primitive(PrimitiveTy::Int32),
                });

                let (ident, param_ty) =
                    resolve_declarator(&param.declarator, base_ty, self.ctx, self.current_scope);

                Parameter {
                    ident,
                    ty: param_ty,
                    kind: param.kind.unwrap_or(ic_syntax::ParamKind::In),
                }
            })
            .collect();

        let raises = self.resolve_exception_paths(&proto.raises);

        ProtoTy {
            ident: proto.name.clone(),
            ty,
            params,
            raises,
            annotations: convert_annotations(self.ctx, &proto.meta.annotations, self.current_scope),
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

        for decl in &attr.declarators {
            let (ident, attr_ty) =
                resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

            attributes.push(Attribute {
                ident,
                ty: attr_ty,
                is_readonly: attr.readonly.is_some(),
                getraises: getraises.clone(),
                setraises: setraises.clone(),
                annotations: convert_annotations(
                    self.ctx,
                    &attr.meta.annotations,
                    self.current_scope,
                ),
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
                                def_id,
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

        for decl in &a.declarators {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            let Some(base_ty) = resolver.resolve_type(&a.ty) else {
                continue;
            };

            let (ident, ty) = resolve_declarator(decl, base_ty, self.ctx, self.current_scope);
            let alias_ty = AliasTy { ty };
            let span = ic_syntax::util::decl_span(decl);

            let def_id = define::define(
                self.ctx,
                self.current_scope,
                &ident,
                span,
                &a.meta.annotations,
                DefKindTag::Alias,
                |_| DefKind::Alias(alias_ty),
            );
            def_ids.push(def_id);
        }

        def_ids
    }

    pub fn process_exception(&mut self, e: &ExceptDef) -> DefId {
        let members = self.process_members(&e.fields);
        let except_ty = ExceptTy { members };

        define::define(
            self.ctx,
            self.current_scope,
            &e.name,
            e.meta.span,
            &e.meta.annotations,
            DefKindTag::Struct,
            |_| DefKind::Except(except_ty),
        )
    }

    fn process_value_members(&mut self, members: &ic_syntax::StateMember) -> Vec<Member> {
        let mut result = Vec::new();

        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let Some(ty) = resolver.resolve_type(&members.ty) else {
            return result;
        };

        let annotations =
            convert_annotations(self.ctx, &members.meta.annotations, self.current_scope);

        for decl in &members.declarators {
            let (ident, member_ty) =
                resolve_declarator(decl, ty.clone(), self.ctx, self.current_scope);

            result.push(Member {
                ident,
                ty: member_ty,
                annotations: annotations.clone(),
            });
        }

        result
    }
}
