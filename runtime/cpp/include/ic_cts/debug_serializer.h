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

#include <cstddef>
#include <cstdint>
#include <iomanip>
#include <limits>
#include <locale>
#include <ostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <type_traits>
#include <vector>

#include "ic_cts/character_encoding.h"
#include "ic_cts/serialization.h"

namespace ic_cts {

class DebugWriter {
  public:
    explicit DebugWriter(std::ostream& stream, bool pretty = false)
        : m_stream(stream), m_pretty(pretty) {}

    SerializerFlags flags() const {
        return m_pretty ? SERIALIZER_PRETTY : SerializerFlagsBits{};
    }

    void begin_type(const TypeInfo& info) {
        if (m_stack.size() >= MAX_NESTED_DEPTH) {
            throw std::runtime_error("Cannot serialize type, nesting too deep");
        }

        before_value();
        switch (info.kind) {
        case dcps::xtypes::TK_STRUCTURE:
        case dcps::xtypes::TK_UNION:
            m_stream << type_name(info.name) << " {";
            break;
        case dcps::xtypes::TK_MAP:
            m_stream.put('{');
            break;
        case dcps::xtypes::TK_SEQUENCE:
        case dcps::xtypes::TK_ARRAY:
            m_stream.put('[');
            break;
        default:
            break;
        }

        if (is_container(info.kind)) {
            ++m_indent_level;
        }

        m_stack.push_back({info.kind, 0, false});
    }

    void end_type() {
        const auto frame = m_stack.back();
        m_stack.pop_back();

        if (is_container(frame.kind)) {
            --m_indent_level;
            if (m_pretty && frame.position != 0) {
                m_stream.put(',');
                newline();
            }
        }

        switch (frame.kind) {
        case dcps::xtypes::TK_STRUCTURE:
        case dcps::xtypes::TK_UNION:
            m_stream << (m_pretty || frame.position == 0 ? "}" : " }");
            break;
        case dcps::xtypes::TK_MAP:
            m_stream.put('}');
            break;
        case dcps::xtypes::TK_SEQUENCE:
        case dcps::xtypes::TK_ARRAY:
            m_stream.put(']');
            break;
        default:
            break;
        }
    }

    uint32_t type_level() const {
        return static_cast<uint32_t>(m_stack.size());
    }

    bool is_relevant(const MemberInfo&) const {
        return true;
    }

    bool begin_member(const MemberInfo& member) {
        auto& frame = m_stack.back();
        next_element(frame.position++);
        if (!m_pretty && frame.position == 1) {
            m_stream.put(' ');
        }

        m_stream << member.name << ": ";
        frame.optional = false;
        return true;
    }

    bool begin_optional_member(const MemberInfo& member, bool present) {
        begin_member(member);
        m_stack.back().optional = present;
        m_stream << (present ? "Some(" : "None");
        if (present && m_pretty) {
            ++m_indent_level;
            newline();
        }

        return present;
    }

    void end_member() {
        if (m_stack.back().optional) {
            if (m_pretty) {
                --m_indent_level;
                m_stream.put(',');
                newline();
            }

            m_stream.put(')');
            m_stack.back().optional = false;
        }
    }

    void write_length(uint32_t) {}

    template <typename CharT>
    void write(const std::basic_string<CharT>& value) {
        before_value();
        write_quoted(value, '"');
    }

    template <typename T>
    void write(const T* values, uint32_t count, const TypeInfo& info) {
        for (uint32_t i = 0; i < count; ++i) {
            before_value();
            if constexpr (std::is_same_v<T, bool>) {
                m_stream << (values[i] ? "true" : "false");
            } else if constexpr (std::is_same_v<T, char>) {
                m_stream.put('\'');
                write_escaped(static_cast<unsigned char>(values[i]), '\'', true);
                m_stream.put('\'');
            } else if constexpr (
                std::is_same_v<T, char16_t> || std::is_same_v<T, char32_t> ||
                std::is_same_v<T, wchar_t>
            ) {
                write_quoted(std::basic_string<T>(1, values[i]), '\'');
            } else if constexpr (std::is_integral_v<T>) {
                write_integer(values[i], info);
            } else {
                write_number(values[i]);
            }
        }
    }

  private:
    struct Frame {
        uint16_t kind;
        size_t position;
        bool optional;
    };

    static bool is_container(uint16_t kind) {
        return kind == dcps::xtypes::TK_STRUCTURE || kind == dcps::xtypes::TK_UNION ||
               kind == dcps::xtypes::TK_MAP || kind == dcps::xtypes::TK_SEQUENCE ||
               kind == dcps::xtypes::TK_ARRAY;
    }

    void newline() {
        m_stream.put('\n');
        for (size_t i = 0; i < m_indent_level; ++i) {
            m_stream << "    ";
        }
    }

    void next_element(size_t position) {
        if (position != 0) {
            m_stream.put(',');
        }

        if (m_pretty) {
            newline();
        } else if (position != 0) {
            m_stream.put(' ');
        }
    }

    static std::string_view type_name(const char* name) {
        const std::string_view scoped = name ? name : "";
        const auto separator = scoped.rfind("::");
        return separator == std::string_view::npos ? scoped : scoped.substr(separator + 2);
    }

    void before_value() {
        if (m_stack.empty()) {
            return;
        }

        auto& frame = m_stack.back();
        switch (frame.kind) {
        case dcps::xtypes::TK_MAP:
            if (frame.position % 2 != 0) {
                m_stream << ": ";
            } else {
                next_element(frame.position);
            }

            ++frame.position;
            break;
        case dcps::xtypes::TK_SEQUENCE:
        case dcps::xtypes::TK_ARRAY:
            next_element(frame.position++);
            break;
        default:
            break;
        }
    }

    template <typename T>
    void write_number(T value) {
        std::ostringstream text;
        text.imbue(std::locale::classic());
        if constexpr (std::is_floating_point_v<T>) {
            text << std::setprecision(std::numeric_limits<T>::max_digits10);
        }

        text << +value;
        m_stream << text.str();
    }

    template <typename T>
    void write_integer(T value, const TypeInfo& info) {
        if (info.kind != dcps::xtypes::TK_ENUM && info.kind != dcps::xtypes::TK_BITMASK) {
            write_number(value);
            return;
        }

        std::string label;
        const bool named = enum_to_string(label, value, &info);
        m_stream << type_name(info.name);
        if (info.kind == dcps::xtypes::TK_ENUM && named) {
            m_stream << "::" << label;
            return;
        }

        m_stream.put('(');
        if (named && !label.empty()) {
            for (const char c : label) {
                if (c == '|') {
                    m_stream << " | ";
                } else {
                    m_stream.put(c);
                }
            }
        } else {
            write_number(value);
        }

        m_stream.put(')');
    }

    template <typename CharT>
    void write_quoted(const std::basic_string<CharT>& value, char quote) {
        m_stream.put(quote);
        const auto* current = value.c_str();
        const auto* end = current + value.size();
        while (current < end) {
            char32_t code = 0;
            current += read_char_code(code, current);
            write_escaped(code, quote);
        }

        m_stream.put(quote);
    }

    void write_escaped(char32_t code, char quote, bool byte = false) {
        constexpr char digits[] = "0123456789abcdef";
        if (byte && code >= 0x80) {
            m_stream << "\\x";
            m_stream.put(digits[code / 16]);
            m_stream.put(digits[code % 16]);
            return;
        }

        switch (code) {
        case '\0':
            m_stream << "\\0";
            return;
        case '\n':
            m_stream << "\\n";
            return;
        case '\r':
            m_stream << "\\r";
            return;
        case '\t':
            m_stream << "\\t";
            return;
        case '\\':
            m_stream << "\\\\";
            return;
        default:
            break;
        }

        if (code == static_cast<char32_t>(quote)) {
            m_stream.put('\\');
            m_stream.put(quote);
        } else if (code < 0x20 || code == 0x7f) {
            m_stream << "\\u{";
            if (code >= 16) {
                m_stream.put(digits[code / 16]);
            }

            m_stream.put(digits[code % 16]);
            m_stream.put('}');
        } else {
            char bytes[4];
            const auto length = write_char_code<UTF8>(bytes, code);
            m_stream.write(bytes, length);
        }
    }

    std::ostream& m_stream;
    bool m_pretty;
    size_t m_indent_level{0};
    std::vector<Frame> m_stack;
};

template <typename T>
std::ostream& marshal_debug(std::ostream& stream, const T& value, bool pretty = false) {
    DebugWriter writer(stream, pretty);
    TGenericMarshal<DebugWriter>(writer).io(value);
    return stream;
}

}
