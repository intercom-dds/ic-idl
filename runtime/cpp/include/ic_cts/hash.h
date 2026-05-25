// Copyright 2025 KONGSBERG
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

#include <array>
#include <cstddef>
#include <functional>
#include <map>
#include <optional>
#include <vector>

namespace ic_cts {

template <class T>
inline void hash_combine(std::size_t& seed, const T& x);

template <typename T>
inline void hash_combine(std::size_t& seed, const std::vector<T>& x);

inline void hash_combine(std::size_t& seed, const std::vector<bool>& x);

template <typename T, std::size_t N>
inline void hash_combine(std::size_t& seed, const std::array<T, N>& x);

template <typename K, typename V>
inline void hash_combine(std::size_t& seed, const std::map<K, V>& x);

template <class T>
inline void hash_combine(std::size_t& seed, const std::optional<T>& x);

template <class T>
inline void hash_combine(std::size_t& seed, const T& x) {
    seed ^= std::hash<T>{}(x) + static_cast<std::size_t>(0x9e3779b97f4a7c15ULL) + (seed << 6) +
            (seed >> 2);
}

template <typename T>
inline void hash_combine(std::size_t& seed, const std::vector<T>& x) {
    hash_combine(seed, x.size());
    for (const auto& elem : x) {
        hash_combine(seed, elem);
    }
}

inline void hash_combine(std::size_t& seed, const std::vector<bool>& x) {
    hash_combine(seed, x.size());
    for (bool elem : x) {
        hash_combine(seed, elem);
    }
}

template <typename T, std::size_t N>
inline void hash_combine(std::size_t& seed, const std::array<T, N>& x) {
    hash_combine(seed, N);
    for (const auto& elem : x) {
        hash_combine(seed, elem);
    }
}

template <typename K, typename V>
inline void hash_combine(std::size_t& seed, const std::map<K, V>& x) {
    hash_combine(seed, x.size());
    for (const auto& [key, value] : x) {
        hash_combine(seed, key);
        hash_combine(seed, value);
    }
}

template <class T>
inline void hash_combine(std::size_t& seed, const std::optional<T>& x) {
    hash_combine(seed, x.has_value());
    if (x.has_value()) {
        hash_combine(seed, *x);
    }
}

template <typename... Args>
inline std::size_t hash_all(const Args&... args) {
    std::size_t seed = 0;
    ((hash_combine(seed, args)), ...);
    return seed;
}

}  // namespace ic_cts
