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

use ic_diagnostic::{Label, error_span};
use ic_syntax::{AnnotationDef, BitmaskDef, BitsetDef, ConstDef, EnumDef};

use super::LoweringContext;
use super::eval::ConstEvaluator;
use super::registry::DefKindTag;
use super::type_resolver::TypeResolver;
use super::utils::TyExt;
use crate::Context;
use crate::hir::{
    AnnParam, AnnotationTy, BitmaskTy, BitsetField, BitsetTy, ConstTy, Def, DefFlags, DefId,
    DefKind, EnumTy, Numeric, PrimitiveTy, Ty, TyKind,
};
use crate::scope::ScopeId;

/// Processes value items (constants, enums, bitmasks).
pub struct ValueItemProcessor<'ctx> {
    pub(super) ctx: &'ctx mut LoweringContext,
    current_scope: ScopeId,
}

impl<'ctx> ValueItemProcessor<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    /// Process a constant definition.
    pub fn process_const(&mut self, c: &ConstDef) -> DefId {
        // Resolve the base type first
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let base_ty = resolver.resolve_type(&c.ty).unwrap_or_else(|| {
            // Use a default type on error
            Ty {
                span: ic_syntax::util::ty_span(&c.ty),
                kind: TyKind::Primitive(PrimitiveTy::Int32),
            }
        });

        // Process the declarator to get identifier and full type (including array dimensions)
        let (ident, ty) = resolve_declarator(&c.decl, base_ty, self.ctx, self.current_scope);

        // Evaluate the value using the promotion-aware evaluator
        let mut eval = ConstEvaluator::new(self.ctx, self.current_scope);
        let value = eval.eval_for_type(&c.value, &ty);

        // Create the constant definition
        let const_ty = ConstTy {
            ty,
            value: value.unwrap_or(Numeric::Null),
        };

        // Convert annotations before the closure
        let annotations = self.convert_annotations(&c.annotations, self.current_scope);

        // Now create the definition fully formed
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: (ident.span),
            kind: DefKind::Const(const_ty),
            flags: DefFlags::nil(),
        });

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &ident,
                DefKindTag::Const,
                def_id,
                &mut self.ctx.diagnostics,
                &self.ctx.context,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx
                .context
                .scopes
                .add_definition(self.current_scope, ident.name.clone(), def_id);
        }

        def_id
    }

    /// Process an enum definition.
    pub fn process_enum(&mut self, e: &EnumDef) -> DefId {
        // Create and register the enum definition
        let enum_id = self.create_enum_definition(e);

        // Create a child scope for the enum to hold its enumerators
        let enum_scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            e.ident.name.clone(),
            Some(enum_id),
        );

        // Process enumerators
        let fields = self.process_enumerators(e, enum_id, enum_scope);

        // Update the enum definition with the collected fields
        if let DefKind::Enum(ref mut enum_ty) = self.ctx.context.definitions.get_mut(enum_id).kind {
            enum_ty.fields = fields;
        }

        enum_id
    }

    /// Create and register an enum definition.
    fn create_enum_definition(&mut self, e: &EnumDef) -> DefId {
        let enum_ty = EnumTy {
            fields: Vec::new(),
            ty: PrimitiveTy::Int32,
        };

        // Convert annotations before the closure
        let annotations = self.convert_annotations(&e.annotations, self.current_scope);

        // Create the enum definition
        let enum_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: e.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: (e.ident.span),
            kind: DefKind::Enum(enum_ty),
            flags: DefFlags::nil(),
        });

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &e.ident,
                DefKindTag::Enum,
                enum_id,
                &mut self.ctx.diagnostics,
                &self.ctx.context,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                e.ident.name.clone(),
                enum_id,
            );
        }

        enum_id
    }

    /// Process all enumerators in an enum.
    fn process_enumerators(
        &mut self,
        e: &EnumDef,
        enum_id: DefId,
        enum_scope: ScopeId,
    ) -> Vec<DefId> {
        let mut fields = Vec::new();
        let mut last_value = -1i64;

        for enumerator in &e.fields {
            // Calculate value
            let value = self.calculate_enumerator_value(enumerator, &mut last_value);
            last_value = value;

            // Check if this enumerator has an explicit value
            let is_explicit = enumerator.value.is_some();

            // Create and register the enumerator
            if let Some(field_id) =
                self.create_enumerator(enumerator, enum_id, value, enum_scope, is_explicit)
            {
                fields.push(field_id);
            }
        }

        // Sort enumerators by their assigned values
        fields.sort_by_key(|&field| enum_key(&self.ctx.context, field));
        fields
    }

    /// Calculate the value for an enumerator.
    fn calculate_enumerator_value(
        &mut self,
        enumerator: &ic_syntax::Enumerator,
        last_value: &mut i64,
    ) -> i64 {
        if let Some(ref expr) = enumerator.value {
            let mut eval = ConstEvaluator::new(self.ctx, self.current_scope);
            if let Some(num) = eval.eval_numeric(expr) {
                match num {
                    Numeric::Int32(v) => i64::from(v),
                    Numeric::Int64(v) => v,
                    Numeric::UInt32(v) => i64::from(v),
                    Numeric::UInt64(v) => v as i64,
                    _ => {
                        self.ctx.diagnostics.error(
                            "enum value must be an integer".to_string(),
                            Label::new(expr.span()).message("expected integer value"),
                        );
                        0
                    }
                }
            } else {
                0
            }
        } else {
            // Auto-increment
            *last_value += 1;
            *last_value
        }
    }

    /// Create an enumerator constant.
    fn create_enumerator(
        &mut self,
        enumerator: &ic_syntax::Enumerator,
        enum_id: DefId,
        value: i64,
        enum_scope: ScopeId,
        is_explicit: bool,
    ) -> Option<DefId> {
        // Convert annotations before the closure
        let annotations = self.convert_annotations(&enumerator.annotations, self.current_scope);

        // Create enumerator as a constant
        let field_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: enumerator.ident.clone(),
            parent: Some(enum_id),
            annotations,
            span: (enumerator.ident.span),
            kind: DefKind::Const(ConstTy {
                ty: Ty {
                    span: (enumerator.ident.span),
                    kind: TyKind::Adt(enum_id),
                },
                value: Numeric::Int32(value as i32),
            }),
            flags: if is_explicit {
                DefFlags::IS_ENUMERATED
            } else {
                DefFlags::nil()
            },
        });

        // Register enumerator through the registry to check for duplicates
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &enumerator.ident,
                DefKindTag::Const,
                field_id,
                &mut self.ctx.diagnostics,
                &self.ctx.context,
            )
            .is_some()
        {
            // Add to parent scope (for unscoped access like TWO)
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                enumerator.ident.name.clone(),
                field_id,
            );

            // Also add to enum's own scope (for scoped access like MyEnum::TWO)
            self.ctx.context.scopes.add_definition(
                enum_scope,
                enumerator.ident.name.clone(),
                field_id,
            );

            Some(field_id)
        } else {
            None
        }
    }

    /// Process a bitmask definition.
    pub fn process_bitmask(&mut self, b: &BitmaskDef) -> DefId {
        let bitmask_ty = BitmaskTy {
            ty: PrimitiveTy::UInt32,
            flags: Vec::new(),
        };

        let annotations = self.convert_annotations(&b.annotations, self.current_scope);
        let bitmask_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: b.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: (b.ident.span),
            kind: DefKind::Bitmask(bitmask_ty),
            flags: DefFlags::nil(),
        });

        // Register with the registry
        if self
            .ctx
            .registry
            .register_definition(
                self.current_scope,
                &b.ident,
                DefKindTag::Bitmask,
                bitmask_id,
                &mut self.ctx.diagnostics,
                &self.ctx.context,
            )
            .is_some()
        {
            // Register in scope only if registry registration succeeded
            self.ctx.context.scopes.add_definition(
                self.current_scope,
                b.ident.name.clone(),
                bitmask_id,
            );
        }

        // Process flags
        let mut flag_ids = Vec::new();
        let mut last_bit = 0u32;
        for (i, flag) in b.bits.iter().enumerate() {
            // Check if this flag has an explicit position
            let is_explicit = flag.value.is_some();

            // Calculate bit position
            let bit_pos = if let Some(ref expr) = flag.value {
                let mut eval = ConstEvaluator::new(self.ctx, self.current_scope);
                eval.eval_nonneg_bound(expr).unwrap_or(0) as u32
            } else {
                // Auto-increment bit position
                if i == 0 { 0 } else { last_bit + 1 }
            };

            last_bit = bit_pos;

            // Calculate value (1 << bit_pos)
            let value = 1u64 << bit_pos;

            // Create flag as a constant in the parent scope
            let flag_ty = Ty {
                span: (flag.ident.span),
                kind: TyKind::Adt(bitmask_id),
            };

            // Convert annotations
            let flag_annotations = self.convert_annotations(&flag.annotations, self.current_scope);

            let flag_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
                id,
                ident: flag.ident.clone(),
                parent: Some(bitmask_id),
                annotations: flag_annotations,
                span: (flag.ident.span),
                kind: DefKind::Const(ConstTy {
                    ty: flag_ty,
                    value: Numeric::UInt64(value),
                }),
                flags: if is_explicit {
                    DefFlags::IS_ENUMERATED
                } else {
                    DefFlags::nil()
                },
            });

            // Register flag in parent scope (not bitmask scope)
            if self
                .ctx
                .registry
                .register_definition(
                    self.current_scope,
                    &flag.ident,
                    DefKindTag::Const,
                    flag_id,
                    &mut self.ctx.diagnostics,
                    &self.ctx.context,
                )
                .is_some()
            {
                self.ctx.context.scopes.add_definition(
                    self.current_scope,
                    flag.ident.name.clone(),
                    flag_id,
                );

                // Add to list of flag IDs
                flag_ids.push(flag_id);
            }
        }

        // Sort flags by their assigned values
        flag_ids.sort_by_key(|&flag_id| {
            let def = self.ctx.context.definitions.get(flag_id);
            if let DefKind::Const(const_ty) = &def.kind {
                match &const_ty.value {
                    Numeric::UInt64(v) => *v,
                    _ => 0,
                }
            } else {
                0
            }
        });

        // Update the bitmask definition with the collected flags
        if let DefKind::Bitmask(ref mut bitmask_ty) =
            self.ctx.context.definitions.get_mut(bitmask_id).kind
        {
            bitmask_ty.flags = flag_ids;
        }

        bitmask_id
    }

    /// Process a bitset definition.
    pub fn process_bitset(&mut self, b: &BitsetDef) -> DefId {
        // Resolve parent bitset if present
        let parent = if let Some(ref parent_path) = b.parent {
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver.resolve_path_type(parent_path).and_then(|ty| {
                if let Some(parent_id) = ty.as_adt() {
                    Some(parent_id)
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be a bitset type".to_string(),
                        Label::new(super::utils::path_span(parent_path))
                            .message("expected bitset type"),
                    );
                    None
                }
            })
        } else {
            None
        };

        // Process bitset fields
        let mut fields = Vec::new();
        for field in &b.fields {
            // Evaluate the size expression
            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
            let Some(size) = evaluator.eval_nonneg_bound(&field.size) else {
                self.ctx.diagnostics.error(
                    "bitfield size must be a non-negative constant expression".to_string(),
                    Label::new(field.size.span()).message("expected constant expression"),
                );
                continue;
            };

            // Resolve the type if present, otherwise default to appropriate unsigned type
            let ty = if let Some(ref field_ty) = field.ty {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                match resolver.resolve_type(field_ty) {
                    Some(ty) => ty,
                    None => continue, // Error already reported
                }
            } else {
                // Default type based on size
                let prim_ty = if size == 1 {
                    PrimitiveTy::Bool
                } else if size <= 8 {
                    PrimitiveTy::UInt8
                } else if size <= 16 {
                    PrimitiveTy::UInt16
                } else if size <= 32 {
                    PrimitiveTy::UInt32
                } else {
                    PrimitiveTy::UInt64
                };
                Ty {
                    span: field.ident.span,
                    kind: TyKind::Primitive(prim_ty),
                }
            };

            // Convert annotations for the field
            let field_annotations =
                self.convert_annotations(&field.annotations, self.current_scope);

            fields.push(BitsetField {
                ident: field.ident.clone(),
                size,
                ty,
                annotations: field_annotations,
            });
        }

        // Create the bitset definition
        let bitset_ty = BitsetTy { parent, fields };

        // Convert annotations before the closure
        let annotations = self.convert_annotations(&b.annotations, self.current_scope);

        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: b.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: b.ident.span,
            kind: DefKind::Bitset(bitset_ty),
            flags: DefFlags::nil(),
        });

        // Bitsets are not forward-declarable, just register in the scope
        self.ctx
            .context
            .scopes
            .add_definition(self.current_scope, b.ident.name.clone(), def_id);

        def_id
    }

    /// Process an annotation definition.
    pub fn process_annotation(&mut self, a: &AnnotationDef) -> DefId {
        // Create scope for the annotation
        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            a.ident.name.clone(),
            None,
        );

        // Convert annotations before the closure
        let annotations = self.convert_annotations(&a.annotations, self.current_scope);

        // Create a placeholder annotation definition first
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: a.ident.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: a.ident.span,
            kind: DefKind::Annotation(AnnotationTy {
                params: Vec::new(),
                types: Vec::new(),
            }), // Placeholder
            flags: DefFlags::nil(),
        });

        // Update the scope's def_id BEFORE processing contents
        self.ctx.context.scopes.get_scope_mut(scope).def_id = Some(def_id);

        // Process annotation parameters and nested types
        let mut params = Vec::new();
        let mut types = Vec::new();

        for field in &a.params {
            match field {
                ic_syntax::AnnotationField::Member(member) => {
                    // Resolve the base type first using the annotation's scope
                    let mut resolver = TypeResolver::new(self.ctx, scope);
                    let Some(base_ty) = resolver.resolve_type(&member.ty) else {
                        continue; // Error already reported
                    };

                    // Use resolve_declarator to handle array types and validate bounds
                    let (ident, ty) = resolve_declarator(&member.decl, base_ty, self.ctx, scope);

                    // Evaluate default value if present
                    let default = if let Some(ref default_expr) = member.default {
                        let mut evaluator = ConstEvaluator::new(self.ctx, scope);
                        evaluator.eval_numeric(default_expr)
                    } else {
                        None
                    };

                    params.push(AnnParam { ident, ty, default });
                }
                ic_syntax::AnnotationField::Item(item) => {
                    // Process nested type definition
                    // Create a new HirBuilder to process this nested item
                    let mut builder = super::builder::HirBuilder::new(self.ctx);

                    // Save current scope and switch to annotation scope
                    let prev_scope = builder.current_scope;
                    builder.current_scope = scope;

                    // Process the nested item and collect its DefIds
                    let item_defs = builder.process_item(item);

                    // Restore previous scope
                    builder.current_scope = prev_scope;

                    // Add the returned DefIds to our types list
                    types.extend(item_defs);
                }
            }
        }

        // Update the annotation definition with collected params and types
        if let DefKind::Annotation(ref mut annotation_ty) =
            self.ctx.context.definitions.get_mut(def_id).kind
        {
            annotation_ty.params = params;
            annotation_ty.types = types;
        }

        // Register in annotation namespace
        if let Some(prev_def_id) =
            self.ctx
                .context
                .scopes
                .add_annotation(self.current_scope, a.ident.name.clone(), def_id)
        {
            let existing_def = self.ctx.context.definitions.get(prev_def_id);
            self.ctx.diagnostics.errors.push(
                error_span(
                    format!("duplicate annotation definition `@{}`", a.ident.name),
                    Label::new(existing_def.ident.span).message("originally defined here"),
                )
                .label(Label::new(a.ident.span).message("redefined here")),
            );
        }

        def_id
    }
}

/// Resolves a declarator to produce an identifier and type.
/// Handles array declarators by building array types from the base type.
pub(super) fn resolve_declarator(
    decl: &ic_syntax::Declarator,
    base_ty: Ty,
    ctx: &mut LoweringContext,
    scope: ScopeId,
) -> (ic_syntax::Ident, Ty) {
    match decl {
        ic_syntax::Declarator::Simple(ident) => (ident.clone(), base_ty),
        ic_syntax::Declarator::Array(arr) => {
            // Build array type from rightmost to leftmost bound
            // For int[2][3], we want Array<Array<int, 3>, 2>
            let mut ty = base_ty;

            // Process bounds in reverse order
            for bound_expr in arr.bounds.iter().rev() {
                // Evaluate the bound expression
                let mut evaluator = ConstEvaluator::new(ctx, scope);
                let len = evaluator.eval_nonneg_bound(bound_expr).unwrap_or(1);

                ty = Ty {
                    span: ty.span,
                    kind: TyKind::Array {
                        ty: Box::new(ty.clone()),
                        len,
                        len_span: ic_syntax::util::expr_span(bound_expr),
                    },
                };
            }
            (arr.ident.clone(), ty)
        }
    }
}

fn enum_key(ctx: &Context, field: DefId) -> i64 {
    let def = ctx.definitions.get(field);
    if let DefKind::Const(ref const_ty) = def.kind {
        resolve_numeric_value(ctx, &const_ty.value)
    } else {
        0
    }
}

/// Recursively resolve a numeric value, following constant references
fn resolve_numeric_value(ctx: &Context, value: &Numeric) -> i64 {
    match value {
        Numeric::Int8(v) => i64::from(*v),
        Numeric::Int16(v) => i64::from(*v),
        Numeric::Int32(v) => i64::from(*v),
        Numeric::Int64(v) => *v,
        Numeric::Octet(v) => i64::from(*v),
        Numeric::UInt16(v) => i64::from(*v),
        Numeric::UInt32(v) => i64::from(*v),
        Numeric::UInt64(v) => *v as i64,
        Numeric::Const(def_id) => {
            let def = ctx.definitions.get(*def_id);
            if let DefKind::Const(ref const_ty) = def.kind {
                resolve_numeric_value(ctx, &const_ty.value)
            } else {
                0
            }
        }
        _ => 0,
    }
}
