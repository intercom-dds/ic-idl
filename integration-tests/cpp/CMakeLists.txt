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
project(ic_idl_cpp_integration_tests CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Add compiler warnings
if(MSVC)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -pedantic)
endif()

# Sanitizer support (ASAN + UBSAN are compatible and commonly used together)
option(ENABLE_SANITIZERS "Enable AddressSanitizer and UndefinedBehaviorSanitizer" OFF)

if(ENABLE_SANITIZERS)
    message(STATUS "Sanitizers enabled: AddressSanitizer + UndefinedBehaviorSanitizer")
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g)
    add_link_options(-fsanitize=address,undefined)
endif()

include(FetchContent)
FetchContent_Declare(
    googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG v1.14.0
)
FetchContent_MakeAvailable(googletest)

enable_testing()

set(GENERATED_SOURCES
    # generated/annotations.cpp
    # generated/any_type.cpp
    generated/bitmasks.cpp
    generated/bounded_types.cpp
    generated/circular_types.cpp
    generated/constants.cpp
    generated/deep_generics.cpp
    generated/defaults.cpp
    generated/enums.cpp
    generated/exceptions.cpp
    generated/interfaces.cpp
    generated/multi_module.cpp
    generated/nested_modules.cpp
    generated/strings.cpp
    generated/structs.cpp
    generated/typedefs.cpp
    generated/unions.cpp
    generated/valuetypes.cpp
)

add_library(generated_idl ${GENERATED_SOURCES})

target_include_directories(generated_idl PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/
    ${CMAKE_CURRENT_SOURCE_DIR}/../../library/cpp/defs
)

add_executable(integration_tests
    # tests/test_annotations.cpp
    # tests/test_any_type.cpp
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
)

target_link_libraries(integration_tests PRIVATE
    generated_idl
    GTest::gtest_main
)

target_include_directories(integration_tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/
    ${CMAKE_CURRENT_SOURCE_DIR}/../../library/cpp/defs
)

include(GoogleTest)
gtest_discover_tests(integration_tests)
