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

#include "InterCOM/dyn_link.h"

#ifdef __cplusplus
extern "C" {
#endif

enum PlacementKind { BEGIN_FILE, BEFORE_DECLARATION, BEGIN_DECLARATION, END_DECLARATION, AFTER_DECLARATION, END_FILE };

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

struct parser;

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

    N_CASE,       // A case value inside a union
    N_NULL,       // An explicit null node
    N_MEMBER,     // A member variable in a module, enum, bitset, bitmask, struct, union or valuetype
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
    /// enum member with explicit value, != the implicit value e.g. "enum E { V0, V1 = 1, ENUMERATED = 5, V3 };"
    /// \note also applied to the enum node, if any members have it
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

struct position {
    int line;
    int column;
};

struct identifier {
    const char* name;
    struct position pos;
};

struct ptree;

void clear_namespace_nodes(void);

void pragma_arg(const char* pragma);

void pragma_finish(void);

void switch_file(char* filename, char* included_as, int is_system_include);

void add_comment(const char* text);

void reset_comment(void);

void comment_complete(struct position first, struct position last);

extern enum node_kind ANY_KIND[];

extern int ZERO_BOUNDS;

INTERCOM_PUBLIC extern struct numeric num_undef;

const struct numeric* expr_convert(const struct numeric* value, enum numeric_kind kind);

const struct numeric* expr_unary(char op, const struct numeric* v);

const struct numeric* expr_binary(char op, const struct numeric* v1, const struct numeric* v2);

int idlerror(const char* msg);

int idlwarning(const char* msg);

extern struct position current_pos;

extern const char* current_input_file;

INTERCOM_PUBLIC extern struct ptree boolean_type;
INTERCOM_PUBLIC extern struct ptree int8_type;
INTERCOM_PUBLIC extern struct ptree octet_type;
INTERCOM_PUBLIC extern struct ptree char_type;
INTERCOM_PUBLIC extern struct ptree wchar_type;
INTERCOM_PUBLIC extern struct ptree short_type;
INTERCOM_PUBLIC extern struct ptree ushort_type;
INTERCOM_PUBLIC extern struct ptree long_type;
INTERCOM_PUBLIC extern struct ptree ulong_type;
INTERCOM_PUBLIC extern struct ptree longlong_type;
INTERCOM_PUBLIC extern struct ptree ulonglong_type;
INTERCOM_PUBLIC extern struct ptree float_type;
INTERCOM_PUBLIC extern struct ptree double_type;
INTERCOM_PUBLIC extern struct ptree ldouble_type;
INTERCOM_PUBLIC extern struct ptree fixed_type;
INTERCOM_PUBLIC extern struct ptree unbounded_string_type;
INTERCOM_PUBLIC extern struct ptree unbounded_wstring_type;
INTERCOM_PUBLIC extern struct ptree any_type;
INTERCOM_PUBLIC extern struct ptree object_type;

INTERCOM_PUBLIC extern struct ptree* annotation_type_id;
INTERCOM_PUBLIC extern struct ptree* annotation_type_autoid;
INTERCOM_PUBLIC extern struct ptree* annotation_type_optional;
INTERCOM_PUBLIC extern struct ptree* annotation_type_position;
INTERCOM_PUBLIC extern struct ptree* annotation_type_value;
INTERCOM_PUBLIC extern struct ptree* annotation_type_empty;
INTERCOM_PUBLIC extern struct ptree* annotation_type_extensibility;
INTERCOM_PUBLIC extern struct ptree* annotation_type_final;
INTERCOM_PUBLIC extern struct ptree* annotation_type_mutable;
INTERCOM_PUBLIC extern struct ptree* annotation_type_appendable;
INTERCOM_PUBLIC extern struct ptree* annotation_type_shared;
INTERCOM_PUBLIC extern struct ptree* annotation_type_key;
INTERCOM_PUBLIC extern struct ptree* annotation_type_must_understand;
INTERCOM_PUBLIC extern struct ptree* annotation_type_default;
INTERCOM_PUBLIC extern struct ptree* annotation_type_default_literal;
INTERCOM_PUBLIC extern struct ptree* annotation_type_range;
INTERCOM_PUBLIC extern struct ptree* annotation_type_min;
INTERCOM_PUBLIC extern struct ptree* annotation_type_max;
INTERCOM_PUBLIC extern struct ptree* annotation_type_unit;
INTERCOM_PUBLIC extern struct ptree* annotation_type_bit_bound;
INTERCOM_PUBLIC extern struct ptree* annotation_type_external;
INTERCOM_PUBLIC extern struct ptree* annotation_type_nested;
INTERCOM_PUBLIC extern struct ptree* annotation_type_verbatim;
INTERCOM_PUBLIC extern struct ptree* annotation_type_service;
INTERCOM_PUBLIC extern struct ptree* annotation_type_topic;
INTERCOM_PUBLIC extern struct ptree* annotation_type_dds_service;
INTERCOM_PUBLIC extern struct ptree* annotation_type_dds_request_topic;
INTERCOM_PUBLIC extern struct ptree* annotation_type_dds_reply_topic;
INTERCOM_PUBLIC extern struct ptree* annotation_type_oneway;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ami;
INTERCOM_PUBLIC extern struct ptree* annotation_type_bitset_old;
INTERCOM_PUBLIC extern struct ptree* annotation_type_bit_bound_old;
INTERCOM_PUBLIC extern struct ptree* annotation_type_must_understand_old;
INTERCOM_PUBLIC extern struct ptree* annotation_type_minimum_type_check_old;
INTERCOM_PUBLIC extern struct ptree* annotation_type_hashid;
INTERCOM_PUBLIC extern struct ptree* annotation_type_default_nested;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ignore_literal_names;
INTERCOM_PUBLIC extern struct ptree* annotation_type_try_construct;
INTERCOM_PUBLIC extern struct ptree* annotation_type_non_serialized;
INTERCOM_PUBLIC extern struct ptree* annotation_type_data_representation;
INTERCOM_PUBLIC extern struct ptree* annotation_type_doc;
INTERCOM_PUBLIC extern struct ptree* annotation_type_merge;
INTERCOM_PUBLIC extern struct ptree* annotation_type_const;
INTERCOM_PUBLIC extern struct ptree* annotation_type_static;
INTERCOM_PUBLIC extern struct ptree* annotation_type_derive;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_rename;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_builder;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_doc;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_minimum_type_check;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_suppress;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_no_constructor;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_no_serializer;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_listener;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_length_bit_bound;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_value_offset;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_length_value_offset;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_repeat_count;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_vmf_xri;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_vmf_decimal;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_string_constants;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_jaus_presence_vector;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_jaus_integer;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_jaus_integer_function;
INTERCOM_PUBLIC extern struct ptree* annotation_type_ext_protobuf_type;
INTERCOM_PUBLIC extern struct ptree* annotation_type_jaus;

#ifdef __cplusplus
};  // extern C

#  include <string>

#  include "cidl/numeric.h"

INTERCOM_PUBLIC int integer_value(const numeric& v);
INTERCOM_PUBLIC unsigned long unsigned_value(const numeric& v);
INTERCOM_PUBLIC long long long_long_value(const numeric& v);
INTERCOM_PUBLIC float float_value(const numeric& v);
INTERCOM_PUBLIC double double_value(const numeric& v);
INTERCOM_PUBLIC std::string string_value(const numeric& v);

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
