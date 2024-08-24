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

cmake_minimum_required(VERSION 3.20)

project(ic-ptree VERSION 0.1)

set(CMAKE_CXX_STANDARD 17)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_MODULES}/cmake")

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/gen/cidl)

add_compile_definitions(
    FMT_HEADER_ONLY
    FMT_CONSTEVAL=
    INTERCOM_FMTLIB
)

add_subdirectory(
    ../../../external/utils
    ${CMAKE_CURRENT_BINARY_DIR}/utils
)

add_library(ic_ptree STATIC
    src/ast.cpp
    src/ast_dump.cpp
    src/code_gen.cpp
    src/commandline.cpp
    src/constants.cpp
    src/idl_parser.cpp
    src/memf.cpp
    src/ptree_builder.cpp
    src/ptree_ffi.cpp
    src/ptree_helpers.cpp
    src/symbols.cpp

    ../../ic-codegen-cxx/cpp/src/cpp_code_gen.cpp
    ../../ic-codegen-cxx/cpp/src/cpp_typeinfo.cpp
    ../../ic-codegen-idl/cpp/src/idl_code_gen.cpp
    ../../ic-codegen-json/cpp/src/json_code_gen.cpp
    ../../ic-codegen-protobuf/cpp/src/proto_code_gen.cpp
    ../../ic-codegen-python/cpp/src/python_code_gen.cpp
)

target_compile_definitions(ic_ptree PRIVATE CIDL_BOOTSTRAP)

target_include_directories(ic_ptree
    PUBLIC defs
    PRIVATE ../../../library/cpp/defs
)

target_include_directories(ic_ptree SYSTEM PRIVATE
    ../../../external/fmt/defs
)

target_link_libraries(ic_ptree PRIVATE ic_utils)

install(
    TARGETS ic_ptree
    DESTINATION lib
)

install(
    DIRECTORY defs/cidl
    DESTINATION include
)
