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

#include <map>

#include "InterCOM/cdr_serializer.h"
#include "InterCOM/json_parser.h"
#include "InterCOM/optional.h"
#include "InterCOM/serialization_support.h"

namespace intercom {
namespace dcps {
namespace cts {
using JsonMarshal = TGenericMarshal<JsonWriter>;
using KeyOnlyJsonMarshal = TGenericMarshal<KeyOnlyWriter<JsonWriter>>;
using JsonUnmarshal = TGenericUnmarshal<JsonReader>;
using KeyOnlyJsonUnmarshal = TGenericUnmarshal<KeyOnlyReader<JsonReader>>;
}  // namespace cts
}  // namespace dcps

template <typename T>
std::ostream& marshal_json(std::ostream& stream, const T& value, bool pretty = false) {
    SerializerFlags flags = pretty ? SERIALIZER_PRETTY : SerializerFlagsBits{};
    JsonWriter writer(stream, flags);
    dcps::cts::GenericMarshal(writer).io(value);
    return stream;
}

template <>
inline std::ostream&
marshal_json(std::ostream& stream, const dcps::DynamicDataPtr& value, bool pretty) {
    SerializerFlags flags = pretty ? SERIALIZER_PRETTY : SerializerFlagsBits{};
    JsonWriter writer(stream, flags);
    dcps::cts::GenericMarshal(writer).io(value);
    return stream;
}

template <typename T>
std::ostream& marshal_json(std::ostream& stream, const T& value, SerializerFlags flags) {
    JsonWriter writer(stream, flags);
    dcps::cts::GenericMarshal(writer).io(value);
    return stream;
}

template <>
inline std::ostream&
marshal_json(std::ostream& stream, const dcps::DynamicDataPtr& value, SerializerFlags flags) {
    JsonWriter writer(stream, flags);
    dcps::cts::GenericMarshal(writer).io(value);
    return stream;
}

template <typename T>
std::istream& unmarshal_json(std::istream& stream, T& value, SerializerFlags flags) {
    JsonReader reader(stream, flags);
    dcps::cts::GenericUnmarshal(reader).io(value);
    return stream;
}

template <typename T>
std::istream& unmarshal_json(std::istream& stream, T& value, bool strict = false) {
    SerializerFlags flags = strict ? SERIALIZER_STRICT : SerializerFlagsBits{};
    JsonReader reader(stream, flags);
    dcps::cts::GenericUnmarshal(reader).io(value);
    return stream;
}
}  // namespace intercom
