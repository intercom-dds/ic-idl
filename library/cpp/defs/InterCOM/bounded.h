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
#include <map>
#include <string>
#include <vector>

namespace intercom {

template <typename CharT, uint32_t N>
class bounded_basic_string final : public std::basic_string<CharT> {
  public:
    using std::basic_string<CharT>::basic_string;
    static constexpr uint32_t max_size() {
        return N;
    };

    template <uint32_t S>
    bounded_basic_string(const bounded_basic_string<CharT, S>& other);
    template <uint32_t S>
    bounded_basic_string(bounded_basic_string<CharT, S>&& other);
    template <uint32_t S>
    bounded_basic_string& operator=(const bounded_basic_string<CharT, S>& other);
    template <uint32_t S>
    bounded_basic_string& operator=(bounded_basic_string<CharT, S>&& other);

    bounded_basic_string() = default;
    bounded_basic_string(const CharT* other);
    bounded_basic_string(const std::basic_string<CharT>& other);
    bounded_basic_string(std::basic_string<CharT>&& other);
    bounded_basic_string& operator=(const CharT* other);
    bounded_basic_string& operator=(const std::basic_string<CharT>& other);
    bounded_basic_string& operator=(std::basic_string<CharT>&& other);
};

template <typename K, typename V, uint32_t N>
class bounded_map final : public std::map<K, V> {
  public:
    using std::map<K, V>::map;
    static constexpr uint32_t max_size() {
        return N;
    };

    template <uint32_t S>
    bounded_map(const bounded_map<K, V, S>& other);
    template <uint32_t S>
    bounded_map(bounded_map<K, V, S>&& other);
    template <uint32_t S>
    bounded_map& operator=(const bounded_map<K, V, S>& other);
    template <uint32_t S>
    bounded_map& operator=(bounded_map<K, V, S>&& other);

    bounded_map() = default;
    bounded_map(const std::map<K, V>& other);
    bounded_map(std::map<K, V>&& other);
    bounded_map& operator=(const std::map<K, V>& other);
    bounded_map& operator=(std::map<K, V>&& other);
};

template <typename T, uint32_t N>
class bounded_vector final : public std::vector<T> {
  public:
    using std::vector<T>::vector;
    static constexpr uint32_t max_size() {
        return N;
    };

    template <uint32_t S>
    bounded_vector(const bounded_vector<T, S>& other);
    template <uint32_t S>
    bounded_vector(bounded_vector<T, S>&& other);
    template <uint32_t S>
    bounded_vector& operator=(const bounded_vector<T, S>& other);
    template <uint32_t S>
    bounded_vector& operator=(bounded_vector<T, S>&& other);

    bounded_vector() = default;
    bounded_vector(const std::vector<T>& other);
    bounded_vector(std::vector<T>&& other);
    bounded_vector& operator=(const std::vector<T>& other);
    bounded_vector& operator=(std::vector<T>&& other);
};

template <uint32_t N>
using bounded_string = bounded_basic_string<char, N>;

template <uint32_t N>
using bounded_u16string = bounded_basic_string<char16_t, N>;

template <uint32_t N>
using bounded_u32string = bounded_basic_string<char32_t, N>;

template <uint32_t N>
using bounded_wstring = bounded_basic_string<wchar_t, N>;

}  // namespace intercom

#include "detail/bounded.ic"  // IWYU pragma: export
