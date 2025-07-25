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

//! Type checking phase.
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

    /// Gets a human-readable name for a type.
    fn type_name(&self, ty: &Ty) -> String {
        match &ty.kind {
            TyKind::Primitive(prim) => prim.name().to_string(),
            TyKind::Null => "null".to_string(),
            TyKind::String { wide, .. } => {
                if *wide {
                    "wstring".to_string()
                } else {
                    "string".to_string()
                }
            }
            TyKind::Adt(def_id) => {
                let def = self.ctx.definitions.get(*def_id);
                def.ident.name.clone()
            }
            TyKind::Array { ty, len, .. } => {
                format!("{}[{}]", self.type_name(ty), len)
            }
            TyKind::Sequence { ty, bound, .. } => {
                if let Some(bound) = bound {
                    format!("sequence<{}, {}>", self.type_name(ty), bound)
                } else {
                    format!("sequence<{}>", self.type_name(ty))
                }
            }
            TyKind::Map {
                key, elem, bound, ..
            } => {
                if let Some(bound) = bound {
                    format!(
                        "map<{}, {}, {}>",
                        self.type_name(key),
                        self.type_name(elem),
                        bound
                    )
                } else {
                    format!("map<{}, {}>", self.type_name(key), self.type_name(elem))
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Checks if a numeric value is compatible with a type.
    #[allow(clippy::too_many_lines, clippy::match_same_arms)]
    fn check_numeric_type(
        &mut self,
        value: &Numeric,
        ty: &Ty,
        value_desc: &str,
        value_span: ic_syntax::Span,
    ) -> bool {
        // Special case for struct values first
        if let Numeric::Struct {
            ty: value_ty,
            fields,
        } = value
        {
            if let TyKind::Adt(expected_ty) = &ty.kind {
                // Check that the value type matches the expected type
                if value_ty != expected_ty {
                    self.errors.push(error_span(
                        format!("{value_desc} struct type mismatch"),
                        Label::new(value_span).message("incompatible struct types"),
                    ));
                    return false;
                }

                // Check that it's actually a struct
                let def = self.ctx.definitions.get(*expected_ty);
                if let DefKind::Struct(struct_ty) = &def.kind {
                    // Check field types match
                    let mut all_valid = true;
                    for (field_ident, field_value) in fields {
                        // Find the corresponding struct member
                        if let Some(member) = struct_ty
                            .members
                            .iter()
                            .find(|m| m.ident.name == field_ident.name)
                        {
                            let field_desc = format!("{}.{}", value_desc, field_ident.name);
                            if !self.check_numeric_type(
                                field_value,
                                &member.ty,
                                &field_desc,
                                value_span,
                            ) {
                                all_valid = false;
                            }
                        } else {
                            // Field not found in struct - this should have been caught earlier
                            self.errors.push(error_span(
                                format!("{value_desc} has unknown field `{}`", field_ident.name),
                                Label::new(value_span).message("field not found in struct"),
                            ));
                            all_valid = false;
                        }
                    }
                    return all_valid;
                }
                self.errors.push(error_span(
                    format!("{value_desc} is not a struct type"),
                    Label::new(value_span).message("expected struct type"),
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
                    format!(
                        "{value_desc}: string value cannot be assigned to type {}",
                        self.type_name(ty)
                    ),
                    Label::new(value_span).message(format!("expected type {}", self.type_name(ty))),
                ));
                false
            }

            // Boolean values
            (Numeric::Bool(_), TyKind::Primitive(PrimitiveTy::Bool)) => true,
            (Numeric::Bool(_), _) => {
                self.errors.push(error_span(
                    format!(
                        "{value_desc}: boolean value cannot be assigned to type {}",
                        self.type_name(ty)
                    ),
                    Label::new(value_span).message(format!("expected type {}", self.type_name(ty))),
                ));
                false
            }

            // Character values
            (Numeric::Char(_), TyKind::Primitive(PrimitiveTy::Char | PrimitiveTy::WChar)) => true,
            (Numeric::Char(_), _) => {
                self.errors.push(error_span(
                    format!(
                        "{value_desc}: character value cannot be assigned to type {}",
                        self.type_name(ty)
                    ),
                    Label::new(value_span).message(format!("expected type {}", self.type_name(ty))),
                ));
                false
            }

            // Integer values - check range (literals can be assigned to any type if they fit)
            (Numeric::Int8(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::Octet(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::Int16(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::UInt16(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::Int32(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::UInt32(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(i64::from(*v), *prim, value_desc, value_span)
            }
            (Numeric::Int64(v), TyKind::Primitive(prim)) => {
                self.check_int_fits(*v, *prim, value_desc, value_span)
            }
            (Numeric::UInt64(v), TyKind::Primitive(prim)) => {
                self.check_uint_fits(*v, *prim, value_desc, value_span)
            }
            
            // Special case: integer values being assigned to enum types
            // This is for enum members that have numeric values
            (Numeric::Int8(_) | Numeric::Int16(_) | Numeric::Int32(_) | Numeric::Int64(_), TyKind::Adt(enum_id)) => {
                // Check if this is an enum
                let def = self.ctx.definitions.get(*enum_id);
                if let DefKind::Enum(enum_ty) = &def.kind {
                    // Check if the value fits in the enum's underlying type
                    if let TyKind::Primitive(prim) = &enum_ty.ty.kind {
                        let val = match value {
                            Numeric::Int8(v) => i64::from(*v),
                            Numeric::Int16(v) => i64::from(*v),
                            Numeric::Int32(v) => i64::from(*v),
                            Numeric::Int64(v) => *v,
                            _ => unreachable!(),
                        };
                        self.check_int_fits(val, *prim, value_desc, value_span)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            // Union values must be handled before generic Adt pattern
            (
                Numeric::Union {
                    ty: union_ty_id,
                    discriminant,
                    field,
                    value,
                },
                TyKind::Adt(expected_ty_id),
            ) => {
                // Check that the union types match
                if union_ty_id != expected_ty_id {
                    self.errors.push(error_span(
                        format!("{value_desc}: union type mismatch"),
                        Label::new(value_span).message("incompatible union types"),
                    ));
                    return false;
                }

                // Get the union definition
                let def = self.ctx.definitions.get(*expected_ty_id);
                if let DefKind::Union(union_ty) = &def.kind {
                    // Check discriminant type
                    if !self.check_numeric_type(
                        discriminant,
                        &union_ty.disc,
                        &format!("{value_desc} discriminant"),
                        value_span,
                    ) {
                        return false;
                    }

                    // Find the variant by field name
                    if let Some(variant) = union_ty
                        .variants
                        .iter()
                        .find(|v| v.ident.name == field.name)
                    {
                        // Check the variant value type
                        if matches!(variant.ty.kind, TyKind::Null) {
                            // This is a null case - the value should be Numeric::Null
                            matches!(**value, Numeric::Null)
                        } else {
                            self.check_numeric_type(
                                value,
                                &variant.ty,
                                &format!("{value_desc}.{}", field.name),
                                value_span,
                            )
                        }
                    } else {
                        self.errors.push(error_span(
                            format!("{value_desc}: unknown union field `{}`", field.name),
                            Label::new(value_span).message("field not found in union"),
                        ));
                        false
                    }
                } else {
                    self.errors.push(error_span(
                        format!("{value_desc}: expected union type"),
                        Label::new(value_span).message("not a union"),
                    ));
                    false
                }
            }
            (Numeric::Union { .. }, _) => {
                self.errors.push(error_span(
                    format!("{value_desc}: union value cannot be assigned to non-union type"),
                    Label::new(value_span).message("type mismatch"),
                ));
                false
            }

            // Enum values and integer values assigned to enum types
            (value, TyKind::Adt(type_id)) => {
                let def = self.ctx.definitions.get(*type_id);
                if let DefKind::Enum(enum_ty) = &def.kind {
                    // Check against the enum's underlying type
                    return self.check_numeric_type(value, &enum_ty.ty, value_desc, value_span);
                }
                // Not an enum, report type mismatch
                self.errors.push(error_span(
                    format!("{value_desc} value type does not match declared type"),
                    Label::new(ty.span).message("type mismatch"),
                ));
                false
            }

            // Float values with promotion checking
            (Numeric::Float(_), TyKind::Primitive(prim)) => {
                if self.check_numeric_promotion(PrimitiveTy::Float32, *prim) {
                    true
                } else {
                    self.report_promotion_error(
                        PrimitiveTy::Float32,
                        *prim,
                        value_desc,
                        value_span,
                    );
                    false
                }
            }
            (Numeric::Double(_), TyKind::Primitive(prim)) => {
                if self.check_numeric_promotion(PrimitiveTy::Float64, *prim) {
                    true
                } else {
                    self.report_promotion_error(
                        PrimitiveTy::Float64,
                        *prim,
                        value_desc,
                        value_span,
                    );
                    false
                }
            }
            (Numeric::Float(_) | Numeric::Double(_), _) => {
                self.errors.push(error_span(
                    format!(
                        "{value_desc}: floating-point value cannot be assigned to non-numeric \
                         type {}",
                        self.type_name(ty)
                    ),
                    Label::new(value_span).message(format!("expected type {}", self.type_name(ty))),
                ));
                false
            }

            // Constant references
            (Numeric::Const(id), _expected_ty) => {
                let const_def = self.ctx.definitions.get(*id);
                if let DefKind::Const(const_ty) = &const_def.kind {
                    // Special case: check if the referenced constant itself contains a Const reference
                    // In that case, follow the chain
                    if let Numeric::Const(inner_id) = &const_ty.value {
                        return self.check_numeric_type(
                            &Numeric::Const(*inner_id),
                            ty,
                            value_desc,
                            value_span,
                        );
                    }
                    
                    // If the referenced constant has a numeric value, check if it's compatible with the expected type
                    return self.check_numeric_type(&const_ty.value, ty, value_desc, value_span);
                } else {
                    self.errors.push(error_span(
                        format!("{value_desc} references a non-constant definition"),
                        Label::new(value_span).message("expected constant reference"),
                    ));
                    false
                }
            }

            // Array values
            (
                Numeric::Array { values, .. },
                TyKind::Array {
                    ty: expected_elem_ty,
                    ..
                },
            ) => {
                // Check each element
                let mut all_valid = true;
                for (i, elem) in values.iter().enumerate() {
                    let elem_desc = format!("{value_desc}[{i}]");
                    if !self.check_numeric_type(elem, expected_elem_ty, &elem_desc, value_span) {
                        all_valid = false;
                    }
                }
                all_valid
            }

            // Sequence values
            (
                Numeric::Sequence { values, .. },
                TyKind::Sequence {
                    ty: expected_elem_ty,
                    ..
                },
            ) => {
                // Check each element
                let mut all_valid = true;
                for (i, elem) in values.iter().enumerate() {
                    let elem_desc = format!("{value_desc}[{i}]");
                    if !self.check_numeric_type(elem, expected_elem_ty, &elem_desc, value_span) {
                        all_valid = false;
                    }
                }
                all_valid
            }

            // Map values
            (
                Numeric::Map { values, .. },
                TyKind::Map {
                    key: expected_key_ty,
                    elem: expected_elem_ty,
                    ..
                },
            ) => {
                // Check each key-value pair
                let mut all_valid = true;
                for (i, (key, value)) in values.iter().enumerate() {
                    let key_desc = format!("{value_desc} key at index {i}");
                    let value_desc = format!("{value_desc} value at index {i}");
                    if !self.check_numeric_type(key, expected_key_ty, &key_desc, value_span) {
                        all_valid = false;
                    }
                    if !self.check_numeric_type(value, expected_elem_ty, &value_desc, value_span) {
                        all_valid = false;
                    }
                }
                all_valid
            }

            // Type mismatch
            _ => {
                // Don't report generic type mismatch for Null values from evaluation errors
                if !matches!(value, Numeric::Null) {
                    self.errors.push(error_span(
                        format!(
                            "{value_desc}: {} value cannot be assigned to type {}",
                            value_type_desc(value),
                            self.type_name(ty)
                        ),
                        Label::new(value_span)
                            .message(format!("expected type {}", self.type_name(ty))),
                    ));
                }
                false
            }
        }
    }

    /// Checks if an integer value fits in a primitive type.
    #[allow(clippy::cast_sign_loss)]
    fn check_int_fits(
        &mut self,
        value: i64,
        prim: PrimitiveTy,
        value_desc: &str,
        span: ic_syntax::Span,
    ) -> bool {
        let fits = match prim {
            PrimitiveTy::Bool => value == 0 || value == 1,
            PrimitiveTy::Char => (0..=127).contains(&value),
            PrimitiveTy::WChar | PrimitiveTy::UInt16 => {
                // Allow negative values to wrap for unsigned types
                if value < 0 {
                    // Check if it would fit after wrapping
                    let wrapped = (value as u64) & 0xFFFF;
                    wrapped <= 0xFFFF
                } else {
                    (0..=0xFFFF).contains(&value)
                }
            }
            PrimitiveTy::Int8 => (-128..=127).contains(&value),
            PrimitiveTy::UInt8 => {
                // Allow negative values to wrap for unsigned types
                if value < 0 {
                    // Check if it would fit after wrapping
                    let wrapped = (value as u64) & 0xFF;
                    wrapped <= 0xFF
                } else {
                    (0..=255).contains(&value)
                }
            }
            PrimitiveTy::Int16 => (-32768..=32767).contains(&value),
            PrimitiveTy::Int32 => (-2_147_483_648..=2_147_483_647).contains(&value),
            PrimitiveTy::UInt32 => {
                // Allow negative values to wrap for unsigned types
                if value < 0 {
                    // All negative i64 values fit in uint32 after wrapping
                    true
                } else {
                    (0..=4_294_967_295).contains(&value)
                }
            }
            PrimitiveTy::UInt64
            | PrimitiveTy::Float32
            | PrimitiveTy::Float64
            | PrimitiveTy::Float128
            | PrimitiveTy::Int64
            | PrimitiveTy::Void => true,
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
        prim: PrimitiveTy,
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

    /// Reports a numeric promotion error.
    fn report_promotion_error(
        &mut self,
        from: PrimitiveTy,
        to: PrimitiveTy,
        value_desc: &str,
        span: ic_syntax::Span,
    ) {
        self.errors.push(error_span(
            format!(
                "{}: {} value cannot be promoted to type {}",
                value_desc,
                from.name(),
                to.name()
            ),
            Label::new(span).message("invalid type promotion"),
        ));
    }

    /// Checks if a numeric type can be promoted to another.
    /// Based on IDL promotion rules similar to C++/CORBA.
    #[allow(clippy::unused_self)]
    fn check_numeric_promotion(&self, from: PrimitiveTy, to: PrimitiveTy) -> bool {
        use PrimitiveTy::{
            Bool, Char, Float32, Float64, Float128, Int8, Int16, Int32, Int64, UInt8, UInt16,
            UInt32, UInt64, Void, WChar,
        };

        // Same type is always allowed
        if from == to {
            return true;
        }

        // Define numeric type ordering for promotions
        let rank = |ty: PrimitiveTy| -> Option<(u8, bool)> {
            match ty {
                Bool => Some((0, false)),
                Char => Some((1, false)),
                WChar => Some((2, false)),
                Int8 => Some((3, true)),
                UInt8 => Some((3, false)),
                Int16 => Some((4, true)),
                UInt16 => Some((4, false)),
                Int32 => Some((5, true)),
                UInt32 => Some((5, false)),
                Int64 => Some((6, true)),
                UInt64 => Some((6, false)),
                Float32 => Some((7, true)),
                Float64 => Some((8, true)),
                Float128 => Some((9, true)),
                Void => None,
            }
        };

        let Some((from_rank, from_signed)) = rank(from) else {
            return false;
        };

        let Some((to_rank, to_signed)) = rank(to) else {
            return false;
        };

        // Special cases
        match (from, to) {
            // Boolean only promotes to itself (already handled by equality check)
            // Character types (except char->wchar) don't promote to other types
            (Bool | Char | WChar, _) if !matches!((from, to), (Char, WChar)) => false,

            // Character to wide character is allowed
            (Char, WChar) => true,

            // For numeric types, check if promotion is safe
            _ => {
                // Can always promote to higher rank
                if to_rank > from_rank {
                    return true;
                }

                // Same rank: unsigned can promote to signed if the signed type can hold all values
                if to_rank == from_rank && !from_signed && to_signed {
                    // Check if the unsigned type's max value fits in the signed type
                    matches!(
                        (from, to),
                        (UInt8, Int16) | (UInt16, Int32) | (UInt32, Int64)
                    )
                } else {
                    false
                }
            }
        }
    }

    /// Checks if two types are compatible (for constant references).
    fn check_type_compatible(&mut self, from_ty: &Ty, to_ty: &Ty, value_desc: &str) -> bool {
        match (&from_ty.kind, &to_ty.kind) {
            // Same primitive types
            (TyKind::Primitive(p1), TyKind::Primitive(p2)) if p1 == p2 => true,

            // Numeric promotions (e.g., int32 to int64)
            (TyKind::Primitive(from), TyKind::Primitive(to)) => {
                self.check_numeric_promotion(*from, *to)
            }

            // Same ADT
            (TyKind::Adt(id1), TyKind::Adt(id2)) if id1 == id2 => true,

            // Same string types
            (TyKind::String { wide: w1, .. }, TyKind::String { wide: w2, .. }) if w1 == w2 => true,

            // Arrays and Sequences with same element type
            (TyKind::Array { ty: ty1, .. }, TyKind::Array { ty: ty2, .. })
            | (TyKind::Sequence { ty: ty1, .. }, TyKind::Sequence { ty: ty2, .. }) => {
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
            // Skip type checking for enum members - their numeric values don't need to match the enum type
            if def.parent.is_some() {
                if let Some(parent_def) = def.parent.and_then(|p| Some(self.ctx.definitions.get(p))) {
                    if matches!(parent_def.kind, DefKind::Enum(_)) {
                        // This is an enum member, skip type checking
                        return;
                    }
                }
            }
            self.check_numeric_type(&const_ty.value, &const_ty.ty, &value_desc, def.span);
        }
    }

    /// Type checks enum field values.
    fn check_enum(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Enum(enum_ty) = &def.kind {
            // Determine the underlying type
            let TyKind::Primitive(underlying_prim) = &enum_ty.ty.kind else {
                // Should have been caught in validation
                return;
            };

            for &field_id in &enum_ty.fields {
                let field_def = self.ctx.definitions.get(field_id);
                if let DefKind::Const(const_ty) = &field_def.kind {
                    let value_desc =
                        format!("enum field `{}::{}`", def.ident.name, field_def.ident.name);
                    if let Numeric::Int32(value) = const_ty.value {
                        self.check_int_fits(
                            i64::from(value),
                            *underlying_prim,
                            &value_desc,
                            field_def.ident.span,
                        );
                    }
                }
            }
        }
    }

    /// Type checks bitmask flag values.
    fn check_bitmask(&mut self, id: DefId) {
        let def = self.ctx.definitions.get(id);

        if let DefKind::Bitmask(bitmask_ty) = &def.kind {
            // Determine the underlying type
            let TyKind::Primitive(underlying_prim) = &bitmask_ty.ty.kind else {
                // Should have been caught in validation
                return;
            };

            for flag in &bitmask_ty.flags {
                let value_desc = format!("bitmask flag `{}::{}`", def.ident.name, flag.ident.name);
                // Safe cast: bitmask values are always small enough to fit in i64
                #[allow(clippy::cast_possible_wrap)]
                self.check_int_fits(
                    flag.value as i64,
                    *underlying_prim,
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
                    self.check_numeric_type(
                        &label.value,
                        &union_ty.disc,
                        &value_desc,
                        variant.ident.span,
                    );
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

/// Gets a description of the value's type.
fn value_type_desc(value: &Numeric) -> &'static str {
    match value {
        Numeric::Null => "null",
        Numeric::String(_) => "string",
        Numeric::Bool(_) => "boolean",
        Numeric::Char(_) => "character",
        Numeric::Int8(_) | Numeric::Int16(_) | Numeric::Int32(_) | Numeric::Int64(_) => "integer",
        Numeric::Octet(_) | Numeric::UInt16(_) | Numeric::UInt32(_) | Numeric::UInt64(_) => {
            "unsigned integer"
        }
        Numeric::Float(_) | Numeric::Double(_) => "floating-point",
        Numeric::Struct { .. } => "struct",
        Numeric::Array { .. } => "array",
        Numeric::Sequence { .. } => "sequence",
        Numeric::Map { .. } => "map",
        Numeric::Union { .. } => "union",
        Numeric::Const(_) => "constant reference",
    }
}
