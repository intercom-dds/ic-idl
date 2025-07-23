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

//! Expression evaluation context implementation.

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Diag, Label, error_span};
use ic_expr::{EvalContext, ExprResult, GenericNumeric};
use ic_syntax::{Ident, Span};

use crate::Context;
use crate::hir::{DefId, DefKind, Numeric};
use super::numeric_conversion::{IdlLiteral, from_hir_numeric};

/// Context for evaluating IDL expressions.
pub struct IdlEvalContext<'a> {
    pub hir: &'a Context,
    pub variables: HashMap<String, IdlLiteral>,
    pub const_map: CaseMap<(DefId, Span)>,
    pub enum_map: CaseMap<CaseMap<(DefId, DefId, Ident)>>,
    pub errors: &'a mut Vec<Diag>,
}

impl ic_expr::EvalContext<IdlLiteral> for IdlEvalContext<'_> {
    type Value = GenericNumeric;

    fn eval_literal(&mut self, lit: &IdlLiteral) -> ExprResult<Self::Value> {
        if let Some(const_id) = lit.const_id {
            // This is a reference to another constant
            let def = self.hir.definitions.get(const_id);
            
            if let DefKind::Const(const_ty) = &def.kind {
                // Try to convert the constant's value
                if let Some(generic) = from_hir_numeric(&const_ty.value) {
                    return Ok(generic);
                }
                
                // If it's a const reference, recursively evaluate
                if let Numeric::Const(ref_id) = &const_ty.value {
                    return self.eval_literal(&IdlLiteral {
                        const_id: Some(*ref_id),
                        enum_id: None,
                        field: None,
                        numeric: GenericNumeric::Null,
                    });
                }
            }
        }
        
        Ok(lit.numeric.clone())
    }

    fn lookup_var(&mut self, name: &str) -> ExprResult<Self::Value> {
        // Try to look up as a constant
        if let Some(&(const_id, span)) = self.const_map.get(name) {
            let def = self.hir.definitions.get(const_id);
            
            if let DefKind::Const(const_ty) = &def.kind {
                if let Some(generic) = from_hir_numeric(&const_ty.value) {
                    return Ok(generic);
                }
                
                // Handle forward references or complex values
                self.errors.push(error_span(
                    format!("constant `{name}` has non-evaluable value"),
                    Label::new(span).message("used here"),
                ));
                return Err(ic_expr::Error::UnknownVariable(name.to_string()));
            }
        }

        // Try to look up as an enum field
        if let Some(parts) = name.split_once("::") {
            let (enum_name, field_name) = parts;
            
            if let Some(enum_fields) = self.enum_map.get(enum_name) {
                if let Some(&(enum_id, field_id, ref field_ident)) = enum_fields.get(field_name) {
                    // Get the enum definition to find the field value
                    let enum_def = self.hir.definitions.get(enum_id);
                    
                    if let DefKind::Enum(enum_ty) = &enum_def.kind {
                        // Find the field in the enum
                        if let Some(field) = enum_ty.fields.iter().find(|f| f.ident.name == field_ident.name) {
                            return Ok(GenericNumeric::Int(i64::from(field.value)));
                        }
                    }
                }
            }
        }

        // Check if it's in our local variables
        if let Some(lit) = self.variables.get(name) {
            return self.eval_literal(lit);
        }

        Err(ic_expr::Error::UnknownVariable(name.to_string()))
    }
}