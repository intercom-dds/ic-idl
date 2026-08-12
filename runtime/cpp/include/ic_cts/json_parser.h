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

#include "ic_cts/serialization.h"

#ifndef HISTORY_JSON_PARSER_H
#  define HISTORY_JSON_PARSER_H

#  include <cstdio>
#  include <cstring>
#  include <limits>
#  include <map>
#  include <memory>
#  include <ostream>
#  include <string>
#  include <vector>

#  include "span.h"

#  ifdef _MSC_VER
#    pragma warning(push)
#    pragma warning(disable : 4251)
#  endif

namespace ic_cts {
enum JsonType {
    JSON_FLOAT,
    JSON_INTEGER,
    JSON_STRING,
    JSON_ARRAY,
    JSON_OBJECT,
    JSON_BOOL,
    JSON_NULL,
    JSON_PARSE_ERROR
};

struct JsonData {
    using state_type = JsonData;

    JsonData(const char* str, size_t length) : m_str(str), m_length(length), m_pos(0), m_line(0) {}

    JsonData(const char* str, size_t length, size_t pos, size_t line)
        : m_str(str), m_length(length), m_pos(pos), m_line(line) {}

    int peek() const {
        return (m_pos >= m_length) ? EOF : m_str[m_pos];
    }

    int getc() {
        int c = (m_pos >= m_length) ? EOF : m_str[m_pos++];
        if (c == '\n') {
            ++m_line;
        }
        return c;
    }

    size_t pos() const {
        return m_pos;
    }

    const char* str() const {
        return m_str + pos();
    }

    size_t line() const {
        return m_line;
    }

    size_t length() const {
        return m_length - m_pos;
    }

    state_type current() const {
        return *this;
    }

    JsonData from_state(const state_type& prev) const {
        return {m_str, m_pos, prev.m_pos, prev.m_line};
    }

    char32_t read_unicode();

  private:
    const char* m_str;
    size_t m_length;
    size_t m_pos;
    size_t m_line;
};

struct JsonStream {
    explicit JsonStream(std::istream& stream) : m_stream(stream) {}

    struct state_type {
        JsonData from_state(state_type& prev) const {
            return {parent_->m_data.data(), pos_, prev.pos_, prev.line_};
        }

        size_t line() const {
            return line_;
        }

        const JsonStream* parent_;
        size_t pos_;
        size_t line_;
    };

    int peek() const {
        return m_stream.peek();
    }

    int getc() {
        int c = m_stream.get();
        if (c == '\n') {
            ++m_line;
        }
        if (c != EOF) {
            m_data.push_back(static_cast<char>(c));
        }
        return c;
    }

    size_t line() const {
        return m_line;
    }

    size_t pos() const {
        return m_data.size();
    }

    state_type current() const {
        return {this, m_data.size(), m_line};
    }

    JsonData from_state(state_type& prev) const {
        return {m_data.data(), m_data.size(), prev.pos_, prev.line_};
    }

  private:
    std::istream& m_stream;
    std::string m_data;
    size_t m_line = 0;
};

class JsonNode {
  public:
    JsonNode() : m_type(JSON_NULL), m_data(nullptr, 0), m_value_count(0) {}

    JsonNode(JsonType type, std::string_view str)
        : m_type(type), m_data(str.data(), str.length()), m_value_count(0) {}

    JsonNode(JsonType type, const JsonData& data) : m_type(type), m_data(data), m_value_count(0) {}

    JsonNode(JsonType type, const JsonData& data, size_t value_count)
        : m_type(type), m_data(data), m_value_count(value_count) {}

    static JsonNode from_data(std::string_view str, bool strict = false);

    static JsonNode from_data(JsonData& data, bool strict = false);

    static JsonNode from_data(JsonStream& data, bool strict = false);

    template <typename T>
    bool get_string(T& value) const {
        using string_value_type = typename T::value_type;
        using encoding_type = encoding_type_of<string_value_type>;
        using character_type = typename character_type_of<encoding_type::kind>::type;
        if (m_type == JSON_STRING) {
            value.clear();
            JsonData data(m_data.str(), m_data.length());
            while (data.pos() < m_data.length()) {
                char32_t code = data.read_unicode();
                character_type buf[4] = {0};
                int len = write_char_code<encoding_type::kind>(buf, code);
                for (int i = 0; i < len; ++i) {
                    value.push_back(static_cast<string_value_type>(buf[i]));
                }
            }
            return true;
        }
        return false;
    }

    bool get_array(std::vector<JsonNode>& value) const;

    bool get_object(std::map<std::string, JsonNode>& value) const;

    bool get_object(std::vector<std::pair<JsonNode, JsonNode>>& value) const;

    bool get_object_as_vector(std::vector<JsonNode>& value) const;

    bool get_bool(bool& value) const;

    bool get_number(double& value) const;

    template <typename T>
    bool get_integer(T& value) const {
        value = 0;
        if (m_type != JSON_INTEGER) {
            return false;
        }
        int sign = 1;
        if (m_data.str()[0] == '-') {
            sign = -1;

            if ((std::numeric_limits<T>::min)() == 0) {
                return false;
            }
        } else {
            value = m_data.str()[0] - '0';
        }

        T max = (std::numeric_limits<T>::max)() / 10;
        T lim = ((std::numeric_limits<T>::max)() % 10) + (sign == -1);

        for (size_t i = 1; i < m_data.length(); ++i) {
            T digit = m_data.str()[i] - '0';

            if (value * static_cast<T>(sign) > max ||
                (value * static_cast<T>(sign) == max && digit > lim)) {
                return false;
            }

            value *= 10;
            value += digit * static_cast<T>(sign);
        }
        return true;
    }

    JsonType get_type() const {
        return m_type;
    }

    JsonData get_data() const {
        return m_data;
    }

  private:
    JsonType m_type;
    JsonData m_data;
    size_t m_value_count;
};

class JsonReader : public GenericReader {
  public:
    explicit JsonReader(std::string_view a_text, SerializerFlags a_flags = 0);

    JsonReader(const char* a_text, uint32_t a_length, SerializerFlags a_flags = 0);

    explicit JsonReader(std::istream& a_stream, SerializerFlags a_flags = 0);

    explicit JsonReader(const JsonNode& a_node, SerializerFlags a_flags = 0);

    SerializerFlags flags() const override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

    uint32_t type_level() override;

    bool find_member(const MemberInfo& a_member) override;

    void end_member() override;

    bool can_skip_type() override;

    uint32_t read_length() override;

    void read(std::string& a_value) override;

    void read(std::wstring& a_value) override;

    void read(std::u16string& a_value) override;

    void read(bool* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(float* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(double* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(long double* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(char* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void read(char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void add_const_value(const std::string& a_name, const JsonNode& value);

  private:
    template <typename T>
    void read_integer(T& a_value, const TypeInfo& a_info);
    template <typename T>
    bool read_from_string(T& a_value, const TypeInfo& a_info);

    struct Stack {
        TypeInfo info{};
        JsonNode current;
        std::vector<JsonNode> current_vec;
        std::map<std::string, JsonNode> current_map;
        std::set<std::string> consumed_keys;
        std::string current_string;
        uint32_t count{0};

        void next_value();
    };
    std::unique_ptr<JsonData> m_data;
    std::unique_ptr<JsonStream> m_stream;
    SerializerFlags m_flags{0};
    uint32_t m_level{0};
    std::map<std::string, JsonNode> m_const_map;
    Stack m_type_stack[MAX_NESTED_DEPTH];
};

class JsonWriter : public GenericWriter {
  public:
    explicit JsonWriter(
        std::ostream& out,
        SerializerFlags flags = 0,
        int indent_step = 2,
        int indent_level = 0
    );

    JsonWriter(std::ostream& out, bool pretty, int indent_step = 2, int indent_level = 0);

    SerializerFlags flags() const override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

    uint32_t type_level() override;

    bool is_relevant(const MemberInfo& a_member) override;

    bool begin_member(const MemberInfo& a_member) override;

    bool begin_optional_member(const MemberInfo& a_member, bool a_present) override;

    void end_member() override;

    void write_length(uint32_t a_length) override;

    void write(const std::string& a_value) override;

    void write(const std::wstring& a_value) override;

    void write(const std::u16string& a_value) override;

    void write(const bool* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const float* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const double* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const long double* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const char* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void write(const char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override;

    void reset(int indent_level = 0);

    void start_object();

    void end_object();

    void start_array();

    void end_array();

    std::ostream& stream();

    template <typename T>
    void write_key(const T& key) {
        write_string(key);
        if (!is_key()) {
            put(':');
            if (is_pretty()) {
                put(' ');
            }
            m_needComma = false;
        }
    }

    template <typename T>
    void write_json(const T& value) {
        maybe_comma();
        std::stringstream str;
        str << value;
        for (auto c : str.str()) {
            put(c);
        }
        m_needComma = true;
    }

    void write_string(const std::string& value);

    void write_string(const std::wstring& value);

    void write_string(const std::u16string& value);

    template <typename T>
    void write_integer(T value) {
        maybe_comma();
        if (value == 0) {
            put('0');
        } else {
            char buf[32];
            int len = 0;
            if (value < 0) {
                put('-');
                buf[len++] = -1 * (value % 10) + '0';
                value /= T(-10);
            }
            while (value != 0) {
                buf[len++] = (value % 10) + '0';
                value /= 10;
            }
            while (len > 0) {
                put(buf[--len]);
            }
        }
        m_needComma = true;
    }

    template <typename T>
    void write_unsigned(T value) {
        maybe_comma();
        if (value == 0) {
            put('0');
        } else {
            char buf[32];
            int len = 0;
            while (value != 0) {
                buf[len++] = (value % 10) + '0';
                value /= 10;
            }
            while (len > 0) {
                put(buf[--len]);
            }
        }
        m_needComma = true;
    }

    void write(bool value);

    void write_null();

    void write(unsigned char value);

    void write(char value);

    void write(uint16_t value);

    void write(int16_t value);

    void write(uint32_t value);

    void write(int32_t value);

    void write(uint64_t value);

    void write(int64_t value);

    void write(float value);

    void write(double value);

    void write(long double value);

    void write(const JsonNode& node);

    bool is_pretty() const;

    void set_pretty(bool pretty);

  private:
    template <typename T>
    void write_integer(T& a_value, const TypeInfo& a_info);

    bool is_key() const;

    void maybe_comma();

    void newline();

    void put_escaped(char c);

    void put(char c);

    struct Stack {
        TypeInfo info;
        std::unique_ptr<std::stringstream> tmp_out;
        bool tmp_is_new_line{false};
        uint32_t pos{0};
        bool is_object{false};
    };

    std::ostream& m_out;
    SerializerFlags m_flags{0};
    uint32_t m_indentStep{2U};
    uint32_t m_indentLevel{0U};
    bool m_is_new_line{false};
    bool m_needComma{false};
    Stack m_type_stack[MAX_NESTED_DEPTH]{{}};
    uint32_t m_level{0};
};

}  // namespace ic_cts

#  ifdef _MSC_VER
#    pragma warning(pop)
#  endif

#  include "detail/json_parser.ic"

#endif  // HISTORY_JSON_PARSER_H
