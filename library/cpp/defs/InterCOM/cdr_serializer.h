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
#include "InterCOM/platform_config.h"
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
    intercom::span<const uint8_t> a_data,
    SerializerFlags& a_encoding,
    uint32_t& a_data_length
);

void writeEncapsulation(Buffer& a_buffer, SerializerFlags a_encoding, const TypeInfo& a_type_info);

class CdrWriter : public GenericWriter {
  public:
    CdrWriter(Buffer& a_out, SerializerFlags a_flags);

    SerializerFlags flags() const override;

    uint32_t type_level() override;

    bool is_relevant(const MemberInfo& a_member) override;

    void begin_type(const TypeInfo& a_info) override;

    void end_type() override;

    bool begin_member(const MemberInfo& a_member) override;

    bool begin_optional_member(const MemberInfo& a_member, bool a_present) override;

    void end_member() override;

    void write_length(uint32_t a_length) override;

    void write(const std::string& a_value) override;

    void write(const std::wstring& a_value) override;

    void write(const std::u16string& a_value) override;

    void write(const bool* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const int8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const uint8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const int16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const uint16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const int32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const uint32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const int64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const uint64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const float* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const double* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const long double* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const char* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

    void write(const char16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        write_values(a_values, a_count);
    }

  private:
    struct TypeStackElement {
        uint32_t type_base{0};
        uint32_t align_base{0};
        const TypeInfo* type_info{nullptr};
        const MemberInfo* member_info{nullptr};
    };

    template <typename T>
    void reserve(uint32_t a_count);

    inline uint32_t align_base() const;

    template <typename T>
    void write_value(uint8_t* buf, T value);

    template <typename T>
    void write_values(const T* a_values, uint32_t a_count);

    void end_mutable_member_v1();

    void begin_mutable_member_v2();

    void end_mutable_member_v2();

    Buffer& m_buffer;
    SerializerFlags m_flags;
    std::array<TypeStackElement, MAX_NESTED_DEPTH> m_type_stack;
    uint32_t m_level;
};

class CdrReader : public GenericReader {
  public:
    CdrReader(const Buffer& a_out, SerializerFlags a_flags);

    CdrReader(const CdrReader& a_other, SerializerFlags a_flags);

    SerializerFlags flags() const override;

    uint32_t type_level() override;

    void begin_type(const TypeInfo& a_info) override;
    void end_type() override;
    bool find_member(const MemberInfo& a_member) override;
    void end_member() override;
    bool can_skip_type() override;

    uint32_t read_length() override;

    void read(std::string& a_value) override;
    void read(std::wstring& a_value) override;
    void read(std::u16string& a_value) override;

    void read(bool* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(int8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(uint8_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(int16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(uint16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(int32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(uint32_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(int64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(uint64_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(float* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(double* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(long double* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(char* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

    void read(char16_t* a_values, uint32_t a_count, const TypeInfo&) override {
        read_values(a_values, a_count);
    }

  private:
    struct TypeStackElement {
        bool length_delimited{false};
        uint32_t type_base{0};
        uint32_t type_length{0};
        uint32_t member_base{0};
        uint32_t member_length{0};
        uint32_t align_base{0};
        const TypeInfo* type_info{nullptr};
    };

    template <typename T>
    void reserve(uint32_t a_count);

    inline uint32_t available_bytes() const;

    inline uint32_t align_base() const;

    template <typename T>
    void read_value(const uint8_t* buf, T& value);

    template <typename T>
    void read_values(T* a_values, uint32_t a_count);

    bool begin_mutable_member_v1(const MemberInfo& a_member);

    bool begin_mutable_member_v2(const MemberInfo& a_member);

    bool begin_optional_member(const MemberInfo& a_member);

    void read_param_header_v1(xtypes::MemberId& id, uint32_t& flag, uint32_t& length);

    void read_param_header_v2(xtypes::MemberId& id, uint32_t& flag, uint32_t& length);

    const Buffer& m_buffer;
    SerializerFlags m_flags;
    std::array<TypeStackElement, MAX_NESTED_DEPTH> m_type_stack;
    uint32_t m_level;
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

template <typename T>
inline void unmarshal_cdr(dcps::Buffer& a_buffer, SerializerFlags a_encoding, T& a_value) {
    dcps::cts::CdrReader reader(a_buffer, a_encoding);
    dcps::cts::CdrUnmarshal(reader).io(a_value);
}

template <typename T>
inline void
unmarshal_cdr(intercom::span<const uint8_t> a_data, SerializerFlags a_encoding, T& a_value) {
    dcps::Buffer buffer(a_data.data(), static_cast<uint32_t>(a_data.size()));
    unmarshal_cdr(buffer, a_encoding, a_value);
}

void transform_cdr(
    dcps::Buffer& a_out,
    SerializerFlags a_out_encoding,
    dcps::Buffer& a_in,
    SerializerFlags a_in_encoding,
    const TypeInfo& a_type_info
);

}  // namespace intercom

#include "detail/cdr_serializer.ic"  // IWYU pragma: export
