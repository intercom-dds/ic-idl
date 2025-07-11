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

//! C preprocessor expression evaluation adapter
//!
//! This module provides types and functions to evaluate C preprocessor
//! expressions using the generic expression evaluation framework.

use crate::{EvalConfig, EvalContext, OverflowBehavior, Result, SimpleInt};

/// Number base for parsing integer literals
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Base {
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

/// Literal values that can appear in C preprocessor expressions
#[derive(Debug, Clone)]
pub enum CLiteral {
    /// Integer literal
    Int(i128),

    /// Character literal
    Char(char),

    /// Identifier (may be a macro or special identifier like __LINE__)
    Ident(String),
}

/// Parse an integer literal from C source
///
/// # Errors
///
/// Returns an error if the string is not a valid integer in the given base
pub fn parse_integer(str: &str, base: Base) -> std::result::Result<i128, &'static str> {
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

    i128::from_str_radix(str, base as u32).map_err(|_| "invalid literal")
}

/// Parse a character literal to its integer value
///
/// # Errors
///
/// Returns an error if the string is not a valid character literal
///
/// # Panics
///
/// Panics if the content is empty after stripping quotes and processing escapes
pub fn parse_character(lit: &str) -> std::result::Result<char, &'static str> {
    // Remove surrounding quotes
    let content = lit.trim_start_matches('\'').trim_end_matches('\'');

    if content.is_empty() {
        return Err("empty character literal");
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
            Some(c) => c,
            None => {
                return Err("invalid escape sequence in character literal");
            }
        }
    } else if content.len() == 1 {
        content.chars().next().unwrap()
    } else {
        return Err("character literal contains multiple characters");
    };

    Ok(ch)
}

/// Function type for identifier resolution
type IdentResolver<'a> = Box<dyn FnMut(&str) -> Option<i128> + 'a>;

/// Context for evaluating C preprocessor expressions
pub struct CContext<'a> {
    /// Configuration for evaluation
    config: EvalConfig,
    /// Callback to resolve identifiers
    resolve_ident: IdentResolver<'a>,
}

impl<'a> CContext<'a> {
    /// Create a new C evaluation context
    pub fn new<F>(resolve_ident: F) -> Self
    where
        F: FnMut(&str) -> Option<i128> + 'a,
    {
        Self {
            config: EvalConfig {
                overflow: OverflowBehavior::Wrap,
                max_shift: 127,
            },
            resolve_ident: Box::new(resolve_ident),
        }
    }

    /// Create a context with custom configuration
    pub fn with_config<F>(config: EvalConfig, resolve_ident: F) -> Self
    where
        F: FnMut(&str) -> Option<i128> + 'a,
    {
        Self {
            config,
            resolve_ident: Box::new(resolve_ident),
        }
    }
}

impl EvalContext<CLiteral> for CContext<'_> {
    type Value = SimpleInt;

    fn eval_literal(&mut self, lit: &CLiteral) -> Result<Self::Value> {
        match lit {
            CLiteral::Int(n) => Ok(SimpleInt(*n)),
            CLiteral::Char(c) => Ok(SimpleInt(*c as i128)),
            CLiteral::Ident(name) => {
                // Try to resolve the identifier
                let value = (self.resolve_ident)(name).unwrap_or(0);
                Ok(SimpleInt(value))
            }
        }
    }

    fn config(&self) -> EvalConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Binary, Expr, Op, Unary, eval};

    #[test]
    fn test_simple_arithmetic() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(10)),
            op: Op::Add,
            rhs: Expr::Lit(CLiteral::Int(20)),
        }));

        let mut ctx = CContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result.0, 30);
    }

    #[test]
    fn test_division_by_zero() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(10)),
            op: Op::Div,
            rhs: Expr::Lit(CLiteral::Int(0)),
        }));

        let mut ctx = CContext::new(|_| None);
        let result = eval(&expr, &mut ctx);
        assert!(matches!(result, Err(crate::Error::DivisionByZero)));
    }

    #[test]
    fn test_macro_resolution() {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Ident("FOO".to_string())),
            op: Op::Mul,
            rhs: Expr::Lit(CLiteral::Int(2)),
        }));

        let mut ctx = CContext::new(|name| match name {
            "FOO" => Some(42),
            _ => None,
        });

        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result.0, 84);
    }

    #[test]
    fn test_logical_short_circuit() {
        // Test that && short-circuits on false
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(0)),
            op: Op::And,
            // This would error if evaluated
            rhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(CLiteral::Int(1)),
                op: Op::Div,
                rhs: Expr::Lit(CLiteral::Int(0)),
            })),
        }));

        let mut ctx = CContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result.0, 0);
    }

    #[test]
    fn test_char_literal() {
        let expr = Expr::Unary(Box::new(Unary {
            op: Op::Add,
            expr: Expr::Lit(CLiteral::Char('A')),
        }));

        let mut ctx = CContext::new(|_| None);
        let result = eval(&expr, &mut ctx).unwrap();
        assert_eq!(result.0, 65); // ASCII value of 'A'
    }
}
