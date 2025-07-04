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

use ic_expr::Op;
use ic_lexer::token::{Base, Kind, Token};

use crate::Span;
use crate::state::Error;

pub type Expr = ic_expr::Expr<Token>;

/// Expression evaluation context
pub trait ExpressionContext {
    /// Get the source text for a span
    fn source_of(&self, span: Span) -> &str;
}

/// Evaluate a preprocessor expression to an integer value
pub fn evaluate_expression(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<i128, Error> {
    match expr {
        Expr::Lit(v) => {
            let lit = ctx.source_of(v.span);
            match v.kind {
                Kind::Number { base } => parse_integer(lit, base, Some(v.span)),
                Kind::Char => parse_character(lit, Some(v.span)),
                Kind::Ident | Kind::Keyword(_) => {
                    // Handle predefined macros in expressions
                    match lit {
                        "__LINE__" => {
                            // For now, just return a non-zero value so #if __LINE__ > 0 works
                            // Proper implementation would require access to span location info
                            Ok(1)
                        }
                        _ => Ok(0), // Undefined macros evaluate to 0
                    }
                }
                _ => unreachable!(),
            }
        }
        Expr::Unary(v) => {
            let expr = evaluate_expression(&v.expr, ctx)?;
            match v.op {
                Op::Add => Ok(expr),
                Op::Sub => Ok(-expr),
                Op::Not => Ok(i128::from(expr == 0)),
                Op::BitNot => Ok(!expr),
                v => unreachable!("invalid unary operator: {v:?}"),
            }
        }
        Expr::Binary(v) => {
            let lhs = evaluate_expression(&v.lhs, ctx)?;
            let rhs = evaluate_expression(&v.rhs, ctx)?;
            match v.op {
                Op::And => Ok(i128::from(lhs != 0 && rhs != 0)),
                Op::Or => Ok(i128::from(lhs != 0 || rhs != 0)),
                Op::EqEq => Ok(i128::from(lhs == rhs)),
                Op::NotEq => Ok(i128::from(lhs != rhs)),
                Op::Gt => Ok(i128::from(lhs > rhs)),
                Op::GtEq => Ok(i128::from(lhs >= rhs)),
                Op::Lt => Ok(i128::from(lhs < rhs)),
                Op::LtEq => Ok(i128::from(lhs <= rhs)),
                Op::BitAnd => Ok(lhs & rhs),
                Op::BitOr => Ok(lhs | rhs),
                Op::BitXor => Ok(lhs ^ rhs),
                Op::Add => Ok(lhs.wrapping_add(rhs)),
                Op::Sub => Ok(lhs.wrapping_sub(rhs)),
                Op::Mul => Ok(lhs.wrapping_mul(rhs)),
                Op::Div => checked_div(lhs, rhs),
                Op::Mod => checked_mod(lhs, rhs),
                Op::LShift => Ok(lhs.wrapping_shl(rhs.try_into().unwrap_or(128))),
                Op::RShift => Ok(lhs.wrapping_shr(rhs.try_into().unwrap_or(128))),
                v => unreachable!("invalid binary operator: {v:?}"),
            }
        }
        Expr::Ternary(v) => {
            if is_true(&v.cond, ctx)? {
                evaluate_expression(&v.then, ctx)
            } else {
                evaluate_expression(&v.els, ctx)
            }
        }
    }
}

/// Check if an expression evaluates to true (non-zero)
pub fn is_true(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<bool, Error> {
    evaluate_expression(expr, ctx).map(|v| v != 0)
}

/// Parse an integer literal
fn parse_integer(str: &str, base: Base, span: Option<Span>) -> Result<i128, Error> {
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

/// Checked division with proper error handling
fn checked_div(lhs: i128, rhs: i128) -> Result<i128, Error> {
    if rhs == 0 {
        Err(Error::Expr {
            message: "division by zero",
            span: None,
        })
    } else {
        Ok(lhs.wrapping_div(rhs))
    }
}

/// Checked modulo with proper error handling
fn checked_mod(lhs: i128, rhs: i128) -> Result<i128, Error> {
    if rhs == 0 {
        Err(Error::Expr {
            message: "modulo by zero",
            span: None,
        })
    } else {
        Ok(lhs.wrapping_rem(rhs))
    }
}

/// Parse a character literal to its integer value
fn parse_character(lit: &str, span: Option<Span>) -> Result<i128, Error> {
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
