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
// obtain one at https://www.ncia.nato.int/downloads/NCoDe_Licence_V1.0.pdf

#pragma once

#ifdef _WIN32
#  pragma warning(push)
#  pragma warning(disable : 4065)
#  pragma warning(disable : 4127)
#endif

#include <InterCOM/bounded.h>
#include <InterCOM/span.h>

// NOLINTBEGIN

namespace intercom {
using ParameterId_t = uint16_t;
const ParameterId_t PID_SENTINEL{0x1U};
const ParameterId_t PID_EXTENDED{0x3f01U};
const ParameterId_t PID_LIST_END{0x3f02U};
const ParameterId_t PID_IGNORE{0x3f03U};
const ParameterId_t PID_FLAG_MUST_UNDERSTAND{0x4000U};
const ParameterId_t PID_FLAG_IMPL_EXTENSION{0x8000U};
const ParameterId_t PID_PID_MASK{0x3fffU};
enum SerializerFlagsBits : uint32_t {
    SERIALIZER_KEY_ONLY = 0x4ULL,
    SERIALIZER_SKIP_MISSING = 0x8ULL,
    SERIALIZER_PRETTY = 0x10ULL,
    SERIALIZER_STRICT = 0x20ULL,
    CDR_LITTLE_ENDIAN = 0x40ULL,
    CDR_BIG_ENDIAN = 0x80ULL,
    CDR_XCDR1 = 0x100ULL,
    CDR_XCDR2 = 0x200ULL,
    CDR_XCDR_PLAIN = 0x400ULL,
    CDR_XCDR_BUILTIN = 0x800ULL
};

using SerializerFlags = uint32_t;

enum EncapsulationSchemeIdentifier : int32_t {
    ENC_CDR_BE = 0,
    ENC_CDR_LE = 1,
    ENC_PL_CDR_BE = 2,
    ENC_PL_CDR_LE = 3,
    ENC_XML = 4,
    ENC_CDR2_BE = 6,
    ENC_CDR2_LE = 7,
    ENC_DELIMITED_CDR2_BE = 8,
    ENC_DELIMITED_CDR2_LE = 9,
    ENC_PL_CDR2_BE = 10,
    ENC_PL_CDR2_LE = 11,
    ENC_CDR2_BE_OLD = 16,
    ENC_CDR2_LE_OLD = 17,
    ENC_PL_CDR2_BE_OLD = 18,
    ENC_PL_CDR2_LE_OLD = 19,
    ENC_DELIMITED_CDR2_BE_OLD = 20,
    ENC_DELIMITED_CDR2_LE_OLD = 21,
    ENC_PLAIN_CDR_BE = 128,
    ENC_PLAIN_CDR_LE = 129
};

const uint16_t ENCAPSULATION_SIZE{4U};
}  // namespace intercom

namespace intercom {
namespace dcps {
namespace xtypes {
using TypeKind = uint8_t;
const TypeKind TK_NONE{0x00U};
const TypeKind TK_BOOLEAN{0x01U};
const TypeKind TK_BYTE{0x02U};
const TypeKind TK_INT16{0x03U};
const TypeKind TK_INT32{0x04U};
const TypeKind TK_INT64{0x05U};
const TypeKind TK_UINT16{0x06U};
const TypeKind TK_UINT32{0x07U};
const TypeKind TK_UINT64{0x08U};
const TypeKind TK_FLOAT32{0x09U};
const TypeKind TK_FLOAT64{0x0aU};
const TypeKind TK_FLOAT128{0x0bU};
const TypeKind TK_INT8{0x0cU};
const TypeKind TK_UINT8{0x0dU};
const TypeKind TK_CHAR8{0x10U};
const TypeKind TK_CHAR16{0x11U};
const TypeKind TK_STRING8{0x20U};
const TypeKind TK_STRING16{0x21U};
const TypeKind TK_ALIAS{0x30U};
const TypeKind TK_ENUM{0x40U};
const TypeKind TK_BITMASK{0x41U};
const TypeKind TK_ANNOTATION{0x50U};
const TypeKind TK_STRUCTURE{0x51U};
const TypeKind TK_UNION{0x52U};
const TypeKind TK_BITSET{0x53U};
const TypeKind TK_SEQUENCE{0x60U};
const TypeKind TK_ARRAY{0x61U};
const TypeKind TK_MAP{0x62U};
using EquivalenceKind = uint8_t;
const uint8_t EK_MINIMAL{0xf1U};
const uint8_t EK_COMPLETE{0xf2U};
const uint8_t EK_BOTH{0xf3U};
using TypeIdentiferKind = uint8_t;
const uint8_t TI_STRING8_SMALL{0x70U};
const uint8_t TI_STRING8_LARGE{0x71U};
const uint8_t TI_STRING16_SMALL{0x72U};
const uint8_t TI_STRING16_LARGE{0x73U};
const uint8_t TI_PLAIN_SEQUENCE_SMALL{0x80U};
const uint8_t TI_PLAIN_SEQUENCE_LARGE{0x81U};
const uint8_t TI_PLAIN_ARRAY_SMALL{0x90U};
const uint8_t TI_PLAIN_ARRAY_LARGE{0x91U};
const uint8_t TI_PLAIN_MAP_SMALL{0xa0U};
const uint8_t TI_PLAIN_MAP_LARGE{0xa1U};
const uint8_t TI_STRONGLY_CONNECTED_COMPONENT{0xb0U};
const int32_t MEMBER_NAME_MAX_LENGTH{256};
using MemberName = ::intercom::bounded_string<256>;
const int32_t TYPE_NAME_MAX_LENGTH{256};
using QualifiedTypeName = ::intercom::bounded_string<256>;
using PrimitiveTypeId = uint8_t;
using EquivalenceHash = ::std::array<uint8_t, 14>;
using NameHash = ::std::array<uint8_t, 4>;
using LBound = uint32_t;
using LBoundSeq = ::std::vector<LBound>;
const LBound INVALID_LBOUND{0U};
using SBound = uint8_t;
using SBoundSeq = ::std::vector<SBound>;
const SBound INVALID_SBOUND{0U};
enum MemberFlagBits : uint16_t {
    TRY_CONSTRUCT1 = 0x1U,
    TRY_CONSTRUCT2 = 0x2U,
    IS_EXTERNAL = 0x4U,
    IS_OPTIONAL = 0x8U,
    IS_MUST_UNDERSTAND = 0x10U,
    IS_KEY = 0x20U,
    IS_DEFAULT = 0x40U
};

using MemberFlag = uint16_t;

enum MemberFlagExtendedBits : uint32_t {
    IS_DISCRIMINATOR = 0x10000U,
    IS_AIR_DUMMY = 0x20000U,
    IS_XRI_SEQUENCE = 0x40000U,
    IS_ELEMENT_SIZE = 0x80000U,
    HAS_DYNAMIC_ELEMENT_SIZE = 0x100000U,
    IS_VENDOR_EXTENSION = 0x200000U,
    IS_IMPLICIT_KEY = 0x400000U,
    IS_INTEGER_RANGE_VALUE = 0x800000U,
    IS_USE_DEFAULT_TRY_CONSTRUCT = 0x1000000U,
    IS_DISCARD_TRY_CONSTRUCT = 0x2000000U,
    IS_TRIM_TRY_CONSTRUCT = 0x4000000U
};

using MemberFlagExtended = uint32_t;

using CollectionElementFlag = MemberFlag;
using StructMemberFlag = MemberFlag;
using UnionMemberFlag = MemberFlag;
using UnionDiscriminatorFlag = MemberFlag;
using EnumeratedLiteralFlag = MemberFlag;
using AnnotationParameterFlag = MemberFlag;
using AliasMemberFlag = MemberFlag;
using BitflagFlag = MemberFlag;
using BitsetMemberFlag = MemberFlag;
const uint16_t MemberFlagMinimalMask{0x3fU};
enum TypeFlagBits : uint16_t {
    IS_FINAL = 0x1U,
    IS_APPENDABLE = 0x2U,
    IS_MUTABLE = 0x4U,
    IS_NESTED = 0x8U,
    IS_AUTOID_HASH = 0x10U
};

using TypeFlag = uint16_t;

using StructTypeFlag = TypeFlag;
using UnionTypeFlag = TypeFlag;
using CollectionTypeFlag = TypeFlag;
using AnnotationTypeFlag = TypeFlag;
using AliasTypeFlag = TypeFlag;
using EnumTypeFlag = TypeFlag;
using BitmaskTypeFlag = TypeFlag;
using BitsetTypeFlag = TypeFlag;
const uint16_t TypeFlagMinimalMask{0x7U};
struct TypeIdentifier;
using MemberId = uint32_t;
const MemberId MEMBER_ID_INVALID{0xfffffffU};
}  // namespace xtypes
}  // namespace dcps
}  // namespace intercom

#ifdef _WIN32
#  pragma warning(pop)
#endif

// NOLINTEND
