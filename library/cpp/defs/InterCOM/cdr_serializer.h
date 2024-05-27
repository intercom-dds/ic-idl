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

#include "InterCOM/buffer.h"
#include "InterCOM/dds_xtypes_constants.h"
#include "InterCOM/dyn_link.h"
#include "InterCOM/serialization.h"
#include "InterCOM/span.h"

namespace intercom {

const SerializerFlags ENC_PLATFORM_ENDIAN =
    (INTERCOM_PLATFORM_ENDIAN == BigEndian) ? CDR_BIG_ENDIAN : CDR_LITTLE_ENDIAN;

struct Encapsulation {
    unsigned short encapsulation{0};  // EncapsulationSchemeIdentifier
    unsigned short options{0};
};

struct ParameterHeader {
    ParameterId_t parameterId{0};
    unsigned short length{0};
};

namespace dcps {
namespace cts {

SerializerFlags encodingForEncapsulation(EncapsulationSchemeIdentifier a_scheme);

void readEncapsulation(
    intercom::span<const Octet> a_data,
    SerializerFlags& a_encoding,
    ULong& a_data_length
);

void writeEncapsulation(Buffer& a_buffer, SerializerFlags a_encoding, const TypeInfo& a_type_info);

class INTERCOM_PUBLIC CdrWriter : public GenericWriter {
  public:
    CdrWriter(Buffer& a_out, SerializerFlags a_flags);

    SerializerFlags flags() const override;

    ULong type_level() override;

    bool is_relevant(const MemberInfo& a_member) override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

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

    void write(const Boolean* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Int8* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const UInt8* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Int16* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const UInt16* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Int32* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const UInt32* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Int64* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const UInt64* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Float32* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Float64* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Float128* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Char8* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Char16* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const Char32* a_values, ULong a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

  private:
    struct TypeStackElement {
        ULong type_base{0};
        ULong align_base{0};
        const TypeInfo* type_info{nullptr};
        const MemberInfo* member_info{nullptr};
    };

    template <typename T>
    void reserve(ULong a_count);

    inline ULong align_base() const;

    template <typename T>
    void write_value(Octet* buf, T value);

    template <typename T>
    void write_values(const T* a_values, ULong a_count);

    void end_mutable_member_v1();

    void begin_mutable_member_v2();

    void end_mutable_member_v2();

    Buffer& m_buffer;
    SerializerFlags m_flags;
    std::array<TypeStackElement, MAX_NESTED_DEPTH> m_type_stack;
    ULong m_level;
};

class INTERCOM_PUBLIC CdrReader : public GenericReader {
  public:
    CdrReader(const Buffer& a_out, SerializerFlags a_flags);

    CdrReader(const CdrReader& a_other, SerializerFlags a_flags);

    SerializerFlags flags() const override;

    ULong type_level() override;

    void begin_type(const TypeInfo& a_info) override;
    void end_type() override;
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

    void read(Boolean* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Int8* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(UInt8* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Int16* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(UInt16* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Int32* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(UInt32* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Int64* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(UInt64* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Float32* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Float64* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Float128* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Char8* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Char16* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(Char32* a_values, ULong a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

  private:
    struct TypeStackElement {
        bool length_delimited{false};
        ULong type_base{0};
        ULong type_length{0};
        ULong member_base{0};
        ULong member_length{0};
        ULong align_base{0};
        const TypeInfo* type_info{nullptr};
    };

    template <typename T>
    void reserve(ULong a_count);

    inline ULong available_bytes() const;

    inline ULong align_base() const;

    template <typename T>
    void read_value(const Octet* buf, T& value);

    template <typename T>
    void read_values(T* a_values, ULong a_count);

    bool begin_mutable_member_v1(const MemberInfo& a_member);

    bool begin_mutable_member_v2(const MemberInfo& a_member);

    bool begin_optional_member(const MemberInfo& a_member);

    void read_param_header_v1(xtypes::MemberId& id, ULong& flag, ULong& length);

    void read_param_header_v2(xtypes::MemberId& id, ULong& flag, ULong& length);

    const Buffer& m_buffer;
    SerializerFlags m_flags;
    std::array<TypeStackElement, MAX_NESTED_DEPTH> m_type_stack;
    ULong m_level;
};

using CdrUnmarshal = TGenericUnmarshal<CdrReader>;
using KeyOnlyCdrUnmarshal = TGenericUnmarshal<KeyOnlyReader<CdrReader>>;

using CdrMarshal = TGenericMarshal<CdrWriter>;
using KeyOnlyCdrMarshal = TGenericMarshal<KeyOnlyWriter<CdrWriter>>;

}  // namespace cts
}  // namespace dcps

template <typename T>
inline void marshal_cdr(dcps::Buffer& a_buffer, SerializerFlags a_encoding, const T& a_value) {
    dcps::cts::CdrWriter writer(a_buffer, a_encoding);
    dcps::cts::CdrMarshal(writer).io(a_value);
}

template <>
inline void marshal_cdr<dcps::DynamicDataPtr>(
    dcps::Buffer& a_buffer,
    SerializerFlags a_encoding,
    const dcps::DynamicDataPtr& a_value
) {
    dcps::cts::CdrWriter writer(a_buffer, a_encoding);
    dcps::cts::GenericMarshal(writer).io(a_value);
}

template <typename T>
inline void unmarshal_cdr(dcps::Buffer& a_buffer, SerializerFlags a_encoding, T& a_value) {
    dcps::cts::CdrReader reader(a_buffer, a_encoding);
    dcps::cts::CdrUnmarshal(reader).io(a_value);
}

template <>
inline void unmarshal_cdr<dcps::DynamicDataPtr>(
    dcps::Buffer& a_buffer,
    SerializerFlags a_encoding,
    dcps::DynamicDataPtr& a_value
) {
    dcps::cts::CdrReader reader(a_buffer, a_encoding);
    dcps::cts::GenericUnmarshal(reader).io(a_value);
}

template <typename T>
inline void
unmarshal_cdr(intercom::span<const Octet> a_data, SerializerFlags a_encoding, T& a_value) {
    dcps::Buffer buffer(a_data.data(), static_cast<ULong>(a_data.size()));
    unmarshal_cdr(buffer, a_encoding, a_value);
}

INTERCOM_PUBLIC void transform_cdr(
    dcps::Buffer& a_out,
    SerializerFlags a_out_encoding,
    dcps::Buffer& a_in,
    SerializerFlags a_in_encoding,
    const TypeInfo& a_type_info
);

template <typename T>
inline void transform_cdr(
    dcps::Buffer& a_out,
    SerializerFlags a_out_encoding,
    const dcps::Buffer& a_in,
    SerializerFlags a_in_encoding
) {
    transform_cdr(a_out, a_out_encoding, a_in, a_in_encoding, TypeTraits<T>::type_info);
}
}  // namespace intercom

#include "detail/CdrSerializer.ic"  // IWYU pragma: export
