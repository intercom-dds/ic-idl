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

use anyhow::Result;
use chumsky::prelude::*;
use chumsky::text::{Character, TextParser};
use chumsky::{Error, Parser, Stream};
use ic_alloc::ptr::P;
use ic_syntax::{
    AnnotationDef, ConstDef, DeclKind, Definition, EnumDef, Enumerator, ExceptDef, Field, Fixed,
    Ident, Item, ItemKind, Label, ModuleDef, Path, Span, StructDef, Type, Typedef, UnionDef,
    ValuetypeDef,
};

use crate::lexer::{Kind, Token};

// Workaround until trait aliases are stabilized
pub trait IdlParser<T>: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone {}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<Kind, T, Error = Simple<Kind>> + Clone> IdlParser<T> for U {}

fn ident() -> impl IdlParser<Ident> {
    let ident = select! { Kind::Ident(v) => v };
    ident
        .map_with_span(|name, span| Ident { name, span })
        .labelled("identifier")
}

fn ty() -> impl IdlParser<Ident> {
    ident().labelled("type")
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
    just(Kind::StringLit)
}

fn lshift() -> impl IdlParser<[Kind; 2]> {
    just([Kind::Less, Kind::Less]).labelled("<<")
}

fn rshift() -> impl IdlParser<[Kind; 2]> {
    just([Kind::Greater, Kind::Greater]).labelled(">>")
}

// Rule 1
#[must_use]
pub fn specification() -> impl IdlParser<Vec<Definition>> {
    definition().repeated().then_ignore(end())
}

// Rule 2 with the rule 71 and 218 extensions
fn definition() -> impl IdlParser<Definition> {
    // Semicolons are not checked here. This is a PEG parser, so it picks the
    // first rule that matches. To prevent "struct Foo {};" from matching with
    // `struct_forward_dcl`, we need to include terminator in the rule.
    recursive(|defs| {
        let def = choice((
            module_dcl(defs),
            const_dcl(),
            type_dcl(),
            except_dcl(),
            interface_dcl(),
            annotation_dcl(),
            value_dcl(),
        ));

        // If an error occurred, skip all tokens until the next semicolon then
        // continue from there. We won't be able to create a full AST, but we
        // can address other syntax errors we encounter.
        def.recover_with(skip_then_retry_until([Kind::Semi]))
    })
}

// Rule 3
fn module_dcl(
    state: Recursive<'_, Kind, Definition, Simple<Kind>>,
) -> impl IdlParser<Definition> + '_ {
    let items = state
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let module_def = just(Kind::Module)
        .ignore_then(ident())
        .then(items)
        .labelled("module definition")
        .then_ignore(just(Kind::Semi));

    module_def.map_with_span(|v, span| ModuleDef::new(v.0, v.1, span))
}

// Rule 4
// TODO: should probably return Type?
fn scoped_name() -> impl IdlParser<Path> {
    let leading = just(Kind::DColon).map_with_span(|_, span| span).or_not();
    let path = leading.then(ident().separated_by(just(Kind::DColon)).at_least(1));

    path.map(|(leading_colons, segments)| Path {
        leading_colons,
        segments,
    })
}

// Rule 5
fn const_dcl() -> impl IdlParser<Definition> {
    let def = just(Kind::Const)
        .ignore_then(const_type())
        .then(ident())
        .then_ignore(just(Kind::Eq))
        .then(complex_const_expr())
        .then_ignore(just(Kind::Semi))
        .labelled("const declaration");

    def.map_with_span(|((ty, name), _), span| ConstDef::new(name, ty, span))
}

// InterCOM extension for complex constants
fn complex_const_expr() -> impl IdlParser<()> {
    recursive(|state| {
        // let basic = const_expr().ignored();
        let basic = ident().ignored();
        let params = state.separated_by(just(Kind::Comma));
        let complex = just(Kind::LBrace)
            .ignore_then(params)
            .then_ignore(just(Kind::RBrace))
            .ignored();

        choice((basic, complex))
    })
    .ignored()
}

// Rule 6
fn const_type() -> impl IdlParser<Type> {
    choice((template_type_spec(), scoped_name().map(Type::Path)))
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
            .foldl(|_, _| Kind::Decimal);

        // Addition and subtraction have equal precedence
        let add = just(Kind::Plus);
        let subtract = just(Kind::Minus);
        let op = choice((add, subtract));

        let sum = product
            .clone()
            .then(op.then(product).repeated())
            .foldl(|_, _| Kind::Decimal);

        // Bitwise shift operations have equal precedence
        let lshift = lshift();
        let rshift = rshift();
        let op = choice((lshift, rshift));

        sum.clone()
            .then(op.then(sum).repeated())
            .foldl(|_, _| Kind::Decimal)
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
fn type_spec() -> impl IdlParser<Type> {
    choice((simple_type_spec(), template_type_spec()))
}

// Rule 22 + 23
fn simple_type_spec() -> impl IdlParser<Type> {
    choice((base_type_spec(), scoped_name().map(Type::Path)))
}

// Rule 23 with the rule 69 extension
fn base_type_spec() -> impl IdlParser<Type> {
    // Minor deviation: we do not treat primitive types as keywords for the
    // sole reason that it serves no purpose other than further complicating
    // the grammar.
    choice((any_type(), scoped_name().map(Type::Path)))
}

// Rule 38 with the rule 197 extension
fn template_type_spec() -> impl IdlParser<Type> {
    recursive(|state| {
        choice((
            string_type(),
            wide_string_type(),
            fixed_pt_type(),
            sequence_type(state.clone()),
            map_type(state),
        ))
    })
}

// Rule 39
fn sequence_type(state: Recursive<'_, Kind, Type, Simple<Kind>>) -> impl IdlParser<Type> + '_ {
    // let bound = positive_int_const().or_not();
    let inner = state.delimited_by(just(Kind::Less), just(Kind::Greater));
    let seq = just(Kind::Sequence).ignore_then(inner);
    // .then_ignore(just(Kind::Comma))
    // .then(bound)

    seq.map(|elem| Type::Sequence {
        ty: P(elem),
        bound: None,
    })
}

// Rule 40
fn string_type() -> impl IdlParser<Type> {
    // let bound = positive_int_const().or_not();
    // just(Kind::String).then(bound).ignored()
    just(Kind::String).map(|v| Type::String {
        wide: false,
        bound: None,
    })
}

// Rule 41
fn wide_string_type() -> impl IdlParser<Type> {
    // let bound = positive_int_const().or_not();
    // just(Kind::WString).then(bound).ignored()
    just(Kind::WString).map(|v| Type::String {
        wide: true,
        bound: None,
    })
}

// Rule 42
fn fixed_pt_type() -> impl IdlParser<Type> {
    let def = just(Kind::Fixed)
        .then_ignore(just(Kind::Less))
        .ignore_then(positive_int_const())
        .then_ignore(just(Kind::Comma))
        .then(positive_int_const())
        .then_ignore(just(Kind::Greater));

    def.map_with_span(|(tot, frac), span| Type::Fixed {
        span,
        bounds: Some(Fixed {
            total: 0,
            fractional: 0,
        }),
    })
}

// Rule 43
fn fixed_pt_const_type() -> impl IdlParser<Type> {
    just(Kind::Fixed).map_with_span(|_, span| Type::Fixed { span, bounds: None })
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

    def.map_with_span(|v, span| StructDef::new(v.1.0.0, v.1.1, None, span))
}

// Rule 47
fn member() -> impl IdlParser<Field> {
    let field = type_spec()
        .then(declarators())
        .then_ignore(just(Kind::Semi))
        .labelled("struct member");

    field.map(|(ty, names)| Field { names, ty })
}

// Rule 48
fn struct_forward_dcl() -> impl IdlParser<Definition> {
    let decl = just(Kind::Struct)
        .ignore_then(ident())
        .labelled("struct declaration")
        .then_ignore(just(Kind::Semi));

    decl.map_with_span(|name, span| Item::decl(name, DeclKind::Struct, span))
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

    def.map_with_span(|v, span| UnionDef::new(v.0.0, vec![], span))
}

// Rule 51
fn switch_type_spec() -> impl IdlParser<Path> {
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
fn case_label() -> impl IdlParser<Label> {
    let case = just(Kind::Case)
        .ignore_then(scoped_name())
        .map(|ident| Label::Case { ident })
        .labelled("case label");

    let default = just(Kind::Default)
        .map(|_| Label::Default)
        .labelled("default label");

    let null = just(Kind::Null).map(|_| Label::Null).labelled("null label");

    choice((case, default, null))
}

// Rule 55
fn element_spec() -> impl IdlParser<(Type, Ident)> {
    type_spec().then(declarator())
}

// Rule 56
fn union_forward_dcl() -> impl IdlParser<Definition> {
    let decl = just(Kind::Union)
        .ignore_then(ident())
        .labelled("union declaration")
        .then_ignore(just(Kind::Semi));

    decl.map_with_span(|name, span| Item::decl(name, DeclKind::Union, span))
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

    def.map_with_span(|(name, fields), span| EnumDef::new(name, fields, span))
}

// Rule 58
fn enumerator() -> impl IdlParser<Enumerator> {
    // Grammar extension for `MY_ENUMERATOR = 1`
    let value = just(Kind::Eq).ignore_then(const_expr()).or_not();

    ident()
        .then(value)
        .map_with_span(|(name, _), span| Enumerator::new(name, span))
        .labelled("enumerator")
}

// Rule 59
fn array_declarator() -> impl IdlParser<Type> {
    let bounds = fixed_array_size().repeated().at_least(1);
    ident().then(bounds).map(|(name, bound)| Type::Array {
        ty: Path::new(vec![name]),
        bound: vec![],
    })
}

// Rule 60
fn fixed_array_size() -> impl IdlParser<Kind> {
    just([Kind::LBracket, Kind::Decimal, Kind::RBracket]).map(|v| v[1])
}

// Rule 61
fn native_dcl() -> impl IdlParser<Definition> {
    just(Kind::Native)
        .ignore_then(simple_declarator())
        .map_with_span(|name, span| Item::decl(name, DeclKind::Native, span))
}

// Rule 62
fn simple_declarator() -> impl IdlParser<Ident> {
    ident()
}

// Rule 63
fn typedef_dcl() -> impl IdlParser<Definition> {
    let def = just(Kind::Typedef)
        .ignore_then(type_declarator())
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(ty, decls), span| Typedef::new(Ident::default(), ty, span))
}

// Rule 64
fn type_declarator() -> impl IdlParser<(Type, Vec<Type>)> {
    let ty = choice((
        simple_type_spec(),
        template_type_spec(),
        // constr_type_dcl().ignored(),
    ));

    ty.then(any_declarators())
}

// Rule 65
fn any_declarators() -> impl IdlParser<Vec<Type>> {
    any_declarator().repeated().at_least(1)
}

// Rule 66
// no -- this should be the name of the typedef, not the actual type.. so just ident(?)
fn any_declarator() -> impl IdlParser<Type> {
    let decl = simple_declarator().map(|v| {
        Type::Path(Path {
            leading_colons: None,
            segments: vec![v],
        })
    });

    choice((decl, array_declarator()))
}

// Rule 67
fn declarators() -> impl IdlParser<Vec<Ident>> {
    declarator().separated_by(just(Kind::Comma)).at_least(1)
}

// Rule 68
fn declarator() -> impl IdlParser<Ident> {
    simple_declarator()
}

// Rule 70
fn any_type() -> impl IdlParser<Type> {
    just(Kind::Any).map_with_span(|_, span| Type::Any { span })
}

// Rule 72
fn except_dcl() -> impl IdlParser<Definition> {
    let body = member()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    just(Kind::Exception)
        .ignore_then(ident())
        .then(body)
        .map_with_span(|(i, mem), span| ExceptDef::new(i, mem, span))
}

// Rule 73
fn interface_dcl() -> impl IdlParser<Definition> {
    choice((interface_def(), interface_forward_dcl()))
}

// Rule 74
fn interface_def() -> impl IdlParser<Definition> {
    let body = interface_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));
    let def = interface_header().then(body).then_ignore(just(Kind::Semi));
    // TODO
    def.map_with_span(|(_, _), span| ModuleDef::new(Ident::default(), vec![], span))
}

// Rule 75
fn interface_forward_dcl() -> impl IdlParser<Definition> {
    let def = interface_kind()
        .ignore_then(ident())
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|ident, span| Definition::decl(ident, DeclKind::Interface, span))
}

// Rule 76
fn interface_header() -> impl IdlParser<()> {
    interface_kind()
        .then(ident())
        .then(interface_inheritance_spec().or_not())
        .ignored()
}

// Rule 77 with the rule 121 extension
fn interface_kind() -> impl IdlParser<(Option<Kind>, Kind)> {
    // TODO: propagate `local` to def
    just(Kind::Local).or_not().then(just(Kind::Interface))
}

// Rule 78
fn interface_inheritance_spec() -> impl IdlParser<Vec<Path>> {
    just(Kind::Colon).ignore_then(interface_name().separated_by(just(Kind::Comma)))
}

// Rule 79
fn interface_name() -> impl IdlParser<Path> {
    scoped_name()
}

// Rule 80
fn interface_body() -> impl IdlParser<Vec<()>> {
    export().repeated()
}

// Rule 81 with the rule 97 extension
fn export() -> impl IdlParser<()> {
    choice((
        op_dcl(),
        attr_dcl(),
        type_dcl().ignored(),
        const_dcl().ignored(),
        except_dcl().ignored(),
    ))
}

// Rule 82
fn op_dcl() -> impl IdlParser<()> {
    let params = parameter_dcls().delimited_by(just(Kind::LParen), just(Kind::RParen));

    op_type_spec()
        .then(ident())
        .then(params)
        .then(raises_expr().or_not())
        .ignore_then(just(Kind::Semi))
        .ignored()
}

// Rule 83
fn op_type_spec() -> impl IdlParser<Type> {
    // Minor deviation: we do not treat `void` as a keyword, so we only use
    // `type_spec` here.
    type_spec()
}

// Rule 84
fn parameter_dcls() -> impl IdlParser<()> {
    // TODO: make sure this works even with 0 params
    param_dcl().separated_by(just(Kind::Comma)).ignored()
}

// Rule 85
fn param_dcl() -> impl IdlParser<()> {
    param_attribute()
        .then(type_spec())
        .then(simple_declarator())
        .ignored()
}

// Rule 86
fn param_attribute() -> impl IdlParser<Kind> {
    one_of([Kind::In, Kind::Out, Kind::InOut])
}

// Rule 87
fn raises_expr() -> impl IdlParser<()> {
    let exceptions = scoped_name()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .delimited_by(just(Kind::LParen), just(Kind::RParen));

    just(Kind::Raises).then(exceptions).ignored()
}

// Rule 88
fn attr_dcl() -> impl IdlParser<()> {
    choice((readonly_attr_spec(), attr_spec()))
}

// Rule 89
fn readonly_attr_spec() -> impl IdlParser<()> {
    just(Kind::ReadOnly)
        .ignore_then(just(Kind::Attribute))
        .then_ignore(type_spec())
        .then(readonly_attr_declarator())
        .then_ignore(just(Kind::Semi))
        .ignored()
}

// Rule 90
fn readonly_attr_declarator() -> impl IdlParser<()> {
    choice((
        simple_declarator().then(raises_expr()).ignored(),
        simple_declarator()
            .separated_by(just(Kind::Comma))
            .at_least(1)
            .ignored(),
    ))
}

// Rule 91
fn attr_spec() -> impl IdlParser<()> {
    just(Kind::Attribute)
        .ignore_then(type_spec())
        .then(attr_declarator())
        .ignored()
}

// Rule 92
fn attr_declarator() -> impl IdlParser<()> {
    choice((
        simple_declarator().then(attr_raises_expr()).ignored(),
        simple_declarator()
            .separated_by(just(Kind::Comma))
            .at_least(1)
            .ignored(),
    ))
}

// Rule 93
fn attr_raises_expr() -> impl IdlParser<()> {
    choice((
        get_excep_expr().then(set_excep_expr().or_not()).ignored(),
        set_excep_expr(),
    ))
}

// Rule 94
fn get_excep_expr() -> impl IdlParser<()> {
    just(Kind::GetRaises)
        .ignore_then(exception_list())
        .ignored()
}

// Rule 95
fn set_excep_expr() -> impl IdlParser<()> {
    just(Kind::SetRaises)
        .ignore_then(exception_list())
        .ignored()
}

// Rule 96
fn exception_list() -> impl IdlParser<()> {
    scoped_name()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .delimited_by(just(Kind::LParen), just(Kind::RParen))
        .ignored()
}

// Rule 99
fn value_dcl() -> impl IdlParser<Definition> {
    choice((value_forward_dcl(), value_def()))
}

// Rule 100
fn value_def() -> impl IdlParser<Definition> {
    let body = value_element()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = value_header().then(body).then_ignore(just(Kind::Semi));

    def.map_with_span(|((ident, (inherits, supports)), _), span| {
        ValuetypeDef::new(ident, vec![], inherits, supports, span)
    })
}

// Rule 101
fn value_header() -> impl IdlParser<(Ident, (Option<Path>, Option<Path>))> {
    value_kind()
        .ignore_then(ident())
        .then(value_inheritance_spec())
}

// Rule 102
fn value_kind() -> impl IdlParser<Kind> {
    just(Kind::Valuetype)
}

// Rule 103
fn value_inheritance_spec() -> impl IdlParser<(Option<Path>, Option<Path>)> {
    let inherit = just(Kind::Colon).ignore_then(value_name()).or_not();
    let supports = just(Kind::Supports).ignore_then(interface_name()).or_not();
    inherit.then(supports)
}

// Rule 104
fn value_name() -> impl IdlParser<Path> {
    scoped_name()
}

// Rule 105
fn value_element() -> impl IdlParser<()> {
    choice((export(), state_member(), init_dcl()))
}

// Rule 106
fn state_member() -> impl IdlParser<()> {
    one_of([Kind::Public, Kind::Private])
        .then(type_spec())
        .then(declarators())
        .then_ignore(just(Kind::Semi))
        .ignored()
}

// Rule 107
fn init_dcl() -> impl IdlParser<()> {
    let params = init_param_dcls().delimited_by(just(Kind::LParen), just(Kind::RParen));
    let raises = raises_expr().or_not();

    just(Kind::Factory)
        .ignore_then(ident())
        .then(params)
        .then(raises)
        .then_ignore(just(Kind::Semi))
        .ignored()
}

// Rule 108
fn init_param_dcls() -> impl IdlParser<()> {
    init_param_dcl().separated_by(just(Kind::Comma)).ignored()
}

// Rule 109
fn init_param_dcl() -> impl IdlParser<()> {
    just(Kind::In)
        .ignore_then(type_spec())
        .then(simple_declarator())
        .ignored()
}

// Rule 110
fn value_forward_dcl() -> impl IdlParser<Definition> {
    let def = value_kind()
        .ignore_then(ident())
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|name, span| Definition::decl(name, DeclKind::Valuetype, span))
}

// Rule 119
fn op_oneway_dcl() -> impl IdlParser<()> {
    let params = in_parameter_dcls().delimited_by(just(Kind::LParen), just(Kind::RParen));

    just(Kind::Oneway)
        .ignore_then(ty())
        .then(ident())
        .then(params)
        .ignored()
}

// Rule 120
fn in_parameter_dcls() -> impl IdlParser<()> {
    // TODO: remove at_least? might be supported in cidl
    in_param_dcl()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .ignored()
}

// Rule 121
fn in_param_dcl() -> impl IdlParser<()> {
    // TODO: optional in here as well?
    just(Kind::In)
        .then(type_spec())
        .then(simple_declarator())
        .ignored()
}

// Rule 199
fn map_type(state: Recursive<'_, Kind, Type, Simple<Kind>>) -> impl IdlParser<Type> + '_ {
    let key = state.clone();
    let value = state;
    // let bound = positive_int_const().or_not();
    let inner = key.then_ignore(just(Kind::Comma)).then(value);

    let def =
        just(Kind::Map).ignore_then(inner.delimited_by(just(Kind::Less), just(Kind::Greater)));
    // .then_ignore(just(Kind::Comma))
    // .then(bound)

    def.map(|(key, value)| Type::Map {
        key: P(key),
        value: P(value),
        bound: None,
    })
}

// Rule 219
fn annotation_dcl() -> impl IdlParser<Definition> {
    let params = annotation_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));
    let def = annotation_header()
        .then(params)
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(i, params), span| AnnotationDef::new(i, vec![], span))
}

// Rule 220
fn annotation_header() -> impl IdlParser<Ident> {
    just(Kind::Annotation).ignore_then(ident())
}

// Rule 221
fn annotation_body() -> impl IdlParser<Vec<()>> {
    let defs = choice((
        annotation_member(),
        enum_dcl().ignored(),
        const_dcl().ignored(),
        typedef_dcl().ignored(),
    ));
    defs.repeated()
}

// Rule 222
fn annotation_member() -> impl IdlParser<()> {
    let param = annotation_member_type().then(simple_declarator());
    let default = just(Kind::Default).then(ident()).ignored();
    let body = param.then(default.or_not()).ignored();

    // TODO: custom delimited_by-like function for semicolons
    body.then(just(Kind::Semi)).ignored()
}

// Rule 223
fn annotation_member_type() -> impl IdlParser<()> {
    // `scoped_name` is omitted because it's already included in `const_type`
    choice((const_type().ignored(), any_const_type()))
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
