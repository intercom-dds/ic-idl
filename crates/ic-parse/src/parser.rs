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

#[cfg(test)]
mod tests;

use anyhow::Result;
use chumsky::prelude::*;
use chumsky::text::{Character, TextParser};
use chumsky::{Error, Parser, Stream};
use ic_alloc::ptr::P;

use crate::lexer::{Kind, Token};
use crate::syntax::{
    ConstDef, DeclKind, Definition, EnumDef, Enumerator, Ident, Item, ItemKind, ModuleDef, Span,
    StructDef, Type, Typedef, UnionDef,
};

// Workaround until trait aliases are stabilized
pub trait IdlParser<T>: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone {}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone> IdlParser<T> for U {}

fn ident() -> impl IdlParser<Kind> {
    just(Kind::Ident).labelled("identifier")
}

fn ty() -> impl IdlParser<Kind> {
    just(Kind::Ident).labelled("type")
}

fn integer_literal() -> impl IdlParser<Kind> {
    one_of([Kind::Octal, Kind::Decimal, Kind::Hex])
}

fn floating_pt_literal() -> impl IdlParser<Kind> {
    just(Kind::Float)
}

fn character_literal() -> impl IdlParser<Kind> {
    one_of([Kind::Octal, Kind::Decimal, Kind::Hex])
}

fn string_literal() -> impl IdlParser<Kind> {
    just(Kind::String)
}

// Rule 1
#[must_use]
pub fn specification() -> impl IdlParser<Vec<Definition>> {
    definition().repeated().then_ignore(end())
}

// Rule 2 with the rule 218 extension
fn definition() -> impl IdlParser<Definition> {
    recursive(|defs| choice((module_dcl(defs), const_dcl(), type_dcl())))
}

// Rule 3
fn module_dcl<'a>(
    state: Recursive<'a, Kind, Definition, Simple<Kind>>,
) -> impl IdlParser<Definition> + 'a {
    let items = state
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let module_def = just(Kind::Module)
        .ignore_then(ident())
        .then(items)
        .labelled("module definition")
        .then_ignore(just(Kind::Semi));

    module_def.map(|v| ModuleDef::new(Ident::default(), v.1))
}

// Rule 4
fn scoped_name() -> impl IdlParser<()> {
    let inner = just([Kind::Colon, Kind::Colon]).then(ident());
    choice((
        ident().then(inner.clone().repeated()).ignored(),
        inner.clone().then(inner.repeated()).ignored(),
    ))
}

// Rule 5
fn const_dcl() -> impl IdlParser<Definition> {
    let def = just(Kind::Const)
        .ignore_then(const_type())
        .then(ident())
        .then_ignore(just(Kind::Eq))
        .then(const_expr())
        .then_ignore(just(Kind::Semi))
        .labelled("const declaration");

    def.map(|_| ConstDef::new(Ident::default(), Type::Path(Ident::default())))
}

// Rule 6
fn const_type() -> impl IdlParser<()> {
    scoped_name().ignored()
    // choice((scoped_name().ignored(), just(Kind::Decimal).ignored()))
    // scoped_name().or(just(Kind::Decimal)).ignored()
}

// Rule 7, 8, 9, 10, 11, 12 and 13
fn const_expr() -> impl IdlParser<()> {
    // The parser is constructed "bottom up" so we start off with the
    // operations that have the lowest precedence.
    recursive(|expr| {
        let val = just(Kind::Decimal).labelled("value");
        let atom = val.or(expr);

        // Multiplication, division and modulus all have the same precedence
        let mult = just(Kind::Star);
        let div = just(Kind::Slash);
        let modulus = just(Kind::Modulo);
        let op = choice((mult, div, modulus));

        let product = atom
            .clone()
            .then(op.then(atom).repeated())
            .foldl(|_, _| Kind::Ident);

        // Addition and subtraction have equal precedence
        let add = just(Kind::Plus);
        let subtract = just(Kind::Minus);
        let op = choice((add, subtract));

        let sum = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(|_, _| Kind::Ident);

        // Bitwise shift operations have equal precedence
        let lshift = just(Kind::LShift);
        let rshift = just(Kind::RShift);
        let op = choice((lshift, rshift));

        let shifted = sum
            .clone()
            .then(op.then(sum).repeated())
            .foldl(|_, _| Kind::Ident);

        shifted
    })
    .ignored()
}

// Rule 14
fn unary_expr() -> impl IdlParser<()> {
    primary_expr()
}

// Rule 15
fn unary_operator() -> impl IdlParser<Kind> {
    one_of([Kind::Minus, Kind::Plus, Kind::Tilde])
}

// Rule 16
fn primary_expr() -> impl IdlParser<()> {
    let nested = const_expr()
        .delimited_by(just(Kind::LParen), just(Kind::RParen))
        .ignored();

    choice((scoped_name().ignored(), literal().ignored(), nested))
}

// Rule 17
fn literal() -> impl IdlParser<Kind> {
    choice((
        integer_literal(),
        floating_pt_literal(),
        character_literal(),
        boolean_literal(),
        string_literal(),
    ))
}

// Rule 18
fn boolean_literal() -> impl IdlParser<Kind> {
    just(Kind::True).or(just(Kind::False))
}

// Rule 19
fn positive_int_const() -> impl IdlParser<()> {
    const_expr().ignored()
}

// Rule 20
fn type_dcl() -> impl IdlParser<Definition> {
    choice((constr_type_dcl(), typedef_dcl(), native_dcl()))
}

// Rule 21
fn type_spec() -> impl IdlParser<()> {
    simple_type_spec().ignored()
}

// Rule 22
fn simple_type_spec() -> impl IdlParser<()> {
    scoped_name()
}

// Rule 23
fn base_type_spec() -> impl IdlParser<()> {
    // We do not treat primitive types as keywords for the sole reason that it
    // serves no purpose other than further complicating the grammar.
    ident().ignored()
}

// Rule 44
fn constr_type_dcl() -> impl IdlParser<Definition> {
    choice((struct_dcl(), union_dcl(), enum_dcl()))
}

// Rule 45
fn struct_dcl() -> impl IdlParser<Definition> {
    choice((struct_forward_dcl(), struct_def()))
}

// Rule 46
fn struct_def() -> impl IdlParser<Definition> {
    let fields = member()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace))
        .labelled("struct member");

    let parent = just(Kind::Colon).ignore_then(scoped_name());

    let struct_def = just(Kind::Struct)
        .ignore_then(ident())
        .then(parent.or_not())
        .then(fields)
        .labelled("struct definition");

    let def = annotation_appl()
        .repeated()
        .then(struct_def)
        .then_ignore(just(Kind::Semi));

    def.map(|_| StructDef::new(Ident::default()))
}

// Rule 47
fn member() -> impl IdlParser<()> {
    type_spec()
        .then(declarators())
        .then_ignore(just(Kind::Semi))
        .labelled("struct member")
        .ignored()
}

// Rule 48
fn struct_forward_dcl() -> impl IdlParser<Definition> {
    let decl = just(Kind::Struct)
        .then(ident())
        .labelled("struct declaration")
        .then_ignore(just(Kind::Semi));

    decl.map(|_| Item::decl(Ident::default(), DeclKind::Struct))
}

// Rule 49
fn union_dcl() -> impl IdlParser<Definition> {
    choice((union_def(), union_forward_dcl()))
}

// Rule 50
fn union_def() -> impl IdlParser<Definition> {
    // `switch(foo)`
    let disc = just(Kind::Switch)
        .ignore_then(switch_type_spec().delimited_by(just(Kind::LParen), just(Kind::RParen)));

    // Case labels + members
    let body = switch_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Union)
        .ignore_then(ident())
        .then(disc)
        .then(body)
        .then_ignore(just(Kind::Semi));

    def.map(|_| UnionDef::new(Ident::default(), vec![]))
}

// Rule 51
fn switch_type_spec() -> impl IdlParser<()> {
    scoped_name()
}

// Rule 52
fn switch_body() -> impl IdlParser<()> {
    case().repeated().ignored()
}

// Rule 53
fn case() -> impl IdlParser<()> {
    // TODO: or should we just accept whatever the validate it later?
    // E.g. multiple values with no case labels.
    case_label()
        .repeated()
        .at_least(1)
        .then(element_spec())
        .then_ignore(just(Kind::Semi))
        .ignored()
}

// Rule 54
fn case_label() -> impl IdlParser<()> {
    let case = just(Kind::Case).ignore_then(just(Kind::Ident));
    let default = just(Kind::Default);

    choice((case, default))
        .then_ignore(just(Kind::Colon))
        .ignored()
        .labelled("case label")
}

// Rule 55
fn element_spec() -> impl IdlParser<()> {
    type_spec().then(declarator()).ignored()
}

// Rule 56
fn union_forward_dcl() -> impl IdlParser<Definition> {
    let decl = just(Kind::Union)
        .then(ident())
        .labelled("union declaration")
        .then_ignore(just(Kind::Semi));

    decl.map(|_| Item::decl(Ident::default(), DeclKind::Union))
}

// Rule 57
fn enum_dcl() -> impl IdlParser<Definition> {
    let enumerators = enumerator()
        .separated_by(just(Kind::Comma))
        .allow_trailing()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Enum)
        .ignore_then(ident())
        .then(enumerators)
        .labelled("enum")
        .then_ignore(just(Kind::Semi));

    def.map(|(_, fields)| EnumDef::new(Ident::default(), fields))
}

// Rule 58
fn enumerator() -> impl IdlParser<Enumerator> {
    // Grammar extension for `MY_ENUMERATOR = 1`
    let value = just(Kind::Eq).ignore_then(const_expr()).or_not();

    ident()
        .then(value)
        .map(|v| Enumerator::new(Span::default()))
        .labelled("enumerator")
}

// Rule 59
fn array_declarator() -> impl IdlParser<()> {
    let bounds = fixed_array_size().repeated().at_least(1);
    ident().then(bounds).ignored()
}

// Rule 60
fn fixed_array_size() -> impl IdlParser<()> {
    just([Kind::LBracket, Kind::Decimal, Kind::RBracket]).ignored()
}

// Rule 61
fn native_dcl() -> impl IdlParser<Definition> {
    just(Kind::Native)
        .ignore_then(simple_declarator())
        .map(|v| Item::decl(Ident::default(), DeclKind::Native))
}

// Rule 62
fn simple_declarator() -> impl IdlParser<()> {
    ident().ignored()
}

// Rule 63
fn typedef_dcl() -> impl IdlParser<Definition> {
    let def = just(Kind::Typedef)
        .ignore_then(type_declarator())
        .then_ignore(just(Kind::Semi));

    def.map(|v| Typedef::new(Ident::default(), Ident::default()))
}

// Rule 64
fn type_declarator() -> impl IdlParser<()> {
    let ty = choice((
        simple_type_spec(),
        // template_type_spec(),
        constr_type_dcl().ignored(),
    ));

    ty.then(any_declarators()).ignored()
}

// Rule 65
fn any_declarators() -> impl IdlParser<()> {
    any_declarator().repeated().at_least(1).ignored()
}

// Rule 66
fn any_declarator() -> impl IdlParser<()> {
    simple_declarator().or(array_declarator())
}

// Rule 67
fn declarators() -> impl IdlParser<()> {
    declarator()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .ignored()
}

// Rule 68
fn declarator() -> impl IdlParser<()> {
    simple_declarator()
}

// Rule 219
fn annotation_dcl() -> impl IdlParser<()> {
    annotation_header()
        .then(annotation_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace)))
        .ignored()
}

// Rule 220
fn annotation_header() -> impl IdlParser<Kind> {
    just(Kind::Annotation).ignore_then(ident())
}

// Rule 221
fn annotation_body() -> impl IdlParser<()> {
    // TODO
    annotation_member()
}

// Rule 222
fn annotation_member() -> impl IdlParser<()> {
    let param = annotation_member_type().then(simple_declarator());
    let body = choice((
        param
            .clone()
            .then_ignore(just(Kind::Default))
            .then(ident())
            .ignored(),
        param.ignored(),
    ));

    // TODO: custom delimited_by-like function for semicolons
    body.then(just(Kind::Semi)).repeated().ignored()
}

// Rule 223
fn annotation_member_type() -> impl IdlParser<()> {
    // `scoped_name` is omitted because it's already included in `const_type`.
    // This is a flaw in the official IDL grammar.
    choice((const_type(), any_const_type()))
}

// Rule 224
fn any_const_type() -> impl IdlParser<()> {
    just(Kind::Any).ignored()
}

// Rule 225
fn annotation_appl() -> impl IdlParser<()> {
    choice((
        just(Kind::AnnotationAppl)
            // TODO: custom function that handles this since it's repeated often
            .then(annotation_appl_params().delimited_by(just(Kind::LParen), just(Kind::RParen)))
            .ignored(),
        just(Kind::AnnotationAppl).ignored(),
    ))
}

// Rule 226
fn annotation_appl_params() -> impl IdlParser<()> {
    annotation_appl_param()
        .separated_by(just(Kind::Comma))
        .allow_trailing()
        .ignored()
}

// Rule 227
fn annotation_appl_param() -> impl IdlParser<()> {
    choice((
        ident().then_ignore(just(Kind::Eq)).then(ident()).ignored(),
        ident().ignored(),
    ))
}
