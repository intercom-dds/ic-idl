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

#include <type_traits>

#include "enums.h"

TEST_CASE("enum_members_exist" * doctest::test_suite("enums")) {
    enum_types::Color red = enum_types::Color::RED;
    enum_types::Color green = enum_types::Color::GREEN;
    enum_types::Color blue = enum_types::Color::BLUE;
    (void)red;
    (void)green;
    (void)blue;
}

TEST_CASE("enum_is_enum_type" * doctest::test_suite("enums")) {
    CHECK(std::is_enum<enum_types::Color>::value);
    CHECK(std::is_enum<enum_types::Status>::value);
    CHECK(std::is_enum<enum_types::GappedEnum>::value);
    CHECK(std::is_enum<enum_types::NegativeEnum>::value);
    CHECK(std::is_enum<enum_types::MixedEnum>::value);
}

TEST_CASE("enum_auto_values" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::Color::RED) == 0);
    CHECK(static_cast<int32_t>(enum_types::Color::GREEN) == 1);
    CHECK(static_cast<int32_t>(enum_types::Color::BLUE) == 2);
}

TEST_CASE("enum_explicit_values" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::Status::OK) == 0);
    CHECK(static_cast<int32_t>(enum_types::Status::WARNING) == 100);
    CHECK(static_cast<int32_t>(enum_types::Status::ERROR) == 200);
}

TEST_CASE("enum_comparison" * doctest::test_suite("enums")) {
    CHECK(enum_types::Color::RED == enum_types::Color::RED);
    CHECK_FALSE(enum_types::Color::RED == enum_types::Color::BLUE);
    CHECK(enum_types::Color::RED != enum_types::Color::BLUE);
    CHECK_FALSE(enum_types::Color::RED != enum_types::Color::RED);

    CHECK(enum_types::Status::WARNING == enum_types::Status::WARNING);
    CHECK(enum_types::Status::OK != enum_types::Status::ERROR);
}

TEST_CASE("enum_by_value" * doctest::test_suite("enums")) {
    enum_types::Color c0 = static_cast<enum_types::Color>(0);
    enum_types::Color c1 = static_cast<enum_types::Color>(1);
    enum_types::Color c2 = static_cast<enum_types::Color>(2);

    CHECK(c0 == enum_types::Color::RED);
    CHECK(c1 == enum_types::Color::GREEN);
    CHECK(c2 == enum_types::Color::BLUE);

    enum_types::Status s100 = static_cast<enum_types::Status>(100);
    CHECK(s100 == enum_types::Status::WARNING);
}

TEST_CASE("enum_gapped_values" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::GappedEnum::FIRST) == 0);
    CHECK(static_cast<int32_t>(enum_types::GappedEnum::SECOND) == 5);
    CHECK(static_cast<int32_t>(enum_types::GappedEnum::THIRD) == 10);
    CHECK(static_cast<int32_t>(enum_types::GappedEnum::FOURTH) == 100);

    enum_types::GappedEnum g5 = static_cast<enum_types::GappedEnum>(5);
    CHECK(g5 == enum_types::GappedEnum::SECOND);

    enum_types::GappedEnum g100 = static_cast<enum_types::GappedEnum>(100);
    CHECK(g100 == enum_types::GappedEnum::FOURTH);
}

TEST_CASE("enum_negative_values" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::NegativeEnum::NEG_TWO) == -2);
    CHECK(static_cast<int32_t>(enum_types::NegativeEnum::NEG_ONE) == -1);
    CHECK(static_cast<int32_t>(enum_types::NegativeEnum::ZERO) == 0);
    CHECK(static_cast<int32_t>(enum_types::NegativeEnum::POS_ONE) == 1);

    enum_types::NegativeEnum neg = static_cast<enum_types::NegativeEnum>(-2);
    CHECK(neg == enum_types::NegativeEnum::NEG_TWO);

    enum_types::NegativeEnum zero = static_cast<enum_types::NegativeEnum>(0);
    CHECK(zero == enum_types::NegativeEnum::ZERO);
}

TEST_CASE("enum_const_from_enum_value" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::ENUM_CONST) == 100);
    CHECK(enum_types::ENUM_CONST == enum_types::Status::WARNING);
}

TEST_CASE("enum_mixed_explicit_auto" * doctest::test_suite("enums")) {
    CHECK(static_cast<int32_t>(enum_types::MixedEnum::AUTO_FIRST) == 0);
    CHECK(static_cast<int32_t>(enum_types::MixedEnum::EXPLICIT_TEN) == 10);
    CHECK(static_cast<int32_t>(enum_types::MixedEnum::AUTO_ELEVEN) == 11);
    CHECK(static_cast<int32_t>(enum_types::MixedEnum::EXPLICIT_HUNDRED) == 100);
    CHECK(static_cast<int32_t>(enum_types::MixedEnum::AUTO_HUNDRED_ONE) == 101);

    enum_types::MixedEnum m11 = static_cast<enum_types::MixedEnum>(11);
    CHECK(m11 == enum_types::MixedEnum::AUTO_ELEVEN);

    enum_types::MixedEnum m101 = static_cast<enum_types::MixedEnum>(101);
    CHECK(m101 == enum_types::MixedEnum::AUTO_HUNDRED_ONE);
}
