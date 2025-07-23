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

//! Expression conversion from AST to ic-expr format.

use ic_expr::GenericNumeric;
use ic_syntax::{Expr, LiteralValue, Path, ScopedIdent};

use super::numeric_conversion::IdlLiteral;

/// Converts a path to a string representation.
pub fn path_to_string(path: &Path) -> String {
    let mut result = String::new();
    
    if path.global {
        result.push_str("::");
    }
    
    for (i, segment) in path.segments.iter().enumerate() {
        if i > 0 {
            result.push_str("::");
        }
        result.push_str(&segment.name);
    }
    
    result
}

/// Extracts the constant name from a constant definition.
pub fn extract_const_name(def: &ic_syntax::ConstDef) -> &str {
    match &def.decl {
        ic_syntax::Declarator::Simple(ident) => &ident.name,
        ic_syntax::Declarator::Array { name, .. } => &name.name,
    }
}

/// Converts an AST expression to ic-expr format.
pub fn convert_expr(expr: &Expr) -> Result<ic_expr::Expr<IdlLiteral>, String> {
    use ic_expr::Expr as E;

    match expr {
        Expr::Literal(lit) => {
            let numeric = match &lit.value {
                LiteralValue::Bool(v) => GenericNumeric::Bool(*v),
                LiteralValue::Char(v) => GenericNumeric::Char(*v),
                LiteralValue::String(s) => GenericNumeric::String(s.clone()),
                LiteralValue::Int(i) => GenericNumeric::Int(*i),
                LiteralValue::Float(f) => GenericNumeric::Float(*f),
            };
            
            Ok(E::Literal(IdlLiteral {
                const_id: None,
                enum_id: None,
                field: None,
                numeric,
            }))
        }
        
        Expr::Const(scoped) => {
            let name = match scoped {
                ScopedIdent::Unscoped(ident) => ident.name.clone(),
                ScopedIdent::Scoped(path) => path_to_string(path),
            };
            Ok(E::Variable(name))
        }
        
        Expr::Binary { left, op, right } => {
            let left = Box::new(convert_expr(left)?);
            let right = Box::new(convert_expr(right)?);
            
            use ic_expr::BinOp;
            use ic_syntax::BinOp as SynOp;
            
            let op = match op {
                SynOp::Or => BinOp::Or,
                SynOp::Xor => BinOp::Xor,
                SynOp::And => BinOp::And,
                SynOp::Shl => BinOp::Shl,
                SynOp::Shr => BinOp::Shr,
                SynOp::Add => BinOp::Add,
                SynOp::Sub => BinOp::Sub,
                SynOp::Mul => BinOp::Mul,
                SynOp::Div => BinOp::Div,
                SynOp::Mod => BinOp::Mod,
            };
            
            Ok(E::Binary { left, op, right })
        }
        
        Expr::Unary { op, expr } => {
            let expr = Box::new(convert_expr(expr)?);
            
            use ic_expr::UnOp;
            use ic_syntax::UnOp as SynOp;
            
            let op = match op {
                SynOp::Plus => UnOp::Plus,
                SynOp::Minus => UnOp::Minus,
                SynOp::Not => UnOp::Not,
            };
            
            Ok(E::Unary { op, expr })
        }
        
        Expr::Paren(expr) => convert_expr(expr),
    }
}