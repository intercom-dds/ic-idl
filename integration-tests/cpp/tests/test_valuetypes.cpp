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

#include "valuetypes.h"

TEST_CASE("valuetype_instantiation" * doctest::test_suite("valuetypes")) {
    valuetype_types::SimpleValue sv(42, "test");
    CHECK(sv.id == 42);
    CHECK(sv.name == "test");
}

TEST_CASE("valuetype_defaults" * doctest::test_suite("valuetypes")) {
    valuetype_types::SimpleValue sv;
    CHECK(sv.id == 0);
    CHECK(sv.name == "");
}

TEST_CASE("valuetype_inheritance" * doctest::test_suite("valuetypes")) {
    valuetype_types::DerivedValue dv(1, "base", "derived");
    CHECK(dv.id == 1);
    CHECK(dv.name == "base");
    CHECK(dv.description == "derived");

    valuetype_types::SimpleValue* sv = &dv;
    CHECK(sv->id == 1);
    CHECK(sv->name == "base");
}

TEST_CASE("valuetype_empty" * doctest::test_suite("valuetypes")) {
    valuetype_types::Empty e;
    CHECK(e == e);
}

TEST_CASE("valuetype_with_attribute" * doctest::test_suite("valuetypes")) {
    CHECK((std::is_same<decltype(valuetype_types::WithAttribute::name), std::string>::value));
    CHECK((std::is_same<decltype(valuetype_types::WithAttribute::count), int32_t>::value));
    CHECK((std::is_same<decltype(valuetype_types::WithAttribute::iface), valuetype_types::Named*>::value));

    valuetype_types::WithAttribute wa;
    CHECK(wa.name == "");
    CHECK(wa.count == 0);
    CHECK(wa.iface == nullptr);
}

TEST_CASE("valuetype_with_sequence" * doctest::test_suite("valuetypes")) {
    std::vector<int32_t> nums = {1, 2, 3, 4, 5};
    std::vector<std::string> names = {"a", "b", "c"};
    valuetype_types::WithSequence ws(nums, names);

    CHECK(ws.numbers.size() == 5);
    CHECK(ws.names.size() == 3);
    CHECK(ws.numbers[0] == 1);
    CHECK(ws.names[1] == "b");
}

TEST_CASE("valuetype_equality" * doctest::test_suite("valuetypes")) {
    valuetype_types::SimpleValue v1(10, "test");
    valuetype_types::SimpleValue v2(10, "test");
    valuetype_types::SimpleValue v3(20, "other");

    CHECK(v1 == v2);
    CHECK(v1 != v3);
}

TEST_CASE("valuetype_supports_interface" * doctest::test_suite("valuetypes")) {
    valuetype_types::IdentifiableValue iv(123, "data");
    CHECK(iv.id == 123);
    CHECK(iv.data == "data");
}

TEST_CASE("valuetype_supports_named" * doctest::test_suite("valuetypes")) {
    valuetype_types::NamedValue nv("test_name", 456);
    CHECK(nv.name == "test_name");
    CHECK(nv.value == 456);
}

TEST_CASE("valuetype_inheritance_and_supports" * doctest::test_suite("valuetypes")) {
    valuetype_types::FullValue fv(1, "name", "extra");
    CHECK(fv.id == 1);
    CHECK(fv.name == "name");
    CHECK(fv.extra == "extra");

    valuetype_types::SimpleValue* sv = &fv;
    CHECK(sv->id == 1);
    CHECK(sv->name == "name");
}

TEST_CASE("valuetype_field_types" * doctest::test_suite("valuetypes")) {
    static_assert(
        std::is_same<decltype(valuetype_types::SimpleValue::id), int32_t>::value,
        "id should be int32_t"
    );
    static_assert(
        std::is_same<decltype(valuetype_types::SimpleValue::name), std::string>::value,
        "name should be std::string"
    );
}

TEST_CASE("valuetype_sequence_field_types" * doctest::test_suite("valuetypes")) {
    static_assert(
        std::is_same<decltype(valuetype_types::WithSequence::numbers), std::vector<int32_t>>::value,
        "numbers should be std::vector<int32_t>"
    );
    static_assert(
        std::is_same<decltype(valuetype_types::WithSequence::names), std::vector<std::string>>::
            value,
        "names should be std::vector<std::string>"
    );
}

TEST_CASE("valuetype_derived_field_types" * doctest::test_suite("valuetypes")) {
    static_assert(
        std::is_same<decltype(valuetype_types::DerivedValue::description), std::string>::value,
        "description should be std::string"
    );
    static_assert(
        std::is_base_of<valuetype_types::SimpleValue, valuetype_types::DerivedValue>::value,
        "DerivedValue should inherit from SimpleValue"
    );
}

TEST_CASE("valuetype_comparison_operators" * doctest::test_suite("valuetypes")) {
    valuetype_types::SimpleValue v1(10, "test");
    valuetype_types::SimpleValue v2(10, "test");
    valuetype_types::SimpleValue v3(5, "other");
    valuetype_types::SimpleValue v4(10, "zzz");

    CHECK(v1 == v2);
    CHECK_FALSE(v1 == v3);
    CHECK(v1 != v3);
    CHECK(v3 < v1);
    CHECK(v1 > v3);
    CHECK(v1 < v4);
    CHECK(v1 <= v2);
    CHECK(v1 >= v2);
}

TEST_CASE("valuetype_type_traits" * doctest::test_suite("valuetypes")) {
    CHECK(std::has_virtual_destructor<valuetype_types::Empty>::value);
    CHECK(!std::is_abstract<valuetype_types::Empty>::value);
}
