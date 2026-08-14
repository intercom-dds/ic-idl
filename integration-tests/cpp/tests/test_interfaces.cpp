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

#include "interfaces.h"

TEST_CASE("interface_is_abc" * doctest::test_suite("interfaces")) {
    CHECK(std::is_abstract<interface_types::Reader>::value);
}

TEST_CASE("interface_has_abstract_methods" * doctest::test_suite("interfaces")) {
    CHECK(std::is_abstract<interface_types::Reader>::value);
}

TEST_CASE("interface_method_signature_no_params" * doctest::test_suite("interfaces")) {
    using ReadReturnType = decltype(std::declval<interface_types::Reader>().read());
    CHECK((std::is_same<ReadReturnType, std::string>::value));
}

TEST_CASE("interface_method_signature_with_params" * doctest::test_suite("interfaces")) {
    using AddReturnType = decltype(std::declval<interface_types::Calculator>().add(0, 0));
    CHECK((std::is_same<AddReturnType, int32_t>::value));
}

TEST_CASE("interface_void_return" * doctest::test_suite("interfaces")) {
    using FlushReturnType = decltype(std::declval<interface_types::Writer>().flush());
    CHECK((std::is_same<FlushReturnType, void>::value));
}

TEST_CASE("empty_interface" * doctest::test_suite("interfaces")) {
    CHECK_FALSE(std::is_abstract_v<interface_types::Empty>);
    CHECK(std::has_virtual_destructor_v<interface_types::Empty>);
    CHECK(std::is_polymorphic_v<interface_types::Empty>);
}

TEST_CASE("operation_failed_exception" * doctest::test_suite("interfaces")) {
    interface_types::OperationFailed ex(42, "Test error");
    CHECK(ex.error_code == 42);
    CHECK(ex.reason == "Test error");
}

TEST_CASE("invalid_input_exception" * doctest::test_suite("interfaces")) {
    interface_types::InvalidInput ex("param_name");
    CHECK(ex.parameter_name == "param_name");
}

TEST_CASE("exception_can_be_raised" * doctest::test_suite("interfaces")) {
    try {
        throw interface_types::OperationFailed(500, "Server error");
        FAIL("Expected OperationFailed to be thrown");
    } catch (const interface_types::OperationFailed& e) {
        CHECK(e.error_code == 500);
        CHECK(e.reason == "Server error");
    } catch (...) {
        FAIL("Exception not caught properly");
    }
}

TEST_CASE("interface_with_out_params_exists" * doctest::test_suite("interfaces")) {
    CHECK(std::is_abstract<interface_types::WithOutParams>::value);
}

TEST_CASE("interface_with_raises_exists" * doctest::test_suite("interfaces")) {
    CHECK(std::is_abstract<interface_types::WithRaises>::value);
}

TEST_CASE("combined_features_interface" * doctest::test_suite("interfaces")) {
    CHECK(std::is_abstract<interface_types::CombinedFeatures>::value);
}

TEST_CASE("interface_calculator_all_signatures" * doctest::test_suite("interfaces")) {
    using AddReturnType = decltype(std::declval<interface_types::Calculator>().add(0, 0));
    using SubtractReturnType = decltype(std::declval<interface_types::Calculator>().subtract(0, 0));
    using DivideReturnType = decltype(std::declval<interface_types::Calculator>().divide(0.0, 0.0));

    CHECK((std::is_same<AddReturnType, int32_t>::value));
    CHECK((std::is_same<SubtractReturnType, int32_t>::value));
    CHECK((std::is_same<DivideReturnType, double>::value));
}

TEST_CASE("interface_writer_parameter_types" * doctest::test_suite("interfaces")) {
    using WriteMemberFunc = void (interface_types::Writer::*)(std::string_view);
    WriteMemberFunc write_ptr = &interface_types::Writer::write;
    CHECK(write_ptr != nullptr);
}

class ConcreteReader : public interface_types::Reader {
  public:
    std::string read() override {
        return "test data";
    }

    bool has_more() override {
        return false;
    }
};

TEST_CASE("interface_can_be_implemented" * doctest::test_suite("interfaces")) {
    ConcreteReader reader;
    CHECK(reader.read() == "test data");
    CHECK_FALSE(reader.has_more());
}
