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

//! Phase 4: Type checking.
//!
//! This phase validates that:
//! - Constant values match their declared types
//! - Array/sequence/map bounds are valid
//! - Enum discriminator values fit in the underlying type
//! - Bitmask values fit in the underlying type
//! - Union case labels match the discriminator type
//!
//! At this point, all expressions have been evaluated, so we can check types.

use ic_diagnostic::{Diag, Label, error_span};

use crate::Context;
use crate::hir::{DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};

/// Type checks the HIR.
pub struct TypeChecker<'a> {
    ctx: &'a Context,
    errors: Vec<Diag>,
}

impl<'a> TypeChecker<'a> {
    fn new(ctx: &'a Context) -> Self {
        Self {
            ctx,
            errors: Vec::new(),
        }
    }

    /// Checks if a numeric value is compatible with a type.
    fn check_numeric_type(&mut self, value: &Numeric, ty: &Ty, value_desc: &str) -> bool {
        // Special case for struct values first
        if let Numeric::Struct {
            ty: value_ty,
            fields: _,
        } = value
        {
            if let TyKind::Adt(expected_ty) = &ty.kind {
                // Check that the value type matches the expected type
                if value_ty != expected_ty {
                    self.errors.push(error_span(
                        format!("{value_desc} struct type mismatch"),
                        Label::new(ty.span).message("incompatible struct types"),
                    ));
                    return false;
                }

                // Check that it's actually a struct
                let def = self.ctx.definitions.get(*expected_ty);
                if let DefKind::Struct(_struct_ty) = &def.kind {
                    // TODO: Check field types match
                    return true;
                }
                self.errors.push(error_span(
                    format!("{value_desc} is not a struct type"),
                    Label::new(ty.span).message("expected struct type"),
                ));
                return false;
            }
        }

        match (&value, &ty.kind) {
            // Null is never valid in constants
            (Numeric::Null, _) => {
                // Null values usually indicate an earlier evaluation error
                // Only report if this looks like an actual null literal (rare in IDL)
                // This helps reduce cascading errors
                false
            }

            // String values
            (Numeric::String(_), TyKind::String { .. }) => true,
            (Numeric::String(_), _) => {
                self.errors.push(error_span(
                    format!("{value_desc} has string value but type is not string"),
                    Label::new(ty.span).message("expected string type"),
                ));
                false
            }

            // Boolean values
            (Numeric::Bool(_), TyKind::Primitive(PrimitiveTy::Bool)) => true,
            (Numeric::Bool(_), _) => {
                self.errors.push(error_span(
                    format!("{value_desc} has boolean value but type is not boolean"),
                    Label::new(ty.span).message("expected boolean type"),
                ));
                false
            }

            // Enum values - check that the numeric value is compatible with the enum's underlying type
            (value, TyKind::Adt(type_id)) => {
                let def = self.ctx.definitions.get(*type_id);
                if let DefKind::Enum(enum_ty) = &def.kind {
                    // Check against the enum's underlying type
                    return self.check_numeric_type(value, &enum_ty.ty, value_desc);
                }
                // Not an enum, report type mismatch
                self.errors.push(error_span(
                    format!("{value_desc} value type does not match declared type"),
                    Label::new(ty.span).message("type mismatch"),
                ));
                false
            }

            // Character values
            (Numeric::Char(_), TyKind::Primitive(PrimitiveTy::Char)) => true,
            (Numeric::Char(_c), TyKind::Primitive(PrimitiveTy::WChar)) => {
                // Check if char fits in wchar
                true
            }
            (Numeric::Char(_), _) => {
                self.errors.push(error_span(
                    format!("{value_desc} has character value but type is not char/wchar"),
                    Label::new(ty.span).message("expected character type"),
                ));
                false
            }

            // Integer values - check range
            (Numeric::Int8(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::Octet(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::Int16(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::UInt16(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::Int32(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::UInt32(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), prim, value_desc, ty.span)
            }
            (Numeric::Int64(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(*v, prim, value_desc, ty.span)
            }
            (Numeric::UInt64(v), TyKind::Primitive(prim)) => {
                self.check_uint_fits(*v, prim, value_desc, ty.span)
            }

            // Float values
            (Numeric::Float(_), TyKind::Primitive(PrimitiveTy::Float32)) => true,
            (Numeric::Float(_v), TyKind::Primitive(PrimitiveTy::Float64)) => true, // float promotes to double
            (Numeric::Double(_), TyKind::Primitive(PrimitiveTy::Float64)) => true,
            (Numeric::Double(_), TyKind::Primitive(PrimitiveTy::Float128)) => true, // double promotes to long double
            (Numeric::Float(_) | Numeric::Double(_), _) => {
                self.errors.push(error_span(
                    format!("{value_desc} has floating-point value but type is not float/double"),
                    Label::new(ty.span).message("expected floating-point type"),
                ));
                false
            }

            // Constant references
            (Numeric::Const(id), _expected_ty) => {
                let const_def = self.ctx.definitions.get(*id);
                if let DefKind::Const(const_ty) = &const_def.kind {
                    // Check if the referenced constant's type matches
                    self.check_type_compatible(&const_ty.ty, ty, value_desc)
                } else {
                    self.errors.push(error_span(
                        format!("{value_desc} references a non-constant definition"),
                        Label::new(ty.span).message("expected constant reference"),
                    ));
                    false
                }
            }

            // Array/sequence/map/union values
            (
                Numeric::Array { .. }
                | Numeric::Sequence { .. }
                | Numeric::Map { .. }
                | Numeric::Union { .. },
                _,
            ) => {
                // TODO: Implement complex type checking
                true
            }

            // Type mismatch
            _ => {
                // Don't report generic type mismatch for Null values from evaluation errors
                if !matches!(value, Numeric::Null) {
                    self.errors.push(error_span(
                        format!("{value_desc} value type does not match declared type"),
                        Label::new(ty.span).message("type mismatch"),
                    ));
                }
                false
            }
        }
    }

    /// Checks if an integer value fits in a primitive type.
    fn check_int_fits(
        &mut self,
        value: i64,
        prim: &PrimitiveTy,
        value_desc: &str,
        span: ic_syntax::Span,
    ) -> bool {
        let fits = match prim {
            PrimitiveTy::Bool => value == 0 || value == 1,
            PrimitiveTy::Char => (0..=127).contains(&value),
            PrimitiveTy::WChar => (0..=0xFFFF).contains(&value),
            PrimitiveTy::Int8 => (-128..=127).contains(&value),
            PrimitiveTy::UInt8 => (0..=255).contains(&value),
            PrimitiveTy::Int16 => (-32768..=32767).contains(&value),
            PrimitiveTy::UInt16 => (0..=65535).contains(&value),
            PrimitiveTy::Int32 => (-2_147_483_648..=2_147_483_647).contains(&value),
            PrimitiveTy::UInt32 => (0..=4_294_967_295).contains(&value),
            PrimitiveTy::Int64 => true, // Always fits
            PrimitiveTy::UInt64 => value >= 0,
            PrimitiveTy::Float32 | PrimitiveTy::Float64 | PrimitiveTy::Float128 => true, // Can convert to float
            PrimitiveTy::Void => false,
        };

        if !fits {
            self.errors.push(error_span(
                format!(
                    "{} value {} does not fit in type {}",
                    value_desc,
                    value,
                    prim.name()
                ),
                Label::new(span).message("value out of range"),
            ));
        }

        fits
    }

    /// Checks if an unsigned integer value fits in a primitive type.
    fn check_uint_fits(
        &mut self,
        value: u64,
        prim: &PrimitiveTy,
        value_desc: &str,
        span: ic_syntax::Span,
    ) -> bool {
        let fits = match prim {
            PrimitiveTy::UInt64 => true,
            PrimitiveTy::Int64 => i64::try_from(value).is_ok(),
            _ => {
                // For smaller types, delegate to check_int_fits if it fits in i64
                if let Ok(v) = i64::try_from(value) {
                    return self.check_int_fits(v, prim, value_desc, span);
                }
                false
            }
        };

        if !fits {
            self.errors.push(error_span(
                format!(
                    "{} value {} does not fit in type {}",
                    value_desc,
                    value,
                    prim.name()
                ),
                Label::new(span).message("value out of range"),
            ));
        }

        fits
    }

    /// Checks if two types are compatible (for constant references).
    fn check_type_compatible(&mut self, from_ty: &Ty, to_ty: &Ty, value_desc: &str) -> bool {
        match (&from_ty.kind, &to_ty.kind) {
            // Same primitive types
            (TyKind::Primitive(p1), TyKind::Primitive(p2)) if p1 == p2 => true,

            // Numeric promotions (e.g., int32 to int64)
            (TyKind::Primitive(_from), TyKind::Primitive(_to)) => {
                // TODO: Implement proper numeric promotion rules
                false
            }

            // Same ADT
            (TyKind::Adt(id1), TyKind::Adt(id2)) if id1 == id2 => true,

            // Same string types
            (TyKind::String { wide: w1, .. }, TyKind::String { wide: w2, .. }) if w1 == w2 => true,

            // Arrays with same element type
            (TyKind::Array { ty: ty1, .. }, TyKind::Array { ty: ty2, .. }) => {
                self.check_type_compatible(ty1, ty2, value_desc)
            }

            // Sequences with same element type
            (TyKind::Sequence { ty: ty1, .. }, TyKind::Sequence { ty: ty2, .. }) => {
                self.check_type_compatible(ty1, ty2, value_desc)
            }

            // Maps with same key and element types
            (
                TyKind::Map {
                    key: k1, elem: e1, ..
                },
                TyKind::Map {
                    key: k2, elem: e2, ..
                },
            ) => {
                self.check_type_compatible(k1, k2, value_desc)
                    && self.check_type_compatible(e1, e2, value_desc)
            }

            _ => {
                self.errors.push(error_span(
                    format!("{value_desc} type mismatch"),
                    Label::new(to_ty.span).message("incompatible types"),
                ));
                false
            }
        }
    }

    /// Type checks a constant definition.
    fn check_const(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Const(const_ty) = &def.kind {
            let value_desc = format!("constant `{}`", def.ident.name);
            self.check_numeric_type(&const_ty.value, &const_ty.ty, &value_desc);
        }
    }

    /// Type checks enum field values.
    fn check_enum(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Enum(enum_ty) = &def.kind {
            // Determine the underlying type
            let underlying_prim = match &enum_ty.ty.kind {
                TyKind::Primitive(p) => p,
                _ => {
                    // Should have been caught in validation
                    return;
                }
            };

            for field in &enum_ty.fields {
                let value_desc = format!("enum field `{}::{}`", def.ident.name, field.ident.name);
                self.check_int_fits(
                    i64::try_from(field.value).unwrap(),
                    underlying_prim,
                    &value_desc,
                    field.ident.span,
                );
            }
        }
    }

    /// Type checks bitmask flag values.
    fn check_bitmask(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Bitmask(bitmask_ty) = &def.kind {
            // Determine the underlying type
            let underlying_prim = match &bitmask_ty.ty.kind {
                TyKind::Primitive(p) => p,
                _ => {
                    // Should have been caught in validation
                    return;
                }
            };

            for flag in &bitmask_ty.flags {
                let value_desc = format!("bitmask flag `{}::{}`", def.ident.name, flag.ident.name);
                self.check_int_fits(
                    i64::try_from(flag.value).unwrap(),
                    underlying_prim,
                    &value_desc,
                    flag.ident.span,
                );
            }
        }
    }

    /// Type checks union case labels.
    fn check_union(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Union(union_ty) = &def.kind {
            for variant in &union_ty.variants {
                for label in &variant.labels {
                    let value_desc = format!(
                        "union case label for variant `{}::{}`",
                        def.ident.name, variant.ident.name
                    );
                    self.check_numeric_type(label, &union_ty.disc, &value_desc);
                }
            }
        }
    }

    /// Type checks all definitions.
    fn check_all(&mut self, order: &[DefId]) {
        for &id in order {
            let def = self.ctx.definitions.get(id);

            match &def.kind {
                DefKind::Const(_) => self.check_const(id),
                DefKind::Enum(_) => self.check_enum(id),
                DefKind::Bitmask(_) => self.check_bitmask(id),
                DefKind::Union(_) => self.check_union(id),
                _ => {}
            }
        }
    }
}

/// Type checks all definitions in the HIR.
pub fn typecheck_hir(ctx: &Context, order: &[DefId]) -> Vec<Diag> {
    let mut checker = TypeChecker::new(ctx);
    checker.check_all(order);
    checker.errors
}
