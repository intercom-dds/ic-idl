// Copyright 2026 KONGSBERG
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

#include "annotations.h"
#include "any_type.h"
#include "bitmasks.h"
#include "bounded_types.h"
#include "circular_types.h"
#include "constants.h"
#include "deep_generics.h"
#include "defaults.h"
#include "enums.h"
#include "exceptions.h"
#include "interfaces.h"
#include "multi_module.h"
#include "nested_modules.h"
#include "string_types.h"
#include "structs.h"
#include "typedefs.h"
#include "unions.h"
#include "valuetypes.h"

char16_t unicode_character_constant(void) {
    return char_wstring_types_WCHAR_OMEGA;
}

const char* unicode_string_constant(void) {
    return unicode_types_JAPANESE unicode_types_EMOJI;
}

const char16_t* wide_string_constant(void) {
    return char_wstring_types_WSTRING_UNICODE char_wstring_types_WSTRING_EMOJI;
}

float integral_float_constant(void) {
    return constant_types_FLOAT_INTEGRAL;
}

_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->input,
        idl_status_t (*)(void*, const int32_t*, idl_error_t*): true,
        default: false
    ),
    "input array must decay to a const element pointer"
);
_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->output,
        idl_status_t (*)(void*, int32_t*, idl_error_t*): true,
        default: false
    ),
    "output array must decay to an element pointer"
);
_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->update,
        idl_status_t (*)(void*, int32_t(*)[3], idl_error_t*): true,
        default: false
    ),
    "multidimensional array must preserve inner dimensions"
);
_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->strings,
        idl_status_t (*)(void*, const char* const*, idl_error_t*): true,
        default: false
    ),
    "string array must not duplicate const qualification"
);
_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->get_items,
        idl_status_t (*)(void*, int32_t*, idl_error_t*): true,
        default: false
    ),
    "array attribute getter must accept an element pointer"
);
_Static_assert(
    _Generic(
        ((interface_types_WithArrayParams*)0)->set_items,
        idl_status_t (*)(void*, const int32_t*, idl_error_t*): true,
        default: false
    ),
    "array attribute setter must accept a const element pointer"
);

_Static_assert(
    _Generic(
        ((interface_types_WithAliasedCollections*)0)->consume_items,
        idl_status_t (*)(void*, const idl_sequence_t*, idl_error_t*): true,
        default: false
    ),
    "sequence alias input must use the sequence handle ABI"
);
_Static_assert(
    _Generic(
        ((interface_types_WithAliasedCollections*)0)->set_lookup,
        idl_status_t (*)(void*, const idl_map_t*, idl_error_t*): true,
        default: false
    ),
    "map alias setter must use the map handle ABI"
);
_Static_assert(
    _Generic(
        ((interface_types_WithAliasedCollections*)0)->set_payload,
        idl_status_t (*)(void*, const idl_any_t*, idl_error_t*): true,
        default: false
    ),
    "any alias setter must use the any handle ABI"
);

_Static_assert(
    _Generic(((annotation_types_OptionalStruct*)0)->optional_int, int32_t*: true, default: false),
    "optional integer must be a pointer"
);
_Static_assert(
    _Generic(
        ((annotation_types_OptionalStruct*)0)->optional_string,
        const char*: true,
        default: false
    ),
    "optional string must remain a pointer"
);
_Static_assert(
    _Generic(
        ((annotation_types_OptionalStruct*)0)->optional_seq,
        idl_sequence_t*: true,
        default: false
    ),
    "optional sequence must remain a pointer"
);

bool optional_members_are_null(void) {
    annotation_types_OptionalStruct value = {0};

    return value.optional_int == NULL && value.optional_string == NULL &&
           value.optional_seq == NULL;
}
