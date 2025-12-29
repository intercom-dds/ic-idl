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
    AliasDef, ArrayDeclarator, Attribute, Bit, Bitfield, BitmaskDef, BitsetDef, ConstDef, Decl,
    DeclKind, Declarator, Discriminator, Empty, EnumDef, Enumerator, ExceptDef, Field,
    InterfaceDef, InterfaceMember, Item, Label, ModuleDef, Param, ParamKind, Prototype, StructDef,
    UnionDef, UnionElement, UnionField, UnionMember, UnionNull, ValueElement, ValueMember,
    ValuetypeDef,
};

use super::Parser;
use crate::error::Result;

impl Parser<'_> {
    // Rule 2
    // <definition> ::= <type_dcl> | <const_dcl> | <except_dcl> | <interface_dcl>
    //                | <module> | <value_dcl> | <annotation_dcl>
    pub fn definition(&mut self) -> Result<Item> {
        match self.peek() {
            Kind::Keyword(Kw::Module) => self.module_dcl(),
            Kind::Keyword(Kw::Struct | Kw::Union | Kw::Enum) => self.constr_type_dcl(),
            Kind::Keyword(Kw::Const) => self.const_dcl(),
            Kind::Keyword(Kw::Typedef) => self.typedef_dcl(),
            Kind::Keyword(Kw::Interface | Kw::Local) => self.interface_dcl(),
            Kind::Keyword(Kw::Valuetype) => self.valuetype_dcl(),
            Kind::Keyword(Kw::Annotation) => self.annotation_dcl(),
            Kind::Keyword(Kw::Exception) => self.except_dcl(),
            Kind::Keyword(Kw::Native) => self.native_dcl(),
            Kind::Keyword(Kw::Bitmask) => self.bitmask_dcl(),
            Kind::Keyword(Kw::Bitset) => self.bitset_dcl(),
            _ => Err(self.error_expected("definition")),
        }
    }

    // Rule 3
    fn module_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Module)?;
        let ident = self.ident()?;

        let (definitions, mut annotations) = self.braced(super::Parser::definitions)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::ModuleValue(ModuleDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            definitions,
        }))
    }

    fn definitions(&mut self) -> Result<Vec<Item>> {
        self.enter_nested()?;
        let mut items = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            items.push(self.definition()?);
        }
        self.leave_nested();
        Ok(items)
    }

    // Rule 4
    fn native_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Native)?;
        let ident = self.ident()?;
        let mut annotations = self.take_annotations();
        annotations.extend(self.expect_semi()?);

        Ok(Item::DeclValue(Decl {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            kind: DeclKind::Native,
        }))
    }

    // Rule 5
    pub(super) fn const_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Const)?;
        let ty = self.type_spec()?;
        let decl = self.declarator()?;
        self.expect(Kind::Eq)?;
        let value = self.const_expr()?;
        let mut annotations = self.take_annotations();
        annotations.extend(self.expect_semi()?);

        Ok(Item::ConstValue(ConstDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            decl,
            ty,
            value,
        }))
    }

    // Rule 44
    // <constr_type_dcl> ::= <struct_dcl> | <union_dcl> | <enum_dcl>
    fn constr_type_dcl(&mut self) -> Result<Item> {
        match self.peek() {
            Kind::Keyword(Kw::Struct) => self.struct_dcl(),
            Kind::Keyword(Kw::Union) => self.union_dcl(),
            Kind::Keyword(Kw::Enum) => self.enum_dcl(),
            _ => Err(self.error_expected("struct, union, or enum")),
        }
    }

    // Rule 45
    pub(super) fn struct_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Struct)?;
        let ident = self.ident()?;

        let mut annotations = self.take_annotations();

        // Rule 48: forward declaration
        if self.eat(Kind::Semi) {
            annotations.extend(self.take_annotations());
            return Ok(Item::DeclValue(Decl {
                span: self.make_span(start, self.prev_span),
                annotations,
                ident,
                kind: DeclKind::Struct,
            }));
        }

        // Rule 46: struct definition
        let parent = if self.eat(Kind::Colon) {
            Some(self.scoped_name()?)
        } else {
            None
        };

        let (members, body_annotations) = self.braced(super::Parser::members)?;
        annotations.extend(body_annotations);
        annotations.extend(self.expect_semi()?);

        Ok(Item::StructValue(StructDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            members,
            parent,
        }))
    }

    // Rule 47
    fn member(&mut self) -> Result<Field> {
        let start = self.span();
        let ty = self.type_spec()?;
        let names = self.declarators()?;
        let mut annotations = self.take_annotations();
        annotations.extend(self.expect_semi()?);

        Ok(Field {
            span: self.make_span(start, self.prev_span),
            annotations,
            names,
            ty,
        })
    }

    fn members(&mut self) -> Result<Vec<Field>> {
        let mut fields = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            fields.push(self.member()?);
        }
        Ok(fields)
    }

    // Rule 49
    pub(super) fn union_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Union)?;
        let ident = self.ident()?;

        let mut annotations = self.take_annotations();

        // Rule 56: forward declaration
        if self.eat(Kind::Semi) {
            annotations.extend(self.take_annotations());
            return Ok(Item::DeclValue(Decl {
                span: self.make_span(start, self.prev_span),
                annotations,
                ident,
                kind: DeclKind::Union,
            }));
        }

        // Rule 50: union definition
        self.expect_keyword(Kw::Switch)?;
        self.expect(Kind::LParen)?;

        // Rule 51: switch_type_spec
        let disc_annotations = self.take_annotations();
        let disc_ty = self.switch_type_spec()?;

        self.expect(Kind::RParen)?;

        // Rule 52: switch_body
        let (fields, body_annotations) = self.braced(super::Parser::switch_body)?;
        annotations.extend(body_annotations);
        annotations.extend(self.expect_semi()?);

        Ok(Item::UnionValue(UnionDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            disc: Discriminator {
                annotations: disc_annotations,
                ty: disc_ty,
            },
            fields,
        }))
    }

    // Rule 51
    fn switch_type_spec(&mut self) -> Result<ic_syntax::Type> {
        // Minor deviation: we don't treat `boolean` or `char` as keywords, so we
        // instead rely on `scoped_name` to resolve to said types. We restrict what
        // types are allowed as a discriminator during linting.
        self.type_spec()
    }

    // Rule 52
    fn switch_body(&mut self) -> Result<Vec<UnionField>> {
        let mut fields = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            fields.push(self.case()?);
        }
        Ok(fields)
    }

    // Rule 53
    fn case(&mut self) -> Result<UnionField> {
        let start = self.span();

        // Rule 54: at least one case label
        let mut labels = vec![self.case_label()?];
        while self.at(Kind::Keyword(Kw::Case)) || self.at(Kind::Keyword(Kw::Default)) {
            labels.push(self.case_label()?);
        }

        // Rule 55: element_spec
        let mut annotations = self.take_annotations();
        let field = self.element_spec()?;
        annotations.extend(self.take_trailing_comments());

        Ok(UnionField {
            span: self.make_span(start, self.prev_span),
            annotations,
            labels,
            field,
        })
    }

    // Rule 54
    fn case_label(&mut self) -> Result<Label> {
        if self.eat_keyword(Kw::Case).is_some() {
            let expr = self.const_expr()?;
            self.expect(Kind::Colon)?;
            Ok(Label::Case(expr))
        } else if self.eat_keyword(Kw::Default).is_some() {
            self.expect(Kind::Colon)?;
            Ok(Label::Default(Empty {}))
        } else {
            Err(self.error_expected("case or default"))
        }
    }

    // Rule 55
    fn element_spec(&mut self) -> Result<UnionElement> {
        // InterCOM extension that lets you define an "empty" member.
        if self.eat_keyword(Kw::Null).is_some() {
            let span = self.prev_span;
            self.expect(Kind::Semi)?;
            return Ok(UnionElement::Null(UnionNull { span }));
        }

        let ty = self.type_spec()?;
        let decl = self.declarator()?;
        self.expect(Kind::Semi)?;

        Ok(UnionElement::Member(UnionMember {
            ty: Box::new(ty),
            decl,
        }))
    }

    // Rule 57
    pub(super) fn enum_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Enum)?;
        let ident = self.ident()?;

        let mut annotations = self.take_annotations();

        let (fields, body_annotations) = self.braced(super::Parser::enumerators)?;
        annotations.extend(body_annotations);
        annotations.extend(self.expect_semi()?);

        Ok(Item::EnumValue(EnumDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            fields,
        }))
    }

    // Rule 58
    fn enumerator(&mut self) -> Result<Enumerator> {
        let mut annotations = self.take_annotations();
        let ident = self.ident()?;

        // Grammar extension: `MY_ENUMERATOR = 1`
        let value = if self.eat(Kind::Eq) {
            Some(self.const_expr()?)
        } else {
            None
        };

        annotations.extend(self.take_annotations());

        Ok(Enumerator {
            ident,
            annotations,
            value,
        })
    }

    fn enumerators(&mut self) -> Result<Vec<Enumerator>> {
        let mut enumerators = Vec::new();
        if !self.at(Kind::RBrace) {
            enumerators.push(self.enumerator()?);
            while self.at(Kind::Comma) {
                let comma_span = self.advance().span;
                if let Some(last) = enumerators.last_mut() {
                    last.annotations.extend(self.take_trailing_comments());
                }
                if self.at(Kind::RBrace) {
                    return Err(self.error_message(comma_span, "trailing comma is not allowed"));
                }
                enumerators.push(self.enumerator()?);
            }
        }
        Ok(enumerators)
    }

    // Rule 59
    // <array_declarator> ::= <identifier> <fixed_array_size>+
    fn array_declarator(&mut self, ident: ic_syntax::Ident) -> Result<Declarator> {
        let mut bounds = Vec::new();
        while self.at(Kind::LBracket) {
            bounds.push(self.fixed_array_size()?);
        }
        Ok(Declarator::Array(ArrayDeclarator { ident, bounds }))
    }

    // Rule 60
    // <fixed_array_size> ::= "[" <positive_int_const> "]"
    fn fixed_array_size(&mut self) -> Result<ic_syntax::Expr> {
        self.expect(Kind::LBracket)?;
        let expr = self.positive_int_const()?;
        self.expect(Kind::RBracket)?;
        Ok(expr)
    }

    // Rule 61
    // <positive_int_const> ::= <const_expr>
    pub(super) fn positive_int_const(&mut self) -> Result<ic_syntax::Expr> {
        self.const_expr()
    }

    // Rule 63
    pub(super) fn typedef_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Typedef)?;
        let ty = self.type_spec()?;
        let decl = self.any_declarators()?;
        let mut annotations = self.take_annotations();
        annotations.extend(self.expect_semi()?);

        Ok(Item::AliasValue(AliasDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            decl,
            ty,
        }))
    }

    // Rule 65
    fn any_declarators(&mut self) -> Result<Vec<Declarator>> {
        self.declarators()
    }

    // Rule 67
    fn declarators(&mut self) -> Result<Vec<Declarator>> {
        let mut decls = vec![self.declarator()?];
        while self.eat(Kind::Comma) {
            decls.push(self.declarator()?);
        }
        Ok(decls)
    }

    // Rule 68
    // <declarator> ::= <simple_declarator> | <array_declarator>
    fn declarator(&mut self) -> Result<Declarator> {
        let ident = self.ident()?;

        if self.at(Kind::LBracket) {
            self.array_declarator(ident)
        } else {
            Ok(Declarator::Simple(ident))
        }
    }

    pub(super) fn simple_declarator(&mut self) -> Result<Declarator> {
        let ident = self.ident()?;
        Ok(Declarator::Simple(ident))
    }

    // Rule 72
    fn except_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Exception)?;
        let ident = self.ident()?;

        let (members, mut annotations) = self.braced(super::Parser::members)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::ExceptionValue(ExceptDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            members,
        }))
    }

    // Rule 73
    // <interface_dcl> ::= <interface_def> | <interface_forward_dcl>
    fn interface_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        let local = self.interface_kind();

        self.expect_keyword(Kw::Interface)?;
        let ident = self.ident()?;

        // Rule 75: forward declaration
        if self.at(Kind::Semi) {
            return self.interface_forward_dcl(start, local, ident);
        }

        // Rule 76: interface definition
        self.interface_def(start, local, ident)
    }

    // Rule 74
    // <interface_forward_dcl> ::= <interface_kind> "interface" <identifier>
    fn interface_forward_dcl(
        &mut self,
        start: ic_vfs::Span,
        _local: Option<ic_vfs::Span>,
        ident: ic_syntax::Ident,
    ) -> Result<Item> {
        let annotations = self.expect_semi()?;
        Ok(Item::DeclValue(Decl {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            kind: DeclKind::Interface,
        }))
    }

    // Rule 76
    // <interface_def> ::= <interface_header> "{" <interface_body> "}"
    fn interface_def(
        &mut self,
        start: ic_vfs::Span,
        local: Option<ic_vfs::Span>,
        ident: ic_syntax::Ident,
    ) -> Result<Item> {
        // Rule 78: interface_inheritance_spec
        let inherits = self.interface_inheritance_spec()?;

        // Rule 80: interface_body
        let (members, mut annotations) = self.braced(super::Parser::interface_body)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::InterfaceValue(InterfaceDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            members,
            inherits,
            local,
        }))
    }

    // Rule 77
    // <interface_kind> ::= "local" | ε
    fn interface_kind(&mut self) -> Option<ic_vfs::Span> {
        if self.eat_keyword(Kw::Local).is_some() {
            Some(self.prev_span)
        } else {
            None
        }
    }

    // Rule 78
    // <interface_inheritance_spec> ::= ":" <interface_name> { "," <interface_name> }
    fn interface_inheritance_spec(&mut self) -> Result<Vec<ic_syntax::Path>> {
        if self.eat(Kind::Colon) {
            self.interface_names()
        } else {
            Ok(Vec::new())
        }
    }

    // Rule 79
    fn interface_names(&mut self) -> Result<Vec<ic_syntax::Path>> {
        let mut names = vec![self.scoped_name()?];
        while self.eat(Kind::Comma) {
            names.push(self.scoped_name()?);
        }
        Ok(names)
    }

    // Rule 80
    fn interface_body(&mut self) -> Result<Vec<InterfaceMember>> {
        let mut members = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            members.push(self.export()?);
        }
        Ok(members)
    }

    // Rule 81 + 97
    // <export> ::= <op_dcl> ";" | <attr_dcl> ";"
    //            | <type_dcl> ";" | <const_dcl> ";" | <except_dcl> ";"
    fn export(&mut self) -> Result<InterfaceMember> {
        match self.peek() {
            // Rule 119: oneway operation
            Kind::Keyword(Kw::Oneway) => {
                let annotations = self.take_annotations();
                let start = self.span();
                self.advance();
                let mut proto = self.op_dcl(start, annotations)?;
                proto.oneway = Some(start);
                Ok(InterfaceMember::Proto(proto))
            }
            // Rule 88-92: attribute
            Kind::Keyword(Kw::ReadOnly | Kw::Attribute) => {
                let annotations = self.take_annotations();
                Ok(InterfaceMember::Attr(self.attr_dcl(annotations)?))
            }
            // Rule 97
            Kind::Keyword(Kw::Typedef) => Ok(InterfaceMember::Item(self.typedef_dcl()?)),
            Kind::Keyword(Kw::Const) => Ok(InterfaceMember::Item(self.const_dcl()?)),
            Kind::Keyword(Kw::Exception) => Ok(InterfaceMember::Item(self.except_dcl()?)),
            Kind::Keyword(Kw::Struct) => Ok(InterfaceMember::Item(self.struct_dcl()?)),
            Kind::Keyword(Kw::Enum) => Ok(InterfaceMember::Item(self.enum_dcl()?)),
            Kind::Keyword(Kw::Union) => Ok(InterfaceMember::Item(self.union_dcl()?)),
            Kind::Keyword(Kw::Bitset) => Ok(InterfaceMember::Item(self.bitset_dcl()?)),
            Kind::Keyword(Kw::Bitmask) => Ok(InterfaceMember::Item(self.bitmask_dcl()?)),
            Kind::Keyword(Kw::Native) => Ok(InterfaceMember::Item(self.native_dcl()?)),
            // Rule 82: operation declaration
            _ => {
                let annotations = self.take_annotations();
                let start = self.span();
                Ok(InterfaceMember::Proto(self.op_dcl(start, annotations)?))
            }
        }
    }

    // Rule 82
    fn op_dcl(
        &mut self,
        start: ic_vfs::Span,
        mut annotations: Vec<ic_syntax::AnnotationAppl>,
    ) -> Result<Prototype> {
        // Rule 83: op_type_spec
        let ret = self.type_spec()?;
        let ident = self.ident()?;

        // Rule 84: parameter_dcls
        self.expect(Kind::LParen)?;
        let params = self.parameter_dcls()?;
        self.expect(Kind::RParen)?;

        // Rule 87: raises_expr
        let raises = if self.eat_keyword(Kw::Raises).is_some() {
            self.exception_list()?
        } else {
            Vec::new()
        };

        self.expect(Kind::Semi)?;
        annotations.extend(self.take_trailing_comments());

        Ok(Prototype {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            ret,
            params,
            raises,
            oneway: None,
        })
    }

    // Rule 84
    fn parameter_dcls(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if !self.at(Kind::RParen) {
            params.push(self.param_dcl()?);
            while self.eat(Kind::Comma) {
                params.push(self.param_dcl()?);
            }
        }
        Ok(params)
    }

    // Rule 85
    fn param_dcl(&mut self) -> Result<Param> {
        // Rule 86: param_attribute
        let kind = match self.peek() {
            Kind::Keyword(Kw::In) => {
                self.advance();
                Some(ParamKind::In)
            }
            Kind::Keyword(Kw::Out) => {
                self.advance();
                Some(ParamKind::Out)
            }
            Kind::Keyword(Kw::InOut) => {
                self.advance();
                Some(ParamKind::Inout)
            }
            _ => None,
        };

        let ty = self.type_spec()?;
        let decl = self.declarator()?;

        Ok(Param { decl, ty, kind })
    }

    // Rule 88
    fn attr_dcl(&mut self, mut annotations: Vec<ic_syntax::AnnotationAppl>) -> Result<Attribute> {
        let start = self.span();

        // Rule 89: readonly_attr_spec
        if self.eat_keyword(Kw::ReadOnly).is_some() {
            let readonly = Some(self.prev_span);
            self.expect_keyword(Kw::Attribute)?;
            annotations.extend(self.take_annotations());
            let ty = self.type_spec()?;

            // Rule 90: readonly_attr_declarator
            let first_decl = self.simple_declarator()?;

            let (decl, getraises) = if self.at(Kind::Keyword(Kw::Raises)) {
                self.advance();
                let raises = self.exception_list()?;
                (vec![first_decl], raises)
            } else {
                let mut decls = vec![first_decl];
                while self.eat(Kind::Comma) {
                    decls.push(self.simple_declarator()?);
                }
                (decls, Vec::new())
            };

            self.expect(Kind::Semi)?;
            annotations.extend(self.take_trailing_comments());

            return Ok(Attribute {
                span: self.make_span(start, self.prev_span),
                annotations,
                decl,
                ty,
                readonly,
                getraises,
                setraises: Vec::new(),
            });
        }

        // Rule 91: attr_spec
        self.expect_keyword(Kw::Attribute)?;
        annotations.extend(self.take_annotations());
        let ty = self.type_spec()?;

        // Rule 92: attr_declarator
        let first_decl = self.simple_declarator()?;

        let (decl, getraises, setraises) =
            if self.at(Kind::Keyword(Kw::GetRaises)) || self.at(Kind::Keyword(Kw::SetRaises)) {
                let (get, set) = self.attr_raises_expr()?;
                (vec![first_decl], get, set)
            } else {
                let mut decls = vec![first_decl];
                while self.eat(Kind::Comma) {
                    decls.push(self.simple_declarator()?);
                }
                (decls, Vec::new(), Vec::new())
            };

        self.expect(Kind::Semi)?;
        annotations.extend(self.take_trailing_comments());

        Ok(Attribute {
            span: self.make_span(start, self.prev_span),
            annotations,
            decl,
            ty,
            readonly: None,
            getraises,
            setraises,
        })
    }

    // Rule 93
    fn attr_raises_expr(&mut self) -> Result<(Vec<ic_syntax::Path>, Vec<ic_syntax::Path>)> {
        let mut getraises = Vec::new();
        let mut setraises = Vec::new();

        // Rule 94: get_excep_expr
        if self.eat_keyword(Kw::GetRaises).is_some() {
            getraises = self.exception_list()?;
        }

        // Rule 95: set_excep_expr
        if self.eat_keyword(Kw::SetRaises).is_some() {
            setraises = self.exception_list()?;
        }

        if getraises.is_empty() && setraises.is_empty() && self.eat_keyword(Kw::SetRaises).is_some()
        {
            setraises = self.exception_list()?;
        }

        Ok((getraises, setraises))
    }

    // Rule 96
    fn exception_list(&mut self) -> Result<Vec<ic_syntax::Path>> {
        self.expect(Kind::LParen)?;
        let mut exceptions = vec![self.scoped_name()?];
        while self.eat(Kind::Comma) {
            exceptions.push(self.scoped_name()?);
        }
        self.expect(Kind::RParen)?;
        Ok(exceptions)
    }

    // Rule 99
    // <value_dcl> ::= <value_def> | <value_forward_dcl>
    fn valuetype_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.value_kind()?;
        let ident = self.ident()?;

        // Rule 110: forward declaration
        if self.at(Kind::Semi) {
            return self.value_forward_dcl(start, ident);
        }

        // Rule 100: value definition
        self.value_def(start, ident)
    }

    // Rule 100
    // <value_def> ::= <value_header> "{" { <value_element> } "}"
    fn value_def(&mut self, start: ic_vfs::Span, ident: ic_syntax::Ident) -> Result<Item> {
        // Rule 103: value_inheritance_spec
        let (inherits, supports) = self.value_inheritance_spec()?;

        let (elements, mut annotations) = self.braced(super::Parser::value_elements)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::ValuetypeValue(ValuetypeDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            elements,
            inherits,
            supports,
        }))
    }

    // Rule 101
    // <value_header> ::= <value_kind> <identifier> [ <value_inheritance_spec> ]

    // Rule 102
    // <value_kind> ::= "valuetype"
    fn value_kind(&mut self) -> Result<()> {
        self.expect_keyword(Kw::Valuetype)?;
        Ok(())
    }

    // Rule 103
    // <value_inheritance_spec> ::= [ ":" <value_name> ] [ "supports" <interface_name> ]
    fn value_inheritance_spec(
        &mut self,
    ) -> Result<(Option<ic_syntax::Path>, Option<ic_syntax::Path>)> {
        let inherits = if self.eat(Kind::Colon) {
            Some(self.value_name()?)
        } else {
            None
        };

        let supports = if self.eat_keyword(Kw::Supports).is_some() {
            Some(self.interface_name()?)
        } else {
            None
        };

        Ok((inherits, supports))
    }

    // Rule 104
    // <value_name> ::= <scoped_name>
    fn value_name(&mut self) -> Result<ic_syntax::Path> {
        self.scoped_name()
    }

    // Rule 105
    fn value_element(&mut self) -> Result<ValueElement> {
        let annotations = self.take_annotations();

        match self.peek() {
            // Rule 106: state_member
            Kind::Keyword(Kw::Public | Kw::Private) => {
                Ok(ValueElement::State(self.state_member()?))
            }
            Kind::Keyword(Kw::Oneway) => {
                let start = self.span();
                self.advance();
                let mut proto = self.op_dcl(start, annotations)?;
                proto.oneway = Some(start);
                Ok(ValueElement::Proto(proto))
            }
            Kind::Keyword(Kw::ReadOnly | Kw::Attribute) => {
                Ok(ValueElement::Attr(self.attr_dcl(annotations)?))
            }
            Kind::Keyword(Kw::Typedef) => Ok(ValueElement::Item(self.typedef_dcl()?)),
            Kind::Keyword(Kw::Const) => Ok(ValueElement::Item(self.const_dcl()?)),
            Kind::Keyword(Kw::Exception) => Ok(ValueElement::Item(self.except_dcl()?)),
            Kind::Keyword(Kw::Struct) => Ok(ValueElement::Item(self.struct_dcl()?)),
            Kind::Keyword(Kw::Enum) => Ok(ValueElement::Item(self.enum_dcl()?)),
            Kind::Keyword(Kw::Union) => Ok(ValueElement::Item(self.union_dcl()?)),
            Kind::Keyword(Kw::Bitset) => Ok(ValueElement::Item(self.bitset_dcl()?)),
            Kind::Keyword(Kw::Bitmask) => Ok(ValueElement::Item(self.bitmask_dcl()?)),
            Kind::Keyword(Kw::Native) => Ok(ValueElement::Item(self.native_dcl()?)),
            _ => {
                let start = self.span();
                Ok(ValueElement::Proto(self.op_dcl(start, annotations)?))
            }
        }
    }

    fn value_elements(&mut self) -> Result<Vec<ValueElement>> {
        let mut elements = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            elements.push(self.value_element()?);
        }
        Ok(elements)
    }

    // Rule 106
    fn state_member(&mut self) -> Result<ValueMember> {
        let visibility_span = self.span();
        let is_public = if self.eat_keyword(Kw::Public).is_some() {
            true
        } else if self.eat_keyword(Kw::Private).is_some() {
            false
        } else {
            return Err(self.error_expected("public or private"));
        };
        let visibility = self.make_span(visibility_span, self.prev_span);

        let ty = self.type_spec()?;
        let decl = self.declarators()?;
        self.expect(Kind::Semi)?;
        let _ = self.take_trailing_comments();

        Ok(ValueMember {
            decl,
            ty,
            visibility,
            is_public,
        })
    }

    // Rule 110
    // <value_forward_dcl> ::= <value_kind> <identifier>
    fn value_forward_dcl(&mut self, start: ic_vfs::Span, ident: ic_syntax::Ident) -> Result<Item> {
        let annotations = self.expect_semi()?;
        Ok(Item::DeclValue(Decl {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            kind: DeclKind::Valuetype,
        }))
    }

    fn interface_name(&mut self) -> Result<ic_syntax::Path> {
        self.scoped_name()
    }

    // Rule 200
    fn bitset_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Bitset)?;
        let ident = self.ident()?;

        let parent = if self.eat(Kind::Colon) {
            Some(self.scoped_name()?)
        } else {
            None
        };

        let (fields, mut annotations) = self.braced(super::Parser::bitfields)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::BitsetValue(BitsetDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            parent,
            fields,
        }))
    }

    // Rule 201
    fn bitfield(&mut self) -> Result<Bitfield> {
        let start = self.span();
        let mut annotations = self.take_annotations();

        // Rule 202: bitfield_spec
        self.expect_keyword(Kw::Bitfield)?;
        let (size, ty) = self.template_args(|p| {
            let size = p.const_expr()?;
            // Rule 203: optional destination_type
            let ty = if p.eat(Kind::Comma) {
                Some(p.type_spec()?)
            } else {
                None
            };
            Ok((size, ty))
        })?;

        let names = if self.at(Kind::Ident) {
            self.declarators()?
        } else {
            Vec::new()
        };

        annotations.extend(self.expect_semi()?);

        Ok(Bitfield {
            span: self.make_span(start, self.prev_span),
            annotations,
            names,
            size,
            ty,
        })
    }

    fn bitfields(&mut self) -> Result<Vec<Bitfield>> {
        let mut fields = Vec::new();
        while !self.at(Kind::RBrace) && !self.at(Kind::Eoi) {
            fields.push(self.bitfield()?);
        }
        Ok(fields)
    }

    // Rule 204
    pub(super) fn bitmask_dcl(&mut self) -> Result<Item> {
        let start = self.span();
        self.expect_keyword(Kw::Bitmask)?;
        let ident = self.ident()?;

        let (bits, mut annotations) = self.braced(super::Parser::bit_values)?;
        annotations.extend(self.expect_semi()?);

        Ok(Item::BitmaskValue(BitmaskDef {
            span: self.make_span(start, self.prev_span),
            annotations,
            ident,
            bits,
        }))
    }

    // Rule 205
    fn bit_value(&mut self) -> Result<Bit> {
        let start = self.span();
        let annotations = self.take_annotations();
        let ident = self.ident()?;

        let value = if self.eat(Kind::Eq) {
            Some(self.const_expr()?)
        } else {
            None
        };

        let mut all_annotations = annotations;
        all_annotations.extend(self.take_annotations());

        Ok(Bit {
            span: self.make_span(start, self.prev_span),
            annotations: all_annotations,
            ident,
            value,
        })
    }

    fn bit_values(&mut self) -> Result<Vec<Bit>> {
        let mut bits = Vec::new();
        if !self.at(Kind::RBrace) {
            bits.push(self.bit_value()?);
            while self.at(Kind::Comma) {
                let comma_span = self.advance().span;
                if let Some(last) = bits.last_mut() {
                    last.annotations.extend(self.take_trailing_comments());
                }
                if self.at(Kind::RBrace) {
                    return Err(self.error_message(comma_span, "trailing comma is not allowed"));
                }
                bits.push(self.bit_value()?);
            }
        }
        Ok(bits)
    }
}
