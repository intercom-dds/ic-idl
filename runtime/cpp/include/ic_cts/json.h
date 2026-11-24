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
#include <initializer_list>
#include <istream>
#include <iterator>
#include <map>
#include <memory>
#include <ostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <variant>
#include <vector>

#include "ic_cts/character_encoding.h"
#include "ic_cts/json_parser.h"
#include "ic_cts/json_serializer.h"

namespace ic_cts {

class JsonValue;

namespace detail {

inline void write_json_value(JsonWriter& a_writer, const JsonValue& a_value);

inline void read_json_value(const JsonNode& a_node, JsonValue& a_value);

}  // namespace detail

class JsonValue {
  public:
    bool operator<(const JsonValue& a_other) const {
        return m_data < a_other.m_data;
    }

    bool operator==(const JsonValue& a_other) const {
        return m_data == a_other.m_data;
    }

    bool operator!=(const JsonValue& a_other) const {
        return !(*this == a_other);
    }

    bool operator>(const JsonValue& a_other) const {
        return a_other < *this;
    }

    bool operator<=(const JsonValue& a_other) const {
        return !(a_other < *this);
    }

    bool operator>=(const JsonValue& a_other) const {
        return !(*this < a_other);
    }

    double& float_value() {
        return member<double>("float_value");
    }

    double float_value() const {
        return member<double>("float_value");
    }

    void float_value(double a_value) {
        m_data.emplace<double>(a_value);
    }

    int64_t& integer_value() {
        return member<int64_t>("integer_value");
    }

    int64_t integer_value() const {
        return member<int64_t>("integer_value");
    }

    void integer_value(int64_t a_value) {
        m_data.emplace<int64_t>(a_value);
    }

    std::string& string_value() {
        return member<std::string>("string_value");
    }

    const std::string& string_value() const {
        return member<std::string>("string_value");
    }

    void string_value(std::string a_value) {
        m_data.emplace<std::string>(std::move(a_value));
    }

    std::vector<JsonValue>& array_value() {
        return member<std::vector<JsonValue>>("array_value");
    }

    const std::vector<JsonValue>& array_value() const {
        return member<std::vector<JsonValue>>("array_value");
    }

    std::map<std::string, JsonValue>& object_value() {
        return member<std::map<std::string, JsonValue>>("object_value");
    }

    const std::map<std::string, JsonValue>& object_value() const {
        return member<std::map<std::string, JsonValue>>("object_value");
    }

    bool& bool_value() {
        return member<bool>("bool_value");
    }

    bool bool_value() const {
        return member<bool>("bool_value");
    }

    void bool_value(bool a_value) {
        m_data.emplace<bool>(a_value);
    }

  private:
    friend class JsonRef;
    friend class Json;
    friend void detail::write_json_value(JsonWriter& a_writer, const JsonValue& a_value);
    friend void detail::read_json_value(const JsonNode& a_node, JsonValue& a_value);

    enum JsonValueType {
        JSON_VALUE_NULL,
        JSON_VALUE_FLOAT,
        JSON_VALUE_INTEGER,
        JSON_VALUE_STRING,
        JSON_VALUE_ARRAY,
        JSON_VALUE_OBJECT,
        JSON_VALUE_BOOL
    };

    JsonValueType _d() const {
        return static_cast<JsonValueType>(m_data.index());
    }

    void _d(JsonValueType discriminator) {
        if (discriminator == _d()) {
            return;
        }

        switch (discriminator) {
        case JSON_VALUE_FLOAT:
            m_data.emplace<double>();
            break;
        case JSON_VALUE_INTEGER:
            m_data.emplace<int64_t>();
            break;
        case JSON_VALUE_STRING:
            m_data.emplace<std::string>();
            break;
        case JSON_VALUE_ARRAY:
            m_data.emplace<std::vector<JsonValue>>();
            break;
        case JSON_VALUE_OBJECT:
            m_data.emplace<std::map<std::string, JsonValue>>();
            break;
        case JSON_VALUE_BOOL:
            m_data.emplace<bool>();
            break;
        default:
            m_data.emplace<std::monostate>();
            break;
        }
    }

    template <typename T>
    T& member(const char* a_name) {
        T* value = std::get_if<T>(&m_data);
        if (value == nullptr) {
            throw std::logic_error(std::string("Union JsonValue not set to value ") + a_name);
        }
        return *value;
    }

    template <typename T>
    const T& member(const char* a_name) const {
        const T* value = std::get_if<T>(&m_data);
        if (value == nullptr) {
            throw std::logic_error(std::string("Union JsonValue not set to value ") + a_name);
        }
        return *value;
    }

    std::variant<
        std::monostate,
        double,
        int64_t,
        std::string,
        std::vector<JsonValue>,
        std::map<std::string, JsonValue>,
        bool>
        m_data;
};

namespace detail {

inline void write_json_value(JsonWriter& a_writer, const JsonValue& a_value) {
    switch (a_value._d()) {
    case JsonValue::JSON_VALUE_FLOAT:
        a_writer.write(a_value.float_value());
        break;
    case JsonValue::JSON_VALUE_INTEGER:
        a_writer.write(a_value.integer_value());
        break;
    case JsonValue::JSON_VALUE_STRING:
        a_writer.write_string(a_value.string_value());
        break;
    case JsonValue::JSON_VALUE_ARRAY:
        a_writer.start_array();
        for (const auto& element : a_value.array_value()) {
            write_json_value(a_writer, element);
        }
        a_writer.end_array();
        break;
    case JsonValue::JSON_VALUE_OBJECT:
        a_writer.start_object();
        for (const auto& member : a_value.object_value()) {
            a_writer.write_key(member.first);
            write_json_value(a_writer, member.second);
        }
        a_writer.end_object();
        break;
    case JsonValue::JSON_VALUE_BOOL:
        a_writer.write(a_value.bool_value());
        break;
    default:
        a_writer.write_null();
        break;
    }
}

inline void read_json_value(const JsonNode& a_node, JsonValue& a_value) {
    switch (a_node.get_type()) {
    case JSON_FLOAT: {
        double value = 0.0;
        if (!a_node.get_number(value)) {
            throw std::runtime_error("Cannot deserialize type, failed to read float");
        }
        a_value.float_value(value);
    } break;
    case JSON_INTEGER: {
        int64_t value = 0;
        if (!a_node.get_integer(value)) {
            throw std::runtime_error("Failed to read integer value");
        }
        a_value.integer_value(value);
    } break;
    case JSON_STRING: {
        std::string value;
        if (!a_node.get_string(value)) {
            throw std::runtime_error("Cannot deserialize type, failed to read string");
        }
        a_value.string_value(std::move(value));
    } break;
    case JSON_ARRAY: {
        std::vector<JsonNode> nodes;
        a_node.get_array(nodes);

        std::vector<JsonValue> elements(nodes.size());
        for (size_t i = 0; i < nodes.size(); ++i) {
            read_json_value(nodes[i], elements[i]);
        }

        a_value._d(JsonValue::JSON_VALUE_ARRAY);
        a_value.array_value() = std::move(elements);
    } break;
    case JSON_OBJECT: {
        std::map<std::string, JsonNode> nodes;
        a_node.get_object(nodes);

        std::map<std::string, JsonValue> members;
        for (const auto& node : nodes) {
            read_json_value(node.second, members[node.first]);
        }

        a_value._d(JsonValue::JSON_VALUE_OBJECT);
        a_value.object_value() = std::move(members);
    } break;
    case JSON_BOOL: {
        bool value = false;
        a_node.get_bool(value);
        a_value.bool_value(value);
    } break;
    case JSON_NULL:
        a_value._d(JsonValue::JSON_VALUE_NULL);
        break;
    case JSON_PARSE_ERROR:
        throw std::runtime_error("Invalid JSON data supplied to JsonReader");
    }
}

}  // namespace detail

inline std::ostream& operator<<(std::ostream& out, const JsonValue& value) {
    JsonWriter writer(out);
    detail::write_json_value(writer, value);
    return out;
}

inline std::istream& operator>>(std::istream& in, JsonValue& value) {
    JsonStream stream(in);
    detail::read_json_value(JsonNode::from_data(stream), value);
    return in;
}

class JsonRef {
  public:
    using value_type = JsonRef;
    using pointer = value_type*;
    using const_pointer = const value_type*;
    using reference = value_type&;
    using const_reference = const value_type&;

    std::string ref() const {
        if (m_parent) {
            return m_parent->ref() + "/" + m_name;
        }
        return "#";
    }

    void ref(std::string_view name) {
        operator[]("$ref") = name;
    }

    void ref(const JsonRef& a_other) {
        ref(a_other.ref());
    }

    template <typename JsonValue_T>
    class IteratorImpl {
      public:
        using iterator_category = std::forward_iterator_tag;
        using value_type = JsonRef::value_type;
        using reference =
            std::conditional_t<std::is_const_v<JsonValue_T>, const value_type&, value_type&>;
        using pointer =
            std::conditional_t<std::is_const_v<JsonValue_T>, const value_type*, value_type*>;
        using array_iterator = std::conditional_t<
            std::is_const_v<JsonValue_T>,
            typename std::vector<JsonValue>::const_iterator,
            typename std::vector<JsonValue>::iterator>;
        using object_iterator = std::conditional_t<
            std::is_const_v<JsonValue_T>,
            typename std::map<std::string, JsonValue>::const_iterator,
            typename std::map<std::string, JsonValue>::iterator>;

        explicit IteratorImpl(const object_iterator& a_map, const JsonRef* parent) {
            m_it.m = a_map;
            m_parent = std::make_shared<const JsonRef>(*parent);
            m_kind = JsonValue::JSON_VALUE_OBJECT;
        }

        explicit IteratorImpl(const array_iterator& a_vector, const JsonRef* parent, size_t idx) {
            m_it.v = a_vector;
            m_parent = std::make_shared<const JsonRef>(*parent);
            m_it.array_index = idx;
            m_kind = JsonValue::JSON_VALUE_ARRAY;
        }

        bool operator==(const IteratorImpl& a_other) const {
            if (m_kind != a_other.m_kind || m_parent->m_value != a_other.m_parent->m_value) {
                return false;
            }
            switch (m_kind) {
            case JsonValue::JSON_VALUE_OBJECT:
                return m_it.m == a_other.m_it.m;
            case JsonValue::JSON_VALUE_ARRAY:
                return m_it.v == a_other.m_it.v;
            default:
                throw std::runtime_error("Invalid value type for json iterator");
            }
        }

        bool operator!=(const IteratorImpl& a_other) const {
            return !operator==(a_other);
        }

        value_type operator*() {
            switch (m_kind) {
            case JsonValue::JSON_VALUE_OBJECT:
                return value_type(const_cast<JsonValue*>(&m_it.m->second), key(), m_parent.get());
            case JsonValue::JSON_VALUE_ARRAY:
                return value_type(const_cast<JsonValue*>(&(*m_it.v)), key(), m_parent.get());
            default:
                throw std::runtime_error("Invalid value type for json iterator");
            }
        }

        std::string key() {
            switch (m_kind) {
            case JsonValue::JSON_VALUE_OBJECT:
                return m_it.m->first;
            case JsonValue::JSON_VALUE_ARRAY:
                return std::to_string(m_it.array_index);
            default:
                throw std::runtime_error("Invalid value type for json iterator");
            }
        }

        std::string ref() {
            return m_parent->ref() + "/" + key();
        }

        IteratorImpl operator++(int) {
            auto result = *this;
            ++(*this);
            return result;
        }

        IteratorImpl& operator++() {
            switch (m_kind) {
            case JsonValue::JSON_VALUE_OBJECT:
                ++m_it.m;
                break;
            case JsonValue::JSON_VALUE_ARRAY:
                ++m_it.array_index;
                ++m_it.v;
                break;
            default:
                throw std::runtime_error("Invalid value type for json iterator");
            }
            return *this;
        }

      private:
        struct {
            size_t array_index{0};
            array_iterator v;
            object_iterator m;
        } m_it;
        std::shared_ptr<const JsonRef> m_parent;
        JsonValue::JsonValueType m_kind;
    };

    using iterator = IteratorImpl<JsonValue>;
    using const_iterator = IteratorImpl<const JsonValue>;

    explicit JsonRef(JsonValue* value, std::string_view name = "", const JsonRef* parent = nullptr)
        : m_value(value), m_name(name) {
        if (parent) {
            m_parent = std::make_unique<JsonRef>(*parent);
        }
    }
    JsonRef(const JsonRef& a_other) : m_value(a_other.m_value), m_name(a_other.m_name) {
        if (a_other.m_parent) {
            m_parent = std::make_unique<JsonRef>(*a_other.m_parent);
        }
    }
    JsonRef& operator=(const JsonRef& a_other) {
        if (this != &a_other) {
            JsonValue value = *a_other.m_value;
            *m_value = std::move(value);
        }
        return *this;
    }

    JsonRef operator[](std::string_view a_key) {
        if (a_key == "#") {
            if (m_parent) {
                return m_parent->operator[](a_key);
            }
            return *this;
        }

        size_t sep_pos = a_key.find('/');
        if (sep_pos != std::string::npos && a_key.substr(0, sep_pos) == "#") {
            if (m_parent) {
                return m_parent->operator[](a_key);
            }
            return operator[](a_key.substr(sep_pos + 1));
        }

        if (is_null()) {
            m_value->_d(JsonValue::JSON_VALUE_OBJECT);
        }
        if (sep_pos != std::string::npos) {
            auto it = find(a_key.substr(0, sep_pos));
            if (it != end()) {
                return (*it).operator[](a_key.substr(sep_pos + 1));
            }
            throw std::runtime_error(std::string("Unknown json path ") + std::string(a_key));
        }
        if (is_array()) {
            auto it = find(a_key);
            if (it != end()) {
                return *it;
            }
            throw std::runtime_error(std::string("Unknown json path ") + std::string(a_key));
        }
        auto& obj = m_value->object_value();
        return JsonRef(&obj[std::string(a_key)], a_key, this);
    }

    JsonRef operator[](size_t a_offset) {
        auto& obj = m_value->array_value()[a_offset];
        return JsonRef(&obj, std::to_string(a_offset), this);
    }

    JsonRef operator[](int a_offset) {
        return operator[](static_cast<size_t>(a_offset));
    }

    iterator at(int a_offset) {
        if (a_offset < 0 || a_offset > static_cast<int>(size())) {
            throw std::runtime_error("Index out of bounds in JsonRef::at");
        }
        return iterator(
            m_value->array_value().begin() + a_offset, this, static_cast<size_t>(a_offset)
        );
    }

    iterator find(std::string_view a_key) {
        size_t sep_pos = a_key.find('/');
        if (sep_pos != std::string::npos) {
            if (a_key.substr(0, sep_pos) == "#") {
                if (m_parent) {
                    auto result = m_parent->find(a_key);
                    if ((is_object() || is_array()) && result == m_parent->end()) {
                        return end();
                    }
                    return result;
                }
                return find(a_key.substr(sep_pos + 1));
            }
            auto it = find(a_key.substr(0, sep_pos));
            if (it != end()) {
                JsonRef child = *it;
                auto result = child.find(a_key.substr(sep_pos + 1));
                if (result == child.end()) {
                    return end();
                }
                return result;
            }
            throw std::runtime_error(std::string("Unknown json path ") + std::string(a_key));
        }
        if (is_object()) {
            return iterator(m_value->object_value().find(std::string(a_key)), this);
        }
        if (is_array()) {
            int index;
            JsonReader reader(a_key, 0);
            GenericUnmarshal(reader).io(index);
            return at(index);
        }
        throw std::runtime_error(
            "Attempting to find subelement in JsonRef that is not an object or an array"
        );
    }

    void update_references(std::string_view old_prefix) {
        update_references(old_prefix, ref());
    }

    void update_references(std::string_view old_prefix, std::string_view new_prefix) {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            for (auto&& child : *this) {
                child.update_references(old_prefix, new_prefix);
            }
            break;
        case JsonValue::JSON_VALUE_OBJECT: {
            auto ref_it = find("$ref");
            if (ref_it != end() && (*ref_it).is_string()) {
                std::string& str = (*ref_it).json_value().string_value();
                if (str.size() >= old_prefix.size() &&
                    str.substr(0, old_prefix.size()) == old_prefix) {
                    bool at_boundary = old_prefix.empty() || old_prefix.back() == '/' ||
                                       str.size() == old_prefix.size() ||
                                       str[old_prefix.size()] == '/';
                    if (at_boundary) {
                        str = std::string(new_prefix) + str.substr(old_prefix.size());
                    }
                }
            }
            for (auto&& child : *this) {
                child.update_references(old_prefix, new_prefix);
            }
        } break;
        default:
            break;
        }
    }

    iterator begin() {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return iterator(m_value->array_value().begin(), this, 0);
        case JsonValue::JSON_VALUE_OBJECT:
            return iterator(m_value->object_value().begin(), this);
        default:
            throw std::runtime_error("Invalid value type for json iterator");
        }
    }

    iterator end() {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return iterator(m_value->array_value().end(), this, m_value->array_value().size());
        case JsonValue::JSON_VALUE_OBJECT:
            return iterator(m_value->object_value().end(), this);
        default:
            throw std::runtime_error("Invalid value type for json iterator");
        }
    }

    const_iterator begin() const {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return const_iterator(m_value->array_value().begin(), this, 0);
        case JsonValue::JSON_VALUE_OBJECT:
            return const_iterator(m_value->object_value().begin(), this);
        default:
            throw std::runtime_error("Invalid value type for json iterator");
        }
    }

    const_iterator end() const {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return const_iterator(
                m_value->array_value().end(), this, m_value->array_value().size()
            );
        case JsonValue::JSON_VALUE_OBJECT:
            return const_iterator(m_value->object_value().end(), this);
        default:
            throw std::runtime_error("Invalid value type for json iterator");
        }
    }

    size_t size() const {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return m_value->array_value().size();
        case JsonValue::JSON_VALUE_OBJECT:
            return m_value->object_value().size();
        case JsonValue::JSON_VALUE_STRING:
            return m_value->string_value().size();
        default:
            return 0;
        }
    }

    bool empty() const {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            return m_value->array_value().empty();
        case JsonValue::JSON_VALUE_OBJECT:
            return m_value->object_value().empty();
        case JsonValue::JSON_VALUE_STRING:
            return m_value->string_value().empty();
        case JsonValue::JSON_VALUE_NULL:
            return true;
        default:
            return false;
        }
    }

    void clear() {
        switch (m_value->_d()) {
        case JsonValue::JSON_VALUE_ARRAY:
            m_value->array_value().clear();
            break;
        case JsonValue::JSON_VALUE_OBJECT:
            m_value->object_value().clear();
            break;
        case JsonValue::JSON_VALUE_STRING:
            m_value->string_value().clear();
            break;
        default:
            m_value->_d(JsonValue::JSON_VALUE_NULL);
        }
    }

    void reset() {
        m_value->_d(JsonValue::JSON_VALUE_NULL);
    }

    bool is_null() const {
        return m_value->_d() == JsonValue::JSON_VALUE_NULL;
    }
    bool is_float() const {
        return m_value->_d() == JsonValue::JSON_VALUE_FLOAT;
    }
    bool is_integer() const {
        return m_value->_d() == JsonValue::JSON_VALUE_INTEGER;
    }
    bool is_string() const {
        return m_value->_d() == JsonValue::JSON_VALUE_STRING;
    }
    bool is_array() const {
        return m_value->_d() == JsonValue::JSON_VALUE_ARRAY;
    }
    bool is_object() const {
        return m_value->_d() == JsonValue::JSON_VALUE_OBJECT;
    }
    bool is_bool() const {
        return m_value->_d() == JsonValue::JSON_VALUE_BOOL;
    }

    template <typename T>
    void push_back(const T& a_value) {
        JsonValue element;
        auto json_value = JsonRef(&element);
        json_value = a_value;

        if (m_value->_d() == JsonValue::JSON_VALUE_NULL) {
            m_value->_d(JsonValue::JSON_VALUE_ARRAY);
        }
        m_value->array_value().push_back(std::move(element));
    }

    template <typename T, typename... Ts>
    void emplace(std::string_view a_key, const T& a_value, Ts... args) {
        if (m_value->_d() == JsonValue::JSON_VALUE_NULL) {
            m_value->_d(JsonValue::JSON_VALUE_OBJECT);
        }
        auto& obj = m_value->object_value();
        JsonRef base(&obj[std::string(a_key)]);
        base = a_value;
        emplace(args...);
    }

    template <typename T, typename... Ts>
    void emplace_back(const T& a_value, Ts... args) {
        push_back(a_value);
        emplace_back(args...);
    }

    void operator=(bool a_value) {
        m_value->bool_value(a_value);
    }
    void operator=(int8_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(uint8_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(int16_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(uint16_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(int32_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(uint32_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(int64_t a_value) {
        m_value->integer_value(a_value);
    }
    void operator=(uint64_t a_value) {
        m_value->integer_value(static_cast<int64_t>(a_value));
    }
    void operator=(float a_value) {
        m_value->float_value(a_value);
    }
    void operator=(double a_value) {
        m_value->float_value(a_value);
    }
    void operator=(const std::string& a_value) {
        m_value->string_value(a_value);
    }
    void operator=(const char* a_value) {
        m_value->string_value(a_value);
    }
    void operator=(std::string_view a_value) {
        m_value->string_value(std::string(a_value));
    }
    void operator=(std::u16string_view a_value) {
        std::u16string u16_str(a_value);
        std::string u8_str;
        const char16_t* current = u16_str.c_str();
        const char16_t* end_of_string = current + u16_str.size();
        while (current < end_of_string) {
            char32_t code = 0;
            current += read_char_code(code, current);
            char buf[4];
            int len = write_char_code<UTF8>(buf, code);
            u8_str.append(buf, static_cast<size_t>(len));
        }
        m_value->string_value(u8_str);
    }
    void operator=(std::nullptr_t) {
        m_value->_d(JsonValue::JSON_VALUE_NULL);
    }
    template <typename T>
    void operator=(std::vector<T> a_list) {
        reset();
        m_value->_d(JsonValue::JSON_VALUE_ARRAY);
        for (auto& obj : a_list) {
            push_back(obj);
        }
    }
    template <typename T>
    void operator=(const std::map<std::string, T>& a_map) {
        reset();
        m_value->_d(JsonValue::JSON_VALUE_OBJECT);
        auto& members = m_value->object_value();
        for (auto& obj : a_map) {
            JsonRef member(&members[obj.first]);
            member = obj.second;
        }
    }

    std::string str() const {
        std::stringstream out;
        out << *m_value;
        return out.str();
    }

    JsonValue& json_value() {
        return *m_value;
    }
    const JsonValue& json_value() const {
        return *m_value;
    }

    template <typename T>
    T& get();
    template <typename T>
    T get() const;

  private:
    void emplace() {}
    void emplace_back() {}

    JsonValue* m_value;
    std::string m_name;
    std::unique_ptr<JsonRef> m_parent;
};

template <>
inline bool& JsonRef::get<bool>() {
    return m_value->bool_value();
}

template <>
inline int64_t& JsonRef::get<int64_t>() {
    return m_value->integer_value();
}
template <>
inline double& JsonRef::get<double>() {
    return m_value->float_value();
}
template <>
inline std::string& JsonRef::get<std::string>() {
    return m_value->string_value();
}

template <>
inline bool JsonRef::get<bool>() const {
    return m_value->bool_value();
}
template <>
inline int64_t JsonRef::get<int64_t>() const {
    return m_value->integer_value();
}
template <>
inline double JsonRef::get<double>() const {
    return m_value->float_value();
}
template <>
inline std::string JsonRef::get<std::string>() const {
    return m_value->string_value();
}

class Json : public JsonRef {
  public:
    Json() : JsonRef(&m_value) {}
    Json(const Json& a_other) : JsonRef(&m_value), m_value(a_other.m_value) {}
    Json(Json&& a_other) noexcept : JsonRef(&m_value), m_value(std::move(a_other.m_value)) {}
    ~Json() = default;
    explicit Json(JsonValue a_value) : JsonRef(&m_value), m_value(std::move(a_value)) {}
    template <typename T>
    explicit Json(const T& a_value) : JsonRef(&m_value) {
        operator=(a_value);
    }

    Json& operator=(const Json& a_other) {
        if (this != &a_other) {
            m_value = a_other.m_value;
        }
        return *this;
    }
    Json& operator=(Json&& a_other) noexcept {
        if (this != &a_other) {
            std::swap(m_value, a_other.m_value);
        }
        return *this;
    }

    template <typename T>
    void operator=(const T& a_value) {
        JsonRef::operator=(a_value);
    }
    void operator=(const char* a_value) {
        JsonRef::operator=(a_value);
    }

    static Json parse(std::istream& a_stream) {
        JsonValue value;
        a_stream >> value;
        return Json(value);
    }

    static Json parse(std::string_view a_string) {
        JsonValue value;
        detail::read_json_value(JsonNode::from_data(a_string), value);
        return Json(value);
    }

    static Json array() {
        return Json(JsonValue::JSON_VALUE_ARRAY);
    }
    static Json object() {
        return Json(JsonValue::JSON_VALUE_OBJECT);
    }
    static Json null() {
        return Json(JsonValue::JSON_VALUE_NULL);
    }
    static Json number(double value = 0.0) {
        return Json(value);
    }
    static Json integer(int64_t value = 0) {
        return Json(value);
    }
    static Json string(std::string_view value = "") {
        return Json(value);
    }
    static Json boolean(bool value = false) {
        return Json(value);
    }

    template <typename... Ts>
    static Json array(Ts... args) {
        Json json = array();
        json.emplace_back(args...);
        return json;
    }

    template <typename... Ts>
    static Json object(Ts... args) {
        Json json = object();
        json.emplace(args...);
        return json;
    }

  private:
    explicit Json(JsonValue::JsonValueType a_type) : JsonRef(&m_value) {
        m_value._d(a_type);
    }

    JsonValue m_value;
};

inline bool operator==(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() == rhs.json_value();
}
inline bool operator!=(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() != rhs.json_value();
}
inline bool operator<(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() < rhs.json_value();
}
inline bool operator<=(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() <= rhs.json_value();
}
inline bool operator>(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() > rhs.json_value();
}
inline bool operator>=(const JsonRef& lhs, const JsonRef& rhs) {
    return lhs.json_value() >= rhs.json_value();
}

inline std::ostream& operator<<(std::ostream& out, const JsonRef& value) {
    out << value.json_value();
    return out;
}

inline std::istream& operator>>(std::istream& in, JsonRef& value) {
    in >> value.json_value();
    return in;
}

}  // namespace ic_cts
