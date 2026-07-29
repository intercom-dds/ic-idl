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

use ic_lexer::token::{Base, Kind, Kw};
use ic_syntax::{BinaryExpr, Expr, ExprKind, Literal, Op, Spanned, UnaryExpr};
use ic_vfs::Span;

use super::Parser;
use crate::error::Result;

/// Precedence levels for binary operators (higher = binds tighter).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Precedence(u8);

impl Precedence {
    const NONE: Self = Self(0);
    const OR: Self = Self(1);
    const XOR: Self = Self(2);
    const AND: Self = Self(3);
    const SHIFT: Self = Self(4);
    const ADD: Self = Self(5);
    const MUL: Self = Self(6);
}

/// Controls how shift operators (`<<`, `>>`) are handled during parsing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ShiftMode {
    /// Shift operators are always recognized.
    Allow,

    /// Template argument context: `>>` uses lookahead to disambiguate.
    Template { allow_comma: bool },
}

impl Parser<'_> {
    // Rule 7
    // <const_expr> ::= <or_expr>
    // Parses a full constant expression including shift operators.
    pub fn const_expr(&mut self) -> Result<Expr> {
        self.expr_bp(Precedence::NONE, ShiftMode::Allow)
    }

    /// Expression parser for template bounds.
    pub(super) fn bound_expr(&mut self, allow_comma: bool) -> Result<Expr> {
        self.expr_bp(Precedence::NONE, ShiftMode::Template { allow_comma })
    }

    /// Pratt parser: parse expression with minimum binding power.
    /// Implements Rules 8-13 via precedence climbing.
    fn expr_bp(&mut self, min_prec: Precedence, shift_mode: ShiftMode) -> Result<Expr> {
        let mut lhs = self.unary_expr(shift_mode)?;
        loop {
            // In template mode, comma is a hard terminator (expression ends at comma)
            if matches!(shift_mode, ShiftMode::Template { .. }) && self.peek() == Kind::Comma {
                break;
            }

            let Some((op_kind, prec)) = self.infix_op(shift_mode) else {
                break;
            };

            if prec <= min_prec {
                break;
            }

            let op_span = self.consume_infix_op(op_kind);
            let rhs = self.expr_bp(prec, shift_mode)?;
            let span = self.make_span(lhs.span, rhs.span);
            lhs = Spanned {
                span,
                value: ExprKind::Binary(Box::new(BinaryExpr {
                    lhs,
                    op: Spanned {
                        span: op_span,
                        value: op_kind,
                    },
                    rhs,
                })),
            };
        }

        Ok(lhs)
    }

    // Rule 14
    // <unary_expr> ::= <unary_operator> <primary_expr> | <primary_expr>
    fn unary_expr(&mut self, shift_mode: ShiftMode) -> Result<Expr> {
        if let Some(op) = self.unary_operator() {
            let expr = self.unary_expr(shift_mode)?;
            Ok(Spanned {
                span: self.make_span(op.span, expr.span),
                value: ExprKind::Unary(Box::new(UnaryExpr { op, operand: expr })),
            })
        } else {
            self.primary_expr(shift_mode)
        }
    }

    // Rule 15
    // <unary_operator> ::= "-" | "+" | "~"
    fn unary_operator(&mut self) -> Option<Spanned<Op>> {
        match self.peek() {
            Kind::Minus => {
                let tok = self.advance();
                Some(Spanned {
                    span: tok.span,
                    value: Op::Sub,
                })
            }
            Kind::Plus => {
                let tok = self.advance();
                Some(Spanned {
                    span: tok.span,
                    value: Op::Add,
                })
            }
            Kind::BitNot => {
                let tok = self.advance();
                Some(Spanned {
                    span: tok.span,
                    value: Op::Not,
                })
            }
            _ => None,
        }
    }

    // Rule 16
    // <primary_expr> ::= <scoped_name> | <literal> | "(" <const_expr> ")"
    fn primary_expr(&mut self, shift_mode: ShiftMode) -> Result<Expr> {
        let kind = self.peek();

        match kind {
            // Literal (Rule 17)
            Kind::Number { .. }
            | Kind::Float
            | Kind::String { .. }
            | Kind::WString { .. }
            | Kind::Char
            | Kind::WChar => self.literal(),
            Kind::Keyword(Kw::True | Kw::False) => Ok(self.boolean_literal()),

            // Scoped name
            Kind::Ident | Kind::DColon => {
                let path = self.scoped_name()?;
                Ok(Spanned {
                    span: ic_syntax::util::path_span(&path),
                    value: ExprKind::Path(path),
                })
            }

            Kind::LParen => {
                let start = self.span();
                self.advance();
                let inner = self.const_expr()?;
                self.expect(Kind::RParen)?;
                Ok(Spanned {
                    span: self.make_span(start, self.prev_span),
                    value: ExprKind::Group(Box::new(inner)),
                })
            }

            Kind::LBrace => self.init_list(shift_mode),

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
            Kind::String { .. } => self.string_literal(false),
            Kind::WString { .. } => self.string_literal(true),
            Kind::Char => self.character_literal(false),
            Kind::WChar => self.character_literal(true),
            Kind::Keyword(Kw::True | Kw::False) => Ok(self.boolean_literal()),
            _ => Err(self.error_expected("literal")),
        }
    }

    // Rule 18
    // <integer_literal> ::= <decimal_integer> | <octal_integer> | <hexadecimal_integer>
    fn integer_literal(&mut self, base: Base) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let Some(value) = parse_integer(text, base) else {
            return Err(self
                .error_message(tok.span, "invalid integer literal")
                .with_label("invalid integer"));
        };

        Ok(Spanned {
            span: tok.span,
            value: ExprKind::Literal(Literal::Int(value)),
        })
    }

    // Rule 18 (floating point variant)
    fn floating_pt_literal(&mut self) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let Ok(value) = text.parse() else {
            return Err(self
                .error_message(tok.span, "invalid floating point literal")
                .with_label("invalid float"));
        };

        Ok(Spanned {
            span: tok.span,
            value: ExprKind::Literal(Literal::Float(value)),
        })
    }

    // Rule 19
    // <string_literal> ::= <string_literal>+
    fn string_literal(&mut self, wide: bool) -> Result<Expr> {
        let start = self.span();
        let mut value = String::new();

        // Collect all adjacent string literals of the same width.
        loop {
            let terminated = match self.peek() {
                Kind::String { terminated } if !wide => terminated,
                Kind::WString { terminated } if wide => terminated,
                _ => break,
            };
            if !terminated {
                // Consume the bad token
                let tok = self.advance();
                return Err(self
                    .error_message(tok.span, "unterminated string literal")
                    .with_label("unterminated string"));
            }

            let tok = self.advance();
            let text = self.text(tok.span);
            let body = if wide {
                text.strip_prefix('L').unwrap_or(text)
            } else {
                text
            };
            let Some(parsed) = parse_string_literal(body) else {
                return Err(self
                    .error_message(tok.span, "invalid escape sequence in string literal")
                    .with_label("invalid escape sequence"));
            };
            value.push_str(&parsed);
        }

        let span = self.make_span(start, self.prev_span);
        Ok(Spanned {
            span,
            value: ExprKind::Literal(if wide {
                Literal::WString(value)
            } else {
                Literal::String(value)
            }),
        })
    }

    // Rule 20
    // <character_literal> ::= "'" <char> "'"
    fn character_literal(&mut self, wide: bool) -> Result<Expr> {
        let tok = self.advance();
        let text = self.text(tok.span);
        let body = if wide {
            text.strip_prefix('L').unwrap_or(text)
        } else {
            text
        };
        let Some(value) = parse_char_literal(body) else {
            return Err(self
                .error_message(tok.span, "invalid character literal")
                .with_label("invalid character"));
        };

        Ok(Spanned {
            span: tok.span,
            value: ExprKind::Literal(if wide {
                Literal::WChar(value)
            } else {
                Literal::Char(value)
            }),
        })
    }

    // Rule 17
    fn boolean_literal(&mut self) -> Expr {
        let tok = self.advance();
        let value = matches!(tok.kind, Kind::Keyword(Kw::True));
        Spanned {
            span: tok.span,
            value: ExprKind::Literal(Literal::Bool(value)),
        }
    }

    /// Check if current token starts an infix operator; return its kind and precedence.
    fn infix_op(&mut self, shift_mode: ShiftMode) -> Option<(Op, Precedence)> {
        match self.peek() {
            // Rule 8: or_expr
            Kind::BitOr => Some((Op::Or, Precedence::OR)),

            // Rule 9: xor_expr
            Kind::BitXor => Some((Op::Xor, Precedence::XOR)),

            // Rule 10: and_expr
            Kind::BitAnd => Some((Op::And, Precedence::AND)),

            // Rule 12: add_expr
            Kind::Plus => Some((Op::Add, Precedence::ADD)),
            Kind::Minus => Some((Op::Sub, Precedence::ADD)),

            // Rule 13: mult_expr
            Kind::Star => Some((Op::Multiply, Precedence::MUL)),
            Kind::Slash => Some((Op::Divide, Precedence::MUL)),
            Kind::Modulo => Some((Op::Modulo, Precedence::MUL)),

            // Rule 11: shift_expr
            Kind::Lt if self.peek_nth_raw(1) == Kind::Lt => Some((Op::LShift, Precedence::SHIFT)),

            Kind::Gt if self.peek_nth_raw(1) == Kind::Gt => match shift_mode {
                ShiftMode::Allow => Some((Op::RShift, Precedence::SHIFT)),
                ShiftMode::Template { allow_comma } => self.rshift_in_template(allow_comma),
            },

            _ => None,
        }
    }

    /// Disambiguate `>>` in template argument context using lookahead.
    ///
    /// Uses lookahead to properly skim annotations (with correct adjacency rules)
    /// and determine what follows the `>>`.
    fn rshift_in_template(&mut self, allow_comma: bool) -> Option<(Op, Precedence)> {
        let is_rshift = self.lookahead(|p| {
            // Consume the two `>` tokens
            p.advance_raw();
            p.advance_raw();

            // Skim annotations using the real annotation parser
            p.skim_annotations();

            // Check what follows
            let after_shift = p.peek_raw();
            p.is_definite_expr_continuation(after_shift)
                || (matches!(after_shift, Kind::Ident | Kind::DColon)
                    && p.identifier_continues_expression(allow_comma))
        });

        if is_rshift {
            Some((Op::RShift, Precedence::SHIFT))
        } else {
            None
        }
    }

    /// Returns true if the given token definitely continues an expression.
    fn is_definite_expr_continuation(&self, kind: Kind) -> bool {
        matches!(
            kind,
            Kind::Number { .. }
                | Kind::Float
                | Kind::String { .. }
                | Kind::WString { .. }
                | Kind::Char
                | Kind::WChar
                | Kind::Keyword(Kw::True | Kw::False)
                | Kind::Minus
                | Kind::Plus
                | Kind::BitNot
                | Kind::LParen
                | Kind::LBrace
        )
    }

    /// Check if the current identifier continues an expression (for `>>` disambiguation).
    fn identifier_continues_expression(&mut self, allow_comma: bool) -> bool {
        self.lookahead(|p| {
            // Skip over the scoped name
            p.skip_scoped_name_tokens();

            // Skim any annotations after the identifier
            p.skim_annotations();

            let after_ident = p.peek_raw();
            matches!(
                after_ident,
                Kind::Plus
                    | Kind::Minus
                    | Kind::Star
                    | Kind::Slash
                    | Kind::Modulo
                    | Kind::BitOr
                    | Kind::BitXor
                    | Kind::BitAnd
                    | Kind::Lt
                    | Kind::Gt
                    | Kind::DColon
            ) || (allow_comma && after_ident == Kind::Comma)
        })
    }

    /// Consumes tokens that form a scoped name (e.g., `foo`, `::foo`, `foo::bar::baz`).
    fn skip_scoped_name_tokens(&mut self) {
        // Handle leading ::
        if self.peek_raw() == Kind::DColon {
            self.advance_raw();
        }

        // Must have at least one identifier
        if self.peek_raw() == Kind::Ident {
            self.advance_raw();
        } else {
            return;
        }

        // Skip additional ::ident segments
        while self.peek_raw() == Kind::DColon {
            self.advance_raw();
            if self.peek_raw() == Kind::Ident {
                self.advance_raw();
            } else {
                break;
            }
        }
    }

    fn consume_infix_op(&mut self, op_kind: Op) -> Span {
        match op_kind {
            Op::LShift | Op::RShift => {
                let start = self.advance().span;
                let end = self.advance().span;
                self.make_span(start, end)
            }
            _ => self.advance().span,
        }
    }

    fn init_list(&mut self, shift_mode: ShiftMode) -> Result<Expr> {
        let start = self.span();
        self.expect(Kind::LBrace)?;

        let mut values = Vec::new();

        if !self.at(Kind::RBrace) {
            values.push(self.init_list_element(shift_mode)?);
            while self.at(Kind::Comma) {
                let comma_span = self.advance().span;
                if self.at(Kind::RBrace) {
                    return Err(self.error_message(comma_span, "trailing comma is not allowed"));
                }
                values.push(self.init_list_element(shift_mode)?);
            }
        }

        self.expect(Kind::RBrace)?;

        Ok(Spanned {
            span: self.make_span(start, self.prev_span),
            value: ExprKind::InitList(values),
        })
    }

    /// Parses a single init list element: `expr`, `.field = expr`, or `field = expr`
    fn init_list_element(&mut self, shift_mode: ShiftMode) -> Result<ic_syntax::NamedExpr> {
        let ident = if self.eat(Kind::Period) {
            // `.field = expr`
            let id = self.ident()?;
            self.expect(Kind::Eq)?;
            Some(id)
        } else if self.peek() == Kind::Ident && self.peek_nth_raw(1) == Kind::Eq {
            // `field = expr`
            let id = self.ident()?;
            // consume '='
            self.advance();
            Some(id)
        } else {
            None
        };

        let value = self.expr_bp(Precedence::NONE, shift_mode)?;
        Ok(ic_syntax::NamedExpr { name: ident, value })
    }
}

// Rule 18
fn parse_integer(text: &str, base: Base) -> Option<u64> {
    let text = text.replace('_', "");
    let (text, radix) = match base {
        Base::Octal => (text.trim_start_matches('0'), 8),
        Base::Decimal => (text.as_str(), 10),
        Base::Hexadecimal => {
            let stripped = text.trim_start_matches("0x").trim_start_matches("0X");
            // "0x" with no digits is invalid
            if stripped.is_empty() {
                return None;
            }
            (stripped, 16)
        }
    };

    // For octal, empty string after stripping leading zeros means "0"
    if text.is_empty() {
        return Some(0);
    }
    u64::from_str_radix(text, radix).ok()
}

// Rule 19
fn parse_string_literal(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        let escaped = match chars.next()? {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'a' => '\u{07}',
            'b' => '\u{08}',
            'f' => '\u{0C}',
            'v' => '\u{0B}',
            '?' => '?',
            'x' => {
                let hex: String = [chars.next()?, chars.next()?].into_iter().collect();
                u8::from_str_radix(&hex, 16).ok()? as char
            }
            'u' => {
                let hex: String = [chars.next()?, chars.next()?, chars.next()?, chars.next()?]
                    .into_iter()
                    .collect();
                char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?
            }
            c @ '0'..='7' => {
                let mut octal = String::from(c);
                while octal.len() < 3
                    && let Some(c) = chars.peek()
                    && ('0'..='7').contains(c)
                {
                    octal.push(chars.next()?);
                }
                u8::from_str_radix(&octal, 8).ok()? as char
            }
            _ => return None,
        };
        result.push(escaped);
    }

    Some(result)
}

// Rule 20
fn parse_char_literal(text: &str) -> Option<char> {
    let inner = if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        return None;
    };

    if inner.is_empty() {
        return None;
    }

    if inner.starts_with('\\') && inner.len() > 1 {
        let c = match inner.chars().nth(1) {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('0') => '\0',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('"') => '"',
            Some('b') => '\u{0008}',
            Some('f') => '\u{000C}',
            Some('v') => '\u{000B}',
            Some('x') if inner.len() >= 4 => {
                let hex_str = &inner[2..4];
                u8::from_str_radix(hex_str, 16).ok().map(|v| v as char)?
            }
            // unknown escape sequence
            _ => return None,
        };
        Some(c)
    } else {
        // for non-escape sequences, must be exactly one character
        let mut chars = inner.chars();
        let first = chars.next()?;

        // ff there's a second character, it's invalid
        if chars.next().is_some() {
            return None;
        }
        Some(first)
    }
}
