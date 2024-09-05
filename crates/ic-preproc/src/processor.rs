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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::path::PathBuf;
use std::rc::Rc;

use ic_expr::{Binary, Op, Ternary, Unary};
use ic_vfs::{FileId, Include, SourceMap};

use crate::cursor::{Base, Cursor, Directive, Kind, SourceSpan, Token};
use crate::{time, ProcArgs};

#[derive(Debug)]
pub enum Macro {
    Function {
        span: SourceSpan,
        args: Vec<Token>,
        def: Vec<Token>,
    },
    Object {
        span: SourceSpan,
        def: Vec<Token>,
    },
}

type Expr = ic_expr::Expr<Token>;

impl TryFrom<Token> for Op {
    type Error = Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        let op = match value.kind {
            Kind::Not => Op::Not,
            Kind::Or => Op::Or,
            Kind::And => Op::And,
            Kind::EqEq => Op::EqEq,
            Kind::NotEq => Op::NotEq,
            Kind::BitOr => Op::BitOr,
            Kind::BitAnd => Op::BitAnd,
            Kind::BitXor => Op::BitXor,
            Kind::BitNot => Op::BitNot,
            Kind::Lt => Op::Lt,
            Kind::LtEq => Op::LtEq,
            Kind::Gt => Op::Gt,
            Kind::GtEq => Op::GtEq,
            Kind::Plus => Op::Add,
            Kind::Minus => Op::Sub,
            Kind::Star => Op::Mul,
            Kind::Slash => Op::Div,
            Kind::Modulo => Op::Mod,
            _ => Err(Error::Syntax {
                message: "invalid binary operator",
                span: value.span,
            })?,
        };
        Ok(op)
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    Note {
        span: SourceSpan,
        tokens: Vec<Token>,
    },
    Extraneous {
        directive: Directive,
        span: SourceSpan,
        tokens: Vec<Token>,
    },
    Syntax {
        message: &'static str,
        span: SourceSpan,
    },

    // TODO: this should be Error::Syntax, but we don't currently record
    // spans of expression
    Expr {
        message: &'static str,
    },
}

#[derive(Debug)]
enum IfKind {
    If { result: bool },
    Elif { result: bool },
    Else,
}

/// A small state machine for keeping track of the current state of `if`
/// statements and their expressions.
#[derive(Debug)]
struct IfState {
    state: IfKind,
    evaluated: bool,
    defined: SourceSpan,
}

impl IfState {
    fn new_if(result: bool, defined: SourceSpan) -> Self {
        Self {
            state: IfKind::If { result },
            evaluated: false,
            defined,
        }
    }

    fn eval_elif(&mut self, result: bool) -> Result<(), Error> {
        let was_true = match self.state {
            IfKind::If { result } | IfKind::Elif { result } => result,
            IfKind::Else => {
                self.evaluated = true;
                Err(Error::Expr {
                    message: "#elif after #else",
                })?
            }
        };

        self.state = IfKind::Elif { result };
        self.evaluated = self.evaluated || was_true;
        Ok(())
    }

    fn eval_else(&mut self) -> Result<(), Error> {
        let was_true = match self.state {
            IfKind::If { result } | IfKind::Elif { result } => result,
            IfKind::Else => {
                self.evaluated = true;
                Err(Error::Expr {
                    message: "#else after #else",
                })?
            }
        };

        self.state = IfKind::Else;
        self.evaluated = self.evaluated || was_true;
        Ok(())
    }

    fn is_active(&self) -> bool {
        if self.evaluated {
            false
        } else {
            match self.state {
                IfKind::Else => true,
                IfKind::If { result } | IfKind::Elif { result } => result,
            }
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct State<'a> {
    defines: HashMap<String, Macro>,
    errors: Vec<Error>,
    warnings: Vec<Error>,
    queue: VecDeque<Token>,
    vfs: &'a mut SourceMap,
}

impl<'a> State<'a> {
    pub fn new(vfs: &'a mut SourceMap) -> Self {
        State {
            vfs,
            defines: HashMap::default(),
            errors: vec![],
            warnings: vec![],
            queue: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn is_defined(&self, macr: &str) -> bool {
        self.defines.contains_key(macr)
    }

    #[must_use]
    pub fn get_macro(&self, macr: &str) -> Option<&Macro> {
        self.defines.get(macr)
    }

    #[must_use]
    pub fn warnings(&self) -> &[Error] {
        &self.warnings
    }

    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
}

pub trait PragmaHandler {
    /// Name of the pragma, e.g. `once` for `#pragma once`.
    fn name(&self) -> &str;

    // TODO: some way to emit a warning about extra (unused) tokens...
    fn handle(&mut self, tokens: Vec<Token>);
}

/// Operator precedence is defined as follows, from highest to lowest:
///     1. unary `+`, unary `-`, logical `NOT`, bitwise `NOT`
///     2. multiplication, division, modulo
///     3. addition, subtraction
///     4. `<<`, `>>`
///     5. `<`, `<=`, `>`, `>=`
///     6. `==`, `!=`
///     7. bitwise `AND`
///     8. bitwise `XOR`
///     9. bitwise `OR`
///     10. logical `AND`
///     11. logical `OR`
///     12. ternary conditional
#[inline]
const fn infix_precedence(kind: Kind) -> Option<u8> {
    let prec = match kind {
        Kind::Question => 1,
        Kind::Or => 2,
        Kind::And => 3,
        Kind::BitOr => 4,
        Kind::BitXor => 5,
        Kind::BitAnd => 6,
        Kind::EqEq | Kind::NotEq => 7,
        Kind::Lt | Kind::Gt | Kind::LtEq | Kind::GtEq => 8,
        // Kind::LShift | Kind::RShift => 9,
        Kind::Plus | Kind::Minus => 10,
        Kind::Star | Kind::Slash | Kind::Modulo => 11,
        _ => return None,
    };
    Some(prec)
}

// There are only a few prefix operators and they all have the same precedence.
#[inline]
fn prefix_precedence(kind: Kind) -> u8 {
    match kind {
        Kind::Plus | Kind::Minus | Kind::Not | Kind::BitNot => 20,
        _ => unreachable!("invalid unary operator {kind:?}"),
    }
}

#[inline]
fn checked_wdiv(lhs: i128, rhs: i128) -> Result<i128, Error> {
    if rhs == 0 {
        Err(Error::Expr {
            message: "attempted to divide by zero",
        })
    } else {
        Ok(lhs.wrapping_div(rhs))
    }
}

#[inline]
fn checked_wmod(lhs: i128, rhs: i128) -> Result<i128, Error> {
    if rhs == 0 {
        Err(Error::Expr {
            message: "attempted to modulo by zero",
        })
    } else {
        Ok(lhs.wrapping_rem(rhs))
    }
}

#[inline]
fn parse_str(str: &str, base: Base) -> Result<i128, Error> {
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
    })
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

struct Parser<'a, 'ctx> {
    stack: Vec<File>,
    state: &'a mut State<'ctx>,
    args: &'a ProcArgs,

    /// Set of files we've already parsed.
    /// Used to enable `#pragma once`-like functionality.
    parsed_files: HashSet<FileId>,
    // handler: &'a PragmaHandler,
}

impl<'a, 'ctx> Parser<'a, 'ctx> {
    fn with_state(file: File, args: &'a ProcArgs, state: &'a mut State<'ctx>) -> Self {
        Self {
            state,
            stack: vec![file],
            args,
            parsed_files: HashSet::default(),
        }
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
        if let Some(tok) = self.cursor().next() {
            match tok.kind {
                Kind::Ident => self.directive(tok.span),
                Kind::Number { .. } | Kind::Newline => (),
                _ => {
                    self.state.errors.push(Error::Syntax {
                        message: "invalid preprocessing directive",
                        span: tok.span,
                    });
                }
            }
        }
    }

    fn update_builtins(&mut self) {
        let file = self.cursor().file_id();
        let _path = self.state.vfs.path(file).to_string_lossy().to_string();
        let _line = self.cursor().line().to_string();
        let _time = time::utc_time();
        let _date = time::date();

        // let pairs = [
        //     ("__TIME__".to_string(), time),
        //     ("__DATE__".to_string(), date),
        //     ("__FILE__".to_string(), file),
        //     ("__LINE__".to_string(), line),
        // ];
        // self.state.defines.extend(
        //     pairs
        //         .into_iter()
        //         .map(|(key, val)| (key, Macro::Builtin(val))),
        // );
    }

    fn macro_name(&mut self) -> Option<(&'a str, SourceSpan)> {
        let tok = self.expect(Kind::Ident, "macro name must be an identifier")?;
        let Token {
            kind: Kind::Ident,
            span,
        } = tok
        else {
            self.state.errors.push(Error::Syntax {
                message: "macro name must be an identifier",
                span: tok.span,
            });
            return None;
        };
        Some((self.source_of(span), span))
    }

    fn source_of(&self, span: SourceSpan) -> &'a str {
        let Some(file) = self.stack.last() else {
            unreachable!("cursor stack is empty");
        };

        let src = if file.cursor.file_id() == span.file_id {
            file.cursor.source_of(span)
        } else {
            &self.state.vfs.source_str(span.file_id)[span.range()]
        };
        unsafe { std::mem::transmute::<&str, &'a str>(src) }
    }

    fn is_defined(&self, name: &str) -> bool {
        self.state.is_defined(name)
    }

    /// Collects trailing tokens and produces a warning, e.g. for things like
    /// `#undef foo bar`, where "bar" is an extraneous token.
    fn warn_trailing(&mut self, span: SourceSpan, directive: Directive) {
        let tokens = self.cursor().until_newline();
        if !tokens.is_empty() {
            self.state.warnings.push(Error::Extraneous {
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
        self.if_state().last().map_or(true, IfState::is_active)
    }

    fn dir_include(&mut self, span: SourceSpan) {
        let cursor = self.cursor();
        let (kind, path) = match cursor.peek() {
            Some(Kind::Lt) => {
                _ = cursor.next();
                let (_, span) = cursor.until_peek(Kind::Gt);
                _ = cursor.next();
                (Include::System, span)
            }
            Some(Kind::String) => {
                let tok = cursor.next().unwrap();
                (Include::Local, tok.span)
            }
            _ => {
                self.expect(Kind::String, "expected \"file\" or <file>");
                return;
            }
        };
        self.warn_trailing(span, Directive::Include);

        if self.is_active() {
            // Bail if we've hit the recursion depth
            if self.stack.len() >= self.args.recursion_depth {
                self.state.errors.push(Error::Syntax {
                    message: "#include nested too deeply",
                    span,
                });
                return;
            }

            let include = self.source_of(path);
            let include = include.trim_start_matches('"').trim_end_matches('"');

            match self.state.vfs.open(include, kind) {
                Ok((id, source)) => {
                    // Skip files that we've already parsed if they used the
                    // `once` pragma.
                    if !self.parsed_files.contains(&id) {
                        let cursor = File::from_src(source, id);
                        self.stack.push(cursor);
                    }
                }
                Err(e) => self.state.errors.push(Error::Syntax {
                    message: "failed to open file",
                    span,
                }),
            }
        }
    }

    fn dir_define(&mut self) -> Option<()> {
        let (name, span) = self.macro_name()?;
        let is_macro = self.cursor().take_if(Kind::LParen).is_some();

        let definition = if is_macro {
            let mut args = vec![];

            // Empty function macros are allowed
            while let Some(c) = self.cursor().peek() {
                if c == Kind::RParen {
                    break;
                }

                let arg = self.expect(Kind::Ident, "invalid token in macro parameter list")?;
                args.push(arg);

                if self.cursor().take_if(Kind::Comma).is_none() {
                    self.expect(Kind::RParen, "expected comma or end of parameter list")?;
                    break;
                }
            }

            let def = self.cursor().until_newline();
            Macro::Function { span, args, def }
        } else {
            let def = self.cursor().until_newline();
            Macro::Object { span, def }
        };

        if self.is_active()
            && self
                .state
                .defines
                .insert(name.to_string(), definition)
                .is_some()
        {
            self.state.warnings.push(Error::Syntax {
                message: "macro redefined",
                span,
            });
        }
        Some(())
    }

    fn dir_undef(&mut self) {
        let Some((name, span)) = self.macro_name() else {
            return;
        };
        self.warn_trailing(span, Directive::Undef);

        if self.is_active() {
            self.state.defines.remove(name);
        }
    }

    fn expr_and_eval(&mut self) -> bool {
        match self.expr().and_then(|v| self.is_true(&v)) {
            Ok(v) => v,
            Err(e) => {
                self.state.errors.push(e);
                false
            }
        }
    }

    fn dir_if(&mut self, span: SourceSpan) {
        let result = self.expr_and_eval();
        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifdef(&mut self, span: SourceSpan) {
        let result = if let Some((name, name_span)) = self.macro_name() {
            self.warn_trailing(name_span, Directive::Ifdef);
            self.is_defined(name)
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifndef(&mut self, span: SourceSpan) {
        let result = if let Some((name, name_span)) = self.macro_name() {
            self.warn_trailing(name_span, Directive::Ifndef);
            !self.is_defined(name)
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_else(&mut self, span: SourceSpan) {
        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_else() {
                    self.state.errors.push(e);
                }
            }
            None => self.state.errors.push(Error::Syntax {
                message: "#else without #if",
                span,
            }),
        }
    }

    fn dir_elif(&mut self, span: SourceSpan) {
        let result = self.expr_and_eval();

        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_elif(result) {
                    self.state.errors.push(e);
                }
            }
            None => self.state.errors.push(Error::Syntax {
                message: "#elif without #if",
                span,
            }),
        }
    }

    fn dir_endif(&mut self, span: SourceSpan) -> Result<(), Error> {
        if self.if_state().pop().is_some() {
            Ok(())
        } else {
            Err(Error::Syntax {
                message: "#endif without #if",
                span,
            })
        }
    }

    fn dir_warning(&mut self, span: SourceSpan) {
        let tokens = self.cursor().until_newline();
        if self.is_active() {
            self.state.warnings.push(Error::Note { span, tokens });
        }
    }

    fn dir_error(&mut self, span: SourceSpan) {
        let tokens = self.cursor().until_newline();
        if self.is_active() {
            self.state.errors.push(Error::Note { span, tokens });
        }
    }

    // There are no strict requirements for what a pragma must be. It can be an
    // integer, a control character, an UTF-8 character, etc. so we just accept
    // whatever.
    fn dir_pragma(&mut self) {
        let tokens = self.cursor().until_newline();

        // Empty pragmas are allowed, so this is not guaranteed
        if let Some(pragma) = tokens.first() {
            let name = self.source_of(pragma.span);
            if name == "once" {
                let id = self.cursor().file_id();
                self.parsed_files.insert(id);
            }
        }
    }

    // Parse the directive but disregard its contents.
    fn dir_line(&mut self, span: SourceSpan) -> Option<()> {
        // Only decimal numbers allowed here
        let _line = self.expect(
            Kind::Number {
                base: Base::Decimal,
            },
            "expected decimal line number",
        )?;

        let _file = self.expect(Kind::String, "expected file name as string literal")?;
        self.warn_trailing(span, Directive::Line);
        Some(())
    }

    fn expect(&mut self, kind: Kind, message: &'static str) -> Option<Token> {
        if let Some(tok) = self.cursor().next() {
            if tok.kind == kind {
                return Some(tok);
            }

            self.state.errors.push(Error::Syntax {
                message,
                span: tok.span,
            });
            self.cursor().until_newline();
        }
        None
    }

    fn directive(&mut self, span: SourceSpan) {
        match self.source_of(span) {
            "if" => self.dir_if(span),
            "ifdef" => self.dir_ifdef(span),
            "ifndef" => self.dir_ifndef(span),
            "elif" => self.dir_elif(span),
            "else" => self.dir_else(span),
            "endif" => _ = self.dir_endif(span),
            "pragma" => self.dir_pragma(),
            "define" => _ = self.dir_define(),
            "undef" => self.dir_undef(),
            "include" => self.dir_include(span),
            "warning" => self.dir_warning(span),
            "error" => self.dir_error(span),
            "line" => _ = self.dir_line(span),
            v => {
                self.state.errors.push(Error::Syntax {
                    message: "invalid preprocessing directive",
                    span,
                });
            }
        }
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        self.binary_expr(0)
    }

    fn unary_expr(&mut self) -> Result<Expr, Error> {
        let lhs = self.next().ok_or(Error::Expr {
            message: "unexpected end of expression",
        })?;

        let expr = match lhs.kind {
            Kind::Ident | Kind::Number { .. } => Expr::Lit(lhs),
            Kind::Plus | Kind::Minus | Kind::Not | Kind::BitNot => {
                let prefix = prefix_precedence(lhs.kind);
                let expr = self.binary_expr(prefix)?;
                Expr::Unary(Box::new(Unary {
                    op: Op::try_from(lhs)?,
                    expr,
                }))
            }
            Kind::LParen => {
                let expr = self.binary_expr(0)?;
                self.expect(Kind::RParen, "unterminated expression group");
                expr
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

    // Note that this function uses `Parser::next` instead of `Cursor::next` as
    // we need to expand and inline macros during parsing.
    fn binary_expr(&mut self, min_prec: u8) -> Result<Expr, Error> {
        let mut lhs = self.unary_expr()?;

        while let Some(op) = self.next() {
            // We require a lookahead of 1 here, but doing so involves expanding
            // and consuming the next token in the sequence. So if this is not
            // an operator, or an operator of lower precedence, we push it back
            // on the queue.
            let prec = match infix_precedence(op.kind) {
                Some(prec) if prec >= min_prec => prec,
                _ => {
                    self.state.queue.push_front(op);
                    break;
                }
            };

            lhs = if op.kind == Kind::Question {
                let then = self.expr()?;
                self.expect(Kind::Colon, "expected colon in ternary operator");
                let els = self.binary_expr(prec + 1)?;
                Expr::Ternary(Box::new(Ternary {
                    cond: lhs,
                    then,
                    els,
                }))
            } else {
                let op = Op::try_from(op)?;
                let rhs = self.binary_expr(prec + 1)?;
                Expr::Binary(Box::new(Binary { lhs, op, rhs }))
            }
        }
        Ok(lhs)
    }

    fn eval_expr(&self, expr: &Expr) -> Result<i128, Error> {
        let val = match expr {
            Expr::Lit(v) => {
                let lit = self.source_of(v.span);
                match v.kind {
                    Kind::Number { base } => parse_str(lit, base)?,
                    Kind::Ident => 0,
                    _ => unreachable!(),
                }
            }
            Expr::Unary(v) => {
                let expr = self.eval_expr(&v.expr)?;
                match v.op {
                    Op::Add => expr,
                    Op::Sub => -expr,
                    Op::Not => i128::from(expr == 0),
                    v => unreachable!("invalid unary operator: {v:?}"),
                }
            }
            Expr::Binary(v) => {
                let lhs = self.eval_expr(&v.lhs)?;
                let rhs = self.eval_expr(&v.rhs)?;
                match v.op {
                    Op::And => i128::from(lhs != 0 && rhs != 0),
                    Op::Or => i128::from(lhs != 0 || rhs != 0),
                    Op::EqEq => i128::from(lhs == rhs),
                    Op::NotEq => i128::from(lhs != rhs),
                    Op::Gt => i128::from(lhs > rhs),
                    Op::GtEq => i128::from(lhs >= rhs),
                    Op::Lt => i128::from(lhs < rhs),
                    Op::LtEq => i128::from(lhs <= rhs),
                    Op::BitAnd => lhs & rhs,
                    Op::BitOr => lhs | rhs,
                    Op::BitXor => lhs ^ rhs,
                    Op::Add => lhs.wrapping_add(rhs),
                    Op::Sub => lhs.wrapping_sub(rhs),
                    Op::Mul => lhs.wrapping_mul(rhs),
                    Op::Div => checked_wdiv(lhs, rhs)?,
                    Op::Mod => checked_wmod(lhs, rhs)?,
                    v => unreachable!("invalid binary operator: {v:?}"),
                }
            }
            Expr::Ternary(v) => {
                if self.is_true(&v.cond)? {
                    self.eval_expr(&v.then)?
                } else {
                    self.eval_expr(&v.els)?
                }
            }
        };
        Ok(val)
    }

    fn is_true(&self, expr: &Expr) -> Result<bool, Error> {
        self.eval_expr(expr).map(|v| v != 0)
    }

    fn expand_inner(&mut self, token: Token, seen: &mut BTreeSet<&'a str>) {
        // Only identifiers can be macros
        if token.kind != Kind::Ident {
            self.state.queue.push_back(token);
            return;
        }

        let name = self.source_of(token.span);
        if let Some(v) = self.state.defines.get(name) {
            // Macros should not be recursively expanded
            if !seen.insert(name) {
                self.state.queue.push_back(token);
                return;
            }

            // Bail if we've nested too deeply
            if seen.len() >= self.args.recursion_depth {
                self.state.errors.push(Error::Syntax {
                    message: "macro recursion depth limit was reached",
                    span: token.span,
                });
                return;
            }

            match v {
                Macro::Function { .. } => todo!(),
                Macro::Object { def, .. } => {
                    for tok in def.clone() {
                        let name = self.source_of(tok.span);
                        self.expand_inner(tok, seen);
                    }
                }
            };
            seen.remove(name);
        } else {
            self.state.queue.push_back(token);
        }
    }

    /// Expands and enqueues the definition of `tok` if it is a macro.
    /// Returns `true` if the macro was expanded.
    ///
    /// A macro expansion is not allowed to have side effects, so we
    /// can fully expand the entire macro definition here. This is
    /// important as we need to detect and break potential cycles.
    fn expand_macro(&mut self, tok: Token) -> bool {
        let name = self.source_of(tok.span);
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
            if let Some(tok) = self.state.queue.pop_front() {
                return Some(tok);
            }

            // Advance the cursor and continue parsing directives
            if let Some(tok) = self.stack.last_mut()?.cursor.next() {
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
            while let Some(cond) = top.current.pop() {
                self.state.errors.push(Error::Syntax {
                    message: "unterminated conditional directive",
                    span: cond.defined,
                });
            }

            // Cursor is empty, pop the stack
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
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct TokenIter<'a, 'ctx>(Parser<'a, 'ctx>);

impl<'a, 'ctx> TokenIter<'a, 'ctx> {
    #[must_use]
    pub fn source_of(&self, span: SourceSpan) -> &str {
        self.0.source_of(span)
    }
}

impl Iterator for TokenIter<'_, '_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_active()
    }
}

pub fn preprocess<'a, 'ctx>(
    file_id: FileId,
    args: &'a ProcArgs,
    state: &'a mut State<'ctx>,
) -> TokenIter<'a, 'ctx> {
    let source = state.vfs.source(file_id);
    let file = File::from_src(source, file_id);
    let parser = Parser::with_state(file, args, state);
    TokenIter(parser)
}

/// Preprocesses a file, inlines all includes and expands all macro definitions.
/// This does not retain whitespace as we don't currently hold the necessary
/// information to retain whitespace for macro expansion. We could have a type
/// that contains both the span of the now-expanded token and the macro
/// definition, that way we can calculate the whitespace between the last token
/// and the next. Something like:
///
/// ```rust,ignore
/// enum Tok {
///     Text(Token),
///     Expanded {
///         token: Token,
///         original_span: SourceSpan,
///     }
/// }
/// ```
///
/// But that's not been implemented yet. The C standard doesn't require that
/// we actually materialize the preprocessed document in any way. We don't need
/// it either since the preprocessor is effectively a lexer for our IDL parser.
pub fn to_string(file_id: FileId, args: &ProcArgs, state: &mut State<'_>) -> (String, Vec<Error>) {
    let src = state.vfs.source(file_id);
    let mut iter = preprocess(file_id, args, state);
    let mut buffer = String::with_capacity(src.len());

    while let Some(tok) = iter.next() {
        let slice = iter.source_of(tok.span);
        _ = buffer.write_str(slice);
        if tok.kind != Kind::Newline {
            _ = buffer.write_char(' ');
        }
    }
    (buffer, iter.0.state.errors.clone())
}

/// Formats the given set of tokens as a string.
pub fn format_tokens(tokens: &[Token], state: &State<'_>) -> String {
    let mut buffer = String::new();
    for tok in tokens {
        let src = state.vfs.source(tok.span.file_id);
        _ = buffer.write_str(&src[tok.span.range()]);
        if tok.kind != Kind::Newline {
            _ = buffer.write_char(' ');
        }
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pp<'a>(vfs: &'a mut SourceMap, input: &str) -> State<'a> {
        let id = vfs.embed(input);
        let mut state = State::new(vfs);
        preprocess(id, &ProcArgs::default(), &mut state).for_each(drop);
        state
    }

    fn with_state(state: &mut State<'_>, input: &str) -> Vec<Token> {
        let id = state.vfs.embed(input);
        preprocess(id, &ProcArgs::default(), state).collect()
    }

    fn expand(input: &str) -> String {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(input);
        let mut state = State::new(&mut vfs);
        let (output, _) = to_string(id, &ProcArgs::default(), &mut state);
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

        let Some(Macro::Object { def, .. }) = state.get_macro("foo") else {
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

        with_state(&mut state, "#undef foo");
        assert!(!state.is_defined("foo"));
        assert!(state.errors().is_empty());
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn undef_non_existent() {
        let mut vfs = SourceMap::default();
        let mut state = pp(&mut vfs, "#undef foo");
        assert!(state.errors().is_empty());
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn multiline_define() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #define foo bar \
                    baz
                123
            "#,
        );

        let Some(Macro::Object { def, .. }) = state.get_macro("foo") else {
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
            r#"
                #define foo \
                    bar \
                    baz
            "#,
        );

        let Some(Macro::Object { def, .. }) = state.get_macro("foo") else {
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
            r#"
                #define foo bar
                bar 123 bar
            "#,
        );
        assert_eq!(expanded, "bar 123 bar");
    }

    #[test]
    fn recursive_macro() {
        // A macro should not be expanded from the definition of itself, so
        // "foo" in the macro definition is just treated as a normal identifier
        // and not a macro to be expanded.
        let expanded = expand(
            r#"
                #define foo foo foo bar
                foo
            "#,
        );
        assert_eq!(expanded, "foo foo bar");
    }

    #[test]
    fn recursively_expand() {
        let expanded = expand(
            r#"
                #define baz 123
                #define bar baz
                #define foo bar
                foo
            "#,
        );
        assert_eq!(expanded, "123");
    }

    #[test]
    fn recursive_cyclic() {
        // Recursion stops as soon as we find a macro we've already expanded.
        let expanded = expand(
            r#"
                #define foo foo foo
                #define bar baz
                #define baz foo

                foo bar baz
            "#,
        );
        assert_eq!(expanded, "foo foo foo foo foo foo");

        let expanded = expand(
            r#"
                #define foo bar
                #define bar foo
                foo bar
            "#,
        );
        assert_eq!(expanded, "foo bar");
    }

    #[test]
    fn backslash() {
        let mut expanded = expand(
            r#"
                #define foo \ a
                foo
            "#,
        );
        assert_eq!(expanded, "\\ a");
    }

    #[test]
    fn spaceship() {
        let mut vfs = SourceMap::default();
        let state = pp(&mut vfs, "#define foo <==>");

        let Some(Macro::Object { def, .. }) = state.get_macro("foo") else {
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
            r#"
                // some comment \
                foo
            "#,
        );
        assert_eq!(expanded, "foo");
    }

    #[test]
    fn inactive_warnings() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #if 0
                #warning foo
                #endif
            "#,
        );
        assert!(state.warnings().is_empty());
    }

    #[test]
    fn inactive_errors() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #if 0
                #error foo
                #endif
            "#,
        );
        assert!(state.errors().is_empty());
    }

    #[test]
    fn expand_inactive() {
        let expanded = expand(
            r#"
                #define foo bar
                #if 0
                foo
                #endif
            "#,
        );
        assert!(expanded.is_empty());
    }

    #[test]
    fn define_inactive() {
        let expanded = expand(
            r#"
                #if 0
                #define foo bar
                #endif
                foo
            "#,
        );
        assert_eq!(expanded, "foo");
    }

    #[test]
    fn redefine_object() {
        let mut vfs = SourceMap::default();
        let mut state = pp(
            &mut vfs,
            r#"
                #define foo 123
                #define foo 456
                #define foo bar
            "#,
        );
        // Not an error, and the value should be updated, but we should emit
        // a warning each time it is redefined.
        assert!(state.errors().is_empty());
        assert_eq!(state.warnings().len(), 2);

        // Last definition is the one that counts
        let expanded = with_state(&mut state, "foo");
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
            r#"
                #ifdef true foo
                #endif
            "#,
        );
        assert_eq!(state.warnings().len(), 1);
    }

    #[test]
    fn extra_tokens_ifndef() {
        let mut vfs = SourceMap::default();
        let state = pp(
            &mut vfs,
            r#"
                #ifndef true foo
                #endif
            "#,
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
            r#"
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
            "#,
        );
        assert!(state.errors().is_empty());
        assert!(state.warnings().is_empty());
    }
}
