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

#include <type_traits>

#include "generated/bounded_types.h"

namespace {

TEST(BoundedTypesTest, test_bounded_string_typedef_maps_to_str) {
    bounded_types::ShortString short_str = "Hello";
    EXPECT_EQ(short_str, "Hello");
    EXPECT_TRUE((std::is_same_v<bounded_types::ShortString, std::string>));

    bounded_types::MediumString medium_str = "This is a medium length string";
    EXPECT_EQ(medium_str.length(), 30);
    EXPECT_TRUE((std::is_same_v<bounded_types::MediumString, std::string>));

    bounded_types::LongString long_str =
        "This is a very long string that could contain a lot of text";
    EXPECT_GT(long_str.length(), 0);
    EXPECT_TRUE((std::is_same_v<bounded_types::LongString, std::string>));
}

TEST(BoundedTypesTest, test_bounded_sequence_typedef_maps_to_list) {
    bounded_types::SmallIntList small_list = {1, 2, 3, 4, 5};
    EXPECT_EQ(small_list.size(), 5);
    EXPECT_TRUE((std::is_same_v<bounded_types::SmallIntList, std::vector<int32_t>>));

    bounded_types::StringList100 string_list = {"one", "two", "three"};
    EXPECT_EQ(string_list.size(), 3);
    EXPECT_TRUE((std::is_same_v<bounded_types::StringList100, std::vector<std::string>>));

    bounded_types::LargeDoubleList double_list = {1.1, 2.2, 3.3};
    EXPECT_EQ(double_list.size(), 3);
    EXPECT_TRUE((std::is_same_v<bounded_types::LargeDoubleList, std::vector<double>>));
}

TEST(BoundedTypesTest, test_bounded_fields_struct) {
    bounded_types::BoundedFields bf("test", "description", {1, 2, 3}, {"tag1", "tag2"});
    EXPECT_EQ(bf.name, "test");
    EXPECT_EQ(bf.description, "description");
    EXPECT_EQ(bf.values.size(), 3);
    EXPECT_EQ(bf.tags.size(), 2);
    EXPECT_EQ(bf.values[0], 1);
    EXPECT_EQ(bf.tags[0], "tag1");
}

TEST(BoundedTypesTest, test_bounded_fields_annotations) {
    EXPECT_TRUE((std::is_same_v<decltype(bounded_types::BoundedFields::name), std::string>));
    EXPECT_TRUE((std::is_same_v<decltype(bounded_types::BoundedFields::description), std::string>));
    EXPECT_TRUE(
        (std::is_same_v<decltype(bounded_types::BoundedFields::values), std::vector<int32_t>>)
    );
    EXPECT_TRUE(
        (std::is_same_v<decltype(bounded_types::BoundedFields::tags), std::vector<std::string>>)
    );
}

TEST(BoundedTypesTest, test_nested_bounded_struct) {
    std::vector<std::vector<int32_t>> matrix = {{1, 2}, {3, 4}};
    std::map<std::string, std::vector<int32_t>> indexed = {{"key", {5, 6}}};
    bounded_types::NestedBounded nb(matrix, indexed);
    EXPECT_EQ(nb.matrix.size(), 2);
    EXPECT_EQ(nb.matrix[0][1], 2);
    EXPECT_EQ(nb.indexed_lists["key"][0], 5);
}

TEST(BoundedTypesTest, test_nested_bounded_annotations) {
    EXPECT_TRUE((std::is_same_v<
                 decltype(bounded_types::NestedBounded::matrix),
                 std::vector<std::vector<int32_t>>>));
    EXPECT_TRUE((std::is_same_v<
                 decltype(bounded_types::NestedBounded::indexed_lists),
                 std::map<std::string, std::vector<int32_t>>>));
}

TEST(BoundedTypesTest, test_typedef_chain_with_bounds) {
    bounded_types::Name name = "Alice";
    EXPECT_EQ(name, "Alice");
    EXPECT_TRUE((std::is_same_v<bounded_types::Name, std::string>));

    bounded_types::NameList names = {"Alice", "Bob", "Charlie"};
    EXPECT_EQ(names.size(), 3);
    EXPECT_TRUE((std::is_same_v<bounded_types::NameList, std::vector<bounded_types::Name>>));

    bounded_types::NameMap name_map = {{"group1", {"Alice", "Bob"}}};
    EXPECT_EQ(name_map.size(), 1);
    EXPECT_TRUE((std::is_same_v<
                 bounded_types::NameMap,
                 std::map<bounded_types::Name, bounded_types::NameList>>));
}

TEST(BoundedTypesTest, test_mixed_bounds_struct) {
    bounded_types::MixedBounds mb("bounded", "unbounded", {1, 2}, {3, 4, 5});
    EXPECT_EQ(mb.bounded_string, "bounded");
    EXPECT_EQ(mb.unbounded_string, "unbounded");
    EXPECT_EQ(mb.bounded_seq.size(), 2);
    EXPECT_EQ(mb.unbounded_seq.size(), 3);
}

TEST(BoundedTypesTest, test_mixed_bounds_annotations) {
    EXPECT_TRUE((std::is_same_v<decltype(bounded_types::MixedBounds::bounded_string), std::string>)
    );
    EXPECT_TRUE(
        (std::is_same_v<decltype(bounded_types::MixedBounds::unbounded_string), std::string>)
    );
    EXPECT_TRUE(
        (std::is_same_v<decltype(bounded_types::MixedBounds::bounded_seq), std::vector<int32_t>>)
    );
    EXPECT_TRUE(
        (std::is_same_v<decltype(bounded_types::MixedBounds::unbounded_seq), std::vector<int32_t>>)
    );
}

TEST(BoundedTypesTest, test_bounds_not_enforced_at_runtime) {
    bounded_types::ShortString str =
        "This string is longer than 32 characters and should not be truncated at runtime";
    EXPECT_GT(str.length(), 32);

    bounded_types::SmallIntList list;
    for (int i = 0; i < 20; i++) {
        list.push_back(i);
    }
    EXPECT_EQ(list.size(), 20);
    EXPECT_GT(list.size(), 10);
}

} // namespace
