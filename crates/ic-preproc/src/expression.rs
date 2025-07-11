// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

use ic_expr::c_adapter::{CContext, CLiteral};
use ic_expr::{Op, SimpleInt, eval};
use ic_lexer::token::{Base, Kind, Token};

use crate::Span;
use crate::state::Error;

pub type Expr = ic_expr::Expr<Token>;

/// Extract span from an expression (if possible)
fn expr_span(expr: &Expr) -> Option<Span> {
    match expr {
        ic_expr::Expr::Lit(tok) => Some(tok.span),
        ic_expr::Expr::Unary(u) => expr_span(&u.expr),
        ic_expr::Expr::Binary(b) => expr_span(&b.lhs).or_else(|| expr_span(&b.rhs)),
        ic_expr::Expr::Ternary(t) => expr_span(&t.cond),
    }
}

/// Find the span of a division or modulo operation in an expression
fn find_div_mod_span(expr: &Expr) -> Option<Span> {
    match expr {
        ic_expr::Expr::Binary(b) => {
            match b.op {
                Op::Div | Op::Mod => {
                    // For division/modulo, prefer the right operand's span (divisor)
                    expr_span(&b.rhs).or_else(|| expr_span(&b.lhs))
                }
                _ => {
                    // Check recursively in both operands
                    find_div_mod_span(&b.lhs).or_else(|| find_div_mod_span(&b.rhs))
                }
            }
        }
        ic_expr::Expr::Unary(u) => find_div_mod_span(&u.expr),
        ic_expr::Expr::Ternary(t) => find_div_mod_span(&t.cond)
            .or_else(|| find_div_mod_span(&t.then))
            .or_else(|| find_div_mod_span(&t.els)),
        ic_expr::Expr::Lit(_) => None,
    }
}

/// Expression evaluation context
pub trait ExpressionContext {
    /// Get the source text for a span
    fn source_of(&self, span: Span) -> &str;
}

/// Convert a Token to CLiteral for ic-expr evaluation
fn token_to_cliteral(token: &Token, ctx: &dyn ExpressionContext) -> Result<CLiteral, Error> {
    let lit = ctx.source_of(token.span);
    match token.kind {
        Kind::Number { base } => {
            let value = parse_integer(lit, base, token.span)?;
            Ok(CLiteral::Int(value))
        }
        Kind::Char => {
            let value = parse_character(lit, token.span)?;
            Ok(CLiteral::Char(char::from_u32(value as u32).unwrap_or('\0')))
        }
        Kind::Ident | Kind::Keyword(_) => Ok(CLiteral::Ident(lit.to_string())),
        _ => unreachable!("Invalid token kind for literal: {:?}", token.kind),
    }
}

/// Convert our Expr<Token> to Expr<CLiteral> for evaluation
fn convert_expr(
    expr: &Expr,
    ctx: &dyn ExpressionContext,
) -> Result<ic_expr::Expr<CLiteral>, Error> {
    match expr {
        ic_expr::Expr::Lit(token) => {
            let literal = token_to_cliteral(token, ctx)?;
            Ok(ic_expr::Expr::Lit(literal))
        }
        ic_expr::Expr::Unary(unary) => {
            let converted = convert_expr(&unary.expr, ctx)?;
            Ok(ic_expr::Expr::Unary(Box::new(ic_expr::Unary {
                op: unary.op,
                expr: converted,
            })))
        }
        ic_expr::Expr::Binary(binary) => {
            let lhs = convert_expr(&binary.lhs, ctx)?;
            let rhs = convert_expr(&binary.rhs, ctx)?;
            Ok(ic_expr::Expr::Binary(Box::new(ic_expr::Binary {
                lhs,
                op: binary.op,
                rhs,
            })))
        }
        ic_expr::Expr::Ternary(ternary) => {
            let cond = convert_expr(&ternary.cond, ctx)?;
            let then = convert_expr(&ternary.then, ctx)?;
            let els = convert_expr(&ternary.els, ctx)?;
            Ok(ic_expr::Expr::Ternary(Box::new(ic_expr::Ternary {
                cond,
                then,
                els,
            })))
        }
    }
}

/// Evaluate a preprocessor expression to an integer value
pub fn evaluate_expression(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<i128, Error> {
    // Convert to CLiteral expression
    let converted_expr = convert_expr(expr, ctx)?;

    // Create evaluation context with macro resolver
    let mut eval_ctx = CContext::new(|name| {
        // Handle special identifiers
        match name {
            "__LINE__" => Some(1), // For now, just return 1
            _ => Some(0),          // Undefined macros evaluate to 0
        }
    });

    // Evaluate using ic-expr
    match eval(&converted_expr, &mut eval_ctx) {
        Ok(SimpleInt(value)) => Ok(value),
        Err(ic_expr::Error::DivisionByZero) => {
            // Try to find the specific division operation span
            let span = find_div_mod_span(expr)
                .or_else(|| expr_span(expr))
                .unwrap_or_else(|| panic!("Expression should have at least one token with a span"));
            Err(Error::Expr {
                message: "division by zero",
                span,
            })
        }
        Err(ic_expr::Error::ModuloByZero) => {
            // Try to find the specific modulo operation span
            let span = find_div_mod_span(expr)
                .or_else(|| expr_span(expr))
                .unwrap_or_else(|| panic!("Expression should have at least one token with a span"));
            Err(Error::Expr {
                message: "modulo by zero",
                span,
            })
        }
        Err(ic_expr::Error::InvalidShift(_amount)) => {
            let span = expr_span(expr)
                .unwrap_or_else(|| panic!("Expression should have at least one token with a span"));
            Err(Error::Expr {
                message: "invalid shift amount",
                span,
            })
        }
        Err(_e) => {
            let span = expr_span(expr)
                .unwrap_or_else(|| panic!("Expression should have at least one token with a span"));
            Err(Error::Expr {
                message: "expression evaluation error",
                span,
            })
        }
    }
}

/// Check if an expression evaluates to true (non-zero)
pub fn is_true(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<bool, Error> {
    evaluate_expression(expr, ctx).map(|v| v != 0)
}

/// Parse an integer literal
fn parse_integer(str: &str, base: Base, span: Span) -> Result<i128, Error> {
    let str = match base {
        Base::Octal => {
            if str.len() > 1 {
                str.trim_start_matches('0')
            } else {
                str
            }
        }
        Base::Decimal => str,
        Base::Hexadecimal => str.trim_start_matches("0x"),
    };

    i128::from_str_radix(str, base as u32).map_err(|_| Error::Expr {
        message: "invalid literal",
        span,
    })
}

/// Parse a character literal to its integer value
fn parse_character(lit: &str, span: Span) -> Result<i128, Error> {
    // Remove surrounding quotes
    let content = lit.trim_start_matches('\'').trim_end_matches('\'');

    if content.is_empty() {
        return Err(Error::Expr {
            message: "empty character literal",
            span,
        });
    }

    // Handle escape sequences
    let ch = if content.starts_with('\\') && content.len() > 1 {
        match content.chars().nth(1) {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('0') => '\0',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('"') => '"',
            Some(c) => {
                // For now, just use the character as-is
                // A full implementation would handle octal/hex escapes
                c
            }
            None => {
                return Err(Error::Expr {
                    message: "invalid escape sequence in character literal",
                    span,
                });
            }
        }
    } else if content.len() == 1 {
        content.chars().next().unwrap()
    } else {
        return Err(Error::Expr {
            message: "character literal contains multiple characters",
            span,
        });
    };

    Ok(ch as i128)
}

/// Get the precedence of a binary operator
///
/// Operator precedence is defined as follows, from highest to lowest:
///   1. unary `+`, unary `-`, logical `NOT`, bitwise `NOT`
///   2. multiplication, division, modulo
///   3. addition, subtraction
///   4. `<<`, `>>`
///   5. `<`, `<=`, `>`, `>=`
///   6. `==`, `!=`
///   7. bitwise `AND`
///   8. bitwise `XOR`
///   9. bitwise `OR`
///   10. logical `AND`
///   11. logical `OR`
///   12. ternary conditional
pub fn infix_precedence(kind: Kind) -> Option<u8> {
    match kind {
        Kind::Question => Some(1),
        Kind::Or => Some(2),
        Kind::And => Some(3),
        Kind::BitOr => Some(4),
        Kind::BitXor => Some(5),
        Kind::BitAnd => Some(6),
        Kind::EqEq | Kind::NotEq => Some(7),
        Kind::Gt | Kind::GtEq | Kind::Lt | Kind::LtEq => Some(8),
        Kind::LShift | Kind::RShift => Some(9),
        Kind::Plus | Kind::Minus => Some(10),
        Kind::Star | Kind::Slash | Kind::Modulo => Some(11),
        _ => None,
    }
}

/// Get the precedence of a unary operator
pub fn prefix_precedence(kind: Kind) -> u8 {
    match kind {
        Kind::Plus | Kind::Minus | Kind::Not | Kind::BitNot => 12,
        _ => unreachable!(),
    }
}

/// Convert a token to an expression operator
pub fn expr_op(tok: Token) -> Result<Op, Error> {
    match tok.kind {
        Kind::Plus => Ok(Op::Add),
        Kind::Minus => Ok(Op::Sub),
        Kind::Star => Ok(Op::Mul),
        Kind::Slash => Ok(Op::Div),
        Kind::Modulo => Ok(Op::Mod),
        Kind::Not => Ok(Op::Not),
        Kind::BitNot => Ok(Op::BitNot),
        Kind::BitAnd => Ok(Op::BitAnd),
        Kind::BitOr => Ok(Op::BitOr),
        Kind::BitXor => Ok(Op::BitXor),
        Kind::And => Ok(Op::And),
        Kind::Or => Ok(Op::Or),
        Kind::EqEq => Ok(Op::EqEq),
        Kind::NotEq => Ok(Op::NotEq),
        Kind::Lt => Ok(Op::Lt),
        Kind::LtEq => Ok(Op::LtEq),
        Kind::Gt => Ok(Op::Gt),
        Kind::GtEq => Ok(Op::GtEq),
        Kind::LShift => Ok(Op::LShift),
        Kind::RShift => Ok(Op::RShift),
        _ => Err(Error::Syntax {
            message: "expected operator",
            span: tok.span,
        }),
    }
}
