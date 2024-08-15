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

#![allow(unused)]

use chumsky::prelude::*;
use chumsky::Parser;
use ic_syntax::{
    AnnotationField, AnyType, ArrayDeclarator, Attribute, Binary, Bit, Bitfield, DeclKind,
    Declarator, Discriminator, Empty, Enumerator, Expr, Field, Fixed, FixedType, Ident,
    InterfaceMember, Item, Label, LitKind, Literal, MapType, Op, OpKind, Param, ParamKind, Path,
    Prototype, SequenceType, Span, StringType, Type, Unary, UnionElement, UnionField, UnionMember,
    UnionNull,
};

use crate::lexer::Kind;

pub trait IdlParser<T>: chumsky::Parser<Kind, T, Error = Error> + Clone {
    fn parenthesized(self) -> impl IdlParser<T> {
        self.delimited_by(just(Kind::LParen), just(Kind::RParen))
    }

    fn annotated(self) -> impl IdlParser<T> {
        let ann = annotation_appl();
        let doxy = doxy_comment();
        choice((ann, doxy)).repeated().ignore_then(self)
    }
}

// Blanket impl because we really just want an alias
impl<T, U: chumsky::Parser<Kind, T, Error = Error> + Clone> IdlParser<T> for U {}

pub type Error = Simple<Kind, Span>;

fn ident() -> impl IdlParser<Ident> {
    let ident = select! { Kind::Ident(v) => v };
    ident
        .map_with_span(|name, span| Ident { name, span })
        .labelled("identifier")
}

fn ty() -> impl IdlParser<Ident> {
    ident().labelled("type")
}

fn integer_literal() -> impl IdlParser<Literal> {
    let lit = select! {
        Kind::Octal(v) => v,
        Kind::Decimal(v) => v,
        Kind::Hex(v) => v,
    };
    lit.map_with_span(|v, span| Literal {
        span,
        kind: LitKind::LitInt(v),
    })
}

fn floating_pt_literal() -> impl IdlParser<Literal> {
    let lit = select! { Kind::Float(v) => v };
    lit.map_with_span(|v, span| Literal {
        span,
        kind: LitKind::LitFloat(v),
    })
}

fn character_literal() -> impl IdlParser<Literal> {
    let lit = select! { Kind::Char(v) => v };
    lit.map_with_span(|v, span| Literal {
        span,
        kind: LitKind::LitChar(v.unwrap_or_default()),
    })
}

fn string_literal() -> impl IdlParser<Literal> {
    let lit = select! { Kind::StringLit(v) => v };
    lit.map_with_span(|v, span| Literal {
        span,
        kind: LitKind::LitString(v),
    })
}

fn lshift() -> impl IdlParser<Op> {
    just([Kind::Less, Kind::Less])
        .labelled("<<")
        .map_with_span(|_, span| Op {
            span,
            kind: OpKind::OpLshift,
        })
}

fn rshift() -> impl IdlParser<Op> {
    just([Kind::Greater, Kind::Greater])
        .labelled(">>")
        .map_with_span(|_, span| Op {
            span,
            kind: OpKind::OpRshift,
        })
}

// Handles bounds for collections types
fn bound() -> impl IdlParser<Option<Expr>> {
    just(Kind::Comma).ignore_then(positive_int_const()).or_not()
}

fn doxy_comment() -> impl IdlParser<()> {
    select! { Kind::Comment(v) => v }.ignored()
}

// Rule 1
#[must_use]
pub fn specification() -> impl IdlParser<Vec<Item>> {
    definition().repeated().then_ignore(end())
}

// Rule 2 with the rule 71 and 218 extensions
fn definition() -> impl IdlParser<Item> {
    // Semicolons are not checked here. This is a PEG parser, so it picks the
    // first rule that matches. To prevent "struct Foo {};" from matching with
    // `struct_forward_dcl`, we need to include the terminator in the rule.
    let def = recursive(|defs| {
        choice((
            module_dcl(defs),
            const_dcl(),
            type_dcl(),
            except_dcl(),
            interface_dcl(),
            annotation_dcl(),
            value_dcl(),
        ))
    });

    // If an error occurred, skip all tokens until the next semicolon then
    // continue from there. We won't be able to create a full AST, but we
    // can address other syntax errors we encounter.
    def.recover_with(skip_then_retry_until([Kind::Semi]))
}

// Rule 3
fn module_dcl(state: Recursive<'_, Kind, Item, Error>) -> impl IdlParser<Item> + '_ {
    let items = state
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let module_def = just(Kind::Module)
        .ignore_then(ident())
        .then(items)
        .labelled("module definition")
        .annotated()
        .then_ignore(just(Kind::Semi));

    module_def.map_with_span(|v, span| Item::def_module(v.0, v.1, span))
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

// Similar to scoped_name, but for applied annotations
fn annotation_ident() -> impl IdlParser<Path> {
    let name = select! { Kind::AnnotationAppl(v) => v }
        .map_with_span(|name, span| Ident { name, span })
        .labelled("annotation");

    let path = name.then(just(Kind::DColon).ignore_then(ident()).repeated());

    path.map(|(name, segments)| Path {
        leading_colons: None,
        segments: {
            let mut seg = vec![name];
            seg.extend(segments);
            seg
        },
    })
}

// Rule 5
fn const_dcl() -> impl IdlParser<Item> {
    let def = just(Kind::Const)
        .ignore_then(const_type())
        .then(declarator())
        .then_ignore(just(Kind::Eq))
        .then(complex_const_expr())
        .labelled("const declaration")
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|((ty, name), val), span| {
        let name = match name {
            Declarator::Simple(v) => v,
            Declarator::Array(v) => v.ident,
        };
        Item::def_const(name, ty, val, span)
    })
}

// InterCOM extension for complex constants
fn complex_const_expr() -> impl IdlParser<Expr> {
    recursive(|state| {
        // TODO: can be `{}` too?
        let basic = const_expr();
        let named = ident()
            .ignore_then(just(Kind::Eq))
            .ignore_then(state.clone());

        let complex = state
            .separated_by(just(Kind::Comma))
            .delimited_by(just(Kind::LBrace), just(Kind::RBrace))
            .map(Expr::InitList);

        choice((named, complex, basic))
    })
}

// Rule 6
fn const_type() -> impl IdlParser<Type> {
    choice((
        template_type_spec(),
        scoped_name().map(Type::Path),
        fixed_pt_const_type(),
    ))
}

// Rule 7-14, 16
//
// Due to the recursive nature of expressions, these are implemented in a
// single function to avoid propagating the state everywhere.
fn const_expr() -> impl IdlParser<Expr> {
    recursive(|primary| {
        // Rule 16
        let nested = primary.parenthesized();

        let lit = literal().map(Expr::Literal);
        let scoped = scoped_name().map(Expr::Path);
        let expr = choice((scoped, lit, nested));

        // Rule 14: Unary expressions
        let expr = unary_operator().or_not().then(expr).map(to_unary);

        // Rule 13: Multiplication, division and modulus all have the same precedence
        let expr = binary_op(
            expr,
            choice((
                operator(Kind::Star, OpKind::OpMultiply),
                operator(Kind::Slash, OpKind::OpDivide),
                operator(Kind::Modulo, OpKind::OpModulo),
            )),
        );

        // Rule 12: Addition and subtraction have equal precedence
        let expr = binary_op(
            expr,
            choice((
                operator(Kind::Plus, OpKind::OpAdd),
                operator(Kind::Minus, OpKind::OpSub),
            )),
        );

        // Rule 11: Bitwise shift operations have equal precedence
        let expr = binary_op(expr, choice((lshift(), rshift())));

        // Rule 10: Bitwise AND
        let expr = binary_op(expr, operator(Kind::BitAnd, OpKind::OpAnd));

        // Rule 9: Bitwise XOR
        let expr = binary_op(expr, operator(Kind::BitXor, OpKind::OpXor));

        // Rule 8: Bitwise OR
        binary_op(expr, operator(Kind::BitOr, OpKind::OpOr))
    })
}

fn to_unary((op, expr): (Option<Op>, Expr)) -> Expr {
    if let Some(op) = op {
        Expr::Unary(Box::new(Unary { op, expr }))
    } else {
        expr
    }
}

fn operator(from: Kind, to: OpKind) -> impl IdlParser<Op> {
    just(from).map_with_span(move |_, span| Op { span, kind: to })
}

fn binary_op<'a, T, Oper>(expr: T, op: Oper) -> impl IdlParser<Expr> + 'a
where
    T: IdlParser<Expr> + 'a,
    Oper: IdlParser<Op> + 'a,
{
    let expr = expr.boxed();

    expr.clone()
        .then(op.then(expr).repeated())
        .foldl(|lhs, (op, rhs)| Expr::Binary(Box::new(Binary { lhs, op, rhs })))
        .boxed()
}

// Rule 15
fn unary_operator() -> impl IdlParser<Op> {
    choice((
        operator(Kind::Minus, OpKind::OpSub),
        operator(Kind::Plus, OpKind::OpAdd),
        operator(Kind::Tilde, OpKind::OpNot),
    ))
}

// Rule 17
fn literal() -> impl IdlParser<Literal> {
    choice((
        integer_literal(),
        floating_pt_literal(),
        character_literal(),
        boolean_literal(),
        string_literal(),
    ))
}

// Rule 18
fn boolean_literal() -> impl IdlParser<Literal> {
    let val = choice((just(Kind::True).to(true), just(Kind::False).to(false)));
    val.map_with_span(|v, span| Literal {
        span,
        kind: LitKind::LitBool(v),
    })
}

// Rule 19
fn positive_int_const() -> impl IdlParser<Expr> {
    const_expr()
}

// Rule 20
fn type_dcl() -> impl IdlParser<Item> {
    choice((
        constr_type_dcl(),
        typedef_dcl(),
        native_dcl(),
        bitset_dcl(),
        bitmask_dcl(),
    ))
}

// Rule 21 with the rule 216 extension
fn type_spec() -> impl IdlParser<Type> {
    choice((simple_type_spec(), template_type_spec()))
}

// Rule 22
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

// Not standard, but we need this rule here since `type_spec` is recursive.
// Without it, `sequence_type` will call `type_spec` without propagating the
// recursive state and we'll end up with a stack overflow.
fn sequence_element_type(state: Recursive<'_, Kind, Type, Error>) -> impl IdlParser<Type> + '_ {
    choice((state, simple_type_spec()))
}

// Rule 39
fn sequence_type(state: Recursive<'_, Kind, Type, Error>) -> impl IdlParser<Type> + '_ {
    let inner = sequence_element_type(state)
        .then(bound())
        .delimited_by(just(Kind::Less), just(Kind::Greater));

    let seq = just(Kind::Sequence).ignore_then(inner);
    seq.map_with_span(|(elem, bound), span| {
        Type::Sequence(SequenceType {
            ty: Box::new(elem),
            bound,
            span,
        })
    })
}

// Rule 40
fn string_type() -> impl IdlParser<Type> {
    let bound = positive_int_const()
        .delimited_by(just(Kind::Less), just(Kind::Greater))
        .or_not();

    just(Kind::String)
        .ignore_then(bound)
        .map_with_span(|bound, span| {
            Type::String_(StringType {
                wide: false,
                bound,
                span,
            })
        })
}

// Rule 41
fn wide_string_type() -> impl IdlParser<Type> {
    let bound = positive_int_const()
        .delimited_by(just(Kind::Less), just(Kind::Greater))
        .or_not();

    just(Kind::WString)
        .ignore_then(bound)
        .map_with_span(|bound, span| {
            Type::String_(StringType {
                wide: true,
                bound,
                span,
            })
        })
}

// Rule 42
fn fixed_pt_type() -> impl IdlParser<Type> {
    let body = positive_int_const()
        .then_ignore(just(Kind::Comma))
        .then(positive_int_const())
        .delimited_by(just(Kind::Less), just(Kind::Greater));

    just(Kind::Fixed)
        .ignore_then(body)
        .map_with_span(|(tot, frac), span| {
            Type::Fixed(FixedType {
                span,
                bounds: Some(Fixed {
                    total: tot,
                    fractional: frac,
                }),
            })
        })
}

// Rule 43
fn fixed_pt_const_type() -> impl IdlParser<Type> {
    just(Kind::Fixed).map_with_span(|_, span| Type::Fixed(FixedType { span, bounds: None }))
}

// Rule 44
fn constr_type_dcl() -> impl IdlParser<Item> {
    choice((struct_dcl(), union_dcl(), enum_dcl()))
}

// Rule 45
fn struct_dcl() -> impl IdlParser<Item> {
    choice((struct_forward_dcl(), struct_def()))
}

// Rule 46
fn struct_def() -> impl IdlParser<Item> {
    let fields = member()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace))
        .labelled("struct member");

    let parent = just(Kind::Colon).ignore_then(scoped_name());

    let struct_def = just(Kind::Struct)
        .ignore_then(ident())
        .then(parent.or_not())
        .then(fields)
        .labelled("struct definition")
        .annotated()
        .then_ignore(just(Kind::Semi));

    struct_def.map_with_span(|v, span| Item::def_struct(v.0.0, v.1, None, span))
}

// Rule 47
fn member() -> impl IdlParser<Field> {
    let field = type_spec()
        .then(declarators())
        .annotated()
        .then_ignore(just(Kind::Semi))
        .labelled("member");

    field.map(|(ty, names)| Field { names, ty })
}

// Rule 48
fn struct_forward_dcl() -> impl IdlParser<Item> {
    let decl = just(Kind::Struct)
        .ignore_then(ident())
        .labelled("struct declaration")
        .annotated()
        .then_ignore(just(Kind::Semi));

    decl.map_with_span(|name, span| {
        Item::decl(Declarator::Simple(name), DeclKind::DeclStruct, span)
    })
}

// Rule 49
fn union_dcl() -> impl IdlParser<Item> {
    choice((union_def(), union_forward_dcl()))
}

// Rule 50
fn union_def() -> impl IdlParser<Item> {
    // `switch(foo)`
    let disc = just(Kind::Switch)
        .ignore_then(switch_type_spec().parenthesized())
        .map(|path| Discriminator {
            annotations: vec![],
            ty: Type::Path(path),
        });

    // Case labels + members
    let body = switch_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Union)
        .ignore_then(ident())
        .then(disc)
        .then(body)
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|((ident, disc), cases), span| Item::def_union(ident, disc, cases, span))
}

// Rule 51
fn switch_type_spec() -> impl IdlParser<Path> {
    scoped_name()
}

// Rule 52
fn switch_body() -> impl IdlParser<Vec<UnionField>> {
    case().repeated()
}

// Rule 53
fn case() -> impl IdlParser<UnionField> {
    let def = case_label().repeated().at_least(1).then(element_spec());

    def.map(|(labels, field)| UnionField {
        annotations: vec![],
        labels,
        field,
    })
}

// Rule 54
fn case_label() -> impl IdlParser<Label> {
    let case = just(Kind::Case)
        .ignore_then(const_expr())
        .map(Label::Case)
        .labelled("case label")
        .then_ignore(just(Kind::Colon));

    let default = just(Kind::Default)
        .map(|_| Label::Default(Empty {}))
        .labelled("default label")
        .then_ignore(just(Kind::Colon));

    choice((case, default))
}

// Rule 55
fn element_spec() -> impl IdlParser<UnionElement> {
    // InterCOM extension that lets you define an "empty" member.
    let null = just(Kind::Null)
        .then_ignore(just(Kind::Semi))
        .map_with_span(|_, span| UnionElement::Null(UnionNull { span }));

    let ty = type_spec()
        .then(declarator())
        .annotated()
        .then_ignore(just(Kind::Semi))
        .map(|(ty, decl)| {
            UnionElement::Member(UnionMember {
                ty: Box::new(ty),
                decl,
            })
        });

    choice((ty, null))
}

// Rule 56
fn union_forward_dcl() -> impl IdlParser<Item> {
    let decl = just(Kind::Union)
        .ignore_then(ident())
        .labelled("union declaration")
        .annotated()
        .then_ignore(just(Kind::Semi));

    decl.map_with_span(|name, span| Item::decl(Declarator::Simple(name), DeclKind::DeclUnion, span))
}

// Rule 57
fn enum_dcl() -> impl IdlParser<Item> {
    let enumerators = enumerator()
        .annotated()
        .separated_by(just(Kind::Comma))
        .allow_trailing()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Enum)
        .ignore_then(ident())
        .then(enumerators)
        .labelled("enum")
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(name, fields), span| Item::def_enum(name, fields, span))
}

// Rule 58
fn enumerator() -> impl IdlParser<Enumerator> {
    // Grammar extension for `MY_ENUMERATOR = 1`
    let value = just(Kind::Eq).ignore_then(const_expr()).or_not();
    let def = ident().then(value).labelled("enumerator");

    def.map(|(name, value)| Enumerator {
        name,
        value,
        annotations: vec![],
    })
}

// Rule 59
fn array_declarator() -> impl IdlParser<Declarator> {
    let bounds = fixed_array_size().repeated().at_least(1);
    ident()
        .then(bounds)
        .map(|(ident, bounds)| Declarator::Array(ArrayDeclarator { ident, bounds }))
}

// Rule 60
fn fixed_array_size() -> impl IdlParser<Expr> {
    positive_int_const().delimited_by(just(Kind::LBracket), just(Kind::RBracket))
}

// Rule 61
fn native_dcl() -> impl IdlParser<Item> {
    just(Kind::Native)
        .ignore_then(simple_declarator())
        .map_with_span(|name, span| Item::decl(name, DeclKind::DeclNative, span))
}

// Rule 62
// TODO: should this return Ident instead?
fn simple_declarator() -> impl IdlParser<Declarator> {
    ident().map(Declarator::Simple)
}

// Rule 63
fn typedef_dcl() -> impl IdlParser<Item> {
    let def = just(Kind::Typedef)
        .ignore_then(type_declarator())
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(ty, decls), span| {
        Item::typedef(
            Ident {
                // TODO: fix this
                name: ic_syntax::util::decl_name(&decls[0]).to_string(),
                span: Span::default(),
            },
            ty,
            span,
        )
    })
}

// Rule 64
fn type_declarator() -> impl IdlParser<(Type, Vec<Declarator>)> {
    // `constr_type_dcl` is deliberately omitted as anonymous structs, unions,
    // enums and bitmasks are not supported.
    let ty = choice((simple_type_spec(), template_type_spec()));
    ty.then(any_declarators())
}

// Rule 65
fn any_declarators() -> impl IdlParser<Vec<Declarator>> {
    any_declarator().repeated().at_least(1)
}

// Rule 66
fn any_declarator() -> impl IdlParser<Declarator> {
    choice((array_declarator(), simple_declarator()))
}

// Rule 67
fn declarators() -> impl IdlParser<Vec<Declarator>> {
    declarator().separated_by(just(Kind::Comma)).at_least(1)
}

// Rule 68 with the rule 217 extension
fn declarator() -> impl IdlParser<Declarator> {
    choice((array_declarator(), simple_declarator()))
}

// Rule 70
fn any_type() -> impl IdlParser<Type> {
    just(Kind::Any).map_with_span(|_, span| Type::Any(AnyType { span }))
}

// Rule 72
fn except_dcl() -> impl IdlParser<Item> {
    let body = member()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    just(Kind::Exception)
        .ignore_then(ident())
        .then(body)
        .map_with_span(|(i, mem), span| Item::def_exception(i, mem, span))
}

// Rule 73
fn interface_dcl() -> impl IdlParser<Item> {
    choice((interface_def(), interface_forward_dcl()))
}

// Rule 74
fn interface_def() -> impl IdlParser<Item> {
    let body = interface_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));
    let def = interface_header()
        .then(body)
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(((local, name), inherits), _), span| {
        Item::interface(name, local, inherits, vec![], span)
    })
}

// Rule 75
fn interface_forward_dcl() -> impl IdlParser<Item> {
    let def = interface_kind()
        .ignore_then(ident())
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|ident, span| {
        Item::decl(Declarator::Simple(ident), DeclKind::DeclInterface, span)
    })
}

// Rule 76
fn interface_header() -> impl IdlParser<((Option<Span>, Ident), Vec<Path>)> {
    interface_kind()
        .then(ident())
        .then(interface_inheritance_spec())
}

// Rule 77 with the rule 121 extension
fn interface_kind() -> impl IdlParser<Option<Span>> {
    just(Kind::Local)
        .map_with_span(|_, span| span)
        .or_not()
        .then_ignore(just(Kind::Interface))
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
fn interface_body() -> impl IdlParser<Vec<InterfaceMember>> {
    export().repeated()
}

// Rule 81 with the rule 97 extension
fn export() -> impl IdlParser<InterfaceMember> {
    choice((
        op_dcl().map(InterfaceMember::Proto),
        attr_dcl().map(InterfaceMember::Attr),
        type_dcl().map(InterfaceMember::Item),
        const_dcl().map(InterfaceMember::Item),
        except_dcl().map(InterfaceMember::Item),
        op_oneway_dcl().map(InterfaceMember::Proto),
    ))
}

// Rule 82
fn op_dcl() -> impl IdlParser<Prototype> {
    let params = parameter_dcls().parenthesized();
    let proto = op_type_spec()
        .then(ident())
        .then(params)
        .then(raises_expr().or_not())
        .then_ignore(just(Kind::Semi));

    proto.map(|(((_ret, name), params), raises)| Prototype {
        name,
        params,
        raises: raises.unwrap_or_default(),
        oneway: None,
    })
}

// Rule 83
fn op_type_spec() -> impl IdlParser<Type> {
    // Minor deviation: we do not treat `void` as a keyword, so we only use
    // `type_spec` here.
    type_spec()
}

// Rule 84
fn parameter_dcls() -> impl IdlParser<Vec<Param>> {
    param_dcl().separated_by(just(Kind::Comma))
}

// Rule 85
fn param_dcl() -> impl IdlParser<Param> {
    let param = param_attribute()
        .or_not()
        .then(type_spec())
        .then(simple_declarator());

    param.map(|((kind, ty), decl)| Param {
        name: match decl {
            Declarator::Simple(v) => v,
            Declarator::Array(_) => todo!(),
        },
        ty,
        kind,
    })
}

// Rule 86
fn param_attribute() -> impl IdlParser<ParamKind> {
    choice((
        just(Kind::In).to(ParamKind::ParamIn),
        just(Kind::Out).to(ParamKind::ParamOut),
        just(Kind::InOut).to(ParamKind::ParamInout),
    ))
}

// Rule 87
fn raises_expr() -> impl IdlParser<Vec<Path>> {
    let exceptions = scoped_name()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .parenthesized();

    just(Kind::Raises).ignore_then(exceptions)
}

// Rule 88
fn attr_dcl() -> impl IdlParser<Attribute> {
    choice((readonly_attr_spec(), attr_spec()))
}

// Rule 89
fn readonly_attr_spec() -> impl IdlParser<Attribute> {
    let def = just(Kind::ReadOnly)
        .map_with_span(|_, span| span)
        .then_ignore(just(Kind::Attribute))
        .then(type_spec())
        .then(readonly_attr_declarator())
        .then_ignore(just(Kind::Semi));

    def.map(|((readonly, ty), _)| Attribute {
        name: todo!(),
        ty,
        readonly: Some(readonly),
    })
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
fn attr_spec() -> impl IdlParser<Attribute> {
    let def = just(Kind::Attribute)
        .ignore_then(type_spec())
        .then(attr_declarator());

    def.map(|(ty, _)| Attribute {
        name: Ident::default(),
        ty,
        readonly: None,
    })
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
        set_excep_expr().ignored(),
    ))
}

// Rule 94
fn get_excep_expr() -> impl IdlParser<Vec<Path>> {
    just(Kind::GetRaises).ignore_then(exception_list())
}

// Rule 95
fn set_excep_expr() -> impl IdlParser<Vec<Path>> {
    just(Kind::SetRaises).ignore_then(exception_list())
}

// Rule 96
fn exception_list() -> impl IdlParser<Vec<Path>> {
    scoped_name()
        .separated_by(just(Kind::Comma))
        .at_least(1)
        .parenthesized()
}

// Rule 99
fn value_dcl() -> impl IdlParser<Item> {
    choice((value_forward_dcl(), value_def()))
}

// Rule 100
fn value_def() -> impl IdlParser<Item> {
    let body = value_element()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = value_header().then(body).then_ignore(just(Kind::Semi));

    def.map_with_span(|((ident, (inherits, supports)), _), span| {
        Item::valuetype(ident, vec![], inherits, supports, span)
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
    choice((export().ignored(), state_member(), init_dcl()))
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
    let params = init_param_dcls().parenthesized();
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
fn value_forward_dcl() -> impl IdlParser<Item> {
    let def = value_kind()
        .ignore_then(ident())
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|name, span| {
        Item::decl(Declarator::Simple(name), DeclKind::DeclValuetype, span)
    })
}

// Rule 119
fn op_oneway_dcl() -> impl IdlParser<Prototype> {
    let params = in_parameter_dcls().parenthesized();
    let def = just(Kind::Oneway)
        .map_with_span(|_, span| span)
        .then(ty())
        .then(ident())
        .then(params);

    def.map(|(((oneway, _ty), name), _params)| Prototype {
        name,
        params: vec![],
        raises: vec![],
        oneway: Some(oneway),
    })
}

// Rule 120
fn in_parameter_dcls() -> impl IdlParser<()> {
    in_param_dcl().separated_by(just(Kind::Comma)).ignored()
}

// Rule 121
fn in_param_dcl() -> impl IdlParser<()> {
    just(Kind::In)
        .then(type_spec())
        .then(simple_declarator())
        .ignored()
}

// Rule 199
fn map_type(state: Recursive<'_, Kind, Type, Error>) -> impl IdlParser<Type> + '_ {
    let key = map_type_spec(state.clone());
    let value = map_type_spec(state);
    let inner = key.then_ignore(just(Kind::Comma)).then(value).then(bound());
    let def =
        just(Kind::Map).ignore_then(inner.delimited_by(just(Kind::Less), just(Kind::Greater)));

    def.map_with_span(|((key, value), bound), span| {
        Type::Map(MapType {
            key: Box::new(key),
            value: Box::new(value),
            bound,
            span,
        })
    })
}

// Types that can appear in maps as either the key or element type.
// We can't use `type_spec` directly because we need to propagate the state.
fn map_type_spec(state: Recursive<'_, Kind, Type, Error>) -> impl IdlParser<Type> + '_ {
    choice((state, simple_type_spec()))
}

// Rule 200
fn bitset_dcl() -> impl IdlParser<Item> {
    let inherit = just(Kind::Colon).ignore_then(value_name()).or_not();

    let body = bitfield()
        .repeated()
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Bitset)
        .ignore_then(ident())
        .then(inherit)
        .then(body)
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|((ident, parent), fields), span| Item::bitset(ident, parent, fields, span))
}

// Rule 201
fn bitfield() -> impl IdlParser<Bitfield> {
    let def = bitfield_spec().then(ident()).then_ignore(just(Kind::Semi));
    def.map_with_span(|((size, _ty), name), span| Bitfield {
        annotations: vec![],
        name,
        size,
        span,
    })
}

// Rule 202
fn bitfield_spec() -> impl IdlParser<(Expr, Option<Ident>)> {
    just(Kind::Bitfield).ignore_then(
        positive_int_const()
            .then(just(Kind::Comma).ignore_then(destination_type()).or_not())
            .delimited_by(just(Kind::Less), just(Kind::Greater)),
    )
}

// Rule 203
fn destination_type() -> impl IdlParser<Ident> {
    // TOOD: primitive types only
    ident()
}

// Rule 204
fn bitmask_dcl() -> impl IdlParser<Item> {
    let body = bit_value()
        .separated_by(just(Kind::Comma))
        .delimited_by(just(Kind::LBrace), just(Kind::RBrace));

    let def = just(Kind::Bitmask)
        .ignore_then(ident())
        .then(body)
        .annotated()
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(name, flags), span| Item::bitmask(name, flags, span))
}

// Rule 205
fn bit_value() -> impl IdlParser<Bit> {
    let def = ident().then(just(Kind::Eq).ignore_then(const_expr()).or_not());
    def.map_with_span(|(name, value), span| Bit {
        name,
        annotations: vec![],
        value,
        span,
    })
}

// Rule 219
fn annotation_dcl() -> impl IdlParser<Item> {
    let params = annotation_body().delimited_by(just(Kind::LBrace), just(Kind::RBrace));
    let def = annotation_header()
        .then(params)
        .then_ignore(just(Kind::Semi));

    def.map_with_span(|(i, _), span| Item::def_annotation(i, vec![], span))
}

// Rule 220
fn annotation_header() -> impl IdlParser<Ident> {
    just(Kind::Annotation).ignore_then(ident())
}

// Rule 221
fn annotation_body() -> impl IdlParser<Vec<AnnotationField>> {
    choice((
        annotation_member().map(|v| AnnotationField::Arg(Box::new(v))),
        // enum_dcl().map(|v| AnnotationField::Enum(Box::new(v))),
        // const_dcl().map(|v| AnnotationField::Const(Box::new(v))),
        // typedef_dcl().map(|v| AnnotationField::Alias(Box::new(v))),
    ))
    .repeated()
}

// Rule 222
fn annotation_member() -> impl IdlParser<Field> {
    let param = annotation_member_type().then(simple_declarator());
    let default = just(Kind::Default).ignore_then(ident());
    let def = param.then(default.or_not()).then_ignore(just(Kind::Semi));
    def.map(|((ty, decl), _default)| Field {
        names: vec![decl],
        ty,
    })
}

// Rule 223
fn annotation_member_type() -> impl IdlParser<Type> {
    // `scoped_name` is omitted because it's already included in `const_type`
    choice((const_type(), any_const_type()))
}

// Rule 224
fn any_const_type() -> impl IdlParser<Type> {
    just(Kind::Any).map_with_span(|_, span| Type::Any(AnyType { span }))
}

// Rule 225
fn annotation_appl() -> impl IdlParser<()> {
    annotation_ident()
        .then(annotation_appl_params().parenthesized().or_not())
        .ignored()
}

// Rule 226
fn annotation_appl_params() -> impl IdlParser<()> {
    annotation_appl_param()
        .separated_by(just(Kind::Comma))
        .ignored()
}

// Rule 227
fn annotation_appl_param() -> impl IdlParser<()> {
    choice((
        complex_const_expr()
            .then_ignore(just(Kind::Eq))
            .then(ident())
            .ignored(),
        complex_const_expr().ignored(),
    ))
}
