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

package integrationtests;

import static org.junit.jupiter.api.Assertions.*;

import constant_types.*;
import large_integer_types.*;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

class ConstantsTests {

    @Test
    void int_constant() {
        assertEquals(42, INT_CONST.value);
    }

    @Test
    void uint_constant() {
        assertEquals(100, UINT_CONST.value);
    }

    @Test
    void short_constant() {
        assertEquals(-10, SHORT_CONST.value);
    }

    @Test
    void longlong_constant() {
        assertEquals(9999999999L, LONGLONG_CONST.value);
    }

    @Test
    void double_constant() {
        assertEquals(3.14159, DOUBLE_CONST.value, 0.00001);
    }

    @Test
    void float_constant() {
        assertEquals(2.5f, FLOAT_CONST.value, 0.001f);
    }

    @Test
    void string_constant() {
        assertEquals("hello world", STRING_CONST.value);
    }

    @Test
    void bool_constants() {
        assertTrue(BOOL_TRUE.value);
        assertFalse(BOOL_FALSE.value);
    }

    @Test
    void octet_constant() {
        assertEquals(255, OCTET_CONST.value);
    }

    @Test
    void constant_chain() {
        assertEquals(10, CHAIN_1.value);
        assertEquals(10, CHAIN_2.value);
        assertEquals(10, CHAIN_3.value);
        assertEquals(10, CHAIN_4.value);
        assertEquals(10, CHAIN_5.value);
    }

    @Test
    void arithmetic_chain() {
        assertEquals(100, ARITH_BASE.value);
        assertEquals(200, ARITH_DOUBLED.value);
        assertEquals(400, ARITH_QUADRUPLED.value);
        assertEquals(800, ARITH_OCTUPLED.value);
    }

    @Test
    void mixed_arithmetic_chain() {
        assertEquals(5, MATH_1.value);
        assertEquals(15, MATH_2.value);
        assertEquals(30, MATH_3.value);
        assertEquals(25, MATH_4.value);
        assertEquals(5, MATH_5.value);
    }

    @Test
    void negation() {
        assertEquals(-50, NEGATIVE.value);
        assertEquals(50, NEGATED.value);
        assertEquals(-50, DOUBLE_NEGATED.value);
    }

    @Test
    void bitwise_operations() {
        assertEquals(0x0F, BITS_A.value);
        assertEquals(0xF0, BITS_B.value);
        assertEquals(0xFF, BITS_OR.value);
        assertEquals(255, BITS_AND.value);
        assertEquals(240, BITS_XOR.value);
        assertEquals(16, BITS_SHIFT_LEFT.value);
        assertEquals(16, BITS_SHIFT_RIGHT.value);
    }

    @Test
    void float_chain() {
        assertEquals(1.0, FLOAT_A.value, 0.001);
        assertEquals(1.5, FLOAT_B.value, 0.001);
        assertEquals(3.0, FLOAT_C.value, 0.001);
        assertEquals(0.75, FLOAT_D.value, 0.001);
    }

    @Test
    void enum_constant_reference() {
        assertEquals(Priority.HIGH, PRIORITY_VALUE.value);
        assertEquals(Priority.HIGH, PRIORITY_CHAIN.value);
    }

    @Test
    void parenthesized_expressions() {
        assertEquals(30, PAREN_A.value);
        assertEquals(20, PAREN_B.value);
        assertEquals(25, PAREN_C.value);
    }

    @Test
    void modulo_operations() {
        assertEquals(2, MOD_A.value);
        assertEquals(2, MOD_B.value);
    }

    @Test
    void octet_limits() {
        assertEquals(255, OCTET_MAX.value);
        assertEquals(0, OCTET_MIN.value);
    }

    @Test
    void short_limits() {
        assertEquals(32767, SHORT_MAX.value);
        assertEquals(-32768, SHORT_MIN.value);
    }

    @Test
    void ushort_limits() {
        assertEquals(65535, USHORT_MAX.value);
        assertEquals(0, USHORT_MIN.value);
    }

    @Test
    void long_limits() {
        assertEquals(2147483647, IDL_LONG_MAX.value);
        assertEquals(-2147483648, IDL_LONG_MIN.value);
    }

    @Test
    void ulong_limits() {
        assertEquals(4294967295L, IDL_ULONG_MAX.value);
        assertEquals(0L, IDL_ULONG_MIN.value);
    }

    @Test
    void longlong_limits() {
        assertEquals(9223372036854775807L, LONGLONG_MAX.value);
        assertEquals(-9223372036854775808L, LONGLONG_MIN.value);
    }

    @Test
    @Disabled("figure out how to best represent unsigned long long in Java")
    void ulonglong_limits() {
        assertEquals(new java.math.BigInteger("18446744073709551615"), ULONGLONG_MAX.value);
        assertEquals(java.math.BigInteger.ZERO, ULONGLONG_MIN.value);
    }

    @Test
    void hex_literals() {
        assertEquals(0xDEADBEEFL, HEX_DEADBEEF.value);
        assertEquals(0xFFFFFFFFL, HEX_FFFFFFFF.value);
        assertNotNull(HEX_64_BIT.value);
    }

    @Test
    void octal_literals() {
        assertEquals(511, OCTAL_777.value);
    }

    @Test
    void large_int_struct() {
        var s = new LargeIntFields();
        assertEquals(0L, s.getBigSigned());
        assertEquals(0L, s.getBigUnsigned());
        s.setBigSigned(Long.MAX_VALUE);
        s.setBigUnsigned(Long.MIN_VALUE);
        assertEquals(Long.MAX_VALUE, s.getBigSigned());
        assertEquals(Long.MIN_VALUE, s.getBigUnsigned());
    }

    @Test
    void derived_constants() {
        assertEquals(2147483646, LONG_MAX_MINUS_ONE.value);
        assertEquals(9223372036854775806L, LONGLONG_MAX_MINUS_ONE.value);
    }
}
