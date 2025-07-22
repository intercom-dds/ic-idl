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
use ic_hir::hir::{DefId, DefKind, Numeric, TyKind};
use ic_hir::{Context, ResolvedGraph};
use ic_syntax::Span;

use crate::{Category, Lint, LintCtx};

pub struct InitializerListSize<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lint<'a> for InitializerListSize<'a> {
    fn name() -> &'static str {
        "initializer_list_size"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        for (_, def) in &hir.context.definitions {
            if let DefKind::Const(const_ty) = &def.kind {
                validate_numeric_initializer(ctx, &hir.context, &const_ty.value, def.span);
            }
        }
    }
}

fn validate_numeric_initializer(
    ctx: &LintCtx<'_>,
    context: &Context,
    numeric: &Numeric,
    span: Span,
) {
    match numeric {
        Numeric::Array { ty, values } => {
            if let Some(expected_len) = get_array_length(context, *ty) {
                if values.len() != expected_len {
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
            }
            for value in values {
                validate_numeric_initializer(ctx, context, value, span);
            }
        }
        Numeric::Struct { ty, fields } => {
            let struct_def = context.definitions.get(*ty);
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
            }
            for (_, value) in fields {
                validate_numeric_initializer(ctx, context, value, span);
            }
        }
        Numeric::Sequence { values, .. } => {
            for value in values {
                validate_numeric_initializer(ctx, context, value, span);
            }
        }
        Numeric::Map { values, .. } => {
            for (key, value) in values {
                validate_numeric_initializer(ctx, context, key, span);
                validate_numeric_initializer(ctx, context, value, span);
            }
        }
        Numeric::Union { value, .. } => {
            validate_numeric_initializer(ctx, context, value, span);
        }
        _ => {}
    }
}

fn get_array_length(context: &Context, type_id: DefId) -> Option<usize> {
    let def = context.definitions.get(type_id);

    match &def.kind {
        DefKind::Const(const_ty) => {
            if let TyKind::Array { len, .. } = &const_ty.ty.kind {
                Some(*len)
            } else {
                None
            }
        }
        _ => None,
    }
}
