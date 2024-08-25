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

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum PlacementKind {
    BEGIN_FILE,
    BEFORE_DECLARATION,
    BEGIN_DECLARATION,
    END_DECLARATION,
    AFTER_DECLARATION,
    END_FILE
};

enum numeric_kind {
    UNDEF_KIND,
    BOOLEAN_KIND,
    INT8_KIND,
    OCTET_KIND,
    SHORT_KIND,
    USHORT_KIND,
    LONG_KIND,
    ULONG_KIND,
    LONGLONG_KIND,
    ULONGLONG_KIND,
    FLOAT_KIND,
    DOUBLE_KIND,
    CHAR_KIND,
    STRING_KIND,
    PTREE_KIND
};

const char* numeric_kind_str(enum numeric_kind val);

struct numeric;

struct parser_state;

enum node_kind {
    N_UNDEF,
    N_INCLUDE,    // An include statement
    N_PRIMITIVE,  // A primitive type (long, octet, ... )
    N_NATIVE,     // A node with type "native" in IDL

    N_MODULE,     // A module (namespace)
    N_STRUCT,     // A struct type or a struct declaration
    N_UNION,      // A union type or a union declaration
    N_VALUETYPE,  // A valuetype or a valuetype declaration
    N_INTERFACE,  // An interface
    N_EXCEPTION,  // An exception definition
    N_ENUM,       // An enum type
    N_BITSET,     // A bitset type
    N_BITMASK,    // A bitmask type or an enum annotated with @bitset

    N_CASE,    // A case value inside a union
    N_NULL,    // An explicit null node
    N_MEMBER,  // A member variable in a module, enum, bitset, bitmask, struct, union or valuetype
    N_PROTOTYPE,  // A function prototype inside an interface

    N_SEQUENCE,  // A sequence, possibly bounded
    N_MAP,       // A map, possibly bounded
    N_ARRAY,     // An N-dimensional array
    N_STRING,    // A string, possibly bounded
    N_FIXED,     // A fixed type
    N_ALIAS,     // A type alias (typedef)
    N_CONST,     // A constant value

    N_ANNOTATION_DEF,  // An annotation definition
    N_ANNOTATION       // An annotation application
};

const char* node_kind_str(enum node_kind kind);

enum ptree_opts {
    OPT_DECLARATION = (1 << 0),
    OPT_IN = (1 << 1),
    OPT_OUT = (1 << 2),
    OPT_INOUT = (OPT_IN | OPT_OUT),
    OPT_READONLY = (1 << 3),
    OPT_PRIVATE = (1 << 4),
    OPT_DEFAULT = (1 << 5),  //!< default case in union
    OPT_HAS_CHILDREN = (1 << 6),
    /// enum member with explicit value, != the implicit value e.g. "enum E { V0, V1 = 1, ENUMERATED
    /// = 5, V3 };" \note also applied to the enum node, if any members have it
    OPT_ENUMERATED = (1 << 7),
    OPT_EMIT_CODE = (1 << 8),
    OPT_SYSTEM_INCLUDE = (1 << 9),
    OPT_CIRCULAR = (1 << 10),
    OPT_SEQUENCE_LENGTH = (1 << 11),
    OPT_CONST_VALUE = (1 << 12),      //!< node in struct numeric i.e. part of complex value
    OPT_ANONYMOUS_ALIAS = (1 << 13),  //!< e.g. the element_type of "sequence<\@optional int8>"

    // Rust-specific flags:
    OPT_RUST_TRIVIAL = (1 << 14),      // A type comprised solely of primitive types
    OPT_RUST_TOTAL_ORDER = (1 << 15),  // Types that form a total order

    OPT_LOCAL = (1 << 16),
    OPT_BUILTIN = (1 << 17),  // Built-in type definitions, mostly primitives and annotations
};

struct identifier {
    const char* name;
};

struct ptree;

void clear_namespace_nodes(struct parser_state* state);

extern enum node_kind ANY_KIND[];

extern struct numeric num_undef;

const struct numeric*
expr_convert(struct parser_state* state, const struct numeric* value, enum numeric_kind kind);

const struct numeric* expr_unary(struct parser_state* state, char op, const struct numeric* v);

const struct numeric* expr_binary(
    struct parser_state* state,
    char op,
    const struct numeric* v1,
    const struct numeric* v2
);

extern struct ptree boolean_type;
extern struct ptree int8_type;
extern struct ptree octet_type;
extern struct ptree char_type;
extern struct ptree wchar_type;
extern struct ptree short_type;
extern struct ptree ushort_type;
extern struct ptree long_type;
extern struct ptree ulong_type;
extern struct ptree longlong_type;
extern struct ptree ulonglong_type;
extern struct ptree float_type;
extern struct ptree double_type;
extern struct ptree ldouble_type;
extern struct ptree fixed_type;
extern struct ptree unbounded_string_type;
extern struct ptree unbounded_wstring_type;
extern struct ptree any_type;
extern struct ptree object_type;

extern struct ptree* annotation_type_id;
extern struct ptree* annotation_type_autoid;
extern struct ptree* annotation_type_optional;
extern struct ptree* annotation_type_position;
extern struct ptree* annotation_type_value;
extern struct ptree* annotation_type_empty;
extern struct ptree* annotation_type_extensibility;
extern struct ptree* annotation_type_final;
extern struct ptree* annotation_type_mutable;
extern struct ptree* annotation_type_appendable;
extern struct ptree* annotation_type_shared;
extern struct ptree* annotation_type_key;
extern struct ptree* annotation_type_must_understand;
extern struct ptree* annotation_type_default;
extern struct ptree* annotation_type_default_literal;
extern struct ptree* annotation_type_range;
extern struct ptree* annotation_type_min;
extern struct ptree* annotation_type_max;
extern struct ptree* annotation_type_unit;
extern struct ptree* annotation_type_bit_bound;
extern struct ptree* annotation_type_external;
extern struct ptree* annotation_type_nested;
extern struct ptree* annotation_type_verbatim;
extern struct ptree* annotation_type_service;
extern struct ptree* annotation_type_topic;
extern struct ptree* annotation_type_dds_service;
extern struct ptree* annotation_type_dds_request_topic;
extern struct ptree* annotation_type_dds_reply_topic;
extern struct ptree* annotation_type_oneway;
extern struct ptree* annotation_type_ami;
extern struct ptree* annotation_type_bitset_old;
extern struct ptree* annotation_type_bit_bound_old;
extern struct ptree* annotation_type_must_understand_old;
extern struct ptree* annotation_type_minimum_type_check_old;
extern struct ptree* annotation_type_hashid;
extern struct ptree* annotation_type_default_nested;
extern struct ptree* annotation_type_ignore_literal_names;
extern struct ptree* annotation_type_try_construct;
extern struct ptree* annotation_type_non_serialized;
extern struct ptree* annotation_type_data_representation;
extern struct ptree* annotation_type_doc;
extern struct ptree* annotation_type_merge;
extern struct ptree* annotation_type_const;
extern struct ptree* annotation_type_static;
extern struct ptree* annotation_type_derive;
extern struct ptree* annotation_type_ext_rename;
extern struct ptree* annotation_type_ext_builder;
extern struct ptree* annotation_type_ext_doc;
extern struct ptree* annotation_type_ext_minimum_type_check;
extern struct ptree* annotation_type_ext_suppress;
extern struct ptree* annotation_type_ext_no_constructor;
extern struct ptree* annotation_type_ext_no_serializer;
extern struct ptree* annotation_type_ext_listener;
extern struct ptree* annotation_type_ext_length_bit_bound;
extern struct ptree* annotation_type_ext_value_offset;
extern struct ptree* annotation_type_ext_length_value_offset;
extern struct ptree* annotation_type_ext_repeat_count;
extern struct ptree* annotation_type_ext_vmf_xri;
extern struct ptree* annotation_type_ext_vmf_decimal;
extern struct ptree* annotation_type_ext_string_constants;
extern struct ptree* annotation_type_ext_jaus_presence_vector;
extern struct ptree* annotation_type_ext_jaus_integer;
extern struct ptree* annotation_type_ext_jaus_integer_function;
extern struct ptree* annotation_type_ext_protobuf_type;
extern struct ptree* annotation_type_jaus;

#ifdef __cplusplus
};  // extern C

#  include <string>

#  include "cidl/numeric.h"

int integer_value(const numeric& v);
unsigned long unsigned_value(const numeric& v);
long long long_long_value(const numeric& v);
float float_value(const numeric& v);
double double_value(const numeric& v);
std::string string_value(const numeric& v);

#  ifdef INTERCOM_FMTLIB
#    include <fmt/ostream.h>

namespace fmt {
template <>
struct formatter<numeric> : public formatter<std::string> {
    template <typename FormatContext>
    auto format(const numeric& num, FormatContext& ctx) const -> decltype(ctx.out()) {
        return formatter<std::string>::format(string_value(num), ctx);
    }
};
}  // namespace fmt
#  endif

#endif
