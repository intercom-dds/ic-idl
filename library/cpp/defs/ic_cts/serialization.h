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

#include <cstring>
#include <optional>
#include <set>
#include <string>

#include "ic_cts/character_encoding.h"
#include "ic_cts/dds_xtypes_constants.h"
#include "ic_cts/member_info.h"

#ifdef INTERCOM_COMPILER_MICROSOFT
#  pragma warning(push)
#  pragma warning(disable : 4127)  // Conditional expression is constant in template instantiations
#  pragma warning(disable : 4512)  // Assignment operator cannot be generated for classes
#endif

namespace ic_cts {

template <typename T>
bool enum_to_string(std::string& res, T value, const TypeInfo* type);

template <typename T>
bool string_to_enum(T& value, const char* string, const TypeInfo* type);

template <typename T>
struct unsigned_type_of;

const uint32_t MAX_NESTED_DEPTH = 64;

class GenericWriter {
  public:
    virtual ~GenericWriter() = default;
    virtual SerializerFlags flags() const = 0;
    virtual void begin_type(const TypeInfo& a_info) = 0;
    virtual void end_type() = 0;
    virtual uint32_t type_level() = 0;
    virtual bool is_relevant(const MemberInfo& a_member) = 0;
    virtual bool begin_member(const MemberInfo& a_member) = 0;
    virtual bool begin_optional_member(const MemberInfo& a_member, bool a_present) = 0;
    virtual void end_member() = 0;
    virtual void write_length(uint32_t a_length) = 0;

    virtual void write(const std::string& a_value) = 0;
    virtual void write(const std::wstring& a_value) = 0;
    virtual void write(const std::u16string& a_value) = 0;
    virtual void write(const bool* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const float* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const double* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const long double* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const char* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void write(const char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
};

class GenericReader {
  public:
    virtual ~GenericReader() = default;
    virtual SerializerFlags flags() const = 0;
    virtual void begin_type(const TypeInfo& a_info) = 0;
    virtual void end_type() = 0;
    virtual uint32_t type_level() = 0;
    virtual bool find_member(const MemberInfo& a_member) = 0;
    virtual void end_member() = 0;
    virtual bool can_skip_type() = 0;

    virtual uint32_t read_length() = 0;

    virtual void read(std::string& a_value) = 0;
    virtual void read(std::wstring& a_value) = 0;
    virtual void read(std::u16string& a_value) = 0;
    virtual void read(bool* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(float* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(double* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(long double* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(char* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
    virtual void read(char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) = 0;
};

class EmptyWriter : public GenericWriter {
  public:
    SerializerFlags flags() const override {
        return 0;
    }

    void begin_type(const TypeInfo&) override {}

    void end_type() override {}

    uint32_t type_level() override {
        return 0;
    }

    bool is_relevant(const MemberInfo&) override {
        return false;
    }

    bool begin_member(const MemberInfo&) override {
        return false;
    }

    bool begin_optional_member(const MemberInfo&, bool) override {
        return false;
    }

    void end_member() override {}

    void write_length(uint32_t) override {}

    void write(const std::string&) override {}

    void write(const std::wstring&) override {}

    void write(const std::u16string&) override {}

    void write(const bool*, uint32_t, const TypeInfo&) override {}

    void write(const int8_t*, uint32_t, const TypeInfo&) override {}

    void write(const uint8_t*, uint32_t, const TypeInfo&) override {}

    void write(const int16_t*, uint32_t, const TypeInfo&) override {}

    void write(const uint16_t*, uint32_t, const TypeInfo&) override {}

    void write(const int32_t*, uint32_t, const TypeInfo&) override {}

    void write(const uint32_t*, uint32_t, const TypeInfo&) override {}

    void write(const int64_t*, uint32_t, const TypeInfo&) override {}

    void write(const uint64_t*, uint32_t, const TypeInfo&) override {}

    void write(const float*, uint32_t, const TypeInfo&) override {}

    void write(const double*, uint32_t, const TypeInfo&) override {}

    void write(const long double*, uint32_t, const TypeInfo&) override {}

    void write(const char*, uint32_t, const TypeInfo&) override {}

    void write(const char16_t*, uint32_t, const TypeInfo&) override {}
};

class EmptyReader : public GenericReader {
  public:
    EmptyReader() : m_has_member(true) {}
    explicit EmptyReader(bool a_has_member) : m_has_member(a_has_member) {}

    SerializerFlags flags() const override {
        return 0;
    }

    void begin_type(const TypeInfo&) override {}

    void end_type() override {}

    uint32_t type_level() override {
        return 0;
    }

    bool find_member(const MemberInfo&) override {
        return m_has_member;
    }

    void end_member() override {}

    bool can_skip_type() override {
        return true;
    }

    uint32_t read_length() override {
        return 0;
    }

    void read(std::string& a_value) override {
        a_value.clear();
    }

    void read(std::wstring& a_value) override {
        a_value.clear();
    }

    void read(std::u16string& a_value) override {
        a_value.clear();
    }

    void read(bool* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(bool));
    }

    void read(int8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(int8_t));
    }

    void read(uint8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(uint8_t));
    }

    void read(int16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(int16_t));
    }

    void read(uint16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(uint16_t));
    }

    void read(int32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(int32_t));
    }

    void read(uint32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(uint32_t));
    }

    void read(int64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(int64_t));
    }

    void read(uint64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(uint64_t));
    }

    void read(float* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(float));
    }

    void read(double* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(double));
    }

    void read(long double* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(long double));
    }

    void read(char* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(char));
    }

    void read(char16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(char16_t));
    }

  private:
    bool m_has_member;
};

template <typename DELEGATE>
class KeyOnlyWriter : public GenericWriter {
  public:
    explicit KeyOnlyWriter(DELEGATE& a_delegate) : m_delegate(a_delegate) {}

    SerializerFlags flags() const override {
        return m_delegate.flags() | SERIALIZER_KEY_ONLY;
    }

    uint32_t type_level() override {
        return m_delegate.type_level();
    }

    bool is_relevant(const MemberInfo& a_member) override {
        return (a_member.flags & dcps::xtypes::IS_KEY) != 0;
    }

    void begin_type(const TypeInfo& a_info) override {
        m_delegate.begin_type(a_info);
    }

    void end_type() override {
        m_delegate.end_type();
    }

    bool begin_member(const MemberInfo& a_member) override {
        if (a_member.flags & dcps::xtypes::IS_KEY) {
            return m_delegate.begin_member(a_member);
        }
        if (type_level() > 1 && a_member.flags & dcps::xtypes::IS_IMPLICIT_KEY) {
            return m_delegate.begin_member(a_member);
        }
        return false;
    }

    bool begin_optional_member(const MemberInfo& a_member, bool a_present) override {
        if (a_member.flags & dcps::xtypes::IS_KEY) {
            return m_delegate.begin_optional_member(a_member, a_present);
        }
        return false;
    }

    void end_member() override {
        m_delegate.end_member();
    }

    void write_length(uint32_t a_length) override {
        m_delegate.write_length(a_length);
    }

    void write(const std::string& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const std::wstring& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const std::u16string& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const bool* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const float* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const long double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const char* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }

  private:
    DELEGATE& m_delegate;
};

template <typename DELEGATE>
class KeyOnlyReader : public GenericReader {
  public:
    explicit KeyOnlyReader(DELEGATE& a_delegate) : m_delegate(a_delegate) {}

    SerializerFlags flags() const override {
        return m_delegate.flags() | SERIALIZER_KEY_ONLY;
    }

    uint32_t type_level() override {
        return m_delegate.type_level();
    }

    void begin_type(const TypeInfo& a_info) override {
        m_delegate.begin_type(a_info);
    }

    void end_type() override {
        m_delegate.end_type();
    }

    bool find_member(const MemberInfo& a_member) override {
        if (a_member.flags & dcps::xtypes::IS_KEY) {
            return m_delegate.find_member(a_member);
        }
        if (type_level() > 1 && a_member.flags & dcps::xtypes::IS_IMPLICIT_KEY) {
            return m_delegate.find_member(a_member);
        }
        return false;
    }

    void end_member() override {
        m_delegate.end_member();
    }

    bool can_skip_type() override {
        return m_delegate.can_skip_type();
    }

    uint32_t read_length() override {
        return m_delegate.read_length();
    }

    void read(std::string& a_value) override {
        m_delegate.read(a_value);
    }
    void read(std::wstring& a_value) override {
        m_delegate.read(a_value);
    }
    void read(std::u16string& a_value) override {
        m_delegate.read(a_value);
    }
    void read(bool* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(float* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(long double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(char* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }

  private:
    DELEGATE& m_delegate;
};

template <typename DELEGATE>
class FilterMemberReader : public GenericReader {
  public:
    FilterMemberReader(DELEGATE& a_delegate, const std::set<std::string>& a_members)
        : m_delegate(a_delegate), m_members(a_members) {}

    SerializerFlags flags() const override {
        return m_delegate.flags();
    }

    uint32_t type_level() override {
        return m_delegate.type_level();
    }

    void begin_type(const TypeInfo& a_info) override {
        m_delegate.begin_type(a_info);
        if (type_level() == 1) {
            m_seen.clear();
        }
    }

    void end_type() override {
        m_delegate.end_type();
    }

    bool find_member(const MemberInfo& a_member) override {
        if (type_level() != 1) {
            return m_delegate.find_member(a_member);
        }
        if (m_members.find(a_member.name) != m_members.end()) {
            m_seen.insert(a_member.name);
            return m_delegate.find_member(a_member);
        }
        if (type_level() != 1 || m_seen.size() < m_members.size()) {
            if (m_delegate.find_member(a_member)) {
                EmptyWriter empty_writer;
                transform(empty_writer, m_delegate, *a_member.type);
            }
            return false;
        }
        return false;
    }

    void end_member() override {
        m_delegate.end_member();
    }

    bool can_skip_type() override {
        return m_delegate.can_skip_type();
    }

    uint32_t read_length() override {
        return m_delegate.read_length();
    }

    void read(std::string& a_value) override {
        m_delegate.read(a_value);
    }
    void read(std::wstring& a_value) override {
        m_delegate.read(a_value);
    }
    void read(std::u16string& a_value) override {
        m_delegate.read(a_value);
    }
    void read(bool* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint8_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint32_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(int64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(uint64_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(float* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(long double* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(char* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(char16_t* a_values, uint32_t a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }

  private:
    DELEGATE& m_delegate;
    const std::set<std::string>& m_members;
    std::set<std::string> m_seen;
};

template <typename WRITER, typename MARSHAL>
class TGenericMarshalBase {
  public:
    enum { IS_READER = 0, IS_WRITER = 1 };

    class StructValue  //!< \private
    {
      public:
        StructValue(MARSHAL& a_marshal, const TypeInfo* a_info)
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : NULL_TYPE_INFO) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~StructValue() {
            m_marshal.writer().end_type();
        }

        bool skip_member(const MemberInfo& a_member) {
            if (m_marshal.writer().flags() & SERIALIZER_KEY_ONLY) {
                return (
                    a_member.flags & dcps::xtypes::IS_KEY ||
                    (m_marshal.writer().type_level() > 1 &&
                     a_member.flags & dcps::xtypes::IS_IMPLICIT_KEY)
                );
            }
            return false;
        }

        template <typename T>
        void io(const MemberInfo& a_member_info, const T& value) {
            if (skip_member(a_member_info)) {
                return;
            }
            Serializer<MARSHAL, T> serialize;
            if (m_marshal.writer().begin_member(a_member_info)) {
                serialize(m_marshal, *const_cast<T*>(&value), a_member_info.type);
                m_marshal.writer().end_member();
            }
        }

        template <typename T>
        void io(const MemberInfo& a_member_info, const std::optional<T>& value) {
            if (skip_member(a_member_info)) {
                return;
            }
            if (m_marshal.writer().begin_optional_member(a_member_info, value.has_value())) {
                if (value.has_value()) {
                    Serializer<MARSHAL, T> serialize;
                    serialize(m_marshal, *const_cast<T*>(&value.value()), a_member_info.type);
                }
                m_marshal.writer().end_member();
            }
        }

      protected:
        MARSHAL& m_marshal;
        const TypeInfo& m_type_info;
    };

    template <typename T>
    class VectorValue  //!< \private
    {
      public:
        VectorValue(MARSHAL& a_marshal, const TypeInfo* a_info)
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : NULL_SEQ_TYPE_INFO) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~VectorValue() {
            m_marshal.writer().end_type();
        }

        template <typename VECTOR>
        void io(const VECTOR& value) {
            auto len = static_cast<uint32_t>(value.size());
            m_marshal.writer().write_length(len);
            if (len > 0) {
                io(&value[0], len);
            }
        }

        void io(const T* value, uint32_t value_count) {
            Serializer<MARSHAL, T*> serialize;
            serialize(m_marshal, const_cast<T*>(value), value_count, m_type_info.element_type);
        }

      private:
        MARSHAL& m_marshal;
        const TypeInfo& m_type_info;
    };

    template <typename K, typename V>
    class MapValue  //!< \private
    {
      public:
        MapValue(MARSHAL& a_marshal, const TypeInfo* a_info)
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : NULL_MAP_TYPE_INFO) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~MapValue() {
            m_marshal.writer().end_type();
        }

        template <typename T>
        void io(const T& value) {
            auto len = static_cast<uint32_t>(value.size());
            m_marshal.writer().write_length(len);
            Serializer<MARSHAL, K> key_serialize;
            Serializer<MARSHAL, V> value_serialize;
            for (auto it = value.begin(); it != value.end(); ++it) {
                key_serialize(m_marshal, *const_cast<K*>(&it->first), m_type_info.key_type);
                value_serialize(m_marshal, *const_cast<V*>(&it->second), m_type_info.element_type);
            }
        }

        template <typename VECTOR>
        void io(const VECTOR& keys, const VECTOR& values) {
            auto len = static_cast<uint32_t>(values.size());
            m_marshal.writer().write_length(len);
            Serializer<MARSHAL, K> key_serialize;
            Serializer<MARSHAL, V> value_serialize;
            for (uint32_t i = 0; i < len; ++i) {
                key_serialize(m_marshal, *const_cast<K*>(&keys[i]), m_type_info.key_type);
                value_serialize(m_marshal, *const_cast<V*>(&values[i]), m_type_info.element_type);
            }
        }

      private:
        MARSHAL& m_marshal;
        const TypeInfo& m_type_info;
    };

    template <CharacterEncoding ENCODING>
    class StringValue  //!< \private
    {
      public:
        StringValue(MARSHAL& a_marshal, const TypeInfo* a_info)
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : NULL_TYPE_INFO) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~StringValue() {
            m_marshal.writer().end_type();
        }

        template <typename T>
        void io(const T& value) {
            m_marshal.writer().write(value);
        }

      private:
        MARSHAL& m_marshal;
        const TypeInfo& m_type_info;
    };

    template <typename T>
    void io(const T& value, const TypeInfo* a_type_info = nullptr);

    template <typename T>
    void primitive_io(T value, const TypeInfo* a_info = nullptr)  //!< \private
    {
        writer().write(&value, 1, a_info ? *a_info : NULL_TYPE_INFO);
    }

    template <typename T>
    void primitive_io(
        const T* value,
        uint32_t value_count,
        const TypeInfo* a_info = nullptr
    )  //!< \private
    {
        writer().write(value, value_count, a_info ? *a_info : NULL_TYPE_INFO);
    }

    WRITER& writer() {
        return *m_writer;
    }

    void writer(WRITER* a_writer) {
        m_writer = a_writer;
    }

  private:
    explicit TGenericMarshalBase(WRITER& a_writer) : m_writer(&a_writer) {}

    TGenericMarshalBase() : m_writer(nullptr) {}

  private:
    WRITER* m_writer;
    friend MARSHAL;
};

template <typename READER, typename UNMARSHAL>
class TGenericUnmarshalBase {
  public:
    enum { IS_READER = 1, IS_WRITER = 0 };

    class StructValue  //!< \private
    {
      public:
        StructValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info);

        ~StructValue();

        bool skip_member(const MemberInfo& a_member) {
            if (m_unmarshal.reader().flags() & SERIALIZER_KEY_ONLY) {
                return (
                    a_member.flags & dcps::xtypes::IS_KEY ||
                    (m_unmarshal.reader().type_level() > 1 &&
                     a_member.flags & dcps::xtypes::IS_IMPLICIT_KEY)
                );
            }
            return false;
        }

        template <typename T>
        void io(const MemberInfo& a_member_info, T& value);

        template <typename T>
        void io(const MemberInfo& a_member_info, std::optional<T>& value);

      protected:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <typename T>
    class VectorValue  //!< \private
    {
      public:
        VectorValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info)
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : NULL_SEQ_TYPE_INFO) {
            m_unmarshal.reader().begin_type(m_type_info);
        }

        ~VectorValue() {
            m_unmarshal.reader().end_type();
        }

        template <typename VECTOR>
        void io(VECTOR& value) {
            uint32_t len = m_unmarshal.reader().read_length();
            value.resize(len);
            if (len > 0) {
                io(&value[0], len);
            }
        }

        void io(T* value, uint32_t value_count) {
            Serializer<UNMARSHAL, T*> serialize;
            serialize(m_unmarshal, value, value_count, m_type_info.element_type);
        }

      private:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <typename K, typename V>
    class MapValue  //!< \private
    {
      public:
        MapValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info)
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : NULL_MAP_TYPE_INFO) {
            m_unmarshal.reader().begin_type(m_type_info);
        }

        ~MapValue() {
            m_unmarshal.reader().end_type();
        }

        template <typename T>
        void io(T& value) {
            uint32_t len = m_unmarshal.reader().read_length();
            value.clear();
            Serializer<UNMARSHAL, K> key_serialize;
            Serializer<UNMARSHAL, V> value_serialize;
            K key = K();
            for (uint32_t i = 0; i < len; ++i) {
                key_serialize(m_unmarshal, key, m_type_info.key_type);
                value_serialize(m_unmarshal, value[key], m_type_info.element_type);
            }
        }

        template <typename VECTOR>
        void io(VECTOR& keys, VECTOR& values) {
            uint32_t len = m_unmarshal.reader().read_length();
            keys.resize(len);
            values.resize(len);
            Serializer<UNMARSHAL, K> key_serialize;
            Serializer<UNMARSHAL, V> value_serialize;
            for (uint32_t i = 0; i < len; ++i) {
                key_serialize(m_unmarshal, keys[i], m_type_info.key_type);
                value_serialize(m_unmarshal, values[i], m_type_info.element_type);
            }
        }

      private:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <CharacterEncoding ENCODING>
    class StringValue  //!< \private
    {
        using value_type = typename character_type_of<ENCODING>::type;

      public:
        StringValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info)
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : NULL_TYPE_INFO) {
            m_unmarshal.reader().begin_type(m_type_info);
        }

        ~StringValue() {
            m_unmarshal.reader().end_type();
        }

        template <typename T>
        void io(T& value) {
            m_unmarshal.reader().read(value);
        }

      private:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <typename T>
    void io(T& value, const TypeInfo* a_type_info = nullptr);

    template <typename T>
    void primitive_io(T& value, const TypeInfo* a_info = nullptr) {
        reader().read(&value, 1, a_info ? *a_info : NULL_TYPE_INFO);
    }

    template <typename T>
    void primitive_io(T* value, uint32_t value_count, const TypeInfo* a_info = nullptr) {
        reader().read(value, value_count, a_info ? *a_info : NULL_TYPE_INFO);
    }

    READER& reader() {
        return *m_reader;
    }

    void reader(READER* a_reader) {
        m_reader = a_reader;
    }

  private:
    explicit TGenericUnmarshalBase(READER& a_reader) : m_reader(&a_reader) {}

    TGenericUnmarshalBase() : m_reader(nullptr) {}

  private:
    READER* m_reader;
    friend UNMARSHAL;
};

template <typename READER>
class TGenericUnmarshal : public TGenericUnmarshalBase<READER, TGenericUnmarshal<READER>> {
  public:
    explicit TGenericUnmarshal(READER& a_reader)
        : TGenericUnmarshalBase<READER, TGenericUnmarshal<READER>>(a_reader) {}
};

template <typename WRITER>
class TGenericMarshal : public TGenericMarshalBase<WRITER, TGenericMarshal<WRITER>> {
  public:
    explicit TGenericMarshal(WRITER& a_writer)
        : TGenericMarshalBase<WRITER, TGenericMarshal<WRITER>>(a_writer) {}
};

class EmptyUnmarshal : public TGenericUnmarshal<EmptyReader> {
  public:
    EmptyUnmarshal() : TGenericUnmarshal<EmptyReader>(m_reader) {}

  private:
    EmptyReader m_reader;
};

using GenericUnmarshal = TGenericUnmarshal<GenericReader>;
using GenericMarshal = TGenericMarshal<GenericWriter>;

template <typename T, typename WRITER, typename READER>
void transform(WRITER& writer, READER& reader);

template <typename WRITER, typename READER>
void transform(WRITER& writer, READER& reader, const TypeInfo& type_info);

}  // namespace ic_cts

#ifdef INTERCOM_COMPILER_MICROSOFT
#  pragma warning(pop)
#endif

#include "detail/serialization.ic"  // IWYU pragma: export
