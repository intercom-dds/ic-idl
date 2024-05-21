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

#include <cstdarg>
#include <cstdio>
#include <stdexcept>
#include <string>
#include <vector>

#include "InterCOM/PlatformConfig.h"

#ifdef INTERCOM_PLATFORM_WINDOWS
#  define _CRT_NO_VA_START_VALIDATION
#endif

namespace intercom::cidl {

#ifdef __GNUC__
// Allows warnings to be created when format string and parameters mismatch
std::string stdprintf(const char* format, ...) __attribute__((format(printf, 1, 2)));
#endif

inline std::string stdprintf(const char* format, ...) {
    va_list ap;
    va_start(ap, format);
    auto len = std::vsnprintf(nullptr, 0, format, ap);
    va_end(ap);
    if (len < 0) {
        // Might happen for a malformed format string, but a program crash on invalid
        // va_list is much more likely.
        throw std::runtime_error("Invalid format string passed to vsnprintf");
    }
    std::vector<char> vec(len + 1);
    va_start(ap, format);
    std::vsnprintf(vec.data(), len + 1, format, ap);
    va_end(ap);
    return vec.data();
}
}  // namespace intercom::cidl

namespace {
// According to wikipedia, there are ~7.2 digits in an IEE754 float32
inline std::string to_string(float value) {
    return intercom::cidl::stdprintf("%.7e", static_cast<double>(value));
}

// According to wikipedia, there are ~15.9 digits in an IEE754 float64
inline std::string to_string(double value) {
    return intercom::cidl::stdprintf("%.16e", value);
}

inline std::string to_string(int value) {
    return intercom::cidl::stdprintf("%d", value);
}

inline std::string to_string(unsigned int value) {
    return intercom::cidl::stdprintf("%u", value);
}

inline std::string to_string(long value) {
    return intercom::cidl::stdprintf("%ld", value);
}

inline std::string to_string(unsigned long value) {
    return intercom::cidl::stdprintf("%lu", value);
}

inline std::string to_string(long long value) {
    return intercom::cidl::stdprintf("%lld", value);
}

inline std::string to_string(unsigned long long value) {
    return intercom::cidl::stdprintf("%llu", value);
}

inline std::string to_string(bool value) {
    return value ? "true" : "false";
}

}  // namespace
