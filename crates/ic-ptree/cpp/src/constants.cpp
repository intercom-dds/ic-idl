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

#include "cidl/constants.h"

#include <cstdlib>
#include <cstring>
#include <string>

#include "cidl/idl_parser.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"

template <typename T>
static numeric* create_numeric(parser_state* state, T val);

#define CREATE_NUMERIC(type_name, enum_kind, name)                \
    template <>                                                   \
    numeric* create_numeric(parser_state* state, type_name val) { \
        auto n = new_numeric(state, enum_kind);                   \
        n->val.name(val);                                         \
        return n;                                                 \
    }

CREATE_NUMERIC(int32_t, LONG_KIND, l);

CREATE_NUMERIC(uint32_t, ULONG_KIND, ul);

CREATE_NUMERIC(int64_t, LONGLONG_KIND, ll);

CREATE_NUMERIC(uint64_t, ULONGLONG_KIND, ull);

CREATE_NUMERIC(float, FLOAT_KIND, f);

CREATE_NUMERIC(double, DOUBLE_KIND, d);

CREATE_NUMERIC(std::string, STRING_KIND, str);

int numeric_base(const numeric& v) {
    if (v.kind() == PTREE_KIND) {
        return numeric_base(v.val.node()->value);
    }
    return v.base;
}

extern "C" {
numeric num_undef{};

const numeric* expr_convert(parser_state* state, const numeric* v, numeric_kind kind) {
    auto res = new_numeric(state, kind);
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
}

namespace intercom::cidl {

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

}  // namespace intercom::cidl
