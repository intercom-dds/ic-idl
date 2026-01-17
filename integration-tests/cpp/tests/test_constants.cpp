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

#include <cstring>

#include "generated/constants.h"

TEST_CASE("int_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::INT_CONST == 42);
}

TEST_CASE("uint_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::UINT_CONST == 100U);
}

TEST_CASE("short_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::SHORT_CONST == -10);
}

TEST_CASE("longlong_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::LONGLONG_CONST == 9999999999LL);
}

TEST_CASE("double_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::DOUBLE_CONST == doctest::Approx(3.14159));
}

TEST_CASE("float_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::FLOAT_CONST == doctest::Approx(2.5f));
}

TEST_CASE("string_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::STRING_CONST == "hello world");
}

TEST_CASE("bool_constants" * doctest::test_suite("constants")) {
    CHECK(constant_types::BOOL_TRUE == true);
    CHECK(constant_types::BOOL_FALSE == false);
}

TEST_CASE("octet_constant" * doctest::test_suite("constants")) {
    CHECK(constant_types::OCTET_CONST == 255U);
}

TEST_CASE("constant_chain" * doctest::test_suite("constants")) {
    CHECK(constant_types::CHAIN_1 == 10);
    CHECK(constant_types::CHAIN_2 == 10);
    CHECK(constant_types::CHAIN_3 == 10);
    CHECK(constant_types::CHAIN_4 == 10);
    CHECK(constant_types::CHAIN_5 == 10);
    CHECK(constant_types::CHAIN_2 == constant_types::CHAIN_1);
    CHECK(constant_types::CHAIN_3 == constant_types::CHAIN_2);
    CHECK(constant_types::CHAIN_4 == constant_types::CHAIN_3);
    CHECK(constant_types::CHAIN_5 == constant_types::CHAIN_4);
}

TEST_CASE("arithmetic_chain" * doctest::test_suite("constants")) {
    CHECK(constant_types::ARITH_BASE == 100);
    CHECK(constant_types::ARITH_DOUBLED == 200);
    CHECK(constant_types::ARITH_QUADRUPLED == 400);
    CHECK(constant_types::ARITH_OCTUPLED == 800);
    CHECK(constant_types::ARITH_DOUBLED == constant_types::ARITH_BASE * 2);
    CHECK(constant_types::ARITH_QUADRUPLED == constant_types::ARITH_BASE * 4);
    CHECK(constant_types::ARITH_OCTUPLED == constant_types::ARITH_BASE * 8);
}

TEST_CASE("mixed_arithmetic_chain" * doctest::test_suite("constants")) {
    CHECK(constant_types::MATH_1 == 5);
    CHECK(constant_types::MATH_2 == 15);
    CHECK(constant_types::MATH_3 == 30);
    CHECK(constant_types::MATH_4 == 25);
    CHECK(constant_types::MATH_5 == 5);
}

TEST_CASE("negation" * doctest::test_suite("constants")) {
    CHECK(constant_types::NEGATIVE == -50);
    CHECK(constant_types::NEGATED == 50);
    CHECK(constant_types::DOUBLE_NEGATED == -50);
    CHECK(constant_types::NEGATED == -constant_types::NEGATIVE);
    CHECK(constant_types::DOUBLE_NEGATED == -constant_types::NEGATED);
}

TEST_CASE("bitwise_operations" * doctest::test_suite("constants")) {
    CHECK(constant_types::BITS_A == 0x0F);
    CHECK(constant_types::BITS_B == 0xF0);
    CHECK(constant_types::BITS_OR == 0xFF);
    CHECK(constant_types::BITS_AND == 255);
    CHECK(constant_types::BITS_XOR == 240);
    CHECK(constant_types::BITS_SHIFT_LEFT == 16);
    CHECK(constant_types::BITS_SHIFT_RIGHT == 16);
    CHECK(constant_types::BITS_OR == (constant_types::BITS_A | constant_types::BITS_B));
}

TEST_CASE("float_chain" * doctest::test_suite("constants")) {
    CHECK(constant_types::FLOAT_A == doctest::Approx(1.0));
    CHECK(constant_types::FLOAT_B == doctest::Approx(1.5));
    CHECK(constant_types::FLOAT_C == doctest::Approx(3.0));
    CHECK(constant_types::FLOAT_D == doctest::Approx(0.75));
}

TEST_CASE("enum_constant_reference" * doctest::test_suite("constants")) {
    CHECK(constant_types::PRIORITY_VALUE == constant_types::HIGH);
    CHECK(constant_types::PRIORITY_VALUE == 100);
    CHECK(static_cast<int32_t>(constant_types::HIGH) == 100);
    CHECK(constant_types::PRIORITY_CHAIN == constant_types::PRIORITY_VALUE);
}

TEST_CASE("parenthesized_expressions" * doctest::test_suite("constants")) {
    CHECK(constant_types::PAREN_A == 30);
    CHECK(constant_types::PAREN_B == 20);
    CHECK(constant_types::PAREN_C == 25);
}

TEST_CASE("modulo_operations" * doctest::test_suite("constants")) {
    CHECK(constant_types::MOD_A == 2);
    CHECK(constant_types::MOD_B == 2);
}

TEST_CASE("octet_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::OCTET_MAX == 255U);
    CHECK(large_integer_types::OCTET_MIN == 0U);
}

TEST_CASE("short_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::SHORT_MAX == 32767);
    CHECK(large_integer_types::SHORT_MIN == -32768);
}

TEST_CASE("ushort_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::USHORT_MAX == 65535U);
    CHECK(large_integer_types::USHORT_MIN == 0U);
}

TEST_CASE("long_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::LONG_MAX == 2147483647);
    CHECK(large_integer_types::LONG_MIN == -2147483648);
}

TEST_CASE("ulong_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::ULONG_MAX == 4294967295U);
    CHECK(large_integer_types::ULONG_MIN == 0U);
}

TEST_CASE("longlong_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::LONGLONG_MAX == 9223372036854775807LL);
    CHECK(large_integer_types::LONGLONG_MIN == -9223372036854775807LL);
}

TEST_CASE("ulonglong_limits" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::ULONGLONG_MAX == 18446744073709551615ULL);
    CHECK(large_integer_types::ULONGLONG_MIN == 0ULL);
}

TEST_CASE("hex_literals" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::HEX_DEADBEEF == 0xDEADBEEFU);
    CHECK(large_integer_types::HEX_FFFFFFFF == 0xFFFFFFFFU);
    CHECK(large_integer_types::HEX_64BIT == 0x123456789ABCDEF0LL);
}

TEST_CASE("octal_literals" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::OCTAL_777 == 511);
    CHECK(large_integer_types::OCTAL_777 == 0777);
}

TEST_CASE("large_int_struct" * doctest::test_suite("constants")) {
    large_integer_types::LargeIntFields fields(9223372036854775807LL, 18446744073709551615ULL);
    CHECK(fields.big_signed == 9223372036854775807LL);
    CHECK(fields.big_unsigned == 18446744073709551615ULL);
    CHECK(fields.big_signed == large_integer_types::LONGLONG_MAX);
    CHECK(fields.big_unsigned == large_integer_types::ULONGLONG_MAX);

    large_integer_types::LargeIntFields default_fields;
    CHECK(default_fields.big_signed == 0);
    CHECK(default_fields.big_unsigned == 0ULL);

    large_integer_types::LargeIntFields copy_fields = fields;
    CHECK(copy_fields == fields);
    CHECK(copy_fields.big_signed == fields.big_signed);
    CHECK(copy_fields.big_unsigned == fields.big_unsigned);
}

TEST_CASE("derived_constants" * doctest::test_suite("constants")) {
    CHECK(large_integer_types::LONG_MAX_MINUS_ONE == 2147483646);
    CHECK(large_integer_types::LONG_MAX_MINUS_ONE == large_integer_types::LONG_MAX - 1);
    CHECK(large_integer_types::LONGLONG_MAX_MINUS_ONE == 9223372036854775806LL);
    CHECK(large_integer_types::LONGLONG_MAX_MINUS_ONE == large_integer_types::LONGLONG_MAX - 1);
}
