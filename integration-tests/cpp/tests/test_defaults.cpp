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

#include "defaults.h"

TEST_CASE("const_string_values" * doctest::test_suite("defaults")) {
    CHECK(default_types::DEFAULT_NAME == "unnamed");
    CHECK(default_types::DEFAULT_COUNT == 100);
    CHECK(default_types::DEFAULT_RATE == doctest::Approx(0.5));
}

TEST_CASE("struct_const_initializer" * doctest::test_suite("defaults")) {
    CHECK(default_types::DEFAULT_INNER.x == 10);
    CHECK(default_types::DEFAULT_INNER.y == "default");
    CHECK(default_types::NESTED_INNER.x == 99);
    CHECK(default_types::NESTED_INNER.y == "nested");
}

TEST_CASE("optional_fields_are_none" * doctest::test_suite("defaults")) {
    default_types::OptionalFields opt;
    CHECK_FALSE(opt.maybe_int.has_value());
    CHECK_FALSE(opt.maybe_string.has_value());
    CHECK_FALSE(opt.maybe_struct.has_value());
}

TEST_CASE("optional_fields_type_annotations" * doctest::test_suite("defaults")) {
    default_types::OptionalFields opt;
    static_assert(
        std::is_same<decltype(opt.maybe_int), std::optional<int32_t>>::value,
        "maybe_int should be std::optional<int32_t>"
    );
    static_assert(
        std::is_same<decltype(opt.maybe_string), std::optional<std::string>>::value,
        "maybe_string should be std::optional<std::string>"
    );
    static_assert(
        std::is_same<decltype(opt.maybe_struct), std::optional<default_types::Inner>>::value,
        "maybe_struct should be std::optional<Inner>"
    );
}

TEST_CASE("optional_fields_can_be_set" * doctest::test_suite("defaults")) {
    default_types::OptionalFields opt;
    opt.maybe_int = 42;
    opt.maybe_string = "test";
    opt.maybe_struct = default_types::Inner(10, "hello");
    CHECK(opt.maybe_int.has_value());
    CHECK(opt.maybe_int.value() == 42);
    CHECK(opt.maybe_string.has_value());
    CHECK(opt.maybe_string.value() == "test");
    CHECK(opt.maybe_struct.has_value());
    CHECK(opt.maybe_struct.value().x == 10);
    CHECK(opt.maybe_struct.value().y == "hello");
}

TEST_CASE("enum_default_literal_exists" * doctest::test_suite("defaults")) {
    CHECK(static_cast<int32_t>(default_types::Priority::LOW) == 0);
    CHECK(static_cast<int32_t>(default_types::Priority::MEDIUM) == 1);
    CHECK(static_cast<int32_t>(default_types::Priority::HIGH) == 2);
}

TEST_CASE("enum_default_literal_value" * doctest::test_suite("defaults")) {
    CHECK(default_types::EnumDefaults().priority_empty == default_types::Priority::MEDIUM);
}

TEST_CASE("primitive_bool_default" * doctest::test_suite("defaults")) {
    default_types::PrimitiveDefaults p;
    CHECK(p.bool_empty == false);
    CHECK(p.bool_true == true);
    CHECK(p.bool_false == false);
}

TEST_CASE("primitive_int_default" * doctest::test_suite("defaults")) {
    default_types::PrimitiveDefaults p;
    CHECK(p.int_empty == 0);
    CHECK(p.int_value == 42);
    CHECK(p.int_negative == -100);
}

TEST_CASE("primitive_float_default" * doctest::test_suite("defaults")) {
    default_types::PrimitiveDefaults p;
    CHECK(p.float_empty == doctest::Approx(0.0));
    CHECK(p.float_value == doctest::Approx(3.14159).epsilon(0.00001));
    CHECK(p.float_negative == doctest::Approx(-0.5));
}

TEST_CASE("primitive_string_default" * doctest::test_suite("defaults")) {
    default_types::PrimitiveDefaults p;
    CHECK(p.string_empty == "");
    CHECK(p.string_value == "hello");
    CHECK(p.string_from_const == "unnamed");
}

TEST_CASE("array_default_values" * doctest::test_suite("defaults")) {
    default_types::ArrayDefaults a;
    CHECK(a.array_empty.size() == 3U);
    CHECK(a.array_empty[0] == 0);
    CHECK(a.array_empty[1] == 0);
    CHECK(a.array_empty[2] == 0);
    CHECK(a.array_values.size() == 3U);
    CHECK(a.array_values[0] == 1);
    CHECK(a.array_values[1] == 2);
    CHECK(a.array_values[2] == 3);
    CHECK(a.array_partial.size() == 2U);
    CHECK(a.array_partial[0] == 10);
    CHECK(a.array_partial[1] == 20);
    CHECK(a.array_external->size() == 3U);
    const auto& array_external{*a.array_external};
    CHECK(array_external[0] == 1);
    CHECK(array_external[1] == 2);
    CHECK(array_external[2] == 3);
    CHECK(a.string_array_empty.size() == 2U);
    CHECK(a.string_array_empty[0] == "");
    CHECK(a.string_array_empty[1] == "");
    CHECK(a.string_array_values.size() == 2U);
    CHECK(a.string_array_values[0] == "foo");
    CHECK(a.string_array_values[1] == "bar");
}

TEST_CASE("sequence_default_values" * doctest::test_suite("defaults")) {
    default_types::SequenceDefaults s;
    CHECK(s.seq_empty.size() == 0U);
    CHECK(s.seq_values.size() == 5U);
    CHECK(s.seq_values[0] == 1);
    CHECK(s.seq_values[1] == 2);
    CHECK(s.seq_values[2] == 3);
    CHECK(s.seq_values[3] == 4);
    CHECK(s.seq_values[4] == 5);
    CHECK(s.string_seq_empty.size() == 0U);
    CHECK(s.string_seq_values.size() == 3U);
    CHECK(s.string_seq_values[0] == "a");
    CHECK(s.string_seq_values[1] == "b");
    CHECK(s.string_seq_values[2] == "c");
}

TEST_CASE("map_default_values" * doctest::test_suite("defaults")) {
    default_types::MapDefaults m;
    CHECK(m.map_empty.size() == 0U);
    CHECK(m.map_values.size() == 2U);
    CHECK(m.map_values.at("one") == 1);
    CHECK(m.map_values.at("two") == 2);
    CHECK(m.reverse_map_empty.size() == 0U);
    CHECK(m.reverse_map_values.size() == 2U);
    CHECK(m.reverse_map_values.at(1) == "one");
    CHECK(m.reverse_map_values.at(2) == "two");
}

TEST_CASE("enum_field_default" * doctest::test_suite("defaults")) {
    default_types::EnumDefaults e;
    CHECK(e.priority_high == default_types::Priority::HIGH);
}

TEST_CASE("complex_defaults" * doctest::test_suite("defaults")) {
    default_types::ComplexDefaults c;
    CHECK(c.nested_seq.size() == 2U);
    CHECK(c.nested_seq[0].size() == 2U);
    CHECK(c.nested_seq[0][0] == 1);
    CHECK(c.nested_seq[0][1] == 2);
    CHECK(c.nested_seq[1].size() == 2U);
    CHECK(c.nested_seq[1][0] == 3);
    CHECK(c.nested_seq[1][1] == 4);

    CHECK(c.map_of_seq.size() == 2U);
    CHECK(c.map_of_seq.at("a").size() == 2U);
    CHECK(c.map_of_seq.at("a")[0] == 1);
    CHECK(c.map_of_seq.at("a")[1] == 2);
    CHECK(c.map_of_seq.at("b").size() == 2U);
    CHECK(c.map_of_seq.at("b")[0] == 3);
    CHECK(c.map_of_seq.at("b")[1] == 4);

    CHECK(c.ranged_external_field != nullptr);
    CHECK(c.ranged_external_field->value() == 1);
}