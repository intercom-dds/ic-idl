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

use ic_diagnostic::{Color, Diag, Label};
use ic_hir::hir::{self, DefKind, Numeric, TyKind};
use ic_hir::visit::walk_tree;
use ic_hir::{Context, ResolvedGraph};
use ic_syntax::Span;

use crate::{Category, Lint, LintCtx};

pub struct InitializerListSize<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
}

impl<'a> Lint<'a> for InitializerListSize<'a> {
    fn name() -> &'static str {
        "initializer-list-size"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Errors when initializer list size doesn't match type"
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ResolvedGraph) {
        let mut lint = Self { ctx, hir };
        walk_tree(&mut lint, hir);
    }
}

impl<'a> ic_hir::visit::Visitor<'a> for InitializerListSize<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_const(&mut self, def: &'a hir::Def, data: &'a hir::ConstTy) {
        validate_init_list(
            self.ctx,
            &self.hir.context,
            &data.value,
            &data.ty,
            def.ident.span,
        );
    }

    fn visit_ann_param(&mut self, param: &'a hir::AnnParam) {
        if let Some(num) = &param.default {
            validate_init_list(
                self.ctx,
                &self.hir.context,
                num,
                &param.ty,
                param.ident.span,
            );
        }
    }
}

fn validate_init_list(
    ctx: &LintCtx<'_>,
    context: &Context,
    numeric: &Numeric,
    expected_ty: &hir::Ty,
    span: Span,
) {
    match numeric {
        Numeric::Array { ty, values } => {
            validate_array_init(ctx, context, ty, values, expected_ty, span);
        }
        Numeric::Struct { ty, fields } => {
            validate_struct_init(ctx, context, *ty, fields, expected_ty, span);
        }
        Numeric::Sequence {
            ty: seq_elem_ty,
            values,
        } => {
            validate_sequence_init(ctx, context, seq_elem_ty, values, expected_ty, span);
        }
        Numeric::Map {
            key: map_key_ty,
            value: map_val_ty,
            entries: values,
        } => {
            validate_map_init(
                ctx,
                context,
                map_key_ty,
                map_val_ty,
                values,
                expected_ty,
                span,
            );
        }
        Numeric::Union { value, .. } => {
            validate_init_list(ctx, context, value, expected_ty, span);
        }
        _ => {}
    }
}

fn validate_array_init(
    ctx: &LintCtx<'_>,
    context: &Context,
    ty: &hir::Ty,
    values: &[Numeric],
    expected_ty: &hir::Ty,
    span: Span,
) {
    if let TyKind::Array {
        len: expected_len,
        ty: elem_ty,
        ..
    } = &expected_ty.kind
    {
        if values.len() != *expected_len {
            ctx.report(
                InitializerListSize::name(),
                InitializerListSize::category(),
                Diag::error(format!(
                    "array initializer has {} elements but array type expects {}",
                    values.len(),
                    expected_len
                ))
                .label(
                    Label::new(span)
                        .message(format!("expected {expected_len} elements"))
                        .color(Color::Red),
                ),
            );
        }
        for value in values {
            validate_init_list(ctx, context, value, elem_ty, span);
        }
    } else {
        for value in values {
            validate_init_list(ctx, context, value, ty, span);
        }
    }
}

fn validate_struct_init(
    ctx: &LintCtx<'_>,
    context: &Context,
    ty: hir::DefId,
    fields: &[(ic_syntax::Ident, Numeric)],
    expected_ty: &hir::Ty,
    span: Span,
) {
    let struct_def = context.definitions.get(ty);
    if let DefKind::Struct(struct_ty) = &struct_def.kind {
        let expected_count = struct_ty.members.len();
        if fields.len() != expected_count {
            ctx.report(
                InitializerListSize::name(),
                InitializerListSize::category(),
                Diag::error(format!(
                    "struct initializer has {} fields but struct '{}' has {} members",
                    fields.len(),
                    struct_def.ident.name,
                    expected_count
                ))
                .label(
                    Label::new(span)
                        .message(format!("expected {expected_count} fields"))
                        .color(Color::Red),
                ),
            );
        }
        for (field_name, value) in fields {
            let field_ty = struct_ty
                .members
                .iter()
                .find(|m| m.ident.name == field_name.name)
                .map_or(expected_ty, |m| &m.ty);
            validate_init_list(ctx, context, value, field_ty, span);
        }
    }
}

fn validate_sequence_init(
    ctx: &LintCtx<'_>,
    context: &Context,
    seq_elem_ty: &hir::Ty,
    values: &[Numeric],
    expected_ty: &hir::Ty,
    span: Span,
) {
    let elem_ty = if let TyKind::Sequence {
        ty: expected_elem_ty,
        ..
    } = &expected_ty.kind
    {
        expected_elem_ty
    } else {
        seq_elem_ty
    };

    for value in values {
        validate_init_list(ctx, context, value, elem_ty, span);
    }
}

fn validate_map_init(
    ctx: &LintCtx<'_>,
    context: &Context,
    map_key_ty: &hir::Ty,
    map_val_ty: &hir::Ty,
    values: &[(Numeric, Numeric)],
    expected_ty: &hir::Ty,
    span: Span,
) {
    if let TyKind::Map {
        key: expected_key_ty,
        elem: expected_val_ty,
        ..
    } = &expected_ty.kind
    {
        for (key, value) in values {
            validate_init_list(ctx, context, key, expected_key_ty, span);
            validate_init_list(ctx, context, value, expected_val_ty, span);
        }
    } else {
        for (key, value) in values {
            validate_init_list(ctx, context, key, map_key_ty, span);
            validate_init_list(ctx, context, value, map_val_ty, span);
        }
    }
}

//
// impl<'a> Visitor<'a> for InitializerListSize<'a> {
//     fn context(&self) -> &'a Context {
//         &self.hir.context
//     }
//
//     fn visit_numeric(&mut self, numeric: &'a Numeric) {
//         match numeric {
//             Numeric::Array { ty, values } => {
//                 if let Some(expected_len) = array_len(context, *ty) {
//                     if values.len() != expected_len {
//                         self.report(
//                             Self::name(),
//                             Self::category(),
//                             Diag::error(format!(
//                                 "array initializer has {} elements but array type expects {}",
//                                 values.len(),
//                                 expected_len
//                             ))
//                             .label(
//                                 Label::new(span)
//                                     .message(format!("expected {expected_len} elements"))
//                                     .color(Color::Red),
//                             ),
//                         );
//                     }
//                 }
//                 for value in values {
//                     self.visit_numeric(value);
//                 }
//             }
//             Numeric::Struct { ty, fields } => {
//                 let struct_def = self.ctx.definitions.get(*ty);
//                 if let DefKind::Struct(struct_ty) = &struct_def.kind {
//                     let expected_count = struct_ty.members.len();
//                     if fields.len() != expected_count {
//                         self.report(
//                             InitializerListSize::name(),
//                             InitializerListSize::category(),
//                             Diag::error(format!(
//                                 "struct initializer has {} fields but struct '{}' has {} members",
//                                 fields.len(),
//                                 struct_def.ident.name,
//                                 expected_count
//                             ))
//                             .label(
//                                 Label::new(span)
//                                     .message(format!("expected {expected_count} fields"))
//                                     .color(Color::Red),
//                             ),
//                         );
//                     }
//                 }
//                 for (_, value) in fields {
//                     validate_numeric_initializer(ctx, context, value, span);
//                 }
//             }
//             Numeric::Sequence { values, .. } => {
//                 for value in values {
//                     validate_numeric_initializer(ctx, context, value, span);
//                 }
//             }
//             Numeric::Map { values, .. } => {
//                 for (key, value) in values {
//                     validate_numeric_initializer(ctx, context, key, span);
//                     validate_numeric_initializer(ctx, context, value, span);
//                 }
//             }
//             Numeric::Union { value, .. } => {
//                 validate_numeric_initializer(ctx, context, value, span);
//             }
//             _ => {}
//         }
//     }
// }
//
// fn array_len(context: &Context, type_id: DefId) -> Option<usize> {
//     let def = context.definitions.get(type_id);
//
//     match &def.kind {
//         DefKind::Const(const_ty) => {
//             if let TyKind::Array { len, .. } = &const_ty.ty.kind {
//                 Some(*len)
//             } else {
//                 None
//             }
//         }
//         _ => None,
//     }
// }
