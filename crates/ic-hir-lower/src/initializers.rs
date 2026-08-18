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

//! Initializer list evaluation for structs, arrays, sequences, and maps.

use ic_diagnostic::Label;
use ic_hir::hir::{DefId, DefKind, Member, Numeric, Ty};
use ic_syntax::{ExprKind, NamedExpr};

use crate::eval::ConstEvaluator;

/// Handles evaluation of initializer lists for complex types.
pub struct InitializerEvaluator<'a, 'b> {
    evaluator: &'a mut ConstEvaluator<'b>,
}

impl<'a, 'b> InitializerEvaluator<'a, 'b> {
    /// Creates a new initializer evaluator.
    pub fn new(evaluator: &'a mut ConstEvaluator<'b>) -> Self {
        Self { evaluator }
    }

    /// Evaluates an initializer list for a struct type.
    pub fn eval_struct(
        &mut self,
        init_list: &[NamedExpr],
        struct_def_id: DefId,
        _struct_ty: &Ty,
        init_span: ic_syntax::Span,
    ) -> Option<Numeric> {
        let struct_name = {
            let struct_def = self.evaluator.ctx.context.definitions.get(struct_def_id);
            if !matches!(struct_def.kind, DefKind::Struct(_)) {
                return None;
            }
            struct_def.ident.name.clone()
        };
        let struct_members = self.collect_members(struct_def_id);

        if init_list.is_empty() {
            self.evaluator.diagnostics().error(
                "struct initializer cannot be empty".to_string(),
                Label::new(init_span).message("expected field values"),
            );
            return None;
        }

        let is_named = init_list.iter().any(|v| v.name.is_some());
        let fields = if is_named {
            self.eval_struct_named(init_list, &struct_name, &struct_members, init_span)?
        } else {
            self.eval_struct_positional(init_list, &struct_name, &struct_members, init_span)?
        };

        Some(Numeric::Struct {
            ty: struct_def_id,
            fields: fields.into_boxed_slice(),
        })
    }

    /// Collects the members of a struct, inherited members first.
    fn collect_members(&self, struct_def_id: DefId) -> Vec<Member> {
        let context = &self.evaluator.ctx.context;

        let mut chain = vec![];
        let mut next = Some(struct_def_id);
        while let Some(def_id) = next {
            let DefKind::Struct(struct_ty) = &context.definitions.get(def_id).kind else {
                break;
            };
            chain.push(struct_ty);
            next = struct_ty.parent.map(|parent| parent.def_id);
        }

        chain
            .into_iter()
            .rev()
            .flat_map(|struct_ty| struct_ty.members.iter().cloned())
            .collect()
    }

    /// Evaluates named struct initialization: `{ .field1 = value1, .field2 = value2 }`
    fn eval_struct_named(
        &mut self,
        init_list: &[NamedExpr],
        struct_name: &str,
        struct_members: &[Member],
        init_span: ic_syntax::Span,
    ) -> Option<Vec<Numeric>> {
        let field_map: std::collections::HashMap<_, _> = struct_members
            .iter()
            .map(|m| (m.ident.name.clone(), m.ty.clone()))
            .collect();

        let mut value_map = std::collections::HashMap::new();
        let mut field_spans = std::collections::HashMap::new();
        let mut has_error = false;

        for named_expr in init_list {
            if let Some(ref ident) = named_expr.name {
                if let Some(first_span) = field_spans.insert(ident.name.clone(), ident.span) {
                    self.evaluator.diagnostics().errors.push(
                        ic_diagnostic::error_span(
                            format!("field `{}` specified more than once", ident.name),
                            Label::new(first_span).message("first assignment"),
                        )
                        .label(Label::new(ident.span).message("duplicate assignment")),
                    );
                    has_error = true;
                    continue;
                }

                if let Some(field_ty) = field_map.get(&ident.name) {
                    if let Some(value) = self.evaluator.eval_for_type(&named_expr.value, field_ty) {
                        value_map.insert(ident.name.clone(), value);
                    } else {
                        has_error = true;
                    }
                } else {
                    self.evaluator.diagnostics().error(
                        format!("struct `{struct_name}` has no field named `{}`", ident.name),
                        Label::new(ident.span).message("unknown field"),
                    );
                    has_error = true;
                }
            } else {
                self.evaluator.diagnostics().error(
                    "mixing named and positional initialization is not allowed".to_string(),
                    Label::new(named_expr.value.span).message("expected named field"),
                );
                has_error = true;
            }
        }

        let mut fields = Vec::new();
        for member in struct_members {
            if let Some(value) = value_map.remove(&member.ident.name) {
                fields.push(value);
            } else {
                self.evaluator.diagnostics().error(
                    format!(
                        "missing initializer for field `{}` in struct `{struct_name}`",
                        member.ident.name
                    ),
                    Label::new(init_span).message("incomplete initialization"),
                );
                has_error = true;
            }
        }

        if has_error { None } else { Some(fields) }
    }

    /// Evaluates positional struct initialization: `{ value1, value2 }`
    fn eval_struct_positional(
        &mut self,
        init_list: &[NamedExpr],
        struct_name: &str,
        struct_members: &[Member],
        init_span: ic_syntax::Span,
    ) -> Option<Vec<Numeric>> {
        if init_list.len() != struct_members.len() {
            self.evaluator.diagnostics().error(
                format!(
                    "struct `{struct_name}` expects {} fields, but {} were provided",
                    struct_members.len(),
                    init_list.len()
                ),
                Label::new(init_span).message("incorrect number of fields"),
            );
            return None;
        }

        let mut fields = Vec::new();
        let mut has_error = false;
        for (i, named_expr) in init_list.iter().enumerate() {
            let member = &struct_members[i];
            if let Some(value) = self.evaluator.eval_for_type(&named_expr.value, &member.ty) {
                fields.push(value);
            } else {
                has_error = true;
            }
        }

        if has_error { None } else { Some(fields) }
    }

    /// Evaluates an array initializer list.
    pub fn eval_array(
        &mut self,
        init_list: &[NamedExpr],
        elem_ty: &Ty,
        expected_len: usize,
        init_span: ic_syntax::Span,
    ) -> Option<Numeric> {
        let mut elements = Vec::new();

        // Check that we have the correct number of elements
        if init_list.len() != expected_len {
            self.evaluator.diagnostics().error(
                format!(
                    "array expects {} elements, but {} were provided",
                    expected_len,
                    init_list.len()
                ),
                Label::new(init_span).message("incorrect number of elements"),
            );
            return None;
        }

        // Evaluate each element
        for named_expr in init_list {
            if named_expr.name.is_some() {
                self.evaluator.diagnostics().error(
                    "array elements cannot have names".to_string(),
                    Label::new(named_expr.value.span).message("unexpected named element"),
                );
                return None;
            }
            {
                let value = self.evaluator.eval_for_type(&named_expr.value, elem_ty)?;
                elements.push(value);
            }
        }

        Some(Numeric::Array {
            ty: elem_ty.clone(),
            values: elements.into_boxed_slice(),
        })
    }

    /// Evaluates a sequence initializer list.
    pub fn eval_sequence(
        &mut self,
        init_list: &[NamedExpr],
        elem_ty: &Ty,
        _init_span: ic_syntax::Span,
    ) -> Option<Numeric> {
        let mut elements = Vec::new();

        // Evaluate each element
        for named_expr in init_list {
            if named_expr.name.is_some() {
                self.evaluator.diagnostics().error(
                    "sequence elements cannot have names".to_string(),
                    Label::new(named_expr.value.span).message("unexpected named element"),
                );
                return None;
            }
            {
                let value = self.evaluator.eval_for_type(&named_expr.value, elem_ty)?;
                elements.push(value);
            }
        }

        Some(Numeric::Sequence {
            ty: elem_ty.clone(),
            values: elements.into_boxed_slice(),
        })
    }

    /// Evaluates a map initializer list.
    pub fn eval_map(
        &mut self,
        init_list: &[NamedExpr],
        key_ty: &Ty,
        elem_ty: &Ty,
        _init_span: ic_syntax::Span,
    ) -> Option<Numeric> {
        let mut pairs = Vec::new();

        // Each element should be a pair initializer list {key, value}
        for named_expr in init_list {
            if named_expr.name.is_some() {
                self.evaluator.diagnostics().error(
                    "map entries cannot have names".to_string(),
                    Label::new(named_expr.value.span).message("unexpected named element"),
                );
                return None;
            }

            // Each element must itself be an initializer list with 2 elements
            if let ExprKind::InitList(pair_init) = &named_expr.value.value {
                if pair_init.len() != 2 {
                    self.evaluator.diagnostics().error(
                        "map entry must have exactly 2 elements (key and value)".to_string(),
                        Label::new(named_expr.value.span).message("expected {key, value}"),
                    );
                    return None;
                }

                // Check no names in pair
                if pair_init.iter().any(|v| v.name.is_some()) {
                    self.evaluator.diagnostics().error(
                        "map key and value cannot have names".to_string(),
                        Label::new(named_expr.value.span).message("unexpected named elements"),
                    );
                    return None;
                }

                // Evaluate key and value with their correct types
                let key = self.evaluator.eval_for_type(&pair_init[0].value, key_ty)?;
                let value = self.evaluator.eval_for_type(&pair_init[1].value, elem_ty)?;
                pairs.push((key, value));
            } else {
                self.evaluator.diagnostics().error(
                    "map entry must be an initializer list {key, value}".to_string(),
                    Label::new(named_expr.value.span).message("expected initializer list"),
                );
                return None;
            }
        }

        Some(Numeric::Map {
            key: key_ty.clone(),
            value: elem_ty.clone(),
            entries: pairs.into_boxed_slice(),
        })
    }
}
