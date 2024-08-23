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

#include "cidl/constants.h"

#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>

#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"

namespace {
template <typename T>
numeric* create_numeric(T val);

#define CREATE_NUMERIC(type_name, enum_kind, name) \
    template <>                                    \
    numeric* create_numeric(type_name val) {       \
        auto n = new_numeric(enum_kind);           \
        n->val.name(val);                          \
        return n;                                  \
    }

CREATE_NUMERIC(bool, BOOLEAN_KIND, b);

CREATE_NUMERIC(int32_t, LONG_KIND, l);

CREATE_NUMERIC(uint32_t, ULONG_KIND, ul);

CREATE_NUMERIC(int64_t, LONGLONG_KIND, ll);

CREATE_NUMERIC(uint64_t, ULONGLONG_KIND, ull);

CREATE_NUMERIC(float, FLOAT_KIND, f);

CREATE_NUMERIC(double, DOUBLE_KIND, d);

CREATE_NUMERIC(std::string, STRING_KIND, str);

template <typename T1, typename T2>
numeric* expr_binary_t2(char op, T1 v1, T2 v2) {
    switch (op) {
    case '<':
        return create_numeric(v1 << v2);
    case '>':
        return create_numeric(v1 >> v2);
    case '|':
        return create_numeric(v1 | v2);
    case '^':
        return create_numeric(v1 ^ v2);
    case '&':
        return create_numeric(v1 & v2);
    case '+':
        return create_numeric(v1 + v2);
    case '-':
        return create_numeric(v1 - v2);
    case '*':
        return create_numeric(v1 * v2);
    case '%': {
        if (v2 == 0) {
            break;
        }
        return create_numeric(v1 % v2);
    }
    case '/': {
        if (v2 == 0) {
            break;
        }
        return create_numeric(v1 / v2);
    }
    default:
        break;
    }
    return &num_undef;
}

template <typename T1, typename T2>
numeric* expr_binary_nobitop_t2(char op, T1 v1, T2 v2) {
    switch (op) {
    case '+':
        return create_numeric(v1 + v2);
    case '-':
        return create_numeric(v1 - v2);
    case '*':
        return create_numeric(v1 * v2);
    case '/':
        return create_numeric(v1 / v2);
    default:
        break;
    }
    return &num_undef;
}

template <typename T>
numeric* expr_binary_t1(char op, T v1, const numeric& v2) {
    switch (v2.kind()) {
    case UNDEF_KIND:
        return &num_undef;
    case BOOLEAN_KIND:
        return expr_binary_t2(op, v1, v2.val.b());
    case INT8_KIND:
        return expr_binary_t2(op, v1, v2.val.i8());
    case OCTET_KIND:
        return expr_binary_t2(op, v1, v2.val.o());
    case SHORT_KIND:
        return expr_binary_t2(op, v1, v2.val.s());
    case USHORT_KIND:
        return expr_binary_t2(op, v1, v2.val.us());
    case LONG_KIND:
        return expr_binary_t2(op, v1, v2.val.l());
    case ULONG_KIND:
        return expr_binary_t2(op, v1, v2.val.ul());
    case LONGLONG_KIND:
        return expr_binary_t2(op, v1, v2.val.ll());
    case ULONGLONG_KIND:
        return expr_binary_t2(op, v1, v2.val.ull());
    case FLOAT_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.f());
    case DOUBLE_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.d());
    case CHAR_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.c());
    case STRING_KIND:
        return &num_undef;
    case PTREE_KIND:
        return expr_binary_t1(op, v1, v2.val.node()->value);
    }
    return &num_undef;
}

template <typename T>
numeric* expr_binary_nobitop_t1(char op, T v1, const numeric& v2) {
    switch (v2.kind()) {
    case UNDEF_KIND:
        return &num_undef;
    case BOOLEAN_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.b());
    case INT8_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.i8());
    case OCTET_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.o());
    case SHORT_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.s());
    case USHORT_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.us());
    case LONG_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.l());
    case ULONG_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.ul());
    case LONGLONG_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.ll());
    case ULONGLONG_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.ull());
    case FLOAT_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.f());
    case DOUBLE_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.d());
    case CHAR_KIND:
        return expr_binary_nobitop_t2(op, v1, v2.val.c());
    case STRING_KIND:
        return &num_undef;
    case PTREE_KIND:
        return expr_binary_nobitop_t1(op, v1, v2.val.node()->value);
    }
    return &num_undef;
}

template <typename T>
numeric* expr_unary(char op, T val) {
    if (op == '~') {
        return create_numeric(~val);
    }
    if (op == '-') {
        return create_numeric(-val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, unsigned char val) {
    if (op == '-' || op == '~') {
        return create_numeric(~val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, unsigned short val) {
    if (op == '-' || op == '~') {
        return create_numeric(~val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, uint32_t val) {
    if (op == '-' || op == '~') {
        return create_numeric(~val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, uint64_t val) {
    if (op == '-' || op == '~') {
        return create_numeric(~val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, float val) {
    if (op == '-') {
        return create_numeric(-val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char op, double val) {
    if (op == '-') {
        return create_numeric(-val);
    }
    return &num_undef;
}

template <>
numeric* expr_unary(char, std::string) {
    return &num_undef;
}
}  // namespace

int integer_value(const numeric& v) {
    return value<int>(v);
}

unsigned long unsigned_value(const numeric& v) {
    return value<unsigned long>(v);
}

long long long_long_value(const numeric& v) {
    return value<long long>(v);
}

float float_value(const numeric& v) {
    return value<float>(v);
}

double double_value(const numeric& v) {
    return value<double>(v);
}

std::string string_value(const numeric& v) {
    return value<std::string>(v);
}

int numeric_base(const numeric& v) {
    if (v.kind() == PTREE_KIND) {
        return numeric_base(v.val.node()->value);
    }
    return v.base;
}

extern "C" {
bool is_signed(const numeric& v);
bool is_unsigned(const numeric& v);

numeric num_undef{};

const numeric* expr_convert(const numeric* v, numeric_kind kind) {
    auto res = new_numeric(kind);
    res->base = numeric_base(*v);
    if (kind == UNDEF_KIND || kind == PTREE_KIND || kind == v->kind()) {
        *res = *v;
    } else {
        switch (kind) {
        case BOOLEAN_KIND:
            res->val.b(value<int>(*v) != 0);
            break;
        case INT8_KIND:
            res->val.i8(value<int8_t>(*v));
            break;
        case OCTET_KIND:
            res->val.o(value<uint8_t>(*v));
            break;
        case CHAR_KIND:
            res->val.c(value<int>(*v));
            break;
        case SHORT_KIND:
            res->val.s(value<int16_t>(*v));
            break;
        case USHORT_KIND:
            res->val.us(value<uint16_t>(*v));
            break;
        case LONG_KIND:
            res->val.l(value<int32_t>(*v));
            break;
        case ULONG_KIND:
            res->val.ul(value<uint32_t>(*v));
            break;
        case LONGLONG_KIND:
            res->val.ll(value<int64_t>(*v));
            break;
        case ULONGLONG_KIND:
            res->val.ull(value<uint64_t>(*v));
            break;
        case FLOAT_KIND:
            res->val.f(value<float>(*v));
            break;
        case DOUBLE_KIND:
            res->val.d(value<double>(*v));
            break;
        case STRING_KIND:
            res->val.str(value<std::string>(*v));
            break;
        default:
            *res = *v;
            break;
        }
    }
    return res;
}

const numeric* expr_unary(char op, const numeric* v) {
    auto res = new_numeric(v->kind());
    *res = *v;
    if (op == '-') {
        switch (res->kind()) {
        case USHORT_KIND:
            res->val.s(value<int16_t>(*res));
            break;
        case ULONG_KIND:
            res->val.l(value<int32_t>(*res));
            break;
        case ULONGLONG_KIND:
            res->val.ll(value<int64_t>(*res));
            break;
        default:
            break;
        }
    }
    switch (res->kind()) {
    case UNDEF_KIND:
        *res = num_undef;
        break;
    case BOOLEAN_KIND:
        *res = *expr_unary(op, res->val.b());
        break;
    case INT8_KIND:
        *res = *expr_unary(op, res->val.i8());
        break;
    case OCTET_KIND:
        *res = *expr_unary(op, res->val.o());
        break;
    case SHORT_KIND:
        *res = *expr_unary(op, res->val.s());
        break;
    case USHORT_KIND:
        *res = *expr_unary(op, res->val.us());
        break;
    case LONG_KIND:
        *res = *expr_unary(op, res->val.l());
        break;
    case ULONG_KIND:
        *res = *expr_unary(op, res->val.ul());
        break;
    case LONGLONG_KIND:
        *res = *expr_unary(op, res->val.ll());
        break;
    case ULONGLONG_KIND:
        *res = *expr_unary(op, res->val.ull());
        break;
    case FLOAT_KIND:
        *res = *expr_unary(op, res->val.f());
        break;
    case DOUBLE_KIND:
        *res = *expr_unary(op, res->val.d());
        break;
    case CHAR_KIND:
        *res = *expr_unary(op, res->val.c());
        break;
    case STRING_KIND:
        *res = *expr_unary(op, res->val.str());
        break;
    case PTREE_KIND:
        *res = *expr_unary(op, &res->val.node()->value);
        break;
    }
    if (res->kind() == UNDEF_KIND) {
        ERR << "Invalid unary operator";
    }
    if (numeric_base(*v) != 0) {
        res->base = numeric_base(*v);
    }
    return res;
}

const numeric* expr_binary(char op, const numeric* v1, const numeric* v2) {
    auto res = new_numeric(UNDEF_KIND);
    switch (v1->kind()) {
    case UNDEF_KIND:
        *res = num_undef;
        break;
    case BOOLEAN_KIND:
        *res = *expr_binary_t1(op, v1->val.b(), *v2);
        break;
    case INT8_KIND:
        *res = *expr_binary_t1(op, v1->val.i8(), *v2);
        break;
    case OCTET_KIND:
        *res = *expr_binary_t1(op, v1->val.o(), *v2);
        break;
    case SHORT_KIND:
        *res = *expr_binary_t1(op, v1->val.s(), *v2);
        break;
    case USHORT_KIND:
        *res = *expr_binary_t1(op, v1->val.us(), *v2);
        break;
    case LONG_KIND:
        *res = *expr_binary_t1(op, v1->val.l(), *v2);
        break;
    case ULONG_KIND:
        *res = *expr_binary_t1(op, v1->val.ul(), *v2);
        break;
    case LONGLONG_KIND:
        *res = *expr_binary_t1(op, v1->val.ll(), *v2);
        break;
    case ULONGLONG_KIND:
        *res = *expr_binary_t1(op, v1->val.ull(), *v2);
        break;
    case FLOAT_KIND:
        *res = *expr_binary_nobitop_t1(op, v1->val.f(), *v2);
        break;
    case DOUBLE_KIND:
        *res = *expr_binary_nobitop_t1(op, v1->val.d(), *v2);
        break;
    case CHAR_KIND:
        *res = *expr_binary_t1(op, v1->val.c(), *v2);
        break;
    case STRING_KIND:
        if (v2->kind() == STRING_KIND) {
            *res = *create_numeric(v1->val.str() + v2->val.str());
        } else {
            *res = num_undef;
        }
        break;
    case PTREE_KIND:
        *res = *expr_binary(op, &v1->val.node()->value, v2);
        break;
    }

    if (res->kind() == UNDEF_KIND) {
        ERR << "Invalid binary operator";
    }
    if (numeric_base(*v1) == numeric_base(*v2) && numeric_base(*v1) != 0) {
        res->base = numeric_base(*v1);
    }
    return res;
}
}
