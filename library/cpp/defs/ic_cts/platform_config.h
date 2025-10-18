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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
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

#define INTERCOM_BIG_ENDIAN 0
#define INTERCOM_LITTLE_ENDIAN 1

#define INTERCOM_CONCAT_(a, b) a##b
#define INTERCOM_CONCAT(a, b) INTERCOM_CONCAT_(a, b)

#if defined(_MSC_VER)
#  define INTERCOM_COMPILER "VISUAL_STUDIO"
#  define INTERCOM_COMPILER_MICROSOFT
#  define INTERCOM_COMPILER_VERSION _MSC_VER

#elif defined(__GNUC__)
#  define INTERCOM_COMPILER "GNU"
#  define INTERCOM_COMPILER_GNUC
#  define INTERCOM_COMPILER_VERSION #error "dont know how to find compiler version"

#endif

#if defined(linux) || defined(__linux) || defined(__linux__)
#  define INTERCOM_PLATFORM "LINUX"
#  define INTERCOM_PLATFORM_LINUX
#  define INTERCOM_PLATFORM_ENDIAN LittleEndian
#  define INTERCOM_PLATFORM_VERSION #error "dont know how to find OS version"
#  define INTERCOM_XOPEN_VERSION _XOPEN_VERSION
#  define INTERCOM_PLATFORM_PRINTABLE "Linux"
#  ifndef __GLIBC__
#    define INTERCOM_MUSL
#  endif

#elif defined(WIN32) || defined(_WIN32) || defined(__WIN32__)
#  define INTERCOM_PLATFORM "WINDOWS"
#  define INTERCOM_PLATFORM_WINDOWS
#  define INTERCOM_PLATFORM_ENDIAN LittleEndian
#  define INTERCOM_PLATFORM_VERSION #error "dont know how to find OS version"
#  define INTERCOM_PLATFORM_PRINTABLE "Windows"
#  define PATH_SEPARATOR '\\'
#else
#  error "Unable to figure out target platform, is platform supported?"

#endif
