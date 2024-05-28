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

#include "InterCOM/serialization.h"

#ifndef HISTORY_JSON_PARSER_H
#  define HISTORY_JSON_PARSER_H

#  include <cstdio>
#  include <cstring>
#  include <limits>
#  include <map>
#  include <ostream>
#  include <string>
#  include <vector>

#  include "InterCOM/CORBA.h"

#  ifdef _MSC_VER
#    pragma warning(push)
#    pragma warning(disable : 4251)
#  endif

namespace intercom {
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

    JsonData(const char* str, size_t length) : str_(str), length_(length), pos_(0), line_(0) {}

    JsonData(const char* str, size_t length, size_t pos, size_t line)
        : str_(str), length_(length), pos_(pos), line_(line) {}

    int peek() const {
        return (pos_ >= length_) ? EOF : str_[pos_];
    }

    int getc() {
        int c = (pos_ >= length_) ? EOF : str_[pos_++];
        if (c == '\n') {
            ++line_;
        }
        return c;
    }

    size_t pos() const {
        return pos_;
    }

    const char* str() const {
        return str_ + pos();
    }

    size_t line() const {
        return line_;
    }

    size_t length() const {
        return length_ - pos_;
    }

    state_type current() const {
        return *this;
    }

    JsonData fromState(const state_type& prev) const {
        return {str_, pos_, prev.pos_, prev.line_};
    }

    Char32 read_unicode();

  private:
    const char* str_;
    size_t length_;
    size_t pos_;
    size_t line_;
};

struct JsonStream {
    JsonStream(std::istream& stream) : stream_(stream), line_(0) {}

    struct state_type {
        JsonData fromState(state_type& prev) const {
            return {parent_->data_.data(), pos_, prev.pos_, prev.line_};
        }

        size_t line() const {
            return line_;
        }

        const JsonStream* parent_;
        size_t pos_;
        size_t line_;
    };

    int peek() const {
        return stream_.peek();
    }

    int getc() {
        int c = stream_.get();
        if (c == '\n') {
            ++line_;
        }
        if (c != EOF) {
            data_.push_back(static_cast<char>(c));
        }
        return c;
    }

    size_t line() const {
        return line_;
    }

    size_t pos() const {
        return data_.size();
    }

    state_type current() const {
        return {this, data_.size(), line_};
    }

    JsonData fromState(state_type& prev) const {
        return {data_.data(), data_.size(), prev.pos_, prev.line_};
    }

  private:
    std::istream& stream_;
    std::string data_;
    size_t line_;
};

class JsonNode {
  public:
    JsonNode() : m_type(JSON_NULL), m_data(nullptr, 0), m_value_count(0) {}

    JsonNode(JsonType type, intercom::string_view str)
        : m_type(type), m_data(str.data(), str.length()), m_value_count(0) {}

    JsonNode(JsonType type, const JsonData& data) : m_type(type), m_data(data), m_value_count(0) {}

    JsonNode(JsonType type, const JsonData& data, size_t value_count)
        : m_type(type), m_data(data), m_value_count(value_count) {}

    static JsonNode from_data(intercom::string_view str);

    static JsonNode from_data(JsonData& data);

    static JsonNode from_data(JsonStream& data);

    template <typename T>
    bool get_string(T& value) const {
        using string_value_type = typename T::value_type;
        using encoding_type = encoding_type_of<string_value_type>;
        using character_type = typename character_type_of<encoding_type::kind>::type;
        if (m_type == JSON_STRING) {
            value.clear();
            JsonData data(m_data.str(), m_data.length());
            while (data.pos() < m_data.length()) {
                Char32 code = data.read_unicode();
                character_type buf[4] = {0};
                int len = writeCharCode<encoding_type::kind>(buf, code);
                for (int i = 0; i < len; ++i) {
                    value.push_back(static_cast<string_value_type>(buf[i]));
                }
            }
            return true;
        }
        return false;
    }

    template <CharacterEncoding ENCODING>
    bool get_string(corba::TString_var<ENCODING>& value) const {
        using value_type = typename corba::TString_var<ENCODING>::value_type;
        if (m_type == JSON_STRING) {
            corba::Sequence<value_type> str;
            JsonData data(m_data.str(), m_data.length());
            while (data.pos() < m_data.length()) {
                Char32 code = data.read_unicode();
                value_type buf[4] = {0};
                int len = writeCharCode<ENCODING>(buf, code);
                for (int i = 0; i < len; ++i) {
                    str.push_back(buf[i]);
                }
            }
            str.push_back(0);
            value = str.get_buffer(true);
            return true;
        } else {
            return false;
        }
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
        T lim = (std::numeric_limits<T>::max)() % 10 + (sign == -1);

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

    inline JsonType get_type() const {
        return m_type;
    }

    inline JsonData get_data() const {
        return m_data;
    }

  private:
    JsonType m_type;
    JsonData m_data;
    size_t m_value_count;
};

class JsonReader : public dcps::cts::GenericReader {
  public:
    explicit JsonReader(intercom::string_view a_text, SerializerFlags a_flags = 0);

    JsonReader(const char* a_text, ULong a_length, SerializerFlags a_flags = 0);

    explicit JsonReader(std::istream& a_stream, SerializerFlags a_flags = 0);

    explicit JsonReader(const JsonNode& a_node, SerializerFlags a_flags = 0);

    SerializerFlags flags() const override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

    ULong type_level() override;

    bool find_member(const MemberInfo& a_member) override;

    void end_member() override;

    bool can_skip_type() override;

    ULong read_length() override;

    void read(corba::EightBitString_var& a_value) override;

    void read(corba::Utf8String_var& a_value) override;

    void read(corba::Utf16String_var& a_value) override;

    void read(corba::Utf32String_var& a_value) override;

    void read(std::string& a_value) override;

    void read(std::wstring& a_value) override;

    void read(std::u16string& a_value) override;

    void read(Boolean* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Int8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(UInt8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Int16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(UInt16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Int32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(UInt32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Int64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(UInt64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Float32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Float64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Float128* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Char8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Char16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void read(Char32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void add_const_value(const std::string& a_name, const JsonNode& value);

  private:
    template <typename T>
    void read_integer(T& a_value, const TypeInfo& a_info);
    template <typename T>
    bool read_from_string(T& a_value, const TypeInfo& a_info);

    struct Stack {
        TypeInfo info;
        JsonNode current;
        std::vector<JsonNode> current_vec;
        std::map<std::string, JsonNode> current_map;
        std::set<std::string> consumed_keys;
        std::string current_string;
        ULong count{0};

        void next_value();
    };
    std::unique_ptr<JsonData> m_data;
    std::unique_ptr<JsonStream> m_stream;
    SerializerFlags m_flags{0};
    ULong m_level{0};
    std::map<std::string, JsonNode> m_const_map;
    Stack m_type_stack[dcps::cts::MAX_NESTED_DEPTH];
};

class JsonWriter : public dcps::cts::GenericWriter {
  public:
    explicit JsonWriter(
        std::ostream& out,
        SerializerFlags flags = 0,
        int indentStep = 2,
        int indentLevel = 0
    );

    JsonWriter(std::ostream& out, bool pretty, int indentStep = 2, int indentLevel = 0);

    SerializerFlags flags() const override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

    ULong type_level() override;

    bool is_relevant(const MemberInfo& a_member) override;

    bool begin_member(const MemberInfo& a_member) override;

    bool begin_optional_member(const MemberInfo& a_member, bool a_present) override;

    void end_member() override;

    void write_length(ULong a_length) override;

    void write(const corba::EightBitString_var& a_value) override;

    void write(const corba::Utf8String_var& a_value) override;

    void write(const corba::Utf16String_var& a_value) override;

    void write(const corba::Utf32String_var& a_value) override;

    void write(const std::string& a_value) override;

    void write(const std::wstring& a_value) override;

    void write(const std::u16string& a_value) override;

    void write(const Boolean* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Int8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const UInt8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Int16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const UInt16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Int32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const UInt32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Int64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const UInt64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Float32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Float64* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Float128* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Char8* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Char16* a_values, ULong a_count, const TypeInfo& a_info) override;

    void write(const Char32* a_values, ULong a_count, const TypeInfo& a_info) override;

    void reset(int indentLevel = 0);

    void startObject();

    void endObject();

    void startArray();

    void endArray();

    std::ostream& stream();

    template <typename T>
    void writeKey(const T& key) {
        writeString(key);
        if (!is_key()) {
            put(':');
            if (isPretty()) {
                put(' ');
            }
            m_needComma = false;
        }
    }

    template <typename T>
    void writeJson(const T& value) {
        maybeComma();
        std::stringstream str;
        str << value;
        for (auto c : str.str()) {
            put(c);
        }
        m_needComma = true;
    }

    void writeString(const std::string& value);

    void writeString(const std::wstring& value);

    void writeString(const std::u16string& value);

    template <CharacterEncoding ENCODING>
    void writeString(const corba::TString_var<ENCODING>& value) {
        using value_type = typename corba::TString_var<ENCODING>::value_type;
        maybeComma();
        put('"');
        const value_type* str = value.in();
        while (*str) {
            Char32 code = 0;
            str += readCharCode<ENCODING>(code, str);
            char buf[4];
            int len = writeCharCode<UTF8>(buf, code);
            if (len == 1) {
                putEscaped(buf[0]);
            } else {
                for (int i = 0; i < len; ++i) {
                    put(buf[i]);
                }
            }
        }
        put('"');
        m_needComma = true;
    }

    template <typename T>
    void writeInteger(T value) {
        maybeComma();
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
    void writeUnsigned(T value) {
        maybeComma();
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

    void writeNull();

    void write(unsigned char value);

    void write(char value);

    void write(UShort value);

    void write(Short value);

    void write(ULong value);

    void write(Long value);

    void write(ULongLong value);

    void write(LongLong value);

    void write(Float value);

    void write(Double value);

    void write(LongDouble value);

    void write(const JsonNode& node);

    bool isPretty() const;

    void setPretty(bool pretty);

  private:
    template <typename T>
    void write_integer(T& a_value, const TypeInfo& a_info);

    bool is_key() const;

    void maybeComma();

    void newline();

    void putEscaped(char c);

    void put(char c);

    struct Stack {
        TypeInfo info;
        std::unique_ptr<std::stringstream> tmp_out;
        bool tmp_is_new_line{false};
        ULong pos{0};
        bool is_object{false};
    };

    std::ostream& m_out;
    SerializerFlags m_flags{0};
    ULong m_indentStep{2U};
    ULong m_indentLevel{0U};
    bool m_is_new_line{false};
    bool m_needComma{false};
    Stack m_type_stack[dcps::cts::MAX_NESTED_DEPTH]{{}};
    ULong m_level{0};
};

}  // namespace intercom

#  ifdef _MSC_VER
#    pragma warning(pop)
#  endif

#  include "detail/json_parser.ic"

#endif  // HISTORY_JSON_PARSER_H
