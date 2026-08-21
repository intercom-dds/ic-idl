# Copyright 2026 KONGSBERG
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
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
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
project(cpp_integration_tests CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_INCLUDE_CURRENT_DIR ON)

if(MSVC)
    add_compile_options(
        /MP
        /W4
        /w14265
        /w14263
        /w14242
        /WX
    )
else()
    add_compile_options(
        -Wall
        -Wextra
        -Wpedantic
        -Wconversion
        -Wshadow
        -Wnon-virtual-dtor
        -Wold-style-cast
        -Woverloaded-virtual
        -Wunused
        -Wcast-align
        -Wno-switch-bool
        -Wnull-dereference
        -Werror
    )
endif()

option(ENABLE_SANITIZERS "Enable AddressSanitizer and UndefinedBehaviorSanitizer" OFF)

if(ENABLE_SANITIZERS)
    message(STATUS "Sanitizers enabled: AddressSanitizer + UndefinedBehaviorSanitizer")
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g)
    add_link_options(-fsanitize=address,undefined)
endif()

include(FetchContent)

FetchContent_Declare(
    doctest
    GIT_REPOSITORY https://github.com/doctest/doctest.git
    GIT_TAG v2.4.12
)
FetchContent_MakeAvailable(doctest)

set(IC_IDL_EXECUTABLE "${CMAKE_CURRENT_SOURCE_DIR}/../../target/debug/ic-idl")

find_package(
    ic_idl REQUIRED
    PATHS ../../runtime/cpp/cmake
    NO_DEFAULT_PATH
)

idl_generate(
    LANGUAGE CPP
    INPUT_IDL ../corpus
)

add_executable(cpp_integration_tests
    tests/main.cpp
    tests/test_annotations.cpp
    tests/test_any_type.cpp
    tests/test_bitmasks.cpp
    tests/test_bounded_types.cpp
    tests/test_circular_types.cpp
    tests/test_comparison.cpp
    tests/test_constants.cpp
    tests/test_deep_generics.cpp
    tests/test_defaults.cpp
    tests/test_enums.cpp
    tests/test_exceptions.cpp
    tests/test_interfaces.cpp
    tests/test_multi_module.cpp
    tests/test_nested_modules.cpp
    tests/test_strings.cpp
    tests/test_structs.cpp
    tests/test_typedefs.cpp
    tests/test_unions.cpp
    tests/test_valuetypes.cpp
    ${IDL_GENERATE_OUTPUTS}
)

target_link_libraries(cpp_integration_tests PRIVATE
    ic_idl::ic_cts
    doctest::doctest
)

add_custom_target(test
    COMMAND cpp_integration_tests
    DEPENDS cpp_integration_tests
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)

include(CheckCXXSourceCompiles)

set(SAVED_CXX_STANDARD ${CMAKE_CXX_STANDARD})
set(CMAKE_CXX_STANDARD 20)
check_cxx_source_compiles("
    #include <format>
    #include <string>
    int main() { return std::format(\"{}\", 1).empty() ? 1 : 0; }
    " HAVE_STD_FORMAT)
set(CMAKE_CXX_STANDARD ${SAVED_CXX_STANDARD})

if(HAVE_STD_FORMAT)
    idl_generate(
        LANGUAGE CPP
        INPUT_IDL ../corpus
        DESTINATION use_fmt
        FLAGS --use-fmt
        OUTPUT_VAR IDL_FORMAT_OUTPUTS
    )

    add_executable(cpp_format_tests
        tests/main.cpp
        tests/test_format.cpp
        ${IDL_FORMAT_OUTPUTS}
    )

    target_compile_features(cpp_format_tests PRIVATE cxx_std_20)

    target_link_libraries(cpp_format_tests PRIVATE
        ic_idl::ic_cts
        doctest::doctest
    )

    add_custom_target(format_test
        COMMAND cpp_format_tests
        DEPENDS cpp_format_tests
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    )

    add_dependencies(test format_test)
else()
    message(STATUS "std::format not available, skipping --use-fmt tests")
endif()
