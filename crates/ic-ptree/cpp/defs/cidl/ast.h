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
// obtain one at https://www.ncia.nato.int/downloads/NCoDe_Licence_V1.0.pdf

#pragma once

#ifdef _WIN32
#  pragma warning(push)
#  pragma warning(disable : 4065)
#  pragma warning(disable : 4127)
#endif

#include <InterCOM/cdr_serializer.h>
#include <InterCOM/json_serializer.h>
#include <InterCOM/memory.h>
#include <InterCOM/span.h>

#include <functional>
#include <optional>

namespace ast {

struct Type;
struct Item;
struct Expr;
struct InterfaceMember;
struct Unary;
struct Binary;
struct NamedExpr;
struct EnumDef;
struct BitmaskDef;
struct ConstDef;
struct Field;

struct Span {
    Span() = default;
    Span(const Span&) = default;
    Span& operator=(const Span&) = default;
    Span(Span&&) = default;
    Span& operator=(Span&&) = default;
    Span(uint32_t a_start, uint32_t a_end);
    bool operator<(const Span& a_other) const;
    bool operator==(const Span& a_other) const;
    bool operator!=(const Span& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Span& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Span& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Span& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Byte offset to the start of the span.
     */
    uint32_t start{0U};
    /**
     * Byte offset to the end of the span.
     */
    uint32_t end{0U};
};

struct Ident {
    Ident() = default;
    Ident(const Ident&) = default;
    Ident& operator=(const Ident&) = default;
    Ident(Ident&&) = default;
    Ident& operator=(Ident&&) = default;
    Ident(::std::string a_name, Span a_span);
    bool operator<(const Ident& a_other) const;
    bool operator==(const Ident& a_other) const;
    bool operator!=(const Ident& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Ident& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Ident& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Ident& a_other) const {
        return !(*this < a_other);
    }

    /**
     * The actual identifier.
     */
    ::std::string name;
    /**
     * Span of the symbol.
     */
    Span span;
};

struct Path {
    Path() = default;
    Path(const Path&) = default;
    Path& operator=(const Path&) = default;
    Path(Path&&) = default;
    Path& operator=(Path&&) = default;
    Path(::std::optional<Span> a_leading_colons, ::std::vector<Ident> a_segments);
    bool operator<(const Path& a_other) const;
    bool operator==(const Path& a_other) const;
    bool operator!=(const Path& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Path& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Path& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Path& a_other) const {
        return !(*this < a_other);
    }

    ::std::optional<Span> leading_colons;
    ::std::vector<Ident> segments;
};

enum LitKind : int32_t { LIT_BOOL, LIT_INT, LIT_FLOAT, LIT_CHAR, LIT_STRING };

struct LitBool {
    LitBool() = default;
    LitBool(const LitBool&) = default;
    LitBool& operator=(const LitBool&) = default;
    LitBool(LitBool&&) = default;
    LitBool& operator=(LitBool&&) = default;
    LitBool(bool a_uppercase, bool a_value);
    bool operator<(const LitBool& a_other) const;
    bool operator==(const LitBool& a_other) const;
    bool operator!=(const LitBool& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const LitBool& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const LitBool& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const LitBool& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Indicates whether the bool was written in uppercase or lowercase.
     */
    bool uppercase{false};
    /**
     * The assigned value of the boolean.
     */
    bool value{false};
};

struct LiteralValue {
    LiteralValue();
    LiteralValue(const LiteralValue& a_other);
    LiteralValue& operator=(const LiteralValue& a_other);
    LiteralValue(LiteralValue&& a_other) noexcept;
    LiteralValue& operator=(LiteralValue&& a_other) noexcept;
    ~LiteralValue() noexcept;

    bool operator<(const LiteralValue& a_other) const;
    bool operator==(const LiteralValue& a_other) const;
    bool operator!=(const LiteralValue& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const LiteralValue& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const LiteralValue& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const LiteralValue& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(LiteralValue& a_first, LiteralValue& a_second) noexcept;

    LitKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(LitKind discriminator);

    LitBool& bool_();
    const LitBool& bool_() const;
    void bool_(const LitBool& a_value);
    void bool_(LitBool&& a_value);

    uint64_t& int_();
    uint64_t int_() const;
    void int_(uint64_t a_value);

    char& char_();
    char char_() const;
    void char_(char a_value);

    ::std::string& string();
    const ::std::string& string() const;
    void string(const ::std::string& a_value);
    void string(::std::string&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        LitBool bool_;
        uint64_t int_;
        char char_;
        ::std::string string;
    } ic_union_value_;
    LitKind ic_discriminator_value_;
    void free_union_();
};
struct Literal {
    Literal() = default;
    Literal(const Literal&) = default;
    Literal& operator=(const Literal&) = default;
    Literal(Literal&&) = default;
    Literal& operator=(Literal&&) = default;
    Literal(Span a_span, LiteralValue a_value);
    bool operator<(const Literal& a_other) const;
    bool operator==(const Literal& a_other) const;
    bool operator!=(const Literal& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Literal& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Literal& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Literal& a_other) const {
        return !(*this < a_other);
    }

    Span span;
    LiteralValue value;
};

enum OpKind : int32_t {
    /**
     * `+`
     */
    OP_ADD,
    /**
     * `-`
     */
    OP_SUB,
    /**
     * `*`
     */
    OP_MULTIPLY,
    /**
     * `/`
     */
    OP_DIVIDE,
    /**
     * `%`
     */
    OP_MODULO,
    /**
     * `<<`
     */
    OP_LSHIFT,
    /**
     * `>>`
     */
    OP_RSHIFT,
    /**
     * `|`
     */
    OP_OR,
    /**
     * `^`
     */
    OP_XOR,
    /**
     * `&`
     */
    OP_AND,
    /**
     * `~`
     */
    OP_NOT
};

struct Op {
    Op() = default;
    Op(const Op&) = default;
    Op& operator=(const Op&) = default;
    Op(Op&&) = default;
    Op& operator=(Op&&) = default;
    Op(Span a_span, OpKind a_kind);
    bool operator<(const Op& a_other) const;
    bool operator==(const Op& a_other) const;
    bool operator!=(const Op& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Op& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Op& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Op& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Span of the token.
     */
    Span span;
    /**
     * The operation kind.
     */
    OpKind kind{OP_ADD};
};

enum ExprKind : int32_t {
    /**
     * A single literal like `1` or `"foo"`
     */
    EXPR_LITERAL,
    /**
     * A possibly scoped identifier like `foo` or `::foo::bar`
     */
    EXPR_PATH,
    /**
     * `-a` or `a`
     */
    EXPR_UNARY,
    /**
     * `a + b`
     */
    EXPR_BINARY,
    /**
     * Initializer list for complex types, e.g. `{1, 2, {3}}`
     */
    EXPR_INIT_LIST
};

struct InitList {
    InitList() = default;
    InitList(const InitList&) = default;
    InitList& operator=(const InitList&) = default;
    InitList(InitList&&) = default;
    InitList& operator=(InitList&&) = default;
    explicit InitList(::std::vector<NamedExpr> a_values);
    bool operator<(const InitList& a_other) const;
    bool operator==(const InitList& a_other) const;
    bool operator!=(const InitList& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const InitList& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const InitList& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const InitList& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<NamedExpr> values;
};

struct Expr {
    Expr();
    Expr(const Expr& a_other);
    Expr& operator=(const Expr& a_other);
    Expr(Expr&& a_other) noexcept;
    Expr& operator=(Expr&& a_other) noexcept;
    ~Expr() noexcept;

    bool operator<(const Expr& a_other) const;
    bool operator==(const Expr& a_other) const;
    bool operator!=(const Expr& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Expr& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Expr& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Expr& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(Expr& a_first, Expr& a_second) noexcept;

    ExprKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(ExprKind discriminator);

    Literal& literal();
    const Literal& literal() const;
    void literal(const Literal& a_value);
    void literal(Literal&& a_value);

    Path& path();
    const Path& path() const;
    void path(const Path& a_value);
    void path(Path&& a_value);

    ::std::unique_ptr<Unary>& unary();
    const ::std::unique_ptr<Unary>& unary() const;
    void unary(const ::std::unique_ptr<Unary>& a_value);
    void unary(::std::unique_ptr<Unary>&& a_value);

    ::std::unique_ptr<Binary>& binary();
    const ::std::unique_ptr<Binary>& binary() const;
    void binary(const ::std::unique_ptr<Binary>& a_value);
    void binary(::std::unique_ptr<Binary>&& a_value);

    InitList& init_list();
    const InitList& init_list() const;
    void init_list(const InitList& a_value);
    void init_list(InitList&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        Literal literal;
        Path path;
        ::std::unique_ptr<Unary> unary;
        ::std::unique_ptr<Binary> binary;
        InitList init_list;
    } ic_union_value_;
    ExprKind ic_discriminator_value_;
    void free_union_();
};
struct NamedExpr {
    NamedExpr() = default;
    NamedExpr(const NamedExpr&) = default;
    NamedExpr& operator=(const NamedExpr&) = default;
    NamedExpr(NamedExpr&&) = default;
    NamedExpr& operator=(NamedExpr&&) = default;
    NamedExpr(::std::optional<Ident> a_ident, Expr a_value);
    bool operator<(const NamedExpr& a_other) const;
    bool operator==(const NamedExpr& a_other) const;
    bool operator!=(const NamedExpr& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const NamedExpr& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const NamedExpr& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const NamedExpr& a_other) const {
        return !(*this < a_other);
    }

    ::std::optional<Ident> ident;
    Expr value;
};

struct Unary {
    Unary() = default;
    Unary(const Unary&) = default;
    Unary& operator=(const Unary&) = default;
    Unary(Unary&&) = default;
    Unary& operator=(Unary&&) = default;
    Unary(Op a_op, Expr a_expr);
    bool operator<(const Unary& a_other) const;
    bool operator==(const Unary& a_other) const;
    bool operator!=(const Unary& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Unary& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Unary& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Unary& a_other) const {
        return !(*this < a_other);
    }

    Op op;
    Expr expr;
};

struct Binary {
    Binary() = default;
    Binary(const Binary&) = default;
    Binary& operator=(const Binary&) = default;
    Binary(Binary&&) = default;
    Binary& operator=(Binary&&) = default;
    Binary(Expr a_lhs, Op a_op, Expr a_rhs);
    bool operator<(const Binary& a_other) const;
    bool operator==(const Binary& a_other) const;
    bool operator!=(const Binary& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Binary& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Binary& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Binary& a_other) const {
        return !(*this < a_other);
    }

    Expr lhs;
    Op op;
    Expr rhs;
};

struct AnyType {
    AnyType() = default;
    AnyType(const AnyType&) = default;
    AnyType& operator=(const AnyType&) = default;
    AnyType(AnyType&&) = default;
    AnyType& operator=(AnyType&&) = default;
    explicit AnyType(Span a_span);
    bool operator<(const AnyType& a_other) const;
    bool operator==(const AnyType& a_other) const;
    bool operator!=(const AnyType& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AnyType& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AnyType& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AnyType& a_other) const {
        return !(*this < a_other);
    }

    Span span;
};

struct SequenceType {
    SequenceType();
    SequenceType(const SequenceType& a_other);
    SequenceType& operator=(const SequenceType& a_other);

    SequenceType(SequenceType&&) = default;
    SequenceType& operator=(SequenceType&&) = default;
    SequenceType(::std::unique_ptr<Type> a_ty, ::std::optional<Expr> a_bound, Span a_span);
    SequenceType(Type a_ty, ::std::optional<Expr> a_bound, Span a_span);
    bool operator<(const SequenceType& a_other) const;
    bool operator==(const SequenceType& a_other) const;
    bool operator!=(const SequenceType& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const SequenceType& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const SequenceType& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const SequenceType& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(SequenceType& a_first, SequenceType& a_second) noexcept;

    ::std::unique_ptr<Type> ty;
    ::std::optional<Expr> bound;
    Span span;
};

struct StringType {
    StringType() = default;
    StringType(const StringType&) = default;
    StringType& operator=(const StringType&) = default;
    StringType(StringType&&) = default;
    StringType& operator=(StringType&&) = default;
    StringType(bool a_wide, ::std::optional<Expr> a_bound, Span a_span);
    bool operator<(const StringType& a_other) const;
    bool operator==(const StringType& a_other) const;
    bool operator!=(const StringType& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const StringType& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const StringType& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const StringType& a_other) const {
        return !(*this < a_other);
    }

    bool wide{false};
    ::std::optional<Expr> bound;
    Span span;
};

struct MapType {
    MapType();
    MapType(const MapType& a_other);
    MapType& operator=(const MapType& a_other);

    MapType(MapType&&) = default;
    MapType& operator=(MapType&&) = default;
    MapType(
        ::std::unique_ptr<Type> a_key,
        ::std::unique_ptr<Type> a_value,
        ::std::optional<Expr> a_bound,
        Span a_span
    );
    MapType(Type a_key, Type a_value, ::std::optional<Expr> a_bound, Span a_span);
    bool operator<(const MapType& a_other) const;
    bool operator==(const MapType& a_other) const;
    bool operator!=(const MapType& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const MapType& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const MapType& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const MapType& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(MapType& a_first, MapType& a_second) noexcept;

    ::std::unique_ptr<Type> key;
    ::std::unique_ptr<Type> value;
    ::std::optional<Expr> bound;
    Span span;
};

struct Fixed {
    Fixed() = default;
    Fixed(const Fixed&) = default;
    Fixed& operator=(const Fixed&) = default;
    Fixed(Fixed&&) = default;
    Fixed& operator=(Fixed&&) = default;
    Fixed(Expr a_total, Expr a_fractional);
    bool operator<(const Fixed& a_other) const;
    bool operator==(const Fixed& a_other) const;
    bool operator!=(const Fixed& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Fixed& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Fixed& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Fixed& a_other) const {
        return !(*this < a_other);
    }

    Expr total;
    Expr fractional;
};

struct FixedType {
    FixedType() = default;
    FixedType(const FixedType&) = default;
    FixedType& operator=(const FixedType&) = default;
    FixedType(FixedType&&) = default;
    FixedType& operator=(FixedType&&) = default;
    FixedType(Span a_span, ::std::optional<Fixed> a_bounds);
    bool operator<(const FixedType& a_other) const;
    bool operator==(const FixedType& a_other) const;
    bool operator!=(const FixedType& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const FixedType& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const FixedType& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const FixedType& a_other) const {
        return !(*this < a_other);
    }

    Span span;
    ::std::optional<Fixed> bounds;
};

enum TypeKind : int32_t { TYPE_ANY, TYPE_SEQUENCE, TYPE_STRING, TYPE_MAP, TYPE_FIXED, TYPE_PATH };

struct Type {
    Type();
    Type(const Type& a_other);
    Type& operator=(const Type& a_other);
    Type(Type&& a_other) noexcept;
    Type& operator=(Type&& a_other) noexcept;
    ~Type() noexcept;

    bool operator<(const Type& a_other) const;
    bool operator==(const Type& a_other) const;
    bool operator!=(const Type& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Type& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Type& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Type& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(Type& a_first, Type& a_second) noexcept;

    TypeKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(TypeKind discriminator);

    AnyType& any();
    const AnyType& any() const;
    void any(const AnyType& a_value);
    void any(AnyType&& a_value);

    SequenceType& sequence();
    const SequenceType& sequence() const;
    void sequence(const SequenceType& a_value);
    void sequence(SequenceType&& a_value);

    StringType& string();
    const StringType& string() const;
    void string(const StringType& a_value);
    void string(StringType&& a_value);

    MapType& map();
    const MapType& map() const;
    void map(const MapType& a_value);
    void map(MapType&& a_value);

    FixedType& fixed();
    const FixedType& fixed() const;
    void fixed(const FixedType& a_value);
    void fixed(FixedType&& a_value);

    Path& path();
    const Path& path() const;
    void path(const Path& a_value);
    void path(Path&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        /**
         * The `any` type.
         */
        AnyType any;
        /**
         * Sequence of another type, e.g. `sequence<string>`.
         */
        SequenceType sequence;
        /**
         * A possibly bounded string.
         */
        StringType string;
        /**
         * (key, value) pair of types, e.g. `map<string, string>`.
         */
        MapType map;
        /**
         * Fixed-point type, e.g. `fixed` or `fixed<4, 2>`.
         */
        FixedType fixed;
        /**
         * A possibly qualified identifier of a type, e.g. `foo::Bar`.
         */
        Path path;
    } ic_union_value_;
    TypeKind ic_discriminator_value_;
    void free_union_();
};
enum DeclaratorKind : int32_t {
    /**
     * A single, non-qualified identifier.
     */
    DECLARATOR_SIMPLE,
    /**
     * An array declarator, e.g. `value[3][4][5]`.
     */
    DECLARATOR_ARRAY
};

struct ArrayDeclarator {
    ArrayDeclarator() = default;
    ArrayDeclarator(const ArrayDeclarator&) = default;
    ArrayDeclarator& operator=(const ArrayDeclarator&) = default;
    ArrayDeclarator(ArrayDeclarator&&) = default;
    ArrayDeclarator& operator=(ArrayDeclarator&&) = default;
    ArrayDeclarator(Ident a_ident, ::std::vector<Expr> a_bounds);
    bool operator<(const ArrayDeclarator& a_other) const;
    bool operator==(const ArrayDeclarator& a_other) const;
    bool operator!=(const ArrayDeclarator& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ArrayDeclarator& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ArrayDeclarator& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ArrayDeclarator& a_other) const {
        return !(*this < a_other);
    }

    Ident ident;
    ::std::vector<Expr> bounds;
};

struct Declarator {
    Declarator();
    Declarator(const Declarator& a_other);
    Declarator& operator=(const Declarator& a_other);
    Declarator(Declarator&& a_other) noexcept;
    Declarator& operator=(Declarator&& a_other) noexcept;
    ~Declarator() noexcept;

    bool operator<(const Declarator& a_other) const;
    bool operator==(const Declarator& a_other) const;
    bool operator!=(const Declarator& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Declarator& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Declarator& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Declarator& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(Declarator& a_first, Declarator& a_second) noexcept;

    DeclaratorKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(DeclaratorKind discriminator);

    Ident& simple();
    const Ident& simple() const;
    void simple(const Ident& a_value);
    void simple(Ident&& a_value);

    ArrayDeclarator& array();
    const ArrayDeclarator& array() const;
    void array(const ArrayDeclarator& a_value);
    void array(ArrayDeclarator&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        Ident simple;
        ArrayDeclarator array;
    } ic_union_value_;
    DeclaratorKind ic_discriminator_value_;
    void free_union_();
};
/**
 * A parameter inside an applied annotation, e.g. `value=true` in
 * `@optional(value=true)`.
 */
struct AnnotationArg {
    AnnotationArg() = default;
    AnnotationArg(const AnnotationArg&) = default;
    AnnotationArg& operator=(const AnnotationArg&) = default;
    AnnotationArg(AnnotationArg&&) = default;
    AnnotationArg& operator=(AnnotationArg&&) = default;
    AnnotationArg(::std::optional<Ident> a_ident, Span a_span, Expr a_value);
    bool operator<(const AnnotationArg& a_other) const;
    bool operator==(const AnnotationArg& a_other) const;
    bool operator!=(const AnnotationArg& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AnnotationArg& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AnnotationArg& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AnnotationArg& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Name of the parameter if one was specified.
     * May be omitted for annotations with only a single, non-default member,
     * but this is not enforced by the parser.
     */
    ::std::optional<Ident> ident;
    /**
     * Span of the entire parameter.
     */
    Span span;
    /**
     * The specified value of the parameter.
     */
    Expr value;
};

struct AnnotationAppl {
    AnnotationAppl() = default;
    AnnotationAppl(const AnnotationAppl&) = default;
    AnnotationAppl& operator=(const AnnotationAppl&) = default;
    AnnotationAppl(AnnotationAppl&&) = default;
    AnnotationAppl& operator=(AnnotationAppl&&) = default;
    AnnotationAppl(Ident a_ident, Span a_span, ::std::vector<AnnotationArg> a_args);
    bool operator<(const AnnotationAppl& a_other) const;
    bool operator==(const AnnotationAppl& a_other) const;
    bool operator!=(const AnnotationAppl& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AnnotationAppl& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AnnotationAppl& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AnnotationAppl& a_other) const {
        return !(*this < a_other);
    }

    Ident ident;
    Span span;
    ::std::vector<AnnotationArg> args;
};

struct Stmt {
    Stmt() = default;
    Stmt(const Stmt&) = default;
    Stmt& operator=(const Stmt&) = default;
    Stmt(Stmt&&) = default;
    Stmt& operator=(Stmt&&) = default;
    virtual ~Stmt() noexcept {}
    Stmt(Ident a_ident, Span a_span, ::std::vector<AnnotationAppl> a_annotations);
    bool operator<(const Stmt& a_other) const;
    bool operator==(const Stmt& a_other) const;
    bool operator!=(const Stmt& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Stmt& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Stmt& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Stmt& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Name of the item.
     */
    Ident ident;
    /**
     * Span of the entire item, from start to end. For example, given the
     * following IDL:
     *
     * ```idl
     * module foo { ... };
     * ````
     *
     * The span of the above module will start at 'm' and end at '}'.
     */
    Span span;
    /**
     * Annotations that were applied to this item.
     */
    ::std::vector<AnnotationAppl> annotations;
};

enum AnnotationFieldKind : int32_t { FIELD_DEFINITION, FIELD_MEMBER };

struct AnnotationField {
    AnnotationField();
    AnnotationField(const AnnotationField& a_other);
    AnnotationField& operator=(const AnnotationField& a_other);
    AnnotationField(AnnotationField&& a_other) noexcept;
    AnnotationField& operator=(AnnotationField&& a_other) noexcept;
    ~AnnotationField() noexcept;

    bool operator<(const AnnotationField& a_other) const;
    bool operator==(const AnnotationField& a_other) const;
    bool operator!=(const AnnotationField& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AnnotationField& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AnnotationField& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AnnotationField& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(AnnotationField& a_first, AnnotationField& a_second) noexcept;

    AnnotationFieldKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(AnnotationFieldKind discriminator);

    ::std::unique_ptr<Item>& item();
    const ::std::unique_ptr<Item>& item() const;
    void item(const ::std::unique_ptr<Item>& a_value);
    void item(::std::unique_ptr<Item>&& a_value);

    ::std::unique_ptr<Field>& member();
    const ::std::unique_ptr<Field>& member() const;
    void member(const ::std::unique_ptr<Field>& a_value);
    void member(::std::unique_ptr<Field>&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        ::std::unique_ptr<Item> item;
        ::std::unique_ptr<Field> member;
    } ic_union_value_;
    AnnotationFieldKind ic_discriminator_value_;
    void free_union_();
};
struct AnnotationDef : public Stmt {
    AnnotationDef() = default;
    AnnotationDef(const AnnotationDef&) = default;
    AnnotationDef& operator=(const AnnotationDef&) = default;
    AnnotationDef(AnnotationDef&&) = default;
    AnnotationDef& operator=(AnnotationDef&&) = default;
    AnnotationDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<AnnotationField> a_params
    );
    bool operator<(const AnnotationDef& a_other) const;
    bool operator==(const AnnotationDef& a_other) const;
    bool operator!=(const AnnotationDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AnnotationDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AnnotationDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AnnotationDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<AnnotationField> params;
};

struct ModuleDef : public Stmt {
    ModuleDef() = default;
    ModuleDef(const ModuleDef&) = default;
    ModuleDef& operator=(const ModuleDef&) = default;
    ModuleDef(ModuleDef&&) = default;
    ModuleDef& operator=(ModuleDef&&) = default;
    ModuleDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Item> a_definitions
    );
    bool operator<(const ModuleDef& a_other) const;
    bool operator==(const ModuleDef& a_other) const;
    bool operator!=(const ModuleDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ModuleDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ModuleDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ModuleDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Item> definitions;
};

struct Field {
    Field() = default;
    Field(const Field&) = default;
    Field& operator=(const Field&) = default;
    Field(Field&&) = default;
    Field& operator=(Field&&) = default;
    Field(::std::vector<Declarator> a_names, Type a_ty);
    bool operator<(const Field& a_other) const;
    bool operator==(const Field& a_other) const;
    bool operator!=(const Field& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Field& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Field& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Field& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Declarator> names;
    Type ty;
};

struct StructDef : public Stmt {
    StructDef() = default;
    StructDef(const StructDef&) = default;
    StructDef& operator=(const StructDef&) = default;
    StructDef(StructDef&&) = default;
    StructDef& operator=(StructDef&&) = default;
    StructDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Field> a_members,
        ::std::optional<Path> a_parent
    );
    bool operator<(const StructDef& a_other) const;
    bool operator==(const StructDef& a_other) const;
    bool operator!=(const StructDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const StructDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const StructDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const StructDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Field> members;
    ::std::optional<Path> parent;
};

struct Discriminator {
    Discriminator() = default;
    Discriminator(const Discriminator&) = default;
    Discriminator& operator=(const Discriminator&) = default;
    Discriminator(Discriminator&&) = default;
    Discriminator& operator=(Discriminator&&) = default;
    Discriminator(::std::vector<AnnotationAppl> a_annotations, Type a_ty);
    bool operator<(const Discriminator& a_other) const;
    bool operator==(const Discriminator& a_other) const;
    bool operator!=(const Discriminator& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Discriminator& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Discriminator& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Discriminator& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<AnnotationAppl> annotations;
    Type ty;
};

struct Empty {
    Empty() = default;
    Empty(const Empty&) = default;
    Empty& operator=(const Empty&) = default;
    Empty(Empty&&) = default;
    Empty& operator=(Empty&&) = default;
    bool operator<(const Empty&) const {
        return false;
    }
    bool operator!=(const Empty&) const {
        return false;
    }
    bool operator==(const Empty&) const {
        return true;
    }
    bool operator>(const Empty&) const {
        return false;
    }
    bool operator<=(const Empty&) const {
        return true;
    }
    bool operator>=(const Empty&) const {
        return true;
    }
};

enum LabelKind : int32_t { LABEL_CASE, LABEL_DEFAULT };

struct Label {
    Label();
    Label(const Label& a_other);
    Label& operator=(const Label& a_other);
    Label(Label&& a_other) noexcept;
    Label& operator=(Label&& a_other) noexcept;
    ~Label() noexcept;

    bool operator<(const Label& a_other) const;
    bool operator==(const Label& a_other) const;
    bool operator!=(const Label& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Label& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Label& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Label& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(Label& a_first, Label& a_second) noexcept;

    LabelKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(LabelKind discriminator);

    Expr& case_();
    const Expr& case_() const;
    void case_(const Expr& a_value);
    void case_(Expr&& a_value);

    Empty& default_();
    const Empty& default_() const;
    void default_(const Empty& a_value);
    void default_(Empty&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        Expr case_;
        Empty default_;
    } ic_union_value_;
    LabelKind ic_discriminator_value_;
    void free_union_();
};
enum UnionElementKind : int32_t { ELEMENT_MEMBER, ELEMENT_NULL };

struct UnionMember {
    UnionMember();
    UnionMember(const UnionMember& a_other);
    UnionMember& operator=(const UnionMember& a_other);

    UnionMember(UnionMember&&) = default;
    UnionMember& operator=(UnionMember&&) = default;
    UnionMember(::std::unique_ptr<Type> a_ty, Declarator a_decl);
    UnionMember(Type a_ty, Declarator a_decl);
    bool operator<(const UnionMember& a_other) const;
    bool operator==(const UnionMember& a_other) const;
    bool operator!=(const UnionMember& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const UnionMember& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const UnionMember& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const UnionMember& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(UnionMember& a_first, UnionMember& a_second) noexcept;

    ::std::unique_ptr<Type> ty;
    Declarator decl;
};

struct UnionNull {
    UnionNull() = default;
    UnionNull(const UnionNull&) = default;
    UnionNull& operator=(const UnionNull&) = default;
    UnionNull(UnionNull&&) = default;
    UnionNull& operator=(UnionNull&&) = default;
    explicit UnionNull(Span a_span);
    bool operator<(const UnionNull& a_other) const;
    bool operator==(const UnionNull& a_other) const;
    bool operator!=(const UnionNull& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const UnionNull& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const UnionNull& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const UnionNull& a_other) const {
        return !(*this < a_other);
    }

    Span span;
};

struct UnionElement {
    UnionElement();
    UnionElement(const UnionElement& a_other);
    UnionElement& operator=(const UnionElement& a_other);
    UnionElement(UnionElement&& a_other) noexcept;
    UnionElement& operator=(UnionElement&& a_other) noexcept;
    ~UnionElement() noexcept;

    bool operator<(const UnionElement& a_other) const;
    bool operator==(const UnionElement& a_other) const;
    bool operator!=(const UnionElement& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const UnionElement& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const UnionElement& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const UnionElement& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(UnionElement& a_first, UnionElement& a_second) noexcept;

    UnionElementKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(UnionElementKind discriminator);

    UnionMember& member();
    const UnionMember& member() const;
    void member(const UnionMember& a_value);
    void member(UnionMember&& a_value);

    UnionNull& null();
    const UnionNull& null() const;
    void null(const UnionNull& a_value);
    void null(UnionNull&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        UnionMember member;
        UnionNull null;
    } ic_union_value_;
    UnionElementKind ic_discriminator_value_;
    void free_union_();
};
struct UnionField {
    UnionField() = default;
    UnionField(const UnionField&) = default;
    UnionField& operator=(const UnionField&) = default;
    UnionField(UnionField&&) = default;
    UnionField& operator=(UnionField&&) = default;
    UnionField(
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Label> a_labels,
        UnionElement a_field
    );
    bool operator<(const UnionField& a_other) const;
    bool operator==(const UnionField& a_other) const;
    bool operator!=(const UnionField& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const UnionField& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const UnionField& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const UnionField& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<AnnotationAppl> annotations;
    /**
     * Case labels that map to this variant.
     */
    ::std::vector<Label> labels;
    UnionElement field;
};

struct UnionDef : public Stmt {
    UnionDef() = default;
    UnionDef(const UnionDef&) = default;
    UnionDef& operator=(const UnionDef&) = default;
    UnionDef(UnionDef&&) = default;
    UnionDef& operator=(UnionDef&&) = default;
    UnionDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        Discriminator a_disc,
        ::std::vector<UnionField> a_fields
    );
    bool operator<(const UnionDef& a_other) const;
    bool operator==(const UnionDef& a_other) const;
    bool operator!=(const UnionDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const UnionDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const UnionDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const UnionDef& a_other) const {
        return !(*this < a_other);
    }

    /**
     * The discriminator component of the union.
     */
    Discriminator disc;
    /**
     * All variants of the union. The case labels that map to each variant can
     * be found in `UnionField`.
     */
    ::std::vector<UnionField> fields;
};

struct ConstDef : public Stmt {
    ConstDef() = default;
    ConstDef(const ConstDef&) = default;
    ConstDef& operator=(const ConstDef&) = default;
    ConstDef(ConstDef&&) = default;
    ConstDef& operator=(ConstDef&&) = default;
    ConstDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        Declarator a_decl,
        Type a_ty,
        Expr a_value
    );
    bool operator<(const ConstDef& a_other) const;
    bool operator==(const ConstDef& a_other) const;
    bool operator!=(const ConstDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ConstDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ConstDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ConstDef& a_other) const {
        return !(*this < a_other);
    }

    Declarator decl;
    Type ty;
    Expr value;
};

struct Enumerator {
    Enumerator() = default;
    Enumerator(const Enumerator&) = default;
    Enumerator& operator=(const Enumerator&) = default;
    Enumerator(Enumerator&&) = default;
    Enumerator& operator=(Enumerator&&) = default;
    Enumerator(
        Ident a_ident,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::optional<Expr> a_value
    );
    bool operator<(const Enumerator& a_other) const;
    bool operator==(const Enumerator& a_other) const;
    bool operator!=(const Enumerator& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Enumerator& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Enumerator& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Enumerator& a_other) const {
        return !(*this < a_other);
    }

    Ident ident;
    ::std::vector<AnnotationAppl> annotations;
    /**
     * An explicit value, e.g. `enum Foo { VALUE = 1 };`
     * The `@value` annotation will *not* populate this field.
     */
    ::std::optional<Expr> value;
};

struct EnumDef : public Stmt {
    EnumDef() = default;
    EnumDef(const EnumDef&) = default;
    EnumDef& operator=(const EnumDef&) = default;
    EnumDef(EnumDef&&) = default;
    EnumDef& operator=(EnumDef&&) = default;
    EnumDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Enumerator> a_fields
    );
    bool operator<(const EnumDef& a_other) const;
    bool operator==(const EnumDef& a_other) const;
    bool operator!=(const EnumDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const EnumDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const EnumDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const EnumDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Enumerator> fields;
};

struct ExceptDef : public Stmt {
    ExceptDef() = default;
    ExceptDef(const ExceptDef&) = default;
    ExceptDef& operator=(const ExceptDef&) = default;
    ExceptDef(ExceptDef&&) = default;
    ExceptDef& operator=(ExceptDef&&) = default;
    ExceptDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Field> a_members
    );
    bool operator<(const ExceptDef& a_other) const;
    bool operator==(const ExceptDef& a_other) const;
    bool operator!=(const ExceptDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ExceptDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ExceptDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ExceptDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Field> members;
};

struct AliasDef : public Stmt {
    AliasDef() = default;
    AliasDef(const AliasDef&) = default;
    AliasDef& operator=(const AliasDef&) = default;
    AliasDef(AliasDef&&) = default;
    AliasDef& operator=(AliasDef&&) = default;
    AliasDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Declarator> a_decl,
        Type a_ty
    );
    bool operator<(const AliasDef& a_other) const;
    bool operator==(const AliasDef& a_other) const;
    bool operator!=(const AliasDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const AliasDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const AliasDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const AliasDef& a_other) const {
        return !(*this < a_other);
    }

    /**
     * List of all declarators. Always contains at least one declarator.
     */
    ::std::vector<Declarator> decl;
    /**
     * The underlying type of the typedef.
     */
    Type ty;
};

struct Bit : public Stmt {
    Bit() = default;
    Bit(const Bit&) = default;
    Bit& operator=(const Bit&) = default;
    Bit(Bit&&) = default;
    Bit& operator=(Bit&&) = default;
    Bit(Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::optional<Expr> a_value);
    bool operator<(const Bit& a_other) const;
    bool operator==(const Bit& a_other) const;
    bool operator!=(const Bit& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Bit& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Bit& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Bit& a_other) const {
        return !(*this < a_other);
    }

    ::std::optional<Expr> value;
};

struct BitmaskDef : public Stmt {
    BitmaskDef() = default;
    BitmaskDef(const BitmaskDef&) = default;
    BitmaskDef& operator=(const BitmaskDef&) = default;
    BitmaskDef(BitmaskDef&&) = default;
    BitmaskDef& operator=(BitmaskDef&&) = default;
    BitmaskDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<Bit> a_bits
    );
    bool operator<(const BitmaskDef& a_other) const;
    bool operator==(const BitmaskDef& a_other) const;
    bool operator!=(const BitmaskDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const BitmaskDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const BitmaskDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const BitmaskDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<Bit> bits;
};

struct Bitfield : public Stmt {
    Bitfield() = default;
    Bitfield(const Bitfield&) = default;
    Bitfield& operator=(const Bitfield&) = default;
    Bitfield(Bitfield&&) = default;
    Bitfield& operator=(Bitfield&&) = default;
    Bitfield(Ident a_ident, Span a_span, ::std::vector<AnnotationAppl> a_annotations, Expr a_size);
    bool operator<(const Bitfield& a_other) const;
    bool operator==(const Bitfield& a_other) const;
    bool operator!=(const Bitfield& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Bitfield& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Bitfield& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Bitfield& a_other) const {
        return !(*this < a_other);
    }

    Expr size;
};

struct BitsetDef : public Stmt {
    BitsetDef() = default;
    BitsetDef(const BitsetDef&) = default;
    BitsetDef& operator=(const BitsetDef&) = default;
    BitsetDef(BitsetDef&&) = default;
    BitsetDef& operator=(BitsetDef&&) = default;
    BitsetDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::optional<Path> a_parent,
        ::std::vector<Bitfield> a_fields
    );
    bool operator<(const BitsetDef& a_other) const;
    bool operator==(const BitsetDef& a_other) const;
    bool operator!=(const BitsetDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const BitsetDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const BitsetDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const BitsetDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::optional<Path> parent;
    ::std::vector<Bitfield> fields;
};

struct Attribute {
    Attribute() = default;
    Attribute(const Attribute&) = default;
    Attribute& operator=(const Attribute&) = default;
    Attribute(Attribute&&) = default;
    Attribute& operator=(Attribute&&) = default;
    Attribute(Ident a_ident, Type a_ty, ::std::optional<Span> a_readonly);
    bool operator<(const Attribute& a_other) const;
    bool operator==(const Attribute& a_other) const;
    bool operator!=(const Attribute& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Attribute& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Attribute& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Attribute& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Name of the attribute.
     */
    Ident ident;
    /**
     * The type of the attribute.
     */
    Type ty;
    /**
     * Indicates whether this attribute was marked as `readonly`, and if
     * so, the span of the keyword.
     */
    ::std::optional<Span> readonly;
};

enum ParamKind : int32_t {
    /**
     * Explicitly marked as `in`
     */
    PARAM_IN,
    /**
     * Explicitly marked as `out`
     */
    PARAM_OUT,
    /**
     * Explicitly marked as `inout`
     */
    PARAM_INOUT
};

struct Param {
    Param() = default;
    Param(const Param&) = default;
    Param& operator=(const Param&) = default;
    Param(Param&&) = default;
    Param& operator=(Param&&) = default;
    Param(Ident a_ident, Type a_ty, ::std::optional<ParamKind> a_kind);
    bool operator<(const Param& a_other) const;
    bool operator==(const Param& a_other) const;
    bool operator!=(const Param& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Param& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Param& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Param& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Name of the parameter.
     */
    Ident ident;
    /**
     * Type of the parameter.
     */
    Type ty;
    /**
     * Specifies whether this is an `in`, `out`, or `inout` parameter.
     */
    ::std::optional<ParamKind> kind;
};

struct Prototype {
    Prototype() = default;
    Prototype(const Prototype&) = default;
    Prototype& operator=(const Prototype&) = default;
    Prototype(Prototype&&) = default;
    Prototype& operator=(Prototype&&) = default;
    Prototype(
        Ident a_ident,
        Type a_ret,
        ::std::vector<Param> a_params,
        ::std::vector<Path> a_raises,
        ::std::optional<Span> a_oneway
    );
    bool operator<(const Prototype& a_other) const;
    bool operator==(const Prototype& a_other) const;
    bool operator!=(const Prototype& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Prototype& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Prototype& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Prototype& a_other) const {
        return !(*this < a_other);
    }

    /**
     * Name of the prototype.
     */
    Ident ident;
    /**
     * Return type.
     */
    Type ret;
    ::std::vector<Param> params;
    ::std::vector<Path> raises;
    /**
     * Indicates whether this function was prefixed with the `oneway` keyword.
     * Does not account for the `@oneway` annotation.
     */
    ::std::optional<Span> oneway;
};

struct InterfaceDef : public Stmt {
    InterfaceDef() = default;
    InterfaceDef(const InterfaceDef&) = default;
    InterfaceDef& operator=(const InterfaceDef&) = default;
    InterfaceDef(InterfaceDef&&) = default;
    InterfaceDef& operator=(InterfaceDef&&) = default;
    InterfaceDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<InterfaceMember> a_members,
        ::std::vector<Path> a_inherits,
        ::std::optional<Span> a_local
    );
    bool operator<(const InterfaceDef& a_other) const;
    bool operator==(const InterfaceDef& a_other) const;
    bool operator!=(const InterfaceDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const InterfaceDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const InterfaceDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const InterfaceDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<InterfaceMember> members;
    ::std::vector<Path> inherits;
    ::std::optional<Span> local;
};

struct ValueMember {
    ValueMember() = default;
    ValueMember(const ValueMember&) = default;
    ValueMember& operator=(const ValueMember&) = default;
    ValueMember(ValueMember&&) = default;
    ValueMember& operator=(ValueMember&&) = default;
    ValueMember(Ident a_ident, Type a_ty, ::std::optional<Span> a_public_);
    bool operator<(const ValueMember& a_other) const;
    bool operator==(const ValueMember& a_other) const;
    bool operator!=(const ValueMember& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ValueMember& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ValueMember& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ValueMember& a_other) const {
        return !(*this < a_other);
    }

    Ident ident;
    Type ty;
    ::std::optional<Span> public_;
};

struct ValuetypeDef : public Stmt {
    ValuetypeDef() = default;
    ValuetypeDef(const ValuetypeDef&) = default;
    ValuetypeDef& operator=(const ValuetypeDef&) = default;
    ValuetypeDef(ValuetypeDef&&) = default;
    ValuetypeDef& operator=(ValuetypeDef&&) = default;
    ValuetypeDef(
        Ident a_ident,
        Span a_span,
        ::std::vector<AnnotationAppl> a_annotations,
        ::std::vector<ValueMember> a_members,
        ::std::vector<Prototype> a_prototypes,
        ::std::optional<Path> a_inherits,
        ::std::optional<Path> a_supports
    );
    bool operator<(const ValuetypeDef& a_other) const;
    bool operator==(const ValuetypeDef& a_other) const;
    bool operator!=(const ValuetypeDef& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const ValuetypeDef& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const ValuetypeDef& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const ValuetypeDef& a_other) const {
        return !(*this < a_other);
    }

    ::std::vector<ValueMember> members;
    ::std::vector<Prototype> prototypes;
    ::std::optional<Path> inherits;
    ::std::optional<Path> supports;
};

enum DeclKind : int32_t { DECL_STRUCT, DECL_UNION, DECL_NATIVE, DECL_INTERFACE, DECL_VALUETYPE };

struct Decl : public Stmt {
    Decl() = default;
    Decl(const Decl&) = default;
    Decl& operator=(const Decl&) = default;
    Decl(Decl&&) = default;
    Decl& operator=(Decl&&) = default;
    Decl(Ident a_ident, Span a_span, ::std::vector<AnnotationAppl> a_annotations, DeclKind a_kind);
    bool operator<(const Decl& a_other) const;
    bool operator==(const Decl& a_other) const;
    bool operator!=(const Decl& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Decl& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Decl& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Decl& a_other) const {
        return !(*this < a_other);
    }

    DeclKind kind{DECL_STRUCT};
};

enum ItemKind : int32_t {
    /**
     * A definition of an annotation
     */
    ITEM_ANNOTATION,
    /**
     * Module declaration
     */
    ITEM_MODULE,
    /**
     * Struct definition
     */
    ITEM_STRUCT,
    /**
     * Union definition
     */
    ITEM_UNION,
    /**
     * Enum definition
     */
    ITEM_ENUM,
    /**
     * Exception definition
     */
    ITEM_EXCEPTION,
    /**
     * Bitmask definition
     */
    ITEM_BITMASK,
    /**
     * Bitset definition
     */
    ITEM_BITSET,
    /**
     * Declaration of a `const`
     */
    ITEM_CONST,
    /**
     * Typedef definition
     */
    ITEM_TYPEDEF,
    /**
     * Interface definition
     */
    ITEM_INTERFACE,
    /**
     * Valuetype definition
     */
    ITEM_VALUETYPE,
    /**
     * A forward declaration
     */
    ITEM_DECL
};

struct Item {
    Item();
    Item(const Item& a_other);
    Item& operator=(const Item& a_other);
    Item(Item&& a_other) noexcept;
    Item& operator=(Item&& a_other) noexcept;
    ~Item() noexcept;

    bool operator<(const Item& a_other) const;
    bool operator==(const Item& a_other) const;
    bool operator!=(const Item& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const Item& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const Item& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const Item& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(Item& a_first, Item& a_second) noexcept;

    ItemKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(ItemKind discriminator);

    AnnotationDef& annotation_value();
    const AnnotationDef& annotation_value() const;
    void annotation_value(const AnnotationDef& a_value);
    void annotation_value(AnnotationDef&& a_value);

    ModuleDef& module_value();
    const ModuleDef& module_value() const;
    void module_value(const ModuleDef& a_value);
    void module_value(ModuleDef&& a_value);

    StructDef& struct_value();
    const StructDef& struct_value() const;
    void struct_value(const StructDef& a_value);
    void struct_value(StructDef&& a_value);

    UnionDef& union_value();
    const UnionDef& union_value() const;
    void union_value(const UnionDef& a_value);
    void union_value(UnionDef&& a_value);

    EnumDef& enum_value();
    const EnumDef& enum_value() const;
    void enum_value(const EnumDef& a_value);
    void enum_value(EnumDef&& a_value);

    ExceptDef& exception_value();
    const ExceptDef& exception_value() const;
    void exception_value(const ExceptDef& a_value);
    void exception_value(ExceptDef&& a_value);

    BitmaskDef& bitmask_value();
    const BitmaskDef& bitmask_value() const;
    void bitmask_value(const BitmaskDef& a_value);
    void bitmask_value(BitmaskDef&& a_value);

    BitsetDef& bitset_value();
    const BitsetDef& bitset_value() const;
    void bitset_value(const BitsetDef& a_value);
    void bitset_value(BitsetDef&& a_value);

    ConstDef& const_value();
    const ConstDef& const_value() const;
    void const_value(const ConstDef& a_value);
    void const_value(ConstDef&& a_value);

    AliasDef& alias_value();
    const AliasDef& alias_value() const;
    void alias_value(const AliasDef& a_value);
    void alias_value(AliasDef&& a_value);

    InterfaceDef& interface_value();
    const InterfaceDef& interface_value() const;
    void interface_value(const InterfaceDef& a_value);
    void interface_value(InterfaceDef&& a_value);

    ValuetypeDef& valuetype_value();
    const ValuetypeDef& valuetype_value() const;
    void valuetype_value(const ValuetypeDef& a_value);
    void valuetype_value(ValuetypeDef&& a_value);

    Decl& decl_value();
    const Decl& decl_value() const;
    void decl_value(const Decl& a_value);
    void decl_value(Decl&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        /**
         * A definition of an annotation
         */
        AnnotationDef annotation_value;
        /**
         * Module declaration
         */
        ModuleDef module_value;
        /**
         * Struct definition
         */
        StructDef struct_value;
        /**
         * Union definition
         */
        UnionDef union_value;
        /**
         * Enum definition
         */
        EnumDef enum_value;
        /**
         * Exception definition
         */
        ExceptDef exception_value;
        /**
         * Bitmask definition
         */
        BitmaskDef bitmask_value;
        /**
         * Bitset definition
         */
        BitsetDef bitset_value;
        /**
         * Declaration of a `const`
         */
        ConstDef const_value;
        /**
         * Typedef definition
         */
        AliasDef alias_value;
        /**
         * Interface definition
         */
        InterfaceDef interface_value;
        /**
         * Valuetype definition
         */
        ValuetypeDef valuetype_value;
        /**
         * A forward declaration
         */
        Decl decl_value;
    } ic_union_value_;
    ItemKind ic_discriminator_value_;
    void free_union_();
};
enum InterfaceMemberKind : int32_t {
    /**
     * An interface attribute.
     */
    INTERFACE_ATTRIBUTE,
    /**
     * Function definition.
     */
    INTERFACE_PROTOTYPE,
    /**
     * Type definition nested inside the interface.
     */
    INTERFACE_ITEM
};

struct InterfaceMember {
    InterfaceMember();
    InterfaceMember(const InterfaceMember& a_other);
    InterfaceMember& operator=(const InterfaceMember& a_other);
    InterfaceMember(InterfaceMember&& a_other) noexcept;
    InterfaceMember& operator=(InterfaceMember&& a_other) noexcept;
    ~InterfaceMember() noexcept;

    bool operator<(const InterfaceMember& a_other) const;
    bool operator==(const InterfaceMember& a_other) const;
    bool operator!=(const InterfaceMember& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const InterfaceMember& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const InterfaceMember& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const InterfaceMember& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(InterfaceMember& a_first, InterfaceMember& a_second) noexcept;

    InterfaceMemberKind _d() const {
        return ic_discriminator_value_;
    }
    void _d(InterfaceMemberKind discriminator);

    Attribute& attr();
    const Attribute& attr() const;
    void attr(const Attribute& a_value);
    void attr(Attribute&& a_value);

    Prototype& proto();
    const Prototype& proto() const;
    void proto(const Prototype& a_value);
    void proto(Prototype&& a_value);

    Item& item();
    const Item& item() const;
    void item(const Item& a_value);
    void item(Item&& a_value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        /**
         * An interface attribute.
         */
        Attribute attr;
        /**
         * Function definition.
         */
        Prototype proto;
        /**
         * Type definition nested inside the interface.
         */
        Item item;
    } ic_union_value_;
    InterfaceMemberKind ic_discriminator_value_;
    void free_union_();
};
}  // namespace ast

namespace std {
template <>
struct hash<ast::Span> {
    using argument_type = ast::Span;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Ident> {
    using argument_type = ast::Ident;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Path> {
    using argument_type = ast::Path;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::LitBool> {
    using argument_type = ast::LitBool;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::LiteralValue> {
    using argument_type = ast::LiteralValue;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Literal> {
    using argument_type = ast::Literal;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Op> {
    using argument_type = ast::Op;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::InitList> {
    using argument_type = ast::InitList;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Expr> {
    using argument_type = ast::Expr;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::NamedExpr> {
    using argument_type = ast::NamedExpr;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Unary> {
    using argument_type = ast::Unary;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Binary> {
    using argument_type = ast::Binary;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AnyType> {
    using argument_type = ast::AnyType;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::SequenceType> {
    using argument_type = ast::SequenceType;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::StringType> {
    using argument_type = ast::StringType;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::MapType> {
    using argument_type = ast::MapType;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Fixed> {
    using argument_type = ast::Fixed;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::FixedType> {
    using argument_type = ast::FixedType;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Type> {
    using argument_type = ast::Type;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ArrayDeclarator> {
    using argument_type = ast::ArrayDeclarator;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Declarator> {
    using argument_type = ast::Declarator;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AnnotationArg> {
    using argument_type = ast::AnnotationArg;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AnnotationAppl> {
    using argument_type = ast::AnnotationAppl;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Stmt> {
    using argument_type = ast::Stmt;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AnnotationField> {
    using argument_type = ast::AnnotationField;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AnnotationDef> {
    using argument_type = ast::AnnotationDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ModuleDef> {
    using argument_type = ast::ModuleDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Field> {
    using argument_type = ast::Field;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::StructDef> {
    using argument_type = ast::StructDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Discriminator> {
    using argument_type = ast::Discriminator;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Empty> {
    using argument_type = ast::Empty;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Label> {
    using argument_type = ast::Label;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::UnionMember> {
    using argument_type = ast::UnionMember;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::UnionNull> {
    using argument_type = ast::UnionNull;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::UnionElement> {
    using argument_type = ast::UnionElement;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::UnionField> {
    using argument_type = ast::UnionField;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::UnionDef> {
    using argument_type = ast::UnionDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ConstDef> {
    using argument_type = ast::ConstDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Enumerator> {
    using argument_type = ast::Enumerator;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::EnumDef> {
    using argument_type = ast::EnumDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ExceptDef> {
    using argument_type = ast::ExceptDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::AliasDef> {
    using argument_type = ast::AliasDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Bit> {
    using argument_type = ast::Bit;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::BitmaskDef> {
    using argument_type = ast::BitmaskDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Bitfield> {
    using argument_type = ast::Bitfield;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::BitsetDef> {
    using argument_type = ast::BitsetDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Attribute> {
    using argument_type = ast::Attribute;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Param> {
    using argument_type = ast::Param;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Prototype> {
    using argument_type = ast::Prototype;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::InterfaceDef> {
    using argument_type = ast::InterfaceDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ValueMember> {
    using argument_type = ast::ValueMember;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::ValuetypeDef> {
    using argument_type = ast::ValuetypeDef;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Decl> {
    using argument_type = ast::Decl;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::Item> {
    using argument_type = ast::Item;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct hash<ast::InterfaceMember> {
    using argument_type = ast::InterfaceMember;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
}  // namespace std

namespace ast {

inline Span::Span(uint32_t a_start, uint32_t a_end) : start(a_start), end(a_end) {}

inline bool Span::operator<(const Span& a_other) const {
    if (this->start < a_other.start) {
        return true;
    }
    if (a_other.start < this->start) {
        return false;
    }
    return this->end < a_other.end;
}

inline bool Span::operator==(const Span& a_other) const {
    if (!(this->start == a_other.start)) {
        return false;
    }
    if (!(this->end == a_other.end)) {
        return false;
    }
    return true;
}

inline Ident::Ident(::std::string a_name, Span a_span)
    : name(std::move(a_name)), span(std::move(a_span)) {}

inline bool Ident::operator<(const Ident& a_other) const {
    if (this->name < a_other.name) {
        return true;
    }
    if (a_other.name < this->name) {
        return false;
    }
    return this->span < a_other.span;
}

inline bool Ident::operator==(const Ident& a_other) const {
    if (!(this->name == a_other.name)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline Path::Path(::std::optional<Span> a_leading_colons, ::std::vector<Ident> a_segments)
    : leading_colons(std::move(a_leading_colons)), segments(std::move(a_segments)) {}

inline bool Path::operator<(const Path& a_other) const {
    if (this->leading_colons < a_other.leading_colons) {
        return true;
    }
    if (a_other.leading_colons < this->leading_colons) {
        return false;
    }
    return this->segments < a_other.segments;
}

inline bool Path::operator==(const Path& a_other) const {
    if (!(this->leading_colons == a_other.leading_colons)) {
        return false;
    }
    if (!(this->segments == a_other.segments)) {
        return false;
    }
    return true;
}

inline LitBool::LitBool(bool a_uppercase, bool a_value) : uppercase(a_uppercase), value(a_value) {}

inline bool LitBool::operator<(const LitBool& a_other) const {
    if (this->uppercase < a_other.uppercase) {
        return true;
    }
    if (a_other.uppercase < this->uppercase) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool LitBool::operator==(const LitBool& a_other) const {
    if (!(this->uppercase == a_other.uppercase)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline LiteralValue::LiteralValue() {
    ic_discriminator_value_ = LIT_BOOL;
    intercom::construct_at(&ic_union_value_.bool_, LitBool{});
}

inline LiteralValue::LiteralValue(const LiteralValue& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case LIT_BOOL:
        intercom::construct_at(&ic_union_value_.bool_, a_other.ic_union_value_.bool_);
        break;
    case LIT_INT:
        ic_union_value_.int_ = a_other.ic_union_value_.int_;
        break;
    case LIT_CHAR:
        ic_union_value_.char_ = a_other.ic_union_value_.char_;
        break;
    case LIT_STRING:
        intercom::construct_at(&ic_union_value_.string, a_other.ic_union_value_.string);
        break;
    default:
        break;
    }
}

inline LiteralValue& LiteralValue::operator=(const LiteralValue& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case LIT_BOOL:
            intercom::construct_at(&ic_union_value_.bool_, a_other.ic_union_value_.bool_);
            break;
        case LIT_INT:
            ic_union_value_.int_ = a_other.ic_union_value_.int_;
            break;
        case LIT_CHAR:
            ic_union_value_.char_ = a_other.ic_union_value_.char_;
            break;
        case LIT_STRING:
            intercom::construct_at(&ic_union_value_.string, a_other.ic_union_value_.string);
            break;
        default:
            break;
        }
    }

    return *this;
}

inline LiteralValue::LiteralValue(LiteralValue&& a_other) noexcept : LiteralValue() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case LIT_BOOL:
        intercom::construct_at(&ic_union_value_.bool_, std::move(a_other.ic_union_value_.bool_));
        break;
    case LIT_INT:
        ic_union_value_.int_ = a_other.ic_union_value_.int_;
        break;
    case LIT_CHAR:
        ic_union_value_.char_ = a_other.ic_union_value_.char_;
        break;
    case LIT_STRING:
        intercom::construct_at(&ic_union_value_.string, std::move(a_other.ic_union_value_.string));
        break;
    default:
        break;
    }
}

inline LiteralValue& LiteralValue::operator=(LiteralValue&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case LIT_BOOL:
            intercom::construct_at(
                &ic_union_value_.bool_, std::move(a_other.ic_union_value_.bool_)
            );
            break;
        case LIT_INT:
            ic_union_value_.int_ = a_other.ic_union_value_.int_;
            break;
        case LIT_CHAR:
            ic_union_value_.char_ = a_other.ic_union_value_.char_;
            break;
        case LIT_STRING:
            intercom::construct_at(
                &ic_union_value_.string, std::move(a_other.ic_union_value_.string)
            );
            break;
        default:
            break;
        }
    }
    return *this;
}

inline LiteralValue::~LiteralValue() noexcept {
    free_union_();
}

inline bool LiteralValue::operator<(const LiteralValue& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case LIT_BOOL:
        return this->bool_() < a_other.bool_();
    case LIT_INT:
        return this->int_() < a_other.int_();
    case LIT_CHAR:
        return this->char_() < a_other.char_();
    case LIT_STRING:
        return this->string() < a_other.string();
    default:
        return false;
    }
}

inline bool LiteralValue::operator==(const LiteralValue& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case LIT_BOOL:
        return this->bool_() == a_other.bool_();
    case LIT_INT:
        return this->int_() == a_other.int_();
    case LIT_CHAR:
        return this->char_() == a_other.char_();
    case LIT_STRING:
        return this->string() == a_other.string();
    default:
        return true;
    }
}

inline void swap(LiteralValue& a_first, LiteralValue& a_second) noexcept {
    LiteralValue a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void LiteralValue::_d(LitKind discriminator) {
    switch (discriminator) {
    case LIT_BOOL:
        if (ic_discriminator_value_ != LIT_BOOL) {
            free_union_();
            intercom::construct_at(&ic_union_value_.bool_, LitBool{});
        }
        break;
    case LIT_INT:
        if (ic_discriminator_value_ != LIT_INT) {
            free_union_();
            ic_union_value_.int_ = 0ULL;
        }
        break;
    case LIT_CHAR:
        if (ic_discriminator_value_ != LIT_CHAR) {
            free_union_();
            ic_union_value_.char_ = '\000';
        }
        break;
    case LIT_STRING:
        if (ic_discriminator_value_ != LIT_STRING) {
            free_union_();
            intercom::construct_at(&ic_union_value_.string, ::std::string{});
        }
        break;
    default:
        free_union_();
        break;
    }
    ic_discriminator_value_ = discriminator;
}

inline LitBool& LiteralValue::bool_() {
    if (ic_discriminator_value_ != LIT_BOOL) {
        throw std::logic_error("Union LiteralValue not set to value bool_");
    }
    return ic_union_value_.bool_;
}

inline const LitBool& LiteralValue::bool_() const {
    if (ic_discriminator_value_ != LIT_BOOL) {
        throw std::logic_error("Union LiteralValue not set to value bool_");
    }
    return ic_union_value_.bool_;
}

inline void LiteralValue::bool_(const LitBool& a_value) {
    if (ic_discriminator_value_ != LIT_BOOL) {
        free_union_();
        ic_discriminator_value_ = LIT_BOOL;
        intercom::construct_at(&ic_union_value_.bool_, a_value);
    } else {
        ic_union_value_.bool_ = a_value;
    }
}

inline void LiteralValue::bool_(LitBool&& a_value) {
    if (ic_discriminator_value_ != LIT_BOOL) {
        free_union_();
        ic_discriminator_value_ = LIT_BOOL;
        intercom::construct_at(&ic_union_value_.bool_, std::move(a_value));
    } else {
        ic_union_value_.bool_ = std::move(a_value);
    }
}

inline uint64_t& LiteralValue::int_() {
    if (ic_discriminator_value_ != LIT_INT) {
        throw std::logic_error("Union LiteralValue not set to value int_");
    }
    return ic_union_value_.int_;
}

inline uint64_t LiteralValue::int_() const {
    if (ic_discriminator_value_ != LIT_INT) {
        throw std::logic_error("Union LiteralValue not set to value int_");
    }
    return ic_union_value_.int_;
}

inline void LiteralValue::int_(uint64_t a_value) {
    if (ic_discriminator_value_ != LIT_INT) {
        free_union_();
        ic_discriminator_value_ = LIT_INT;
    }
    ic_union_value_.int_ = a_value;
}

inline char& LiteralValue::char_() {
    if (ic_discriminator_value_ != LIT_CHAR) {
        throw std::logic_error("Union LiteralValue not set to value char_");
    }
    return ic_union_value_.char_;
}

inline char LiteralValue::char_() const {
    if (ic_discriminator_value_ != LIT_CHAR) {
        throw std::logic_error("Union LiteralValue not set to value char_");
    }
    return ic_union_value_.char_;
}

inline void LiteralValue::char_(char a_value) {
    if (ic_discriminator_value_ != LIT_CHAR) {
        free_union_();
        ic_discriminator_value_ = LIT_CHAR;
    }
    ic_union_value_.char_ = a_value;
}

inline ::std::string& LiteralValue::string() {
    if (ic_discriminator_value_ != LIT_STRING) {
        throw std::logic_error("Union LiteralValue not set to value string");
    }
    return ic_union_value_.string;
}

inline const ::std::string& LiteralValue::string() const {
    if (ic_discriminator_value_ != LIT_STRING) {
        throw std::logic_error("Union LiteralValue not set to value string");
    }
    return ic_union_value_.string;
}

inline void LiteralValue::string(const ::std::string& a_value) {
    if (ic_discriminator_value_ != LIT_STRING) {
        free_union_();
        ic_discriminator_value_ = LIT_STRING;
        intercom::construct_at(&ic_union_value_.string, a_value);
    } else {
        ic_union_value_.string = a_value;
    }
}

inline void LiteralValue::string(::std::string&& a_value) {
    if (ic_discriminator_value_ != LIT_STRING) {
        free_union_();
        ic_discriminator_value_ = LIT_STRING;
        intercom::construct_at(&ic_union_value_.string, std::move(a_value));
    } else {
        ic_union_value_.string = std::move(a_value);
    }
}

inline void LiteralValue::free_union_() {
    switch (ic_discriminator_value_) {
    case LIT_BOOL:
        std::destroy_at(&ic_union_value_.bool_);
        break;
    case LIT_INT:
        break;
    case LIT_CHAR:
        break;
    case LIT_STRING:
        std::destroy_at(&ic_union_value_.string);
        break;
    default:
        break;
    }
}

inline Literal::Literal(Span a_span, LiteralValue a_value)
    : span(std::move(a_span)), value(std::move(a_value)) {}

inline bool Literal::operator<(const Literal& a_other) const {
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool Literal::operator==(const Literal& a_other) const {
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline Op::Op(Span a_span, OpKind a_kind) : span(std::move(a_span)), kind(a_kind) {}

inline bool Op::operator<(const Op& a_other) const {
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->kind < a_other.kind;
}

inline bool Op::operator==(const Op& a_other) const {
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->kind == a_other.kind)) {
        return false;
    }
    return true;
}

inline InitList::InitList(::std::vector<NamedExpr> a_values) : values(std::move(a_values)) {}

inline bool InitList::operator<(const InitList& a_other) const {
    return this->values < a_other.values;
}

inline bool InitList::operator==(const InitList& a_other) const {
    if (!(this->values == a_other.values)) {
        return false;
    }
    return true;
}

inline Expr::Expr() {
    ic_discriminator_value_ = EXPR_LITERAL;
    intercom::construct_at(&ic_union_value_.literal, Literal{});
}

inline Expr::Expr(const Expr& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case EXPR_LITERAL:
        intercom::construct_at(&ic_union_value_.literal, a_other.ic_union_value_.literal);
        break;
    case EXPR_PATH:
        intercom::construct_at(&ic_union_value_.path, a_other.ic_union_value_.path);
        break;
    case EXPR_UNARY:
        intercom::construct_at(&ic_union_value_.unary, new Unary(*a_other.ic_union_value_.unary));
        break;
    case EXPR_BINARY:
        intercom::construct_at(
            &ic_union_value_.binary, new Binary(*a_other.ic_union_value_.binary)
        );
        break;
    case EXPR_INIT_LIST:
        intercom::construct_at(&ic_union_value_.init_list, a_other.ic_union_value_.init_list);
        break;
    }
}

inline Expr& Expr::operator=(const Expr& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case EXPR_LITERAL:
            intercom::construct_at(&ic_union_value_.literal, a_other.ic_union_value_.literal);
            break;
        case EXPR_PATH:
            intercom::construct_at(&ic_union_value_.path, a_other.ic_union_value_.path);
            break;
        case EXPR_UNARY:
            intercom::construct_at(
                &ic_union_value_.unary, new Unary(*a_other.ic_union_value_.unary)
            );
            break;
        case EXPR_BINARY:
            intercom::construct_at(
                &ic_union_value_.binary, new Binary(*a_other.ic_union_value_.binary)
            );
            break;
        case EXPR_INIT_LIST:
            intercom::construct_at(&ic_union_value_.init_list, a_other.ic_union_value_.init_list);
            break;
        }
    }

    return *this;
}

inline Expr::Expr(Expr&& a_other) noexcept : Expr() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case EXPR_LITERAL:
        intercom::construct_at(
            &ic_union_value_.literal, std::move(a_other.ic_union_value_.literal)
        );
        break;
    case EXPR_PATH:
        intercom::construct_at(&ic_union_value_.path, std::move(a_other.ic_union_value_.path));
        break;
    case EXPR_UNARY:
        intercom::construct_at(&ic_union_value_.unary, std::move(a_other.ic_union_value_.unary));
        break;
    case EXPR_BINARY:
        intercom::construct_at(&ic_union_value_.binary, std::move(a_other.ic_union_value_.binary));
        break;
    case EXPR_INIT_LIST:
        intercom::construct_at(
            &ic_union_value_.init_list, std::move(a_other.ic_union_value_.init_list)
        );
        break;
    }
}

inline Expr& Expr::operator=(Expr&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case EXPR_LITERAL:
            intercom::construct_at(
                &ic_union_value_.literal, std::move(a_other.ic_union_value_.literal)
            );
            break;
        case EXPR_PATH:
            intercom::construct_at(&ic_union_value_.path, std::move(a_other.ic_union_value_.path));
            break;
        case EXPR_UNARY:
            intercom::construct_at(
                &ic_union_value_.unary, std::move(a_other.ic_union_value_.unary)
            );
            break;
        case EXPR_BINARY:
            intercom::construct_at(
                &ic_union_value_.binary, std::move(a_other.ic_union_value_.binary)
            );
            break;
        case EXPR_INIT_LIST:
            intercom::construct_at(
                &ic_union_value_.init_list, std::move(a_other.ic_union_value_.init_list)
            );
            break;
        }
    }
    return *this;
}

inline Expr::~Expr() noexcept {
    free_union_();
}

inline bool Expr::operator<(const Expr& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case EXPR_LITERAL:
        return this->literal() < a_other.literal();
    case EXPR_PATH:
        return this->path() < a_other.path();
    case EXPR_UNARY:
        if (!this->unary() || !a_other.unary()) {
            return this->unary() < a_other.unary();
        }
        return *(this->unary()) < *a_other.unary();
    case EXPR_BINARY:
        if (!this->binary() || !a_other.binary()) {
            return this->binary() < a_other.binary();
        }
        return *(this->binary()) < *a_other.binary();
    case EXPR_INIT_LIST:
        return this->init_list() < a_other.init_list();
    }
    return false;
}

inline bool Expr::operator==(const Expr& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case EXPR_LITERAL:
        return this->literal() == a_other.literal();
    case EXPR_PATH:
        return this->path() == a_other.path();
    case EXPR_UNARY:
        if (this->unary() == a_other.unary()) {
            return true;
        }
        if (!this->unary() || !a_other.unary()) {
            return false;
        }
        return *(this->unary()) == *a_other.unary();
    case EXPR_BINARY:
        if (this->binary() == a_other.binary()) {
            return true;
        }
        if (!this->binary() || !a_other.binary()) {
            return false;
        }
        return *(this->binary()) == *a_other.binary();
    case EXPR_INIT_LIST:
        return this->init_list() == a_other.init_list();
    }
    return true;
}

inline void swap(Expr& a_first, Expr& a_second) noexcept {
    Expr a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void Expr::_d(ExprKind discriminator) {
    switch (discriminator) {
    case EXPR_LITERAL:
        if (ic_discriminator_value_ != EXPR_LITERAL) {
            free_union_();
            intercom::construct_at(&ic_union_value_.literal, Literal{});
        }
        break;
    case EXPR_PATH:
        if (ic_discriminator_value_ != EXPR_PATH) {
            free_union_();
            intercom::construct_at(&ic_union_value_.path, Path{});
        }
        break;
    case EXPR_UNARY:
        if (ic_discriminator_value_ != EXPR_UNARY) {
            free_union_();
            intercom::construct_at(&ic_union_value_.unary, ::std::unique_ptr<Unary>(new Unary{}));
        }
        break;
    case EXPR_BINARY:
        if (ic_discriminator_value_ != EXPR_BINARY) {
            free_union_();
            intercom::construct_at(
                &ic_union_value_.binary, ::std::unique_ptr<Binary>(new Binary{})
            );
        }
        break;
    case EXPR_INIT_LIST:
        if (ic_discriminator_value_ != EXPR_INIT_LIST) {
            free_union_();
            intercom::construct_at(&ic_union_value_.init_list, InitList{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union Expr");
    }
    ic_discriminator_value_ = discriminator;
}

inline Literal& Expr::literal() {
    if (ic_discriminator_value_ != EXPR_LITERAL) {
        throw std::logic_error("Union Expr not set to value literal");
    }
    return ic_union_value_.literal;
}

inline const Literal& Expr::literal() const {
    if (ic_discriminator_value_ != EXPR_LITERAL) {
        throw std::logic_error("Union Expr not set to value literal");
    }
    return ic_union_value_.literal;
}

inline void Expr::literal(const Literal& a_value) {
    if (ic_discriminator_value_ != EXPR_LITERAL) {
        free_union_();
        ic_discriminator_value_ = EXPR_LITERAL;
        intercom::construct_at(&ic_union_value_.literal, a_value);
    } else {
        ic_union_value_.literal = a_value;
    }
}

inline void Expr::literal(Literal&& a_value) {
    if (ic_discriminator_value_ != EXPR_LITERAL) {
        free_union_();
        ic_discriminator_value_ = EXPR_LITERAL;
        intercom::construct_at(&ic_union_value_.literal, std::move(a_value));
    } else {
        ic_union_value_.literal = std::move(a_value);
    }
}

inline Path& Expr::path() {
    if (ic_discriminator_value_ != EXPR_PATH) {
        throw std::logic_error("Union Expr not set to value path");
    }
    return ic_union_value_.path;
}

inline const Path& Expr::path() const {
    if (ic_discriminator_value_ != EXPR_PATH) {
        throw std::logic_error("Union Expr not set to value path");
    }
    return ic_union_value_.path;
}

inline void Expr::path(const Path& a_value) {
    if (ic_discriminator_value_ != EXPR_PATH) {
        free_union_();
        ic_discriminator_value_ = EXPR_PATH;
        intercom::construct_at(&ic_union_value_.path, a_value);
    } else {
        ic_union_value_.path = a_value;
    }
}

inline void Expr::path(Path&& a_value) {
    if (ic_discriminator_value_ != EXPR_PATH) {
        free_union_();
        ic_discriminator_value_ = EXPR_PATH;
        intercom::construct_at(&ic_union_value_.path, std::move(a_value));
    } else {
        ic_union_value_.path = std::move(a_value);
    }
}

inline ::std::unique_ptr<Unary>& Expr::unary() {
    if (ic_discriminator_value_ != EXPR_UNARY) {
        throw std::logic_error("Union Expr not set to value unary");
    }
    return ic_union_value_.unary;
}

inline const ::std::unique_ptr<Unary>& Expr::unary() const {
    if (ic_discriminator_value_ != EXPR_UNARY) {
        throw std::logic_error("Union Expr not set to value unary");
    }
    return ic_union_value_.unary;
}

inline void Expr::unary(const ::std::unique_ptr<Unary>& a_value) {
    if (ic_discriminator_value_ != EXPR_UNARY) {
        free_union_();
        ic_discriminator_value_ = EXPR_UNARY;
        intercom::construct_at(&ic_union_value_.unary, new Unary(*a_value));
    } else {
        ic_union_value_.unary.reset(new Unary(*a_value));
    }
}

inline void Expr::unary(::std::unique_ptr<Unary>&& a_value) {
    if (ic_discriminator_value_ != EXPR_UNARY) {
        free_union_();
        ic_discriminator_value_ = EXPR_UNARY;
        intercom::construct_at(&ic_union_value_.unary, std::move(a_value));
    } else {
        ic_union_value_.unary = std::move(a_value);
    }
}

inline ::std::unique_ptr<Binary>& Expr::binary() {
    if (ic_discriminator_value_ != EXPR_BINARY) {
        throw std::logic_error("Union Expr not set to value binary");
    }
    return ic_union_value_.binary;
}

inline const ::std::unique_ptr<Binary>& Expr::binary() const {
    if (ic_discriminator_value_ != EXPR_BINARY) {
        throw std::logic_error("Union Expr not set to value binary");
    }
    return ic_union_value_.binary;
}

inline void Expr::binary(const ::std::unique_ptr<Binary>& a_value) {
    if (ic_discriminator_value_ != EXPR_BINARY) {
        free_union_();
        ic_discriminator_value_ = EXPR_BINARY;
        intercom::construct_at(&ic_union_value_.binary, new Binary(*a_value));
    } else {
        ic_union_value_.binary.reset(new Binary(*a_value));
    }
}

inline void Expr::binary(::std::unique_ptr<Binary>&& a_value) {
    if (ic_discriminator_value_ != EXPR_BINARY) {
        free_union_();
        ic_discriminator_value_ = EXPR_BINARY;
        intercom::construct_at(&ic_union_value_.binary, std::move(a_value));
    } else {
        ic_union_value_.binary = std::move(a_value);
    }
}

inline InitList& Expr::init_list() {
    if (ic_discriminator_value_ != EXPR_INIT_LIST) {
        throw std::logic_error("Union Expr not set to value init_list");
    }
    return ic_union_value_.init_list;
}

inline const InitList& Expr::init_list() const {
    if (ic_discriminator_value_ != EXPR_INIT_LIST) {
        throw std::logic_error("Union Expr not set to value init_list");
    }
    return ic_union_value_.init_list;
}

inline void Expr::init_list(const InitList& a_value) {
    if (ic_discriminator_value_ != EXPR_INIT_LIST) {
        free_union_();
        ic_discriminator_value_ = EXPR_INIT_LIST;
        intercom::construct_at(&ic_union_value_.init_list, a_value);
    } else {
        ic_union_value_.init_list = a_value;
    }
}

inline void Expr::init_list(InitList&& a_value) {
    if (ic_discriminator_value_ != EXPR_INIT_LIST) {
        free_union_();
        ic_discriminator_value_ = EXPR_INIT_LIST;
        intercom::construct_at(&ic_union_value_.init_list, std::move(a_value));
    } else {
        ic_union_value_.init_list = std::move(a_value);
    }
}

inline void Expr::free_union_() {
    switch (ic_discriminator_value_) {
    case EXPR_LITERAL:
        std::destroy_at(&ic_union_value_.literal);
        break;
    case EXPR_PATH:
        std::destroy_at(&ic_union_value_.path);
        break;
    case EXPR_UNARY:
        std::destroy_at(&ic_union_value_.unary);
        break;
    case EXPR_BINARY:
        std::destroy_at(&ic_union_value_.binary);
        break;
    case EXPR_INIT_LIST:
        std::destroy_at(&ic_union_value_.init_list);
        break;
    }
}

inline NamedExpr::NamedExpr(::std::optional<Ident> a_ident, Expr a_value)
    : ident(std::move(a_ident)), value(std::move(a_value)) {}

inline bool NamedExpr::operator<(const NamedExpr& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool NamedExpr::operator==(const NamedExpr& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline Unary::Unary(Op a_op, Expr a_expr) : op(std::move(a_op)), expr(std::move(a_expr)) {}

inline bool Unary::operator<(const Unary& a_other) const {
    if (this->op < a_other.op) {
        return true;
    }
    if (a_other.op < this->op) {
        return false;
    }
    return this->expr < a_other.expr;
}

inline bool Unary::operator==(const Unary& a_other) const {
    if (!(this->op == a_other.op)) {
        return false;
    }
    if (!(this->expr == a_other.expr)) {
        return false;
    }
    return true;
}

inline Binary::Binary(Expr a_lhs, Op a_op, Expr a_rhs)
    : lhs(std::move(a_lhs)), op(std::move(a_op)), rhs(std::move(a_rhs)) {}

inline bool Binary::operator<(const Binary& a_other) const {
    if (this->lhs < a_other.lhs) {
        return true;
    }
    if (a_other.lhs < this->lhs) {
        return false;
    }
    if (this->op < a_other.op) {
        return true;
    }
    if (a_other.op < this->op) {
        return false;
    }
    return this->rhs < a_other.rhs;
}

inline bool Binary::operator==(const Binary& a_other) const {
    if (!(this->lhs == a_other.lhs)) {
        return false;
    }
    if (!(this->op == a_other.op)) {
        return false;
    }
    if (!(this->rhs == a_other.rhs)) {
        return false;
    }
    return true;
}

inline AnyType::AnyType(Span a_span) : span(std::move(a_span)) {}

inline bool AnyType::operator<(const AnyType& a_other) const {
    return this->span < a_other.span;
}

inline bool AnyType::operator==(const AnyType& a_other) const {
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline SequenceType::SequenceType() : ty{new Type{}} {}

inline SequenceType::SequenceType(const SequenceType& a_other)
    : ty(!a_other.ty ? nullptr : std::unique_ptr<Type>(new Type(*a_other.ty))),
      bound(std::move(a_other.bound)),
      span(std::move(a_other.span)) {}

inline SequenceType& SequenceType::operator=(const SequenceType& a_other) {
    SequenceType a_copy(a_other);
    swap(*this, a_copy);
    return *this;
}

inline SequenceType::SequenceType(
    ::std::unique_ptr<Type> a_ty,
    ::std::optional<Expr> a_bound,
    Span a_span
)
    : ty(!a_ty ? nullptr : std::unique_ptr<Type>(new Type(*a_ty))),
      bound(std::move(a_bound)),
      span(std::move(a_span)) {}

inline SequenceType::SequenceType(Type a_ty, ::std::optional<Expr> a_bound, Span a_span)
    : ty(std::unique_ptr<Type>(new Type(a_ty))),
      bound(std::move(a_bound)),
      span(std::move(a_span)) {}

inline bool SequenceType::operator<(const SequenceType& a_other) const {
    if (!this->ty || !a_other.ty) {
        if (this->ty != a_other.ty) {
            return this->ty < a_other.ty;
        }
    } else {
        if (*(this->ty) < *a_other.ty) {
            return true;
        }
        if (*a_other.ty < *(this->ty)) {
            return false;
        }
    }
    if (this->bound < a_other.bound) {
        return true;
    }
    if (a_other.bound < this->bound) {
        return false;
    }
    return this->span < a_other.span;
}

inline bool SequenceType::operator==(const SequenceType& a_other) const {
    if (!(this->ty == a_other.ty)) {
        if (!this->ty || !a_other.ty) {
            return false;
        }
        if (!(*this->ty == *a_other.ty)) {
            return false;
        }
    }
    if (!(this->bound == a_other.bound)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline void swap(SequenceType& a_first, SequenceType& a_second) noexcept {
    using std::swap;
    swap(a_first.ty, a_second.ty);
    swap(a_first.bound, a_second.bound);
    swap(a_first.span, a_second.span);
}

inline StringType::StringType(bool a_wide, ::std::optional<Expr> a_bound, Span a_span)
    : wide(a_wide), bound(std::move(a_bound)), span(std::move(a_span)) {}

inline bool StringType::operator<(const StringType& a_other) const {
    if (this->wide < a_other.wide) {
        return true;
    }
    if (a_other.wide < this->wide) {
        return false;
    }
    if (this->bound < a_other.bound) {
        return true;
    }
    if (a_other.bound < this->bound) {
        return false;
    }
    return this->span < a_other.span;
}

inline bool StringType::operator==(const StringType& a_other) const {
    if (!(this->wide == a_other.wide)) {
        return false;
    }
    if (!(this->bound == a_other.bound)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline MapType::MapType() : key{new Type{}}, value{new Type{}} {}

inline MapType::MapType(const MapType& a_other)
    : key(!a_other.key ? nullptr : std::unique_ptr<Type>(new Type(*a_other.key))),
      value(!a_other.value ? nullptr : std::unique_ptr<Type>(new Type(*a_other.value))),
      bound(std::move(a_other.bound)),
      span(std::move(a_other.span)) {}

inline MapType& MapType::operator=(const MapType& a_other) {
    MapType a_copy(a_other);
    swap(*this, a_copy);
    return *this;
}

inline MapType::MapType(
    ::std::unique_ptr<Type> a_key,
    ::std::unique_ptr<Type> a_value,
    ::std::optional<Expr> a_bound,
    Span a_span
)
    : key(!a_key ? nullptr : std::unique_ptr<Type>(new Type(*a_key))),
      value(!a_value ? nullptr : std::unique_ptr<Type>(new Type(*a_value))),
      bound(std::move(a_bound)),
      span(std::move(a_span)) {}

inline MapType::MapType(Type a_key, Type a_value, ::std::optional<Expr> a_bound, Span a_span)
    : key(std::unique_ptr<Type>(new Type(a_key))),
      value(std::unique_ptr<Type>(new Type(a_value))),
      bound(std::move(a_bound)),
      span(std::move(a_span)) {}

inline bool MapType::operator<(const MapType& a_other) const {
    if (!this->key || !a_other.key) {
        if (this->key != a_other.key) {
            return this->key < a_other.key;
        }
    } else {
        if (*(this->key) < *a_other.key) {
            return true;
        }
        if (*a_other.key < *(this->key)) {
            return false;
        }
    }
    if (!this->value || !a_other.value) {
        if (this->value != a_other.value) {
            return this->value < a_other.value;
        }
    } else {
        if (*(this->value) < *a_other.value) {
            return true;
        }
        if (*a_other.value < *(this->value)) {
            return false;
        }
    }
    if (this->bound < a_other.bound) {
        return true;
    }
    if (a_other.bound < this->bound) {
        return false;
    }
    return this->span < a_other.span;
}

inline bool MapType::operator==(const MapType& a_other) const {
    if (!(this->key == a_other.key)) {
        if (!this->key || !a_other.key) {
            return false;
        }
        if (!(*this->key == *a_other.key)) {
            return false;
        }
    }
    if (!(this->value == a_other.value)) {
        if (!this->value || !a_other.value) {
            return false;
        }
        if (!(*this->value == *a_other.value)) {
            return false;
        }
    }
    if (!(this->bound == a_other.bound)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline void swap(MapType& a_first, MapType& a_second) noexcept {
    using std::swap;
    swap(a_first.key, a_second.key);
    swap(a_first.value, a_second.value);
    swap(a_first.bound, a_second.bound);
    swap(a_first.span, a_second.span);
}

inline Fixed::Fixed(Expr a_total, Expr a_fractional)
    : total(std::move(a_total)), fractional(std::move(a_fractional)) {}

inline bool Fixed::operator<(const Fixed& a_other) const {
    if (this->total < a_other.total) {
        return true;
    }
    if (a_other.total < this->total) {
        return false;
    }
    return this->fractional < a_other.fractional;
}

inline bool Fixed::operator==(const Fixed& a_other) const {
    if (!(this->total == a_other.total)) {
        return false;
    }
    if (!(this->fractional == a_other.fractional)) {
        return false;
    }
    return true;
}

inline FixedType::FixedType(Span a_span, ::std::optional<Fixed> a_bounds)
    : span(std::move(a_span)), bounds(std::move(a_bounds)) {}

inline bool FixedType::operator<(const FixedType& a_other) const {
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->bounds < a_other.bounds;
}

inline bool FixedType::operator==(const FixedType& a_other) const {
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->bounds == a_other.bounds)) {
        return false;
    }
    return true;
}

inline Type::Type() {
    ic_discriminator_value_ = TYPE_ANY;
    intercom::construct_at(&ic_union_value_.any, AnyType{});
}

inline Type::Type(const Type& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case TYPE_ANY:
        intercom::construct_at(&ic_union_value_.any, a_other.ic_union_value_.any);
        break;
    case TYPE_SEQUENCE:
        intercom::construct_at(&ic_union_value_.sequence, a_other.ic_union_value_.sequence);
        break;
    case TYPE_STRING:
        intercom::construct_at(&ic_union_value_.string, a_other.ic_union_value_.string);
        break;
    case TYPE_MAP:
        intercom::construct_at(&ic_union_value_.map, a_other.ic_union_value_.map);
        break;
    case TYPE_FIXED:
        intercom::construct_at(&ic_union_value_.fixed, a_other.ic_union_value_.fixed);
        break;
    case TYPE_PATH:
        intercom::construct_at(&ic_union_value_.path, a_other.ic_union_value_.path);
        break;
    }
}

inline Type& Type::operator=(const Type& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case TYPE_ANY:
            intercom::construct_at(&ic_union_value_.any, a_other.ic_union_value_.any);
            break;
        case TYPE_SEQUENCE:
            intercom::construct_at(&ic_union_value_.sequence, a_other.ic_union_value_.sequence);
            break;
        case TYPE_STRING:
            intercom::construct_at(&ic_union_value_.string, a_other.ic_union_value_.string);
            break;
        case TYPE_MAP:
            intercom::construct_at(&ic_union_value_.map, a_other.ic_union_value_.map);
            break;
        case TYPE_FIXED:
            intercom::construct_at(&ic_union_value_.fixed, a_other.ic_union_value_.fixed);
            break;
        case TYPE_PATH:
            intercom::construct_at(&ic_union_value_.path, a_other.ic_union_value_.path);
            break;
        }
    }

    return *this;
}

inline Type::Type(Type&& a_other) noexcept : Type() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case TYPE_ANY:
        intercom::construct_at(&ic_union_value_.any, std::move(a_other.ic_union_value_.any));
        break;
    case TYPE_SEQUENCE:
        intercom::construct_at(
            &ic_union_value_.sequence, std::move(a_other.ic_union_value_.sequence)
        );
        break;
    case TYPE_STRING:
        intercom::construct_at(&ic_union_value_.string, std::move(a_other.ic_union_value_.string));
        break;
    case TYPE_MAP:
        intercom::construct_at(&ic_union_value_.map, std::move(a_other.ic_union_value_.map));
        break;
    case TYPE_FIXED:
        intercom::construct_at(&ic_union_value_.fixed, std::move(a_other.ic_union_value_.fixed));
        break;
    case TYPE_PATH:
        intercom::construct_at(&ic_union_value_.path, std::move(a_other.ic_union_value_.path));
        break;
    }
}

inline Type& Type::operator=(Type&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case TYPE_ANY:
            intercom::construct_at(&ic_union_value_.any, std::move(a_other.ic_union_value_.any));
            break;
        case TYPE_SEQUENCE:
            intercom::construct_at(
                &ic_union_value_.sequence, std::move(a_other.ic_union_value_.sequence)
            );
            break;
        case TYPE_STRING:
            intercom::construct_at(
                &ic_union_value_.string, std::move(a_other.ic_union_value_.string)
            );
            break;
        case TYPE_MAP:
            intercom::construct_at(&ic_union_value_.map, std::move(a_other.ic_union_value_.map));
            break;
        case TYPE_FIXED:
            intercom::construct_at(
                &ic_union_value_.fixed, std::move(a_other.ic_union_value_.fixed)
            );
            break;
        case TYPE_PATH:
            intercom::construct_at(&ic_union_value_.path, std::move(a_other.ic_union_value_.path));
            break;
        }
    }
    return *this;
}

inline Type::~Type() noexcept {
    free_union_();
}

inline bool Type::operator<(const Type& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case TYPE_ANY:
        return this->any() < a_other.any();
    case TYPE_SEQUENCE:
        return this->sequence() < a_other.sequence();
    case TYPE_STRING:
        return this->string() < a_other.string();
    case TYPE_MAP:
        return this->map() < a_other.map();
    case TYPE_FIXED:
        return this->fixed() < a_other.fixed();
    case TYPE_PATH:
        return this->path() < a_other.path();
    }
    return false;
}

inline bool Type::operator==(const Type& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case TYPE_ANY:
        return this->any() == a_other.any();
    case TYPE_SEQUENCE:
        return this->sequence() == a_other.sequence();
    case TYPE_STRING:
        return this->string() == a_other.string();
    case TYPE_MAP:
        return this->map() == a_other.map();
    case TYPE_FIXED:
        return this->fixed() == a_other.fixed();
    case TYPE_PATH:
        return this->path() == a_other.path();
    }
    return true;
}

inline void swap(Type& a_first, Type& a_second) noexcept {
    Type a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void Type::_d(TypeKind discriminator) {
    switch (discriminator) {
    case TYPE_ANY:
        if (ic_discriminator_value_ != TYPE_ANY) {
            free_union_();
            intercom::construct_at(&ic_union_value_.any, AnyType{});
        }
        break;
    case TYPE_SEQUENCE:
        if (ic_discriminator_value_ != TYPE_SEQUENCE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.sequence, SequenceType{});
        }
        break;
    case TYPE_STRING:
        if (ic_discriminator_value_ != TYPE_STRING) {
            free_union_();
            intercom::construct_at(&ic_union_value_.string, StringType{});
        }
        break;
    case TYPE_MAP:
        if (ic_discriminator_value_ != TYPE_MAP) {
            free_union_();
            intercom::construct_at(&ic_union_value_.map, MapType{});
        }
        break;
    case TYPE_FIXED:
        if (ic_discriminator_value_ != TYPE_FIXED) {
            free_union_();
            intercom::construct_at(&ic_union_value_.fixed, FixedType{});
        }
        break;
    case TYPE_PATH:
        if (ic_discriminator_value_ != TYPE_PATH) {
            free_union_();
            intercom::construct_at(&ic_union_value_.path, Path{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union Type");
    }
    ic_discriminator_value_ = discriminator;
}

inline AnyType& Type::any() {
    if (ic_discriminator_value_ != TYPE_ANY) {
        throw std::logic_error("Union Type not set to value any");
    }
    return ic_union_value_.any;
}

inline const AnyType& Type::any() const {
    if (ic_discriminator_value_ != TYPE_ANY) {
        throw std::logic_error("Union Type not set to value any");
    }
    return ic_union_value_.any;
}

inline void Type::any(const AnyType& a_value) {
    if (ic_discriminator_value_ != TYPE_ANY) {
        free_union_();
        ic_discriminator_value_ = TYPE_ANY;
        intercom::construct_at(&ic_union_value_.any, a_value);
    } else {
        ic_union_value_.any = a_value;
    }
}

inline void Type::any(AnyType&& a_value) {
    if (ic_discriminator_value_ != TYPE_ANY) {
        free_union_();
        ic_discriminator_value_ = TYPE_ANY;
        intercom::construct_at(&ic_union_value_.any, std::move(a_value));
    } else {
        ic_union_value_.any = std::move(a_value);
    }
}

inline SequenceType& Type::sequence() {
    if (ic_discriminator_value_ != TYPE_SEQUENCE) {
        throw std::logic_error("Union Type not set to value sequence");
    }
    return ic_union_value_.sequence;
}

inline const SequenceType& Type::sequence() const {
    if (ic_discriminator_value_ != TYPE_SEQUENCE) {
        throw std::logic_error("Union Type not set to value sequence");
    }
    return ic_union_value_.sequence;
}

inline void Type::sequence(const SequenceType& a_value) {
    if (ic_discriminator_value_ != TYPE_SEQUENCE) {
        free_union_();
        ic_discriminator_value_ = TYPE_SEQUENCE;
        intercom::construct_at(&ic_union_value_.sequence, a_value);
    } else {
        ic_union_value_.sequence = a_value;
    }
}

inline void Type::sequence(SequenceType&& a_value) {
    if (ic_discriminator_value_ != TYPE_SEQUENCE) {
        free_union_();
        ic_discriminator_value_ = TYPE_SEQUENCE;
        intercom::construct_at(&ic_union_value_.sequence, std::move(a_value));
    } else {
        ic_union_value_.sequence = std::move(a_value);
    }
}

inline StringType& Type::string() {
    if (ic_discriminator_value_ != TYPE_STRING) {
        throw std::logic_error("Union Type not set to value string");
    }
    return ic_union_value_.string;
}

inline const StringType& Type::string() const {
    if (ic_discriminator_value_ != TYPE_STRING) {
        throw std::logic_error("Union Type not set to value string");
    }
    return ic_union_value_.string;
}

inline void Type::string(const StringType& a_value) {
    if (ic_discriminator_value_ != TYPE_STRING) {
        free_union_();
        ic_discriminator_value_ = TYPE_STRING;
        intercom::construct_at(&ic_union_value_.string, a_value);
    } else {
        ic_union_value_.string = a_value;
    }
}

inline void Type::string(StringType&& a_value) {
    if (ic_discriminator_value_ != TYPE_STRING) {
        free_union_();
        ic_discriminator_value_ = TYPE_STRING;
        intercom::construct_at(&ic_union_value_.string, std::move(a_value));
    } else {
        ic_union_value_.string = std::move(a_value);
    }
}

inline MapType& Type::map() {
    if (ic_discriminator_value_ != TYPE_MAP) {
        throw std::logic_error("Union Type not set to value map");
    }
    return ic_union_value_.map;
}

inline const MapType& Type::map() const {
    if (ic_discriminator_value_ != TYPE_MAP) {
        throw std::logic_error("Union Type not set to value map");
    }
    return ic_union_value_.map;
}

inline void Type::map(const MapType& a_value) {
    if (ic_discriminator_value_ != TYPE_MAP) {
        free_union_();
        ic_discriminator_value_ = TYPE_MAP;
        intercom::construct_at(&ic_union_value_.map, a_value);
    } else {
        ic_union_value_.map = a_value;
    }
}

inline void Type::map(MapType&& a_value) {
    if (ic_discriminator_value_ != TYPE_MAP) {
        free_union_();
        ic_discriminator_value_ = TYPE_MAP;
        intercom::construct_at(&ic_union_value_.map, std::move(a_value));
    } else {
        ic_union_value_.map = std::move(a_value);
    }
}

inline FixedType& Type::fixed() {
    if (ic_discriminator_value_ != TYPE_FIXED) {
        throw std::logic_error("Union Type not set to value fixed");
    }
    return ic_union_value_.fixed;
}

inline const FixedType& Type::fixed() const {
    if (ic_discriminator_value_ != TYPE_FIXED) {
        throw std::logic_error("Union Type not set to value fixed");
    }
    return ic_union_value_.fixed;
}

inline void Type::fixed(const FixedType& a_value) {
    if (ic_discriminator_value_ != TYPE_FIXED) {
        free_union_();
        ic_discriminator_value_ = TYPE_FIXED;
        intercom::construct_at(&ic_union_value_.fixed, a_value);
    } else {
        ic_union_value_.fixed = a_value;
    }
}

inline void Type::fixed(FixedType&& a_value) {
    if (ic_discriminator_value_ != TYPE_FIXED) {
        free_union_();
        ic_discriminator_value_ = TYPE_FIXED;
        intercom::construct_at(&ic_union_value_.fixed, std::move(a_value));
    } else {
        ic_union_value_.fixed = std::move(a_value);
    }
}

inline Path& Type::path() {
    if (ic_discriminator_value_ != TYPE_PATH) {
        throw std::logic_error("Union Type not set to value path");
    }
    return ic_union_value_.path;
}

inline const Path& Type::path() const {
    if (ic_discriminator_value_ != TYPE_PATH) {
        throw std::logic_error("Union Type not set to value path");
    }
    return ic_union_value_.path;
}

inline void Type::path(const Path& a_value) {
    if (ic_discriminator_value_ != TYPE_PATH) {
        free_union_();
        ic_discriminator_value_ = TYPE_PATH;
        intercom::construct_at(&ic_union_value_.path, a_value);
    } else {
        ic_union_value_.path = a_value;
    }
}

inline void Type::path(Path&& a_value) {
    if (ic_discriminator_value_ != TYPE_PATH) {
        free_union_();
        ic_discriminator_value_ = TYPE_PATH;
        intercom::construct_at(&ic_union_value_.path, std::move(a_value));
    } else {
        ic_union_value_.path = std::move(a_value);
    }
}

inline void Type::free_union_() {
    switch (ic_discriminator_value_) {
    case TYPE_ANY:
        std::destroy_at(&ic_union_value_.any);
        break;
    case TYPE_SEQUENCE:
        std::destroy_at(&ic_union_value_.sequence);
        break;
    case TYPE_STRING:
        std::destroy_at(&ic_union_value_.string);
        break;
    case TYPE_MAP:
        std::destroy_at(&ic_union_value_.map);
        break;
    case TYPE_FIXED:
        std::destroy_at(&ic_union_value_.fixed);
        break;
    case TYPE_PATH:
        std::destroy_at(&ic_union_value_.path);
        break;
    }
}

inline ArrayDeclarator::ArrayDeclarator(Ident a_ident, ::std::vector<Expr> a_bounds)
    : ident(std::move(a_ident)), bounds(std::move(a_bounds)) {}

inline bool ArrayDeclarator::operator<(const ArrayDeclarator& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    return this->bounds < a_other.bounds;
}

inline bool ArrayDeclarator::operator==(const ArrayDeclarator& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->bounds == a_other.bounds)) {
        return false;
    }
    return true;
}

inline Declarator::Declarator() {
    ic_discriminator_value_ = DECLARATOR_SIMPLE;
    intercom::construct_at(&ic_union_value_.simple, Ident{});
}

inline Declarator::Declarator(const Declarator& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case DECLARATOR_SIMPLE:
        intercom::construct_at(&ic_union_value_.simple, a_other.ic_union_value_.simple);
        break;
    case DECLARATOR_ARRAY:
        intercom::construct_at(&ic_union_value_.array, a_other.ic_union_value_.array);
        break;
    }
}

inline Declarator& Declarator::operator=(const Declarator& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case DECLARATOR_SIMPLE:
            intercom::construct_at(&ic_union_value_.simple, a_other.ic_union_value_.simple);
            break;
        case DECLARATOR_ARRAY:
            intercom::construct_at(&ic_union_value_.array, a_other.ic_union_value_.array);
            break;
        }
    }

    return *this;
}

inline Declarator::Declarator(Declarator&& a_other) noexcept : Declarator() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case DECLARATOR_SIMPLE:
        intercom::construct_at(&ic_union_value_.simple, std::move(a_other.ic_union_value_.simple));
        break;
    case DECLARATOR_ARRAY:
        intercom::construct_at(&ic_union_value_.array, std::move(a_other.ic_union_value_.array));
        break;
    }
}

inline Declarator& Declarator::operator=(Declarator&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case DECLARATOR_SIMPLE:
            intercom::construct_at(
                &ic_union_value_.simple, std::move(a_other.ic_union_value_.simple)
            );
            break;
        case DECLARATOR_ARRAY:
            intercom::construct_at(
                &ic_union_value_.array, std::move(a_other.ic_union_value_.array)
            );
            break;
        }
    }
    return *this;
}

inline Declarator::~Declarator() noexcept {
    free_union_();
}

inline bool Declarator::operator<(const Declarator& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case DECLARATOR_SIMPLE:
        return this->simple() < a_other.simple();
    case DECLARATOR_ARRAY:
        return this->array() < a_other.array();
    }
    return false;
}

inline bool Declarator::operator==(const Declarator& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case DECLARATOR_SIMPLE:
        return this->simple() == a_other.simple();
    case DECLARATOR_ARRAY:
        return this->array() == a_other.array();
    }
    return true;
}

inline void swap(Declarator& a_first, Declarator& a_second) noexcept {
    Declarator a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void Declarator::_d(DeclaratorKind discriminator) {
    switch (discriminator) {
    case DECLARATOR_SIMPLE:
        if (ic_discriminator_value_ != DECLARATOR_SIMPLE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.simple, Ident{});
        }
        break;
    case DECLARATOR_ARRAY:
        if (ic_discriminator_value_ != DECLARATOR_ARRAY) {
            free_union_();
            intercom::construct_at(&ic_union_value_.array, ArrayDeclarator{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union Declarator");
    }
    ic_discriminator_value_ = discriminator;
}

inline Ident& Declarator::simple() {
    if (ic_discriminator_value_ != DECLARATOR_SIMPLE) {
        throw std::logic_error("Union Declarator not set to value simple");
    }
    return ic_union_value_.simple;
}

inline const Ident& Declarator::simple() const {
    if (ic_discriminator_value_ != DECLARATOR_SIMPLE) {
        throw std::logic_error("Union Declarator not set to value simple");
    }
    return ic_union_value_.simple;
}

inline void Declarator::simple(const Ident& a_value) {
    if (ic_discriminator_value_ != DECLARATOR_SIMPLE) {
        free_union_();
        ic_discriminator_value_ = DECLARATOR_SIMPLE;
        intercom::construct_at(&ic_union_value_.simple, a_value);
    } else {
        ic_union_value_.simple = a_value;
    }
}

inline void Declarator::simple(Ident&& a_value) {
    if (ic_discriminator_value_ != DECLARATOR_SIMPLE) {
        free_union_();
        ic_discriminator_value_ = DECLARATOR_SIMPLE;
        intercom::construct_at(&ic_union_value_.simple, std::move(a_value));
    } else {
        ic_union_value_.simple = std::move(a_value);
    }
}

inline ArrayDeclarator& Declarator::array() {
    if (ic_discriminator_value_ != DECLARATOR_ARRAY) {
        throw std::logic_error("Union Declarator not set to value array");
    }
    return ic_union_value_.array;
}

inline const ArrayDeclarator& Declarator::array() const {
    if (ic_discriminator_value_ != DECLARATOR_ARRAY) {
        throw std::logic_error("Union Declarator not set to value array");
    }
    return ic_union_value_.array;
}

inline void Declarator::array(const ArrayDeclarator& a_value) {
    if (ic_discriminator_value_ != DECLARATOR_ARRAY) {
        free_union_();
        ic_discriminator_value_ = DECLARATOR_ARRAY;
        intercom::construct_at(&ic_union_value_.array, a_value);
    } else {
        ic_union_value_.array = a_value;
    }
}

inline void Declarator::array(ArrayDeclarator&& a_value) {
    if (ic_discriminator_value_ != DECLARATOR_ARRAY) {
        free_union_();
        ic_discriminator_value_ = DECLARATOR_ARRAY;
        intercom::construct_at(&ic_union_value_.array, std::move(a_value));
    } else {
        ic_union_value_.array = std::move(a_value);
    }
}

inline void Declarator::free_union_() {
    switch (ic_discriminator_value_) {
    case DECLARATOR_SIMPLE:
        std::destroy_at(&ic_union_value_.simple);
        break;
    case DECLARATOR_ARRAY:
        std::destroy_at(&ic_union_value_.array);
        break;
    }
}

inline AnnotationArg::AnnotationArg(::std::optional<Ident> a_ident, Span a_span, Expr a_value)
    : ident(std::move(a_ident)), span(std::move(a_span)), value(std::move(a_value)) {}

inline bool AnnotationArg::operator<(const AnnotationArg& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool AnnotationArg::operator==(const AnnotationArg& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline AnnotationAppl::AnnotationAppl(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationArg> a_args
)
    : ident(std::move(a_ident)), span(std::move(a_span)), args(std::move(a_args)) {}

inline bool AnnotationAppl::operator<(const AnnotationAppl& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->args < a_other.args;
}

inline bool AnnotationAppl::operator==(const AnnotationAppl& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->args == a_other.args)) {
        return false;
    }
    return true;
}

inline Stmt::Stmt(Ident a_ident, Span a_span, ::std::vector<AnnotationAppl> a_annotations)
    : ident(std::move(a_ident)), span(std::move(a_span)), annotations(std::move(a_annotations)) {}

inline bool Stmt::operator<(const Stmt& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    return this->annotations < a_other.annotations;
}

inline bool Stmt::operator==(const Stmt& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    return true;
}

inline AnnotationField::AnnotationField() {
    ic_discriminator_value_ = FIELD_DEFINITION;
    intercom::construct_at(&ic_union_value_.item, ::std::unique_ptr<Item>(new Item{}));
}

inline AnnotationField::AnnotationField(const AnnotationField& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case FIELD_DEFINITION:
        intercom::construct_at(&ic_union_value_.item, new Item(*a_other.ic_union_value_.item));
        break;
    case FIELD_MEMBER:
        intercom::construct_at(&ic_union_value_.member, new Field(*a_other.ic_union_value_.member));
        break;
    }
}

inline AnnotationField& AnnotationField::operator=(const AnnotationField& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case FIELD_DEFINITION:
            intercom::construct_at(&ic_union_value_.item, new Item(*a_other.ic_union_value_.item));
            break;
        case FIELD_MEMBER:
            intercom::construct_at(
                &ic_union_value_.member, new Field(*a_other.ic_union_value_.member)
            );
            break;
        }
    }

    return *this;
}

inline AnnotationField::AnnotationField(AnnotationField&& a_other) noexcept : AnnotationField() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case FIELD_DEFINITION:
        intercom::construct_at(&ic_union_value_.item, std::move(a_other.ic_union_value_.item));
        break;
    case FIELD_MEMBER:
        intercom::construct_at(&ic_union_value_.member, std::move(a_other.ic_union_value_.member));
        break;
    }
}

inline AnnotationField& AnnotationField::operator=(AnnotationField&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case FIELD_DEFINITION:
            intercom::construct_at(&ic_union_value_.item, std::move(a_other.ic_union_value_.item));
            break;
        case FIELD_MEMBER:
            intercom::construct_at(
                &ic_union_value_.member, std::move(a_other.ic_union_value_.member)
            );
            break;
        }
    }
    return *this;
}

inline AnnotationField::~AnnotationField() noexcept {
    free_union_();
}

inline bool AnnotationField::operator<(const AnnotationField& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case FIELD_DEFINITION:
        if (!this->item() || !a_other.item()) {
            return this->item() < a_other.item();
        }
        return *(this->item()) < *a_other.item();
    case FIELD_MEMBER:
        if (!this->member() || !a_other.member()) {
            return this->member() < a_other.member();
        }
        return *(this->member()) < *a_other.member();
    }
    return false;
}

inline bool AnnotationField::operator==(const AnnotationField& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case FIELD_DEFINITION:
        if (this->item() == a_other.item()) {
            return true;
        }
        if (!this->item() || !a_other.item()) {
            return false;
        }
        return *(this->item()) == *a_other.item();
    case FIELD_MEMBER:
        if (this->member() == a_other.member()) {
            return true;
        }
        if (!this->member() || !a_other.member()) {
            return false;
        }
        return *(this->member()) == *a_other.member();
    }
    return true;
}

inline void swap(AnnotationField& a_first, AnnotationField& a_second) noexcept {
    AnnotationField a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void AnnotationField::_d(AnnotationFieldKind discriminator) {
    switch (discriminator) {
    case FIELD_DEFINITION:
        if (ic_discriminator_value_ != FIELD_DEFINITION) {
            free_union_();
            intercom::construct_at(&ic_union_value_.item, ::std::unique_ptr<Item>(new Item{}));
        }
        break;
    case FIELD_MEMBER:
        if (ic_discriminator_value_ != FIELD_MEMBER) {
            free_union_();
            intercom::construct_at(&ic_union_value_.member, ::std::unique_ptr<Field>(new Field{}));
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union AnnotationField");
    }
    ic_discriminator_value_ = discriminator;
}

inline ::std::unique_ptr<Item>& AnnotationField::item() {
    if (ic_discriminator_value_ != FIELD_DEFINITION) {
        throw std::logic_error("Union AnnotationField not set to value item");
    }
    return ic_union_value_.item;
}

inline const ::std::unique_ptr<Item>& AnnotationField::item() const {
    if (ic_discriminator_value_ != FIELD_DEFINITION) {
        throw std::logic_error("Union AnnotationField not set to value item");
    }
    return ic_union_value_.item;
}

inline void AnnotationField::item(const ::std::unique_ptr<Item>& a_value) {
    if (ic_discriminator_value_ != FIELD_DEFINITION) {
        free_union_();
        ic_discriminator_value_ = FIELD_DEFINITION;
        intercom::construct_at(&ic_union_value_.item, new Item(*a_value));
    } else {
        ic_union_value_.item.reset(new Item(*a_value));
    }
}

inline void AnnotationField::item(::std::unique_ptr<Item>&& a_value) {
    if (ic_discriminator_value_ != FIELD_DEFINITION) {
        free_union_();
        ic_discriminator_value_ = FIELD_DEFINITION;
        intercom::construct_at(&ic_union_value_.item, std::move(a_value));
    } else {
        ic_union_value_.item = std::move(a_value);
    }
}

inline ::std::unique_ptr<Field>& AnnotationField::member() {
    if (ic_discriminator_value_ != FIELD_MEMBER) {
        throw std::logic_error("Union AnnotationField not set to value member");
    }
    return ic_union_value_.member;
}

inline const ::std::unique_ptr<Field>& AnnotationField::member() const {
    if (ic_discriminator_value_ != FIELD_MEMBER) {
        throw std::logic_error("Union AnnotationField not set to value member");
    }
    return ic_union_value_.member;
}

inline void AnnotationField::member(const ::std::unique_ptr<Field>& a_value) {
    if (ic_discriminator_value_ != FIELD_MEMBER) {
        free_union_();
        ic_discriminator_value_ = FIELD_MEMBER;
        intercom::construct_at(&ic_union_value_.member, new Field(*a_value));
    } else {
        ic_union_value_.member.reset(new Field(*a_value));
    }
}

inline void AnnotationField::member(::std::unique_ptr<Field>&& a_value) {
    if (ic_discriminator_value_ != FIELD_MEMBER) {
        free_union_();
        ic_discriminator_value_ = FIELD_MEMBER;
        intercom::construct_at(&ic_union_value_.member, std::move(a_value));
    } else {
        ic_union_value_.member = std::move(a_value);
    }
}

inline void AnnotationField::free_union_() {
    switch (ic_discriminator_value_) {
    case FIELD_DEFINITION:
        std::destroy_at(&ic_union_value_.item);
        break;
    case FIELD_MEMBER:
        std::destroy_at(&ic_union_value_.member);
        break;
    }
}

inline AnnotationDef::AnnotationDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<AnnotationField> a_params
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      params(std::move(a_params)) {}

inline bool AnnotationDef::operator<(const AnnotationDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->params < a_other.params;
}

inline bool AnnotationDef::operator==(const AnnotationDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->params == a_other.params)) {
        return false;
    }
    return true;
}

inline ModuleDef::ModuleDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Item> a_definitions
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      definitions(std::move(a_definitions)) {}

inline bool ModuleDef::operator<(const ModuleDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->definitions < a_other.definitions;
}

inline bool ModuleDef::operator==(const ModuleDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->definitions == a_other.definitions)) {
        return false;
    }
    return true;
}

inline Field::Field(::std::vector<Declarator> a_names, Type a_ty)
    : names(std::move(a_names)), ty(std::move(a_ty)) {}

inline bool Field::operator<(const Field& a_other) const {
    if (this->names < a_other.names) {
        return true;
    }
    if (a_other.names < this->names) {
        return false;
    }
    return this->ty < a_other.ty;
}

inline bool Field::operator==(const Field& a_other) const {
    if (!(this->names == a_other.names)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    return true;
}

inline StructDef::StructDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Field> a_members,
    ::std::optional<Path> a_parent
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      members(std::move(a_members)),
      parent(std::move(a_parent)) {}

inline bool StructDef::operator<(const StructDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->members < a_other.members) {
        return true;
    }
    if (a_other.members < this->members) {
        return false;
    }
    return this->parent < a_other.parent;
}

inline bool StructDef::operator==(const StructDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->members == a_other.members)) {
        return false;
    }
    if (!(this->parent == a_other.parent)) {
        return false;
    }
    return true;
}

inline Discriminator::Discriminator(::std::vector<AnnotationAppl> a_annotations, Type a_ty)
    : annotations(std::move(a_annotations)), ty(std::move(a_ty)) {}

inline bool Discriminator::operator<(const Discriminator& a_other) const {
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->ty < a_other.ty;
}

inline bool Discriminator::operator==(const Discriminator& a_other) const {
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    return true;
}

inline Label::Label() {
    ic_discriminator_value_ = LABEL_CASE;
    intercom::construct_at(&ic_union_value_.case_, Expr{});
}

inline Label::Label(const Label& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case LABEL_CASE:
        intercom::construct_at(&ic_union_value_.case_, a_other.ic_union_value_.case_);
        break;
    case LABEL_DEFAULT:
        intercom::construct_at(&ic_union_value_.default_, a_other.ic_union_value_.default_);
        break;
    }
}

inline Label& Label::operator=(const Label& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case LABEL_CASE:
            intercom::construct_at(&ic_union_value_.case_, a_other.ic_union_value_.case_);
            break;
        case LABEL_DEFAULT:
            intercom::construct_at(&ic_union_value_.default_, a_other.ic_union_value_.default_);
            break;
        }
    }

    return *this;
}

inline Label::Label(Label&& a_other) noexcept : Label() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case LABEL_CASE:
        intercom::construct_at(&ic_union_value_.case_, std::move(a_other.ic_union_value_.case_));
        break;
    case LABEL_DEFAULT:
        intercom::construct_at(
            &ic_union_value_.default_, std::move(a_other.ic_union_value_.default_)
        );
        break;
    }
}

inline Label& Label::operator=(Label&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case LABEL_CASE:
            intercom::construct_at(
                &ic_union_value_.case_, std::move(a_other.ic_union_value_.case_)
            );
            break;
        case LABEL_DEFAULT:
            intercom::construct_at(
                &ic_union_value_.default_, std::move(a_other.ic_union_value_.default_)
            );
            break;
        }
    }
    return *this;
}

inline Label::~Label() noexcept {
    free_union_();
}

inline bool Label::operator<(const Label& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case LABEL_CASE:
        return this->case_() < a_other.case_();
    case LABEL_DEFAULT:
        return this->default_() < a_other.default_();
    }
    return false;
}

inline bool Label::operator==(const Label& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case LABEL_CASE:
        return this->case_() == a_other.case_();
    case LABEL_DEFAULT:
        return this->default_() == a_other.default_();
    }
    return true;
}

inline void swap(Label& a_first, Label& a_second) noexcept {
    Label a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void Label::_d(LabelKind discriminator) {
    switch (discriminator) {
    case LABEL_CASE:
        if (ic_discriminator_value_ != LABEL_CASE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.case_, Expr{});
        }
        break;
    case LABEL_DEFAULT:
        if (ic_discriminator_value_ != LABEL_DEFAULT) {
            free_union_();
            intercom::construct_at(&ic_union_value_.default_, Empty{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union Label");
    }
    ic_discriminator_value_ = discriminator;
}

inline Expr& Label::case_() {
    if (ic_discriminator_value_ != LABEL_CASE) {
        throw std::logic_error("Union Label not set to value case_");
    }
    return ic_union_value_.case_;
}

inline const Expr& Label::case_() const {
    if (ic_discriminator_value_ != LABEL_CASE) {
        throw std::logic_error("Union Label not set to value case_");
    }
    return ic_union_value_.case_;
}

inline void Label::case_(const Expr& a_value) {
    if (ic_discriminator_value_ != LABEL_CASE) {
        free_union_();
        ic_discriminator_value_ = LABEL_CASE;
        intercom::construct_at(&ic_union_value_.case_, a_value);
    } else {
        ic_union_value_.case_ = a_value;
    }
}

inline void Label::case_(Expr&& a_value) {
    if (ic_discriminator_value_ != LABEL_CASE) {
        free_union_();
        ic_discriminator_value_ = LABEL_CASE;
        intercom::construct_at(&ic_union_value_.case_, std::move(a_value));
    } else {
        ic_union_value_.case_ = std::move(a_value);
    }
}

inline Empty& Label::default_() {
    if (ic_discriminator_value_ != LABEL_DEFAULT) {
        throw std::logic_error("Union Label not set to value default_");
    }
    return ic_union_value_.default_;
}

inline const Empty& Label::default_() const {
    if (ic_discriminator_value_ != LABEL_DEFAULT) {
        throw std::logic_error("Union Label not set to value default_");
    }
    return ic_union_value_.default_;
}

inline void Label::default_(const Empty& a_value) {
    if (ic_discriminator_value_ != LABEL_DEFAULT) {
        free_union_();
        ic_discriminator_value_ = LABEL_DEFAULT;
        intercom::construct_at(&ic_union_value_.default_, a_value);
    } else {
        ic_union_value_.default_ = a_value;
    }
}

inline void Label::default_(Empty&& a_value) {
    if (ic_discriminator_value_ != LABEL_DEFAULT) {
        free_union_();
        ic_discriminator_value_ = LABEL_DEFAULT;
        intercom::construct_at(&ic_union_value_.default_, std::move(a_value));
    } else {
        ic_union_value_.default_ = std::move(a_value);
    }
}

inline void Label::free_union_() {
    switch (ic_discriminator_value_) {
    case LABEL_CASE:
        std::destroy_at(&ic_union_value_.case_);
        break;
    case LABEL_DEFAULT:
        std::destroy_at(&ic_union_value_.default_);
        break;
    }
}

inline UnionMember::UnionMember() : ty{new Type{}} {}

inline UnionMember::UnionMember(const UnionMember& a_other)
    : ty(!a_other.ty ? nullptr : std::unique_ptr<Type>(new Type(*a_other.ty))),
      decl(std::move(a_other.decl)) {}

inline UnionMember& UnionMember::operator=(const UnionMember& a_other) {
    UnionMember a_copy(a_other);
    swap(*this, a_copy);
    return *this;
}

inline UnionMember::UnionMember(::std::unique_ptr<Type> a_ty, Declarator a_decl)
    : ty(!a_ty ? nullptr : std::unique_ptr<Type>(new Type(*a_ty))), decl(std::move(a_decl)) {}

inline UnionMember::UnionMember(Type a_ty, Declarator a_decl)
    : ty(std::unique_ptr<Type>(new Type(a_ty))), decl(std::move(a_decl)) {}

inline bool UnionMember::operator<(const UnionMember& a_other) const {
    if (!this->ty || !a_other.ty) {
        if (this->ty != a_other.ty) {
            return this->ty < a_other.ty;
        }
    } else {
        if (*(this->ty) < *a_other.ty) {
            return true;
        }
        if (*a_other.ty < *(this->ty)) {
            return false;
        }
    }
    return this->decl < a_other.decl;
}

inline bool UnionMember::operator==(const UnionMember& a_other) const {
    if (!(this->ty == a_other.ty)) {
        if (!this->ty || !a_other.ty) {
            return false;
        }
        if (!(*this->ty == *a_other.ty)) {
            return false;
        }
    }
    if (!(this->decl == a_other.decl)) {
        return false;
    }
    return true;
}

inline void swap(UnionMember& a_first, UnionMember& a_second) noexcept {
    using std::swap;
    swap(a_first.ty, a_second.ty);
    swap(a_first.decl, a_second.decl);
}

inline UnionNull::UnionNull(Span a_span) : span(std::move(a_span)) {}

inline bool UnionNull::operator<(const UnionNull& a_other) const {
    return this->span < a_other.span;
}

inline bool UnionNull::operator==(const UnionNull& a_other) const {
    if (!(this->span == a_other.span)) {
        return false;
    }
    return true;
}

inline UnionElement::UnionElement() {
    ic_discriminator_value_ = ELEMENT_MEMBER;
    intercom::construct_at(&ic_union_value_.member, UnionMember{});
}

inline UnionElement::UnionElement(const UnionElement& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case ELEMENT_MEMBER:
        intercom::construct_at(&ic_union_value_.member, a_other.ic_union_value_.member);
        break;
    case ELEMENT_NULL:
        intercom::construct_at(&ic_union_value_.null, a_other.ic_union_value_.null);
        break;
    }
}

inline UnionElement& UnionElement::operator=(const UnionElement& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case ELEMENT_MEMBER:
            intercom::construct_at(&ic_union_value_.member, a_other.ic_union_value_.member);
            break;
        case ELEMENT_NULL:
            intercom::construct_at(&ic_union_value_.null, a_other.ic_union_value_.null);
            break;
        }
    }

    return *this;
}

inline UnionElement::UnionElement(UnionElement&& a_other) noexcept : UnionElement() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case ELEMENT_MEMBER:
        intercom::construct_at(&ic_union_value_.member, std::move(a_other.ic_union_value_.member));
        break;
    case ELEMENT_NULL:
        intercom::construct_at(&ic_union_value_.null, std::move(a_other.ic_union_value_.null));
        break;
    }
}

inline UnionElement& UnionElement::operator=(UnionElement&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case ELEMENT_MEMBER:
            intercom::construct_at(
                &ic_union_value_.member, std::move(a_other.ic_union_value_.member)
            );
            break;
        case ELEMENT_NULL:
            intercom::construct_at(&ic_union_value_.null, std::move(a_other.ic_union_value_.null));
            break;
        }
    }
    return *this;
}

inline UnionElement::~UnionElement() noexcept {
    free_union_();
}

inline bool UnionElement::operator<(const UnionElement& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case ELEMENT_MEMBER:
        return this->member() < a_other.member();
    case ELEMENT_NULL:
        return this->null() < a_other.null();
    }
    return false;
}

inline bool UnionElement::operator==(const UnionElement& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case ELEMENT_MEMBER:
        return this->member() == a_other.member();
    case ELEMENT_NULL:
        return this->null() == a_other.null();
    }
    return true;
}

inline void swap(UnionElement& a_first, UnionElement& a_second) noexcept {
    UnionElement a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void UnionElement::_d(UnionElementKind discriminator) {
    switch (discriminator) {
    case ELEMENT_MEMBER:
        if (ic_discriminator_value_ != ELEMENT_MEMBER) {
            free_union_();
            intercom::construct_at(&ic_union_value_.member, UnionMember{});
        }
        break;
    case ELEMENT_NULL:
        if (ic_discriminator_value_ != ELEMENT_NULL) {
            free_union_();
            intercom::construct_at(&ic_union_value_.null, UnionNull{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union UnionElement");
    }
    ic_discriminator_value_ = discriminator;
}

inline UnionMember& UnionElement::member() {
    if (ic_discriminator_value_ != ELEMENT_MEMBER) {
        throw std::logic_error("Union UnionElement not set to value member");
    }
    return ic_union_value_.member;
}

inline const UnionMember& UnionElement::member() const {
    if (ic_discriminator_value_ != ELEMENT_MEMBER) {
        throw std::logic_error("Union UnionElement not set to value member");
    }
    return ic_union_value_.member;
}

inline void UnionElement::member(const UnionMember& a_value) {
    if (ic_discriminator_value_ != ELEMENT_MEMBER) {
        free_union_();
        ic_discriminator_value_ = ELEMENT_MEMBER;
        intercom::construct_at(&ic_union_value_.member, a_value);
    } else {
        ic_union_value_.member = a_value;
    }
}

inline void UnionElement::member(UnionMember&& a_value) {
    if (ic_discriminator_value_ != ELEMENT_MEMBER) {
        free_union_();
        ic_discriminator_value_ = ELEMENT_MEMBER;
        intercom::construct_at(&ic_union_value_.member, std::move(a_value));
    } else {
        ic_union_value_.member = std::move(a_value);
    }
}

inline UnionNull& UnionElement::null() {
    if (ic_discriminator_value_ != ELEMENT_NULL) {
        throw std::logic_error("Union UnionElement not set to value null");
    }
    return ic_union_value_.null;
}

inline const UnionNull& UnionElement::null() const {
    if (ic_discriminator_value_ != ELEMENT_NULL) {
        throw std::logic_error("Union UnionElement not set to value null");
    }
    return ic_union_value_.null;
}

inline void UnionElement::null(const UnionNull& a_value) {
    if (ic_discriminator_value_ != ELEMENT_NULL) {
        free_union_();
        ic_discriminator_value_ = ELEMENT_NULL;
        intercom::construct_at(&ic_union_value_.null, a_value);
    } else {
        ic_union_value_.null = a_value;
    }
}

inline void UnionElement::null(UnionNull&& a_value) {
    if (ic_discriminator_value_ != ELEMENT_NULL) {
        free_union_();
        ic_discriminator_value_ = ELEMENT_NULL;
        intercom::construct_at(&ic_union_value_.null, std::move(a_value));
    } else {
        ic_union_value_.null = std::move(a_value);
    }
}

inline void UnionElement::free_union_() {
    switch (ic_discriminator_value_) {
    case ELEMENT_MEMBER:
        std::destroy_at(&ic_union_value_.member);
        break;
    case ELEMENT_NULL:
        std::destroy_at(&ic_union_value_.null);
        break;
    }
}

inline UnionField::UnionField(
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Label> a_labels,
    UnionElement a_field
)
    : annotations(std::move(a_annotations)),
      labels(std::move(a_labels)),
      field(std::move(a_field)) {}

inline bool UnionField::operator<(const UnionField& a_other) const {
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->labels < a_other.labels) {
        return true;
    }
    if (a_other.labels < this->labels) {
        return false;
    }
    return this->field < a_other.field;
}

inline bool UnionField::operator==(const UnionField& a_other) const {
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->labels == a_other.labels)) {
        return false;
    }
    if (!(this->field == a_other.field)) {
        return false;
    }
    return true;
}

inline UnionDef::UnionDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    Discriminator a_disc,
    ::std::vector<UnionField> a_fields
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      disc(std::move(a_disc)),
      fields(std::move(a_fields)) {}

inline bool UnionDef::operator<(const UnionDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->disc < a_other.disc) {
        return true;
    }
    if (a_other.disc < this->disc) {
        return false;
    }
    return this->fields < a_other.fields;
}

inline bool UnionDef::operator==(const UnionDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->disc == a_other.disc)) {
        return false;
    }
    if (!(this->fields == a_other.fields)) {
        return false;
    }
    return true;
}

inline ConstDef::ConstDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    Declarator a_decl,
    Type a_ty,
    Expr a_value
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      decl(std::move(a_decl)),
      ty(std::move(a_ty)),
      value(std::move(a_value)) {}

inline bool ConstDef::operator<(const ConstDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->decl < a_other.decl) {
        return true;
    }
    if (a_other.decl < this->decl) {
        return false;
    }
    if (this->ty < a_other.ty) {
        return true;
    }
    if (a_other.ty < this->ty) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool ConstDef::operator==(const ConstDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->decl == a_other.decl)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline Enumerator::Enumerator(
    Ident a_ident,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::optional<Expr> a_value
)
    : ident(std::move(a_ident)), annotations(std::move(a_annotations)), value(std::move(a_value)) {}

inline bool Enumerator::operator<(const Enumerator& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool Enumerator::operator==(const Enumerator& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline EnumDef::EnumDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Enumerator> a_fields
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      fields(std::move(a_fields)) {}

inline bool EnumDef::operator<(const EnumDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->fields < a_other.fields;
}

inline bool EnumDef::operator==(const EnumDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->fields == a_other.fields)) {
        return false;
    }
    return true;
}

inline ExceptDef::ExceptDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Field> a_members
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      members(std::move(a_members)) {}

inline bool ExceptDef::operator<(const ExceptDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->members < a_other.members;
}

inline bool ExceptDef::operator==(const ExceptDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->members == a_other.members)) {
        return false;
    }
    return true;
}

inline AliasDef::AliasDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Declarator> a_decl,
    Type a_ty
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      decl(std::move(a_decl)),
      ty(std::move(a_ty)) {}

inline bool AliasDef::operator<(const AliasDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->decl < a_other.decl) {
        return true;
    }
    if (a_other.decl < this->decl) {
        return false;
    }
    return this->ty < a_other.ty;
}

inline bool AliasDef::operator==(const AliasDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->decl == a_other.decl)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    return true;
}

inline Bit::Bit(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::optional<Expr> a_value
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      value(std::move(a_value)) {}

inline bool Bit::operator<(const Bit& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->value < a_other.value;
}

inline bool Bit::operator==(const Bit& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->value == a_other.value)) {
        return false;
    }
    return true;
}

inline BitmaskDef::BitmaskDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<Bit> a_bits
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      bits(std::move(a_bits)) {}

inline bool BitmaskDef::operator<(const BitmaskDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->bits < a_other.bits;
}

inline bool BitmaskDef::operator==(const BitmaskDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->bits == a_other.bits)) {
        return false;
    }
    return true;
}

inline Bitfield::Bitfield(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    Expr a_size
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      size(std::move(a_size)) {}

inline bool Bitfield::operator<(const Bitfield& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->size < a_other.size;
}

inline bool Bitfield::operator==(const Bitfield& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->size == a_other.size)) {
        return false;
    }
    return true;
}

inline BitsetDef::BitsetDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::optional<Path> a_parent,
    ::std::vector<Bitfield> a_fields
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      parent(std::move(a_parent)),
      fields(std::move(a_fields)) {}

inline bool BitsetDef::operator<(const BitsetDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->parent < a_other.parent) {
        return true;
    }
    if (a_other.parent < this->parent) {
        return false;
    }
    return this->fields < a_other.fields;
}

inline bool BitsetDef::operator==(const BitsetDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->parent == a_other.parent)) {
        return false;
    }
    if (!(this->fields == a_other.fields)) {
        return false;
    }
    return true;
}

inline Attribute::Attribute(Ident a_ident, Type a_ty, ::std::optional<Span> a_readonly)
    : ident(std::move(a_ident)), ty(std::move(a_ty)), readonly(std::move(a_readonly)) {}

inline bool Attribute::operator<(const Attribute& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->ty < a_other.ty) {
        return true;
    }
    if (a_other.ty < this->ty) {
        return false;
    }
    return this->readonly < a_other.readonly;
}

inline bool Attribute::operator==(const Attribute& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    if (!(this->readonly == a_other.readonly)) {
        return false;
    }
    return true;
}

inline Param::Param(Ident a_ident, Type a_ty, ::std::optional<ParamKind> a_kind)
    : ident(std::move(a_ident)), ty(std::move(a_ty)), kind(std::move(a_kind)) {}

inline bool Param::operator<(const Param& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->ty < a_other.ty) {
        return true;
    }
    if (a_other.ty < this->ty) {
        return false;
    }
    return this->kind < a_other.kind;
}

inline bool Param::operator==(const Param& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    if (!(this->kind == a_other.kind)) {
        return false;
    }
    return true;
}

inline Prototype::Prototype(
    Ident a_ident,
    Type a_ret,
    ::std::vector<Param> a_params,
    ::std::vector<Path> a_raises,
    ::std::optional<Span> a_oneway
)
    : ident(std::move(a_ident)),
      ret(std::move(a_ret)),
      params(std::move(a_params)),
      raises(std::move(a_raises)),
      oneway(std::move(a_oneway)) {}

inline bool Prototype::operator<(const Prototype& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->ret < a_other.ret) {
        return true;
    }
    if (a_other.ret < this->ret) {
        return false;
    }
    if (this->params < a_other.params) {
        return true;
    }
    if (a_other.params < this->params) {
        return false;
    }
    if (this->raises < a_other.raises) {
        return true;
    }
    if (a_other.raises < this->raises) {
        return false;
    }
    return this->oneway < a_other.oneway;
}

inline bool Prototype::operator==(const Prototype& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->ret == a_other.ret)) {
        return false;
    }
    if (!(this->params == a_other.params)) {
        return false;
    }
    if (!(this->raises == a_other.raises)) {
        return false;
    }
    if (!(this->oneway == a_other.oneway)) {
        return false;
    }
    return true;
}

inline InterfaceDef::InterfaceDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<InterfaceMember> a_members,
    ::std::vector<Path> a_inherits,
    ::std::optional<Span> a_local
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      members(std::move(a_members)),
      inherits(std::move(a_inherits)),
      local(std::move(a_local)) {}

inline bool InterfaceDef::operator<(const InterfaceDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->members < a_other.members) {
        return true;
    }
    if (a_other.members < this->members) {
        return false;
    }
    if (this->inherits < a_other.inherits) {
        return true;
    }
    if (a_other.inherits < this->inherits) {
        return false;
    }
    return this->local < a_other.local;
}

inline bool InterfaceDef::operator==(const InterfaceDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->members == a_other.members)) {
        return false;
    }
    if (!(this->inherits == a_other.inherits)) {
        return false;
    }
    if (!(this->local == a_other.local)) {
        return false;
    }
    return true;
}

inline ValueMember::ValueMember(Ident a_ident, Type a_ty, ::std::optional<Span> a_public_)
    : ident(std::move(a_ident)), ty(std::move(a_ty)), public_(std::move(a_public_)) {}

inline bool ValueMember::operator<(const ValueMember& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->ty < a_other.ty) {
        return true;
    }
    if (a_other.ty < this->ty) {
        return false;
    }
    return this->public_ < a_other.public_;
}

inline bool ValueMember::operator==(const ValueMember& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->ty == a_other.ty)) {
        return false;
    }
    if (!(this->public_ == a_other.public_)) {
        return false;
    }
    return true;
}

inline ValuetypeDef::ValuetypeDef(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    ::std::vector<ValueMember> a_members,
    ::std::vector<Prototype> a_prototypes,
    ::std::optional<Path> a_inherits,
    ::std::optional<Path> a_supports
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)),
      members(std::move(a_members)),
      prototypes(std::move(a_prototypes)),
      inherits(std::move(a_inherits)),
      supports(std::move(a_supports)) {}

inline bool ValuetypeDef::operator<(const ValuetypeDef& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    if (this->members < a_other.members) {
        return true;
    }
    if (a_other.members < this->members) {
        return false;
    }
    if (this->prototypes < a_other.prototypes) {
        return true;
    }
    if (a_other.prototypes < this->prototypes) {
        return false;
    }
    if (this->inherits < a_other.inherits) {
        return true;
    }
    if (a_other.inherits < this->inherits) {
        return false;
    }
    return this->supports < a_other.supports;
}

inline bool ValuetypeDef::operator==(const ValuetypeDef& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->members == a_other.members)) {
        return false;
    }
    if (!(this->prototypes == a_other.prototypes)) {
        return false;
    }
    if (!(this->inherits == a_other.inherits)) {
        return false;
    }
    if (!(this->supports == a_other.supports)) {
        return false;
    }
    return true;
}

inline Decl::Decl(
    Ident a_ident,
    Span a_span,
    ::std::vector<AnnotationAppl> a_annotations,
    DeclKind a_kind
)
    : Stmt(std::move(a_ident), std::move(a_span), std::move(a_annotations)), kind(a_kind) {}

inline bool Decl::operator<(const Decl& a_other) const {
    if (this->ident < a_other.ident) {
        return true;
    }
    if (a_other.ident < this->ident) {
        return false;
    }
    if (this->span < a_other.span) {
        return true;
    }
    if (a_other.span < this->span) {
        return false;
    }
    if (this->annotations < a_other.annotations) {
        return true;
    }
    if (a_other.annotations < this->annotations) {
        return false;
    }
    return this->kind < a_other.kind;
}

inline bool Decl::operator==(const Decl& a_other) const {
    if (!(this->ident == a_other.ident)) {
        return false;
    }
    if (!(this->span == a_other.span)) {
        return false;
    }
    if (!(this->annotations == a_other.annotations)) {
        return false;
    }
    if (!(this->kind == a_other.kind)) {
        return false;
    }
    return true;
}

inline Item::Item() {
    ic_discriminator_value_ = ITEM_ANNOTATION;
    intercom::construct_at(&ic_union_value_.annotation_value, AnnotationDef{});
}

inline Item::Item(const Item& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case ITEM_ANNOTATION:
        intercom::construct_at(
            &ic_union_value_.annotation_value, a_other.ic_union_value_.annotation_value
        );
        break;
    case ITEM_MODULE:
        intercom::construct_at(&ic_union_value_.module_value, a_other.ic_union_value_.module_value);
        break;
    case ITEM_STRUCT:
        intercom::construct_at(&ic_union_value_.struct_value, a_other.ic_union_value_.struct_value);
        break;
    case ITEM_UNION:
        intercom::construct_at(&ic_union_value_.union_value, a_other.ic_union_value_.union_value);
        break;
    case ITEM_ENUM:
        intercom::construct_at(&ic_union_value_.enum_value, a_other.ic_union_value_.enum_value);
        break;
    case ITEM_EXCEPTION:
        intercom::construct_at(
            &ic_union_value_.exception_value, a_other.ic_union_value_.exception_value
        );
        break;
    case ITEM_BITMASK:
        intercom::construct_at(
            &ic_union_value_.bitmask_value, a_other.ic_union_value_.bitmask_value
        );
        break;
    case ITEM_BITSET:
        intercom::construct_at(&ic_union_value_.bitset_value, a_other.ic_union_value_.bitset_value);
        break;
    case ITEM_CONST:
        intercom::construct_at(&ic_union_value_.const_value, a_other.ic_union_value_.const_value);
        break;
    case ITEM_TYPEDEF:
        intercom::construct_at(&ic_union_value_.alias_value, a_other.ic_union_value_.alias_value);
        break;
    case ITEM_INTERFACE:
        intercom::construct_at(
            &ic_union_value_.interface_value, a_other.ic_union_value_.interface_value
        );
        break;
    case ITEM_VALUETYPE:
        intercom::construct_at(
            &ic_union_value_.valuetype_value, a_other.ic_union_value_.valuetype_value
        );
        break;
    case ITEM_DECL:
        intercom::construct_at(&ic_union_value_.decl_value, a_other.ic_union_value_.decl_value);
        break;
    }
}

inline Item& Item::operator=(const Item& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case ITEM_ANNOTATION:
            intercom::construct_at(
                &ic_union_value_.annotation_value, a_other.ic_union_value_.annotation_value
            );
            break;
        case ITEM_MODULE:
            intercom::construct_at(
                &ic_union_value_.module_value, a_other.ic_union_value_.module_value
            );
            break;
        case ITEM_STRUCT:
            intercom::construct_at(
                &ic_union_value_.struct_value, a_other.ic_union_value_.struct_value
            );
            break;
        case ITEM_UNION:
            intercom::construct_at(
                &ic_union_value_.union_value, a_other.ic_union_value_.union_value
            );
            break;
        case ITEM_ENUM:
            intercom::construct_at(&ic_union_value_.enum_value, a_other.ic_union_value_.enum_value);
            break;
        case ITEM_EXCEPTION:
            intercom::construct_at(
                &ic_union_value_.exception_value, a_other.ic_union_value_.exception_value
            );
            break;
        case ITEM_BITMASK:
            intercom::construct_at(
                &ic_union_value_.bitmask_value, a_other.ic_union_value_.bitmask_value
            );
            break;
        case ITEM_BITSET:
            intercom::construct_at(
                &ic_union_value_.bitset_value, a_other.ic_union_value_.bitset_value
            );
            break;
        case ITEM_CONST:
            intercom::construct_at(
                &ic_union_value_.const_value, a_other.ic_union_value_.const_value
            );
            break;
        case ITEM_TYPEDEF:
            intercom::construct_at(
                &ic_union_value_.alias_value, a_other.ic_union_value_.alias_value
            );
            break;
        case ITEM_INTERFACE:
            intercom::construct_at(
                &ic_union_value_.interface_value, a_other.ic_union_value_.interface_value
            );
            break;
        case ITEM_VALUETYPE:
            intercom::construct_at(
                &ic_union_value_.valuetype_value, a_other.ic_union_value_.valuetype_value
            );
            break;
        case ITEM_DECL:
            intercom::construct_at(&ic_union_value_.decl_value, a_other.ic_union_value_.decl_value);
            break;
        }
    }

    return *this;
}

inline Item::Item(Item&& a_other) noexcept : Item() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case ITEM_ANNOTATION:
        intercom::construct_at(
            &ic_union_value_.annotation_value, std::move(a_other.ic_union_value_.annotation_value)
        );
        break;
    case ITEM_MODULE:
        intercom::construct_at(
            &ic_union_value_.module_value, std::move(a_other.ic_union_value_.module_value)
        );
        break;
    case ITEM_STRUCT:
        intercom::construct_at(
            &ic_union_value_.struct_value, std::move(a_other.ic_union_value_.struct_value)
        );
        break;
    case ITEM_UNION:
        intercom::construct_at(
            &ic_union_value_.union_value, std::move(a_other.ic_union_value_.union_value)
        );
        break;
    case ITEM_ENUM:
        intercom::construct_at(
            &ic_union_value_.enum_value, std::move(a_other.ic_union_value_.enum_value)
        );
        break;
    case ITEM_EXCEPTION:
        intercom::construct_at(
            &ic_union_value_.exception_value, std::move(a_other.ic_union_value_.exception_value)
        );
        break;
    case ITEM_BITMASK:
        intercom::construct_at(
            &ic_union_value_.bitmask_value, std::move(a_other.ic_union_value_.bitmask_value)
        );
        break;
    case ITEM_BITSET:
        intercom::construct_at(
            &ic_union_value_.bitset_value, std::move(a_other.ic_union_value_.bitset_value)
        );
        break;
    case ITEM_CONST:
        intercom::construct_at(
            &ic_union_value_.const_value, std::move(a_other.ic_union_value_.const_value)
        );
        break;
    case ITEM_TYPEDEF:
        intercom::construct_at(
            &ic_union_value_.alias_value, std::move(a_other.ic_union_value_.alias_value)
        );
        break;
    case ITEM_INTERFACE:
        intercom::construct_at(
            &ic_union_value_.interface_value, std::move(a_other.ic_union_value_.interface_value)
        );
        break;
    case ITEM_VALUETYPE:
        intercom::construct_at(
            &ic_union_value_.valuetype_value, std::move(a_other.ic_union_value_.valuetype_value)
        );
        break;
    case ITEM_DECL:
        intercom::construct_at(
            &ic_union_value_.decl_value, std::move(a_other.ic_union_value_.decl_value)
        );
        break;
    }
}

inline Item& Item::operator=(Item&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case ITEM_ANNOTATION:
            intercom::construct_at(
                &ic_union_value_.annotation_value,
                std::move(a_other.ic_union_value_.annotation_value)
            );
            break;
        case ITEM_MODULE:
            intercom::construct_at(
                &ic_union_value_.module_value, std::move(a_other.ic_union_value_.module_value)
            );
            break;
        case ITEM_STRUCT:
            intercom::construct_at(
                &ic_union_value_.struct_value, std::move(a_other.ic_union_value_.struct_value)
            );
            break;
        case ITEM_UNION:
            intercom::construct_at(
                &ic_union_value_.union_value, std::move(a_other.ic_union_value_.union_value)
            );
            break;
        case ITEM_ENUM:
            intercom::construct_at(
                &ic_union_value_.enum_value, std::move(a_other.ic_union_value_.enum_value)
            );
            break;
        case ITEM_EXCEPTION:
            intercom::construct_at(
                &ic_union_value_.exception_value, std::move(a_other.ic_union_value_.exception_value)
            );
            break;
        case ITEM_BITMASK:
            intercom::construct_at(
                &ic_union_value_.bitmask_value, std::move(a_other.ic_union_value_.bitmask_value)
            );
            break;
        case ITEM_BITSET:
            intercom::construct_at(
                &ic_union_value_.bitset_value, std::move(a_other.ic_union_value_.bitset_value)
            );
            break;
        case ITEM_CONST:
            intercom::construct_at(
                &ic_union_value_.const_value, std::move(a_other.ic_union_value_.const_value)
            );
            break;
        case ITEM_TYPEDEF:
            intercom::construct_at(
                &ic_union_value_.alias_value, std::move(a_other.ic_union_value_.alias_value)
            );
            break;
        case ITEM_INTERFACE:
            intercom::construct_at(
                &ic_union_value_.interface_value, std::move(a_other.ic_union_value_.interface_value)
            );
            break;
        case ITEM_VALUETYPE:
            intercom::construct_at(
                &ic_union_value_.valuetype_value, std::move(a_other.ic_union_value_.valuetype_value)
            );
            break;
        case ITEM_DECL:
            intercom::construct_at(
                &ic_union_value_.decl_value, std::move(a_other.ic_union_value_.decl_value)
            );
            break;
        }
    }
    return *this;
}

inline Item::~Item() noexcept {
    free_union_();
}

inline bool Item::operator<(const Item& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case ITEM_ANNOTATION:
        return this->annotation_value() < a_other.annotation_value();
    case ITEM_MODULE:
        return this->module_value() < a_other.module_value();
    case ITEM_STRUCT:
        return this->struct_value() < a_other.struct_value();
    case ITEM_UNION:
        return this->union_value() < a_other.union_value();
    case ITEM_ENUM:
        return this->enum_value() < a_other.enum_value();
    case ITEM_EXCEPTION:
        return this->exception_value() < a_other.exception_value();
    case ITEM_BITMASK:
        return this->bitmask_value() < a_other.bitmask_value();
    case ITEM_BITSET:
        return this->bitset_value() < a_other.bitset_value();
    case ITEM_CONST:
        return this->const_value() < a_other.const_value();
    case ITEM_TYPEDEF:
        return this->alias_value() < a_other.alias_value();
    case ITEM_INTERFACE:
        return this->interface_value() < a_other.interface_value();
    case ITEM_VALUETYPE:
        return this->valuetype_value() < a_other.valuetype_value();
    case ITEM_DECL:
        return this->decl_value() < a_other.decl_value();
    }
    return false;
}

inline bool Item::operator==(const Item& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case ITEM_ANNOTATION:
        return this->annotation_value() == a_other.annotation_value();
    case ITEM_MODULE:
        return this->module_value() == a_other.module_value();
    case ITEM_STRUCT:
        return this->struct_value() == a_other.struct_value();
    case ITEM_UNION:
        return this->union_value() == a_other.union_value();
    case ITEM_ENUM:
        return this->enum_value() == a_other.enum_value();
    case ITEM_EXCEPTION:
        return this->exception_value() == a_other.exception_value();
    case ITEM_BITMASK:
        return this->bitmask_value() == a_other.bitmask_value();
    case ITEM_BITSET:
        return this->bitset_value() == a_other.bitset_value();
    case ITEM_CONST:
        return this->const_value() == a_other.const_value();
    case ITEM_TYPEDEF:
        return this->alias_value() == a_other.alias_value();
    case ITEM_INTERFACE:
        return this->interface_value() == a_other.interface_value();
    case ITEM_VALUETYPE:
        return this->valuetype_value() == a_other.valuetype_value();
    case ITEM_DECL:
        return this->decl_value() == a_other.decl_value();
    }
    return true;
}

inline void swap(Item& a_first, Item& a_second) noexcept {
    Item a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void Item::_d(ItemKind discriminator) {
    switch (discriminator) {
    case ITEM_ANNOTATION:
        if (ic_discriminator_value_ != ITEM_ANNOTATION) {
            free_union_();
            intercom::construct_at(&ic_union_value_.annotation_value, AnnotationDef{});
        }
        break;
    case ITEM_MODULE:
        if (ic_discriminator_value_ != ITEM_MODULE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.module_value, ModuleDef{});
        }
        break;
    case ITEM_STRUCT:
        if (ic_discriminator_value_ != ITEM_STRUCT) {
            free_union_();
            intercom::construct_at(&ic_union_value_.struct_value, StructDef{});
        }
        break;
    case ITEM_UNION:
        if (ic_discriminator_value_ != ITEM_UNION) {
            free_union_();
            intercom::construct_at(&ic_union_value_.union_value, UnionDef{});
        }
        break;
    case ITEM_ENUM:
        if (ic_discriminator_value_ != ITEM_ENUM) {
            free_union_();
            intercom::construct_at(&ic_union_value_.enum_value, EnumDef{});
        }
        break;
    case ITEM_EXCEPTION:
        if (ic_discriminator_value_ != ITEM_EXCEPTION) {
            free_union_();
            intercom::construct_at(&ic_union_value_.exception_value, ExceptDef{});
        }
        break;
    case ITEM_BITMASK:
        if (ic_discriminator_value_ != ITEM_BITMASK) {
            free_union_();
            intercom::construct_at(&ic_union_value_.bitmask_value, BitmaskDef{});
        }
        break;
    case ITEM_BITSET:
        if (ic_discriminator_value_ != ITEM_BITSET) {
            free_union_();
            intercom::construct_at(&ic_union_value_.bitset_value, BitsetDef{});
        }
        break;
    case ITEM_CONST:
        if (ic_discriminator_value_ != ITEM_CONST) {
            free_union_();
            intercom::construct_at(&ic_union_value_.const_value, ConstDef{});
        }
        break;
    case ITEM_TYPEDEF:
        if (ic_discriminator_value_ != ITEM_TYPEDEF) {
            free_union_();
            intercom::construct_at(&ic_union_value_.alias_value, AliasDef{});
        }
        break;
    case ITEM_INTERFACE:
        if (ic_discriminator_value_ != ITEM_INTERFACE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.interface_value, InterfaceDef{});
        }
        break;
    case ITEM_VALUETYPE:
        if (ic_discriminator_value_ != ITEM_VALUETYPE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.valuetype_value, ValuetypeDef{});
        }
        break;
    case ITEM_DECL:
        if (ic_discriminator_value_ != ITEM_DECL) {
            free_union_();
            intercom::construct_at(&ic_union_value_.decl_value, Decl{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union Item");
    }
    ic_discriminator_value_ = discriminator;
}

inline AnnotationDef& Item::annotation_value() {
    if (ic_discriminator_value_ != ITEM_ANNOTATION) {
        throw std::logic_error("Union Item not set to value annotation_value");
    }
    return ic_union_value_.annotation_value;
}

inline const AnnotationDef& Item::annotation_value() const {
    if (ic_discriminator_value_ != ITEM_ANNOTATION) {
        throw std::logic_error("Union Item not set to value annotation_value");
    }
    return ic_union_value_.annotation_value;
}

inline void Item::annotation_value(const AnnotationDef& a_value) {
    if (ic_discriminator_value_ != ITEM_ANNOTATION) {
        free_union_();
        ic_discriminator_value_ = ITEM_ANNOTATION;
        intercom::construct_at(&ic_union_value_.annotation_value, a_value);
    } else {
        ic_union_value_.annotation_value = a_value;
    }
}

inline void Item::annotation_value(AnnotationDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_ANNOTATION) {
        free_union_();
        ic_discriminator_value_ = ITEM_ANNOTATION;
        intercom::construct_at(&ic_union_value_.annotation_value, std::move(a_value));
    } else {
        ic_union_value_.annotation_value = std::move(a_value);
    }
}

inline ModuleDef& Item::module_value() {
    if (ic_discriminator_value_ != ITEM_MODULE) {
        throw std::logic_error("Union Item not set to value module_value");
    }
    return ic_union_value_.module_value;
}

inline const ModuleDef& Item::module_value() const {
    if (ic_discriminator_value_ != ITEM_MODULE) {
        throw std::logic_error("Union Item not set to value module_value");
    }
    return ic_union_value_.module_value;
}

inline void Item::module_value(const ModuleDef& a_value) {
    if (ic_discriminator_value_ != ITEM_MODULE) {
        free_union_();
        ic_discriminator_value_ = ITEM_MODULE;
        intercom::construct_at(&ic_union_value_.module_value, a_value);
    } else {
        ic_union_value_.module_value = a_value;
    }
}

inline void Item::module_value(ModuleDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_MODULE) {
        free_union_();
        ic_discriminator_value_ = ITEM_MODULE;
        intercom::construct_at(&ic_union_value_.module_value, std::move(a_value));
    } else {
        ic_union_value_.module_value = std::move(a_value);
    }
}

inline StructDef& Item::struct_value() {
    if (ic_discriminator_value_ != ITEM_STRUCT) {
        throw std::logic_error("Union Item not set to value struct_value");
    }
    return ic_union_value_.struct_value;
}

inline const StructDef& Item::struct_value() const {
    if (ic_discriminator_value_ != ITEM_STRUCT) {
        throw std::logic_error("Union Item not set to value struct_value");
    }
    return ic_union_value_.struct_value;
}

inline void Item::struct_value(const StructDef& a_value) {
    if (ic_discriminator_value_ != ITEM_STRUCT) {
        free_union_();
        ic_discriminator_value_ = ITEM_STRUCT;
        intercom::construct_at(&ic_union_value_.struct_value, a_value);
    } else {
        ic_union_value_.struct_value = a_value;
    }
}

inline void Item::struct_value(StructDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_STRUCT) {
        free_union_();
        ic_discriminator_value_ = ITEM_STRUCT;
        intercom::construct_at(&ic_union_value_.struct_value, std::move(a_value));
    } else {
        ic_union_value_.struct_value = std::move(a_value);
    }
}

inline UnionDef& Item::union_value() {
    if (ic_discriminator_value_ != ITEM_UNION) {
        throw std::logic_error("Union Item not set to value union_value");
    }
    return ic_union_value_.union_value;
}

inline const UnionDef& Item::union_value() const {
    if (ic_discriminator_value_ != ITEM_UNION) {
        throw std::logic_error("Union Item not set to value union_value");
    }
    return ic_union_value_.union_value;
}

inline void Item::union_value(const UnionDef& a_value) {
    if (ic_discriminator_value_ != ITEM_UNION) {
        free_union_();
        ic_discriminator_value_ = ITEM_UNION;
        intercom::construct_at(&ic_union_value_.union_value, a_value);
    } else {
        ic_union_value_.union_value = a_value;
    }
}

inline void Item::union_value(UnionDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_UNION) {
        free_union_();
        ic_discriminator_value_ = ITEM_UNION;
        intercom::construct_at(&ic_union_value_.union_value, std::move(a_value));
    } else {
        ic_union_value_.union_value = std::move(a_value);
    }
}

inline EnumDef& Item::enum_value() {
    if (ic_discriminator_value_ != ITEM_ENUM) {
        throw std::logic_error("Union Item not set to value enum_value");
    }
    return ic_union_value_.enum_value;
}

inline const EnumDef& Item::enum_value() const {
    if (ic_discriminator_value_ != ITEM_ENUM) {
        throw std::logic_error("Union Item not set to value enum_value");
    }
    return ic_union_value_.enum_value;
}

inline void Item::enum_value(const EnumDef& a_value) {
    if (ic_discriminator_value_ != ITEM_ENUM) {
        free_union_();
        ic_discriminator_value_ = ITEM_ENUM;
        intercom::construct_at(&ic_union_value_.enum_value, a_value);
    } else {
        ic_union_value_.enum_value = a_value;
    }
}

inline void Item::enum_value(EnumDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_ENUM) {
        free_union_();
        ic_discriminator_value_ = ITEM_ENUM;
        intercom::construct_at(&ic_union_value_.enum_value, std::move(a_value));
    } else {
        ic_union_value_.enum_value = std::move(a_value);
    }
}

inline ExceptDef& Item::exception_value() {
    if (ic_discriminator_value_ != ITEM_EXCEPTION) {
        throw std::logic_error("Union Item not set to value exception_value");
    }
    return ic_union_value_.exception_value;
}

inline const ExceptDef& Item::exception_value() const {
    if (ic_discriminator_value_ != ITEM_EXCEPTION) {
        throw std::logic_error("Union Item not set to value exception_value");
    }
    return ic_union_value_.exception_value;
}

inline void Item::exception_value(const ExceptDef& a_value) {
    if (ic_discriminator_value_ != ITEM_EXCEPTION) {
        free_union_();
        ic_discriminator_value_ = ITEM_EXCEPTION;
        intercom::construct_at(&ic_union_value_.exception_value, a_value);
    } else {
        ic_union_value_.exception_value = a_value;
    }
}

inline void Item::exception_value(ExceptDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_EXCEPTION) {
        free_union_();
        ic_discriminator_value_ = ITEM_EXCEPTION;
        intercom::construct_at(&ic_union_value_.exception_value, std::move(a_value));
    } else {
        ic_union_value_.exception_value = std::move(a_value);
    }
}

inline BitmaskDef& Item::bitmask_value() {
    if (ic_discriminator_value_ != ITEM_BITMASK) {
        throw std::logic_error("Union Item not set to value bitmask_value");
    }
    return ic_union_value_.bitmask_value;
}

inline const BitmaskDef& Item::bitmask_value() const {
    if (ic_discriminator_value_ != ITEM_BITMASK) {
        throw std::logic_error("Union Item not set to value bitmask_value");
    }
    return ic_union_value_.bitmask_value;
}

inline void Item::bitmask_value(const BitmaskDef& a_value) {
    if (ic_discriminator_value_ != ITEM_BITMASK) {
        free_union_();
        ic_discriminator_value_ = ITEM_BITMASK;
        intercom::construct_at(&ic_union_value_.bitmask_value, a_value);
    } else {
        ic_union_value_.bitmask_value = a_value;
    }
}

inline void Item::bitmask_value(BitmaskDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_BITMASK) {
        free_union_();
        ic_discriminator_value_ = ITEM_BITMASK;
        intercom::construct_at(&ic_union_value_.bitmask_value, std::move(a_value));
    } else {
        ic_union_value_.bitmask_value = std::move(a_value);
    }
}

inline BitsetDef& Item::bitset_value() {
    if (ic_discriminator_value_ != ITEM_BITSET) {
        throw std::logic_error("Union Item not set to value bitset_value");
    }
    return ic_union_value_.bitset_value;
}

inline const BitsetDef& Item::bitset_value() const {
    if (ic_discriminator_value_ != ITEM_BITSET) {
        throw std::logic_error("Union Item not set to value bitset_value");
    }
    return ic_union_value_.bitset_value;
}

inline void Item::bitset_value(const BitsetDef& a_value) {
    if (ic_discriminator_value_ != ITEM_BITSET) {
        free_union_();
        ic_discriminator_value_ = ITEM_BITSET;
        intercom::construct_at(&ic_union_value_.bitset_value, a_value);
    } else {
        ic_union_value_.bitset_value = a_value;
    }
}

inline void Item::bitset_value(BitsetDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_BITSET) {
        free_union_();
        ic_discriminator_value_ = ITEM_BITSET;
        intercom::construct_at(&ic_union_value_.bitset_value, std::move(a_value));
    } else {
        ic_union_value_.bitset_value = std::move(a_value);
    }
}

inline ConstDef& Item::const_value() {
    if (ic_discriminator_value_ != ITEM_CONST) {
        throw std::logic_error("Union Item not set to value const_value");
    }
    return ic_union_value_.const_value;
}

inline const ConstDef& Item::const_value() const {
    if (ic_discriminator_value_ != ITEM_CONST) {
        throw std::logic_error("Union Item not set to value const_value");
    }
    return ic_union_value_.const_value;
}

inline void Item::const_value(const ConstDef& a_value) {
    if (ic_discriminator_value_ != ITEM_CONST) {
        free_union_();
        ic_discriminator_value_ = ITEM_CONST;
        intercom::construct_at(&ic_union_value_.const_value, a_value);
    } else {
        ic_union_value_.const_value = a_value;
    }
}

inline void Item::const_value(ConstDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_CONST) {
        free_union_();
        ic_discriminator_value_ = ITEM_CONST;
        intercom::construct_at(&ic_union_value_.const_value, std::move(a_value));
    } else {
        ic_union_value_.const_value = std::move(a_value);
    }
}

inline AliasDef& Item::alias_value() {
    if (ic_discriminator_value_ != ITEM_TYPEDEF) {
        throw std::logic_error("Union Item not set to value alias_value");
    }
    return ic_union_value_.alias_value;
}

inline const AliasDef& Item::alias_value() const {
    if (ic_discriminator_value_ != ITEM_TYPEDEF) {
        throw std::logic_error("Union Item not set to value alias_value");
    }
    return ic_union_value_.alias_value;
}

inline void Item::alias_value(const AliasDef& a_value) {
    if (ic_discriminator_value_ != ITEM_TYPEDEF) {
        free_union_();
        ic_discriminator_value_ = ITEM_TYPEDEF;
        intercom::construct_at(&ic_union_value_.alias_value, a_value);
    } else {
        ic_union_value_.alias_value = a_value;
    }
}

inline void Item::alias_value(AliasDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_TYPEDEF) {
        free_union_();
        ic_discriminator_value_ = ITEM_TYPEDEF;
        intercom::construct_at(&ic_union_value_.alias_value, std::move(a_value));
    } else {
        ic_union_value_.alias_value = std::move(a_value);
    }
}

inline InterfaceDef& Item::interface_value() {
    if (ic_discriminator_value_ != ITEM_INTERFACE) {
        throw std::logic_error("Union Item not set to value interface_value");
    }
    return ic_union_value_.interface_value;
}

inline const InterfaceDef& Item::interface_value() const {
    if (ic_discriminator_value_ != ITEM_INTERFACE) {
        throw std::logic_error("Union Item not set to value interface_value");
    }
    return ic_union_value_.interface_value;
}

inline void Item::interface_value(const InterfaceDef& a_value) {
    if (ic_discriminator_value_ != ITEM_INTERFACE) {
        free_union_();
        ic_discriminator_value_ = ITEM_INTERFACE;
        intercom::construct_at(&ic_union_value_.interface_value, a_value);
    } else {
        ic_union_value_.interface_value = a_value;
    }
}

inline void Item::interface_value(InterfaceDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_INTERFACE) {
        free_union_();
        ic_discriminator_value_ = ITEM_INTERFACE;
        intercom::construct_at(&ic_union_value_.interface_value, std::move(a_value));
    } else {
        ic_union_value_.interface_value = std::move(a_value);
    }
}

inline ValuetypeDef& Item::valuetype_value() {
    if (ic_discriminator_value_ != ITEM_VALUETYPE) {
        throw std::logic_error("Union Item not set to value valuetype_value");
    }
    return ic_union_value_.valuetype_value;
}

inline const ValuetypeDef& Item::valuetype_value() const {
    if (ic_discriminator_value_ != ITEM_VALUETYPE) {
        throw std::logic_error("Union Item not set to value valuetype_value");
    }
    return ic_union_value_.valuetype_value;
}

inline void Item::valuetype_value(const ValuetypeDef& a_value) {
    if (ic_discriminator_value_ != ITEM_VALUETYPE) {
        free_union_();
        ic_discriminator_value_ = ITEM_VALUETYPE;
        intercom::construct_at(&ic_union_value_.valuetype_value, a_value);
    } else {
        ic_union_value_.valuetype_value = a_value;
    }
}

inline void Item::valuetype_value(ValuetypeDef&& a_value) {
    if (ic_discriminator_value_ != ITEM_VALUETYPE) {
        free_union_();
        ic_discriminator_value_ = ITEM_VALUETYPE;
        intercom::construct_at(&ic_union_value_.valuetype_value, std::move(a_value));
    } else {
        ic_union_value_.valuetype_value = std::move(a_value);
    }
}

inline Decl& Item::decl_value() {
    if (ic_discriminator_value_ != ITEM_DECL) {
        throw std::logic_error("Union Item not set to value decl_value");
    }
    return ic_union_value_.decl_value;
}

inline const Decl& Item::decl_value() const {
    if (ic_discriminator_value_ != ITEM_DECL) {
        throw std::logic_error("Union Item not set to value decl_value");
    }
    return ic_union_value_.decl_value;
}

inline void Item::decl_value(const Decl& a_value) {
    if (ic_discriminator_value_ != ITEM_DECL) {
        free_union_();
        ic_discriminator_value_ = ITEM_DECL;
        intercom::construct_at(&ic_union_value_.decl_value, a_value);
    } else {
        ic_union_value_.decl_value = a_value;
    }
}

inline void Item::decl_value(Decl&& a_value) {
    if (ic_discriminator_value_ != ITEM_DECL) {
        free_union_();
        ic_discriminator_value_ = ITEM_DECL;
        intercom::construct_at(&ic_union_value_.decl_value, std::move(a_value));
    } else {
        ic_union_value_.decl_value = std::move(a_value);
    }
}

inline void Item::free_union_() {
    switch (ic_discriminator_value_) {
    case ITEM_ANNOTATION:
        std::destroy_at(&ic_union_value_.annotation_value);
        break;
    case ITEM_MODULE:
        std::destroy_at(&ic_union_value_.module_value);
        break;
    case ITEM_STRUCT:
        std::destroy_at(&ic_union_value_.struct_value);
        break;
    case ITEM_UNION:
        std::destroy_at(&ic_union_value_.union_value);
        break;
    case ITEM_ENUM:
        std::destroy_at(&ic_union_value_.enum_value);
        break;
    case ITEM_EXCEPTION:
        std::destroy_at(&ic_union_value_.exception_value);
        break;
    case ITEM_BITMASK:
        std::destroy_at(&ic_union_value_.bitmask_value);
        break;
    case ITEM_BITSET:
        std::destroy_at(&ic_union_value_.bitset_value);
        break;
    case ITEM_CONST:
        std::destroy_at(&ic_union_value_.const_value);
        break;
    case ITEM_TYPEDEF:
        std::destroy_at(&ic_union_value_.alias_value);
        break;
    case ITEM_INTERFACE:
        std::destroy_at(&ic_union_value_.interface_value);
        break;
    case ITEM_VALUETYPE:
        std::destroy_at(&ic_union_value_.valuetype_value);
        break;
    case ITEM_DECL:
        std::destroy_at(&ic_union_value_.decl_value);
        break;
    }
}

inline InterfaceMember::InterfaceMember() {
    ic_discriminator_value_ = INTERFACE_ATTRIBUTE;
    intercom::construct_at(&ic_union_value_.attr, Attribute{});
}

inline InterfaceMember::InterfaceMember(const InterfaceMember& a_other) {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case INTERFACE_ATTRIBUTE:
        intercom::construct_at(&ic_union_value_.attr, a_other.ic_union_value_.attr);
        break;
    case INTERFACE_PROTOTYPE:
        intercom::construct_at(&ic_union_value_.proto, a_other.ic_union_value_.proto);
        break;
    case INTERFACE_ITEM:
        intercom::construct_at(&ic_union_value_.item, a_other.ic_union_value_.item);
        break;
    }
}

inline InterfaceMember& InterfaceMember::operator=(const InterfaceMember& a_other) {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case INTERFACE_ATTRIBUTE:
            intercom::construct_at(&ic_union_value_.attr, a_other.ic_union_value_.attr);
            break;
        case INTERFACE_PROTOTYPE:
            intercom::construct_at(&ic_union_value_.proto, a_other.ic_union_value_.proto);
            break;
        case INTERFACE_ITEM:
            intercom::construct_at(&ic_union_value_.item, a_other.ic_union_value_.item);
            break;
        }
    }

    return *this;
}

inline InterfaceMember::InterfaceMember(InterfaceMember&& a_other) noexcept : InterfaceMember() {
    ic_discriminator_value_ = a_other.ic_discriminator_value_;
    switch (ic_discriminator_value_) {
    case INTERFACE_ATTRIBUTE:
        intercom::construct_at(&ic_union_value_.attr, std::move(a_other.ic_union_value_.attr));
        break;
    case INTERFACE_PROTOTYPE:
        intercom::construct_at(&ic_union_value_.proto, std::move(a_other.ic_union_value_.proto));
        break;
    case INTERFACE_ITEM:
        intercom::construct_at(&ic_union_value_.item, std::move(a_other.ic_union_value_.item));
        break;
    }
}

inline InterfaceMember& InterfaceMember::operator=(InterfaceMember&& a_other) noexcept {
    if (this != &a_other) {
        free_union_();
        ic_discriminator_value_ = a_other.ic_discriminator_value_;
        switch (ic_discriminator_value_) {
        case INTERFACE_ATTRIBUTE:
            intercom::construct_at(&ic_union_value_.attr, std::move(a_other.ic_union_value_.attr));
            break;
        case INTERFACE_PROTOTYPE:
            intercom::construct_at(
                &ic_union_value_.proto, std::move(a_other.ic_union_value_.proto)
            );
            break;
        case INTERFACE_ITEM:
            intercom::construct_at(&ic_union_value_.item, std::move(a_other.ic_union_value_.item));
            break;
        }
    }
    return *this;
}

inline InterfaceMember::~InterfaceMember() noexcept {
    free_union_();
}

inline bool InterfaceMember::operator<(const InterfaceMember& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case INTERFACE_ATTRIBUTE:
        return this->attr() < a_other.attr();
    case INTERFACE_PROTOTYPE:
        return this->proto() < a_other.proto();
    case INTERFACE_ITEM:
        return this->item() < a_other.item();
    }
    return false;
}

inline bool InterfaceMember::operator==(const InterfaceMember& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case INTERFACE_ATTRIBUTE:
        return this->attr() == a_other.attr();
    case INTERFACE_PROTOTYPE:
        return this->proto() == a_other.proto();
    case INTERFACE_ITEM:
        return this->item() == a_other.item();
    }
    return true;
}

inline void swap(InterfaceMember& a_first, InterfaceMember& a_second) noexcept {
    InterfaceMember a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void InterfaceMember::_d(InterfaceMemberKind discriminator) {
    switch (discriminator) {
    case INTERFACE_ATTRIBUTE:
        if (ic_discriminator_value_ != INTERFACE_ATTRIBUTE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.attr, Attribute{});
        }
        break;
    case INTERFACE_PROTOTYPE:
        if (ic_discriminator_value_ != INTERFACE_PROTOTYPE) {
            free_union_();
            intercom::construct_at(&ic_union_value_.proto, Prototype{});
        }
        break;
    case INTERFACE_ITEM:
        if (ic_discriminator_value_ != INTERFACE_ITEM) {
            free_union_();
            intercom::construct_at(&ic_union_value_.item, Item{});
        }
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union InterfaceMember");
    }
    ic_discriminator_value_ = discriminator;
}

inline Attribute& InterfaceMember::attr() {
    if (ic_discriminator_value_ != INTERFACE_ATTRIBUTE) {
        throw std::logic_error("Union InterfaceMember not set to value attr");
    }
    return ic_union_value_.attr;
}

inline const Attribute& InterfaceMember::attr() const {
    if (ic_discriminator_value_ != INTERFACE_ATTRIBUTE) {
        throw std::logic_error("Union InterfaceMember not set to value attr");
    }
    return ic_union_value_.attr;
}

inline void InterfaceMember::attr(const Attribute& a_value) {
    if (ic_discriminator_value_ != INTERFACE_ATTRIBUTE) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_ATTRIBUTE;
        intercom::construct_at(&ic_union_value_.attr, a_value);
    } else {
        ic_union_value_.attr = a_value;
    }
}

inline void InterfaceMember::attr(Attribute&& a_value) {
    if (ic_discriminator_value_ != INTERFACE_ATTRIBUTE) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_ATTRIBUTE;
        intercom::construct_at(&ic_union_value_.attr, std::move(a_value));
    } else {
        ic_union_value_.attr = std::move(a_value);
    }
}

inline Prototype& InterfaceMember::proto() {
    if (ic_discriminator_value_ != INTERFACE_PROTOTYPE) {
        throw std::logic_error("Union InterfaceMember not set to value proto");
    }
    return ic_union_value_.proto;
}

inline const Prototype& InterfaceMember::proto() const {
    if (ic_discriminator_value_ != INTERFACE_PROTOTYPE) {
        throw std::logic_error("Union InterfaceMember not set to value proto");
    }
    return ic_union_value_.proto;
}

inline void InterfaceMember::proto(const Prototype& a_value) {
    if (ic_discriminator_value_ != INTERFACE_PROTOTYPE) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_PROTOTYPE;
        intercom::construct_at(&ic_union_value_.proto, a_value);
    } else {
        ic_union_value_.proto = a_value;
    }
}

inline void InterfaceMember::proto(Prototype&& a_value) {
    if (ic_discriminator_value_ != INTERFACE_PROTOTYPE) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_PROTOTYPE;
        intercom::construct_at(&ic_union_value_.proto, std::move(a_value));
    } else {
        ic_union_value_.proto = std::move(a_value);
    }
}

inline Item& InterfaceMember::item() {
    if (ic_discriminator_value_ != INTERFACE_ITEM) {
        throw std::logic_error("Union InterfaceMember not set to value item");
    }
    return ic_union_value_.item;
}

inline const Item& InterfaceMember::item() const {
    if (ic_discriminator_value_ != INTERFACE_ITEM) {
        throw std::logic_error("Union InterfaceMember not set to value item");
    }
    return ic_union_value_.item;
}

inline void InterfaceMember::item(const Item& a_value) {
    if (ic_discriminator_value_ != INTERFACE_ITEM) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_ITEM;
        intercom::construct_at(&ic_union_value_.item, a_value);
    } else {
        ic_union_value_.item = a_value;
    }
}

inline void InterfaceMember::item(Item&& a_value) {
    if (ic_discriminator_value_ != INTERFACE_ITEM) {
        free_union_();
        ic_discriminator_value_ = INTERFACE_ITEM;
        intercom::construct_at(&ic_union_value_.item, std::move(a_value));
    } else {
        ic_union_value_.item = std::move(a_value);
    }
}

inline void InterfaceMember::free_union_() {
    switch (ic_discriminator_value_) {
    case INTERFACE_ATTRIBUTE:
        std::destroy_at(&ic_union_value_.attr);
        break;
    case INTERFACE_PROTOTYPE:
        std::destroy_at(&ic_union_value_.proto);
        break;
    case INTERFACE_ITEM:
        std::destroy_at(&ic_union_value_.item);
        break;
    }
}

}  // namespace ast

#ifdef _WIN32
#  pragma warning(pop)
#endif
