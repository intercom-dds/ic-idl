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

#include <cctype>

#include "InterCOM/string_view.h"

namespace intercom {
namespace icgen {

enum class Case {
    Snake,
    Camel,
    Pascal,
    Kebab,
};

/// Helper class for performing case conversion of strings.
/// The input string can be converted to:
///   - snake_case
///   - camelCase
///   - PascalCase
///   - kebab-case
class CaseConverter {
  public:
    explicit CaseConverter(Case casing) : m_case(casing) {
        if (m_case == Case::Kebab) {
            m_delim = '-';
        }
    }

    void set_delim(char delim) {
        m_delim = delim;
    }

    std::string convert(intercom::string_view input) {
        m_first = true;
        to_case(input);
        return std::move(m_result);
    }

  private:
    static char lower(char c) {
        return std::tolower(c, std::locale());
    }

    static char upper(char c) {
        return std::toupper(c, std::locale());
    }

    void to_snake(intercom::string_view word) {
        if (!m_first) {
            m_result += m_delim;
        }
        for (const auto& c : word) {
            m_result += lower(c);
        }
    }

    void to_camel(intercom::string_view word) {
        m_result += m_first ? lower(word[0]) : upper(word[0]);
        for (size_t i = 1; i < word.size(); i++) {
            m_result += lower(word[i]);
        }
    }

    void to_pascal(intercom::string_view word) {
        m_result += upper(word[0]);
        for (size_t i = 1; i < word.size(); i++) {
            m_result += lower(word[i]);
        }
    }

    void write(intercom::string_view word) {
        if (word.empty()) {
            return;
        }
        switch (m_case) {
        case Case::Snake:
        case Case::Kebab:
            to_snake(word);
            break;
        case Case::Camel:
            to_camel(word);
            break;
        case Case::Pascal:
            to_pascal(word);
            break;
        }
        m_first = false;
    }

    constexpr static bool is_delim(char c) {
        return c == '_' || c == '-' || isspace(c);
    }

    void to_case(intercom::string_view input) {
        size_t start = 0;
        bool was_upper = false;

        for (size_t i = 0; i < input.size(); i++) {
            auto c = input[i];
            if (is_delim(c)) {
                if (start == i) {
                    start++;
                }
                continue;
            }

            if (i + 1 < input.size()) {
                auto peek = input[i + 1];
                auto len = i - start;

                if (is_delim(peek) || (islower(c) && isupper(peek))) {
                    write(input.substr(start, len + 1));
                    start = i + 1;
                } else if (was_upper && isupper(c) && islower(peek)) {
                    write(input.substr(start, len));
                    start = i;
                }
            } else {
                return write(input.substr(start));
            }
            was_upper = isupper(c) != 0;
        }
    }

  private:
    bool m_first = true;
    char m_delim = '_';
    Case m_case;
    std::string m_result;
};

/// Converts the given string to snake_case.
inline std::string snake_case(intercom::string_view input) {
    CaseConverter conv(Case::Snake);
    return conv.convert(input);
}

/// Converts the given string to camelCase.
inline std::string camel_case(intercom::string_view input) {
    CaseConverter conv(Case::Camel);
    return conv.convert(input);
}

/// Converts the given string to PascalCase.
inline std::string pascal_case(intercom::string_view input) {
    CaseConverter conv(Case::Pascal);
    return conv.convert(input);
}

/// Converts the given string to kebab-case.
inline std::string kebab_case(intercom::string_view input) {
    CaseConverter conv(Case::Kebab);
    return conv.convert(input);
}

}  // namespace icgen
}  // namespace intercom
