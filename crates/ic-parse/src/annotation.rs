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

use ic_lexer::token::{Kind, Kw};
use ic_syntax::{
    Annotation, AnnotationArg, AnnotationDef, AnnotationMember, AnnotationValue, Ident, Item, Meta,
    Path,
};

use super::Parser;
use crate::error::Result;

impl Parser<'_> {
    // Rule 225
    // <annotation_appl> ::= "@" <scoped_name> [ "(" <annotation_appl_params> ")" ]
    //
    /// This method uses raw token access to avoid re-entering annotation skimming.
    #[allow(clippy::result_large_err)]
    pub(super) fn annotation_appl(&mut self) -> Result<Annotation> {
        let start = self.span();

        debug_assert!(self.at_raw(Kind::At));
        self.advance_raw();

        let path = self.parse_annotation_path()?;

        let args = if self.at_raw(Kind::LParen) {
            self.advance_raw();
            let pending_before = self.pending_annotations.len();
            let args = self.annotation_appl_params()?;
            // Check for nested annotations that were skimmed during argument parsing
            if self.pending_annotations.len() > pending_before {
                let nested = &self.pending_annotations[pending_before];
                return Err(self
                    .error_message(nested.span, "nested annotations are not allowed")
                    .with_label("annotation not allowed here"));
            }
            if !self.at_raw(Kind::RParen) {
                return Err(self.error_expected(Kind::RParen));
            }
            self.advance_raw();
            args
        } else {
            Vec::new()
        };

        Ok(Annotation {
            path,
            span: self.make_span(start, self.prev_span),
            arguments: args,
        })
    }

    /// Parses an annotation path, which can include keywords as identifiers.
    ///
    /// For example: `@default`, `@foo::bar`, `@key`
    ///
    /// We only consume `::` if it is immediately adjacent to the previous token
    /// (no whitespace before `::`). This disambiguates:
    /// - `@foo::bar` → annotation with qualified name `foo::bar`
    /// - `@foo:: bar` → annotation with qualified name `foo::bar` (space after `::` is ok)
    /// - `@foo ::bar` → annotation `foo` followed by type `::bar`
    fn parse_annotation_path(&mut self) -> Result<Path> {
        let leading_colons = if self.at_raw(Kind::DColon) && self.is_adjacent() {
            let span = self.span();
            self.advance_raw();
            Some(span)
        } else {
            None
        };

        let first = self.parse_annotation_ident()?;
        let mut segments = vec![first];

        while self.at_raw(Kind::DColon) && self.is_adjacent() {
            self.advance_raw();
            segments.push(self.parse_annotation_ident()?);
        }

        Ok(Path {
            leading_colons,
            segments,
        })
    }

    /// Checks if the current token is immediately adjacent to the previous token.
    fn is_adjacent(&self) -> bool {
        self.prev_span.end.offset == self.span().start.offset
    }

    /// Parses an identifier in annotation context, which allows keywords.
    fn parse_annotation_ident(&mut self) -> Result<Ident> {
        let tok = self.current_raw();
        match tok.kind {
            Kind::Ident => {
                self.advance_raw();
                Ok(Ident {
                    name: self.text(tok.span).to_owned(),
                    span: tok.span,
                })
            }
            Kind::Keyword(kw) => {
                self.advance_raw();
                Ok(Ident {
                    name: kw.to_string(),
                    span: tok.span,
                })
            }
            _ => Err(self.error_expected("identifier")),
        }
    }

    // Rule 226
    // <annotation_appl_params> ::= <const_expr>
    //                            | <annotation_appl_param> { "," <annotation_appl_param> }
    fn annotation_appl_params(&mut self) -> Result<Vec<AnnotationArg>> {
        let mut args = Vec::new();

        if self.at_raw(Kind::RParen) {
            return Ok(args);
        }

        loop {
            let arg = self.annotation_appl_param()?;
            args.push(arg);

            if self.at_raw(Kind::Comma) {
                self.advance_raw();
            } else {
                break;
            }
        }

        Ok(args)
    }

    // Rule 227
    // <annotation_appl_param> ::= <identifier> "=" <const_expr>
    fn annotation_appl_param(&mut self) -> Result<AnnotationArg> {
        let start = self.span();

        if let Some((ident, value)) = self.try_named_arg()? {
            return Ok(AnnotationArg {
                name: Some(ident),
                span: self.make_span(start, self.prev_span),
                value,
            });
        }

        let value = self.const_expr()?;
        Ok(AnnotationArg {
            name: None,
            span: self.make_span(start, self.prev_span),
            value,
        })
    }

    /// Tries to parse a named argument `name = expr`.
    /// Returns None if this doesn't look like a named argument.
    fn try_named_arg(&mut self) -> Result<Option<(Ident, ic_syntax::Expr)>> {
        if !self.at_raw(Kind::Ident) {
            return Ok(None);
        }

        let checkpoint = self.checkpoint();
        let id_tok = self.advance_raw();

        if !self.at_raw(Kind::Eq) {
            self.rewind(checkpoint);
            return Ok(None);
        }

        self.advance_raw();
        let ident = Ident {
            name: self.text(id_tok.span).to_owned(),
            span: id_tok.span,
        };
        let value = self.const_expr()?;

        Ok(Some((ident, value)))
    }

    // Rule 219
    pub(super) fn annotation_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Annotation)?;

        // Rule 220: annotation_header
        let ident = self.ident_or_keyword()?;

        // Rule 221: annotation_body
        let (params, mut annotations) = self.braced(Parser::annotation_body)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::Annotation(AnnotationDef {
            meta: Meta {
                span: self.make_span(start, self.prev_span),
                annotations,
            },
            name: ident,
            members: params,
        }))
    }

    // Rule 221
    fn annotation_body(&mut self) -> Result<Vec<AnnotationMember>> {
        let mut fields = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            let field = match self.peek() {
                Kind::Keyword(Kw::Typedef) => AnnotationMember::Item(self.typedef_dcl()?),
                Kind::Keyword(Kw::Const) => AnnotationMember::Item(self.const_dcl()?),
                Kind::Keyword(Kw::Enum) => AnnotationMember::Item(self.enum_dcl()?),
                Kind::Keyword(Kw::Struct) => AnnotationMember::Item(self.struct_dcl()?),
                Kind::Keyword(Kw::Bitmask) => AnnotationMember::Item(self.bitmask_dcl()?),
                Kind::Keyword(Kw::Union) => AnnotationMember::Item(self.union_dcl()?),
                _ => AnnotationMember::Value(self.annotation_member()?),
            };
            fields.push(field);
        }
        Ok(fields)
    }

    // Rule 222
    fn annotation_member(&mut self) -> Result<AnnotationValue> {
        let start = self.span();
        let mut annotations = self.take_annotations();

        let ty = self.type_spec()?;
        let decl = self.simple_declarator()?;

        let default = if self.eat_keyword(Kw::Default).is_some() {
            Some(self.const_expr()?)
        } else {
            None
        };

        annotations.extend(self.expect_semi()?);

        Ok(AnnotationValue {
            meta: Meta {
                span: self.make_span(start, self.prev_span),
                annotations,
            },
            declarator: decl,
            ty,
            default,
        })
    }
}
