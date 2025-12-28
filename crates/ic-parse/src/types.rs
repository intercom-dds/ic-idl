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

//! Type parsing.

use ic_lexer::token::{Kind, Kw};
use ic_syntax::{Fixed, FixedType, Ident, MapType, Path, SequenceType, StringType, Type};

use super::Parser;
use crate::error::Result;

impl Parser<'_> {
    // Rule 21 with Rule 216 extension
    // <type_spec> ::= <simple_type_spec> | <template_type_spec>
    pub fn type_spec(&mut self) -> Result<Type> {
        match self.peek() {
            // Template types (Rule 38)
            Kind::Keyword(Kw::Sequence) => self.sequence_type(),
            Kind::Keyword(Kw::String) => self.string_type(),
            Kind::Keyword(Kw::WString) => self.wide_string_type(),
            Kind::Keyword(Kw::Map) => self.map_type(),
            Kind::Keyword(Kw::Fixed) => self.fixed_pt_type(),
            // Simple type spec
            _ => self.simple_type_spec(),
        }
    }

    // Rule 22
    // <simple_type_spec> ::= <base_type_spec> | <scoped_name>
    fn simple_type_spec(&mut self) -> Result<Type> {
        match self.peek() {
            // Base types
            Kind::Keyword(Kw::Short | Kw::Long | Kw::Unsigned | Kw::Float | Kw::Double) => {
                self.base_type_spec()
            }
            // Scoped name (user-defined type or built-in like boolean, char, octet)
            Kind::Ident | Kind::DColon => {
                let path = self.scoped_name()?;
                Ok(Type::Path(path))
            }
            _ => Err(self.error_expected("type")),
        }
    }

    // Rule 23
    // <base_type_spec> ::= <floating_pt_type> | <integer_type>
    // Note: char_type, wide_char_type, boolean_type, octet_type are handled via scoped_name
    fn base_type_spec(&mut self) -> Result<Type> {
        match self.peek() {
            Kind::Keyword(Kw::Float | Kw::Double) => self.floating_pt_type(),
            Kind::Keyword(Kw::Long) => {
                // Could be integer (long, long long) or floating (long double)
                // Peek ahead to disambiguate
                if self.peek_nth_raw(1) == Kind::Keyword(Kw::Double) {
                    self.floating_pt_type()
                } else {
                    self.integer_type()
                }
            }
            Kind::Keyword(Kw::Short | Kw::Unsigned) => self.integer_type(),
            _ => Err(self.error_expected("base type")),
        }
    }

    // Rule 24
    // <floating_pt_type> ::= "float" | "double" | "long" "double"
    fn floating_pt_type(&mut self) -> Result<Type> {
        let start = self.span();
        match self.peek() {
            Kind::Keyword(Kw::Float) => {
                self.advance();
                Ok(self.primitive_type("float", self.prev_span))
            }
            Kind::Keyword(Kw::Double) => {
                self.advance();
                Ok(self.primitive_type("double", self.prev_span))
            }
            Kind::Keyword(Kw::Long) => {
                self.advance();
                self.expect_keyword(Kw::Double)?;
                Ok(self.primitive_type("long double", self.make_span(start, self.prev_span)))
            }
            _ => Err(self.error_expected("floating point type")),
        }
    }

    // Rule 25
    // <integer_type> ::= <signed_int> | <unsigned_int>
    fn integer_type(&mut self) -> Result<Type> {
        match self.peek() {
            Kind::Keyword(Kw::Unsigned) => self.unsigned_int(),
            _ => self.signed_int(),
        }
    }

    // Rule 26
    // <signed_int> ::= <signed_short_int> | <signed_long_int> | <signed_longlong_int>
    fn signed_int(&mut self) -> Result<Type> {
        match self.peek() {
            Kind::Keyword(Kw::Short) => self.signed_short_int(),
            Kind::Keyword(Kw::Long) => self.signed_long_int(),
            _ => Err(self.error_expected("signed integer type")),
        }
    }

    // Rule 27
    // <signed_short_int> ::= "short"
    fn signed_short_int(&mut self) -> Result<Type> {
        self.expect_keyword(Kw::Short)?;
        Ok(self.primitive_type("int16", self.prev_span))
    }

    // Rule 28
    // <signed_long_int> ::= "long"
    // Rule 29
    // <signed_longlong_int> ::= "long" "long"
    fn signed_long_int(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Long)?;

        // Check for `long long`
        if self.eat_keyword(Kw::Long).is_some() {
            Ok(self.primitive_type("int64", self.make_span(start, self.prev_span)))
        } else {
            Ok(self.primitive_type("int32", self.prev_span))
        }
    }

    // Rule 30
    // <unsigned_int> ::= <unsigned_short_int> | <unsigned_long_int> | <unsigned_longlong_int>
    fn unsigned_int(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Unsigned)?;

        match self.peek() {
            Kind::Keyword(Kw::Short) => self.unsigned_short_int(start),
            Kind::Keyword(Kw::Long) => self.unsigned_long_int(start),
            _ => Err(self.error_expected("short or long after unsigned")),
        }
    }

    // Rule 31
    // <unsigned_short_int> ::= "unsigned" "short"
    fn unsigned_short_int(&mut self, start: ic_vfs::Span) -> Result<Type> {
        self.expect_keyword(Kw::Short)?;
        Ok(self.primitive_type("uint16", self.make_span(start, self.prev_span)))
    }

    // Rule 32
    // <unsigned_long_int> ::= "unsigned" "long"
    // Rule 33
    // <unsigned_longlong_int> ::= "unsigned" "long" "long"
    fn unsigned_long_int(&mut self, start: ic_vfs::Span) -> Result<Type> {
        self.expect_keyword(Kw::Long)?;

        // Check for `unsigned long long`
        if self.eat_keyword(Kw::Long).is_some() {
            Ok(self.primitive_type("uint64", self.make_span(start, self.prev_span)))
        } else {
            Ok(self.primitive_type("uint32", self.make_span(start, self.prev_span)))
        }
    }

    // Rule 38 with Rule 197 extension
    // <template_type_spec> ::= <sequence_type> | <string_type> | <wide_string_type>
    //                       | <fixed_pt_type> | <map_type>
    #[allow(dead_code)]
    fn template_type_spec(&mut self) -> Result<Type> {
        match self.peek() {
            Kind::Keyword(Kw::Sequence) => self.sequence_type(),
            Kind::Keyword(Kw::String) => self.string_type(),
            Kind::Keyword(Kw::WString) => self.wide_string_type(),
            Kind::Keyword(Kw::Fixed) => self.fixed_pt_type(),
            Kind::Keyword(Kw::Map) => self.map_type(),
            _ => Err(self.error_expected("template type")),
        }
    }

    // Rule 39
    // <sequence_type> ::= "sequence" "<" <type_spec> ">" |
    //                     "sequence" "<" <type_spec> "," <positive_int_const> ">"
    fn sequence_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Sequence)?;

        let (elem, bound, annotations) = self.template_args(|p| {
            let elem = p.type_spec()?;
            let bound = if p.eat(Kind::Comma) {
                Some(p.bound_expr(false)?)
            } else {
                None
            };
            let annotations = p.take_annotations();
            Ok((elem, bound, annotations))
        })?;

        Ok(Type::Sequence(SequenceType {
            ty: Box::new(elem),
            bound,
            span: self.make_span(start, self.prev_span),
            annotations,
        }))
    }

    // Rule 40
    // <string_type> ::= "string" "<" <positive_int_const> ">" | "string"
    fn string_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::String)?;

        let bound = if self.at(Kind::Lt) {
            Some(self.template_args(|p| p.bound_expr(false))?)
        } else {
            None
        };

        Ok(Type::String(StringType {
            wide: false,
            bound,
            span: self.make_span(start, self.prev_span),
        }))
    }

    // Rule 41
    // <wide_string_type> ::= "wstring" "<" <positive_int_const> ">" | "wstring"
    fn wide_string_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::WString)?;

        let bound = if self.at(Kind::Lt) {
            Some(self.template_args(|p| p.bound_expr(false))?)
        } else {
            None
        };

        Ok(Type::String(StringType {
            wide: true,
            bound,
            span: self.make_span(start, self.prev_span),
        }))
    }

    // Rule 42
    // <fixed_pt_type> ::= "fixed" "<" <positive_int_const> "," <positive_int_const> ">"
    fn fixed_pt_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Fixed)?;

        let bounds = if self.at(Kind::Lt) {
            Some(self.template_args(|p| {
                let total = p.bound_expr(true)?;
                p.expect(Kind::Comma)?;
                let fractional = p.bound_expr(false)?;
                Ok(Fixed { total, fractional })
            })?)
        } else {
            None
        };

        Ok(Type::Fixed(FixedType {
            span: self.make_span(start, self.prev_span),
            bounds,
        }))
    }

    // Rule 43
    // <fixed_pt_const_type> ::= "fixed"
    #[allow(dead_code)]
    fn fixed_pt_const_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Fixed)?;
        Ok(Type::Fixed(FixedType {
            span: self.make_span(start, self.prev_span),
            bounds: None,
        }))
    }

    // Rule 199 (DDS-XTypes extension)
    // <map_type> ::= "map" "<" <type_spec> "," <type_spec> ">" |
    //               "map" "<" <type_spec> "," <type_spec> "," <positive_int_const> ">"
    fn map_type(&mut self) -> Result<Type> {
        let start = self.span();
        self.expect_keyword(Kw::Map)?;

        let (key, key_annotations, value, value_annotations, bound) = self.template_args(|p| {
            let key = p.type_spec()?;
            let key_annotations = p.take_annotations();
            p.expect(Kind::Comma)?;
            let value = p.type_spec()?;
            let bound = if p.eat(Kind::Comma) {
                Some(p.bound_expr(false)?)
            } else {
                None
            };
            let value_annotations = p.take_annotations();
            Ok((key, key_annotations, value, value_annotations, bound))
        })?;

        Ok(Type::Map(MapType {
            key: Box::new(key),
            key_annotations,
            value: Box::new(value),
            value_annotations,
            bound,
            span: self.make_span(start, self.prev_span),
        }))
    }

    // Helper: create a primitive type as a Path
    #[allow(clippy::unused_self)]
    fn primitive_type(&self, name: &str, span: ic_vfs::Span) -> Type {
        Type::Path(Path {
            leading_colons: None,
            segments: vec![Ident {
                name: name.to_owned(),
                span,
            }],
        })
    }
}
