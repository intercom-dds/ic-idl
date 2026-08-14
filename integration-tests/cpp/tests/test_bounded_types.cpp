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

#include "bounded_types.h"

TEST_CASE("bounded_string_typedef_maps_to_str" * doctest::test_suite("bounded_types")) {
    bounded_types::ShortString short_str = "Hello";
    CHECK(short_str == "Hello");
    CHECK((std::is_same_v<bounded_types::ShortString, ic_cts::bounded_string<32>>));

    bounded_types::MediumString medium_str = "This is a medium length string";
    CHECK(medium_str.length() == 30);
    CHECK((std::is_same_v<bounded_types::MediumString, ic_cts::bounded_string<256>>));

    bounded_types::LongString long_str =
        "This is a very long string that could contain a lot of text";
    CHECK(long_str.length() > 0);
    CHECK((std::is_same_v<bounded_types::LongString, ic_cts::bounded_string<4096>>));
}

TEST_CASE("bounded_sequence_typedef_maps_to_list" * doctest::test_suite("bounded_types")) {
    bounded_types::SmallIntList small_list = {1, 2, 3, 4, 5};
    CHECK(small_list.size() == 5);
    CHECK((std::is_same_v<bounded_types::SmallIntList, ic_cts::bounded_vector<int32_t, 10>>));

    bounded_types::StringList100 string_list = {"one", "two", "three"};
    CHECK(string_list.size() == 3);
    CHECK((std::is_same_v<bounded_types::StringList100, ic_cts::bounded_vector<std::string, 100>>));

    bounded_types::LargeDoubleList double_list = {1.1, 2.2, 3.3};
    CHECK(double_list.size() == 3);
    CHECK((std::is_same_v<bounded_types::LargeDoubleList, ic_cts::bounded_vector<double, 1000>>));
}

TEST_CASE("bounded_fields_struct" * doctest::test_suite("bounded_types")) {
    bounded_types::BoundedFields bf("test", "description", {1, 2, 3}, {"tag1", "tag2"});
    CHECK(bf.name == "test");
    CHECK(bf.description == "description");
    CHECK(bf.values.size() == 3);
    CHECK(bf.tags.size() == 2);
    CHECK(bf.values[0] == 1);
    CHECK(bf.tags[0] == "tag1");
}

TEST_CASE("bounded_fields_annotations" * doctest::test_suite("bounded_types")) {
    CHECK((std::is_same_v<decltype(bounded_types::BoundedFields::name), ic_cts::bounded_string<64>>)
    );
    CHECK((std::is_same_v<
           decltype(bounded_types::BoundedFields::description),
           ic_cts::bounded_string<256>>));
    CHECK((std::is_same_v<
           decltype(bounded_types::BoundedFields::values),
           ic_cts::bounded_vector<int32_t, 50>>));
    CHECK((std::is_same_v<
           decltype(bounded_types::BoundedFields::tags),
           ic_cts::bounded_vector<ic_cts::bounded_string<32>, 10>>));
}

TEST_CASE("nested_bounded_struct" * doctest::test_suite("bounded_types")) {
    ic_cts::bounded_vector<ic_cts::bounded_vector<int32_t, 10>, 5> matrix = {{1, 2}, {3, 4}};
    std::map<ic_cts::bounded_string<32>, ic_cts::bounded_vector<int32_t, 100>> indexed = {
        {"key", {5, 6}}
    };
    bounded_types::NestedBounded nb(matrix, indexed);
    CHECK(nb.matrix.size() == 2);
    CHECK(nb.matrix[0][1] == 2);
    CHECK(nb.indexed_lists["key"][0] == 5);
}

TEST_CASE("nested_bounded_annotations" * doctest::test_suite("bounded_types")) {
    CHECK((std::is_same_v<
           decltype(bounded_types::NestedBounded::matrix),
           ic_cts::bounded_vector<ic_cts::bounded_vector<int32_t, 10>, 5>>));
    CHECK((std::is_same_v<
           decltype(bounded_types::NestedBounded::indexed_lists),
           std::map<ic_cts::bounded_string<32>, ic_cts::bounded_vector<int32_t, 100>>>));
}

TEST_CASE("typedef_chain_with_bounds" * doctest::test_suite("bounded_types")) {
    bounded_types::Name name = "Alice";
    CHECK(name == "Alice");
    CHECK((std::is_same_v<bounded_types::Name, ic_cts::bounded_string<100>>));

    bounded_types::NameList names = {"Alice", "Bob", "Charlie"};
    CHECK(names.size() == 3);
    CHECK((std::is_same_v<bounded_types::NameList, ic_cts::bounded_vector<bounded_types::Name, 50>>)
    );

    bounded_types::NameMap name_map = {{"group1", {"Alice", "Bob"}}};
    CHECK(name_map.size() == 1);
    CHECK((std::is_same_v<
           bounded_types::NameMap,
           std::map<bounded_types::Name, bounded_types::NameList>>));
}

TEST_CASE("mixed_bounds_struct" * doctest::test_suite("bounded_types")) {
    bounded_types::MixedBounds mb("bounded", "unbounded", {1, 2}, {3, 4, 5});
    CHECK(mb.bounded_string == "bounded");
    CHECK(mb.unbounded_string == "unbounded");
    CHECK(mb.bounded_seq.size() == 2);
    CHECK(mb.unbounded_seq.size() == 3);
}

TEST_CASE("mixed_bounds_annotations" * doctest::test_suite("bounded_types")) {
    CHECK((std::is_same_v<
           decltype(bounded_types::MixedBounds::bounded_string),
           ic_cts::bounded_string<100>>));
    CHECK((std::is_same_v<decltype(bounded_types::MixedBounds::unbounded_string), std::string>));
    CHECK((std::is_same_v<
           decltype(bounded_types::MixedBounds::bounded_seq),
           ic_cts::bounded_vector<int32_t, 10>>));
    CHECK(
        (std::is_same_v<decltype(bounded_types::MixedBounds::unbounded_seq), std::vector<int32_t>>)
    );
}

TEST_CASE("bounds_not_enforced_at_runtime" * doctest::test_suite("bounded_types")) {
    bounded_types::ShortString str =
        "This string is longer than 32 characters and should not be truncated at runtime";
    CHECK(str.length() > 32);

    bounded_types::SmallIntList list;
    for (int i = 0; i < 20; i++) {
        list.push_back(i);
    }
    CHECK(list.size() == 20);
    CHECK(list.size() > 10);
}
