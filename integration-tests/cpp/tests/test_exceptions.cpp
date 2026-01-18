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

#include <gtest/gtest.h>

#include <exception>
#include <stdexcept>

#include "generated/exceptions.h"

namespace {

TEST(ExceptionsTest, test_exception_inherits_from_exception) {
    EXPECT_TRUE((std::is_base_of_v<std::exception, exception_types::SimpleError>));

    // Runtime RTTI check via dynamic_cast
    exception_types::SimpleError err(42, "test error");
    std::exception* base_ptr = dynamic_cast<std::exception*>(&err);
    EXPECT_NE(nullptr, base_ptr);
    EXPECT_STREQ(base_ptr->what(), "SimpleError");
}

TEST(ExceptionsTest, test_exception_instantiation) {
    exception_types::SimpleError err(404, "Not found");
    EXPECT_EQ(err.error_code, 404);
    EXPECT_EQ(err.message, "Not found");
}

TEST(ExceptionsTest, test_exception_raise_and_catch) {
    try {
        throw exception_types::SimpleError(500, "Internal error");
        FAIL() << "Expected SimpleError to be thrown";
    } catch (const exception_types::SimpleError& e) {
        EXPECT_EQ(e.error_code, 500);
        EXPECT_EQ(e.message, "Internal error");
    }
}

TEST(ExceptionsTest, test_exception_catch_as_base) {
    try {
        throw exception_types::SimpleError(403, "Forbidden");
        FAIL() << "Expected SimpleError to be thrown";
    } catch (const std::exception& e) {
        EXPECT_NO_THROW(dynamic_cast<const exception_types::SimpleError&>(e));

        const exception_types::SimpleError& simple =
            dynamic_cast<const exception_types::SimpleError&>(e);
        EXPECT_EQ(simple.error_code, 403);
        EXPECT_EQ(simple.message, "Forbidden");
    }
}

TEST(ExceptionsTest, test_empty_exception) {
    exception_types::EmptyError empty;
    EXPECT_TRUE((std::is_base_of_v<std::exception, exception_types::EmptyError>));

    try {
        throw empty;
        FAIL() << "Expected EmptyError to be thrown";
    } catch (const exception_types::EmptyError&) {
        SUCCEED();
    }
}

TEST(ExceptionsTest, test_detailed_exception_fields) {
    exception_types::DetailedError err(1001, "Database error", "Connection timeout", true);
    EXPECT_EQ(err.code, 1001);
    EXPECT_EQ(err.message, "Database error");
    EXPECT_EQ(err.details, "Connection timeout");
    EXPECT_TRUE(err.recoverable);

    exception_types::DetailedError err2(2002, "Fatal error", "Out of memory", false);
    EXPECT_EQ(err2.code, 2002);
    EXPECT_FALSE(err2.recoverable);
}

TEST(ExceptionsTest, test_validation_error) {
    exception_types::ValidationError verr("email", "Invalid format", 15);
    EXPECT_EQ(verr.field_name, "email");
    EXPECT_EQ(verr.error_message, "Invalid format");
    EXPECT_EQ(verr.position, 15);

    try {
        throw verr;
        FAIL() << "Expected ValidationError to be thrown";
    } catch (const exception_types::ValidationError& e) {
        EXPECT_EQ(e.field_name, "email");
        EXPECT_EQ(e.error_message, "Invalid format");
        EXPECT_EQ(e.position, 15);
    }
}

} // namespace
