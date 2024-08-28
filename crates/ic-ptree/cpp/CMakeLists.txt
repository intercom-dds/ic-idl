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

if (MSVC AND CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    # Enable exception support for clang-cl
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /EHsc")
endif()

if (WIN32)
    # Statically link VCRT
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endif()

add_compile_definitions(
    FMT_HEADER_ONLY
    FMT_CONSTEVAL=
)

include_directories(
    ../../../external/utils/defs
    ../../../library/cpp/defs
)

include_directories(SYSTEM
    ../../../external/fmt/defs
)

add_library(ic_ptree OBJECT
    src/code_gen.cpp
    src/commandline.cpp
    src/constants.cpp
    src/idl_parser.cpp
    src/memf.cpp
    src/ptree_builder.cpp
    src/ptree_dump.cpp
    src/ptree_ffi.cpp
    src/ptree_helpers.cpp
    src/symbols.cpp
)

if (PROJECT_IS_TOP_LEVEL)
    add_subdirectory(
        ../../ic-codegen-cxx/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-cxx
    )
    add_subdirectory(
        ../../ic-codegen-idl/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-idl
    )
    add_subdirectory(
        ../../ic-codegen-json/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-json
    )
    add_subdirectory(
        ../../ic-codegen-protobuf/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-protobuf
    )
    add_subdirectory(
        ../../ic-codegen-python/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-python
    )
    add_subdirectory(
        ../../ic-codegen-rust/cpp
        ${CMAKE_CURRENT_BINARY_DIR}/ic-codegen-rust
    )
endif()

target_compile_definitions(ic_ptree PRIVATE
    CIDL_BOOTSTRAP)

target_include_directories(ic_ptree
    PUBLIC defs
    PRIVATE ../../../library/cpp/defs
)

add_library(ic_core STATIC
    $<TARGET_OBJECTS:ic_ptree>
    $<TARGET_OBJECTS:ic_codegen_cxx>
    $<TARGET_OBJECTS:ic_codegen_idl>
    $<TARGET_OBJECTS:ic_codegen_json>
    $<TARGET_OBJECTS:ic_codegen_protobuf>
    $<TARGET_OBJECTS:ic_codegen_python>
    $<TARGET_OBJECTS:ic_codegen_rust>
)

install(
    TARGETS ic_core
    DESTINATION lib
)

install(
    DIRECTORY defs/cidl
    DESTINATION include
)
