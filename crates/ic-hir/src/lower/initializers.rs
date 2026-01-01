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
use ic_syntax::{Expr, InitList};

use super::eval::ConstEvaluator;
use crate::hir::{DefId, DefKind, Numeric, Ty};

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
        init_list: &InitList,
        struct_def_id: DefId,
        _struct_ty: &Ty,
    ) -> Option<Numeric> {
        let (struct_name, struct_members) = {
            let struct_def = self
                .evaluator
                .context()
                .context
                .definitions
                .get(struct_def_id);
            let DefKind::Struct(struct_ty_info) = &struct_def.kind else {
                return None;
            };
            (
                struct_def.ident.name.clone(),
                struct_ty_info.members.clone(),
            )
        };

        // Build a map of field names to types from the struct definition
        let mut field_map = std::collections::HashMap::new();
        for member in &struct_members {
            field_map.insert(member.ident.name.clone(), member.ty.clone());
        }

        // Process initializer list elements
        let mut fields = Vec::new();

        if init_list.values.is_empty() {
            self.evaluator.diagnostics().error(
                "struct initializer cannot be empty".to_string(),
                Label::new(ic_syntax::util::expr_span(&Expr::InitList(
                    init_list.clone(),
                )))
                .message("expected field values"),
            );
            return None;
        }

        // Handle both named and positional initialization
        let is_named = init_list.values.iter().any(|v| v.ident.is_some());

        if is_named {
            // Named initialization: { .field1 = value1, .field2 = value2 }
            // Build a map of field name -> value from the initializer list
            let mut value_map = std::collections::HashMap::new();
            for named_expr in &init_list.values {
                if let Some(ref ident) = named_expr.ident {
                    if let Some(field_ty) = field_map.get(&ident.name) {
                        if let Some(value) =
                            self.evaluator.eval_for_type(&named_expr.value, field_ty)
                        {
                            value_map.insert(ident.name.clone(), value);
                        }
                    } else {
                        self.evaluator.diagnostics().error(
                            format!(
                                "struct `{}` has no field named `{}`",
                                struct_name, ident.name
                            ),
                            Label::new(ident.span).message("unknown field"),
                        );
                    }
                } else {
                    self.evaluator.diagnostics().error(
                        "mixing named and positional initialization is not allowed".to_string(),
                        Label::new(ic_syntax::util::expr_span(&named_expr.value))
                            .message("expected named field"),
                    );
                }
            }

            // Collect values in struct member declaration order
            for member in &struct_members {
                if let Some(value) = value_map.remove(&member.ident.name) {
                    fields.push(value);
                } else {
                    // Field not provided in initializer - this is an error
                    self.evaluator.diagnostics().error(
                        format!(
                            "missing initializer for field `{}` in struct `{}`",
                            member.ident.name, struct_name
                        ),
                        Label::new(init_list.span).message("incomplete initialization"),
                    );
                }
            }
        } else {
            // Positional initialization: { value1, value2 }
            if init_list.values.len() != struct_members.len() {
                self.evaluator.diagnostics().error(
                    format!(
                        "struct `{}` expects {} fields, but {} were provided",
                        struct_name,
                        struct_members.len(),
                        init_list.values.len()
                    ),
                    Label::new(ic_syntax::util::expr_span(&Expr::InitList(
                        init_list.clone(),
                    )))
                    .message("incorrect number of fields"),
                );
                return None;
            }

            // Match values to fields in declaration order
            let mut has_error = false;
            for (i, named_expr) in init_list.values.iter().enumerate() {
                let member = &struct_members[i];
                if let Some(value) = self.evaluator.eval_for_type(&named_expr.value, &member.ty) {
                    fields.push(value);
                } else {
                    // Error already reported by eval_for_type, but we need to track failure
                    has_error = true;
                }
            }
            if has_error {
                return None;
            }
        }

        Some(Numeric::Struct {
            ty: struct_def_id,
            fields: fields.into_boxed_slice(),
        })
    }

    /// Evaluates an array initializer list.
    pub fn eval_array(
        &mut self,
        init_list: &InitList,
        elem_ty: &Ty,
        expected_len: usize,
    ) -> Option<Numeric> {
        let mut elements = Vec::new();

        // Check that we have the correct number of elements
        if init_list.values.len() != expected_len {
            self.evaluator.diagnostics().error(
                format!(
                    "array expects {} elements, but {} were provided",
                    expected_len,
                    init_list.values.len()
                ),
                Label::new(ic_syntax::util::expr_span(&Expr::InitList(
                    init_list.clone(),
                )))
                .message("incorrect number of elements"),
            );
            return None;
        }

        // Evaluate each element
        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.evaluator.diagnostics().error(
                    "array elements cannot have names".to_string(),
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("unexpected named element"),
                );
                return None;
            }
            if let Some(value) = self.evaluator.eval_for_type(&named_expr.value, elem_ty) {
                elements.push(value);
            } else {
                // eval_for_type already reported error
                return None;
            }
        }

        Some(Numeric::Array {
            ty: elem_ty.clone(),
            values: elements.into_boxed_slice(),
        })
    }

    /// Evaluates a sequence initializer list.
    pub fn eval_sequence(&mut self, init_list: &InitList, elem_ty: &Ty) -> Option<Numeric> {
        let mut elements = Vec::new();

        // Evaluate each element
        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.evaluator.diagnostics().error(
                    "sequence elements cannot have names".to_string(),
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("unexpected named element"),
                );
                return None;
            }
            if let Some(value) = self.evaluator.eval_for_type(&named_expr.value, elem_ty) {
                elements.push(value);
            } else {
                // eval_for_type already reported error
                return None;
            }
        }

        Some(Numeric::Sequence {
            ty: elem_ty.clone(),
            values: elements.into_boxed_slice(),
        })
    }

    /// Evaluates a map initializer list.
    pub fn eval_map(&mut self, init_list: &InitList, key_ty: &Ty, elem_ty: &Ty) -> Option<Numeric> {
        let mut pairs = Vec::new();

        // Each element should be a pair initializer list {key, value}
        for named_expr in &init_list.values {
            if named_expr.ident.is_some() {
                self.evaluator.diagnostics().error(
                    "map entries cannot have names".to_string(),
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("unexpected named element"),
                );
                return None;
            }

            // Each element must itself be an initializer list with 2 elements
            if let Expr::InitList(pair_init) = &named_expr.value {
                if pair_init.values.len() != 2 {
                    self.evaluator.diagnostics().error(
                        "map entry must have exactly 2 elements (key and value)".to_string(),
                        Label::new(ic_syntax::util::expr_span(&named_expr.value))
                            .message("expected {key, value}"),
                    );
                    return None;
                }

                // Check no names in pair
                if pair_init.values.iter().any(|v| v.ident.is_some()) {
                    self.evaluator.diagnostics().error(
                        "map key and value cannot have names".to_string(),
                        Label::new(ic_syntax::util::expr_span(&named_expr.value))
                            .message("unexpected named elements"),
                    );
                    return None;
                }

                // Evaluate key and value with their correct types
                let key = self
                    .evaluator
                    .eval_for_type(&pair_init.values[0].value, key_ty)?;
                let value = self
                    .evaluator
                    .eval_for_type(&pair_init.values[1].value, elem_ty)?;
                pairs.push((key, value));
            } else {
                self.evaluator.diagnostics().error(
                    "map entry must be an initializer list {key, value}".to_string(),
                    Label::new(ic_syntax::util::expr_span(&named_expr.value))
                        .message("expected initializer list"),
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
