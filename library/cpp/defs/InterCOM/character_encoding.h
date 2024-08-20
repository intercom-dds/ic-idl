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

#include <iostream>

#include "InterCOM/integer_types.h"

namespace intercom {
enum CharacterEncoding { EIGHT_BIT = 0, UTF8 = 1, UTF16 = 2, UTF32 = 4 };

#ifdef INTERCOM_DEFAULT_EIGHT_BIT_STRING
const CharacterEncoding DEFAULT_ENCODING = EIGHT_BIT;
#else
const CharacterEncoding DEFAULT_ENCODING = UTF8;
#endif

template <CharacterEncoding ENCODING>
struct character_type_of {};

template <>
struct character_type_of<EIGHT_BIT> {
    using type = char;
};
template <>
struct character_type_of<UTF8> {
    using type = char;
};
template <>
struct character_type_of<UTF16> {
    using type = char16_t;
};
template <>
struct character_type_of<UTF32> {
    using type = char32_t;
};

template <typename T>
struct encoding_type_of {};
template <>
struct encoding_type_of<char> {
    static const CharacterEncoding kind = DEFAULT_ENCODING;
};
template <>
struct encoding_type_of<char16_t> {
    static const CharacterEncoding kind = UTF16;
};
template <>
struct encoding_type_of<char32_t> {
    static const CharacterEncoding kind = UTF32;
};
template <>
struct encoding_type_of<wchar_t> {
    static const CharacterEncoding kind = static_cast<CharacterEncoding>(sizeof(wchar_t));
};

const char32_t UNICODE_INVALID = ~0u;

inline bool isLegalUnicode(char32_t code) {
    return (code <= 0x10FFFF && !(code >= 0xD800 && code <= 0xDFFF));
}

template <CharacterEncoding ENCODING>
int writeCharCode(typename character_type_of<ENCODING>::type* buf, char32_t code);

template <>
inline int writeCharCode<EIGHT_BIT>(character_type_of<EIGHT_BIT>::type* buf, char32_t code) {
    if (code >= 256) {
        return 0;
    }
    *buf = static_cast<char>(code);
    return 1;
}

template <>
inline int writeCharCode<UTF8>(character_type_of<UTF8>::type* buf, char32_t code) {
    if (!isLegalUnicode(code)) {
        return 0;
    } else if (code <= 0x7F) {
        buf[0] = static_cast<char>(code);
        return 1;
    } else if (code <= 0x7FF) {
        buf[0] = static_cast<char>(0xC0 | (code >> 6));
        buf[1] = static_cast<char>(0x80 | (code & 0x3F));
        return 2;
    } else if (code <= 0xFFFF) {
        buf[0] = static_cast<char>(0xE0 | (code >> 12));
        buf[1] = static_cast<char>(0x80 | ((code >> 6) & 0x3F));
        buf[2] = static_cast<char>(0x80 | (code & 0x3F));
        return 3;
    } else if (code <= 0x10FFFF) {
        buf[0] = static_cast<char>(0xF0 | (code >> 18));
        buf[1] = static_cast<char>(0x80 | ((code >> 12) & 0x3F));
        buf[2] = static_cast<char>(0x80 | ((code >> 6) & 0x3F));
        buf[3] = static_cast<char>(0x80 | (code & 0x3F));
        return 4;
    }
    return 0;
}

template <>
inline int writeCharCode<UTF16>(character_type_of<UTF16>::type* buf, char32_t code) {
    if (!isLegalUnicode(code)) {
        return 0;
    }
    if (code <= 0xFFFF) {
        buf[0] = static_cast<char16_t>(code);
        return 1;
    } else {
        code -= 0x10000;
        buf[0] = static_cast<char16_t>(0xD800 | ((code >> 10) & 0x3FF));
        buf[1] = static_cast<char16_t>(0xDC00 | (code & 0x3FF));
        return 2;
    }
}

template <>
inline int writeCharCode<UTF32>(character_type_of<UTF32>::type* buf, char32_t code) {
    if (!isLegalUnicode(code)) {
        return 0;
    }
    *buf = code;
    return 1;
}

template <CharacterEncoding ENCODING>
int readCharCode(char32_t& code, const typename character_type_of<ENCODING>::type* buf);

template <>
inline int readCharCode<EIGHT_BIT>(char32_t& code, const character_type_of<EIGHT_BIT>::type* buf) {
    code = static_cast<unsigned char>(buf[0]);
    if (!isLegalUnicode(code)) {
        code = UNICODE_INVALID;
    }
    return 1;
}

template <>
inline int readCharCode<UTF8>(char32_t& code, const character_type_of<UTF8>::type* buf) {
    int extra_bytes = 0;
    auto c = static_cast<unsigned char>(buf[0]);
    if ((c & 0x80) == 0) {
        extra_bytes = 0;
        // code_min = 0x20;
        code = c;
    } else if ((c & 0xE0) == 0xC0) {
        extra_bytes = 1;
        // code_min = 0x80;
        code = static_cast<char32_t>(c & 0x1F);
    } else if ((c & 0xF0) == 0xE0) {
        extra_bytes = 2;
        // code_min = 0x800;
        code = static_cast<char32_t>(c & 0x0F);
    } else if ((c & 0xF8) == 0xF0) {
        extra_bytes = 3;
        // code_min = 0x10000;
        code = static_cast<char32_t>(c & 0x07);
    }
    for (int i = 1; i <= extra_bytes; ++i) {
        c = static_cast<unsigned char>(buf[i]);
        if ((c & 0xC0) == 0x80) {
            code = (code << 6) + static_cast<char32_t>(c & 0x3F);
        } else {
            code = UNICODE_INVALID;
            extra_bytes = i - 1;
            break;
        }
    }
    if (!isLegalUnicode(code)) {
        code = UNICODE_INVALID;
    }
    return extra_bytes + 1;
}

template <>
inline int readCharCode<UTF16>(char32_t& code, const character_type_of<UTF16>::type* buf) {
    if ((static_cast<uint16_t>(buf[0]) & ~0x7FFu) == 0xD800u &&
        (static_cast<uint16_t>(buf[1]) & ~0x3FFu) == 0xDC00u) {
        code = ((static_cast<uint16_t>(buf[0]) & 0x3FFu) << 10) +
               (static_cast<uint16_t>(buf[1]) & 0x3FFu) + 0x10000u;
        if (!isLegalUnicode(code)) {
            code = UNICODE_INVALID;
        }
        return 2;
    } else {
        code = static_cast<uint16_t>(buf[0]);
        if (!isLegalUnicode(code)) {
            code = UNICODE_INVALID;
        }
        return 1;
    }
}

template <>
inline int readCharCode<UTF32>(char32_t& code, const character_type_of<UTF32>::type* buf) {
    code = *buf;
    if (!isLegalUnicode(code)) {
        code = UNICODE_INVALID;
    }
    return 1;
}

template <typename T>
int readCharCode(char32_t& code, T* buf);

template <>
inline int readCharCode(char32_t& code, const char* buf) {
    return readCharCode<UTF8>(code, buf);
}

template <>
inline int readCharCode(char32_t& code, const char16_t* buf) {
    return readCharCode<UTF16>(code, buf);
}

template <>
inline int readCharCode(char32_t& code, const char32_t* buf) {
    return readCharCode<UTF32>(code, buf);
}

template <>
inline int readCharCode(char32_t& code, const wchar_t* buf) {
    return readCharCode<encoding_type_of<wchar_t>::kind>(
        code, reinterpret_cast<const character_type_of<encoding_type_of<wchar_t>::kind>::type*>(buf)
    );
}

template <typename T>
int writeCharCode(T* buf, char32_t code);

template <>
inline int writeCharCode(char* buf, char32_t code) {
    return writeCharCode<UTF8>(buf, code);
}

template <>
inline int writeCharCode(char16_t* buf, char32_t code) {
    return writeCharCode<UTF16>(buf, code);
}

template <>
inline int writeCharCode(char32_t* buf, char32_t code) {
    return writeCharCode<UTF32>(buf, code);
}

template <>
inline int writeCharCode(wchar_t* buf, char32_t code) {
    return writeCharCode<encoding_type_of<wchar_t>::kind>(
        reinterpret_cast<character_type_of<encoding_type_of<wchar_t>::kind>::type*>(buf), code
    );
}

}  // namespace intercom
