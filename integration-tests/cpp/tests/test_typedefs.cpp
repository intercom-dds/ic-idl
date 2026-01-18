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

#include "generated/typedefs.h"

namespace {

TEST(TypedefsTest, test_primitive_typedef_values) {
    typedef_types::Integer i = 42;
    EXPECT_EQ(i, 42);

    typedef_types::UnsignedInteger ui = 100U;
    EXPECT_EQ(ui, 100U);

    typedef_types::Real r = 3.14;
    EXPECT_DOUBLE_EQ(r, 3.14);

    typedef_types::Text t = "hello";
    EXPECT_EQ(t, "hello");

    typedef_types::Flag f = true;
    EXPECT_TRUE(f);

    typedef_types::Byte b = 255;
    EXPECT_EQ(b, 255);
}

TEST(TypedefsTest, test_sequence_typedef_values) {
    typedef_types::IntList il = {1, 2, 3, 4, 5};
    EXPECT_EQ(il.size(), 5);
    EXPECT_EQ(il[0], 1);
    EXPECT_EQ(il[4], 5);

    typedef_types::StringList sl = {"one", "two", "three"};
    EXPECT_EQ(sl.size(), 3);
    EXPECT_EQ(sl[0], "one");
    EXPECT_EQ(sl[2], "three");

    typedef_types::RealList rl = {1.1, 2.2, 3.3};
    EXPECT_EQ(rl.size(), 3);
    EXPECT_DOUBLE_EQ(rl[0], 1.1);
    EXPECT_DOUBLE_EQ(rl[2], 3.3);
}

TEST(TypedefsTest, test_nested_typedef_values) {
    typedef_types::Count c = 42;
    EXPECT_EQ(c, 42);

    typedef_types::Label l = "test_label";
    EXPECT_EQ(l, "test_label");
}

TEST(TypedefsTest, test_map_typedef_values) {
    typedef_types::StringIntMap sim = {{"one", 1}, {"two", 2}};
    EXPECT_EQ(sim.size(), 2);
    EXPECT_EQ(sim["one"], 1);
    EXPECT_EQ(sim["two"], 2);

    typedef_types::StringStringMap ssm = {{"key1", "value1"}, {"key2", "value2"}};
    EXPECT_EQ(ssm.size(), 2);
    EXPECT_EQ(ssm["key1"], "value1");
}

TEST(TypedefsTest, test_array_typedef_value) {
    typedef_types::LongArray la = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    EXPECT_EQ(la.size(), 10);
    EXPECT_EQ(la[0], 1);
    EXPECT_EQ(la[9], 10);
}

TEST(TypedefsTest, test_struct_with_typedef_fields) {
    typedef_types::Point p(10.5, 20.5);
    EXPECT_DOUBLE_EQ(p.x, 10.5);
    EXPECT_DOUBLE_EQ(p.y, 20.5);
}

TEST(TypedefsTest, test_struct_with_typedef_field_types) {
    static_assert(
        std::is_same<decltype(typedef_types::Point::x), typedef_types::Real>::value,
        "x should be Real"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Point::y), typedef_types::Real>::value,
        "y should be Real"
    );
}

TEST(TypedefsTest, test_person_struct_field_types) {
    static_assert(
        std::is_same<decltype(typedef_types::Person::name), typedef_types::Text>::value,
        "name should be Text"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Person::age), typedef_types::Integer>::value,
        "age should be Integer"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Person::active), typedef_types::Flag>::value,
        "active should be Flag"
    );
}

TEST(TypedefsTest, test_person_struct_values) {
    typedef_types::Person person("Alice", 30, true);
    EXPECT_EQ(person.name, "Alice");
    EXPECT_EQ(person.age, 30);
    EXPECT_TRUE(person.active);
}

TEST(TypedefsTest, test_container_struct_field_types) {
    static_assert(
        std::is_same<decltype(typedef_types::Container::numbers), typedef_types::IntList>::value,
        "numbers should be IntList"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Container::labels), typedef_types::StringList>::value,
        "labels should be StringList"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Container::lookup), typedef_types::StringIntMap>::
            value,
        "lookup should be StringIntMap"
    );
}

TEST(TypedefsTest, test_container_struct_values) {
    typedef_types::IntList nums = {1, 2, 3};
    typedef_types::StringList labs = {"a", "b", "c"};
    typedef_types::StringIntMap lup = {{"x", 10}, {"y", 20}};
    typedef_types::Container container(nums, labs, lup);

    EXPECT_EQ(container.numbers.size(), 3);
    EXPECT_EQ(container.labels.size(), 3);
    EXPECT_EQ(container.lookup.size(), 2);
    EXPECT_EQ(container.numbers[0], 1);
    EXPECT_EQ(container.labels[1], "b");
    EXPECT_EQ(container.lookup["x"], 10);
}

TEST(TypedefsTest, test_nested_typedef_in_struct) {
    static_assert(
        std::is_same<decltype(typedef_types::Measurement::name), typedef_types::Label>::value,
        "name should be Label"
    );
    static_assert(
        std::is_same<decltype(typedef_types::Measurement::value), typedef_types::Count>::value,
        "value should be Count"
    );
}

TEST(TypedefsTest, test_nested_typedef_struct_values) {
    typedef_types::Measurement m("temperature", 42);
    EXPECT_EQ(m.name, "temperature");
    EXPECT_EQ(m.value, 42);
}

TEST(TypedefsTest, test_array_typedef_in_struct) {
    static_assert(
        std::is_same<decltype(typedef_types::WithArrayTypedef::values), typedef_types::LongArray>::
            value,
        "values should be LongArray"
    );
}

TEST(TypedefsTest, test_array_typedef_struct_values) {
    typedef_types::LongArray arr = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    typedef_types::WithArrayTypedef wat(arr);
    EXPECT_EQ(wat.values.size(), 10);
    EXPECT_EQ(wat.values[0], 1);
    EXPECT_EQ(wat.values[9], 10);
}

TEST(TypedefsTest, test_deep_typedef_chain_values) {
    typedef_types::Level1 l1 = 100;
    typedef_types::Level2 l2 = 100;
    typedef_types::Level3 l3 = 100;
    typedef_types::Level4 l4 = 100;
    typedef_types::Level5 l5 = 100;

    EXPECT_EQ(l1, 100);
    EXPECT_EQ(l2, 100);
    EXPECT_EQ(l3, 100);
    EXPECT_EQ(l4, 100);
    EXPECT_EQ(l5, 100);
}

TEST(TypedefsTest, test_deep_sequence_typedef_chain) {
    typedef_types::SeqLevel1 sl1 = {1, 2, 3};
    typedef_types::SeqLevel2 sl2 = {4, 5, 6};
    typedef_types::SeqLevel3 sl3 = {7, 8, 9};

    EXPECT_EQ(sl1.size(), 3);
    EXPECT_EQ(sl2.size(), 3);
    EXPECT_EQ(sl3.size(), 3);
}

TEST(TypedefsTest, test_deep_map_typedef_chain) {
    typedef_types::MapLevel1 ml1 = {{"a", 1}};
    typedef_types::MapLevel2 ml2 = {{"b", 2}};
    typedef_types::MapLevel3 ml3 = {{"c", 3}};

    EXPECT_EQ(ml1.size(), 1);
    EXPECT_EQ(ml2.size(), 1);
    EXPECT_EQ(ml3.size(), 1);
    EXPECT_EQ(ml1["a"], 1);
    EXPECT_EQ(ml2["b"], 2);
    EXPECT_EQ(ml3["c"], 3);
}

TEST(TypedefsTest, test_deep_chain_struct_field_types) {
    static_assert(
        std::is_same<decltype(typedef_types::DeepChainStruct::deep_int), typedef_types::Level5>::
            value,
        "deep_int should be Level5"
    );
    static_assert(
        std::is_same<decltype(typedef_types::DeepChainStruct::deep_seq), typedef_types::SeqLevel3>::
            value,
        "deep_seq should be SeqLevel3"
    );
    static_assert(
        std::is_same<decltype(typedef_types::DeepChainStruct::deep_map), typedef_types::MapLevel3>::
            value,
        "deep_map should be MapLevel3"
    );
}

TEST(TypedefsTest, test_deep_chain_struct_values) {
    typedef_types::Level5 di = 999;
    typedef_types::SeqLevel3 ds = {1, 2, 3, 4, 5};
    typedef_types::MapLevel3 dm = {{"key1", 100}, {"key2", 200}};
    typedef_types::DeepChainStruct dcs(di, ds, dm);

    EXPECT_EQ(dcs.deep_int, 999);
    EXPECT_EQ(dcs.deep_seq.size(), 5);
    EXPECT_EQ(dcs.deep_map.size(), 2);
    EXPECT_EQ(dcs.deep_seq[0], 1);
    EXPECT_EQ(dcs.deep_map["key1"], 100);
}

TEST(TypedefsTest, test_typedef_type_compatibility) {
    typedef_types::Integer i = 42;
    typedef_types::Count c = i;
    EXPECT_EQ(c, 42);

    typedef_types::Text t = "hello";
    typedef_types::Label l = t;
    EXPECT_EQ(l, "hello");
}

} // namespace
