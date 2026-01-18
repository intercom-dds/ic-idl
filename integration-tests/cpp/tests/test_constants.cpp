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

#include <cstring>

#include "generated/constants.h"

namespace {

TEST(ConstantsTest, test_int_constant) {
    EXPECT_EQ(constant_types::INT_CONST, 42);
}

TEST(ConstantsTest, test_uint_constant) {
    EXPECT_EQ(constant_types::UINT_CONST, 100U);
}

TEST(ConstantsTest, test_short_constant) {
    EXPECT_EQ(constant_types::SHORT_CONST, -10);
}

TEST(ConstantsTest, test_longlong_constant) {
    EXPECT_EQ(constant_types::LONGLONG_CONST, 9999999999LL);
}

TEST(ConstantsTest, test_double_constant) {
    EXPECT_DOUBLE_EQ(constant_types::DOUBLE_CONST, 3.14159);
}

TEST(ConstantsTest, test_float_constant) {
    EXPECT_FLOAT_EQ(constant_types::FLOAT_CONST, 2.5f);
}

TEST(ConstantsTest, test_string_constant) {
    EXPECT_STREQ(constant_types::STRING_CONST, "hello world");
}

TEST(ConstantsTest, test_bool_constants) {
    EXPECT_EQ(constant_types::BOOL_TRUE, true);
    EXPECT_EQ(constant_types::BOOL_FALSE, false);
}

TEST(ConstantsTest, test_octet_constant) {
    EXPECT_EQ(constant_types::OCTET_CONST, 255U);
}

TEST(ConstantsTest, test_constant_chain) {
    EXPECT_EQ(constant_types::CHAIN_1, 10);
    EXPECT_EQ(constant_types::CHAIN_2, 10);
    EXPECT_EQ(constant_types::CHAIN_3, 10);
    EXPECT_EQ(constant_types::CHAIN_4, 10);
    EXPECT_EQ(constant_types::CHAIN_5, 10);
    EXPECT_EQ(constant_types::CHAIN_2, constant_types::CHAIN_1);
    EXPECT_EQ(constant_types::CHAIN_3, constant_types::CHAIN_2);
    EXPECT_EQ(constant_types::CHAIN_4, constant_types::CHAIN_3);
    EXPECT_EQ(constant_types::CHAIN_5, constant_types::CHAIN_4);
}

TEST(ConstantsTest, test_arithmetic_chain) {
    EXPECT_EQ(constant_types::ARITH_BASE, 100);
    EXPECT_EQ(constant_types::ARITH_DOUBLED, 200);
    EXPECT_EQ(constant_types::ARITH_QUADRUPLED, 400);
    EXPECT_EQ(constant_types::ARITH_OCTUPLED, 800);
    EXPECT_EQ(constant_types::ARITH_DOUBLED, constant_types::ARITH_BASE * 2);
    EXPECT_EQ(constant_types::ARITH_QUADRUPLED, constant_types::ARITH_BASE * 4);
    EXPECT_EQ(constant_types::ARITH_OCTUPLED, constant_types::ARITH_BASE * 8);
}

TEST(ConstantsTest, test_mixed_arithmetic_chain) {
    EXPECT_EQ(constant_types::MATH_1, 5);
    EXPECT_EQ(constant_types::MATH_2, 15);
    EXPECT_EQ(constant_types::MATH_3, 30);
    EXPECT_EQ(constant_types::MATH_4, 25);
    EXPECT_EQ(constant_types::MATH_5, 5);
}

TEST(ConstantsTest, test_negation) {
    EXPECT_EQ(constant_types::NEGATIVE, -50);
    EXPECT_EQ(constant_types::NEGATED, 50);
    EXPECT_EQ(constant_types::DOUBLE_NEGATED, -50);
    EXPECT_EQ(constant_types::NEGATED, -constant_types::NEGATIVE);
    EXPECT_EQ(constant_types::DOUBLE_NEGATED, -constant_types::NEGATED);
}

TEST(ConstantsTest, test_bitwise_operations) {
    EXPECT_EQ(constant_types::BITS_A, 0x0F);
    EXPECT_EQ(constant_types::BITS_B, 0xF0);
    EXPECT_EQ(constant_types::BITS_OR, 0xFF);
    EXPECT_EQ(constant_types::BITS_AND, 255);
    EXPECT_EQ(constant_types::BITS_XOR, 240);
    EXPECT_EQ(constant_types::BITS_SHIFT_LEFT, 16);
    EXPECT_EQ(constant_types::BITS_SHIFT_RIGHT, 16);
    EXPECT_EQ(constant_types::BITS_OR, constant_types::BITS_A | constant_types::BITS_B);
}

TEST(ConstantsTest, test_float_chain) {
    EXPECT_DOUBLE_EQ(constant_types::FLOAT_A, 1.0);
    EXPECT_DOUBLE_EQ(constant_types::FLOAT_B, 1.5);
    EXPECT_DOUBLE_EQ(constant_types::FLOAT_C, 3.0);
    EXPECT_DOUBLE_EQ(constant_types::FLOAT_D, 0.75);
}

TEST(ConstantsTest, test_enum_constant_reference) {
    EXPECT_EQ(constant_types::PRIORITY_VALUE, constant_types::HIGH);
    EXPECT_EQ(constant_types::PRIORITY_VALUE, 100);
    EXPECT_EQ(static_cast<int32_t>(constant_types::HIGH), 100);
    EXPECT_EQ(constant_types::PRIORITY_CHAIN, constant_types::PRIORITY_VALUE);
}

TEST(ConstantsTest, test_parenthesized_expressions) {
    EXPECT_EQ(constant_types::PAREN_A, 30);
    EXPECT_EQ(constant_types::PAREN_B, 20);
    EXPECT_EQ(constant_types::PAREN_C, 25);
}

TEST(ConstantsTest, test_modulo_operations) {
    EXPECT_EQ(constant_types::MOD_A, 2);
    EXPECT_EQ(constant_types::MOD_B, 2);
}

TEST(ConstantsTest, test_octet_limits) {
    EXPECT_EQ(large_integer_types::OCTET_MAX, 255U);
    EXPECT_EQ(large_integer_types::OCTET_MIN, 0U);
}

TEST(ConstantsTest, test_short_limits) {
    EXPECT_EQ(large_integer_types::SHORT_MAX, 32767);
    EXPECT_EQ(large_integer_types::SHORT_MIN, -32768);
}

TEST(ConstantsTest, test_ushort_limits) {
    EXPECT_EQ(large_integer_types::USHORT_MAX, 65535U);
    EXPECT_EQ(large_integer_types::USHORT_MIN, 0U);
}

TEST(ConstantsTest, test_long_limits) {
    EXPECT_EQ(large_integer_types::LONG_MAX, 2147483647);
    EXPECT_EQ(large_integer_types::LONG_MIN, -2147483648);
}

TEST(ConstantsTest, test_ulong_limits) {
    EXPECT_EQ(large_integer_types::ULONG_MAX, 4294967295U);
    EXPECT_EQ(large_integer_types::ULONG_MIN, 0U);
}

TEST(ConstantsTest, test_longlong_limits) {
    EXPECT_EQ(large_integer_types::LONGLONG_MAX, 9223372036854775807LL);
    EXPECT_EQ(large_integer_types::LONGLONG_MIN, -9223372036854775807LL);
}

TEST(ConstantsTest, test_ulonglong_limits) {
    EXPECT_EQ(large_integer_types::ULONGLONG_MAX, 18446744073709551615ULL);
    EXPECT_EQ(large_integer_types::ULONGLONG_MIN, 0ULL);
}

TEST(ConstantsTest, test_hex_literals) {
    EXPECT_EQ(large_integer_types::HEX_DEADBEEF, 0xDEADBEEFU);
    EXPECT_EQ(large_integer_types::HEX_FFFFFFFF, 0xFFFFFFFFU);
    EXPECT_EQ(large_integer_types::HEX_64BIT, 0x123456789ABCDEF0LL);
}

TEST(ConstantsTest, test_octal_literals) {
    EXPECT_EQ(large_integer_types::OCTAL_777, 511);
    EXPECT_EQ(large_integer_types::OCTAL_777, 0777);
}

TEST(ConstantsTest, test_large_int_struct) {
    large_integer_types::LargeIntFields fields(9223372036854775807LL, 18446744073709551615ULL);
    EXPECT_EQ(fields.big_signed, 9223372036854775807LL);
    EXPECT_EQ(fields.big_unsigned, 18446744073709551615ULL);
    EXPECT_EQ(fields.big_signed, large_integer_types::LONGLONG_MAX);
    EXPECT_EQ(fields.big_unsigned, large_integer_types::ULONGLONG_MAX);

    large_integer_types::LargeIntFields default_fields;
    EXPECT_EQ(default_fields.big_signed, 0);
    EXPECT_EQ(default_fields.big_unsigned, 0ULL);

    large_integer_types::LargeIntFields copy_fields = fields;
    EXPECT_EQ(copy_fields, fields);
    EXPECT_EQ(copy_fields.big_signed, fields.big_signed);
    EXPECT_EQ(copy_fields.big_unsigned, fields.big_unsigned);
}

TEST(ConstantsTest, test_derived_constants) {
    EXPECT_EQ(large_integer_types::LONG_MAX_MINUS_ONE, 2147483646);
    EXPECT_EQ(large_integer_types::LONG_MAX_MINUS_ONE, large_integer_types::LONG_MAX - 1);
    EXPECT_EQ(large_integer_types::LONGLONG_MAX_MINUS_ONE, 9223372036854775806LL);
    EXPECT_EQ(large_integer_types::LONGLONG_MAX_MINUS_ONE, large_integer_types::LONGLONG_MAX - 1);
}

} // namespace
