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

#include "generated/annotations.h"

namespace {

TEST(AnnotationsTest, test_keyed_struct_exists) {
    annotation_types::KeyedStruct ks(1, "test", 3.14);
    EXPECT_EQ(ks.id, 1);
    EXPECT_EQ(ks.name, "test");
    EXPECT_DOUBLE_EQ(ks.value, 3.14);
}

TEST(AnnotationsTest, test_multi_key_struct) {
    annotation_types::MultiKeyStruct mks("namespace1", 42, "data");
    EXPECT_EQ(mks.namespace_, "namespace1");
    EXPECT_EQ(mks.id, 42);
    EXPECT_EQ(mks.data, "data");
}

TEST(AnnotationsTest, test_optional_fields_default_none) {
    annotation_types::OptionalStruct os;
    EXPECT_FALSE(os.optional_int.has_value());
    EXPECT_FALSE(os.optional_string.has_value());
    EXPECT_FALSE(os.optional_seq.has_value());
}

TEST(AnnotationsTest, test_optional_fields_can_be_set) {
    annotation_types::OptionalStruct os;
    os.optional_int = 42;
    os.optional_string = "test";
    os.optional_seq = std::vector<int32_t>{1, 2, 3};

    EXPECT_TRUE(os.optional_int.has_value());
    EXPECT_EQ(os.optional_int.value(), 42);
    EXPECT_TRUE(os.optional_string.has_value());
    EXPECT_EQ(os.optional_string.value(), "test");
    EXPECT_TRUE(os.optional_seq.has_value());
    EXPECT_EQ(os.optional_seq.value().size(), 3u);
}

TEST(AnnotationsTest, test_optional_type_annotations) {
    using OptionalIntType = decltype(annotation_types::OptionalStruct().optional_int);
    using OptionalStringType = decltype(annotation_types::OptionalStruct().optional_string);
    using OptionalSeqType = decltype(annotation_types::OptionalStruct().optional_seq);

    EXPECT_TRUE((std::is_same<OptionalIntType, std::optional<int32_t>>::value));
    EXPECT_TRUE((std::is_same<OptionalStringType, std::optional<std::string>>::value));
    EXPECT_TRUE((std::is_same<OptionalSeqType, std::optional<std::vector<int32_t>>>::value));
}

TEST(AnnotationsTest, test_nested_struct) {
    annotation_types::NestedStruct ns(10, 20);
    EXPECT_EQ(ns.x, 10);
    EXPECT_EQ(ns.y, 20);
}

TEST(AnnotationsTest, test_shared_refs_struct) {
    annotation_types::NestedStruct nested(5, 10);
    annotation_types::SharedRefs sr("shared", nested);
    EXPECT_EQ(sr.shared_string, "shared");
    EXPECT_EQ(sr.shared_struct.x, 5);
    EXPECT_EQ(sr.shared_struct.y, 10);
}

TEST(AnnotationsTest, test_combined_annotations) {
    annotation_types::CombinedAnnotations ca(99, "combined");
    EXPECT_EQ(ca.id, 99);
    EXPECT_TRUE(ca.maybe_shared_name.has_value());
    EXPECT_EQ(ca.maybe_shared_name.value(), "combined");
}

TEST(AnnotationsTest, test_annotated_interface_exists) {
    EXPECT_TRUE(std::is_abstract<annotation_types::AnnotatedInterface>::value);
}

TEST(AnnotationsTest, test_topic_struct) {
    annotation_types::TopicMessage tm(1, "payload", 123456);
    EXPECT_EQ(tm.message_id, 1);
    EXPECT_EQ(tm.payload, "payload");
    EXPECT_EQ(tm.timestamp, 123456);
}

TEST(AnnotationsTest, test_mutable_struct) {
    annotation_types::MutableStruct ms(1, "data");
    EXPECT_EQ(ms.version, 1);
    EXPECT_EQ(ms.data, "data");
}

TEST(AnnotationsTest, test_final_struct) {
    annotation_types::FinalStruct fs(42);
    EXPECT_EQ(fs.fixed_field, 42);
}

TEST(AnnotationsTest, test_optional_assignment) {
    annotation_types::OptionalStruct os;
    os.optional_int = 100;
    EXPECT_TRUE(os.optional_int.has_value());
    EXPECT_EQ(os.optional_int.value(), 100);

    os.optional_int.reset();
    EXPECT_FALSE(os.optional_int.has_value());
}

} // namespace
