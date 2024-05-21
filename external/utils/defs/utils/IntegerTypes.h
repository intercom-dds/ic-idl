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

#include <cstdint>
#include <stdexcept>

namespace intercom {

enum EndianType {
    BigEndian = 0,
    LittleEndian = 1,
    EndianMask = 1,
};

template <int E, typename T>
void get_uint(const uint8_t* b, T& v);

template <>
inline void get_uint<BigEndian, uint8_t>(const uint8_t* b, uint8_t& v) {
    v = b[0];
}

template <>
inline void get_uint<BigEndian, uint16_t>(const uint8_t* b, uint16_t& v) {
    v = static_cast<uint16_t>(b[1]) | static_cast<uint16_t>(b[0]) << 8;
}

template <>
inline void get_uint<BigEndian, uint32_t>(const uint8_t* b, uint32_t& v) {
    v = static_cast<uint32_t>(b[3]) | static_cast<uint32_t>(b[2]) << 8 |
        static_cast<uint32_t>(b[1]) << 16 | static_cast<uint32_t>(b[0]) << 24;
}

template <>
inline void get_uint<BigEndian, uint64_t>(const uint8_t* b, uint64_t& v) {
    v = static_cast<uint64_t>(b[7]) | static_cast<uint64_t>(b[6]) << 8 |
        static_cast<uint64_t>(b[5]) << 16 | static_cast<uint64_t>(b[4]) << 24 |
        static_cast<uint64_t>(b[3]) << 32 | static_cast<uint64_t>(b[2]) << 40 |
        static_cast<uint64_t>(b[1]) << 48 | static_cast<uint64_t>(b[0]) << 56;
}

template <>
inline void get_uint<LittleEndian, uint8_t>(const uint8_t* b, uint8_t& v) {
    v = b[0];
}

template <>
inline void get_uint<LittleEndian, uint16_t>(const uint8_t* b, uint16_t& v) {
    v = static_cast<uint16_t>(b[0]) | static_cast<uint16_t>(b[1]) << 8;
}

template <>
inline void get_uint<LittleEndian, uint32_t>(const uint8_t* b, uint32_t& v) {
    v = static_cast<uint32_t>(b[0]) | static_cast<uint32_t>(b[1]) << 8 |
        static_cast<uint32_t>(b[2]) << 16 | static_cast<uint32_t>(b[3]) << 24;
}

template <>
inline void get_uint<LittleEndian, uint64_t>(const uint8_t* b, uint64_t& v) {
    v = static_cast<uint64_t>(b[0]) | static_cast<uint64_t>(b[1]) << 8 |
        static_cast<uint64_t>(b[2]) << 16 | static_cast<uint64_t>(b[3]) << 24 |
        static_cast<uint64_t>(b[4]) << 32 | static_cast<uint64_t>(b[5]) << 40 |
        static_cast<uint64_t>(b[6]) << 48 | static_cast<uint64_t>(b[7]) << 56;
}

template <int E, typename T>
void put_uint(uint8_t* b, T v);

template <>
inline void put_uint<BigEndian, uint8_t>(uint8_t* b, uint8_t v) {
    b[0] = static_cast<uint8_t>(v);
}

template <>
inline void put_uint<BigEndian, uint16_t>(uint8_t* b, uint16_t v) {
    b[0] = static_cast<uint8_t>(v >> 8);
    b[1] = static_cast<uint8_t>(v);
}

template <>
inline void put_uint<BigEndian, uint32_t>(uint8_t* b, uint32_t v) {
    b[0] = static_cast<uint8_t>(v >> 24);
    b[1] = static_cast<uint8_t>(v >> 16);
    b[2] = static_cast<uint8_t>(v >> 8);
    b[3] = static_cast<uint8_t>(v);
}

template <>
inline void put_uint<BigEndian, uint64_t>(uint8_t* b, uint64_t v) {
    b[0] = static_cast<uint8_t>(v >> 56);
    b[1] = static_cast<uint8_t>(v >> 48);
    b[2] = static_cast<uint8_t>(v >> 40);
    b[3] = static_cast<uint8_t>(v >> 32);
    b[4] = static_cast<uint8_t>(v >> 24);
    b[5] = static_cast<uint8_t>(v >> 16);
    b[6] = static_cast<uint8_t>(v >> 8);
    b[7] = static_cast<uint8_t>(v);
}

template <>
inline void put_uint<LittleEndian, uint8_t>(uint8_t* b, uint8_t v) {
    b[0] = static_cast<uint8_t>(v);
}

template <>
inline void put_uint<LittleEndian, uint16_t>(uint8_t* b, uint16_t v) {
    b[0] = static_cast<uint8_t>(v);
    b[1] = static_cast<uint8_t>(v >> 8);
}

template <>
inline void put_uint<LittleEndian, uint32_t>(uint8_t* b, uint32_t v) {
    b[0] = static_cast<uint8_t>(v);
    b[1] = static_cast<uint8_t>(v >> 8);
    b[2] = static_cast<uint8_t>(v >> 16);
    b[3] = static_cast<uint8_t>(v >> 24);
}

template <>
inline void put_uint<LittleEndian, uint64_t>(uint8_t* b, uint64_t v) {
    b[0] = static_cast<uint8_t>(v);
    b[1] = static_cast<uint8_t>(v >> 8);
    b[2] = static_cast<uint8_t>(v >> 16);
    b[3] = static_cast<uint8_t>(v >> 24);
    b[4] = static_cast<uint8_t>(v >> 32);
    b[5] = static_cast<uint8_t>(v >> 40);
    b[6] = static_cast<uint8_t>(v >> 48);
    b[7] = static_cast<uint8_t>(v >> 56);
}

}  // namespace intercom
