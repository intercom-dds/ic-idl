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

#ifndef SerializationSupport_h___included
#  define SerializationSupport_h___included

#  include <array>
#  include <cstring>
#  include <map>
#  include <memory>
#  include <set>
#  include <sstream>
#  include <stdexcept>
#  include <string>
#  include <vector>

#  include "InterCOM/dds_xtypes_constants.h"
#  include "InterCOM/intercom_dcps.h"
#  include "InterCOM/member_info.h"
#  include "InterCOM/platform_config.h"

#  ifdef INTERCOM_COMPILER_MICROSOFT
#    pragma warning(push)
#    pragma warning(disable : 4127 \
    )  // Conditional expression is constant in template instantiations
#    pragma warning(disable : 4512 \
    )  // Assignment operator cannot be generated for some serialization classes
#  endif

namespace intercom {
template <typename T>
bool enumToString(std::string& res, T value, const TypeInfo* type);
template <typename T>
bool stringToEnum(T& value, const char* string, const TypeInfo* type);

namespace dcps {
namespace cts {
template <typename T>
struct unsigned_type_of;

const ULong MAX_NESTED_DEPTH = 64;

class INTERCOM_PUBLIC GenericWriter {
  public:
    virtual ~GenericWriter() = default;
    virtual SerializerFlags flags() const = 0;
    virtual void begin_type(const TypeInfo& a_info) = 0;
    virtual void end_type() = 0;
    virtual ULong type_level() = 0;
    virtual bool is_relevant(const MemberInfo& a_member) = 0;
    virtual bool begin_member(const MemberInfo& a_member) = 0;
    virtual bool begin_optional_member(const MemberInfo& a_member, bool a_present) = 0;
    virtual void end_member() = 0;
    virtual void write_length(ULong a_length) = 0;

    virtual void write(const corba::EightBitString_var& a_value) = 0;
    virtual void write(const corba::Utf8String_var& a_value) = 0;
    virtual void write(const corba::Utf16String_var& a_value) = 0;
    virtual void write(const corba::Utf32String_var& a_value) = 0;
    virtual void write(const std::string& a_value) = 0;
    virtual void write(const std::wstring& a_value) = 0;
    virtual void write(const std::u16string& a_value) = 0;
    virtual void write(const Boolean* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Int8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const UInt8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Int16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const UInt16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Int32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const UInt32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Int64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const UInt64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Float32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Float64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Float128* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Char8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Char16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void write(const Char32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
};

class INTERCOM_PUBLIC GenericReader {
  public:
    virtual ~GenericReader() = default;
    virtual SerializerFlags flags() const = 0;
    virtual void begin_type(const TypeInfo& a_info) = 0;
    virtual void end_type() = 0;
    virtual ULong type_level() = 0;
    virtual bool find_member(const MemberInfo& a_member) = 0;
    virtual void end_member() = 0;
    virtual bool can_skip_type() = 0;

    virtual ULong read_length() = 0;

    virtual void read(corba::EightBitString_var& a_value) = 0;
    virtual void read(corba::Utf8String_var& a_value) = 0;
    virtual void read(corba::Utf16String_var& a_value) = 0;
    virtual void read(corba::Utf32String_var& a_value) = 0;
    virtual void read(std::string& a_value) = 0;
    virtual void read(std::wstring& a_value) = 0;
    virtual void read(std::u16string& a_value) = 0;
    virtual void read(Boolean* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Int8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(UInt8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Int16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(UInt16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Int32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(UInt32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Int64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(UInt64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Float32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Float64* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Float128* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Char8* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Char16* a_values, ULong a_count, const TypeInfo& a_info) = 0;
    virtual void read(Char32* a_values, ULong a_count, const TypeInfo& a_info) = 0;
};

class EmptyWriter : public GenericWriter {
  public:
    SerializerFlags flags() const override {
        return 0;
    }

    void begin_type(const TypeInfo&) override {}

    void end_type() override {}

    ULong type_level() override {
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

    void write_length(ULong) override {}

    void write(const corba::EightBitString_var&) override {}

    void write(const corba::Utf8String_var&) override {}

    void write(const corba::Utf16String_var&) override {}

    void write(const corba::Utf32String_var&) override {}

    void write(const std::string&) override {}

    void write(const std::wstring&) override {}

    void write(const std::u16string&) override {}

    void write(const Boolean*, ULong, const TypeInfo&) override {}

    void write(const Int8*, ULong, const TypeInfo&) override {}

    void write(const UInt8*, ULong, const TypeInfo&) override {}

    void write(const Int16*, ULong, const TypeInfo&) override {}

    void write(const UInt16*, ULong, const TypeInfo&) override {}

    void write(const Int32*, ULong, const TypeInfo&) override {}

    void write(const UInt32*, ULong, const TypeInfo&) override {}

    void write(const Int64*, ULong, const TypeInfo&) override {}

    void write(const UInt64*, ULong, const TypeInfo&) override {}

    void write(const Float32*, ULong, const TypeInfo&) override {}

    void write(const Float64*, ULong, const TypeInfo&) override {}

    void write(const Float128*, ULong, const TypeInfo&) override {}

    void write(const Char8*, ULong, const TypeInfo&) override {}

    void write(const Char16*, ULong, const TypeInfo&) override {}

    void write(const Char32*, ULong, const TypeInfo&) override {}
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

    ULong type_level() override {
        return 0;
    }

    bool find_member(const MemberInfo&) override {
        return m_has_member;
    }

    void end_member() override {}

    bool can_skip_type() override {
        return true;
    }

    ULong read_length() override {
        return 0;
    }

    void read(corba::EightBitString_var& a_value) override {
        a_value.clear();
    }

    void read(corba::Utf8String_var& a_value) override {
        a_value.clear();
    }

    void read(corba::Utf16String_var& a_value) override {
        a_value.clear();
    }

    void read(corba::Utf32String_var& a_value) override {
        a_value.clear();
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

    void read(Boolean* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Boolean));
    }

    void read(Int8* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Int8));
    }

    void read(UInt8* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(UInt8));
    }

    void read(Int16* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Int16));
    }

    void read(UInt16* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(UInt16));
    }

    void read(Int32* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Int32));
    }

    void read(UInt32* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(UInt32));
    }

    void read(Int64* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Int64));
    }

    void read(UInt64* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(UInt64));
    }

    void read(Float32* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Float32));
    }

    void read(Float64* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Float64));
    }

    void read(Float128* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Float128));
    }

    void read(Char8* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Char8));
    }

    void read(Char16* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Char16));
    }

    void read(Char32* a_values, ULong a_count, const TypeInfo&) override {
        memset(a_values, 0, a_count * sizeof(Char32));
    }

  private:
    bool m_has_member;
};

template <typename DELEGATE>
class KeyOnlyWriter : public GenericWriter {
  public:
    KeyOnlyWriter(DELEGATE& a_delegate) : m_delegate(a_delegate) {}

    SerializerFlags flags() const override {
        return m_delegate.flags() | SERIALIZER_KEY_ONLY;
    }

    ULong type_level() override {
        return m_delegate.type_level();
    }

    bool is_relevant(const MemberInfo& a_member) override {
        return (a_member.flags & xtypes::IS_KEY) != 0;
    }

    void begin_type(const TypeInfo& a_info) override {
        m_delegate.begin_type(a_info);
    }

    void end_type() override {
        m_delegate.end_type();
    }

    bool begin_member(const MemberInfo& a_member) override {
        if (a_member.flags & xtypes::IS_KEY) {
            return m_delegate.begin_member(a_member);
        } else if (type_level() > 1 && a_member.flags & xtypes::IS_IMPLICIT_KEY) {
            return m_delegate.begin_member(a_member);
        }
        return false;
    }

    bool begin_optional_member(const MemberInfo& a_member, bool a_present) override {
        if (a_member.flags & xtypes::IS_KEY) {
            return m_delegate.begin_optional_member(a_member, a_present);
        }
        return false;
    }

    void end_member() override {
        m_delegate.end_member();
    }

    void write_length(ULong a_length) override {
        m_delegate.write_length(a_length);
    }

    void write(const corba::EightBitString_var& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const corba::Utf8String_var& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const corba::Utf16String_var& a_value) override {
        m_delegate.write(a_value);
    }
    void write(const corba::Utf32String_var& a_value) override {
        m_delegate.write(a_value);
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
    void write(const Boolean* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Int8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const UInt8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Int16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const UInt16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Int32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const UInt32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Int64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const UInt64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Float32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Float64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Float128* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Char8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Char16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }
    void write(const Char32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.write(a_values, a_count, a_info);
    }

  private:
    DELEGATE& m_delegate;
};

template <typename DELEGATE>
class KeyOnlyReader : public GenericReader {
  public:
    KeyOnlyReader(DELEGATE& a_delegate) : m_delegate(a_delegate) {}

    SerializerFlags flags() const override {
        return m_delegate.flags() | SERIALIZER_KEY_ONLY;
    }

    ULong type_level() override {
        return m_delegate.type_level();
    }

    void begin_type(const TypeInfo& a_info) override {
        m_delegate.begin_type(a_info);
    }

    void end_type() override {
        m_delegate.end_type();
    }

    bool find_member(const MemberInfo& a_member) override {
        if (a_member.flags & xtypes::IS_KEY) {
            return m_delegate.find_member(a_member);
        } else if (type_level() > 1 && a_member.flags & xtypes::IS_IMPLICIT_KEY) {
            return m_delegate.find_member(a_member);
        } else {
            return false;
        }
    }

    void end_member() override {
        m_delegate.end_member();
    }

    bool can_skip_type() override {
        return m_delegate.can_skip_type();
    }

    ULong read_length() override {
        return m_delegate.read_length();
    }

    void read(corba::EightBitString_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf8String_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf16String_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf32String_var& a_value) override {
        m_delegate.read(a_value);
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
    void read(Boolean* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float128* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char32* a_values, ULong a_count, const TypeInfo& a_info) override {
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

    ULong type_level() override {
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
        } else if (m_members.find(a_member.name) != m_members.end()) {
            m_seen.insert(a_member.name);
            return m_delegate.find_member(a_member);
        } else if (type_level() != 1 || m_seen.size() < m_members.size()) {
            if (m_delegate.find_member(a_member)) {
                EmptyWriter emptyWriter;
                transform(emptyWriter, m_delegate, *a_member.type);
            }
            return false;
        } else {
            return false;
        }
    }

    void end_member() override {
        m_delegate.end_member();
    }

    bool can_skip_type() override {
        return m_delegate.can_skip_type();
    }

    ULong read_length() override {
        return m_delegate.read_length();
    }

    void read(corba::EightBitString_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf8String_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf16String_var& a_value) override {
        m_delegate.read(a_value);
    }
    void read(corba::Utf32String_var& a_value) override {
        m_delegate.read(a_value);
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
    void read(Boolean* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Int64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(UInt64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float32* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float64* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Float128* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char8* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char16* a_values, ULong a_count, const TypeInfo& a_info) override {
        m_delegate.read(a_values, a_count, a_info);
    }
    void read(Char32* a_values, ULong a_count, const TypeInfo& a_info) override {
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
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : Null_type_info) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~StructValue() {
            m_marshal.writer().end_type();
        }

        bool skip_member(const MemberInfo& a_member) {
            if (m_marshal.writer().flags() & SERIALIZER_KEY_ONLY) {
                if (a_member.flags & xtypes::IS_KEY || (m_marshal.writer().type_level() > 1 &&
                                                        a_member.flags & xtypes::IS_IMPLICIT_KEY)) {
                    return false;
                } else {
                    return true;
                }
            } else {
                return false;
            }
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
        void io(const MemberInfo& a_member_info, const optional<T>& value) {
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
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : Null_seq_type_info) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~VectorValue() {
            m_marshal.writer().end_type();
        }

        template <typename VECTOR>
        void io(const VECTOR& value) {
            auto len = static_cast<ULong>(value.size());
            m_marshal.writer().write_length(len);
            if (len > 0) {
                io(&value[0], len);
            }
        }

        void io(const T* value, ULong value_count) {
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
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : Null_map_type_info) {
            m_marshal.writer().begin_type(m_type_info);
        }

        ~MapValue() {
            m_marshal.writer().end_type();
        }

        template <typename T>
        void io(const T& value) {
            auto len = static_cast<ULong>(value.size());
            m_marshal.writer().write_length(len);
            Serializer<MARSHAL, K> keySerialize;
            Serializer<MARSHAL, V> valueSerialize;
            for (auto it = value.begin(); it != value.end(); ++it) {
                keySerialize(m_marshal, *const_cast<K*>(&it->first), m_type_info.key_type);
                valueSerialize(m_marshal, *const_cast<V*>(&it->second), m_type_info.element_type);
            }
        }

        template <typename VECTOR>
        void io(const VECTOR& keys, const VECTOR& values) {
            auto len = static_cast<ULong>(values.size());
            m_marshal.writer().write_length(len);
            Serializer<MARSHAL, K> keySerialize;
            Serializer<MARSHAL, V> valueSerialize;
            for (ULong i = 0; i < len; ++i) {
                keySerialize(m_marshal, *const_cast<K*>(&keys[i]), m_type_info.key_type);
                valueSerialize(m_marshal, *const_cast<V*>(&values[i]), m_type_info.element_type);
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
            : m_marshal(a_marshal), m_type_info(a_info ? *a_info : Null_type_info) {
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

    TGenericMarshalBase(WRITER& a_writer) : m_writer(&a_writer) {}

    TGenericMarshalBase() : m_writer(nullptr) {}

    template <typename T>
    void io(const T& value, const TypeInfo* a_type_info = nullptr);

    template <typename T>
    void primitive_io(T value, const TypeInfo* a_info = nullptr)  //!< \private
    {
        writer().write(&value, 1, a_info ? *a_info : Null_type_info);
    }

    template <typename T>
    void
    primitive_io(const T* value, ULong value_count, const TypeInfo* a_info = nullptr)  //!< \private
    {
        writer().write(value, value_count, a_info ? *a_info : Null_type_info);
    }

    WRITER& writer() {
        return *m_writer;
    }

    void writer(WRITER* a_writer) {
        m_writer = a_writer;
    }

  private:
    WRITER* m_writer;
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
                if (a_member.flags & xtypes::IS_KEY || (m_unmarshal.reader().type_level() > 1 &&
                                                        a_member.flags & xtypes::IS_IMPLICIT_KEY)) {
                    return false;
                } else {
                    return true;
                }
            } else {
                return false;
            }
        }

        template <typename T>
        void io(const MemberInfo& a_member_info, T& value);

        template <typename T>
        void io(const MemberInfo& a_member_info, optional<T>& value);

      protected:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <typename T>
    class VectorValue  //!< \private
    {
      public:
        VectorValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info)
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : Null_seq_type_info) {
            m_unmarshal.reader().begin_type(m_type_info);
        }

        ~VectorValue() {
            m_unmarshal.reader().end_type();
        }

        template <typename VECTOR>
        void io(VECTOR& value) {
            ULong len = m_unmarshal.reader().read_length();
            value.resize(len);
            if (len > 0) {
                io(&value[0], len);
            }
        }

        void io(T* value, ULong value_count) {
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
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : Null_map_type_info) {
            m_unmarshal.reader().begin_type(m_type_info);
        }

        ~MapValue() {
            m_unmarshal.reader().end_type();
        }

        template <typename T>
        void io(T& value) {
            ULong len = m_unmarshal.reader().read_length();
            value.clear();
            Serializer<UNMARSHAL, K> keySerialize;
            Serializer<UNMARSHAL, V> valueSerialize;
            K key = K();
            for (ULong i = 0; i < len; ++i) {
                keySerialize(m_unmarshal, key, m_type_info.key_type);
                valueSerialize(m_unmarshal, value[key], m_type_info.element_type);
            }
        }

        template <typename VECTOR>
        void io(VECTOR& keys, VECTOR& values) {
            ULong len = m_unmarshal.reader().read_length();
            keys.resize(len);
            values.resize(len);
            Serializer<UNMARSHAL, K> keySerialize;
            Serializer<UNMARSHAL, V> valueSerialize;
            for (ULong i = 0; i < len; ++i) {
                keySerialize(m_unmarshal, keys[i], m_type_info.key_type);
                valueSerialize(m_unmarshal, values[i], m_type_info.element_type);
            }
        }

      private:
        UNMARSHAL& m_unmarshal;
        const TypeInfo& m_type_info;
    };

    template <CharacterEncoding ENCODING>
    class StringValue  //!< \private
    {
        using value_type = typename corba::TString_var<ENCODING>::value_type;

      public:
        StringValue(UNMARSHAL& a_unmarshal, const TypeInfo* a_info)
            : m_unmarshal(a_unmarshal), m_type_info(a_info ? *a_info : Null_type_info) {
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

    TGenericUnmarshalBase(READER& a_reader) : m_reader(&a_reader) {}

    TGenericUnmarshalBase() : m_reader(nullptr) {}

    template <typename T>
    void io(T& value, const TypeInfo* a_type_info = nullptr);

    template <typename T>
    void primitive_io(T& value, const TypeInfo* a_info = nullptr) {
        reader().read(&value, 1, a_info ? *a_info : Null_type_info);
    }

    template <typename T>
    void primitive_io(T* value, ULong value_count, const TypeInfo* a_info = nullptr) {
        reader().read(value, value_count, a_info ? *a_info : Null_type_info);
    }

    READER& reader() {
        return *m_reader;
    }

    void reader(READER* a_reader) {
        m_reader = a_reader;
    }

  private:
    READER* m_reader;
};

template <typename READER>
class TGenericUnmarshal : public TGenericUnmarshalBase<READER, TGenericUnmarshal<READER>> {
  public:
    TGenericUnmarshal(READER& a_reader)
        : TGenericUnmarshalBase<READER, TGenericUnmarshal<READER>>(a_reader) {}
};

template <typename WRITER>
class TGenericMarshal : public TGenericMarshalBase<WRITER, TGenericMarshal<WRITER>> {
  public:
    TGenericMarshal(WRITER& a_writer)
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
}  // namespace cts
}  // namespace dcps
}  // namespace intercom

#  ifdef INTERCOM_COMPILER_MICROSOFT
#    pragma warning(pop)
#  endif

#endif

#include "detail/serialization.ic"  // IWYU pragma: export
