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

//! Phase 3: Expression evaluation.
//!
//! This phase:
//! - Evaluates constant expressions using ic-expr
//! - Computes array bounds, sequence bounds, map bounds
//! - Assigns values to enum fields and bitmask flags
//! - Evaluates initializer expressions for constants
//!
//! At this point, all types are resolved, so we can properly evaluate expressions.

use ic_diagnostic::{Diag, Label, error_span};
use ic_expr::{Error as ExprError, GenericNumeric, Result as ExprResult};
use ic_syntax::{Expr, Item};

use crate::Context;
use crate::hir::{
    Ann, BitFlag, DefId, DefKind, Numeric, PrimitiveTy, Span, StructTy, Ty, TyKind, TypeId,
};
use crate::scope::ScopeId;

/// Literal type for ic-expr evaluation.
#[derive(Debug, Clone)]
struct IdlLiteral {
    #[allow(dead_code)]
    span: ic_syntax::Span,
    value: ic_syntax::LiteralValue,
}

/// Type alias for the generic numeric type from ic-expr.
type EvalNumeric = GenericNumeric;

/// Convert from `GenericNumeric` to HIR `Numeric` type
fn to_hir_numeric(val: GenericNumeric) -> Numeric {
    match val {
        GenericNumeric::Bool(v) => Numeric::Bool(v),
        GenericNumeric::Char(v) => Numeric::Char(v),
        GenericNumeric::Int8(v) => Numeric::Int8(v),
        GenericNumeric::UInt8(v) => Numeric::Octet(v),
        GenericNumeric::Int16(v) => Numeric::Int16(v),
        GenericNumeric::UInt16(v) => Numeric::UInt16(v),
        GenericNumeric::Int32(v) => Numeric::Int32(v),
        GenericNumeric::UInt32(v) => Numeric::UInt32(v),
        GenericNumeric::Int64(v) => Numeric::Int64(v),
        GenericNumeric::UInt64(v) => Numeric::UInt64(v),
        GenericNumeric::Float(v) => Numeric::Float(v),
        GenericNumeric::Double(v) => Numeric::Double(v),
    }
}

/// Try to convert from HIR `Numeric` type to `GenericNumeric`
fn from_hir_numeric(n: &Numeric) -> Option<GenericNumeric> {
    match n {
        Numeric::Bool(v) => Some(GenericNumeric::Bool(*v)),
        Numeric::Char(v) => Some(GenericNumeric::Char(*v)),
        Numeric::Int8(v) => Some(GenericNumeric::Int8(*v)),
        Numeric::Octet(v) => Some(GenericNumeric::UInt8(*v)),
        Numeric::Int16(v) => Some(GenericNumeric::Int16(*v)),
        Numeric::UInt16(v) => Some(GenericNumeric::UInt16(*v)),
        Numeric::Int32(v) => Some(GenericNumeric::Int32(*v)),
        Numeric::UInt32(v) => Some(GenericNumeric::UInt32(*v)),
        Numeric::Int64(v) => Some(GenericNumeric::Int64(*v)),
        Numeric::UInt64(v) => Some(GenericNumeric::UInt64(*v)),
        Numeric::Float(v) => Some(GenericNumeric::Float(*v)),
        Numeric::Double(v) => Some(GenericNumeric::Double(*v)),
        _ => None, // Non-arithmetic variants
    }
}

/// Context for evaluating IDL expressions.
struct IdlEvalContext<'a> {
    ctx: &'a Context,
    config: ic_expr::EvalConfig,
    #[allow(dead_code)]
    errors: &'a mut Vec<Diag>,
    current_scope: crate::scope::ScopeId,
}

impl ic_expr::EvalContext<IdlLiteral> for IdlEvalContext<'_> {
    type Value = EvalNumeric;

    fn eval_literal(&mut self, lit: &IdlLiteral) -> ExprResult<Self::Value> {
        match &lit.value {
            ic_syntax::LiteralValue::Null => Err(ExprError::Custom(
                "null literals not supported in constant expressions".to_string(),
            )),
            ic_syntax::LiteralValue::Bool(b) => Ok(GenericNumeric::Bool(*b)),
            ic_syntax::LiteralValue::Char(c) => Ok(GenericNumeric::Char(*c)),
            ic_syntax::LiteralValue::Int(i) => {
                // TODO: Handle different integer types based on suffix
                // For now, assume Int32
                match i32::try_from(*i) {
                    Ok(v) => Ok(GenericNumeric::Int32(v)),
                    Err(_) =>
                    {
                        #[allow(clippy::cast_possible_wrap)]
                        Ok(GenericNumeric::Int64(*i as i64))
                    }
                }
            }
            ic_syntax::LiteralValue::Float(f) => {
                // TODO: Handle float vs double based on suffix
                #[allow(clippy::cast_possible_truncation)]
                Ok(GenericNumeric::Float(*f as f32))
            }
            ic_syntax::LiteralValue::String(_s) => {
                // String literals are not supported in numeric expressions
                Err(ExprError::Custom(
                    "string literals cannot be used in arithmetic expressions".to_string(),
                ))
            }
        }
    }

    fn lookup_var(&mut self, name: &str) -> ExprResult<Self::Value> {
        // Check if name starts with :: for global scope resolution
        let (start_scope, name_without_prefix) = if let Some(stripped) = name.strip_prefix("::") {
            (self.ctx.scopes.root(), stripped)
        } else {
            (self.current_scope, name)
        };

        let parts: Vec<&str> = name_without_prefix.split("::").collect();

        // Try to resolve the path using the scope tree
        if let Some(def_id) = self.ctx.scopes.resolve_path(start_scope, &parts) {
            let def = self.ctx.definitions.get(def_id);

            // Check if it's a constant
            if let DefKind::Const(const_ty) = &def.kind {
                if let Some(eval_num) = from_hir_numeric(&const_ty.value) {
                    return Ok(eval_num);
                }
            }
        }

        // Handle enum field references (e.g., "Color::GREEN" or "MyEnum::VALUE")
        if parts.len() >= 2 {
            // Try different ways to split the path into enum + field
            for split_pos in 1..parts.len() {
                let enum_parts = &parts[..split_pos];
                let field_name = parts[split_pos];

                // Only handle single field name after the enum
                if split_pos == parts.len() - 1 {
                    // Try to resolve the enum path
                    if let Some(enum_id) = self.ctx.scopes.resolve_path(start_scope, enum_parts) {
                        let enum_def = self.ctx.definitions.get(enum_id);
                        if let DefKind::Enum(enum_ty) = &enum_def.kind {
                            // Look for the field
                            for field in &enum_ty.fields {
                                if field.ident.name == field_name {
                                    #[allow(clippy::cast_possible_truncation)]
                                    return Ok(GenericNumeric::Int32(field.value as i32));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle unscoped enum enumerator (e.g., just "RED")
        if parts.len() == 1 {
            let enumerator = parts[0];

            // Get all visible enums from current scope
            let visible_enums = self
                .ctx
                .scopes
                .get_visible_enums(self.current_scope, &self.ctx.definitions);

            // Check each enum for this enumerator
            for enum_id in visible_enums {
                let enum_def = self.ctx.definitions.get(enum_id);
                if let DefKind::Enum(enum_ty) = &enum_def.kind {
                    for field in &enum_ty.fields {
                        if field.ident.name == enumerator {
                            #[allow(clippy::cast_possible_truncation)]
                            return Ok(GenericNumeric::Int32(field.value as i32));
                        }
                    }
                }
            }
        }

        // Handle module-qualified enumerator (e.g., "foo::bar::RED")
        // This is when someone uses a module path directly to an enumerator
        if parts.len() >= 2 {
            let module_parts = &parts[..parts.len() - 1];
            let enumerator = parts[parts.len() - 1];

            // Try to resolve the module path
            if let Some(module_id) = self
                .ctx
                .scopes
                .resolve_path(self.current_scope, module_parts)
            {
                // Find the scope for this module
                for scope in &self.ctx.scopes.scopes {
                    if scope.def_id == Some(module_id) {
                        let scope_id = ScopeId(
                            self.ctx
                                .scopes
                                .scopes
                                .iter()
                                .position(|s| std::ptr::eq(s, scope))
                                .unwrap(),
                        );
                        // Find enums in this module that have the enumerator
                        let enums = self.ctx.scopes.find_enums_with_enumerator(
                            scope_id,
                            enumerator,
                            &self.ctx.definitions,
                        );
                        if let Some(&enum_id) = enums.first() {
                            let enum_def = self.ctx.definitions.get(enum_id);
                            if let DefKind::Enum(enum_ty) = &enum_def.kind {
                                for field in &enum_ty.fields {
                                    if field.ident.name == enumerator {
                                        #[allow(clippy::cast_possible_truncation)]
                                        return Ok(GenericNumeric::Int32(field.value as i32));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(ExprError::Custom(format!(
            "undefined constant or enum value `{name}`"
        )))
    }

    fn config(&self) -> ic_expr::EvalConfig {
        self.config
    }
}

/// Converts a path to its string representation.
fn path_to_string(path: &ic_syntax::Path) -> String {
    let segments = path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("::");

    // Preserve leading :: for global scope resolution
    if path.leading_colons.is_some() {
        format!("::{segments}")
    } else {
        segments
    }
}

/// Extract name from const def
#[allow(dead_code)]
fn extract_const_name(def: &ic_syntax::ConstDef) -> &str {
    match &def.decl {
        ic_syntax::Declarator::Simple(ident) => &ident.name,
        ic_syntax::Declarator::Array(arr) => &arr.ident.name,
    }
}

/// Converts an ic-syntax expression to an ic-expr expression.
fn convert_expr(expr: &ic_syntax::Expr) -> Result<ic_expr::Expr<IdlLiteral>, String> {
    match expr {
        Expr::Literal(lit) => Ok(ic_expr::Expr::Lit(IdlLiteral {
            span: lit.span,
            value: lit.value.clone(),
        })),

        Expr::Path(path) => {
            // Convert path to variable name
            Ok(ic_expr::Expr::Var(path_to_string(path)))
        }

        Expr::Unary(unary) => {
            let op = match unary.op.kind {
                ic_syntax::OpKind::Add => ic_expr::Op::Add,
                ic_syntax::OpKind::Sub => ic_expr::Op::Sub,
                ic_syntax::OpKind::Not => ic_expr::Op::Not,
                _ => return Err(format!("unsupported unary operator: {:?}", unary.op.kind)),
            };

            let inner = convert_expr(&unary.expr)?;
            Ok(ic_expr::Expr::Unary(Box::new(ic_expr::Unary {
                op,
                expr: inner,
            })))
        }

        Expr::Binary(binary) => {
            let op = match binary.op.kind {
                ic_syntax::OpKind::Add => ic_expr::Op::Add,
                ic_syntax::OpKind::Sub => ic_expr::Op::Sub,
                ic_syntax::OpKind::Multiply => ic_expr::Op::Mul,
                ic_syntax::OpKind::Divide => ic_expr::Op::Div,
                ic_syntax::OpKind::Modulo => ic_expr::Op::Mod,
                ic_syntax::OpKind::And => ic_expr::Op::BitAnd,
                ic_syntax::OpKind::Or => ic_expr::Op::BitOr,
                ic_syntax::OpKind::Xor => ic_expr::Op::BitXor,
                ic_syntax::OpKind::Lshift => ic_expr::Op::LShift,
                ic_syntax::OpKind::Rshift => ic_expr::Op::RShift,
                ic_syntax::OpKind::Not => {
                    return Err(format!("unsupported binary operator: {:?}", binary.op.kind));
                }
            };

            let lhs = convert_expr(&binary.lhs)?;
            let rhs = convert_expr(&binary.rhs)?;
            Ok(ic_expr::Expr::Binary(Box::new(ic_expr::Binary {
                lhs,
                op,
                rhs,
            })))
        }

        Expr::Group(group) => convert_expr(&group.expr),

        Expr::InitList(_init) => {
            // For now, return a placeholder - proper evaluation needs type context
            // This will be handled in evaluate_const where we have the expected type
            Err("initializer lists need type context for evaluation".to_string())
        }
    }
}

/// Evaluates expressions in the HIR.
pub struct ExpressionEvaluator<'a> {
    ctx: &'a mut Context,
    errors: Vec<Diag>,
    current_scope: crate::scope::ScopeId,
    /// Track which definitions have been evaluated to avoid duplicate evaluation
    evaluated: std::collections::HashSet<DefId>,
}

impl<'a> ExpressionEvaluator<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            errors: Vec::new(),
            current_scope: root_scope,
            evaluated: std::collections::HashSet::new(),
        }
    }

    /// Evaluates an expression to a numeric value.
    fn eval_expr(&mut self, expr: &Expr) -> Numeric {
        // Convert to ic-expr format
        let ic_expr = match convert_expr(expr) {
            Ok(e) => e,
            Err(msg) => {
                self.errors.push(error_span(
                    msg,
                    Label::new(ic_syntax::util::expr_span(expr))
                        .message("cannot evaluate this expression"),
                ));
                return Numeric::Null;
            }
        };

        // Create evaluation context with overflow detection
        let mut eval_ctx = IdlEvalContext {
            ctx: self.ctx,
            config: ic_expr::EvalConfig {
                overflow: ic_expr::OverflowBehavior::Error,
                max_shift: 127,
            },
            errors: &mut self.errors,
            current_scope: self.current_scope,
        };

        // Evaluate the expression
        match ic_expr::eval(&ic_expr, &mut eval_ctx) {
            Ok(value) => {
                // Convert GenericNumeric to HIR Numeric
                to_hir_numeric(value)
            }
            Err(err) => {
                let msg = match err {
                    ExprError::DivisionByZero => "division by zero in constant expression",
                    ExprError::ModuloByZero => "modulo by zero in constant expression",
                    ExprError::Overflow(op_str) => {
                        self.errors.push(error_span(
                            format!("integer overflow in constant expression: {op_str}"),
                            Label::new(ic_syntax::util::expr_span(expr))
                                .message("overflow detected"),
                        ));
                        // Return Null to indicate error
                        return Numeric::Null;
                    }
                    ExprError::InvalidShift(amount) => {
                        self.errors.push(error_span(
                            format!("invalid shift amount: {amount}"),
                            Label::new(ic_syntax::util::expr_span(expr)).message("invalid shift"),
                        ));
                        return Numeric::Null;
                    }
                    ExprError::InvalidUnaryOp(op) => {
                        self.errors.push(error_span(
                            format!("invalid unary operator: {op:?}"),
                            Label::new(ic_syntax::util::expr_span(expr)).message("cannot apply"),
                        ));
                        return Numeric::Null;
                    }
                    ExprError::Custom(s) => {
                        let mut diag = error_span(
                            s.clone(),
                            Label::new(ic_syntax::util::expr_span(expr))
                                .message("evaluation error"),
                        );

                        // Add helpful notes for common errors
                        if s.contains("undefined constant") {
                            diag = diag.note("check that the name is spelled correctly");
                        } else if s.contains("string literals cannot be used") {
                            diag = diag.note(
                                "string literals can only be used in struct initialization or \
                                 string constants",
                            );
                        }

                        self.errors.push(diag);
                        return Numeric::Null;
                    }
                };

                self.errors.push(error_span(
                    msg,
                    Label::new(ic_syntax::util::expr_span(expr)).message("in this expression"),
                ));
                Numeric::Null
            }
        }
    }

    /// Converts a numeric value to match the declared type.
    /// This handles unsigned wrapping for negative values.
    fn convert_to_type(&mut self, value: Numeric, ty: &Ty) -> Numeric {
        match (&value, &ty.kind) {
            // Convert signed to unsigned with wrapping
            (Numeric::Int8(v), TyKind::Primitive(PrimitiveTy::UInt8)) => Numeric::Octet(*v as u8),
            (Numeric::Int16(v), TyKind::Primitive(PrimitiveTy::UInt16)) => {
                Numeric::UInt16(*v as u16)
            }
            (Numeric::Int32(v), TyKind::Primitive(PrimitiveTy::UInt32)) => {
                Numeric::UInt32(*v as u32)
            }
            (Numeric::Int64(v), TyKind::Primitive(PrimitiveTy::UInt64)) => {
                Numeric::UInt64(*v as u64)
            }

            // Convert between signed integer types
            (Numeric::Int32(v), TyKind::Primitive(PrimitiveTy::Int8)) => Numeric::Int8(*v as i8),
            (Numeric::Int32(v), TyKind::Primitive(PrimitiveTy::Int16)) => Numeric::Int16(*v as i16),
            (Numeric::Int32(v), TyKind::Primitive(PrimitiveTy::Int64)) => Numeric::Int64(*v as i64),

            // Keep the value as-is if types match or no conversion needed
            _ => value,
        }
    }

    /// Evaluates a bound expression to a usize.
    fn eval_bound(&mut self, expr: &Expr) -> usize {
        let result = self.eval_expr(expr);
        match result {
            Numeric::Int32(v) if v >= 0 => usize::try_from(v).unwrap_or(0),
            Numeric::UInt32(v) => usize::try_from(v).unwrap_or(0),
            Numeric::Int64(v) if v >= 0 => usize::try_from(v).unwrap_or(0),
            Numeric::UInt64(v) => usize::try_from(v).unwrap_or(0),
            _ => {
                self.errors.push(error_span(
                    "invalid bound expression",
                    Label::new(ic_syntax::util::expr_span(expr))
                        .message("bound must be a positive integer"),
                ));
                0
            }
        }
    }

    /// Evaluates a bound expression to a usize and returns its span.
    fn eval_bound_with_span(&mut self, expr: &Expr) -> (usize, Span) {
        let span = ic_syntax::util::expr_span(expr);
        let value = self.eval_bound(expr);
        (value, span)
    }

    /// Evaluates array bounds and returns values with spans.
    fn evaluate_array_bounds_with_spans(
        &mut self,
        bounds: &[ic_syntax::Expr],
    ) -> Vec<(usize, Span)> {
        bounds
            .iter()
            .map(|expr| self.eval_bound_with_span(expr))
            .collect()
    }

    /// Updates array bounds in a type using pre-evaluated bounds with spans.
    fn update_type_bounds_with_values_and_spans(ty: &mut Ty, evaluated_bounds: &[(usize, Span)]) {
        let mut current = ty;
        let mut bound_iter = evaluated_bounds.iter();

        while let TyKind::Array {
            ty: inner_ty,
            len,
            len_span,
        } = &mut current.kind
        {
            if let Some(&(bound_value, span)) = bound_iter.next() {
                *len = bound_value;
                *len_span = span;
            }
            current = inner_ty;
        }
    }

    /// Evaluates expressions in a struct definition.
    fn evaluate_struct(&mut self, id: DefId, def: &ic_syntax::StructDef) {
        // Collect array bounds for struct members
        let mut array_bounds = Vec::new();
        let mut member_idx = 0;

        for field in &def.members {
            for decl in &field.names {
                if let ic_syntax::Declarator::Array(arr) = decl {
                    array_bounds.push((member_idx, arr.bounds.clone()));
                }
                member_idx += 1;
            }
        }

        // Evaluate all bounds first with spans
        let evaluated_bounds: Vec<(usize, Vec<(usize, Span)>)> = array_bounds
            .into_iter()
            .map(|(idx, bounds)| {
                let evaluated = self.evaluate_array_bounds_with_spans(&bounds);
                (idx, evaluated)
            })
            .collect();

        // Process bounds and collect updated types
        let mut updates = Vec::new();
        let mut processed_indices = std::collections::HashSet::new();

        // First handle array bounds
        {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Struct(struct_ty) = &hir_def.kind {
                for (idx, bounds) in evaluated_bounds {
                    if let Some(member) = struct_ty.members.get(idx) {
                        let mut ty = member.ty.clone();
                        Self::update_type_bounds_with_values_and_spans(&mut ty, &bounds);
                        updates.push((idx, ty));
                        processed_indices.insert(idx);
                    }
                }
            }
        }

        // Then handle sequence/string/map bounds
        // First collect the member types and AST types
        let member_type_pairs: Vec<(usize, Ty, &ic_syntax::Type)> = {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Struct(struct_ty) = &hir_def.kind {
                let mut pairs = Vec::new();
                let mut member_idx = 0;
                for (field_idx, field) in def.members.iter().enumerate() {
                    for _decl in &field.names {
                        if let Some(member) = struct_ty.members.get(member_idx) {
                            // Only process if we haven't already handled this member's array bounds
                            if !processed_indices.contains(&member_idx) {
                                pairs.push((
                                    member_idx,
                                    member.ty.clone(),
                                    &def.members[field_idx].ty,
                                ));
                            }
                        }
                        member_idx += 1;
                    }
                }
                pairs
            } else {
                Vec::new()
            }
        };

        // Now evaluate bounds outside the borrow scope
        for (idx, mut ty, ast_ty) in member_type_pairs {
            self.evaluate_type_bounds(&mut ty, ast_ty);
            updates.push((idx, ty));
        }

        // Apply updates
        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Struct(struct_ty) = &mut hir_def.kind {
            for (idx, ty) in updates {
                if let Some(member) = struct_ty.members.get_mut(idx) {
                    member.ty = ty;
                }
            }
        }
    }

    /// Evaluates expressions in a union definition.
    fn evaluate_union(&mut self, id: DefId, def: &ic_syntax::UnionDef) {
        // First, evaluate all case labels and collect array bounds
        let mut all_labels = Vec::new();
        let mut array_bounds = Vec::new();

        for (idx, field) in def.fields.iter().enumerate() {
            let mut labels = Vec::new();

            for label in &field.labels {
                if let ic_syntax::Label::Case(expr) = label {
                    labels.push(self.eval_expr(expr));
                }
            }

            all_labels.push(labels);

            // Check if this field has array bounds to evaluate
            if let ic_syntax::UnionElement::Member(m) = &field.field {
                if let ic_syntax::Declarator::Array(arr) = &m.decl {
                    array_bounds.push((idx, arr.bounds.clone()));
                }
            }
        }

        // Evaluate all bounds first with spans
        let evaluated_bounds: Vec<(usize, Vec<(usize, Span)>)> = array_bounds
            .into_iter()
            .map(|(idx, bounds)| {
                let evaluated = self.evaluate_array_bounds_with_spans(&bounds);
                (idx, evaluated)
            })
            .collect();

        // Process bounds and collect updated types
        let mut type_updates = Vec::new();
        {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Union(union_ty) = &hir_def.kind {
                for (idx, bounds) in evaluated_bounds {
                    if let Some(variant) = union_ty.variants.get(idx) {
                        let mut ty = variant.ty.clone();
                        Self::update_type_bounds_with_values_and_spans(&mut ty, &bounds);
                        type_updates.push((idx, ty));
                    }
                }
            }
        }

        // Then update the HIR
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Union(union_ty) = &mut hir_def.kind {
            // Update labels
            for (idx, labels) in all_labels.into_iter().enumerate() {
                if let Some(variant) = union_ty.variants.get_mut(idx) {
                    variant.labels = labels;
                }
            }

            // Update array bounds
            for (idx, ty) in type_updates {
                if let Some(variant) = union_ty.variants.get_mut(idx) {
                    variant.ty = ty;
                }
            }
        }
    }

    /// Evaluates expressions in an exception definition.
    fn evaluate_exception(&mut self, id: DefId, def: &ic_syntax::ExceptDef) {
        // Collect array bounds for exception members
        let mut array_bounds = Vec::new();
        let mut member_idx = 0;

        for field in &def.members {
            for decl in &field.names {
                if let ic_syntax::Declarator::Array(arr) = decl {
                    array_bounds.push((member_idx, arr.bounds.clone()));
                }
                member_idx += 1;
            }
        }

        // Evaluate all bounds first with spans
        let evaluated_bounds: Vec<(usize, Vec<(usize, Span)>)> = array_bounds
            .into_iter()
            .map(|(idx, bounds)| {
                let evaluated = self.evaluate_array_bounds_with_spans(&bounds);
                (idx, evaluated)
            })
            .collect();

        // Process bounds and collect updated types
        let mut updates = Vec::new();
        {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Except(except_ty) = &hir_def.kind {
                for (idx, bounds) in evaluated_bounds {
                    if let Some(member) = except_ty.members.get(idx) {
                        let mut ty = member.ty.clone();
                        Self::update_type_bounds_with_values_and_spans(&mut ty, &bounds);
                        updates.push((idx, ty));
                    }
                }
            }
        }

        // Apply updates
        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Except(except_ty) = &mut hir_def.kind {
            for (idx, ty) in updates {
                if let Some(member) = except_ty.members.get_mut(idx) {
                    member.ty = ty;
                }
            }
        }
    }

    /// Evaluates enum values.
    fn evaluate_enum(&mut self, id: DefId, def: &ic_syntax::EnumDef) {
        // Check if this enum has already been evaluated
        if !self.evaluated.insert(id) {
            // Already evaluated, skip
            return;
        }

        // First evaluate all the values
        let mut field_values = Vec::new();
        let mut last_value = -1isize;

        for field in &def.fields {
            #[allow(clippy::cast_possible_wrap)]
            let value = if let Some(expr) = &field.value {
                self.eval_bound(expr) as isize
            } else {
                last_value + 1
            };

            last_value = value;
            field_values.push(value);
        }

        // Then update the existing enum fields with their evaluated values
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Enum(enum_ty) = &mut hir_def.kind {
            for (i, value) in field_values.into_iter().enumerate() {
                if i < enum_ty.fields.len() {
                    enum_ty.fields[i].value = value;
                }
            }
        }
    }

    /// Evaluates bitmask values.
    fn evaluate_bitmask(&mut self, id: DefId, def: &ic_syntax::BitmaskDef) {
        // Check if this bitmask has already been evaluated
        if !self.evaluated.insert(id) {
            // Already evaluated, skip
            return;
        }
        // Get existing annotations if they were already resolved
        let existing_annotations: Vec<Vec<Ann>> = {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Bitmask(bitmask_ty) = &hir_def.kind {
                bitmask_ty
                    .flags
                    .iter()
                    .map(|f| f.annotations.clone())
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Create the bitmask flags and evaluate their values
        let mut flags = Vec::new();
        let mut last_value = 0usize;

        for (i, bit) in def.bits.iter().enumerate() {
            let value = if let Some(expr) = &bit.value {
                // For bitmasks, the assignment operator specifies bit position, not value
                let position = self.eval_bound(expr);
                1 << position
            } else if last_value == 0 {
                1
            } else {
                last_value << 1
            };

            last_value = value;

            // Preserve existing annotations if available
            let annotations = if i < existing_annotations.len() {
                existing_annotations[i].clone()
            } else {
                Vec::new()
            };

            flags.push(BitFlag {
                ident: bit.ident.clone(),
                value,
                annotations,
            });
        }

        // Then update the HIR
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Bitmask(bitmask_ty) = &mut hir_def.kind {
            bitmask_ty.flags = flags;
        }
    }

    /// Evaluates a bitset definition.
    fn evaluate_bitset(&mut self, id: DefId, def: &ic_syntax::BitsetDef) {
        // Check if this bitset has already been evaluated
        if !self.evaluated.insert(id) {
            // Already evaluated, skip
            return;
        }
        // Get the current bitset definition
        let hir_def = self.ctx.definitions.get(id);

        // Get the current fields with their resolved types
        let mut updated_fields = if let DefKind::Bitset(bitset_ty) = &hir_def.kind {
            bitset_ty.fields.clone()
        } else {
            return;
        };

        // Update the sizes by evaluating the size expressions and assign default types
        for (i, field) in def.fields.iter().enumerate() {
            if let Some(updated_field) = updated_fields.get_mut(i) {
                // Evaluate the size
                updated_field.size = self.eval_bound(&field.size);

                // If the type is a placeholder (Any), assign the appropriate type based on size
                if matches!(updated_field.ty.kind, TyKind::Any) {
                    updated_field.ty = Self::default_bitfield_type(updated_field.size);
                }
            }
        }

        // Update the HIR with evaluated sizes and types
        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Bitset(bitset_ty) = &mut hir_def.kind {
            bitset_ty.fields = updated_fields;
        }
    }

    /// Determines the default type for a bitfield based on its size.
    /// Returns the smallest unsigned integer type that can hold the specified number of bits.
    fn default_bitfield_type(bits: usize) -> Ty {
        let prim = if bits == 1 {
            PrimitiveTy::Bool // Special case: 1-bit fields are booleans
        } else if bits <= 8 {
            PrimitiveTy::UInt8
        } else if bits <= 16 {
            PrimitiveTy::UInt16
        } else if bits <= 32 {
            PrimitiveTy::UInt32
        } else if bits <= 64 {
            PrimitiveTy::UInt64
        } else {
            // For fields larger than 64 bits, we might want to error or use a special type
            // For now, default to uint64
            PrimitiveTy::UInt64
        };

        Ty {
            kind: TyKind::Primitive(prim),
            span: Span::default(), // We don't have a good span for synthetic types
        }
    }

    /// Evaluates an alias (typedef) definition.
    fn evaluate_alias(&mut self, id: DefId, decl: &ic_syntax::Declarator) {
        if let ic_syntax::Declarator::Array(arr) = decl {
            // Evaluate the bounds with spans
            let evaluated_bounds = self.evaluate_array_bounds_with_spans(&arr.bounds);

            // Get the current type
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Alias(alias_ty) = &hir_def.kind {
                let mut ty = alias_ty.ty.clone();
                Self::update_type_bounds_with_values_and_spans(&mut ty, &evaluated_bounds);

                // Update the HIR with the evaluated type
                let hir_def = self.ctx.definitions.get_mut(id);
                if let DefKind::Alias(alias_ty) = &mut hir_def.kind {
                    alias_ty.ty = ty;
                }
            }
        }
    }

    /// Evaluates an expression that may contain string literals (for initializers)
    fn eval_init_expr(&mut self, expr: &Expr) -> Numeric {
        // Check if this is a string literal
        if let Expr::Literal(lit) = expr {
            match &lit.value {
                ic_syntax::LiteralValue::String(s) => return Numeric::String(s.clone()),
                ic_syntax::LiteralValue::Null => return Numeric::Null,
                _ => {}
            }
        }

        // Otherwise evaluate as normal expression
        self.eval_expr(expr)
    }

    /// Evaluates an expression that may contain nested init lists
    fn eval_init_expr_with_type(
        &mut self,
        expr: &Expr,
        expected_ty: &Ty,
        parent_type_id: DefId,
    ) -> Numeric {
        // Check if this is a string literal
        if let Expr::Literal(lit) = expr {
            match &lit.value {
                ic_syntax::LiteralValue::String(s) => return Numeric::String(s.clone()),
                ic_syntax::LiteralValue::Null => return Numeric::Null,
                _ => {}
            }
        }

        // Handle nested init lists
        if let Expr::InitList(init_list) = expr {
            match &expected_ty.kind {
                TyKind::Array { ty, len, .. } => {
                    // Use the parent type ID for nested arrays
                    return self.eval_array_init(
                        init_list,
                        ty.as_ref().clone(),
                        *len,
                        parent_type_id,
                    );
                }
                TyKind::Sequence { ty, .. } => {
                    return self.eval_sequence_init(init_list, ty.as_ref().clone(), parent_type_id);
                }
                TyKind::Map { key, elem, .. } => {
                    return self.eval_map_init(
                        init_list,
                        key.as_ref().clone(),
                        elem.as_ref().clone(),
                        parent_type_id,
                    );
                }
                _ => {
                    self.errors.push(error_span(
                        "unexpected initializer list",
                        Label::new(ic_syntax::util::expr_span(expr))
                            .message("initializer list not allowed here"),
                    ));
                    return Numeric::Null;
                }
            }
        }

        // Otherwise evaluate as normal expression
        self.eval_expr(expr)
    }

    /// Evaluates an initializer list for a struct type
    fn eval_struct_init(
        &mut self,
        init_list: &ic_syntax::InitList,
        struct_ty: &StructTy,
        type_id: TypeId,
    ) -> Numeric {
        let mut fields = Vec::new();

        // If all fields are named, use them directly
        let all_named = init_list.values.iter().all(|v| v.ident.is_some());

        if all_named {
            // Named initialization - match fields to struct member order
            for member in &struct_ty.members {
                // Find the corresponding initializer
                let init_value = init_list
                    .values
                    .iter()
                    .find(|v| {
                        v.ident
                            .as_ref()
                            .is_some_and(|i| i.name == member.ident.name)
                    })
                    .map(|v| self.eval_init_expr(&v.value));

                if let Some(value) = init_value {
                    fields.push((member.ident.clone(), value));
                } else {
                    // Field not provided in initializer
                    let diag = error_span(
                        format!(
                            "missing required field `{}` in struct initializer",
                            member.ident.name
                        ),
                        Label::new(member.ident.span).message("field is required here"),
                    )
                    .note("all struct fields must be initialized")
                    .note("add the missing field to the initializer list");

                    self.errors.push(diag);
                    return Numeric::Null;
                }
            }
        } else if init_list.values.iter().all(|v| v.ident.is_none()) {
            // Positional initialization - match with struct member order
            for (i, named_expr) in init_list.values.iter().enumerate() {
                if let Some(member) = struct_ty.members.get(i) {
                    let value = self.eval_init_expr(&named_expr.value);
                    fields.push((member.ident.clone(), value));
                } else {
                    self.errors.push(error_span(
                        format!(
                            "too many initializers for struct (expected {}, got {})",
                            struct_ty.members.len(),
                            init_list.values.len()
                        ),
                        Label::new(ic_syntax::util::expr_span(&named_expr.value))
                            .message("extra initializer"),
                    ));
                    break;
                }
            }
        } else {
            let diag = error_span(
                "cannot mix named and positional initializers",
                Label::new(init_list.values[0].value.span())
                    .message("mixing initialization styles"),
            )
            .note("use either all named fields (e.g., {.x = 1, .y = 2}) or all positional ({1, 2})")
            .help("consider using named initialization for clarity");

            self.errors.push(diag);
            return Numeric::Null;
        }

        Numeric::Struct {
            ty: type_id,
            fields: fields.into_boxed_slice(),
        }
    }

    /// Evaluates an array initializer list.
    #[allow(clippy::needless_pass_by_value)]
    fn eval_array_init(
        &mut self,
        init_list: &ic_syntax::InitList,
        elem_ty: Ty,
        expected_len: usize,
        array_type_id: DefId,
    ) -> Numeric {
        let mut values = Vec::new();

        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.errors.push(error_span(
                    "array elements cannot have names",
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("remove the field name"),
                ));
                continue;
            }

            let value = self.eval_init_expr_with_type(&named_expr.value, &elem_ty, array_type_id);
            values.push(value);
        }

        // Check array size
        if values.len() != expected_len {
            self.errors.push(error_span(
                format!(
                    "array size mismatch: expected {} elements, found {}",
                    expected_len,
                    values.len()
                ),
                Label::new(Span::default()).message("in this initializer"),
            ));
        }

        Numeric::Array {
            ty: array_type_id,
            values: values.into_boxed_slice(),
        }
    }

    /// Evaluates a sequence initializer list.
    fn eval_sequence_init(
        &mut self,
        init_list: &ic_syntax::InitList,
        _elem_ty: Ty,
        seq_type_id: DefId,
    ) -> Numeric {
        let mut values = Vec::new();

        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.errors.push(error_span(
                    "sequence elements cannot have names",
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("remove the field name"),
                ));
                continue;
            }

            let value = self.eval_init_expr(&named_expr.value);
            values.push(value);
        }

        Numeric::Sequence {
            ty: seq_type_id,
            values: values.into_boxed_slice(),
        }
    }

    /// Evaluates a map initializer list.
    fn eval_map_init(
        &mut self,
        init_list: &ic_syntax::InitList,
        _key_ty: Ty,
        _elem_ty: Ty,
        map_type_id: DefId,
    ) -> Numeric {
        let mut entries = Vec::new();

        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.errors.push(error_span(
                    "map entries cannot have names",
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("remove the field name"),
                ));
                continue;
            }

            // Map entries should be initializer lists with exactly 2 elements
            match &named_expr.value {
                Expr::InitList(entry_list) => {
                    if entry_list.values.len() != 2 {
                        self.errors.push(error_span(
                            format!(
                                "map entry must have exactly 2 elements (key and value), found {}",
                                entry_list.values.len()
                            ),
                            Label::new(Span::default()).message("in this entry"),
                        ));
                        continue;
                    }

                    let key = self.eval_init_expr(&entry_list.values[0].value);
                    let value = self.eval_init_expr(&entry_list.values[1].value);
                    entries.push((key, value));
                }
                _ => {
                    self.errors.push(error_span(
                        "map entries must be initializer lists with {key, value}",
                        Label::new(ic_syntax::util::expr_span(&named_expr.value))
                            .message("expected {key, value}"),
                    ));
                }
            }
        }

        Numeric::Map {
            ty: map_type_id,
            values: entries.into_boxed_slice(),
        }
    }

    /// Evaluates a constant definition.
    fn evaluate_const(&mut self, id: DefId, def: &ic_syntax::ConstDef) {
        // First, handle array bounds if present
        let bounds = if let ic_syntax::Declarator::Array(arr) = &def.decl {
            Some(arr.bounds.clone())
        } else {
            None
        };

        // Update array bounds in the type before evaluating the value
        if let Some(bounds) = bounds {
            let hir_def = self.ctx.definitions.get(id);
            let ty_to_update = if let DefKind::Const(const_ty) = &hir_def.kind {
                Some(const_ty.ty.clone())
            } else {
                None
            };

            if let Some(mut ty) = ty_to_update {
                // Evaluate the bounds with spans
                let evaluated_bounds = self.evaluate_array_bounds_with_spans(&bounds);
                Self::update_type_bounds_with_values_and_spans(&mut ty, &evaluated_bounds);

                // Update the type with evaluated bounds
                let hir_def = self.ctx.definitions.get_mut(id);
                if let DefKind::Const(const_ty) = &mut hir_def.kind {
                    const_ty.ty = ty;
                }
            }
        }

        // Check if this is a string constant with a string literal
        let hir_def = self.ctx.definitions.get(id);
        let is_string_const = if let DefKind::Const(const_ty) = &hir_def.kind {
            matches!(const_ty.ty.kind, TyKind::String { .. })
        } else {
            false
        };

        // Now evaluate the value with the updated type
        let value = match &def.value {
            // Handle string literals for string constants
            ic_syntax::Expr::Literal(lit) if is_string_const => match &lit.value {
                ic_syntax::LiteralValue::String(s) => Numeric::String(s.clone()),
                _ => self.eval_expr(&def.value),
            },
            ic_syntax::Expr::InitList(init_list) => {
                // Get the type information (now with evaluated bounds)
                let hir_def = self.ctx.definitions.get(id);
                if let DefKind::Const(const_ty) = &hir_def.kind {
                    match &const_ty.ty.kind {
                        TyKind::Adt(type_id) => {
                            // Look up the ADT definition
                            let adt_def = self.ctx.definitions.get(*type_id);
                            if let DefKind::Struct(struct_ty) = &adt_def.kind {
                                let struct_ty = struct_ty.clone();
                                self.eval_struct_init(init_list, &struct_ty, *type_id)
                            } else {
                                self.errors.push(error_span(
                                    "initializer lists can only be used with struct types",
                                    Label::new(def.span).message("not a struct type"),
                                ));
                                Numeric::Null
                            }
                        }
                        TyKind::Array { ty, len, .. } => {
                            self.eval_array_init(init_list, ty.as_ref().clone(), *len, id)
                        }
                        TyKind::Sequence { ty, .. } => {
                            self.eval_sequence_init(init_list, ty.as_ref().clone(), id)
                        }
                        TyKind::Map { key, elem, .. } => self.eval_map_init(
                            init_list,
                            key.as_ref().clone(),
                            elem.as_ref().clone(),
                            id,
                        ),
                        _ => {
                            self.errors.push(error_span(
                                "initializer lists can only be used with struct, array, sequence, \
                                 or map types",
                                Label::new(def.span).message("incompatible type"),
                            ));
                            Numeric::Null
                        }
                    }
                } else {
                    Numeric::Null
                }
            }
            _ => self.eval_expr(&def.value),
        };

        // Get the declared type for conversion
        let declared_ty = {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Const(const_ty) = &hir_def.kind {
                Some(const_ty.ty.clone())
            } else {
                None
            }
        };

        // Convert value to match declared type (handles unsigned wrapping)
        let final_value = if let Some(ty) = declared_ty {
            self.convert_to_type(value, &ty)
        } else {
            value
        };

        // Update the value
        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Const(const_ty) = &mut hir_def.kind {
            const_ty.value = final_value;
        }
    }

    /// Evaluates bounds in a type (for sequence/string/map)
    fn evaluate_type_bounds_in_alias(&mut self, id: DefId, ast_ty: &ic_syntax::Type) {
        // Get the current HIR type
        let hir_def = self.ctx.definitions.get(id);
        if let DefKind::Alias(alias_ty) = &hir_def.kind {
            let mut ty = alias_ty.ty.clone();
            self.evaluate_type_bounds(&mut ty, ast_ty);

            // Update the HIR with the evaluated type
            let hir_def = self.ctx.definitions.get_mut(id);
            if let DefKind::Alias(alias_ty) = &mut hir_def.kind {
                alias_ty.ty = ty;
            }
        }
    }

    /// Recursively evaluates bounds in a type
    fn evaluate_type_bounds(&mut self, hir_ty: &mut Ty, ast_ty: &ic_syntax::Type) {
        use ic_syntax::Type;

        match (ast_ty, &mut hir_ty.kind) {
            (Type::Sequence(seq), TyKind::Sequence { ty, bound, .. }) => {
                if let Some(ref bound_expr) = seq.bound {
                    *bound = Some(self.eval_bound(bound_expr));
                }
                // Recursively handle nested type
                self.evaluate_type_bounds(ty, &seq.ty);
            }
            (Type::String(str), TyKind::String { bound, .. }) => {
                if let Some(ref bound_expr) = str.bound {
                    *bound = Some(self.eval_bound(bound_expr));
                }
            }
            (
                Type::Map(map),
                TyKind::Map {
                    key, elem, bound, ..
                },
            ) => {
                if let Some(ref bound_expr) = map.bound {
                    *bound = Some(self.eval_bound(bound_expr));
                }
                // Recursively handle key and value types
                self.evaluate_type_bounds(key, &map.key);
                self.evaluate_type_bounds(elem, &map.value);
            }
            _ => {} // Other types don't have bounds
        }
    }

    /// Evaluates expressions in type definitions.
    #[allow(clippy::too_many_lines)]
    fn evaluate_types(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::ModuleValue(v) => {
                    // Look up the definition and evaluate the module
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_module(def_id, v);
                    }
                }
                Item::StructValue(v) => {
                    // Look up the definition in the current scope
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_struct(def_id, v);
                    }
                }
                Item::UnionValue(v) => {
                    // Look up the definition in the current scope
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_union(def_id, v);
                    }
                }
                Item::EnumValue(v) => {
                    // First check if we've already evaluated this enum
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        if self.evaluated.contains(&def_id) {
                            continue;
                        }
                        self.evaluate_enum(def_id, v);
                    }
                }
                Item::BitmaskValue(v) => {
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_bitmask(def_id, v);
                    }
                }
                Item::BitsetValue(v) => {
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_bitset(def_id, v);
                    }
                }
                Item::ConstValue(v) => {
                    let name = match &v.decl {
                        ic_syntax::Declarator::Simple(n) => &n.name,
                        ic_syntax::Declarator::Array(a) => &a.ident.name,
                    };
                    if let Some(def_id) = self.ctx.scopes.resolve_name(self.current_scope, name) {
                        self.evaluate_const(def_id, v);
                    }
                }
                Item::AliasValue(v) => {
                    // Handle each declarator in the alias
                    for decl in &v.decl {
                        let name = match decl {
                            ic_syntax::Declarator::Simple(n) => &n.name,
                            ic_syntax::Declarator::Array(a) => &a.ident.name,
                        };
                        if let Some(def_id) = self.ctx.scopes.resolve_name(self.current_scope, name)
                        {
                            self.evaluate_alias(def_id, decl);
                            // Also evaluate bounds in the type itself
                            self.evaluate_type_bounds_in_alias(def_id, &v.ty);
                        }
                    }
                }
                Item::ExceptionValue(v) => {
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_exception(def_id, v);
                    }
                }
                Item::AnnotationValue(v) => {
                    // Look up the definition in the current scope
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_annotation(def_id, v);
                    }
                }
                Item::InterfaceValue(v) => {
                    // Evaluate nested items in interfaces
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
                        self.evaluate_interface(def_id, v);
                    }
                }
                _ => {}
            }
        }
    }

    /// Evaluates default values in an annotation definition.
    fn evaluate_annotation(&mut self, def_id: DefId, ast: &ic_syntax::AnnotationDef) {
        // Save current scope and enter annotation scope
        let saved_scope = self.current_scope;
        if let Some(annotation_scope) = self.ctx.scopes.find_scope_for_def(def_id) {
            self.current_scope = annotation_scope;
        }

        // First, evaluate nested types (enums, etc.)
        let nested_items: Vec<Item> = ast
            .params
            .iter()
            .filter_map(|field| {
                if let ic_syntax::AnnotationField::Item(item) = field {
                    Some((**item).clone())
                } else {
                    None
                }
            })
            .collect();
        if !nested_items.is_empty() {
            self.evaluate_types(&nested_items);
        }

        // Then collect default values to evaluate
        let mut default_values = Vec::new();
        let mut member_idx = 0;
        for field in &ast.params {
            if let ic_syntax::AnnotationField::Member(ast_member) = field {
                if let Some(default_expr) = &ast_member.default {
                    // Handle string literals specially since they can't go through arithmetic evaluation
                    let value = match default_expr {
                        Expr::Literal(lit)
                            if matches!(lit.value, ic_syntax::LiteralValue::String(_)) =>
                        {
                            // Extract string value directly
                            if let ic_syntax::LiteralValue::String(s) = &lit.value {
                                Numeric::String(s.clone())
                            } else {
                                Numeric::Null
                            }
                        }
                        _ => {
                            // Evaluate other expressions normally
                            self.eval_expr(default_expr)
                        }
                    };
                    default_values.push((member_idx, value));
                }
                member_idx += 1;
            }
        }

        // Now update the annotation definition with evaluated values
        let def = self.ctx.definitions.get_mut(def_id);
        if let DefKind::Annotation(ann) = &mut def.kind {
            for (idx, value) in default_values {
                if idx < ann.members.len() {
                    ann.members[idx].default_value = Some(value);
                }
            }
        }

        // Restore scope
        self.current_scope = saved_scope;
    }

    /// Evaluates nested items in a module.
    fn evaluate_module(&mut self, def_id: DefId, ast: &ic_syntax::ModuleDef) {
        // Save current scope and enter module scope
        let saved_scope = self.current_scope;
        // For modules, we need to find the scope differently because reopened modules
        // share the same scope but have different DefIds
        let module_scope = if let DefKind::Module(_) = &self.ctx.definitions.get(def_id).kind {
            // For modules, look for a child scope with the module's name in the parent scope
            let parent_scope = self.ctx.scopes.get_scope(self.current_scope);
            parent_scope.children.get(&ast.ident.name).copied()
        } else {
            self.ctx.scopes.find_scope_for_def(def_id)
        };

        if let Some(module_scope) = module_scope {
            self.current_scope = module_scope;
        }

        // Evaluate all nested definitions
        self.evaluate_types(&ast.definitions);

        // Restore scope
        self.current_scope = saved_scope;
    }

    /// Evaluates nested items in an interface.
    fn evaluate_interface(&mut self, def_id: DefId, ast: &ic_syntax::InterfaceDef) {
        // Save current scope and enter interface scope
        let saved_scope = self.current_scope;
        if let Some(interface_scope) = self.ctx.scopes.find_scope_for_def(def_id) {
            self.current_scope = interface_scope;
        }

        // Extract nested type definitions from interface members
        let nested_items: Vec<Item> = ast
            .members
            .iter()
            .filter_map(|member| {
                if let ic_syntax::InterfaceMember::Item(item) = member {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect();

        // Evaluate all nested items
        if !nested_items.is_empty() {
            self.evaluate_types(&nested_items);
        }

        // Restore scope
        self.current_scope = saved_scope;
    }
}

/// Evaluates all expressions in the HIR.
pub fn evaluate_expressions(
    ctx: &mut Context,
    _name_map: &super::collect::NameMap,
    items: &[Item],
) -> Vec<Diag> {
    let mut evaluator = ExpressionEvaluator::new(ctx);

    evaluator.evaluate_types(items);
    evaluator.errors
}
