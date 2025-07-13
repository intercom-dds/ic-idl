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

use ic_diagnostic::{Diag, Label, error_span, warn_span};
use ic_expr::{
    Error as ExprError, EvalConfig, GenericNumeric, NumericValue, OverflowBehavior,
    Result as ExprResult,
};
use ic_syntax::{Expr, Item};

use crate::Context;
use crate::hir::*;
use crate::scope::ScopeId;

/// Literal type for ic-expr evaluation.
#[derive(Debug, Clone)]
struct IdlLiteral {
    span: ic_syntax::Span,
    value: ic_syntax::LiteralValue,
}

/// Type alias for the generic numeric type from ic-expr.
type EvalNumeric = GenericNumeric;

/// Convert from GenericNumeric to HIR Numeric type
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

/// Try to convert from HIR Numeric type to GenericNumeric
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
    errors: &'a mut Vec<Diag>,
    current_scope: crate::scope::ScopeId,
}

impl<'a> ic_expr::EvalContext<IdlLiteral> for IdlEvalContext<'a> {
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
                Ok(GenericNumeric::Int32(*i as i32))
            }
            ic_syntax::LiteralValue::Float(f) => {
                // TODO: Handle float vs double based on suffix
                Ok(GenericNumeric::Float(*f as f32))
            }
            ic_syntax::LiteralValue::String(s) => Err(ExprError::Custom(format!(
                "string literals not supported in constant expressions: \"{}\"",
                s
            ))),
        }
    }

    fn lookup_var(&mut self, name: &str) -> ExprResult<Self::Value> {
        // Check if name starts with :: for global scope resolution
        let (start_scope, name_without_prefix) = if name.starts_with("::") {
            (self.ctx.scopes.root(), &name[2..])
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
                                        return Ok(GenericNumeric::Int32(field.value as i32));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(ExprError::Custom(format!("undefined variable: {}", name)))
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
        format!("::{}", segments)
    } else {
        segments
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
                _ => return Err(format!("unsupported binary operator: {:?}", binary.op.kind)),
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

        Expr::InitList(_) => {
            Err("initializer lists not supported in constant expressions".to_string())
        }
    }
}

/// Evaluates expressions in the HIR.
pub struct ExpressionEvaluator<'a> {
    ctx: &'a mut Context,
    errors: Vec<Diag>,
    current_scope: crate::scope::ScopeId,
}

impl<'a> ExpressionEvaluator<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            errors: Vec::new(),
            current_scope: root_scope,
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

        // Create evaluation context
        let mut eval_ctx = IdlEvalContext {
            ctx: &self.ctx,
            config: ic_expr::EvalConfig::default(),
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
                    ExprError::DivisionByZero => "division by zero",
                    ExprError::ModuloByZero => "modulo by zero",
                    ExprError::Overflow(op_str) => {
                        self.errors.push(warn_span(
                            format!("arithmetic overflow in {} operation", op_str),
                            Label::new(ic_syntax::util::expr_span(expr))
                                .message("value wraps around"),
                        ));
                        // For now, just return 0
                        // TODO: Actually compute the wrapped value
                        return Numeric::Int32(0);
                    }
                    ExprError::InvalidShift(amount) => {
                        self.errors.push(error_span(
                            format!("invalid shift amount: {}", amount),
                            Label::new(ic_syntax::util::expr_span(expr)).message("invalid shift"),
                        ));
                        return Numeric::Null;
                    }
                    ExprError::InvalidUnaryOp(op) => {
                        self.errors.push(error_span(
                            format!("invalid unary operator: {:?}", op),
                            Label::new(ic_syntax::util::expr_span(expr)).message("cannot apply"),
                        ));
                        return Numeric::Null;
                    }
                    ExprError::Custom(s) => {
                        self.errors.push(error_span(
                            s,
                            Label::new(ic_syntax::util::expr_span(expr))
                                .message("evaluation error"),
                        ));
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

    /// Handles overflow by issuing a warning and returning a wrapped value.
    fn handle_overflow(&mut self, expr: &Expr, op: ic_expr::Op) -> Numeric {
        self.errors.push(warn_span(
            format!("arithmetic overflow in {:?} operation", op),
            Label::new(ic_syntax::util::expr_span(expr)).message("value wraps around"),
        ));

        // For now, just return 0
        // TODO: Actually compute the wrapped value
        Numeric::Int32(0)
    }

    /// Evaluates a bound expression to a usize.
    fn eval_bound(&mut self, expr: &Expr) -> usize {
        match self.eval_expr(expr) {
            Numeric::Int32(v) if v >= 0 => v as usize,
            Numeric::UInt32(v) => v as usize,
            Numeric::Int64(v) if v >= 0 => v as usize,
            Numeric::UInt64(v) => v as usize,
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

    /// Updates array bounds in a type.
    fn update_type_bounds(&mut self, ty: &mut Ty, bounds: &[ic_syntax::Expr]) {
        let mut current = ty;
        let mut bound_iter = bounds.iter().rev(); // Process from outermost to innermost

        loop {
            match &mut current.kind {
                TyKind::Array { ty: inner_ty, len } => {
                    if let Some(bound_expr) = bound_iter.next() {
                        *len = self.eval_bound(bound_expr);
                    }
                    current = inner_ty;
                }
                _ => break,
            }
        }
    }

    /// Evaluates expressions in a union definition.
    fn evaluate_union(&mut self, id: DefId, def: &ic_syntax::UnionDef) {
        // First, evaluate all case labels
        let mut all_labels = Vec::new();
        for field in &def.fields {
            let mut labels = Vec::new();

            for label in &field.labels {
                if let ic_syntax::Label::Case(expr) = label {
                    labels.push(self.eval_expr(expr));
                }
            }

            all_labels.push(labels);
        }

        // Then update the HIR
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Union(union_ty) = &mut hir_def.kind {
            for (idx, labels) in all_labels.into_iter().enumerate() {
                if let Some(variant) = union_ty.variants.get_mut(idx) {
                    variant.labels = labels;
                }
            }
        }
    }

    /// Evaluates enum values.
    fn evaluate_enum(&mut self, id: DefId, def: &ic_syntax::EnumDef) {
        // First, create the enum fields and evaluate their values
        let mut fields = Vec::new();
        let mut last_value = -1isize;

        for field in &def.fields {
            let value = if let Some(expr) = &field.value {
                self.eval_bound(expr) as isize
            } else {
                last_value + 1
            };

            last_value = value;

            fields.push(EnumLit {
                ident: field.ident.clone(),
                value,
                annotations: Vec::new(), // TODO: Convert annotations
            });
        }

        // Then update the HIR
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Enum(enum_ty) = &mut hir_def.kind {
            enum_ty.fields = fields;
        }
    }

    /// Evaluates bitmask values.
    fn evaluate_bitmask(&mut self, id: DefId, def: &ic_syntax::BitmaskDef) {
        // First, create the bitmask flags and evaluate their values
        let mut flags = Vec::new();
        let mut last_value = 0usize;

        for bit in &def.bits {
            let value = if let Some(expr) = &bit.value {
                self.eval_bound(expr)
            } else {
                if last_value == 0 { 1 } else { last_value << 1 }
            };

            last_value = value;

            flags.push(BitFlag {
                ident: bit.ident.clone(),
                value,
                annotations: Vec::new(), // TODO: Convert annotations
            });
        }

        // Then update the HIR
        let hir_def = self.ctx.definitions.get_mut(id);

        if let DefKind::Bitmask(bitmask_ty) = &mut hir_def.kind {
            bitmask_ty.flags = flags;
        }
    }

    /// Evaluates an alias (typedef) definition.
    fn evaluate_alias(&mut self, id: DefId, decl: &ic_syntax::Declarator) {
        if let ic_syntax::Declarator::Array(arr) = decl {
            // Get the current type
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Alias(alias_ty) = &hir_def.kind {
                let mut ty = alias_ty.ty.clone();
                self.update_type_bounds(&mut ty, &arr.bounds);

                // Update the HIR with the evaluated type
                let hir_def = self.ctx.definitions.get_mut(id);
                if let DefKind::Alias(alias_ty) = &mut hir_def.kind {
                    alias_ty.ty = ty;
                }
            }
        }
    }

    /// Evaluates a constant definition.
    fn evaluate_const(&mut self, id: DefId, def: &ic_syntax::ConstDef) {
        // Check if this is a string constant
        let hir_def = self.ctx.definitions.get(id);
        let is_string = if let DefKind::Const(const_ty) = &hir_def.kind {
            matches!(const_ty.ty.kind, TyKind::String { .. })
        } else {
            false
        };

        let value = if is_string {
            // For string constants, we don't evaluate them as expressions
            // Just store an empty string as placeholder
            Numeric::String(String::new())
        } else {
            self.eval_expr(&def.value)
        };

        // Handle array bounds separately
        let bounds = if let ic_syntax::Declarator::Array(arr) = &def.decl {
            Some(arr.bounds.clone())
        } else {
            None
        };

        // Clone the type to avoid mutable borrow issues
        let ty_to_update = if let Some(ref bounds) = bounds {
            let hir_def = self.ctx.definitions.get(id);
            if let DefKind::Const(const_ty) = &hir_def.kind {
                Some(const_ty.ty.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Update the type bounds if needed
        if let (Some(mut ty), Some(bounds)) = (ty_to_update, bounds.clone()) {
            self.update_type_bounds(&mut ty, &bounds);

            // Now update the HIR with both value and updated type
            let hir_def = self.ctx.definitions.get_mut(id);
            if let DefKind::Const(const_ty) = &mut hir_def.kind {
                const_ty.value = value;
                const_ty.ty = ty;
            }
        } else {
            // Just update the value
            let hir_def = self.ctx.definitions.get_mut(id);
            if let DefKind::Const(const_ty) = &mut hir_def.kind {
                const_ty.value = value;
            }
        }
    }

    /// Evaluates expressions in type definitions.
    fn evaluate_types(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::ModuleValue(v) => {
                    // Find the child scope for this module
                    let current_scope_data = self.ctx.scopes.get_scope(self.current_scope);
                    if let Some(&module_scope) = current_scope_data.children.get(&v.ident.name) {
                        // Save current scope
                        let saved_scope = self.current_scope;
                        self.current_scope = module_scope;

                        // Recursively evaluate module contents
                        self.evaluate_types(&v.definitions);

                        // Restore scope
                        self.current_scope = saved_scope;
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
                    if let Some(def_id) = self
                        .ctx
                        .scopes
                        .resolve_name(self.current_scope, &v.ident.name)
                    {
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
                        }
                    }
                }
                // TODO: Handle sequence/map/string bounds
                _ => {}
            }
        }
    }
}

/// Evaluates all expressions in the HIR.
pub fn evaluate_expressions(
    ctx: &mut Context,
    name_map: &super::collect::NameMap,
    items: &[Item],
) -> Vec<Diag> {
    let mut evaluator = ExpressionEvaluator::new(ctx);
    evaluator.evaluate_types(items);
    evaluator.errors
}
