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

#include "unions.h"

TEST_CASE("union_int_variant" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.int_val(42);
    REQUIRE(u._d() == 1);
    REQUIRE(u.int_val() == 42);
}

TEST_CASE("union_string_variant" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.str_val("hello");
    REQUIRE(u._d() == 2);
    REQUIRE(u.str_val() == "hello");
}

TEST_CASE("union_wrong_variant_raises" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.int_val(42);
    REQUIRE_THROWS_AS(std::ignore = u.str_val(), std::logic_error);

    u.str_val("test");
    REQUIRE_THROWS_AS(std::ignore = u.int_val(), std::logic_error);
}

TEST_CASE("union_enum_discriminator" * doctest::test_suite("unions")) {
    union_types::TypedValue tv;
    REQUIRE((std::is_same_v<decltype(tv._d()), union_types::ValueKind>));
}

TEST_CASE("union_enum_string_variant" * doctest::test_suite("unions")) {
    union_types::TypedValue tv;
    tv.string_value("test string");
    REQUIRE(tv._d() == union_types::ValueKind::STRING_KIND);
    REQUIRE(tv.string_value() == "test string");
}

TEST_CASE("union_bool_discriminator" * doctest::test_suite("unions")) {
    union_types::BoolSwitch bs;
    bs.true_val(100);
    REQUIRE(bs._d());
    REQUIRE(bs.true_val() == 100);

    bs.false_val("false branch");
    REQUIRE_FALSE(bs._d());
    REQUIRE(bs.false_val() == "false branch");
}

TEST_CASE("union_multi_case" * doctest::test_suite("unions")) {
    union_types::MultiCase mc;
    mc.small_val(5);
    REQUIRE(mc._d() == 1);

    mc._d(2);
    REQUIRE(mc._d() == 2);
    REQUIRE(mc.small_val() == 5);
    mc.small_val(10);
    REQUIRE(mc._d() == 2);
    REQUIRE(mc.small_val() == 10);

    mc._d(3);
    REQUIRE(mc._d() == 3);
    REQUIRE(mc.small_val() == 10);

    mc._d(10);
    mc.text_val("test");
    REQUIRE((mc._d() == 10 || mc._d() == 20));
    REQUIRE(mc.text_val() == "test");
}

TEST_CASE("union_default_method" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.str_val("hello");
    u.default_val(true);
    REQUIRE(u._d() != 1);
    REQUIRE(u._d() != 2);
    REQUIRE(u.default_val());
}

TEST_CASE("union_discriminator_property" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.int_val(42);
    REQUIRE(u._d() == 1);

    u.str_val("test");
    REQUIRE(u._d() == 2);

    union_types::TypedValue tv;
    tv.int_value(100);
    REQUIRE(tv._d() == union_types::ValueKind::INT_KIND);
}

TEST_CASE("union_equality" * doctest::test_suite("unions")) {
    union_types::IntOrString u1;
    u1.int_val(42);

    union_types::IntOrString u2;
    u2.int_val(42);

    union_types::IntOrString u3;
    u3.int_val(99);

    REQUIRE(u1 == u2);
    REQUIRE_FALSE(u1 == u3);
    REQUIRE_FALSE(u1 != u2);
    REQUIRE(u1 != u3);
}

TEST_CASE("union_default_constructor_uses_default_discriminator_case" * doctest::test_suite("unions")) {
    union_types::DefaultDiscriminatorCase u;
    REQUIRE(u._d() == 0);
    REQUIRE(u.value() == 0);
}

TEST_CASE("union_default_constructor_with_default_case" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    REQUIRE(u._d() == 0);
    REQUIRE_FALSE(u.default_val());
}

TEST_CASE("union_default_constructor_without_default_case" * doctest::test_suite("unions")) {
    union_types::MultiCase mc;
    REQUIRE(mc._d() == 0);
    REQUIRE_FALSE(mc.flag());
}

TEST_CASE("union_default_constructor_enum_discriminator" * doctest::test_suite("unions")) {
    union_types::TypedValue tv;
    REQUIRE(tv._d() == union_types::ValueKind::INT_KIND);
    REQUIRE(tv.int_value() == 0);
}

TEST_CASE("union_default_variant_sets_discriminator" * doctest::test_suite("unions")) {
    union_types::IntOrString u;
    u.int_val(42);
    REQUIRE(u._d() == 1);

    u.default_val(true);
    REQUIRE(u._d() == 0);
    REQUIRE(u.default_val());
}

TEST_CASE("union_swap" * doctest::test_suite("unions")) {
    union_types::IntOrString u1;
    u1.int_val(42);

    union_types::IntOrString u2;
    u2.str_val("hello");

    using std::swap;
    swap(u1, u2);

    REQUIRE(u1._d() == 2);
    REQUIRE(u1.str_val() == "hello");
    REQUIRE(u2._d() == 1);
    REQUIRE(u2.int_val() == 42);
}

TEST_CASE("union_swap_same_discriminator" * doctest::test_suite("unions")) {
    union_types::TypedValue tv1;
    tv1.int_value(100);

    union_types::TypedValue tv2;
    tv2.int_value(200);

    using std::swap;
    swap(tv1, tv2);

    REQUIRE(tv1._d() == union_types::ValueKind::INT_KIND);
    REQUIRE(tv1.int_value() == 200);
    REQUIRE(tv2._d() == union_types::ValueKind::INT_KIND);
    REQUIRE(tv2.int_value() == 100);
}

TEST_CASE("union_implicit_primitive_discriminator_default" * doctest::test_suite("unions")) {
    union_types::ImplicitPrimitiveDisc u;
    REQUIRE(u._d() == 0);

    u.long_val(123);
    REQUIRE(u._d() != 0);

    u._default();
    REQUIRE(u._d() == 0);
}

TEST_CASE("union_implicit_enum_discriminator_default" * doctest::test_suite("unions")) {
    union_types::ImplicitEnumDisc u;
    REQUIRE(u._d() == union_types::ValueKind::INT_KIND);
    REQUIRE(u.int_val() == 0);

    u.int_val(123);
    REQUIRE(u._d() == union_types::ValueKind::INT_KIND);

    u._default();
    REQUIRE(u._d() == union_types::ValueKind::STRING_KIND);
}

TEST_CASE("union_default_case_external_variant" * doctest::test_suite("unions")) {
    union_types::DefaultCaseExternalVariant u;
    REQUIRE(u._d() == 0);

    auto ptr = std::make_unique<bool>(true);
    u.other(std::move(ptr), 10);
    REQUIRE(u._d() == 10);
    REQUIRE(*u.other().get() == true);
}