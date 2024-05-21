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

#include <array>
#include <cassert>
#include <cctype>
#include <cstdio>
#include <functional>
#include <map>
#include <string>
#include <vector>

#include "InterCOM/string_view.h"

namespace intercom {
namespace icgen {

// case-insensitive comparison of keys
struct kw_comp {
    struct case_comp {
        bool operator()(char lhs, char rhs) const {
            return tolower(lhs) < tolower(rhs);
        }
    };

    bool operator()(intercom::string_view lhs, intercom::string_view rhs) const {
        return std::lexicographical_compare(
            lhs.begin(), lhs.end(), rhs.begin(), rhs.end(), case_comp()
        );
    }
};

enum class TokenKind {
    // Logical operators
    And,
    Or,
    Not,

    // If statements
    If,
    Else,
    Elif,
    EndIf,

    // For-each loops
    For,
    In,
    EndFor,

    // Code block delimiters
    LBracket,
    RBracket,

    // Parentheses for group expressions and function calls
    LParen,
    RParen,

    Dot,
    Comma,
    Semi,
    String,
    Eq, // =
    EqEq, // ==
    NotEq, // !=

    // Everything outside of code blocks is regarded as text, including
    // whitespace. Will be reproduced in its entirety by the template engine,
    // so it's left as-is and not split into individual tokens.
    Text,

    // Leading/trailing whitespace of code blocks will be stripped if the code
    // block expands to nothing.
    Whitespace,

    // Variable or function identifier
    Ident,

    // An invalid token is an invalid identifier. Instead of throwing an
    // error in the lexer, we let the parser handle it as it's capable of
    // producing more sensible error messages.
    Invalid,

    // Last token
    Eof,
};

static const std::array<std::string, 26> TOKEN_NAMES = {
    "and",    "or", "not", "if",   "else", "elif",       "endif",   "for",       "in",
    "endfor", "[[", "]]",  "(",    ")",    "dot",        "comma",   "semicolon", "string",
    "=",      "==", "!=",  "text", "ws",   "identifier", "invalid", "eof",
};

static_assert(
    std::tuple_size<decltype(TOKEN_NAMES)>::value == size_t(TokenKind::Eof) + 1,
    "token count mismatch"
);

static const std::map<intercom::string_view, TokenKind, kw_comp> KEYWORDS = {
    {"if", TokenKind::If},     {"else", TokenKind::Else},
    {"elif", TokenKind::Elif}, {"endif", TokenKind::EndIf},
    {"and", TokenKind::And},   {"or", TokenKind::Or},
    {"not", TokenKind::Not},   {"for", TokenKind::For},
    {"in", TokenKind::In},     {"endfor", TokenKind::EndFor},
};

struct Position {
    const size_t line;
    const size_t col;
    const size_t index;

    constexpr Position(size_t line, size_t col, size_t index)
        : line(line)
        , col(col)
        , index(index) {}
};

struct Token {
    intercom::string_view view;
    TokenKind kind;
    size_t line{};
    size_t col{};
    size_t index{};

    constexpr Token(intercom::string_view view, TokenKind kind, size_t line, size_t col, size_t index)
            : view(view), kind(kind), line(line), col(col), index(index) {}
};

// NOLINTNEXTLINE
constexpr static const Token Eof = Token(intercom::string_view(), TokenKind::Eof, 0, 0, 0);

inline constexpr bool operator==(const Token& lhs, TokenKind kind) {
    return lhs.kind == kind;
}

inline constexpr bool operator!=(const Token& lhs, TokenKind kind) {
    return lhs.kind != kind;
}

/// Performs a single pass of the input view and generates tokens.
/// The lexer will only create individual tokens for lexemes inside code blocks;
/// lexemes that appear outside of code blocks, regardless of whether they are
/// keywords or not, will be grouped into a single `Text` token.
///
/// Does not perform any copies of the underlying buffer. The lifetime of
/// each token is bound by the lifetime of the input buffer.
class Lexer {
  public:
    explicit Lexer(intercom::string_view view)
        : m_view(view) {}

    std::vector<Token> scan() {
        while (get() != EOF) {
            if (is_block_start()) {
                scan_block();
            } else if (is_comment()) {
                scan_comment();
            } else {
                scan_text();
            }
        }
        return std::move(m_tokens);
    }

  private:
    char get() const {
        return m_idx < m_view.length() ? m_view[m_idx] : EOF; // NOLINT
    }

    char peek() const {
        return (m_idx + 1) < m_view.length() ? m_view[m_idx + 1] : EOF; // NOLINT
    }

    size_t index() const {
        return m_idx;
    }

    bool is_block_start() const {
        return get() == '[' && (peek() == '[' || peek() == '%');
    }

    bool is_block_end() const {
        return (get() == ']' || get() == '%') && peek() == ']';
    }

    bool is_comment() const {
        return (get() == '#' || get() == '[') && peek() == '#';
    }

    intercom::string_view slice(size_t start, size_t count = 0) const {
        return count == 0 ? m_view.substr(start, index() - start)
                          : m_view.substr(start, count);
    }

    static bool is_ident(char c) {
        return isalnum(c) || c == '_';
    }

    void emplace_token(intercom::string_view text, TokenKind kind) {
        m_tokens.emplace_back(text, kind, m_line, m_col - text.length(), m_idx - text.length());
    }

    /// Advances to the next non-whitespace character
    char next() {
        while (get() != EOF && isspace(get())) {
            advance();
        }
        return get();
    }

    void advance(size_t n = 1) {
        for (size_t i = 0; i < n; i++) {
            m_idx++;

            if (get() == '\n') {
                m_line += 1;
                m_col = 1;
            } else {
                m_col++;
            }
        }
    }

    intercom::string_view take_while(const std::function<bool(char)>& predicate) {
        auto start = index();
        while (get() != EOF && predicate(get())) {
            advance();
        }
        return slice(start);
    }

    intercom::string_view take_until(char c) {
        auto start = index();
        while (get() != EOF && get() != c) {
            advance();
        }
        return slice(start);
    }

    void scan_string() {
        assert(!is_block_start());
        assert(!is_block_end());

        auto c = next();
        advance();
        auto str = take_until(c);
        emplace_token(str, TokenKind::String);
        advance();
    }

    /// `Text` is a group of words, including whitespace.
    /// To maintain whitespace for variables that expand to multiple lines,
    /// a `Text` token may not span across multiple lines.
    void scan_text() {
        assert(!is_block_start());
        bool is_ws = true;

        auto text = take_while([&](char) -> bool {
            if (get() == '\n') {
                advance();
                return false;
            }
            if(is_comment() || is_block_start()) {
                return false;
            }
            is_ws &= (isspace(get()) != 0);
            return true;
        });
        emplace_token(text, is_ws ? TokenKind::Whitespace : TokenKind::Text);
    }

    /// Identifiers must be alphanumeric, and may contain underscores.
    void scan_ident() {
        assert(!is_block_start());
        assert(!is_block_end());

        auto ident = take_while(is_ident);
        auto it = KEYWORDS.find(ident);
        auto kind = it != KEYWORDS.end() ? it->second : TokenKind::Ident;
        emplace_token(ident, kind);
    }

    /// Comments are skipped by the lexer.
    void scan_comment() {
        if (get() == '#') {
            // single-line comment
            take_while([&](char) -> bool {
                return get() != EOF && get() != '\n';
            });
            advance(1);
        } else {
            // multi-line comment
            take_while([&](char) -> bool {
                return get() != EOF && (get() != '#' || peek() != ']');
            });
            advance(3);
        }
    }

    void scan_invalid() {
        auto view = take_while([&](char c) -> bool {
            return !isspace(c) && !is_block_end() && c != ')';
        });
        emplace_token(view, TokenKind::Invalid);
    }

    /// Scans a single block, e.g. `[[variable]]` or `[% if x %]`.
    /// Blocks do not care about whitespace.
    void scan_block() {
        // consume the leading [[ or [%
        assert(is_block_start());
        advance(2);

        while (next() != EOF && !is_block_end()) {
            if (is_ident(get())) {
                scan_ident();
            } else if (is_comment()) {
                scan_comment();
            } else if (get() == '\'' || get() == '"') {
                scan_string();
            } else if (get() == '(') {
                emplace_token(slice(index(), 1), TokenKind::LParen);
                advance();
            } else if (get() == ')') {
                emplace_token(slice(index(), 1), TokenKind::RParen);
                advance();
            } else if (get() == '.') {
                emplace_token(slice(index(), 1), TokenKind::Dot);
                advance();
            } else if (get() == ',') {
                emplace_token(slice(index(), 1), TokenKind::Comma);
                advance();
            } else if (get() == ';') {
                emplace_token(slice(index(), 1), TokenKind::Semi);
                advance();
            } else if (get() == '!' && peek() == '=') {
                emplace_token(slice(index(), 2), TokenKind::NotEq);
                advance(2);
            } else if (get() == '=' && peek() == '=') {
                emplace_token(slice(index(), 2), TokenKind::EqEq);
                advance(2);
            } else if (get() == '=') {
                emplace_token(slice(index(), 1), TokenKind::Eq);
                advance();
            } else {
                scan_invalid();
            }
        }

        // consume the trailing ]] or %]
        assert(is_block_end());
        auto end = get();
        emplace_token(slice(index(), 2), TokenKind::RBracket);
        advance(2);

        // consume the following newline, but only if the next char is whitespace.
        // usually you want some space around code blocks.
        if (get() == '\n' && end == '%') {
            advance();
        }
    }

  private:
    size_t m_line{1};
    size_t m_col{};
    size_t m_idx{};
    intercom::string_view m_view;
    std::vector<Token> m_tokens;
};

inline std::vector<Token> tokenize(intercom::string_view input) {
    Lexer lexer(input);
    return lexer.scan();
}

inline char handle_escaped(char c) {
    switch (c) {
    case '\\':
        return c;
    case '"':
        return '"';
    case '/':
        return '/';
    case 'b':
        return '\b';
    case 'f':
        return '\f';
    case 'n':
        return '\n';
    case 'r':
        return '\r';
    case 't':
        return '\t';
    default:
        return c;
    }
}

inline std::string escape_str(intercom::string_view input) {
    std::string data;
    for(size_t i = 0; i < input.length(); i++) {
        if (input[i] == '\\' && (i + 1 < input.length())) {
            data.push_back(handle_escaped(input[i + 1]));
            i++;
        } else {
            data.push_back(input[i]);
        }
    }
    return data;
}
} // namespace icgen
} // namespace intercom
