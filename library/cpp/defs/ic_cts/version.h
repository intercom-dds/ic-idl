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

#ifndef INTERCOM_VERSION_H_
#define INTERCOM_VERSION_H_

#define INTERCOM_VERSION 3.16.2.2

#define INTERCOM_VERSION_3_16_2
#define INTERCOM_VERSION_S "3_16_2"
#define INTERCOM_VERSION_FULLS "3_16_2_2"
#define INTERCOM_VERSION_STR "3.16"
#define INTERCOM_VERSION_NS v3_16_2

#define INTERCOM_VERSION_MAJOR 3
#define INTERCOM_VERSION_MINOR 16
#define INTERCOM_VERSION_PATCH 2
#define INTERCOM_VERSION_BUILD 2

#define INTERCOM_VERSION_CREATE_HEX(_major, _minor, _patch, _build) \
    ((_major << 24) | (_minor << 16) | (_patch << 8) | (_build))

#define INTERCOM_VERSION_HEX     \
    INTERCOM_VERSION_CREATE_HEX( \
        INTERCOM_VERSION_MAJOR,  \
        INTERCOM_VERSION_MINOR,  \
        INTERCOM_VERSION_PATCH,  \
        INTERCOM_VERSION_BUILD   \
    )

#define INTERCOM_CXX_STANDARD 11

#define INTERCOM_BUILD_DATE "2024-04-26"
#define INTERCOM_BUILD_BRANCH "develop"
#define INTERCOM_BUILD_COMMIT "83555552a0b59407dd1f8a3d249cdf474caec2b0"
#define INTERCOM_BUILD_VARIANT "int64_ubuntu22.04_clang18.1.0"

#define INTERCOM_BUILD_ARCH "int64"
#define INTERCOM_BUILD_OS "Linux"
#define INTERCOM_BUILD_OS_VERSION "6.5.0-25-generic"
#define INTERCOM_MSVC_TOOLSET_VERSION
#define INTERCOM_C_COMPILER "Clang"
#define INTERCOM_C_COMPILER_VERSION "18.1.0"
#define INTERCOM_CXX_COMPILER "Clang"
#define INTERCOM_CXX_COMPILER_VERSION "18.1.0"

#define INTERCOM_JAVA_VERSION ""
#define INTERCOM_GNATMAKE_VERSION ""
#define INTERCOM_DOTNET_VERSION ""
#define INTERCOM_DOTNET_TARGET_FRAMEWORK ""

#endif  // INTERCOM_VERSION_H
