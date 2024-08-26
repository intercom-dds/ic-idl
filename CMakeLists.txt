# Copyright 2024 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

# Note: this file is only useful for generating a `compile_commands.json` file.
# It is not used by Cargo, and anything compiled using this manifest will not
# be used in the compiled ic-idl binary.

cmake_minimum_required(VERSION 3.20)

project(
    ic-idl
    VERSION 0.1
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 17)

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/gen/cidl)

add_compile_definitions(
    FMT_HEADER_ONLY
    FMT_CONSTEVAL=
)

include_directories(library/cpp/defs)

add_subdirectory(external/utils)
add_subdirectory(external/fmt)

add_subdirectory(crates/ic-ptree/cpp)
add_subdirectory(crates/ic-codegen-cxx/cpp)
add_subdirectory(crates/ic-codegen-idl/cpp)
add_subdirectory(crates/ic-codegen-json/cpp)
add_subdirectory(crates/ic-codegen-protobuf/cpp)
add_subdirectory(crates/ic-codegen-python/cpp)
add_subdirectory(crates/ic-codegen-rust/cpp)
