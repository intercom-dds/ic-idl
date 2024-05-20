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

find_package(BISON REQUIRED)
find_package(FLEX REQUIRED)

file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/gen/cidl)

bison_target(
    CidlGrammar
        src/idl_grammar.y
        ${CMAKE_CURRENT_BINARY_DIR}/gen/cidl/idl_grammar.tab.c
    COMPILE_FLAGS --name-prefix=idl
)

flex_target(
    CidlTokens
        src/idl_tokens.l
        ${CMAKE_CURRENT_BINARY_DIR}/gen/cidl/idl_tokens.c
    COMPILE_FLAGS -Pidl
)

add_flex_bison_dependency(CidlTokens CidlGrammar)

add_compile_definitions(
    FMT_HEADER_ONLY
    FMT_CONSTEVAL=
    INTERCOM_FMTLIB
)

include_directories(
    ../../../external/fmt/defs
    ../../../external/utils/defs
)

add_library(ic_parser OBJECT
    src/idl_parser.cpp
    ${BISON_CidlGrammar_OUTPUTS}
    ${FLEX_CidlTokens_OUTPUTS}
)

target_include_directories(ic_parser PUBLIC
    defs
    ../../../ic-cpp/defs
)

add_library(ic_ptree STATIC
    src/ast_dump.cpp
    src/code_gen.cpp
    src/constants.cpp
    src/pragma_builder.cpp
    src/ptree_builder.cpp
    src/ptree_ffi.cpp
    src/ptree_helpers.cpp
    # src/ptree_json.cpp
    src/symbols.cpp
    src/commandline.cpp
    src/memf.cpp

    src/proto_code_gen.cpp
    ../../ic-codegen-java/cpp/src/java_code_gen.cpp
    # ../../ic-codegen-cxx/cpp/src/cpp_code_gen.cpp
    # ../../ic-codegen-cxx/cpp/src/cpp_rpc_gen.cpp
)

target_include_directories(ic_ptree PUBLIC defs)

target_include_directories(ic_ptree PRIVATE
    ../../../ic-cpp/defs
    ../../../external/utils/defs
)

target_include_directories(ic_ptree SYSTEM PRIVATE
    ../../../external/fmt/defs
)

target_link_libraries(ic_ptree PRIVATE ic_parser)

add_library(ic_utils OBJECT
    ../../../external/utils/src/StringUtils.cpp)

target_include_directories(ic_utils PUBLIC
    ../../../ic-cpp/defs
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
