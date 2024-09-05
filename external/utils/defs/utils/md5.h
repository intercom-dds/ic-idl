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

#include <array>
#include <cstdint>
#include <string>

namespace intercom {

struct MD5Context {
  public:
    MD5Context();

    std::array<unsigned int, 4> state{};  // transformation state

    unsigned int count = 0;  // number of bytes stored

    std::array<uint8_t, 65> buffer{};  // current buffer to transform
};

//! MD5 calculation as defined by RFC-1321 from April 1992.
//! The class creates an object from a buffer/length that contains after initialization a
//! digest that can be extracted directly (16 octets) or as a hexadecimal string (32 chars +
//! null-terminating)
class MD5 {
  public:
    //! creates an MD5 object
    //! \param buffer source data to use for calculation
    //! \param lenth number of bytes in the buffer
    MD5(const uint8_t* buffer, unsigned int length);

    //! Creates an open MD5 object
    MD5();

    //! Append data to the hash
    void append(const uint8_t* buffer, unsigned int length);

    //! Close the MD5 object
    void finalize();

    //! \return digest of the MD5 calculation. The returned buffer contains exactly 16 octets.
    const std::array<uint8_t, 16>& digest() const;

    //! \return digest of the MD5 calculation as a string of hexadecial values (lower-case, no 0x
    //! prefix )
    std::string to_string() const;

  private:
    bool m_closed;

    std::array<uint8_t, 16> m_digest{};

    MD5Context m_context;
};

}  // namespace intercom

#include "md5.ic"
