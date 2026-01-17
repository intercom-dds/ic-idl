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

#include <exception>
#include <stdexcept>

#include "generated/exceptions.h"

TEST_CASE("exception_inherits_from_exception" * doctest::test_suite("exceptions")) {
    CHECK((std::is_base_of_v<std::exception, exception_types::SimpleError>));

    // Runtime RTTI check via dynamic_cast
    exception_types::SimpleError err(42, "test error");
    std::exception* base_ptr = dynamic_cast<std::exception*>(&err);
    CHECK(nullptr != base_ptr);
    CHECK(std::string(base_ptr->what()) == "SimpleError");
}

TEST_CASE("exception_instantiation" * doctest::test_suite("exceptions")) {
    exception_types::SimpleError err(404, "Not found");
    CHECK(err.error_code == 404);
    CHECK(err.message == "Not found");
}

TEST_CASE("exception_raise_and_catch" * doctest::test_suite("exceptions")) {
    try {
        throw exception_types::SimpleError(500, "Internal error");
    } catch (const exception_types::SimpleError& e) {
        CHECK(e.error_code == 500);
        CHECK(e.message == "Internal error");
    } catch (...) {
        FAIL("Exception not caught properly");
    }
}

TEST_CASE("exception_catch_as_base" * doctest::test_suite("exceptions")) {
    try {
        throw exception_types::SimpleError(403, "Forbidden");
    } catch (const std::exception& e) {
        CHECK_NOTHROW(std::ignore = dynamic_cast<const exception_types::SimpleError&>(e));

        const exception_types::SimpleError& simple =
            dynamic_cast<const exception_types::SimpleError&>(e);
        CHECK(simple.error_code == 403);
        CHECK(simple.message == "Forbidden");
    } catch (...) {
        FAIL("Exception not caught properly");
    }
}

TEST_CASE("empty_exception" * doctest::test_suite("exceptions")) {
    exception_types::EmptyError empty;
    CHECK((std::is_base_of_v<std::exception, exception_types::EmptyError>));

    try {
        throw empty;
    } catch (const exception_types::EmptyError&) {
        CHECK(true);
    } catch (...) {
        FAIL("Exception not caught properly");
    }
}

TEST_CASE("detailed_exception_fields" * doctest::test_suite("exceptions")) {
    exception_types::DetailedError err(1001, "Database error", "Connection timeout", true);
    CHECK(err.code == 1001);
    CHECK(err.message == "Database error");
    CHECK(err.details == "Connection timeout");
    CHECK(err.recoverable);

    exception_types::DetailedError err2(2002, "Fatal error", "Out of memory", false);
    CHECK(err2.code == 2002);
    CHECK_FALSE(err2.recoverable);
}

TEST_CASE("validation_error" * doctest::test_suite("exceptions")) {
    exception_types::ValidationError verr("email", "Invalid format", 15);
    CHECK(verr.field_name == "email");
    CHECK(verr.error_message == "Invalid format");
    CHECK(verr.position == 15);

    try {
        throw verr;
    } catch (const exception_types::ValidationError& e) {
        CHECK(e.field_name == "email");
        CHECK(e.error_message == "Invalid format");
        CHECK(e.position == 15);
    } catch (...) {
        FAIL("Exception not caught properly");
    }
}
