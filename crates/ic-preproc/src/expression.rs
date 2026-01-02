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

use std::convert::Infallible;

use ic_expr::{ArithError, EvalContext, IntRank, Op, SpannedError, Value, eval};
use ic_lexer::token::{Base, Kind, Token};

use crate::Span;
use crate::state::Error;

/// Preprocessor expression with span tracking.
pub type Expr = ic_expr::Expr<Token, Span>;

/// Expression evaluation context
pub trait ExpressionContext {
    /// Get the source text for a span
    fn source_of(&self, span: Span) -> &str;
}

/// Context for evaluating preprocessor expressions directly on tokens.
struct PreprocEvalContext<'a> {
    expr_ctx: &'a dyn ExpressionContext,
}

impl EvalContext<Token, Infallible, Span> for PreprocEvalContext<'_> {
    fn eval_literal(
        &mut self,
        token: &Token,
        span: Span,
    ) -> Result<Value<Infallible>, SpannedError<Infallible, Span>> {
        let text = self.expr_ctx.source_of(token.span);
        match token.kind {
            Kind::Number { base } => {
                let value = parse_integer(text, base)
                    .map_err(|msg| (ArithError::Custom(msg.to_string()), span))?;
                Ok(Value::Int(value, IntRank::I64))
            }
            Kind::Char => {
                let ch = parse_character(text)
                    .map_err(|msg| (ArithError::Custom(msg.to_string()), span))?;
                Ok(Value::Int(i128::from(ch as u32), IntRank::I32))
            }
            Kind::Ident | Kind::Keyword(_) => {
                let value = match text {
                    "__LINE__" => 1,
                    _ => 0,
                };
                Ok(Value::Int(value, IntRank::I64))
            }
            _ => Err((
                ArithError::Custom(format!("unexpected token kind: {:?}", token.kind)),
                span,
            )),
        }
    }
}

/// Evaluate a preprocessor expression to an integer value
pub fn evaluate_expression(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<i128, Error> {
    let mut eval_ctx = PreprocEvalContext { expr_ctx: ctx };

    match eval(expr, &mut eval_ctx) {
        Ok(value) => value.to_i128().ok_or_else(|| Error::Expr {
            message: "expression did not evaluate to an integer",
            span: expr.span(),
        }),
        Err((e, span)) => {
            let message = match e {
                ArithError::DivByZero => "division by zero",
                ArithError::ModByZero => "modulo by zero",
                ArithError::ShiftOutOfRange(_) => "invalid shift amount",
                _ => "expression evaluation error",
            };
            Err(Error::Expr { message, span })
        }
    }
}

/// Check if an expression evaluates to true (non-zero)
pub fn is_true(expr: &Expr, ctx: &dyn ExpressionContext) -> Result<bool, Error> {
    evaluate_expression(expr, ctx).map(|v| v != 0)
}

/// Parse an integer literal
fn parse_integer(s: &str, base: Base) -> Result<i128, &'static str> {
    let s = match base {
        Base::Octal => {
            let trimmed = s.trim_start_matches('0');
            if trimmed.is_empty() { "0" } else { trimmed }
        }
        Base::Decimal => s,
        Base::Hexadecimal => s.trim_start_matches("0x"),
    };

    i128::from_str_radix(s, base as u32).map_err(|_| "invalid literal")
}

/// Parse a character literal
fn parse_character(lit: &str) -> Result<char, &'static str> {
    if !lit.starts_with('\'') || !lit.ends_with('\'') {
        return Err("character literal must be surrounded by single quotes");
    }
    let content = &lit[1..lit.len() - 1];

    if content.is_empty() {
        return Err("empty character literal");
    }

    if content.starts_with('\\') && content.len() > 1 {
        let escape_char = content.chars().nth(1);
        match escape_char {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('0') => Ok('\0'),
            Some('\\') => Ok('\\'),
            Some('\'') => Ok('\''),
            Some('"') => Ok('"'),
            Some('b') => Ok('\u{0008}'),
            Some('f') => Ok('\u{000C}'),
            Some('v') => Ok('\u{000B}'),
            Some('x') => {
                if content.len() >= 4 {
                    let hex_str = &content[2..4];
                    u8::from_str_radix(hex_str, 16)
                        .map(|v| v as char)
                        .map_err(|_| "invalid hex escape sequence in character literal")
                } else {
                    Err("incomplete hex escape sequence in character literal")
                }
            }
            Some(c) => Ok(c),
            None => Err("invalid escape sequence in character literal"),
        }
    } else if content.len() == 1 {
        Ok(content.chars().next().unwrap())
    } else {
        Err("character literal contains multiple characters")
    }
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
        _ => Err(Error::Syntax {
            message: "expected operator",
            span: tok.span,
        }),
    }
}
