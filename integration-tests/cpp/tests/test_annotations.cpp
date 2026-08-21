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

#include "annotations.h"

TEST_CASE("effective_member_ids" * doctest::test_suite("annotations")) {
    const auto& struct_info = ic_cts::TypeTraits<autoid_hash_types::ModuleHash>::type_info;
    REQUIRE(struct_info.member_count == 4);
    CHECK(struct_info.members[0].id == 96462948);
    CHECK(struct_info.members[1].id == 37920031);
    CHECK(struct_info.members[2].id == 42);
    CHECK(struct_info.members[3].id == 57943011);

    const auto& union_info = ic_cts::TypeTraits<autoid_hash_types::HashUnion>::type_info;
    REQUIRE(union_info.member_count == 3);
    CHECK(union_info.members[0].id == 0);
    CHECK(union_info.members[1].id == 239892167);
    CHECK(union_info.members[2].id == 256044424);
}

TEST_CASE("keyed_struct_exists" * doctest::test_suite("annotations")) {
    annotation_types::KeyedStruct ks(1, "test", 3.14);
    CHECK(ks.id == 1);
    CHECK(ks.name == "test");
    CHECK(ks.value == doctest::Approx(3.14));
}

TEST_CASE("multi_key_struct" * doctest::test_suite("annotations")) {
    annotation_types::MultiKeyStruct mks("namespace1", 42, "data");
    CHECK(mks.namespace_ == "namespace1");
    CHECK(mks.id == 42);
    CHECK(mks.data == "data");
}

TEST_CASE("optional_fields_default_none" * doctest::test_suite("annotations")) {
    annotation_types::OptionalStruct os;
    CHECK_FALSE(os.optional_int.has_value());
    CHECK_FALSE(os.optional_string.has_value());
    CHECK_FALSE(os.optional_seq.has_value());
}

TEST_CASE("optional_fields_can_be_set" * doctest::test_suite("annotations")) {
    annotation_types::OptionalStruct os;
    os.optional_int = 42;
    os.optional_string = "test";
    os.optional_seq = std::vector<int32_t>{1, 2, 3};

    CHECK(os.optional_int.has_value());
    CHECK(os.optional_int.value() == 42);
    CHECK(os.optional_string.has_value());
    CHECK(os.optional_string.value() == "test");
    CHECK(os.optional_seq.has_value());
    CHECK(os.optional_seq.value().size() == 3u);
}

TEST_CASE("optional_type_annotations" * doctest::test_suite("annotations")) {
    using OptionalIntType = decltype(annotation_types::OptionalStruct().optional_int);
    using OptionalStringType = decltype(annotation_types::OptionalStruct().optional_string);
    using OptionalSeqType = decltype(annotation_types::OptionalStruct().optional_seq);

    CHECK((std::is_same<OptionalIntType, std::optional<int32_t>>::value));
    CHECK((std::is_same<OptionalStringType, std::optional<std::string>>::value));
    CHECK((std::is_same<OptionalSeqType, std::optional<std::vector<int32_t>>>::value));
}

TEST_CASE("nested_struct" * doctest::test_suite("annotations")) {
    annotation_types::NestedStruct ns(10, 20);
    CHECK(ns.x == 10);
    CHECK(ns.y == 20);
}

TEST_CASE("shared_refs_struct" * doctest::test_suite("annotations")) {
    annotation_types::NestedStruct nested(5, 10);
    annotation_types::SharedRefs sr("shared", nested);
    CHECK(sr.shared_string == "shared");
    CHECK(sr.shared_struct.x == 5);
    CHECK(sr.shared_struct.y == 10);
}

TEST_CASE("combined_annotations" * doctest::test_suite("annotations")) {
    annotation_types::CombinedAnnotations ca(99, "combined");
    CHECK(ca.id == 99);
    CHECK(ca.maybe_shared_name.has_value());
    CHECK(ca.maybe_shared_name.value() == "combined");
}

TEST_CASE("annotated_interface_exists" * doctest::test_suite("annotations")) {
    CHECK(std::is_abstract<annotation_types::AnnotatedInterface>::value);
}

TEST_CASE("topic_struct" * doctest::test_suite("annotations")) {
    annotation_types::TopicMessage tm(1, "payload", 123456);
    CHECK(tm.message_id == 1);
    CHECK(tm.payload == "payload");
    CHECK(tm.timestamp == 123456);
}

TEST_CASE("mutable_struct" * doctest::test_suite("annotations")) {
    annotation_types::MutableStruct ms(1, "data");
    CHECK(ms.version == 1);
    CHECK(ms.data == "data");
}

TEST_CASE("final_struct" * doctest::test_suite("annotations")) {
    annotation_types::FinalStruct fs(42);
    CHECK(fs.fixed_field == 42);
}

TEST_CASE("optional_assignment" * doctest::test_suite("annotations")) {
    annotation_types::OptionalStruct os;
    os.optional_int = 100;
    CHECK(os.optional_int.has_value());
    CHECK(os.optional_int.value() == 100);

    os.optional_int.reset();
    CHECK_FALSE(os.optional_int.has_value());
}
