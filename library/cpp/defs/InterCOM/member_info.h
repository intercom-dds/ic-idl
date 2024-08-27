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

#include <cstdint>

namespace intercom {

struct TypeInfo;

template <class Archive, typename T>
struct Serializer;

struct MemberInfo {
    uint32_t id;
    const char* name;
    uint32_t flags;
    const int32_t* case_labels;
    const TypeInfo* type;
    const char* default_value;  // Complex default value, json encoded
};

struct TypeInfo {
    const char* name;
    uint16_t kind;
    uint32_t flags;
    uint32_t bit_size;
    int32_t value_offset;
    uint32_t max_length;
    void* default_value;  // Primitive default value
    void* min_value;      // Primitive min value
    void* max_value;      // Primitive max value
    const TypeInfo* key_type;
    const TypeInfo* element_type;
    uint32_t member_count;
    MemberInfo* members;
};

extern const TypeInfo Null_type_info;
extern const TypeInfo Char_type_info;
extern const TypeInfo Char16_type_info;
extern const TypeInfo Char32_type_info;
extern const TypeInfo Boolean_type_info;
extern const TypeInfo Octet_type_info;
extern const TypeInfo Int8_type_info;
extern const TypeInfo Uint8_type_info;
extern const TypeInfo Short_type_info;
extern const TypeInfo UShort_type_info;
extern const TypeInfo Long_type_info;
extern const TypeInfo ULong_type_info;
extern const TypeInfo LongLong_type_info;
extern const TypeInfo ULongLong_type_info;
extern const TypeInfo Float_type_info;
extern const TypeInfo Double_type_info;
extern const TypeInfo LongDouble_type_info;
extern const TypeInfo Null_seq_type_info;
extern const TypeInfo Null_map_type_info;
extern const int32_t* MemberInfo_empty_case_labels;

}  // namespace intercom
