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

#pragma once

#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "member_info.h"

namespace omg::types {

using string = std::string;
using string_view = std::string_view;
template <size_t N>
using bounded_string = ::ic_cts::bounded_string<N>;

using wstring = std::wstring;
using wstring_view = std::wstring_view;
template <size_t N>
using bounded_wstring = ::ic_cts::bounded_wstring<N>;

template <typename T, size_t N>
using array = std::array<T, N>;

template <typename T>
using sequence = std::vector<T>;

template <typename T, size_t N>
using bounded_sequence = ::ic_cts::bounded_vector<T, N>;

template <typename Key, typename T>
using map = std::map<Key, T>;

template <typename T>
using ref_type = std::shared_ptr<T>;

template <typename T>
using weak_ref_type = std::weak_ptr<T>;

template <typename T>
using optional = std::optional<T>;

// Type traits aliases

template <typename T>
struct value_type {
    using type = typename ::ic_cts::TypeTraits<T>::value_type;
};
template <typename T>
using value_type_t = typename value_type<T>::type;

template <typename T>
struct in_type {
    using type = typename ::ic_cts::TypeTraits<T>::in_type;
};
template <typename T>
using in_type_t = typename in_type<T>::type;

template <typename T>
struct out_type {
    using type = typename ::ic_cts::TypeTraits<T>::out_type;
};
template <typename T>
using out_type_t = typename out_type<T>::type;

template <typename T>
struct inout_type {
    using type = typename ::ic_cts::TypeTraits<T>::inout_type;
};
template <typename T>
using inout_type_t = typename inout_type<T>::type;

template <typename T>
struct is_bounded {
    using value = typename ::ic_cts::TypeTraits<T>::is_bounded;
};
template <typename T>
constexpr auto is_bounded_v = typename is_bounded<T>::value();

template <typename T>
struct bound {
    using value = typename ::ic_cts::TypeTraits<T>::bound;
};
template <typename T>
constexpr auto bound_v = typename bound<T>::value();

template <typename T>
struct bit_bound {
    using value = typename ::ic_cts::TypeTraits<T>::bit_bound;
};
template <typename T>
constexpr auto bit_bound_v = typename bit_bound<T>::value();

template <typename T>
struct underlying_type {
    using type = typename ::ic_cts::TypeTraits<T>::underlying_type;
};
template <typename T>
using underlying_type_t = typename underlying_type<T>::type;

template <typename T>
struct dimensions {
    using value = typename ::ic_cts::TypeTraits<T>::dimensions;
};
template <typename T>
constexpr auto dimensions_v = typename dimensions<T>::value();

template <typename T>
struct key {
    using type = typename ::ic_cts::TypeTraits<T>::key_type;
};
template <typename T>
using key_t = typename key<T>::type;

template <typename T>
struct elements {
    using type = typename ::ic_cts::TypeTraits<T>::element_type;
};
template <typename T>
using elements_t = typename elements<T>::type;

}  // namespace omg::types
