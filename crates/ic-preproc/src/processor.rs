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

#![allow(clippy::cast_possible_truncation)]

use std::borrow::{Borrow, BorrowMut};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ic_expr::{Binary, Op, Ternary, Unary};
use ic_lexer::cursor::Cursor;
use ic_lexer::token::{Base, Kind, Token};
use ic_vfs::{FileId, Include, Location, SourceMap};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::{debug, trace};

use crate::directives::{DirectiveHandler, IfState};
use crate::expression::{
    Expr, ExpressionContext, expr_op, infix_precedence, is_true, prefix_precedence,
};
use crate::macros::Macro;
use crate::state::{Directive, Error, State};
use crate::{ProcArgs, Span, time};

/// Context for function macro expansion
struct MacroExpansionContext<'a> {
    token: Token,
    args: &'a [Token],
    def: &'a [Token],
    variadic: bool,
    actual_args: &'a [Vec<Token>],
}

/// State we keep for each file we process. `File`s are not guaranteed to be
/// unique; multiple includes of the same file create multiple `File` instances
/// as each has to be parsed separately.
struct File {
    cursor: Cursor,
    current: Vec<IfState>,
}

impl File {
    pub fn from_src(source: Rc<str>, file_id: FileId) -> Self {
        let cursor = Cursor::new(source, file_id);
        let current = vec![];
        Self { cursor, current }
    }
}

struct Parser<'a, S> {
    stack: Vec<File>,
    state: S,
    vfs: &'a mut SourceMap,
    includes: FxHashSet<PathBuf>,
    recursion_depth: usize,

    /// Whether to process #pragma once
    enable_pragma_once: bool,

    /// Cache for include file resolution to avoid repeated directory traversals
    /// Maps `(relative_path, is_local)` -> `resolved_path`
    include_cache: FxHashMap<(PathBuf, bool), Option<PathBuf>>,
}

impl<S> ExpressionContext for Parser<'_, S>
where
    S: BorrowMut<State>,
{
    fn source_of(&self, span: Span) -> &str {
        // For the trait, we need to return a reference with the same lifetime as self
        // This is safe because the actual strings are stored in vfs which lives as long as Parser
        self.source_of_internal(span)
    }
}

impl<'a, S> Parser<'a, S>
where
    S: BorrowMut<State>,
{
    fn with_state(file: File, args: ProcArgs, state: S, vfs: &'a mut SourceMap) -> Self {
        let mut this = Self {
            state,
            stack: vec![file],
            enable_pragma_once: true,
            includes: args.include_dirs.into_iter().collect(),
            recursion_depth: args.recursion_depth,
            vfs,
            include_cache: FxHashMap::default(),
        };

        // Inject definitions from `ProcArgs`. This pushes a new virtual file
        // to the top of the stack.
        cli_defines(args.defines, &mut this);

        this
    }

    fn cursor(&mut self) -> &mut Cursor {
        let Some(pair) = self.stack.last_mut() else {
            unreachable!("cursor stack is empty");
        };
        &mut pair.cursor
    }

    fn if_state(&mut self) -> &mut Vec<IfState> {
        let Some(pair) = self.stack.last_mut() else {
            unreachable!("cursor stack is empty");
        };
        &mut pair.current
    }

    fn keyword(&mut self) {
        // Empty directives are allowed, but we should consume the hashtag.
        if let Some(tok) = self.cursor().advance() {
            match tok.kind {
                Kind::Ident | Kind::Keyword(_) => self.directive(tok.span),
                Kind::Number { .. } | Kind::Newline => (),
                _ => {
                    self.state().errors.push(Error::Syntax {
                        message: "invalid preprocessing directive",
                        span: tok.span,
                    });
                }
            }
        }
    }

    fn macro_name(&mut self) -> Option<(&'a str, Span)> {
        let tok = self.cursor().advance()?;
        match tok.kind {
            Kind::Ident | Kind::Keyword(_) => Some((self.source_of(tok.span), tok.span)),
            _ => {
                self.state().errors.push(Error::Syntax {
                    message: "macro name must be an identifier",
                    span: tok.span,
                });
                self.until_newline();
                None
            }
        }
    }

    fn state(&mut self) -> &mut State {
        self.state.borrow_mut()
    }

    fn source_of(&self, span: Span) -> &'a str {
        let Some(file) = self.stack.last() else {
            unreachable!("cursor stack is empty");
        };

        let src = if file.cursor.file_id() == span.start.file_id {
            file.cursor.source_of(span)
        } else {
            &self.vfs.source_str(span.start.file_id)[span.range()]
        };

        // SAFETY: The strings are guaranteed to be owned by `SourceMap`, whose
        // lifetimes are bound by 'a.
        unsafe { std::mem::transmute::<&str, &'a str>(src) }
    }

    fn source_of_internal(&self, span: Span) -> &str {
        let Some(file) = self.stack.last() else {
            unreachable!("cursor stack is empty");
        };

        if file.cursor.file_id() == span.start.file_id {
            file.cursor.source_of(span)
        } else {
            &self.vfs.source_str(span.start.file_id)[span.range()]
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        match name {
            "__LINE__" | "__FILE__" | "__DATE__" | "__TIME__" => true,
            _ => self.state.borrow().is_defined(name),
        }
    }

    fn mark_included(&mut self, file_id: FileId) {
        self.state().mark_parsed(file_id);
    }

    /// Collects tokens until newline and checks for unterminated strings
    fn until_newline(&mut self) -> Vec<Token> {
        let tokens = self.cursor().until_newline();

        // Check for unterminated strings and emit warnings
        for token in &tokens {
            if let Kind::String { terminated: false } = token.kind {
                self.state.borrow_mut().warnings.push(Error::Syntax {
                    message: "missing terminating '\"' character",
                    span: token.span,
                });
            }
        }

        tokens
    }

    /// Collects trailing tokens and produces a warning, e.g. for things like
    /// `#undef foo bar`, where "bar" is an extraneous token.
    fn warn_trailing(&mut self, directive: Directive) {
        let tokens = self.until_newline();
        if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
            let span = Span {
                start: first.span.start,
                end: last.span.end,
            };

            self.state().warnings.push(Error::Extraneous {
                directive,
                span,
                tokens,
            });
        }
    }

    /// If we define a region as the area between an `#if` and its
    /// corresponding `#endif`, then this determines whether the region that we
    /// are currently parsing is active, i.e. whether we should yield tokens
    /// for the current region or if they should be skipped.
    fn is_active(&mut self) -> bool {
        self.if_state().iter().all(IfState::is_active)
    }

    /// Searches through all include directories for a matching file.
    /// Results are cached to avoid repeated directory traversals.
    fn search_includes<P: AsRef<Path>>(&mut self, path: P, kind: Include) -> Option<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            trace!(path = %path.display(), "include path is absolute");
            return Some(path.to_owned());
        }

        // Check cache first
        let cache_key = (path.to_path_buf(), kind == Include::Local);
        if let Some(cached_result) = self.include_cache.get(&cache_key) {
            trace!(path = %path.display(), cached = true, "include cache hit");
            return cached_result.clone();
        }

        // Perform the search
        let mut result = None;

        // Include relative to the current file
        if kind == Include::Local {
            let cur_id = self.cursor().file_id();
            let local = self.vfs.path(cur_id);
            if let Some(parent) = local.parent() {
                let file = parent.join(path);
                trace!(path = %path.display(), trying = %file.display(), "trying local include");
                if file.exists() {
                    result = Some(file);
                }
            }
        }

        // Fall back to searching all include directories if not found
        if result.is_none() {
            for p in &self.includes {
                let file = p.join(path);
                trace!(path = %path.display(), trying = %file.display(), "trying include directory");
                if file.exists() {
                    result = Some(file);
                    break;
                }
            }
        }

        // Cache the result
        self.include_cache.insert(cache_key, result.clone());
        result
    }

    /// Check if a pragma directive is known/recognized
    fn is_known_pragma(name: &str) -> bool {
        matches!(
            name,
            "once" | "warning" | "message" | "region" | "endregion"
        )
    }

    fn expr_and_eval(&mut self, context_span: Span) -> bool {
        let result = self.expr(context_span);
        if let Some(tok) = self.state().queue.front().copied()
            && tok.kind != Kind::Newline
        {
            self.state().errors.push(Error::Syntax {
                message: "missing binary operator before this operand",
                span: tok.span,
            });
            self.until_newline();
            return false;
        }

        match result.and_then(|v| is_true(&v, self)) {
            Ok(v) => v,
            Err(e) => {
                self.state().errors.push(e);
                false
            }
        }
    }

    fn expect(&mut self, kind: Kind, message: &'static str) -> Option<Token> {
        if let Some(tok) = self.cursor().advance() {
            if tok.kind == kind {
                return Some(tok);
            }

            self.state().errors.push(Error::Syntax {
                message,
                span: tok.span,
            });
            self.until_newline();
        }
        None
    }

    fn directive(&mut self, span: Span) {
        let directive_str = self.source_of(span);
        let bytes = directive_str.as_bytes();

        // Quick first character check for better performance
        match bytes.first() {
            Some(b'i') => match directive_str {
                "if" => DirectiveHandler::dir_if(self, span),
                "ifdef" => DirectiveHandler::dir_ifdef(self, span),
                "ifndef" => DirectiveHandler::dir_ifndef(self, span),
                "include" => DirectiveHandler::dir_include(self, span),
                _ => self.invalid_directive(span),
            },
            Some(b'e') => match directive_str {
                "elif" => DirectiveHandler::dir_elif(self, span),
                "else" => DirectiveHandler::dir_else(self, span),
                "endif" => DirectiveHandler::dir_endif(self, span),
                "error" => DirectiveHandler::dir_error(self, span),
                _ => self.invalid_directive(span),
            },
            Some(b'd') => {
                if directive_str == "define" {
                    DirectiveHandler::dir_define(self);
                } else {
                    self.invalid_directive(span);
                }
            }
            Some(b'u') => {
                if directive_str == "undef" {
                    DirectiveHandler::dir_undef(self);
                } else {
                    self.invalid_directive(span);
                }
            }
            Some(b'p') => {
                if directive_str == "pragma" {
                    DirectiveHandler::dir_pragma(self, span);
                } else {
                    self.invalid_directive(span);
                }
            }
            Some(b'w') => {
                if directive_str == "warning" {
                    DirectiveHandler::dir_warning(self, span);
                } else {
                    self.invalid_directive(span);
                }
            }
            Some(b'l') => {
                if directive_str == "line" {
                    DirectiveHandler::dir_line(self);
                } else {
                    self.invalid_directive(span);
                }
            }
            _ => self.invalid_directive(span),
        }
    }

    fn invalid_directive(&mut self, span: Span) {
        self.state().errors.push(Error::Syntax {
            message: "invalid preprocessing directive",
            span,
        });
    }

    fn expr(&mut self, context_span: Span) -> Result<Expr, Error> {
        self.binary_expr(0, context_span)
    }

    fn unary_expr(&mut self, context_span: Span) -> Result<Expr, Error> {
        let lhs = self.next().ok_or(Error::Expr {
            message: "unexpected end of expression",
            span: context_span,
        })?;

        if matches!(lhs.kind, Kind::Ident | Kind::Keyword(_))
            && self.source_of(lhs.span) == "defined"
        {
            return self.parse_defined_operator(lhs.span);
        }

        let expr = match lhs.kind {
            Kind::Ident | Kind::Keyword(_) | Kind::Number { .. } | Kind::Char => {
                Expr::Lit(lhs, lhs.span)
            }
            Kind::Plus | Kind::Minus | Kind::Not | Kind::BitNot => {
                let prefix = prefix_precedence(lhs.kind);
                let expr = self.binary_expr(prefix, context_span)?;
                Expr::Unary(Box::new(Unary {
                    op: expr_op(lhs)?,
                    expr,
                }))
            }
            Kind::LParen => {
                let expr = self.binary_expr(0, context_span)?;
                // In expressions, we need to check for closing paren without consuming the line
                match self.next() {
                    Some(tok) if tok.kind == Kind::RParen => expr,
                    Some(tok) => {
                        // Push back the token we just consumed
                        self.state().queue.push_front(tok);
                        return Err(Error::Syntax {
                            message: "expected ')' in expression",
                            span: tok.span,
                        });
                    }
                    None => {
                        return Err(Error::Expr {
                            message: "unexpected end of expression, expected ')'",
                            span: lhs.span,
                        });
                    }
                }
            }
            _ => {
                return Err(Error::Syntax {
                    message: "expected value in expression",
                    span: lhs.span,
                });
            }
        };
        Ok(expr)
    }

    fn parse_defined_operator(&mut self, defined_span: Span) -> Result<Expr, Error> {
        // defined can be used as:
        // - defined(MACRO)
        // - defined MACRO
        let next = self.next_raw_token().ok_or(Error::Expr {
            message: "unexpected end after 'defined'",
            span: defined_span,
        })?;

        let macro_name = match next.kind {
            Kind::LParen => {
                // defined(MACRO) form
                let lparen_span = next.span;
                let name_tok = self.next_raw_token().ok_or(Error::Expr {
                    message: "expected macro name after 'defined('",
                    span: lparen_span,
                })?;

                // Verify it's an identifier
                if !matches!(name_tok.kind, Kind::Ident | Kind::Keyword(_)) {
                    return Err(Error::Syntax {
                        message: "expected identifier in defined()",
                        span: name_tok.span,
                    });
                }

                let name_span = name_tok.span;

                // Expect closing paren
                match self.next_raw_token() {
                    Some(tok) if tok.kind == Kind::RParen => {}
                    Some(tok) => {
                        self.state().queue.push_front(tok);
                        return Err(Error::Syntax {
                            message: "expected ')' after macro name in defined()",
                            span: tok.span,
                        });
                    }
                    None => {
                        return Err(Error::Expr {
                            message: "unexpected end in defined(), expected ')'",
                            span: name_span,
                        });
                    }
                }

                self.source_of(name_span)
            }
            Kind::Ident | Kind::Keyword(_) => {
                // defined MACRO form
                self.source_of(next.span)
            }
            _ => {
                self.state().queue.push_front(next);
                return Err(Error::Syntax {
                    message: "expected macro name or '(' after 'defined'",
                    span: next.span,
                });
            }
        };

        // Return a literal that evaluates to 1 or 0
        // We need to create a synthetic token that will be evaluated correctly
        let value = if self.is_defined(macro_name) {
            "1"
        } else {
            "0"
        };

        // Create a synthetic file with just the value
        let file_id = self.vfs.embed(value);
        let span = Span {
            start: Location { offset: 0, file_id },
            end: Location { offset: 1, file_id },
        };

        // Create a literal expression with the value
        Ok(Expr::Lit(
            Token {
                kind: Kind::Number {
                    base: Base::Decimal,
                },
                span,
            },
            span,
        ))
    }

    // Note that this function uses `Parser::next` instead of `Cursor::next` as
    // we need to expand and inline macros during parsing, except for 'defined'.
    fn binary_expr(&mut self, min_prec: u8, context_span: Span) -> Result<Expr, Error> {
        let mut lhs = self.unary_expr(context_span)?;

        while let Some(op_token) = self.next() {
            let (actual_op, prec) = if op_token.kind == Kind::Lt || op_token.kind == Kind::Gt {
                if let Some(next) = self.next() {
                    if op_token.kind == Kind::Lt && next.kind == Kind::Lt {
                        (Ok(Op::LShift), Some(9))
                    } else if op_token.kind == Kind::Gt && next.kind == Kind::Gt {
                        (Ok(Op::RShift), Some(9))
                    } else {
                        self.state().queue.push_front(next);
                        (expr_op(op_token), infix_precedence(op_token.kind))
                    }
                } else {
                    (expr_op(op_token), infix_precedence(op_token.kind))
                }
            } else {
                (expr_op(op_token), infix_precedence(op_token.kind))
            };

            // Check precedence
            let prec = match prec {
                Some(prec) if prec >= min_prec => prec,
                _ => {
                    self.state().queue.push_front(op_token);
                    break;
                }
            };

            lhs = if op_token.kind == Kind::Question {
                let then = self.expr(context_span)?;
                // Check for colon without consuming the line on error
                match self.next() {
                    Some(tok) if tok.kind == Kind::Colon => {
                        let els = self.binary_expr(prec + 1, context_span)?;
                        Expr::Ternary(Box::new(Ternary {
                            cond: lhs,
                            then,
                            els,
                        }))
                    }
                    Some(tok) => {
                        self.state().queue.push_front(tok);
                        return Err(Error::Syntax {
                            message: "expected ':' in ternary operator",
                            span: tok.span,
                        });
                    }
                    None => {
                        return Err(Error::Expr {
                            message: "unexpected end of expression, expected ':'",
                            span: op_token.span,
                        });
                    }
                }
            } else {
                let op = actual_op?;
                let rhs = self.binary_expr(prec + 1, context_span)?;
                Expr::Binary(Box::new(Binary { lhs, op, rhs }))
            }
        }
        Ok(lhs)
    }

    fn expand_predefined_macro(&mut self, name: &str, token: Token) -> bool {
        // Quick check: predefined macros start with '__'
        if !name.starts_with("__") {
            return false;
        }

        match name {
            "__LINE__" => {
                let line = self.cursor().line().to_string();
                let file_id = self.vfs.embed(&line);
                let span = Span {
                    start: Location { offset: 0, file_id },
                    end: Location {
                        offset: line.len() as u32,
                        file_id,
                    },
                };
                self.state().queue.push_back(Token {
                    kind: Kind::Number {
                        base: Base::Decimal,
                    },
                    span,
                });
                true
            }
            "__FILE__" => {
                // Get the file name from the vfs
                let filename = self.vfs.included_as(token.span.start.file_id);
                let filename_str = format!("\"{}\"", filename.display());
                let file_id = self.vfs.embed(&filename_str);
                let span = Span {
                    start: Location { offset: 0, file_id },
                    end: Location {
                        offset: filename_str.len() as u32,
                        file_id,
                    },
                };
                self.state().queue.push_back(Token {
                    kind: Kind::String { terminated: true },
                    span,
                });
                true
            }
            "__DATE__" => {
                let date_str = time::date();
                let file_id = self.vfs.embed(&date_str);
                let span = Span {
                    start: Location { offset: 0, file_id },
                    end: Location {
                        offset: date_str.len() as u32,
                        file_id,
                    },
                };
                self.state().queue.push_back(Token {
                    kind: Kind::String { terminated: true },
                    span,
                });
                true
            }
            "__TIME__" => {
                let time_str = time::utc_time();
                let file_id = self.vfs.embed(&time_str);
                let span = Span {
                    start: Location { offset: 0, file_id },
                    end: Location {
                        offset: time_str.len() as u32,
                        file_id,
                    },
                };
                self.state().queue.push_back(Token {
                    kind: Kind::String { terminated: true },
                    span,
                });
                true
            }
            _ => false,
        }
    }

    fn expand_fn_macro(
        &mut self,
        token: Token,
        args: &[Token],
        def: &[Token],
        variadic: bool,
        seen: &mut BTreeSet<&'a str>,
        macro_name: Rc<str>,
    ) {
        // Function-like macros only expand if followed by '('
        // Check if next token is '('
        let next_is_lparen = if let Some(file) = self.stack.last_mut() {
            matches!(file.cursor.peek(), Some(Kind::LParen))
        } else {
            false
        };

        if !next_is_lparen {
            self.state().queue.push_back(token);
            return;
        }

        // Consume the '('
        if let Some(file) = self.stack.last_mut() {
            file.cursor.advance();
        } else {
            self.state().errors.push(Error::Syntax {
                message: "internal error: empty file stack",
                span: token.span,
            });
            return;
        }

        // Parse the actual arguments
        let mut actual_args = self.parse_macro_args();

        // Special case: if we got exactly one empty argument but the macro expects zero args,
        // treat it as zero args. This handles EMPTY() correctly.
        if actual_args.len() == 1 && actual_args[0].is_empty() && args.is_empty() && !variadic {
            actual_args.clear();
        }

        // Check argument count
        if variadic {
            // For variadic macros, we need at least as many args as fixed params
            if actual_args.len() < args.len() {
                self.state().errors.push(Error::Syntax {
                    message: "too few arguments to variadic macro",
                    span: token.span,
                });
                return;
            }
        } else {
            // For non-variadic macros, exact match required
            if actual_args.len() != args.len() {
                self.state().errors.push(Error::Syntax {
                    message: "wrong number of arguments to macro",
                    span: token.span,
                });
                return;
            }
        }

        // Expand the function macro
        let ctx = MacroExpansionContext {
            token,
            args,
            def,
            variadic,
            actual_args: &actual_args,
        };
        self.expand_function_macro_impl(&ctx, seen, macro_name);
    }

    fn expand_function_macro_impl(
        &mut self,
        ctx: &MacroExpansionContext<'_>,
        seen: &mut BTreeSet<&'a str>,
        macro_name: Rc<str>,
    ) {
        // Build argument mapping for substitution
        let mut arg_map = FxHashMap::default();
        for (param, actual) in ctx.args.iter().zip(ctx.actual_args.iter()) {
            let param_name = self.source_of(param.span);
            arg_map.insert(param_name, actual.as_slice());
        }

        // Handle variadic arguments
        let va_args;
        if ctx.variadic {
            // Collect all remaining arguments for __VA_ARGS__
            let extra_args = &ctx.actual_args[ctx.args.len()..];
            let total_len: usize =
                extra_args.iter().map(Vec::len).sum::<usize>() + extra_args.len().saturating_sub(1);

            let mut va_tokens = Vec::with_capacity(total_len);
            for (i, arg) in extra_args.iter().enumerate() {
                if i > 0 {
                    va_tokens.push(Token {
                        kind: Kind::Comma,
                        span: ctx.token.span,
                    });
                }
                va_tokens.extend_from_slice(arg);
            }
            va_args = va_tokens;
            arg_map.insert("__VA_ARGS__", va_args.as_slice());
        }

        // Expand the macro definition with argument substitution
        let mut i = 0;
        let tokens = ctx.def;
        let mut result_tokens = Vec::new();

        while i < tokens.len() {
            let tok = &tokens[i];

            // Check for stringification operator (#)
            if tok.kind == Kind::Hash && i + 1 < tokens.len() {
                let next_tok = &tokens[i + 1];

                // Check for token pasting operator (##)
                if next_tok.kind == Kind::Hash {
                    result_tokens.push(*tok);
                    result_tokens.push(*next_tok);
                    i += 2;
                    continue;
                }

                // This is stringification
                if matches!(next_tok.kind, Kind::Ident | Kind::Keyword(_)) {
                    let param_name = self.source_of(next_tok.span);
                    if let Some(replacement) = arg_map.get(param_name) {
                        let stringified = self.stringify_tokens(replacement);
                        let string_token = self.create_string_token(&stringified);
                        result_tokens.push(string_token);
                        i += 2;
                        continue;
                    }
                }
            }

            // Handle parameter substitution
            if matches!(tok.kind, Kind::Ident | Kind::Keyword(_)) {
                let name = self.source_of(tok.span);

                // Check for __VA_OPT__
                if name == "__VA_OPT__"
                    && ctx.variadic
                    && Self::handle_va_opt_inline(&mut i, tokens, ctx, &mut result_tokens)
                {
                    continue;
                }

                if let Some(replacement) = arg_map.get(name) {
                    // Replace parameter with actual argument
                    result_tokens.extend_from_slice(replacement);
                } else {
                    // Not a parameter, keep as is
                    result_tokens.push(*tok);
                }
            } else {
                // Not an identifier, keep as is
                result_tokens.push(*tok);
            }

            i += 1;
        }

        // Perform token pasting
        let final_tokens = self.perform_token_pasting(&result_tokens);

        // Push all final tokens and record their expansion context
        let invocation_span = ctx.token.span;
        let expansion_info = crate::state::ExpansionInfo {
            invocation_span,
            macro_name,
        };

        for tok in final_tokens {
            // Record that this token came from a macro expansion
            self.state()
                .expansion_info
                .insert(tok.span, expansion_info.clone());
            self.expand_inner(tok, seen);
        }
    }

    fn expand_inner(&mut self, token: Token, seen: &mut BTreeSet<&'a str>) {
        // Only identifiers can be macros
        if !matches!(token.kind, Kind::Ident | Kind::Keyword(_)) {
            self.state().queue.push_back(token);
            return;
        }

        let name = self.source_of(token.span);

        // Handle predefined macros
        if self.expand_predefined_macro(name, token) {
            return;
        }

        let macro_entry = self.state.borrow().get_macro_with_name(name);
        if let Some((macro_name, v)) = macro_entry {
            // Macros should not be recursively expanded
            if !seen.insert(name) {
                self.state().queue.push_back(token);
                return;
            }

            // Bail if we've nested too deeply
            if seen.len() >= self.recursion_depth {
                self.state().errors.push(Error::Syntax {
                    message: "macro recursion depth limit was reached",
                    span: token.span,
                });
                return;
            }

            trace!(name = %name, depth = seen.len(), "expanding macro");

            match v.as_ref() {
                Macro::Function {
                    args,
                    def,
                    variadic,
                    ..
                } => {
                    self.expand_fn_macro(token, args, def, *variadic, seen, macro_name);
                }
                Macro::Object { def, .. } => {
                    // Record expansion context for object macros
                    let expansion_info = crate::state::ExpansionInfo {
                        invocation_span: token.span,
                        macro_name,
                    };

                    for &tok in def {
                        // Record that this token came from a macro expansion
                        self.state()
                            .expansion_info
                            .insert(tok.span, expansion_info.clone());
                        self.expand_inner(tok, seen);
                    }
                }
            }
            seen.remove(name);
        } else {
            self.state().queue.push_back(token);
        }
    }

    /// Returns `true` if the macro was expanded.
    ///
    /// A macro expansion is not allowed to have side effects, so we
    /// can fully expand the entire macro definition here. This is
    /// important as we need to detect and break potential cycles.
    fn handle_pragma_operator(&mut self, tok: Token) -> bool {
        if let Some(lparen) = self.cursor().peek()
            && lparen == Kind::LParen
        {
            // consume the opening parenthesis
            self.cursor().advance();

            // Get the string literal argument
            if let Some(string_tok) = self.cursor().advance()
                && matches!(string_tok.kind, Kind::String { terminated: true })
                && let Some(rparen) = self.cursor().advance()
                && rparen.kind == Kind::RParen
            {
                // Extract the pragma content from the string literal
                let string_content = self.source_of(string_tok.span);
                // Remove quotes
                if string_content.len() >= 2 {
                    let pragma_content = &string_content[1..string_content.len() - 1];

                    // Parse the pragma content as tokens
                    let pragma_id = self.vfs.embed(pragma_content);
                    let pragma_src = self.vfs.source(pragma_id);
                    let mut pragma_cursor = Cursor::new(pragma_src, pragma_id);
                    let mut pragma_tokens = Vec::new();
                    while let Some(tok) = pragma_cursor.advance() {
                        if tok.kind != Kind::Newline {
                            pragma_tokens.push(tok);
                        }
                    }

                    // Handle the pragma
                    if let Some(first_tok) = pragma_tokens.first() {
                        let pragma_name = self.source_of(first_tok.span);
                        if pragma_name == "once" && self.enable_pragma_once {
                            // Handle #pragma once via _Pragma
                            let file_id = tok.span.start.file_id;
                            self.mark_included(file_id);
                        }
                    }
                }
                return true;
            }

            // If we get here, the _Pragma syntax was invalid
            self.state().errors.push(Error::Syntax {
                message: "invalid _Pragma syntax",
                span: tok.span,
            });
            return true;
        }

        // No opening paren after _Pragma
        self.state().queue.push_back(tok);
        false
    }

    fn expand_macro(&mut self, tok: Token) -> bool {
        let name = self.source_of(tok.span);

        // Handle _Pragma operator
        if name == "_Pragma" {
            return self.handle_pragma_operator(tok);
        }

        // Handle predefined macros
        if self.expand_predefined_macro(name, tok) {
            return true;
        }

        if self.is_defined(name) {
            let mut seen = BTreeSet::new();
            self.expand_inner(tok, &mut seen);
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<Token> {
        'outer: loop {
            // Check if we're currently in the middle of a macro expansion, and
            // if so, yield those tokens first.
            if let Some(tok) = self.state().queue.pop_front() {
                return Some(tok);
            }

            // Advance the cursor and continue parsing directives
            if let Some(tok) = self.stack.last_mut()?.cursor.advance() {
                if tok.kind == Kind::Hash {
                    self.keyword();
                    continue 'outer;
                }

                // If the current token is a macro, we expand it and queue up
                // the expanded tokens.
                if self.expand_macro(tok) {
                    continue 'outer;
                }
                return Some(tok);
            }

            // Make sure all conditional directives were terminated
            let top = self.stack.last_mut()?;
            let file_id = top.cursor.file_id();
            while let Some(cond) = top.current.pop() {
                self.state.borrow_mut().errors.push(Error::Syntax {
                    message: "unterminated conditional directive",
                    span: cond.defined,
                });
            }

            // Cursor is empty, pop the stack
            let file_path = self.vfs.path(file_id);
            trace!(
                file = %file_path.display(),
                depth = self.stack.len(),
                "finished processing file"
            );
            self.stack.pop();
        }
    }

    fn next_active(&mut self) -> Option<Token> {
        loop {
            let next = self.next()?;
            if self.is_active() || next.kind == Kind::Newline {
                break Some(next);
            }
        }
    }

    /// Parse macro arguments from the token stream
    fn parse_macro_args(&mut self) -> Vec<Vec<Token>> {
        let mut args = vec![];
        let mut current_arg = vec![];
        let mut paren_depth = 0;
        let mut last_token_span = None;

        loop {
            let tok = if let Some(file) = self.stack.last_mut() {
                file.cursor.advance()
            } else {
                None
            };

            if let Some(tok) = tok {
                last_token_span = Some(tok.span);
                match tok.kind {
                    Kind::LParen => {
                        paren_depth += 1;
                        current_arg.push(tok);
                    }
                    Kind::RParen => {
                        if paren_depth == 0 {
                            // End of macro arguments
                            args.push(current_arg);
                            break;
                        }
                        paren_depth -= 1;
                        current_arg.push(tok);
                    }
                    Kind::Comma if paren_depth == 0 => {
                        // Argument separator
                        args.push(current_arg);
                        current_arg = vec![];
                    }
                    _ => {
                        current_arg.push(tok);
                    }
                }
            } else {
                self.state().errors.push(Error::Syntax {
                    message: "unexpected end of file in macro arguments",
                    span: last_token_span.unwrap_or_default(),
                });
                break;
            }
        }

        args
    }

    /// Determine the token kind for a pasted string by lexing it
    fn determine_token_kind(text: &str) -> Kind {
        // First, check for operators and special tokens
        match text {
            // Multi-character operators
            "==" => return Kind::EqEq,
            "!=" => return Kind::NotEq,
            "<=" => return Kind::LtEq,
            ">=" => return Kind::GtEq,
            "&&" => return Kind::And,
            "||" => return Kind::Or,
            "::" => return Kind::DColon,
            "++" | "--" => return Kind::Ident, // These are not single tokens in IDL

            // Single character operators
            "+" => return Kind::Plus,
            "-" => return Kind::Minus,
            "*" => return Kind::Star,
            "/" => return Kind::Slash,
            "%" => return Kind::Modulo,
            "=" => return Kind::Eq,
            "<" => return Kind::Lt,
            ">" => return Kind::Gt,
            "!" => return Kind::Not,
            "&" => return Kind::BitAnd,
            "|" => return Kind::BitOr,
            "^" => return Kind::BitXor,
            "~" => return Kind::BitNot,
            "?" => return Kind::Question,
            ":" => return Kind::Colon,
            ";" => return Kind::Semi,
            "," => return Kind::Comma,
            "." => return Kind::Period,
            "#" => return Kind::Hash,
            "@" => return Kind::At,
            "{" => return Kind::LBrace,
            "}" => return Kind::RBrace,
            "(" => return Kind::LParen,
            ")" => return Kind::RParen,
            "[" => return Kind::LBracket,
            "]" => return Kind::RBracket,
            "\\" => return Kind::Backslash,
            _ => {}
        }

        // Check if it's a keyword
        if let Some(kw) = ic_lexer::token::Kw::from_str(text) {
            return Kind::Keyword(kw);
        }

        // Check if it's a number
        if let Some(first_char) = text.chars().next()
            && first_char.is_ascii_digit()
        {
            // Simple number detection - could be decimal, octal, or hex
            if text.starts_with("0x") || text.starts_with("0X") {
                return Kind::Number {
                    base: Base::Hexadecimal,
                };
            } else if text.starts_with('0')
                && text.len() > 1
                && text.chars().all(|c| c.is_ascii_digit())
            {
                return Kind::Number { base: Base::Octal };
            } else if text.chars().all(|c| c.is_ascii_digit()) {
                return Kind::Number {
                    base: Base::Decimal,
                };
            }
        }

        // Default to identifier
        Kind::Ident
    }

    fn next_raw_token(&mut self) -> Option<Token> {
        'outer: loop {
            // Check if we're currently in the middle of a macro expansion, and
            // if so, yield those tokens first.
            if let Some(tok) = self.state().queue.pop_front() {
                return Some(tok);
            }

            // Advance the cursor and continue parsing directives
            if let Some(tok) = self.stack.last_mut()?.cursor.advance() {
                if tok.kind == Kind::Hash {
                    self.keyword();
                    continue 'outer;
                }
                return Some(tok);
            }

            // Make sure all conditional directives were terminated
            let top = self.stack.last_mut()?;
            while let Some(cond) = top.current.pop() {
                self.state.borrow_mut().errors.push(Error::Syntax {
                    message: "unterminated conditional directive",
                    span: cond.defined,
                });
            }

            // Cursor is empty, pop the stack
            self.stack.pop();
        }
    }

    /// Check if the next token is a left paren immediately after the macro name
    /// Returns (`is_function_macro`, `lparen_token_if_not_function`)
    fn check_function_macro_type(&mut self, name_span: Span) -> (bool, Option<Token>) {
        if let Some(tok) = self.cursor().take_if(Kind::LParen) {
            // For function-like macros, the '(' must immediately follow the macro name
            let is_adjacent = name_span.end.offset == tok.span.start.offset
                && name_span.end.file_id == tok.span.start.file_id;
            if is_adjacent {
                (true, None)
            } else {
                // Not a function-like macro, return the token
                (false, Some(tok))
            }
        } else {
            (false, None)
        }
    }

    /// Parse function macro parameters
    fn parse_function_macro_params(&mut self, name_span: Span) -> (Vec<Token>, bool) {
        let mut args = vec![];
        let mut variadic = false;

        while let Some(c) = self.cursor().peek() {
            if c == Kind::RParen {
                break;
            }

            // Check for variadic ...
            if c == Kind::Period {
                if self.check_and_consume_ellipsis() {
                    variadic = true;
                    self.expect(Kind::RParen, "variadic parameter must be last");
                    break;
                }

                self.state().errors.push(Error::Syntax {
                    message: "expected identifier or '...'",
                    span: name_span,
                });
                return (args, false);
            }

            // Parse parameter
            let Some(arg) = self.cursor().advance() else {
                return (args, false);
            };
            match arg.kind {
                Kind::Ident | Kind::Keyword(_) => args.push(arg),
                _ => {
                    self.state().errors.push(Error::Syntax {
                        message: "invalid token in macro parameter list",
                        span: arg.span,
                    });
                    self.until_newline();
                    return (args, false);
                }
            }

            if self.cursor().take_if(Kind::Comma).is_none() {
                self.expect(Kind::RParen, "expected comma or end of parameter list");
                break;
            }
        }

        (args, variadic)
    }

    /// Check if we have three consecutive periods (...) and consume them
    fn check_and_consume_ellipsis(&mut self) -> bool {
        let cursor = self.cursor();
        cursor.advance();
        if cursor.peek() == Some(Kind::Period) {
            cursor.advance();
            if cursor.peek() == Some(Kind::Period) {
                cursor.advance();
                return true;
            }
        }
        false
    }

    /// Create an object macro definition, including any lparen token that was consumed
    fn create_object_macro(&mut self, name_span: Span, lparen_tok: Option<Token>) -> Macro {
        let mut def = Vec::new();
        if let Some(tok) = lparen_tok {
            def.push(tok);
        }
        def.extend(self.until_newline());
        Macro::Object {
            span: name_span,
            def,
        }
    }

    /// Store the macro definition and emit warnings if it's a redefinition
    fn store_macro_definition(&mut self, name: &str, name_span: Span, definition: Macro) {
        if self.is_active() {
            let is_function = matches!(definition, Macro::Function { .. });
            trace!(
                name = %name,
                is_function = is_function,
                "defining macro"
            );

            if self
                .state()
                .defines
                .insert(Rc::from(name), Rc::new(definition))
                .is_some()
            {
                self.state().warnings.push(Error::Syntax {
                    message: "macro redefined",
                    span: name_span,
                });
            }
        }
    }

    /// Stringify a single token slice
    fn stringify_tokens(&self, tokens: &[Token]) -> String {
        let estimated_capacity = tokens.len() * 10 + 2;
        let mut stringified = String::with_capacity(estimated_capacity);

        stringified.push('"');
        for (j, arg_tok) in tokens.iter().enumerate() {
            if j > 0 {
                stringified.push(' ');
            }
            stringified.push_str(self.source_of(arg_tok.span));
        }
        stringified.push('"');
        stringified
    }

    /// Create a string token from content
    fn create_string_token(&mut self, content: &str) -> Token {
        let file_id = self.vfs.embed(content);
        let span = Span {
            start: Location { offset: 0, file_id },
            end: Location {
                offset: content.len() as u32,
                file_id,
            },
        };
        Token {
            kind: Kind::String { terminated: true },
            span,
        }
    }

    /// Handle `__VA_OPT__` macro inline
    fn handle_va_opt_inline(
        i: &mut usize,
        tokens: &[Token],
        ctx: &MacroExpansionContext<'_>,
        result_tokens: &mut Vec<Token>,
    ) -> bool {
        if *i + 1 < tokens.len() && tokens[*i + 1].kind == Kind::LParen {
            *i += 2; // Skip __VA_OPT__ and (

            let opt_tokens = Self::extract_va_opt_tokens(tokens, i);

            // Only include the content if there are variadic arguments
            let has_varargs = ctx.actual_args.len() > ctx.args.len();
            if has_varargs {
                result_tokens.extend(opt_tokens);
            }
            return true;
        }
        false
    }

    /// Extract tokens within `__VA_OPT__` parentheses
    fn extract_va_opt_tokens(tokens: &[Token], i: &mut usize) -> Vec<Token> {
        let mut paren_depth = 1;
        let mut opt_tokens = Vec::new();

        while *i < tokens.len() && paren_depth > 0 {
            match tokens[*i].kind {
                Kind::LParen => paren_depth += 1,
                Kind::RParen => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            if paren_depth > 0 {
                opt_tokens.push(tokens[*i]);
            }
            *i += 1;
        }

        opt_tokens
    }

    /// Perform token pasting (##) operations
    fn perform_token_pasting(&mut self, result_tokens: &[Token]) -> Vec<Token> {
        let mut final_tokens = Vec::new();
        let mut i = 0;

        while i < result_tokens.len() {
            if Self::is_token_paste_sequence(result_tokens, i) {
                let left = &result_tokens[i];
                let right = &result_tokens[i + 3];

                let pasted_token = self.paste_tokens(left, right);
                final_tokens.push(pasted_token);
                i += 4;
            } else {
                final_tokens.push(result_tokens[i]);
                i += 1;
            }
        }

        final_tokens
    }

    /// Check if we have a token paste sequence at position i
    fn is_token_paste_sequence(tokens: &[Token], i: usize) -> bool {
        i + 3 < tokens.len() && tokens[i + 1].kind == Kind::Hash && tokens[i + 2].kind == Kind::Hash
    }

    /// Paste two tokens together
    fn paste_tokens(&mut self, left: &Token, right: &Token) -> Token {
        let left_text = self.source_of(left.span);
        let right_text = self.source_of(right.span);
        let mut pasted = String::with_capacity(left_text.len() + right_text.len());
        let _ = write!(&mut pasted, "{left_text}{right_text}");

        let file_id = self.vfs.embed(&pasted);
        let span = Span {
            start: Location { offset: 0, file_id },
            end: Location {
                offset: pasted.len() as u32,
                file_id,
            },
        };

        Token {
            kind: Self::determine_token_kind(&pasted),
            span,
        }
    }
}

impl<S> DirectiveHandler for Parser<'_, S>
where
    S: BorrowMut<State>,
{
    fn dir_include(&mut self, span: Span) {
        let cursor = self.cursor();
        let (kind, path) = match cursor.peek() {
            Some(Kind::Lt) => {
                _ = cursor.advance();
                let span = cursor.until_peek(Kind::Gt);
                self.expect(Kind::Gt, "unterminated include");
                (Include::System, span)
            }
            Some(Kind::String { .. }) => {
                let Some(tok) = cursor.advance() else {
                    self.state().errors.push(Error::Syntax {
                        message: "unexpected end of file in include directive",
                        span,
                    });
                    return;
                };
                (Include::Local, tok.span)
            }
            _ => {
                self.expect(
                    Kind::String { terminated: true },
                    "expected \"file\" or <file>",
                );
                return;
            }
        };
        self.warn_trailing(Directive::Include);

        if self.is_active() {
            // Bail if we've hit the recursion depth
            if self.stack.len() >= self.recursion_depth {
                self.state().errors.push(Error::Syntax {
                    message: "#include nested too deeply",
                    span: path,
                });
                return;
            }

            let include_str = self.source_of(path);
            let include = include_str.trim_start_matches('"').trim_end_matches('"');

            trace!(path = %include, kind = ?kind, "searching for include");

            if let Some(v) = self.search_includes(include, kind) {
                debug!(path = %include, resolved = %v.display(), "resolved include");
                match self.vfs.open(v, kind) {
                    Ok((id, source)) => {
                        // Record the include for lint purposes
                        let including_file = self.cursor().file_id();
                        self.vfs.record_include(ic_vfs::IncludeInfo {
                            path_span: path,
                            included_as: include.to_owned(),
                            included_file: id,
                            including_file,
                            kind,
                        });

                        // Skip files that we've already parsed if they used
                        // the `once` pragma.
                        if self.state().parsed_files.contains(&id) {
                            trace!(path = %include, "skipping already-parsed file (pragma once)");
                        } else {
                            let cursor = File::from_src(source, id);
                            self.stack.push(cursor);
                        }
                    }
                    Err(_) => self.state().errors.push(Error::Syntax {
                        message: "failed to open file",
                        span: path,
                    }),
                }
            } else {
                debug!(path = %include, "include file not found");
                self.state().errors.push(Error::Syntax {
                    message: "file not found",
                    span: path,
                });
            }
        }
    }

    fn dir_define(&mut self) {
        let Some((name, name_span)) = self.macro_name() else {
            return;
        };

        let (is_function_macro, lparen_tok) = self.check_function_macro_type(name_span);
        let definition = if is_function_macro {
            let (args, variadic) = self.parse_function_macro_params(name_span);
            let def = self.until_newline();
            Macro::Function {
                span: name_span,
                args,
                def,
                variadic,
            }
        } else {
            self.create_object_macro(name_span, lparen_tok)
        };

        self.store_macro_definition(name, name_span, definition);
    }

    fn dir_undef(&mut self) {
        let Some((name, _span)) = self.macro_name() else {
            return;
        };
        self.warn_trailing(Directive::Undef);

        if self.is_active() {
            trace!(name = %name, "undefining macro");
            self.state().defines.remove(name);
        }
    }

    fn dir_if(&mut self, span: Span) {
        let result = self.expr_and_eval(span);
        trace!(result = result, "evaluating #if");
        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifdef(&mut self, span: Span) {
        let result = if let Some((name, _span)) = self.macro_name() {
            self.warn_trailing(Directive::Ifdef);
            let defined = self.is_defined(name);
            trace!(name = %name, defined = defined, "evaluating #ifdef");
            defined
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifndef(&mut self, span: Span) {
        let result = if let Some((name, _span)) = self.macro_name() {
            self.warn_trailing(Directive::Ifndef);
            let defined = self.is_defined(name);
            trace!(name = %name, defined = defined, "evaluating #ifndef");
            !defined
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_elif(&mut self, span: Span) {
        let result = self.expr_and_eval(span);
        trace!(result = result, "evaluating #elif");

        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_elif(result, span) {
                    self.state().errors.push(e);
                }
            }
            None => self.state().errors.push(Error::Syntax {
                message: "#elif without #if",
                span,
            }),
        }
    }

    fn dir_else(&mut self, span: Span) {
        trace!("processing #else");
        self.warn_trailing(Directive::Else);
        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_else(span) {
                    self.state().errors.push(e);
                }
            }
            None => self.state().errors.push(Error::Syntax {
                message: "#else without #if",
                span,
            }),
        }
    }

    fn dir_endif(&mut self, span: Span) {
        trace!("processing #endif");
        self.warn_trailing(Directive::Endif);
        if self.if_state().pop().is_none() {
            self.state().errors.push(Error::Syntax {
                message: "#endif without #if",
                span,
            });
        }
    }

    fn dir_pragma(&mut self, _span: Span) {
        let tokens = self.until_newline();

        // Find the first non-newline token, which should be the pragma name
        // This handles line continuation where we might get newline tokens first
        let pragma_token = tokens.iter().find(|t| t.kind != Kind::Newline);

        if let Some(pragma) = pragma_token {
            let name = self.source_of(pragma.span);
            trace!(name = %name, "processing #pragma");

            if name == "once" && self.enable_pragma_once {
                // Handle #pragma once
                let file_id = pragma.span.start.file_id;
                debug!("marking file as pragma once");
                self.mark_included(file_id);
            } else if self.is_active() && !Self::is_known_pragma(name) {
                // Warn about unknown pragmas only in active code
                debug!(name = %name, "unknown pragma");
                self.state().warnings.push(Error::Syntax {
                    message: "unknown pragma directive",
                    span: pragma.span,
                });
            }
        }
        // Empty pragmas are allowed, so no warning if no token found
    }

    fn dir_error(&mut self, span: Span) {
        let tokens = self.until_newline();
        if self.is_active() {
            self.state().errors.push(Error::Note { span, tokens });
        }
    }

    fn dir_warning(&mut self, span: Span) {
        let tokens = self.until_newline();
        if self.is_active() {
            self.state().warnings.push(Error::Note { span, tokens });
        }
    }

    fn dir_line(&mut self) {
        // Only decimal numbers allowed here
        let _line = self.expect(
            Kind::Number {
                base: Base::Decimal,
            },
            "expected decimal line number",
        );

        // Optional filename
        if let Some(tok) = self.cursor().peek()
            && matches!(tok, Kind::String { terminated: true })
        {
            self.cursor().advance();
        }

        self.warn_trailing(Directive::Line);
    }
}

fn cli_defines<S, I>(defines: I, parser: &mut Parser<'_, S>)
where
    S: BorrowMut<State>,
    I: IntoIterator<Item = (String, Option<String>)>,
{
    // Generate a virtual file with the specified command-line arguments. We
    // need to be able to reference these (and their location) in the future,
    // so creating a new virtual file is easier.
    let mut buffer = String::new();
    for (k, v) in defines {
        _ = write!(&mut buffer, "#define {k}");
        if let Some(v) = v {
            _ = write!(&mut buffer, " {v}");
        }
        _ = writeln!(&mut buffer);
    }

    // Insert the generated file into the VFS
    let src: Rc<str> = Rc::from(buffer);
    let cli = parser.vfs.embed_with_name("<command-line>", src.clone());
    let file = File::from_src(src, cli);

    // Push the file to the parser's stack. The definitions will then be parsed
    // when the iterator is evalutaed.
    parser.stack.push(file);
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct TokenIter<'a, S> {
    inner: Parser<'a, S>,
    prev: Option<Span>,
}

impl<S> TokenIter<'_, S>
where
    S: BorrowMut<State>,
{
    #[must_use]
    pub fn source_of(&self, span: Span) -> &str {
        self.inner.source_of(span)
    }

    /// Get access to the preprocessor state, including errors and warnings
    pub fn state(&self) -> &State
    where
        S: Borrow<State>,
    {
        self.inner.state.borrow()
    }

    #[must_use]
    pub fn prev_span(&self) -> Option<Span> {
        self.prev
    }

    /// Get the current `file_id` being processed
    pub fn current_file_id(&mut self) -> Option<FileId> {
        self.inner.stack.last().map(|file| file.cursor.file_id())
    }
}

impl<S> Iterator for TokenIter<'_, S>
where
    S: BorrowMut<State>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.inner.next_active() {
            // Check for unterminated strings and emit warning (like GCC/Clang)
            if let Kind::String { terminated: false } = next.kind {
                self.inner.state.borrow_mut().warnings.push(Error::Syntax {
                    message: "missing terminating '\"' character",
                    span: next.span,
                });
            }
            // Track previous non-newline token for error reporting
            if next.kind != Kind::Newline {
                self.prev = Some(next.span);
            }
            Some(next)
        } else {
            self.prev.take().map(|span| Token {
                kind: Kind::Eoi,
                span,
            })
        }
    }
}

pub fn preprocess<S: BorrowMut<State>>(
    file_id: FileId,
    args: ProcArgs,
    state: S,
    vfs: &mut SourceMap,
) -> TokenIter<'_, S> {
    let source = vfs.source(file_id);
    let file = File::from_src(source, file_id);
    let parser = Parser::with_state(file, args, state, vfs);

    // For empty files, we need a valid span for the EOI token
    // Use the beginning of the file as the span
    let initial_span = Span {
        start: Location::new(0, file_id),
        end: Location::new(0, file_id),
    };

    TokenIter {
        inner: parser,
        prev: Some(initial_span),
    }
}

/// Preprocesses a file, inlines all includes and expands all macro definitions.
/// Attempts to preserve formatting by using original byte offsets between tokens.
pub fn to_string(
    file_id: FileId,
    args: ProcArgs,
    state: &mut State,
    vfs: &mut SourceMap,
) -> String {
    let mut buffer = String::new();
    let mut last_id = file_id;
    let mut last_end = None;
    let mut last_invocation = None;
    let tokens: Vec<_> = preprocess(file_id, args, &mut *state, vfs).collect();

    for tok in tokens {
        if tok.kind == Kind::Eoi {
            break;
        }

        let expansion = state.expansion_info.get(&tok.span);
        let pos_span = expansion.map_or(tok.span, |info| info.invocation_span);
        let current_id = pos_span.start.file_id;

        let same_invocation = expansion.is_some()
            && last_invocation
                .is_some_and(|last| expansion.is_some_and(|info| info.invocation_span == last));

        if last_id != current_id {
            let path = vfs.path(current_id);
            _ = buffer.write_str(&format!("\n#line 1 \"{}\"\n", path.display()));
            last_id = current_id;
            write_token(&mut buffer, tok, vfs);
            last_end = Some(pos_span.end.offset as usize);
        } else if same_invocation {
            _ = buffer.write_char(' ');
            write_token(&mut buffer, tok, vfs);
            last_end = Some(pos_span.end.offset as usize);
        } else {
            if let Some(prev_end) = last_end {
                let source_full = vfs.source(current_id);
                let curr_start = pos_span.start.offset as usize;
                if curr_start > prev_end {
                    let between = &source_full[prev_end..curr_start];
                    if between.chars().all(char::is_whitespace) {
                        _ = buffer.write_str(between);
                    } else if !between.contains('\n') {
                        _ = buffer.write_char(' ');
                    }
                } else if expansion.is_some() {
                    _ = buffer.write_char(' ');
                }
            }
            if tok.kind == Kind::Newline && !buffer.ends_with(|c: char| c.is_whitespace()) {
                _ = buffer.write_char(' ');
            }
            write_token(&mut buffer, tok, vfs);
            last_end = Some(pos_span.end.offset as usize);
        }

        last_invocation = expansion.map(|info| info.invocation_span);
    }
    buffer.trim().to_string()
}

fn write_token(buffer: &mut String, tok: Token, vfs: &SourceMap) {
    let text = tok.kind.as_str().unwrap_or_else(|| {
        let src = vfs.source_str(tok.span.start.file_id);
        &src[tok.span.start.offset as usize..tok.span.end.offset as usize]
    });
    _ = buffer.write_str(text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pp(vfs: &mut SourceMap, input: &str) -> State {
        let id = vfs.embed(input);
        let mut state = State::new();
        preprocess(id, ProcArgs::default(), &mut state, vfs).for_each(drop);
        state
    }

    fn with_state(state: &mut State, vfs: &mut SourceMap, input: &str) -> Vec<Token> {
        let id = vfs.embed(input);
        preprocess(id, ProcArgs::default(), state, vfs).collect()
    }

    fn expand(input: &str) -> String {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(input);
        let mut state = State::new();
        let output = to_string(id, ProcArgs::default(), &mut state, &mut vfs);
        output.trim().to_string()
    }

    #[test]
    fn define_empty() {
        let mut vfs = SourceMap::default();
        let state = pp(&mut vfs, "#define foo");
        assert!(state.is_defined("foo"));
    }

    #[test]
    fn define() {
        let mut vfs = SourceMap::default();
        let state = pp(&mut vfs, "#define foo bar 123");

        let Some(m) = state.get_macro("foo") else {
            panic!();
        };
        let Macro::Object { def, .. } = m.as_ref() else {
            panic!();
        };
        assert_eq!(def.len(), 2);

        let mut iter = def.iter();
        assert_eq!(iter.next().unwrap().kind, Kind::Ident);
        assert_eq!(
            iter.next().unwrap().kind,
            Kind::Number {
                base: Base::Decimal
            }
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn undef() {
        let mut vfs = SourceMap::default();
        let mut state = pp(&mut vfs, "#define foo");
        assert!(state.is_defined("foo"));

        with_state(&mut state, &mut vfs, "#undef foo");
        assert!(!state.is_defined("foo"));
        assert!(state.errors().is_empty());
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn undef_non_existent() {
        let mut vfs = SourceMap::default();
        let state = pp(&mut vfs, "#undef foo");
        assert!(state.errors().is_empty());
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn multiline_define() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #define foo bar \
                    baz
                123
            ",
        );

        let Some(m) = state.get_macro("foo") else {
            panic!();
        };
        let Macro::Object { def, .. } = m.as_ref() else {
            panic!();
        };

        let mut iter = def.iter();
        assert_eq!(iter.next().unwrap().kind, Kind::Ident);
        assert_eq!(iter.next().unwrap().kind, Kind::Newline);
        assert_eq!(iter.next().unwrap().kind, Kind::Ident);
        assert!(iter.next().is_none());
    }

    #[test]
    fn define_immediate_newl() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #define foo \
                    bar \
                    baz
            ",
        );

        let Some(m) = state.get_macro("foo") else {
            panic!();
        };
        let Macro::Object { def, .. } = m.as_ref() else {
            panic!();
        };

        let mut iter = def.iter();
        assert_eq!(iter.next().unwrap().kind, Kind::Newline);
        assert_eq!(iter.next().unwrap().kind, Kind::Ident);
        assert_eq!(iter.next().unwrap().kind, Kind::Newline);
        assert_eq!(iter.next().unwrap().kind, Kind::Ident);
        assert!(iter.next().is_none());
    }

    #[test]
    fn expand_single() {
        let expanded = expand(
            r"
                #define foo bar
                bar 123 bar
            ",
        );
        assert_eq!(expanded, "bar 123 bar");
    }

    #[test]
    fn recursive_macro() {
        // A macro should not be expanded from the definition of itself, so
        // "foo" in the macro definition is just treated as a normal identifier
        // and not a macro to be expanded.
        let expanded = expand(
            r"
                #define foo foo foo bar
                foo
            ",
        );
        assert_eq!(expanded, "foo foo bar");
    }

    #[test]
    fn recursively_expand() {
        let expanded = expand(
            r"
                #define baz 123
                #define bar baz
                #define foo bar
                foo
            ",
        );
        assert_eq!(expanded, "123");
    }

    #[test]
    fn recursive_cyclic() {
        // Recursion stops as soon as we find a macro we've already expanded.
        let expanded = expand(
            r"
                #define foo foo foo
                #define bar baz
                #define baz foo

                foo bar baz
            ",
        );
        assert_eq!(expanded, "foo foo foo foo foo foo");

        let expanded = expand(
            r"
                #define foo bar
                #define bar foo
                foo bar
            ",
        );
        assert_eq!(expanded, "foo bar");
    }

    #[test]
    fn backslash() {
        let expanded = expand(
            r"
                #define foo \ a
                foo
            ",
        );
        assert_eq!(expanded, "\\ a");
    }

    #[test]
    fn spaceship() {
        let mut vfs = SourceMap::default();
        let state = pp(&mut vfs, "#define foo <==>");

        let Some(m) = state.get_macro("foo") else {
            panic!();
        };
        let Macro::Object { def, .. } = m.as_ref() else {
            panic!();
        };

        let mut iter = def.iter();
        assert_eq!(iter.next().unwrap().kind, Kind::LtEq);
        assert_eq!(iter.next().unwrap().kind, Kind::Eq);
        assert_eq!(iter.next().unwrap().kind, Kind::Gt);
        assert!(iter.next().is_none());
    }

    #[test]
    fn comment_escaped_newl() {
        // We don't escape newlines in comments
        let expanded = expand(
            r"
                // some comment \
                foo
            ",
        );
        assert_eq!(expanded, "foo");
    }

    #[test]
    fn inactive_warnings() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #if 0
                #warning foo
                #endif
            ",
        );
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn inactive_errors() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #if 0
                #error foo
                #endif
            ",
        );
        assert!(state.errors().is_empty());
    }

    #[test]
    fn expand_inactive() {
        let expanded = expand(
            r"
                #define foo bar
                #if 0
                foo
                #endif
            ",
        );
        assert!(expanded.is_empty());
    }

    #[test]
    fn define_inactive() {
        let expanded = expand(
            r"
                #if 0
                #define foo bar
                #endif
                foo
            ",
        );
        assert_eq!(expanded, "foo");
    }

    #[test]
    fn redefine_object() {
        let mut vfs = SourceMap::default();
        let mut state = pp(
            &mut vfs,
            r"
                #define foo 123
                #define foo 456
                #define foo bar
            ",
        );
        // Not an error, and the value should be updated, but we should emit
        // a warning each time it is redefined.
        assert!(state.errors().is_empty());
        assert_eq!(state.warnings().len(), 2);

        // Last definition is the one that counts
        let expanded = with_state(&mut state, &mut vfs, "foo");
        assert_eq!(expanded.first().unwrap().kind, Kind::Ident);
    }

    #[test]
    fn expr_expansion() {
        let mut vfs = SourceMap::default();

        // We avoid using `elif` and `else` here to make sure the expression
        // doesn't blindly evaluate to true regardless of its contents.
        let state = pp(
            &mut vfs,
            r#"
                #define one 1
                #define plus +
                #define sum one plus one

                #if sum == 2
                #warning "ok"
                #endif

                #if sum != 2
                #error "fail"
                #endif
            "#,
        );
        assert!(state.errors().is_empty());
        assert_eq!(state.warnings().len(), 1);
    }

    #[test]
    fn expanded_precedence() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #define two 2
                #define three 3
                #define mul *
                #define sum two + three mul three

                #if sum == 11
                #warning "ok"
                #endif

                #if sum != 11
                #error "fail"
                #endif
            "#,
        );
        assert!(state.errors().is_empty());
        assert_eq!(state.warnings().len(), 1);
    }

    #[test]
    fn extra_tokens_ifdef() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #ifdef tru foo
                #endif
            ",
        );
        assert_eq!(state.warnings().len(), 1);
    }

    #[test]
    fn extra_tokens_ifndef() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #ifndef tru foo
                #endif
            ",
        );
        assert_eq!(state.warnings().len(), 1);
    }

    #[test]
    fn if_cond() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #define foo bar

                #if foo
                #error "fail"
                #endif
            "#,
        );
        assert!(state.errors().is_empty());
    }

    #[test]
    fn if_cond_cyclic() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #define foo bar
                #define bar foo

                #if foo
                #error "fail"
                #endif

                #if foo + 1 != 1
                #error "fail"
                #endif
            "#,
        );
        assert!(state.errors().is_empty());
    }

    #[test]
    fn pragma_misc() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #pragma once
                #pragma foo bar baz
                #pragma 🤠
                #pragma!
                #pragma^<[==]>

                #pragma \
                    once

                #pragma warning(push)
                #pragma warning(disable : 4251)
                #pragma warning(pop)

                #pragma
            ",
        );
        assert!(state.errors().is_empty());
        // Now we expect warnings for unknown pragmas
        assert_eq!(state.warnings().len(), 4); // foo, 🤠, !, ^<[==]>
    }

    #[test]
    fn invalid_expr() {
        let output = expand(
            r"
                #if a +
                    abc
                #elif b +
                    def
                #else
                    123
                #endif
            ",
        );
        // Invalid expressions will always evaluate to 0.
        assert_eq!(output, "123");
    }

    #[test]
    fn missing_operator() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r"
                #if 1 2 3
                #endif
            ",
        );
        assert_eq!(state.errors().len(), 1);
    }

    #[test]
    fn undef_break_cycle() {
        let output = expand(
            r"
                #define foo bar
                #define bar baz
                foo // should be baz

                #undef bar
                foo // should be bar
            ",
        );
        assert_eq!(output, "baz \n\nbar");
    }
}
