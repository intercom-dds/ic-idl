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

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Def, DefKind, Numeric, UnionTy};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

/// Lint that checks union case labels don't exceed 32 bits.
pub struct UnionCaseLabelRange<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ResolvedGraph,
}

impl<'a> Lint<'a> for UnionCaseLabelRange<'a> {
    fn name() -> &'static str {
        "union-label-range"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn description() -> &'static str {
        "Union case label values must not exceed 32 bits"
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut lint = UnionCaseLabelRange { ctx, hir };
        ic_hir::visit::walk_tree(&mut lint, hir);
    }
}

impl<'a> UnionCaseLabelRange<'a> {
    fn check_label_range(&mut self, value: &Numeric, span: ic_syntax::Span, union_name: &str) {
        let resolved_value = self.resolve_numeric(value);
        if !is_within_32_bits(resolved_value) {
            let label_desc = match value {
                Numeric::Const(def_id) => {
                    let def = self.context().definitions.get(*def_id);
                    format!("case label '{}'", def.ident.name)
                }
                _ => format!("case label value {}", format_numeric(resolved_value)),
            };

            let diag = self.ctx.diag_span(
                Self::name(),
                Self::category(),
                format!("{label_desc} exceeds 32-bit range in union '{union_name}'"),
                Label::new(span).message("value must fit in 32 bits"),
            );
            Self::report(self.ctx, diag);
        }
    }

    fn resolve_numeric(&self, value: &'a Numeric) -> &'a Numeric {
        match value {
            Numeric::Const(def_id) => {
                let def = self.context().definitions.get(*def_id);
                if let DefKind::Const(const_ty) = &def.kind {
                    self.resolve_numeric(&const_ty.value)
                } else {
                    value
                }
            }
            _ => value,
        }
    }
}

impl<'a> Visitor<'a> for UnionCaseLabelRange<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_union(&mut self, def: &'a Def, union_ty: &'a UnionTy) {
        for variant in &union_ty.variants {
            for label in &variant.labels {
                self.check_label_range(&label.value, label.span, def.ident.name.as_str());
            }
        }
        ic_hir::visit::walk_union(self, union_ty);
    }
}

/// Check if a numeric value fits within 32 bits (signed or unsigned).
#[allow(clippy::match_same_arms)]
fn is_within_32_bits(value: &Numeric) -> bool {
    match value {
        Numeric::Int8(_) | Numeric::Int16(_) | Numeric::Int32(_) => true,
        Numeric::UInt8(_) | Numeric::UInt16(_) | Numeric::UInt32(_) => true,
        Numeric::Bool(_) => true,
        Numeric::Char(_) => true,

        Numeric::Int64(v) => *v >= i64::from(i32::MIN) && *v <= i64::from(u32::MAX),
        Numeric::UInt64(v) => u32::try_from(*v).is_ok(),

        // Constants are resolved by check_label_range before calling this function
        Numeric::Const(_) => true,

        // Other types don't make sense for case labels
        Numeric::Float(_) | Numeric::Double(_) => true,
        Numeric::String(_) => true,
        Numeric::Null => true,
        Numeric::Sequence { .. }
        | Numeric::Array { .. }
        | Numeric::Map { .. }
        | Numeric::Struct { .. }
        | Numeric::Union { .. } => true,
    }
}

/// Format a numeric value for error messages.
fn format_numeric(value: &Numeric) -> String {
    match value {
        Numeric::Int8(v) => v.to_string(),
        Numeric::Int16(v) => v.to_string(),
        Numeric::Int32(v) => v.to_string(),
        Numeric::Int64(v) => v.to_string(),
        Numeric::UInt8(v) => v.to_string(),
        Numeric::UInt16(v) => v.to_string(),
        Numeric::UInt32(v) => v.to_string(),
        Numeric::UInt64(v) => v.to_string(),
        _ => format!("{value:?}"),
    }
}
