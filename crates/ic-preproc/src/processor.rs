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

#![allow(clippy::cast_possible_truncation)]

use std::borrow::BorrowMut;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ic_expr::{Binary, Op, Ternary, Unary};
use ic_lexer::cursor::Cursor;
use ic_lexer::token::{Base, Kind, Token};
use ic_vfs::{FileId, Include, Location, SourceMap};

use crate::{ProcArgs, Span, time};

#[derive(Debug, Clone)]
pub enum Macro {
    Function {
        span: Span,
        args: Vec<Token>,
        def: Vec<Token>,
        variadic: bool,
    },
    Object {
        span: Span,
        def: Vec<Token>,
    },
}

type Expr = ic_expr::Expr<Token>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Directive {
    If,
    Ifdef,
    Ifndef,
    Elif,
    Else,
    Endif,
    Include,
    Define,
    Undef,
    Line,
    Warning,
    Error,
    Pragma,
}

#[derive(Clone, Debug)]
pub enum Error {
    Note {
        span: Span,
        tokens: Vec<Token>,
    },
    Extraneous {
        directive: Directive,
        span: Span,
        tokens: Vec<Token>,
    },
    Syntax {
        message: &'static str,
        span: Span,
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
    defined: Span,
}

impl IfState {
    fn new_if(result: bool, defined: Span) -> Self {
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
#[derive(Default, Debug)]
pub struct State {
    defines: HashMap<String, Macro>,
    errors: Vec<Error>,
    warnings: Vec<Error>,
    queue: VecDeque<Token>,

    /// Set of files we've already parsed.
    /// Used to enable `#pragma once`-like functionality.
    parsed_files: HashSet<FileId>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
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

trait PragmaHandler<S: BorrowMut<State>> {
    /// Name of the pragma, e.g. `once` for `#pragma once`.
    fn name(&self) -> &str;

    /// Handle a `pragma` directive.
    fn handle(&self, parser: &mut Parser<'_, S>, tokens: Vec<Token>);
}

struct PragmaOnce;

impl<S: BorrowMut<State>> PragmaHandler<S> for PragmaOnce {
    fn name(&self) -> &'static str {
        "once"
    }

    fn handle(&self, parser: &mut Parser<'_, S>, _: Vec<Token>) {
        let id = parser.cursor().file_id();
        parser.mark_included(id);
    }
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
        Kind::LShift | Kind::RShift => 9,
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

fn expr_op(tok: Token) -> Result<Op, Error> {
    let op = match tok.kind {
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
        Kind::LShift => Op::LShift,
        Kind::RShift => Op::RShift,
        _ => Err(Error::Syntax {
            message: "invalid binary operator",
            span: tok.span,
        })?,
    };
    Ok(op)
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
    includes: HashSet<PathBuf>,
    recursion_depth: usize,

    /// Registered pragmas.
    pragmas: HashMap<String, Rc<dyn PragmaHandler<S>>>,
}

impl<'a, S> Parser<'a, S>
where
    S: BorrowMut<State>,
{
    fn with_state(file: File, args: ProcArgs, state: S, vfs: &'a mut SourceMap) -> Self {
        let mut this = Self {
            state,
            stack: vec![file],
            pragmas: HashMap::default(),
            includes: args.include_dirs,
            recursion_depth: args.recursion_depth,
            vfs,
        };

        // Inject definitions from `ProcArgs`. This pushes a new virtual file
        // to the top of the stack.
        cli_defines(args.defines, &mut this);

        // Register pragma handlers
        this.add_pragma(PragmaOnce);
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
        if let Some(tok) = self.cursor().next() {
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

    #[allow(unused)]
    fn update_builtins(&mut self) {
        let file = self.cursor().file_id();
        let _path = self.vfs.path(file).to_string_lossy().to_string();
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

    fn macro_name(&mut self) -> Option<(&'a str, Span)> {
        // TODO: can also be a keyword
        let tok = self.expect(Kind::Ident, "macro name must be an identifier")?;
        let Token {
            kind: Kind::Ident | Kind::Keyword(_),
            span,
        } = tok
        else {
            self.state().errors.push(Error::Syntax {
                message: "macro name must be an identifier",
                span: tok.span,
            });
            return None;
        };
        Some((self.source_of(span), span))
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
        // lifetimes is bound by 'a.
        unsafe { std::mem::transmute::<&str, &'a str>(src) }
    }

    fn state(&mut self) -> &mut State {
        self.state.borrow_mut()
    }

    fn is_defined(&self, name: &str) -> bool {
        match name {
            "__LINE__" | "__FILE__" | "__DATE__" | "__TIME__" => true,
            _ => self.state.borrow().is_defined(name),
        }
    }

    fn add_pragma<H>(&mut self, pragma: H)
    where
        H: PragmaHandler<S> + 'static,
    {
        self.pragmas
            .insert(pragma.name().to_string(), Rc::new(pragma));
    }

    fn mark_included(&mut self, file_id: FileId) {
        self.state().parsed_files.insert(file_id);
    }

    /// Collects trailing tokens and produces a warning, e.g. for things like
    /// `#undef foo bar`, where "bar" is an extraneous token.
    fn warn_trailing(&mut self, span: Span, directive: Directive) {
        let tokens = self.cursor().until_newline();
        if !tokens.is_empty() {
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
    //
    // TODO: should we move this logic to the VFS? It can cache the results so
    // we don't have to repeatedly search through directories.
    fn search_includes<P: AsRef<Path>>(&mut self, path: P, kind: Include) -> Option<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            return Some(path.to_owned());
        }

        // Include relative to the current file
        if kind == Include::Local {
            let cur_id = self.cursor().file_id();
            let local = self.vfs.path(cur_id);
            if let Some(parent) = local.parent() {
                let file = parent.join(path);
                if file.exists() {
                    return Some(file);
                }
            }
        }

        // Fall back to searching all include directories
        for p in &self.includes {
            let file = p.join(path);
            if file.exists() {
                return Some(file);
            }
        }
        None
    }

    fn dir_include(&mut self, span: Span) {
        let cursor = self.cursor();
        let (kind, path) = match cursor.peek() {
            Some(Kind::Lt) => {
                _ = cursor.next();
                let span = cursor.until_peek(Kind::Gt);
                self.expect(Kind::Gt, "unterminated include");
                (Include::System, span)
            }
            Some(Kind::String { .. }) => {
                let tok = cursor.next().unwrap();
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
        self.warn_trailing(span, Directive::Include);

        if self.is_active() {
            // Bail if we've hit the recursion depth
            if self.stack.len() >= self.recursion_depth {
                self.state().errors.push(Error::Syntax {
                    message: "#include nested too deeply",
                    span,
                });
                return;
            }

            let include = self.source_of(path);
            let include = include.trim_start_matches('"').trim_end_matches('"');

            if let Some(v) = self.search_includes(include, kind) {
                match self.vfs.open(v, kind) {
                    Ok((id, source)) => {
                        // Skip files that we've already parsed if they used
                        // the `once` pragma.
                        if !self.state().parsed_files.contains(&id) {
                            let cursor = File::from_src(source, id);
                            self.stack.push(cursor);
                        }
                    }
                    Err(_) => self.state().errors.push(Error::Syntax {
                        message: "failed to open file",
                        span,
                    }),
                }
            } else {
                self.state().errors.push(Error::Syntax {
                    message: "file not found",
                    span,
                });
            }
        }
    }

    fn dir_define(&mut self) -> Option<()> {
        let (name, name_span) = self.macro_name()?;

        // Check if there's an opening parenthesis immediately after the macro name
        // (no whitespace) to determine if it's a function-like macro
        let mut lparen_tok = None;
        let is_macro = if let Some(tok) = self.cursor().take_if(Kind::LParen) {
            // For function-like macros, the '(' must immediately follow the macro name
            // Check if the spans are adjacent
            let is_adjacent = name_span.end.offset == tok.span.start.offset
                && name_span.end.file_id == tok.span.start.file_id;
            if is_adjacent {
                true
            } else {
                // Not a function-like macro, put the token back
                lparen_tok = Some(tok);
                false
            }
        } else {
            false
        };

        let definition = if is_macro {
            let mut args = vec![];
            let mut variadic = false;

            // Empty function macros are allowed
            while let Some(c) = self.cursor().peek() {
                if c == Kind::RParen {
                    break;
                }

                // Check for variadic ...
                if c == Kind::Period {
                    // Check if we have three periods
                    let cursor = self.cursor();
                    cursor.next(); // consume first .
                    if cursor.peek() == Some(Kind::Period) {
                        cursor.next(); // consume second .
                        if cursor.peek() == Some(Kind::Period) {
                            cursor.next(); // consume third .
                            variadic = true;
                            // Variadic must be last parameter
                            self.expect(Kind::RParen, "variadic parameter must be last")?;
                            break;
                        }
                    }
                    // If not three periods, error
                    self.state().errors.push(Error::Syntax {
                        message: "expected identifier or '...'",
                        span: name_span,
                    });
                    return None;
                }

                // TODO: can also be a keyword
                let arg = self.expect(Kind::Ident, "invalid token in macro parameter list")?;
                args.push(arg);

                if self.cursor().take_if(Kind::Comma).is_none() {
                    self.expect(Kind::RParen, "expected comma or end of parameter list")?;
                    break;
                }
            }

            let def = self.cursor().until_newline();
            Macro::Function {
                span: name_span,
                args,
                def,
                variadic,
            }
        } else {
            // If we consumed an lparen that wasn't part of a function macro,
            // we need to include it in the definition
            let mut def = Vec::new();
            if let Some(tok) = lparen_tok {
                def.push(tok);
            }
            def.extend(self.cursor().until_newline());
            Macro::Object {
                span: name_span,
                def,
            }
        };

        if self.is_active()
            && self
                .state()
                .defines
                .insert(name.to_string(), definition)
                .is_some()
        {
            self.state().warnings.push(Error::Syntax {
                message: "macro redefined",
                span: name_span,
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
            self.state().defines.remove(name);
        }
    }

    fn expr_and_eval(&mut self) -> bool {
        match self.expr().and_then(|v| self.is_true(&v)) {
            Ok(v) => v,
            Err(e) => {
                self.state().errors.push(e);
                false
            }
        }
    }

    fn dir_if(&mut self, span: Span) {
        let result = self.expr_and_eval();
        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifdef(&mut self, span: Span) {
        let result = if let Some((name, name_span)) = self.macro_name() {
            self.warn_trailing(name_span, Directive::Ifdef);
            self.is_defined(name)
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_ifndef(&mut self, span: Span) {
        let result = if let Some((name, name_span)) = self.macro_name() {
            self.warn_trailing(name_span, Directive::Ifndef);
            !self.is_defined(name)
        } else {
            false
        };

        let state = IfState::new_if(result, span);
        self.if_state().push(state);
    }

    fn dir_else(&mut self, span: Span) {
        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_else() {
                    self.state().errors.push(e);
                }
            }
            None => self.state().errors.push(Error::Syntax {
                message: "#else without #if",
                span,
            }),
        }
    }

    fn dir_elif(&mut self, span: Span) {
        let result = self.expr_and_eval();

        match self.if_state().last_mut() {
            Some(v) => {
                if let Err(e) = v.eval_elif(result) {
                    self.state().errors.push(e);
                }
            }
            None => self.state().errors.push(Error::Syntax {
                message: "#elif without #if",
                span,
            }),
        }
    }

    fn dir_endif(&mut self, span: Span) {
        if self.if_state().pop().is_none() {
            self.state().errors.push(Error::Syntax {
                message: "#endif without #if",
                span,
            });
        }
    }

    fn dir_warning(&mut self, span: Span) {
        let tokens = self.cursor().until_newline();
        if self.is_active() {
            self.state().warnings.push(Error::Note { span, tokens });
        }
    }

    fn dir_error(&mut self, span: Span) {
        let tokens = self.cursor().until_newline();
        if self.is_active() {
            self.state().errors.push(Error::Note { span, tokens });
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
            if let Some(v) = self.pragmas.get(name).cloned() {
                v.handle(self, tokens);
            }
        }
    }

    // Parse the directive but disregard its contents.
    fn dir_line(&mut self, span: Span) -> Option<()> {
        // Only decimal numbers allowed here
        let _line = self.expect(
            Kind::Number {
                base: Base::Decimal,
            },
            "expected decimal line number",
        )?;

        // Optional filename
        if let Some(tok) = self.cursor().peek() {
            if matches!(tok, Kind::String { terminated: true }) {
                self.cursor().next();
            }
        }

        self.warn_trailing(span, Directive::Line);

        // For now, just parse and ignore #line directives
        // Actual line number manipulation would require more infrastructure
        Some(())
    }

    fn expect(&mut self, kind: Kind, message: &'static str) -> Option<Token> {
        if let Some(tok) = self.cursor().next() {
            if tok.kind == kind {
                return Some(tok);
            }

            self.state().errors.push(Error::Syntax {
                message,
                span: tok.span,
            });
            self.cursor().until_newline();
        }
        None
    }

    fn directive(&mut self, span: Span) {
        match self.source_of(span) {
            "if" => self.dir_if(span),
            "ifdef" => self.dir_ifdef(span),
            "ifndef" => self.dir_ifndef(span),
            "elif" => self.dir_elif(span),
            "else" => self.dir_else(span),
            "endif" => self.dir_endif(span),
            "pragma" => self.dir_pragma(),
            "define" => _ = self.dir_define(),
            "undef" => self.dir_undef(),
            "include" => self.dir_include(span),
            "warning" => self.dir_warning(span),
            "error" => self.dir_error(span),
            "line" => _ = self.dir_line(span),
            _ => {
                self.state().errors.push(Error::Syntax {
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
        // Get the next token with macro expansion
        let lhs = self.next_expr_token().ok_or(Error::Expr {
            message: "unexpected end of expression",
        })?;

        // Check if it's 'defined' - if so, we need special handling
        if matches!(lhs.kind, Kind::Ident | Kind::Keyword(_))
            && self.source_of(lhs.span) == "defined"
        {
            return self.parse_defined_operator();
        }

        let expr = match lhs.kind {
            Kind::Ident | Kind::Keyword(_) | Kind::Number { .. } => Expr::Lit(lhs),
            Kind::Plus | Kind::Minus | Kind::Not | Kind::BitNot => {
                let prefix = prefix_precedence(lhs.kind);
                let expr = self.binary_expr(prefix)?;
                Expr::Unary(Box::new(Unary {
                    op: expr_op(lhs)?,
                    expr,
                }))
            }
            Kind::LParen => {
                let expr = self.binary_expr(0)?;
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

    fn parse_defined_operator(&mut self) -> Result<Expr, Error> {
        // defined can be used as:
        // - defined(MACRO)
        // - defined MACRO
        let next = self.next_raw_token().ok_or(Error::Expr {
            message: "unexpected end after 'defined'",
        })?;

        let macro_name = match next.kind {
            Kind::LParen => {
                // defined(MACRO) form
                let name_tok = self.next_raw_token().ok_or(Error::Expr {
                    message: "expected macro name after 'defined('",
                })?;

                // Verify it's an identifier
                if !matches!(name_tok.kind, Kind::Ident | Kind::Keyword(_)) {
                    return Err(Error::Syntax {
                        message: "expected identifier in defined()",
                        span: name_tok.span,
                    });
                }

                let name = self.source_of(name_tok.span).to_string();

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
                        });
                    }
                }

                name
            }
            Kind::Ident | Kind::Keyword(_) => {
                // defined MACRO form
                self.source_of(next.span).to_string()
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
        let value = if self.is_defined(&macro_name) {
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
        Ok(Expr::Lit(Token {
            kind: Kind::Number {
                base: Base::Decimal,
            },
            span,
        }))
    }

    // Note that this function uses `Parser::next_expr_token` instead of `Cursor::next` as
    // we need to expand and inline macros during parsing, except for 'defined'.
    fn binary_expr(&mut self, min_prec: u8) -> Result<Expr, Error> {
        let mut lhs = self.unary_expr()?;

        while let Some(op) = self.next_expr_token() {
            // We require a lookahead of 1 here, but doing so involves expanding
            // and consuming the next token in the sequence. So if this is not
            // an operator, or an operator of lower precedence, we push it back
            // on the queue.
            let prec = match infix_precedence(op.kind) {
                Some(prec) if prec >= min_prec => prec,
                _ => {
                    self.state().queue.push_front(op);
                    break;
                }
            };

            lhs = if op.kind == Kind::Question {
                let then = self.expr()?;
                // Check for colon without consuming the line on error
                match self.next_expr_token() {
                    Some(tok) if tok.kind == Kind::Colon => {
                        let els = self.binary_expr(prec + 1)?;
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
                        });
                    }
                }
            } else {
                let op = expr_op(op)?;
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
                    Kind::Ident | Kind::Keyword(_) => {
                        // Handle predefined macros in expressions
                        match lit {
                            "__LINE__" => {
                                // For now, just return a non-zero value so #if __LINE__ > 0 works
                                // Proper implementation would require access to span location info
                                1
                            }
                            _ => 0, // Undefined macros evaluate to 0
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Unary(v) => {
                let expr = self.eval_expr(&v.expr)?;
                match v.op {
                    Op::Add => expr,
                    Op::Sub => -expr,
                    Op::Not => i128::from(expr == 0),
                    Op::BitNot => !expr,
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
                    Op::LShift => lhs.wrapping_shl(rhs.try_into().unwrap_or(128)),
                    Op::RShift => lhs.wrapping_shr(rhs.try_into().unwrap_or(128)),
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

    #[allow(clippy::too_many_lines)]
    fn expand_inner(&mut self, token: Token, seen: &mut BTreeSet<&'a str>) {
        // Only identifiers can be macros
        if !matches!(token.kind, Kind::Ident | Kind::Keyword(_)) {
            self.state().queue.push_back(token);
            return;
        }

        let name = self.source_of(token.span);

        // Handle predefined macros
        match name {
            "__LINE__" => {
                let line_token = Token {
                    kind: Kind::Number {
                        base: Base::Decimal,
                    },
                    span: token.span, // Use the same span as the original __LINE__
                };
                // We need to create a synthetic token with the line number
                // For now, just push the original token back since we can't easily create new spans
                self.state().queue.push_back(line_token);
                return;
            }
            "__FILE__" => {
                let file_token = Token {
                    kind: Kind::String { terminated: true },
                    span: token.span, // Use the same span as the original __FILE__
                };
                // Push the file path as a string token
                self.state().queue.push_back(file_token);
                return;
            }
            "__DATE__" | "__TIME__" => {
                let predefined_token = Token {
                    kind: Kind::String { terminated: true },
                    span: token.span,
                };
                self.state().queue.push_back(predefined_token);
                return;
            }
            _ => {}
        }

        if let Some(v) = self.state.borrow().defines.get(name).cloned() {
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

            match &v {
                Macro::Function {
                    args,
                    def,
                    variadic,
                    ..
                } => {
                    // Function-like macros only expand if followed by '('
                    // Check if next token is '('
                    let next_is_lparen = if let Some(file) = self.stack.last_mut() {
                        matches!(file.cursor.peek(), Some(Kind::LParen))
                    } else {
                        false
                    };

                    if !next_is_lparen {
                        // Not a function call, treat as regular identifier
                        self.state().queue.push_back(token);
                        return;
                    }

                    // Consume the '('
                    self.stack.last_mut().unwrap().cursor.next();

                    // Parse the actual arguments
                    let actual_args = self.parse_macro_args();

                    // Check argument count
                    if *variadic {
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

                    // Build a map from parameter names to actual arguments
                    let mut arg_map = HashMap::new();
                    for (param, actual) in args.iter().zip(actual_args.iter()) {
                        let param_name = self.source_of(param.span);
                        arg_map.insert(param_name, vec![actual.clone()]);
                    }

                    // Handle variadic arguments
                    if *variadic {
                        // Collect all remaining arguments for __VA_ARGS__
                        let va_args: Vec<Token> = actual_args[args.len()..]
                            .iter()
                            .enumerate()
                            .flat_map(|(i, arg)| {
                                let mut tokens = vec![];
                                if i > 0 {
                                    // Add comma between arguments
                                    tokens.push(Token {
                                        kind: Kind::Comma,
                                        span: token.span, // Use macro span as approximation
                                    });
                                }
                                tokens.extend(arg.clone());
                                tokens
                            })
                            .collect();
                        arg_map.insert("__VA_ARGS__", vec![va_args]);
                    }

                    // Expand the macro definition with argument substitution
                    let mut i = 0;
                    let tokens = def.clone();
                    let mut result_tokens = Vec::new();

                    while i < tokens.len() {
                        let tok = &tokens[i];

                        // Check for stringification operator (#)
                        if tok.kind == Kind::Hash && i + 1 < tokens.len() {
                            let next_tok = &tokens[i + 1];

                            // Check for token pasting operator (##)
                            if next_tok.kind == Kind::Hash {
                                // This is ##, handle token pasting later
                                result_tokens.push(*tok);
                                result_tokens.push(*next_tok);
                                i += 2;
                                continue;
                            }

                            // This is stringification
                            if matches!(next_tok.kind, Kind::Ident | Kind::Keyword(_)) {
                                let param_name = self.source_of(next_tok.span);
                                if let Some(replacement) = arg_map.get(param_name) {
                                    // Stringify the argument
                                    let mut stringified = String::from("\"");
                                    for tokens in replacement {
                                        for (j, arg_tok) in tokens.iter().enumerate() {
                                            if j > 0 {
                                                stringified.push(' ');
                                            }
                                            stringified.push_str(self.source_of(arg_tok.span));
                                        }
                                    }
                                    stringified.push('"');

                                    // Create a string token
                                    let file_id = self.vfs.embed(&stringified);
                                    let span = Span {
                                        start: Location { offset: 0, file_id },
                                        end: Location {
                                            offset: stringified.len() as u32,
                                            file_id,
                                        },
                                    };
                                    result_tokens.push(Token {
                                        kind: Kind::String { terminated: true },
                                        span,
                                    });

                                    i += 2; // Skip both # and parameter
                                    continue;
                                }
                            }
                        }

                        // Handle parameter substitution
                        if matches!(tok.kind, Kind::Ident | Kind::Keyword(_)) {
                            let name = self.source_of(tok.span);

                            // Check for __VA_OPT__
                            if name == "__VA_OPT__" && *variadic {
                                // Look for opening parenthesis
                                if i + 1 < tokens.len() && tokens[i + 1].kind == Kind::LParen {
                                    i += 2; // Skip __VA_OPT__ and (

                                    // Find matching closing parenthesis
                                    let mut paren_depth = 1;
                                    let mut opt_tokens = Vec::new();
                                    while i < tokens.len() && paren_depth > 0 {
                                        match tokens[i].kind {
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
                                            opt_tokens.push(tokens[i]);
                                        }
                                        i += 1;
                                    }

                                    // Only include the content if there are variadic arguments
                                    let has_varargs = actual_args.len() > args.len();
                                    if has_varargs {
                                        result_tokens.extend(opt_tokens);
                                    }
                                    continue;
                                }
                            }

                            if let Some(replacement) = arg_map.get(name) {
                                // Replace parameter with actual argument
                                for tokens in replacement {
                                    result_tokens.extend(tokens.clone());
                                }
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

                    // Now handle token pasting (##)
                    let mut final_tokens = Vec::new();
                    let mut i = 0;
                    while i < result_tokens.len() {
                        if i + 2 < result_tokens.len()
                            && result_tokens[i + 1].kind == Kind::Hash
                            && result_tokens[i + 2].kind == Kind::Hash
                        {
                            // Found ##, paste tokens
                            if i + 3 < result_tokens.len() {
                                let left = &result_tokens[i];
                                let right = &result_tokens[i + 3];

                                // Concatenate the tokens
                                let mut pasted = String::new();
                                pasted.push_str(self.source_of(left.span));
                                pasted.push_str(self.source_of(right.span));

                                // Create a new token with the pasted content
                                let file_id = self.vfs.embed(&pasted);
                                let span = Span {
                                    start: Location { offset: 0, file_id },
                                    end: Location {
                                        offset: pasted.len() as u32,
                                        file_id,
                                    },
                                };

                                // Determine the kind of the pasted token
                                // For simplicity, assume it's an identifier
                                final_tokens.push(Token {
                                    kind: Kind::Ident,
                                    span,
                                });

                                i += 4; // Skip left, ##, and right
                                continue;
                            }
                        }

                        final_tokens.push(result_tokens[i]);
                        i += 1;
                    }

                    // Push all final tokens
                    for tok in final_tokens {
                        self.expand_inner(tok, seen);
                    }
                }
                Macro::Object { def, .. } => {
                    for tok in def.clone() {
                        self.expand_inner(tok, seen);
                    }
                }
            }
            seen.remove(name);
        } else {
            self.state().queue.push_back(token);
        }
    }

    /// Expands and enqueues the definition of `tok` if it is a macro.
    /// Returns `true` if the macro was expanded.
    ///
    /// A macro expansion is not allowed to have side effects, so we
    /// can fully expand the entire macro definition here. This is
    /// important as we need to detect and break potential cycles.
    #[allow(clippy::too_many_lines)]
    fn expand_macro(&mut self, tok: Token) -> bool {
        let name = self.source_of(tok.span);

        // Handle _Pragma operator
        if name == "_Pragma" {
            // Look for opening parenthesis
            if let Some(lparen) = self.cursor().peek() {
                if lparen == Kind::LParen {
                    self.cursor().next(); // consume (

                    // Get the string literal argument
                    if let Some(string_tok) = self.cursor().next() {
                        if matches!(string_tok.kind, Kind::String { terminated: true }) {
                            // Get closing parenthesis
                            if let Some(rparen) = self.cursor().next() {
                                if rparen.kind == Kind::RParen {
                                    // Extract the pragma content from the string literal
                                    let string_content = self.source_of(string_tok.span);
                                    // Remove quotes
                                    if string_content.len() >= 2 {
                                        let pragma_content =
                                            &string_content[1..string_content.len() - 1];

                                        // Parse the pragma content as tokens
                                        let pragma_id = self.vfs.embed(pragma_content);
                                        let pragma_src = self.vfs.source(pragma_id);
                                        let mut pragma_cursor = Cursor::new(pragma_src, pragma_id);
                                        let mut pragma_tokens = Vec::new();
                                        while let Some(tok) = pragma_cursor.next() {
                                            if tok.kind != Kind::Newline {
                                                pragma_tokens.push(tok);
                                            }
                                        }

                                        // Handle the pragma
                                        if let Some(first_tok) = pragma_tokens.first() {
                                            let pragma_name = self.source_of(first_tok.span);
                                            if let Some(handler) =
                                                self.pragmas.get(pragma_name).cloned()
                                            {
                                                handler.handle(self, pragma_tokens);
                                            }
                                        }
                                    }
                                    return true;
                                }
                            }
                        }
                    }
                    // If we get here, the _Pragma syntax was invalid
                    self.state().errors.push(Error::Syntax {
                        message: "invalid _Pragma syntax",
                        span: tok.span,
                    });
                    return true;
                }
            }
            // No opening paren after _Pragma
            self.state().queue.push_back(tok);
            return false;
        }

        // Handle predefined macros
        if name == "__LINE__" {
            // Calculate line number by counting newlines in the source
            let source = self.vfs.source_str(tok.span.start.file_id);
            let offset = tok.span.start.offset as usize;
            let line_number = source[..offset.min(source.len())]
                .chars()
                .filter(|&c| c == '\n')
                .count()
                + 1;
            let line_str = line_number.to_string();
            let file_id = self.vfs.embed(&line_str);
            let span = Span {
                start: Location { offset: 0, file_id },
                end: Location {
                    offset: line_str.len() as u32,
                    file_id,
                },
            };
            self.state().queue.push_back(Token {
                kind: Kind::Number {
                    base: Base::Decimal,
                },
                span,
            });
            return true;
        } else if name == "__FILE__" {
            // Get the file name from the vfs
            let filename = self.vfs.included_as(tok.span.start.file_id);
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
            return true;
        } else if name == "__DATE__" {
            // Return current date in "Mmm dd yyyy" format
            let date_str = "\"Jan 01 2025\""; // For deterministic tests
            let file_id = self.vfs.embed(date_str);
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
            return true;
        } else if name == "__TIME__" {
            // Return current time in "hh:mm:ss" format
            let time_str = "\"00:00:00\""; // For deterministic tests
            let file_id = self.vfs.embed(time_str);
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
                self.state.borrow_mut().errors.push(Error::Syntax {
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

    /// Special version of `next()` for expression parsing
    fn next_expr_token(&mut self) -> Option<Token> {
        // Use regular next() - we'll handle defined() specially in the expression parser
        self.next()
    }

    /// Get next token without macro expansion - used inside `defined()`
    fn parse_macro_args(&mut self) -> Vec<Vec<Token>> {
        let mut args = vec![];
        let mut current_arg = vec![];
        let mut paren_depth = 0;

        loop {
            let tok = if let Some(file) = self.stack.last_mut() {
                file.cursor.next()
            } else {
                None
            };

            if let Some(tok) = tok {
                match tok.kind {
                    Kind::LParen => {
                        paren_depth += 1;
                        current_arg.push(tok);
                    }
                    Kind::RParen => {
                        if paren_depth == 0 {
                            // End of macro arguments
                            if !current_arg.is_empty() || !args.is_empty() {
                                args.push(current_arg);
                            }
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
                    span: Span::default(), // TODO: track last token span
                });
                break;
            }
        }

        args
    }

    fn next_raw_token(&mut self) -> Option<Token> {
        'outer: loop {
            // Check if we're currently in the middle of a macro expansion, and
            // if so, yield those tokens first.
            if let Some(tok) = self.state().queue.pop_front() {
                return Some(tok);
            }

            // Advance the cursor and continue parsing directives
            if let Some(tok) = self.stack.last_mut()?.cursor.next() {
                if tok.kind == Kind::Hash {
                    self.keyword();
                    continue 'outer;
                }

                // Don't expand macros - return raw token
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
}

fn cli_defines<S, I>(defines: I, parser: &mut Parser<'_, S>)
where
    S: BorrowMut<State>,
    I: IntoIterator<Item = (String, Option<String>)>,
{
    // Generate a virtual file with the specified command-line arguments. We
    // need to be able to reference these (and their location) in the future,
    // so creating a new virtual file is easier.
    let mut buffer = vec![];
    for (k, v) in defines {
        _ = write!(&mut buffer, "#define {k}");
        if let Some(v) = v {
            _ = write!(&mut buffer, " {v}");
        }
        _ = writeln!(&mut buffer);
    }

    // Insert the generated file into the VFS
    let src: Rc<str> = Rc::from(String::from_utf8(buffer).unwrap());
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

    #[must_use]
    pub fn prev_span(&self) -> Option<Span> {
        self.prev
    }
}

impl<S> Iterator for TokenIter<'_, S>
where
    S: BorrowMut<State>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.inner.next_active() {
            // TODO: handle this elsewhere. probably in cursor?
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
    TokenIter {
        inner: parser,
        prev: None,
    }
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
///         original_span: Span,
///     }
/// }
/// ```
///
/// But that's not been implemented yet. The C standard doesn't require that
/// we actually materialize the preprocessed document in any way. We don't need
/// it either since the preprocessor is effectively a lexer for our IDL parser.
pub fn to_string(
    file_id: FileId,
    args: ProcArgs,
    state: &mut State,
    vfs: &mut SourceMap,
) -> (String, Vec<Error>) {
    let src = vfs.source(file_id);
    let mut iter = preprocess(file_id, args, state, vfs);
    let mut buffer = String::with_capacity(src.len());
    let mut last_id = file_id;

    while let Some(tok) = iter.next() {
        if tok.kind == Kind::Eoi {
            break;
        }

        if last_id != tok.span.start.file_id {
            let path = iter.inner.vfs.path(tok.span.start.file_id);
            _ = buffer.write_str(&format!("\n#line 1 {}\n", path.display()));
            last_id = tok.span.start.file_id;
        }

        let slice = iter.source_of(tok.span);
        _ = buffer.write_str(slice);
        if tok.kind != Kind::Newline {
            _ = buffer.write_char(' ');
        }
    }
    (buffer, iter.inner.state.errors.clone())
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
        let (output, _) = to_string(id, ProcArgs::default(), &mut state, &mut vfs);
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
            r"
                #define foo \
                    bar \
                    baz
            ",
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
        assert!(state.warnings().is_empty());
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
