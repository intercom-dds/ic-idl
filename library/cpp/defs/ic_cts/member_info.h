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
#include <vector>

#include "ic_cts/bounded.h"

namespace ic_cts {

struct TypeInfo;

template <class Archive, typename T>
struct Serializer;

template <typename T>
struct TypeTraits;

struct MemberInfo {
    uint32_t id;
    const char* name;
    uint32_t flags;
    const int32_t* case_labels;
    const TypeInfo* type;

    /// Complex default value, json encoded
    const char* default_value;
};

struct TypeInfo {
    const char* name;
    uint16_t kind;
    uint32_t flags;
    uint32_t bit_size;
    int32_t value_offset;
    uint32_t max_length;

    /// Primitive default value
    void* default_value;

    /// Primitive min value
    void* min_value;

    /// Primitive max value
    void* max_value;

    const TypeInfo* key_type;
    const TypeInfo* element_type;
    uint32_t member_count;
    MemberInfo* members;
};

#define TYPETRAITS_PRIMITIVE(type)               \
    template <>                                  \
    struct TypeTraits<type> {                    \
        using value_type = type;                 \
        using sequence_type = std::vector<type>; \
        static const TypeInfo type_info;         \
    };

TYPETRAITS_PRIMITIVE(bool);
TYPETRAITS_PRIMITIVE(char);
TYPETRAITS_PRIMITIVE(char16_t);
TYPETRAITS_PRIMITIVE(int8_t);
TYPETRAITS_PRIMITIVE(uint8_t);
TYPETRAITS_PRIMITIVE(int16_t);
TYPETRAITS_PRIMITIVE(uint16_t);
TYPETRAITS_PRIMITIVE(int32_t);
TYPETRAITS_PRIMITIVE(uint32_t);
TYPETRAITS_PRIMITIVE(int64_t);
TYPETRAITS_PRIMITIVE(uint64_t);
TYPETRAITS_PRIMITIVE(float);
TYPETRAITS_PRIMITIVE(double);
TYPETRAITS_PRIMITIVE(long double);

namespace detail {
template <typename T>
struct dimensions_inner {
    static constexpr uint32_t value = 0;  // NOLINT
};

template <typename T, size_t N>
struct dimensions_inner<std::array<T, N>> {
    static constexpr uint32_t value = dimensions_inner<T>::value + 1;  // NOLINT
};

template <typename T>
using array_dimensions = std::integral_constant<uint32_t, dimensions_inner<T>::value>;
}  // namespace detail

template <typename T>
struct TypeTraits<std::vector<T>> {
    using value_type = T;
    using element_traits = TypeTraits<value_type>;
    using is_bounded = std::false_type;
};

template <typename T, uint32_t N>
struct TypeTraits<bounded_vector<T, N>> {
    using value_type = T;
    using element_traits = TypeTraits<value_type>;
    using is_bounded = std::true_type;
    using bound = std::integral_constant<uint32_t, N>;
};

template <typename CharT>
struct TypeTraits<std::basic_string<CharT>> {
    using value_type = CharT;
    using element_traits = TypeTraits<value_type>;
    using is_bounded = std::false_type;
};

template <typename T, uint32_t N>
struct TypeTraits<bounded_basic_string<T, N>> {
    using value_type = T;
    using element_traits = TypeTraits<value_type>;
    using is_bounded = std::true_type;
    using bound = std::integral_constant<uint32_t, N>;
};

template <typename T, size_t N>
struct TypeTraits<std::array<T, N>> {
    using value_type = T;
    using element_traits = TypeTraits<value_type>;
    using dimensions = detail::array_dimensions<std::array<T, N>>;
    using bound = std::integral_constant<uint32_t, N>;
};

template <typename K, typename V>
struct TypeTraits<std::map<K, V>> {
    using key_traits = TypeTraits<K>;
    using value_traits = TypeTraits<V>;
    using is_bounded = std::false_type;
};

template <typename K, typename V, uint32_t N>
struct TypeTraits<bounded_map<K, V, N>> {
    using key_traits = TypeTraits<K>;
    using value_traits = TypeTraits<V>;
    using is_bounded = std::true_type;
    using bound = std::integral_constant<uint32_t, N>;
};

}  // namespace ic_cts

#include "detail/member_info.ic"  // IWYU pragma: export
