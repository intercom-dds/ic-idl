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

//! Expression parsing using Pratt parsing (precedence climbing).
//!
//! IDL expression precedence (lowest to highest):
//!   Rule 8:  Bitwise OR (`|`)
//!   Rule 9:  Bitwise XOR (`^`)
//!   Rule 10: Bitwise AND (`&`)
//!   Rule 11: Shift (`<<`, `>>`)
//!   Rule 12: Add/Sub (`+`, `-`)
//!   Rule 13: Mul/Div/Mod (`*`, `/`, `%`)
//!   Rule 14: Unary (`+`, `-`, `~`)
//!   Rule 16: Primary (literals, paths, parenthesized)

use ic_lexer::token::{Base, Kind, Kw};
use ic_syntax::{Binary, Expr, Group, Literal, LiteralValue, Op, OpKind, Unary};
use ic_vfs::Span;

use super::Parser;
use crate::error::Result;

/// Precedence levels for binary operators (higher = binds tighter).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Precedence(u8);

impl Precedence {
    const NONE: Self = Self(0);
    const OR: Self = Self(1); // Rule 8: |
    const XOR: Self = Self(2); // Rule 9: ^
    const AND: Self = Self(3); // Rule 10: &
    const SHIFT: Self = Self(4); // Rule 11: <<, >>
    const ADD: Self = Self(5); // Rule 12: +, -
    const MUL: Self = Self(6); // Rule 13: *, /, %
    #[allow(dead_code)]
    const UNARY: Self = Self(7); // Rule 14: unary +, -, ~
}

impl Parser<'_> {
    // Rule 7
    // <const_expr> ::= <or_expr>
    // Parses a full constant expression including shift operators.
    pub fn const_expr(&mut self) -> Result<Expr> {
        self.expr_bp_with_shifts(Precedence::NONE, true)
    }

    // Expression parser for bounds inside templates (sequence<T, N>, map<K, V, N>).
    // Does NOT parse shift operators at the top level to avoid consuming `>>`
    // as a right-shift when it's actually two closing angle brackets.
    // Shift operators ARE allowed inside parenthesized expressions.
    pub(super) fn bound_expr(&mut self) -> Result<Expr> {
        self.expr_bp_with_shifts(Precedence::NONE, false)
    }

    // Rule 8
    // <or_expr> ::= <xor_expr> | <or_expr> "|" <xor_expr>
    #[allow(dead_code)]
    fn or_expr(&mut self) -> Result<Expr> {
        self.expr_bp_with_shifts(Precedence::NONE, true)
    }

    /// Pratt parser: parse expression with minimum binding power.
    /// Implements Rules 8-13 via precedence climbing.
    ///
    /// `allow_shifts`: if false, shift operators (`<<`, `>>`) are not recognized
    /// at the top level. This is used when parsing bounds inside template types
    /// like `sequence<T, N>` to avoid consuming `>>` as a shift operator.
    fn expr_bp_with_shifts(&mut self, min_prec: Precedence, allow_shifts: bool) -> Result<Expr> {
        // Rule 14: Parse prefix (unary operators or primary expression)
        let mut lhs = self.unary_expr_with_shifts(allow_shifts)?;

        // Parse infix operators (Rules 8-13)
        while let Some((op_kind, prec)) = self.infix_op_with_shifts(allow_shifts) {
            if prec <= min_prec {
                break;
            }

            // Consume the operator
            let op_span = self.consume_infix_op(op_kind);

            // Parse right-hand side with higher precedence (left-associative)
            let rhs = self.expr_bp_with_shifts(prec, allow_shifts)?;

            lhs = Expr::Binary(Box::new(Binary {
                lhs,
                op: Op {
                    span: op_span,
                    kind: op_kind,
                },
                rhs,
            }));
        }

        Ok(lhs)
    }

    // Rule 14
    // <unary_expr> ::= <unary_operator> <primary_expr> | <primary_expr>
    fn unary_expr_with_shifts(&mut self, allow_shifts: bool) -> Result<Expr> {
        if let Some(op) = self.unary_operator() {
            let expr = self.unary_expr_with_shifts(allow_shifts)?;
            Ok(Expr::Unary(Box::new(Unary { op, expr })))
        } else {
            self.primary_expr_with_shifts(allow_shifts)
        }
    }

    // Rule 15
    // <unary_operator> ::= "-" | "+" | "~"
    fn unary_operator(&mut self) -> Option<Op> {
        match self.peek() {
            Kind::Minus => {
                let tok = self.advance();
                Some(Op {
                    span: tok.span,
                    kind: OpKind::Sub,
                })
            }
            Kind::Plus => {
                let tok = self.advance();
                Some(Op {
                    span: tok.span,
                    kind: OpKind::Add,
                })
            }
            Kind::BitNot => {
                let tok = self.advance();
                Some(Op {
                    span: tok.span,
                    kind: OpKind::Not,
                })
            }
            _ => None,
        }
    }

    // Rule 16
    // <primary_expr> ::= <scoped_name> | <literal> | "(" <const_expr> ")"
    fn primary_expr_with_shifts(&mut self, allow_shifts: bool) -> Result<Expr> {
        let kind = self.peek();

        match kind {
            // Literal (Rule 17)
            Kind::Number { .. } | Kind::Float | Kind::String { .. } | Kind::Char => self.literal(),
            Kind::Keyword(Kw::True | Kw::False) => self.boolean_literal(),
            Kind::Keyword(Kw::Null) => {
                let tok = self.advance();
                Ok(Expr::Literal(Literal {
                    span: tok.span,
                    value: LiteralValue::Null,
                }))
            }

            // Scoped name
            Kind::Ident | Kind::DColon => {
                let path = self.scoped_name()?;
                Ok(Expr::Path(path))
            }

            // Parenthesized expression - always allow shifts inside parens
            Kind::LParen => {
                let start = self.span();
                self.advance(); // consume '('
                // Inside parentheses, we always allow shift operators
                let inner = self.const_expr()?;
                self.expect(Kind::RParen)?;
                Ok(Expr::Group(Box::new(Group {
                    expr: inner,
                    span: self.make_span(start, self.prev_span),
                })))
            }

            // Init list (DDS-XTypes extension for struct/array initialization)
            Kind::LBrace => self.init_list_with_shifts(allow_shifts),

            _ => Err(self.error_expected("expression")),
        }
    }

    // Rule 17
    // <literal> ::= <integer_literal> | <floating_pt_literal> | <character_literal>
    //             | <string_literal> | <boolean_literal>
    fn literal(&mut self) -> Result<Expr> {
        match self.peek() {
            Kind::Number { base } => self.integer_literal(base),
            Kind::Float => self.floating_pt_literal(),
            Kind::String { .. } => self.string_literal(),
            Kind::Char => self.character_literal(),
            Kind::Keyword(Kw::True | Kw::False) => self.boolean_literal(),
            _ => Err(self.error_expected("literal")),
        }
    }

    // Rule 18
    // <integer_literal> ::= <decimal_integer> | <octal_integer> | <hexadecimal_integer>
    fn integer_literal(&mut self, base: Base) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let value = parse_integer(text, base);
        Ok(Expr::Literal(Literal {
            span: tok.span,
            value: LiteralValue::Int(value),
        }))
    }

    // Rule 18 (floating point variant)
    fn floating_pt_literal(&mut self) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let value: f64 = text.parse().unwrap_or(0.0);
        Ok(Expr::Literal(Literal {
            span: tok.span,
            value: LiteralValue::Float(value),
        }))
    }

    // Rule 19
    // <string_literal> ::= <string_literal>+
    fn string_literal(&mut self) -> Result<Expr> {
        let start = self.span();
        let mut value = String::new();

        // Collect all adjacent string literals (concatenation)
        while let Kind::String { .. } = self.peek() {
            let tok = self.advance();
            let text = self.text(tok.span);
            value.push_str(&parse_string_literal(text));
        }

        Ok(Expr::Literal(Literal {
            span: self.make_span(start, self.prev_span),
            value: LiteralValue::String(value),
        }))
    }

    // Rule 20
    // <character_literal> ::= "'" <char> "'"
    fn character_literal(&mut self) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let value = parse_char_literal(text);
        Ok(Expr::Literal(Literal {
            span: tok.span,
            value: LiteralValue::Char(value),
        }))
    }

    // Rule 17 (boolean literal variant)
    fn boolean_literal(&mut self) -> Result<Expr> {
        let tok = self.advance();
        let value = match tok.kind {
            Kind::Keyword(Kw::True) => true,
            Kind::Keyword(Kw::False) => false,
            _ => unreachable!(),
        };
        Ok(Expr::Literal(Literal {
            span: tok.span,
            value: LiteralValue::Bool(value),
        }))
    }

    /// Check if current token starts an infix operator; return its kind and precedence.
    /// If `allow_shifts` is false, shift operators are not recognized.
    fn infix_op_with_shifts(&mut self, allow_shifts: bool) -> Option<(OpKind, Precedence)> {
        match self.peek() {
            // Rule 8: or_expr
            Kind::BitOr => Some((OpKind::Or, Precedence::OR)),
            // Rule 9: xor_expr
            Kind::BitXor => Some((OpKind::Xor, Precedence::XOR)),
            // Rule 10: and_expr
            Kind::BitAnd => Some((OpKind::And, Precedence::AND)),
            // Rule 12: add_expr
            Kind::Plus => Some((OpKind::Add, Precedence::ADD)),
            Kind::Minus => Some((OpKind::Sub, Precedence::ADD)),
            // Rule 13: mult_expr
            Kind::Star => Some((OpKind::Multiply, Precedence::MUL)),
            Kind::Slash => Some((OpKind::Divide, Precedence::MUL)),
            Kind::Modulo => Some((OpKind::Modulo, Precedence::MUL)),
            // Rule 11: shift_expr - << and >> are parsed as two consecutive tokens
            // Only recognize these if allow_shifts is true
            Kind::Lt if allow_shifts => {
                if self.peek_nth_raw(1) == Kind::Lt {
                    Some((OpKind::Lshift, Precedence::SHIFT))
                } else {
                    None
                }
            }
            Kind::Gt if allow_shifts => {
                if self.peek_nth_raw(1) == Kind::Gt {
                    Some((OpKind::Rshift, Precedence::SHIFT))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Consume the infix operator token(s) and return the combined span.
    fn consume_infix_op(&mut self, op_kind: OpKind) -> Span {
        match op_kind {
            OpKind::Lshift | OpKind::Rshift => {
                // Consume two tokens for shift operators
                let start = self.advance().span;
                let end = self.advance().span;
                self.make_span(start, end)
            }
            _ => {
                // Single token operators
                self.advance().span
            }
        }
    }

    // DDS-XTypes extension: initializer list
    fn init_list_with_shifts(&mut self, allow_shifts: bool) -> Result<Expr> {
        let start = self.span();
        self.expect(Kind::LBrace)?;

        let mut values = Vec::new();

        if !self.at(Kind::RBrace) {
            // Parse first element
            values.push(self.init_list_element_with_shifts(allow_shifts)?);

            // Parse remaining elements
            while self.at(Kind::Comma) {
                let comma_span = self.advance().span;
                if self.at(Kind::RBrace) {
                    return Err(self.error_message(comma_span, "trailing comma is not allowed"));
                }
                values.push(self.init_list_element_with_shifts(allow_shifts)?);
            }
        }

        self.expect(Kind::RBrace)?;

        Ok(Expr::InitList(ic_syntax::InitList {
            values,
            span: self.make_span(start, self.prev_span),
        }))
    }

    /// Parses a single init list element: `expr`, `.field = expr`, or `field = expr`
    fn init_list_element_with_shifts(
        &mut self,
        allow_shifts: bool,
    ) -> Result<ic_syntax::NamedExpr> {
        // Check for `.field = expr` or `field = expr` syntax
        let ident = if self.eat(Kind::Period) {
            // `.field = expr` - period is required before identifier
            let id = self.ident()?;
            self.expect(Kind::Eq)?;
            Some(id)
        } else if self.peek() == Kind::Ident && self.peek_nth_raw(1) == Kind::Eq {
            // `field = expr` - identifier followed by equals
            let id = self.ident()?;
            self.advance(); // consume '='
            Some(id)
        } else {
            None
        };

        let value = self.expr_bp_with_shifts(Precedence::NONE, allow_shifts)?;
        Ok(ic_syntax::NamedExpr { ident, value })
    }
}

// Rule 18: Parse integer literal
fn parse_integer(text: &str, base: Base) -> u64 {
    let text = text.replace('_', ""); // Remove underscores
    let (text, radix) = match base {
        Base::Octal => (text.trim_start_matches('0'), 8),
        Base::Decimal => (text.as_str(), 10),
        Base::Hexadecimal => (text.trim_start_matches("0x").trim_start_matches("0X"), 16),
    };
    u64::from_str_radix(text, radix).unwrap_or(0)
}

// Rule 19: Parse string literal
// NOTE: Escape sequences are NOT processed here - they're stored raw.
// Processing happens at a later stage (evaluation/codegen).
fn parse_string_literal(text: &str) -> String {
    // Remove surrounding quotes only
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

// Rule 20: Parse character literal
fn parse_char_literal(text: &str) -> char {
    // Remove surrounding quotes
    let inner = if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        text
    };

    if inner.is_empty() {
        return '\0';
    }

    // Handle escape sequences
    if inner.starts_with('\\') && inner.len() > 1 {
        match inner.chars().nth(1) {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('0') | None => '\0',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('"') => '"',
            Some('b') => '\u{0008}', // backspace
            Some('f') => '\u{000C}', // form feed
            Some('v') => '\u{000B}', // vertical tab
            Some('x') => {
                // Handle hex escape sequences like \x41
                if inner.len() >= 4 {
                    let hex_str = &inner[2..4];
                    u8::from_str_radix(hex_str, 16)
                        .map(|v| v as char)
                        .unwrap_or('\0')
                } else {
                    '\0'
                }
            }
            Some(c) => c, // Unknown escape - return the character itself
        }
    } else if inner.len() == 1 {
        inner.chars().next().unwrap_or('\0')
    } else {
        // Multiple characters - just take the first one
        inner.chars().next().unwrap_or('\0')
    }
}
