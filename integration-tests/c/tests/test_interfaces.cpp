// Copyright 2026 KONGSBERG
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

#include <doctest/doctest.h>

#include <cstdint>

#include "interfaces.h"

namespace {

idl_status_t add(void* context, int32_t a, int32_t b, int32_t* out, idl_error_t* error) {
    if (error != nullptr) {
        *error = {};
    }

    *out = a + b + *static_cast<int32_t*>(context);
    return IDL_STATUS_OK;
}

idl_status_t read(void* context, const char** out, idl_error_t* error) {
    if (error != nullptr) {
        *error = {};
    }

    *out = static_cast<const char*>(context);
    return IDL_STATUS_OK;
}

idl_status_t reset(void* context, idl_error_t* error) {
    if (error != nullptr) {
        *error = {};
    }

    *static_cast<bool*>(context) = true;
    return IDL_STATUS_OK;
}

idl_status_t get_count(void* context, int32_t* out, idl_error_t* error) {
    if (error != nullptr) {
        *error = {};
    }

    *out = *static_cast<int32_t*>(context);
    return IDL_STATUS_OK;
}

idl_status_t set_count(void* context, int32_t value, idl_error_t* error) {
    if (error != nullptr) {
        *error = {};
    }

    *static_cast<int32_t*>(context) = value;
    return IDL_STATUS_OK;
}

}  // namespace

TEST_CASE("interface callback" * doctest::test_suite("interfaces")) {
    interface_types_Calculator calculator{};
    calculator.add = add;

    int32_t offset = 1;
    int32_t result = 0;
    idl_error_t error{reinterpret_cast<const idl_type_t*>(1), reinterpret_cast<void*>(1)};

    CHECK(calculator.add(&offset, 2, 3, &result, &error) == IDL_STATUS_OK);
    CHECK(result == 6);
    CHECK(error.type == nullptr);
    CHECK(error.value == nullptr);
    CHECK(calculator.add(&offset, 2, 3, &result, nullptr) == IDL_STATUS_OK);
}

TEST_CASE("embedded parent tables" * doctest::test_suite("interfaces")) {
    interface_types_ReadWriter reader_writer{};
    reader_writer.interface_types_Reader_base.read = read;
    reader_writer.reset = reset;

    bool was_reset = false;
    char data[] = "data";
    const char* result = nullptr;

    CHECK(reader_writer.interface_types_Reader_base.read(data, &result, nullptr) == IDL_STATUS_OK);
    CHECK(result[0] == 'd');
    CHECK(reader_writer.reset(&was_reset, nullptr) == IDL_STATUS_OK);
    CHECK(was_reset);
}

TEST_CASE("attribute callbacks" * doctest::test_suite("interfaces")) {
    interface_types_WithAttribute attributes{};
    attributes.get_count = get_count;
    attributes.set_count = set_count;

    int32_t count = 4;
    int32_t result = 0;

    CHECK(attributes.get_count(&count, &result, nullptr) == IDL_STATUS_OK);
    CHECK(result == 4);
    CHECK(attributes.set_count(&count, 8, nullptr) == IDL_STATUS_OK);
    CHECK(count == 8);
}
