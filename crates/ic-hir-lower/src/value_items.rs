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
use ic_hir::Context;
use ic_hir::hir::{
    AnnParam, AnnotationTy, BitmaskTy, BitsetField, BitsetTy, ConstTy, Def, DefFlags, DefId,
    DefKind, EnumTy, Numeric, PrimitiveTy, Ty, TyKind,
};
use ic_hir::scope::ScopeId;
use ic_syntax::{AnnotationDef, BitmaskDef, BitsetDef, ConstDef, EnumDef};

use crate::LoweringContext;
use crate::annotation::convert_annotations;
use crate::eval::ConstEvaluator;
use crate::registry::DefKindTag;
use crate::resolve::{TypeResolver, resolve_declarator};
use crate::utils::TyExt;

/// Processes value items (constants, enums, bitmasks).
pub struct ValueItemProcessor<'ctx> {
    pub(super) ctx: &'ctx mut LoweringContext,
    current_scope: ScopeId,
}

impl<'ctx> ValueItemProcessor<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext, current_scope: ScopeId) -> Self {
        Self { ctx, current_scope }
    }

    pub fn process_const(&mut self, c: &ConstDef) -> DefId {
        let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
        let (base_ty, type_resolved) = if let Some(ty) = resolver.resolve_type(&c.ty) {
            (ty, true)
        } else {
            let fallback = Ty {
                span: ic_syntax::util::ty_span(&c.ty),
                kind: TyKind::Primitive(PrimitiveTy::Int32),
            };
            (fallback, false)
        };

        let (ident, ty) = resolve_declarator(&c.declarator, base_ty, self.ctx, self.current_scope);

        // Skip evaluation if type resolution failed to avoid confusing secondary errors
        let value = if type_resolved {
            let mut eval = ConstEvaluator::new(self.ctx, self.current_scope);
            eval.eval_for_type(&c.value, &ty)
        } else {
            None
        };

        let const_ty = ConstTy {
            ty,
            value: value.unwrap_or(Numeric::Null),
        };

        crate::define::define(
            self.ctx,
            self.current_scope,
            &ident,
            c.meta.span,
            &c.meta.annotations,
            DefKindTag::Const,
            |_| DefKind::Const(const_ty),
        )
    }

    pub fn process_enum(&mut self, e: &EnumDef) -> DefId {
        let enum_id = self.create_enum_definition(e);

        let enum_scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            e.name.name.clone(),
            Some(enum_id),
        );

        let fields = self.process_enumerators(e, enum_id, enum_scope);

        if let DefKind::Enum(ref mut enum_ty) = self.ctx.context.definitions.get_mut(enum_id).kind {
            enum_ty.fields = fields;
        }

        enum_id
    }

    fn create_enum_definition(&mut self, e: &EnumDef) -> DefId {
        let enum_ty = EnumTy {
            fields: Vec::new(),
            ty: PrimitiveTy::Int32,
        };

        crate::define::define(
            self.ctx,
            self.current_scope,
            &e.name,
            e.meta.span,
            &e.meta.annotations,
            DefKindTag::Enum,
            |_| DefKind::Enum(enum_ty),
        )
    }

    fn process_enumerators(
        &mut self,
        e: &EnumDef,
        enum_id: DefId,
        enum_scope: ScopeId,
    ) -> Vec<DefId> {
        let mut fields = Vec::new();
        let mut last_value = -1i64;

        for enumerator in &e.enumerators {
            let value = self.calculate_enumerator_value(enumerator, &mut last_value);
            last_value = value;
            let is_explicit = enumerator.value.is_some();

            fields.push(self.create_enumerator(
                enumerator,
                enum_id,
                value,
                enum_scope,
                is_explicit,
            ));
        }

        fields.sort_by_key(|&field| enum_key(&self.ctx.context, field));
        fields
    }

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
                            Label::new(expr.span).message("expected integer value"),
                        );
                        0
                    }
                }
            } else {
                0
            }
        } else {
            *last_value += 1;
            *last_value
        }
    }

    fn create_enumerator(
        &mut self,
        enumerator: &ic_syntax::Enumerator,
        enum_id: DefId,
        value: i64,
        enum_scope: ScopeId,
        is_explicit: bool,
    ) -> DefId {
        let flags = if is_explicit {
            DefFlags::IS_ENUMERATED
        } else {
            DefFlags::nil()
        };
        crate::define::define_scoped_const(
            self.ctx,
            self.current_scope,
            enum_scope,
            &enumerator.name,
            enumerator.name.span,
            &enumerator.meta.annotations,
            flags,
            |_| {
                DefKind::Const(ConstTy {
                    ty: Ty {
                        span: enumerator.name.span,
                        kind: TyKind::Adt(enum_id),
                    },
                    value: Numeric::Int32(value as i32),
                })
            },
        )
    }

    fn process_bitmask_flag(
        &mut self,
        flag: &ic_syntax::Bit,
        i: usize,
        last_bit: &mut u32,
        bitmask_id: DefId,
        bitmask_scope: ScopeId,
    ) -> Option<DefId> {
        let is_explicit = flag.value.is_some();

        let bit_pos = if let Some(ref expr) = flag.value {
            let mut eval = ConstEvaluator::new(self.ctx, self.current_scope);
            eval.eval_nonneg_bound(expr).unwrap_or(0) as u32
        } else if i == 0 {
            0
        } else {
            *last_bit + 1
        };

        *last_bit = bit_pos;

        let Some(value) = 1u64.checked_shl(bit_pos) else {
            self.ctx.diagnostics.errors.push(error_span(
                "bitmask bit position out of range",
                Label::new(flag.meta.span).message(format!(
                    "bit position {bit_pos} exceeds maximum of 63 for 64-bit bitmask"
                )),
            ));
            return None;
        };

        let flag_ty = Ty {
            span: flag.name.span,
            kind: TyKind::Adt(bitmask_id),
        };
        let flags = if is_explicit {
            DefFlags::IS_ENUMERATED
        } else {
            DefFlags::nil()
        };

        let flag_id = crate::define::define_scoped_const(
            self.ctx,
            self.current_scope,
            bitmask_scope,
            &flag.name,
            flag.meta.span,
            &flag.meta.annotations,
            flags,
            |_| {
                DefKind::Const(ConstTy {
                    ty: flag_ty,
                    value: Numeric::UInt64(value),
                })
            },
        );
        Some(flag_id)
    }

    pub fn process_bitmask(&mut self, b: &BitmaskDef) -> DefId {
        let bitmask_ty = BitmaskTy {
            ty: PrimitiveTy::UInt32,
            flags: Vec::new(),
        };

        let bitmask_id = crate::define::define(
            self.ctx,
            self.current_scope,
            &b.name,
            b.meta.span,
            &b.meta.annotations,
            DefKindTag::Bitmask,
            |_| DefKind::Bitmask(bitmask_ty),
        );

        let bitmask_scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            b.name.name.clone(),
            Some(bitmask_id),
        );

        let mut flag_ids = Vec::new();
        let mut last_bit = 0u32;
        for (i, flag) in b.bits.iter().enumerate() {
            if let Some(flag_id) =
                self.process_bitmask_flag(flag, i, &mut last_bit, bitmask_id, bitmask_scope)
            {
                flag_ids.push(flag_id);
            }
        }

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

        if let DefKind::Bitmask(ref mut bitmask_ty) =
            self.ctx.context.definitions.get_mut(bitmask_id).kind
        {
            bitmask_ty.flags = flag_ids;
        }

        bitmask_id
    }

    pub fn process_bitset(&mut self, b: &BitsetDef) -> DefId {
        let parent = if let Some(ref parent_path) = b.parent {
            let path_span = crate::utils::path_span(parent_path);
            let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
            resolver.resolve_path_type(parent_path).and_then(|ty| {
                if let Some(parent_id) = ty.as_adt() {
                    crate::type_items::validate_parent_inheritance(
                        self.ctx,
                        parent_id,
                        "bitset",
                        &b.name.name,
                        "inherit from",
                        path_span,
                    )
                    .map(|value| ic_hir::hir::Spanned {
                        def_id: value,
                        span: path_span,
                    })
                } else {
                    self.ctx.diagnostics.error(
                        "parent must be a bitset type".to_string(),
                        Label::new(path_span).message("expected bitset type"),
                    );
                    None
                }
            })
        } else {
            None
        };

        let mut fields = Vec::new();
        for field in &b.fields {
            // Evaluate the size expression
            let mut evaluator = ConstEvaluator::new(self.ctx, self.current_scope);
            let Some(size) = evaluator.eval_nonneg_bound(&field.size) else {
                self.ctx.diagnostics.error(
                    "bitfield size must be a non-negative constant expression".to_string(),
                    Label::new(field.size.span).message("expected constant expression"),
                );
                continue;
            };

            let ty = if let Some(ref field_ty) = field.ty {
                let mut resolver = TypeResolver::new(self.ctx, self.current_scope);
                match resolver.resolve_type(field_ty) {
                    Some(ty) => ty,
                    None => continue,
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
                    span: field.meta.span,
                    kind: TyKind::Primitive(prim_ty),
                }
            };

            let annotations =
                convert_annotations(self.ctx, &field.meta.annotations, self.current_scope);
            for declarator in &field.declarators {
                let ident = match declarator {
                    ic_syntax::Declarator::Name(ident) => ident.clone(),
                    ic_syntax::Declarator::Array(array) => array.name.clone(),
                };
                fields.push(BitsetField {
                    ident,
                    size,
                    ty: ty.clone(),
                    annotations: annotations.clone(),
                });
            }
        }

        let bitset_ty = BitsetTy { parent, fields };

        crate::define::define(
            self.ctx,
            self.current_scope,
            &b.name,
            b.meta.span,
            &b.meta.annotations,
            DefKindTag::Bitset,
            |_| DefKind::Bitset(bitset_ty),
        )
    }

    pub fn process_annotation(&mut self, a: &AnnotationDef) -> DefId {
        let scope = self.ctx.context.scopes.create_child_scope(
            self.current_scope,
            format!("@{}", a.name.name),
            None,
        );

        let annotations = convert_annotations(self.ctx, &a.meta.annotations, self.current_scope);
        let def_id = self.ctx.context.definitions.alloc_with_id(|id| Def {
            id,
            ident: a.name.clone(),
            parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
            annotations,
            span: a.meta.span,
            kind: DefKind::Annotation(AnnotationTy {
                params: Vec::new(),
                types: Vec::new(),
            }),
            flags: DefFlags::nil(),
        });

        // Must set before processing contents so nested items can reference the annotation
        self.ctx.context.scopes.set_scope_def_id(scope, def_id);

        let mut params = Vec::new();
        let mut types = Vec::new();

        for field in &a.members {
            match field {
                ic_syntax::AnnotationMember::Value(member) => {
                    let mut resolver = TypeResolver::new(self.ctx, scope);
                    let Some(base_ty) = resolver.resolve_type(&member.ty) else {
                        continue;
                    };

                    let (ident, ty) =
                        resolve_declarator(&member.declarator, base_ty, self.ctx, scope);

                    let default = if let Some(ref default_expr) = member.default {
                        let mut evaluator = ConstEvaluator::new(self.ctx, scope);
                        evaluator.eval_numeric(default_expr)
                    } else {
                        None
                    };

                    params.push(AnnParam { ident, ty, default });
                }
                ic_syntax::AnnotationMember::Item(item) => {
                    types.extend(crate::builder::process_nested_item(self.ctx, scope, item));
                }
            }
        }

        if let DefKind::Annotation(ref mut annotation_ty) =
            self.ctx.context.definitions.get_mut(def_id).kind
        {
            annotation_ty.params = params;
            annotation_ty.types = types;
        }

        crate::define::define_annotation(
            self.ctx,
            self.current_scope,
            &a.name,
            def_id,
            |old, new, ctx| are_annotations_consistent(&old.kind, &new.kind, ctx),
        );

        def_id
    }
}

fn are_annotations_consistent(existing: &DefKind, new: &DefKind, ctx: &Context) -> bool {
    let (DefKind::Annotation(existing_ann), DefKind::Annotation(new_ann)) = (existing, new) else {
        return false;
    };

    if existing_ann.params.len() != new_ann.params.len() {
        return false;
    }

    for (existing_param, new_param) in existing_ann.params.iter().zip(&new_ann.params) {
        if existing_param.ident.name != new_param.ident.name {
            return false;
        }

        if !types_equal(&existing_param.ty, &new_param.ty, ctx) {
            return false;
        }

        match (&existing_param.default, &new_param.default) {
            (None, None) => {}
            (Some(e), Some(n)) => {
                if !numerics_equal(e, n, ctx) {
                    return false;
                }
            }
            _ => return false,
        }
    }

    if existing_ann.types.len() != new_ann.types.len() {
        return false;
    }

    true
}

#[allow(clippy::used_underscore_binding)]
pub(crate) fn types_equal(a: &Ty, b: &Ty, _ctx: &Context) -> bool {
    use TyKind::{Adt, Any, Array, Fixed, Map, Null, Primitive, Sequence, String};
    match (&a.kind, &b.kind) {
        (Any, Any) | (Fixed, Fixed) | (Null, Null) => true,
        (Primitive(p1), Primitive(p2)) => p1 == p2,
        (
            Array {
                ty: ty1, len: len1, ..
            },
            Array {
                ty: ty2, len: len2, ..
            },
        ) => len1 == len2 && types_equal(ty1, ty2, _ctx),
        (
            Sequence {
                ty: ty1, bound: b1, ..
            },
            Sequence {
                ty: ty2, bound: b2, ..
            },
        ) => b1 == b2 && types_equal(ty1, ty2, _ctx),
        (
            String {
                wide: w1,
                bound: b1,
                ..
            },
            String {
                wide: w2,
                bound: b2,
                ..
            },
        ) => w1 == w2 && b1 == b2,
        (
            Map {
                key: k1,
                elem: e1,
                bound: b1,
                ..
            },
            Map {
                key: k2,
                elem: e2,
                bound: b2,
                ..
            },
        ) => b1 == b2 && types_equal(k1, k2, _ctx) && types_equal(e1, e2, _ctx),
        (Adt(id1), Adt(id2)) => id1 == id2,
        _ => false,
    }
}

fn resolve_const(ctx: &Context, id: DefId) -> &Numeric {
    let def = ctx.definitions.get(id);
    if let DefKind::Const(ref const_ty) = def.kind {
        &const_ty.value
    } else {
        &Numeric::Null
    }
}

fn compare_array_or_sequence(
    ty1: &Ty,
    ty2: &Ty,
    v1: &[Numeric],
    v2: &[Numeric],
    ctx: &Context,
) -> bool {
    types_equal(ty1, ty2, ctx)
        && v1.len() == v2.len()
        && v1
            .iter()
            .zip(v2.iter())
            .all(|(a, b)| numerics_equal(a, b, ctx))
}

fn compare_map(
    k1: &Ty,
    k2: &Ty,
    v1: &Ty,
    v2: &Ty,
    e1: &[(Numeric, Numeric)],
    e2: &[(Numeric, Numeric)],
    ctx: &Context,
) -> bool {
    types_equal(k1, k2, ctx)
        && types_equal(v1, v2, ctx)
        && e1.len() == e2.len()
        && e1
            .iter()
            .zip(e2.iter())
            .all(|((k1, v1), (k2, v2))| numerics_equal(k1, k2, ctx) && numerics_equal(v1, v2, ctx))
}

#[allow(clippy::too_many_lines)]
fn numerics_equal(a: &Numeric, b: &Numeric, ctx: &Context) -> bool {
    use Numeric::{
        Array, Bool, Char, Const, Double, Float, Int8, Int16, Int32, Int64, Map, Null, Sequence,
        String, Struct, UInt8, UInt16, UInt32, UInt64, Union, WChar, WString,
    };
    match (a, b) {
        (Null, Null) => true,
        (Bool(v1), Bool(v2)) => v1 == v2,
        (Char(v1), Char(v2)) | (WChar(v1), WChar(v2)) => v1 == v2,
        (Int8(v1), Int8(v2)) => v1 == v2,
        (UInt8(v1), UInt8(v2)) => v1 == v2,
        (Int16(v1), Int16(v2)) => v1 == v2,
        (UInt16(v1), UInt16(v2)) => v1 == v2,
        (Int32(v1), Int32(v2)) => v1 == v2,
        (UInt32(v1), UInt32(v2)) => v1 == v2,
        (Int64(v1), Int64(v2)) => v1 == v2,
        (UInt64(v1), UInt64(v2)) => v1 == v2,
        (Float(v1), Float(v2)) => v1.to_bits() == v2.to_bits(),
        (Double(v1), Double(v2)) => v1.to_bits() == v2.to_bits(),
        (String(v1), String(v2)) | (WString(v1), WString(v2)) => v1 == v2,
        (Const(id1), Const(id2)) => {
            if id1 == id2 {
                return true;
            }
            let resolved1 = resolve_const(ctx, *id1);
            let resolved2 = resolve_const(ctx, *id2);
            numerics_equal(resolved1, resolved2, ctx)
        }
        (Const(id), other) | (other, Const(id)) => {
            let resolved = resolve_const(ctx, *id);
            numerics_equal(resolved, other, ctx)
        }
        (
            Array {
                ty: ty1,
                values: v1,
            },
            Array {
                ty: ty2,
                values: v2,
            },
        )
        | (
            Sequence {
                ty: ty1,
                values: v1,
            },
            Sequence {
                ty: ty2,
                values: v2,
            },
        ) => compare_array_or_sequence(ty1, ty2, v1, v2, ctx),
        (
            Map {
                key: k1,
                value: v1,
                entries: e1,
            },
            Map {
                key: k2,
                value: v2,
                entries: e2,
            },
        ) => compare_map(k1, k2, v1, v2, e1, e2, ctx),
        (
            Struct {
                ty: ty1,
                fields: f1,
            },
            Struct {
                ty: ty2,
                fields: f2,
            },
        ) => {
            ty1 == ty2
                && f1.len() == f2.len()
                && f1
                    .iter()
                    .zip(f2.iter())
                    .all(|(v1, v2)| numerics_equal(v1, v2, ctx))
        }
        (
            Union {
                ty: ty1,
                discriminant: d1,
                field_index: f1,
                value: v1,
            },
            Union {
                ty: ty2,
                discriminant: d2,
                field_index: f2,
                value: v2,
            },
        ) => ty1 == ty2 && f1 == f2 && numerics_equal(d1, d2, ctx) && numerics_equal(v1, v2, ctx),
        _ => false,
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
        Numeric::UInt8(v) => i64::from(*v),
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
