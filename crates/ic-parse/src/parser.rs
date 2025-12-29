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

use ic_lexer::token::{Kind, Kw, Token};
use ic_syntax::{
    AnnotationAppl, AnnotationArg, Expr, Ident, Item, Literal, LiteralValue, Path, Span,
};
use ic_vfs::SourceMap;

use crate::error::{Expected, ParseError, Result};

/// A checkpoint that can be used to rewind the parser state.
#[derive(Clone)]
pub struct Checkpoint {
    pos: usize,
    prev_span: Span,
    pending_annotations: Vec<AnnotationAppl>,
    annotation_errors_len: usize,
}

/// Maximum nesting depth for recursive constructs (modules, interfaces, etc.).
/// This prevents stack overflow on maliciously nested input.
pub const MAX_DEPTH: usize = 1024;

/// A recursive descent parser for IDL.
pub struct Parser<'a> {
    /// The token stream.
    tokens: Vec<Token>,

    /// Current position in the token stream.
    pos: usize,

    /// Annotations accumulated during trivia skimming.
    pub pending_annotations: Vec<AnnotationAppl>,

    /// Span of the previously consumed token.
    pub prev_span: Span,

    /// Source map for looking up token text.
    source: &'a SourceMap,

    /// Orphaned annotations that couldn't be attached to any construct.
    orphaned_annotations: Vec<AnnotationAppl>,

    /// Errors that occurred while parsing annotations.
    annotation_errors: Vec<ParseError>,

    /// Current recursion depth for nested constructs.
    depth: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from a token stream.
    pub fn new(tokens: Vec<Token>, source: &'a SourceMap) -> Self {
        Self {
            tokens,
            pos: 0,
            pending_annotations: Vec::new(),
            prev_span: Span::default(),
            source,
            orphaned_annotations: Vec::new(),
            annotation_errors: Vec::new(),
            depth: 0,
        }
    }

    /// Creates a checkpoint that can be used to rewind the parser.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            pos: self.pos,
            prev_span: self.prev_span,
            pending_annotations: self.pending_annotations.clone(),
            annotation_errors_len: self.annotation_errors.len(),
        }
    }

    /// Rewinds the parser to a previous checkpoint.
    pub fn rewind(&mut self, checkpoint: Checkpoint) {
        self.pos = checkpoint.pos;
        self.prev_span = checkpoint.prev_span;
        self.pending_annotations = checkpoint.pending_annotations;
        self.annotation_errors
            .truncate(checkpoint.annotation_errors_len);
    }

    /// Executes a closure for lookahead, then rewinds to the original state.
    /// Returns the result of the closure without committing any parser state changes.
    pub fn lookahead<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let checkpoint = self.checkpoint();
        let result = f(self);
        self.rewind(checkpoint);
        result
    }

    /// Returns the kind of the current token without skimming annotations.
    #[inline]
    pub fn peek_raw(&self) -> Kind {
        self.tokens.get(self.pos).map_or(Kind::Eoi, |tok| tok.kind)
    }

    /// Returns the current token without skimming annotations.
    #[inline]
    pub fn current_raw(&self) -> Token {
        self.tokens.get(self.pos).copied().unwrap_or(Token {
            kind: Kind::Eoi,
            span: self.prev_span,
        })
    }

    /// Advances to the next token without skimming annotations.
    /// Returns the consumed token.
    #[inline]
    pub fn advance_raw(&mut self) -> Token {
        let tok = self.current_raw();
        self.prev_span = tok.span;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    /// Checks if the current token matches the given kind (without skimming).
    #[inline]
    pub fn at_raw(&self, kind: Kind) -> bool {
        self.peek_raw() == kind
    }

    /// Returns the kind of the current token, skimming any annotations first.
    #[inline]
    pub fn peek(&mut self) -> Kind {
        self.skim_annotations();
        self.peek_raw()
    }

    /// Advances to the next token, skimming any annotations first.
    #[inline]
    pub fn advance(&mut self) -> Token {
        self.skim_annotations();
        self.advance_raw()
    }

    /// Checks if the current token matches the given kind.
    #[inline]
    pub fn at(&mut self, kind: Kind) -> bool {
        self.peek() == kind
    }

    /// Checks if the current token is a specific keyword.
    #[inline]
    pub fn at_keyword(&mut self, kw: Kw) -> bool {
        self.peek() == Kind::Keyword(kw)
    }

    /// Consumes the current token if it matches the given kind.
    /// Returns `true` if a token was consumed.
    pub fn eat(&mut self, kind: Kind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consumes the current token if it's a specific keyword.
    /// Returns the span if consumed.
    pub fn eat_keyword(&mut self, kw: Kw) -> Option<Span> {
        if self.at_keyword(kw) {
            Some(self.advance().span)
        } else {
            None
        }
    }

    /// Expects the current token to be of the given kind.
    /// Returns an error if it doesn't match.
    /// Also collects orphaned annotations before terminals (`;`, `)`, `}`, `]`, `>`).
    pub fn expect(&mut self, kind: Kind) -> Result<Token> {
        // Collect orphaned annotations before terminals
        if matches!(
            kind,
            Kind::Semi | Kind::RParen | Kind::RBrace | Kind::RBracket | Kind::Gt
        ) {
            self.collect_orphaned_annotations();
        }

        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(self.error_expected(kind))
        }
    }

    /// Expects the current token to be a specific keyword.
    pub fn expect_keyword(&mut self, kw: Kw) -> Result<Token> {
        if self.at_keyword(kw) {
            Ok(self.advance())
        } else {
            Err(self.error_expected(kw))
        }
    }

    /// Expects a semicolon and returns any trailing comments that follow it.
    /// This is used for constructs that end with `;` and should capture trailing doc comments.
    pub fn expect_semi(&mut self) -> Result<Vec<AnnotationAppl>> {
        self.expect(Kind::Semi)?;
        Ok(self.take_trailing_comments())
    }

    /// Skims annotations and comments from the token stream into the pending buffer.
    #[inline]
    pub fn skim_annotations(&mut self) {
        loop {
            match self.peek_raw() {
                Kind::At => match self.annotation_appl() {
                    Ok(ann) => self.pending_annotations.push(ann),
                    Err(e) => {
                        self.annotation_errors.push(e);
                        break;
                    }
                },
                Kind::Comment {
                    terminated: false, ..
                } => {
                    let tok = self.advance_raw();
                    self.annotation_errors.push(
                        self.error_message(tok.span, "unterminated block comment")
                            .with_label("missing closing */"),
                    );
                    break;
                }
                Kind::Comment { .. } => {
                    let ann = self.comment_to_doc_annotation();
                    self.pending_annotations.push(ann);
                }
                _ => break,
            }
        }
    }

    /// Collects and returns any trailing comments at the current position.
    pub fn take_trailing_comments(&mut self) -> Vec<AnnotationAppl> {
        let mut comments = Vec::new();
        while let Kind::Comment {
            trailing: true,
            terminated,
        } = self.peek_raw()
        {
            if terminated {
                comments.push(self.comment_to_doc_annotation());
            } else {
                let tok = self.advance_raw();
                self.annotation_errors.push(
                    self.error_message(tok.span, "unterminated block comment")
                        .with_label("missing closing */"),
                );
            }
        }
        comments
    }

    /// Converts the current comment token to a `@doc` annotation.
    fn comment_to_doc_annotation(&mut self) -> AnnotationAppl {
        let token = self.advance_raw();
        let text = self.clean_comment_text(token.span);

        AnnotationAppl {
            ident: Path {
                leading_colons: None,
                segments: vec![Ident {
                    name: "doc".to_string(),
                    span: token.span,
                }],
            },
            span: token.span,
            args: vec![AnnotationArg {
                ident: None,
                span: token.span,
                value: Expr::Literal(Literal {
                    span: token.span,
                    value: LiteralValue::String(text),
                }),
            }],
        }
    }

    /// Extracts and cleans the text content from a comment.
    fn clean_comment_text(&self, span: Span) -> String {
        let text = self.text(span);
        let trimmed = if text.starts_with("///<") || text.starts_with("//!<") {
            &text[4..]
        } else if text.starts_with("///") || text.starts_with("//!") {
            &text[3..]
        } else if let Some(stripped) = text.strip_prefix("//") {
            stripped
        } else if text.starts_with("/**<") || text.starts_with("/*!<") {
            strip_block_comment(text, 4)
        } else if text.starts_with("/**") || text.starts_with("/*!") {
            strip_block_comment(text, 3)
        } else if text.starts_with("/*") {
            strip_block_comment(text, 2)
        } else {
            text
        };
        trimmed.trim().to_string()
    }

    /// Takes accumulated annotations, clearing the buffer.
    /// This skims any pending annotations/comments first.
    pub fn take_annotations(&mut self) -> Vec<AnnotationAppl> {
        self.skim_annotations();
        std::mem::take(&mut self.pending_annotations)
    }

    /// Returns the span of the current token.
    pub fn span(&self) -> Span {
        self.current_raw().span
    }

    /// Creates a span from a start span to an end span.
    pub fn make_span(&self, start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
        }
    }

    /// Creates a parse error for the current position.
    pub fn error(&self, expected: Vec<Expected>) -> ParseError {
        let tok = self.current_raw();
        let found = if tok.kind == Kind::Eoi {
            None
        } else {
            Some(tok.kind)
        };
        ParseError::new(tok.span, found, expected)
    }

    /// Creates a parse error expecting a single item.
    pub fn error_expected(&self, expected: impl Into<Expected>) -> ParseError {
        self.error(vec![expected.into()])
    }

    /// Creates a parse error with a custom message at a specific span.
    pub fn error_message(&self, span: Span, message: &'static str) -> ParseError {
        ParseError::new(span, None, vec![Expected::Message(message)])
    }

    /// Increments recursion depth and returns an error if max depth exceeded.
    #[inline]
    pub fn enter_nested(&mut self) -> Result<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            Err(self.error_message(self.span(), "maximum nesting depth exceeded"))
        } else {
            Ok(())
        }
    }

    /// Decrements recursion depth.
    #[inline]
    pub fn leave_nested(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Collects any pending annotations that have no construct to attach to.
    pub fn collect_orphaned_annotations(&mut self) {
        self.orphaned_annotations
            .extend(std::mem::take(&mut self.pending_annotations));
    }

    /// Returns the source text for a span.
    pub fn text(&self, span: Span) -> &str {
        let src = self.source.source_str(span.start.file_id);
        &src[span.range()]
    }

    /// Peeks at the nth token ahead (0 = current token) without skimming.
    #[inline]
    pub(super) fn peek_nth_raw(&self, n: usize) -> Kind {
        self.tokens
            .get(self.pos + n)
            .map_or(Kind::Eoi, |tok| tok.kind)
    }

    /// Peeks at the next token after `n` tokens, skipping over annotations.
    /// This is useful for lookahead disambiguation when annotations can appear.
    pub(super) fn peek_nth_skip_annotations(&self, n: usize) -> Kind {
        let mut pos = self.pos;
        let mut count = 0;
        while pos < self.tokens.len() {
            let kind = self.tokens[pos].kind;
            // Skip annotations and comments
            if matches!(kind, Kind::At | Kind::Comment { .. }) {
                // For annotations, we need to skip the entire annotation including args
                if kind == Kind::At {
                    pos += 1;
                    // Skip the annotation name (identifier or keyword)
                    if pos < self.tokens.len() {
                        pos += 1;
                    }
                    // Skip annotation arguments if present: ( ... )
                    if pos < self.tokens.len() && self.tokens[pos].kind == Kind::LParen {
                        let mut depth = 1;
                        pos += 1;
                        while pos < self.tokens.len() && depth > 0 {
                            match self.tokens[pos].kind {
                                Kind::LParen => depth += 1,
                                Kind::RParen => depth -= 1,
                                _ => {}
                            }
                            pos += 1;
                        }
                    }
                } else {
                    pos += 1;
                }
            } else {
                if count == n {
                    return kind;
                }
                count += 1;
                pos += 1;
            }
        }
        Kind::Eoi
    }

    /// Parses an identifier.
    pub fn ident(&mut self) -> Result<Ident> {
        self.skim_annotations();
        let tok = self.current_raw();
        if tok.kind == Kind::Ident {
            self.advance_raw();
            Ok(Ident {
                name: self.text(tok.span).to_owned(),
                span: tok.span,
            })
        } else {
            Err(self.error_expected("identifier"))
        }
    }

    /// Parses an identifier, allowing keywords (for annotation names).
    pub fn ident_or_keyword(&mut self) -> Result<Ident> {
        self.skim_annotations();
        let tok = self.current_raw();
        match tok.kind {
            Kind::Ident | Kind::Keyword(_) => {
                self.advance_raw();
                Ok(Ident {
                    name: self.text(tok.span).to_owned(),
                    span: tok.span,
                })
            }
            _ => Err(self.error_expected("identifier")),
        }
    }

    /// Parses a scoped name (e.g., `Foo`, `::Foo::Bar`).
    pub fn scoped_name(&mut self) -> Result<Path> {
        let leading_colons = if self.eat(Kind::DColon) {
            Some(self.prev_span)
        } else {
            None
        };

        let mut segments = vec![self.ident()?];
        while self.eat(Kind::DColon) {
            segments.push(self.ident()?);
        }

        Ok(Path {
            leading_colons,
            segments,
        })
    }

    /// Parses `{ ... }`, returning content and annotations before/after braces.
    pub fn braced<T>(
        &mut self,
        parse_content: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<(T, Vec<AnnotationAppl>)> {
        let mut annotations = self.take_annotations();
        self.expect(Kind::LBrace)?;
        annotations.extend(self.take_trailing_comments());
        let content = parse_content(self)?;
        self.expect(Kind::RBrace)?;
        annotations.extend(self.take_annotations());
        Ok((content, annotations))
    }

    /// Parses `< ... >` for template arguments, isolating annotations inside from outside.
    pub fn template_args<T>(
        &mut self,
        parse_content: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.expect(Kind::Lt)?;
        let saved = std::mem::take(&mut self.pending_annotations);
        let content = parse_content(self)?;
        self.orphaned_annotations
            .extend(std::mem::take(&mut self.pending_annotations));
        self.expect(Kind::Gt)?;
        self.pending_annotations = saved;
        Ok(content)
    }

    // Rule 1
    // <specification> ::= <definition>+
    /// Parses the entire specification.
    /// Returns the parsed items, errors, and orphaned annotations.
    pub fn parse(mut self) -> (Vec<Item>, Vec<ParseError>, Vec<AnnotationAppl>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        while !self.at(Kind::Eoi) {
            match self.definition() {
                Ok(item) => items.push(item),
                Err(e) => {
                    errors.push(e);
                    self.recover_to_next_definition();
                }
            }
        }

        // Collect any trailing orphaned annotations (e.g., at end of file)
        self.collect_orphaned_annotations();

        // Include any annotation parsing errors
        errors.extend(std::mem::take(&mut self.annotation_errors));

        (items, errors, self.orphaned_annotations)
    }

    /// Attempts to recover from an error by skipping to the next definition.
    ///
    /// Skips tokens until we find:
    /// - A semicolon at brace depth 0 (end of a definition)
    /// - A keyword that starts a definition (module, struct, enum, etc.)
    /// - End of input
    pub fn recover_to_next_definition(&mut self) {
        let mut brace_depth = 0;

        loop {
            match self.peek_raw() {
                Kind::Eoi => break,

                Kind::LBrace => {
                    brace_depth += 1;
                    self.advance_raw();
                }
                Kind::RBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    self.advance_raw();
                    // If we closed back to depth 0, check if next is `;`
                    if brace_depth == 0 && self.peek_raw() == Kind::Semi {
                        self.advance_raw();
                        break;
                    }
                }
                Kind::Semi if brace_depth == 0 => {
                    self.advance_raw();
                    break;
                }

                // Definition-starting keywords at depth 0 - stop before them
                Kind::Keyword(
                    Kw::Module
                    | Kw::Struct
                    | Kw::Union
                    | Kw::Enum
                    | Kw::Const
                    | Kw::Typedef
                    | Kw::Interface
                    | Kw::Valuetype
                    | Kw::Exception
                    | Kw::Native
                    | Kw::Bitmask
                    | Kw::Bitset
                    | Kw::Annotation,
                ) if brace_depth == 0 => break,

                // Skip annotations during recovery
                Kind::At => {
                    self.advance_raw();
                    // Skip the annotation name and any arguments
                    while !matches!(
                        self.peek_raw(),
                        Kind::Eoi | Kind::Semi | Kind::LBrace | Kind::RBrace | Kind::Keyword(_)
                    ) {
                        if self.peek_raw() == Kind::LParen {
                            self.skip_balanced(Kind::LParen, Kind::RParen);
                        } else {
                            self.advance_raw();
                        }
                    }
                }

                _ => {
                    self.advance_raw();
                }
            }
        }

        // Clear any pending annotations accumulated during recovery
        self.pending_annotations.clear();
    }

    /// Skip balanced delimiters (e.g., parentheses).
    pub fn skip_balanced(&mut self, open: Kind, close: Kind) {
        if !self.at_raw(open) {
            return;
        }
        self.advance_raw();

        let mut depth = 1;
        while depth > 0 && !self.at_raw(Kind::Eoi) {
            if self.at_raw(open) {
                depth += 1;
            } else if self.at_raw(close) {
                depth -= 1;
            }
            self.advance_raw();
        }
    }
}

fn strip_block_comment(text: &str, prefix_len: usize) -> &str {
    let end = if text.ends_with("*/") {
        text.len() - 2
    } else {
        text.len()
    };

    if prefix_len <= end {
        &text[prefix_len..end]
    } else {
        ""
    }
}
