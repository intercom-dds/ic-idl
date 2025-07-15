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

use ic_diagnostic::Label;
use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, Def, Numeric, PrimitiveTy, TyKind};
use ic_hir::visit::Visitor;

use crate::{Category, Lint, LintCtx};

pub struct BitBound<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Lint<'a> for BitBound<'a> {
    fn name() -> &'static str {
        "BitBound"
    }

    fn category() -> Category {
        Category::Semantic
    }

    fn check_hir(ctx: &'a LintCtx<'_>, hir: &ResolvedGraph) {
        let mut visitor = BitBound { ctx };
        ic_hir::visit::walk_tree(&mut visitor, &hir.context.definitions);
    }
}

impl<'a> BitBound<'a> {
    fn check_bit_annotation(&mut self, ann: &Ann, type_bits: u32) {
        let name = ann
            .path
            .segments
            .last()
            .map(|s| s.name.as_str())
            .unwrap_or("");
        if name != "bit" {
            return;
        }

        if let Some(bit_pos) = self.get_bit_position(ann) {
            if bit_pos >= type_bits {
                if let Some(diag) = self.ctx.diag_span(
                    Self::name(),
                    Self::category(),
                    &format!("@bit({bit_pos}) exceeds type bit width of {type_bits}"),
                    Label::new(ic_syntax::util::path_span(&ann.path))
                        .message("bit position out of bounds"),
                ) {
                    self.ctx.report(Self::name(), Self::category(), diag);
                }
            }
        }
    }

    fn get_bit_position(&self, ann: &Ann) -> Option<u32> {
        ann.args.get(0).and_then(|arg| match &arg.value {
            Numeric::Int32(v) if *v >= 0 => Some(*v as u32),
            Numeric::UInt32(v) => Some(*v),
            Numeric::Int64(v) if *v >= 0 && *v <= u32::MAX as i64 => Some(*v as u32),
            Numeric::UInt64(v) if *v <= u32::MAX as u64 => Some(*v as u32),
            _ => None,
        })
    }

    fn get_type_bits(ty: &TyKind) -> Option<u32> {
        match ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveTy::UInt8 => Some(8),
                PrimitiveTy::UInt16 => Some(16),
                PrimitiveTy::UInt32 => Some(32),
                PrimitiveTy::UInt64 => Some(64),
                PrimitiveTy::Int8 => Some(8),
                PrimitiveTy::Int16 => Some(16),
                PrimitiveTy::Int32 => Some(32),
                PrimitiveTy::Int64 => Some(64),
                _ => None,
            },
            TyKind::Adt(_) => {
                // TODO: Follow typedef/alias to get underlying type
                None
            }
            _ => None,
        }
    }
}

impl<'a> Visitor<'a> for BitBound<'a> {
    fn visit_bitmask(&mut self, _def: &'a Def, data: &'a ic_hir::hir::BitmaskTy) {
        // Check bit positions in bitmask flags
        if let Some(type_bits) = Self::get_type_bits(&data.ty.kind) {
            for flag in &data.flags {
                // Check if the flag has a @bit annotation
                for ann in &flag.annotations {
                    self.check_bit_annotation(ann, type_bits);
                }

                // Also check if the explicit value exceeds the type bounds
                if flag.value >= (1u64 << type_bits) as usize {
                    // This would be a different lint, but we can warn here too
                    if let Some(diag) = self.ctx.diag_span(
                        Self::name(),
                        Self::category(),
                        &format!(
                            "bitmask value {value} exceeds type bit width of {type_bits}",
                            value = flag.value
                        ),
                        Label::new(flag.ident.span).message("value out of bounds"),
                    ) {
                        self.ctx.report(Self::name(), Self::category(), diag);
                    }
                }
            }
        }

        // Continue visiting
        ic_hir::visit::walk_bitmask(self, data);
    }
}
