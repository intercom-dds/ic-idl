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

#include <algorithm>
#include <optional>
#include <string>
#include <vector>

namespace intercom::cli::detail {

/// Trims leading and trailing spaces.
inline std::string trim(std::string input) {
    input.erase(0, input.find_first_not_of(' '));
    input.erase(input.find_last_not_of(' ') + 1);
    return input;
}

/// Splits a string at every occurrence of the given delimiter.
/// Returns a list of the split segments.
inline std::vector<std::string> split(std::string str, const std::string& delim) {
    if (str.empty()) {
        return {};
    }
    if (delim.empty()) {
        return {str};
    }
    size_t pos = 0U;
    std::vector<std::string> segments;

    do {
        pos = str.find(delim);
        segments.push_back(trim(str.substr(0, pos)));
        str = str.substr(pos + delim.length());
    } while (pos != std::string::npos);

    return segments;
}

/// Splits a string at every occurrence of the given delimiter.
/// Returns a list of the split segments.
inline std::vector<std::string> split(const std::string& str, const char delim = ',') {
    return split(str, std::string(1, delim));
}

inline std::pair<std::string, std::string>
split_at(const std::string& str, const char delim = ',') {
    size_t pos = str.find(delim);
    std::string left(str.substr(0, pos));
    std::string right(str.substr(pos + 1));
    return {left, right};
}

/// Levenshtein's string approximation algorithm.
/// Returns the distance between two strings.
inline size_t levenshtein(const std::string& a, const std::string& b) {
    const size_t len_a = a.length();
    const size_t len_b = b.length();
    std::vector<size_t> column(len_a);

    for (size_t i = 0; i < len_a + 1; i++) {
        column.push_back(i);
    }

    for (size_t x = 1; x <= len_b; x++) {
        column[0] = x;
        size_t last_diag = x - 1;

        for (size_t y = 1; y <= len_a; y++) {
            const size_t old_diag = column[y];
            const size_t eq = a[y - 1] != b[x - 1];
            const size_t ins_cost = column[y] + 1;
            const size_t sub_cost = column[y - 1] + 1;
            const size_t del_cost = last_diag + eq;

            const size_t min = std::min(ins_cost, sub_cost);
            column[y] = std::min(min, del_cost);
            last_diag = old_diag;
        }
    }

    return column[len_a];
}

/// Compares all the elements in the given container to the given string against
/// an arbitrary criteria, and then returns the closest match (if any).
/// Slightly biased towards strings that are closer in length.
/// This is to prevent e.g. 'vers' from matching with 'help' rather than 'version'
template <typename Container>
inline std::optional<std::string> did_you_mean(const std::string& str, const Container& container) {
    size_t min = SIZE_MAX;
    std::optional<std::string> closest;

    // find the closest match
    for (auto it = container.cbegin(); it != container.cend(); ++it) {
        size_t distance = levenshtein(str, *it) + 2;

        if (distance < min) {
            min = distance;
            const size_t len = str.length();
            const size_t range = (2 * it->length() / 3);
            const size_t suggested = it->length();
            if (min <= range && suggested - range <= len && len <= suggested + range) {
                closest = *it;
            }
        }
    }

    return closest;
}

/// Returns a string that contains `n` spaces.
inline std::string space(size_t n) {
    std::string str;
    str.reserve(n);
    for (size_t i = 0; i < n; i++) {
        str.push_back(' ');
    }
    return str;
}

inline std::string to_lower(std::string str) {
    std::for_each(str.begin(), str.end(), tolower);
    return str;
}

inline std::string replace(std::string str, char a, char b) {
    std::replace(str.begin(), str.end(), a, b);
    return str;
}

template <typename Container>
inline size_t length_of(const Container& container) {
    size_t len = 2 * container.size();
    for (const auto& str : container) {
        len += str.length();
    }
    return len;
}

inline bool len_sort(const std::string& lhs, const std::string& rhs) {
    return lhs.length() < rhs.length();
}

}  // namespace intercom::cli::detail
