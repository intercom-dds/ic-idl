# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

from types import ModuleType


def test_int_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.INT_CONST == 42
    assert isinstance(c.INT_CONST, int)


def test_uint_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.UINT_CONST == 100
    assert isinstance(c.UINT_CONST, int)


def test_short_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.SHORT_CONST == -10
    assert isinstance(c.SHORT_CONST, int)


def test_longlong_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.LONGLONG_CONST == 9999999999
    assert isinstance(c.LONGLONG_CONST, int)


def test_double_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert abs(c.DOUBLE_CONST - 3.14159) < 0.00001
    assert isinstance(c.DOUBLE_CONST, float)


def test_float_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert abs(c.FLOAT_CONST - 2.5) < 0.001
    assert isinstance(c.FLOAT_CONST, float)


def test_string_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.STRING_CONST == "hello world"
    assert isinstance(c.STRING_CONST, str)


def test_bool_constants(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.BOOL_TRUE is True
    assert c.BOOL_FALSE is False
    assert isinstance(c.BOOL_TRUE, bool)
    assert isinstance(c.BOOL_FALSE, bool)


def test_octet_constant(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.OCTET_CONST == 255
    assert isinstance(c.OCTET_CONST, int)


def test_constant_chain(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.CHAIN_1 == 10
    assert c.CHAIN_2 == 10
    assert c.CHAIN_3 == 10
    assert c.CHAIN_4 == 10
    assert c.CHAIN_5 == 10


def test_arithmetic_chain(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.ARITH_BASE == 100
    assert c.ARITH_DOUBLED == 200
    assert c.ARITH_QUADRUPLED == 400
    assert c.ARITH_OCTUPLED == 800


def test_mixed_arithmetic_chain(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.MATH_1 == 5
    assert c.MATH_2 == 15
    assert c.MATH_3 == 30
    assert c.MATH_4 == 25
    assert c.MATH_5 == 5


def test_negation(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.NEGATIVE == -50
    assert c.NEGATED == 50
    assert c.DOUBLE_NEGATED == -50


def test_bitwise_operations(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.BITS_A == 0x0F
    assert c.BITS_B == 0xF0
    assert c.BITS_OR == 0xFF
    assert c.BITS_AND == 0xFF
    assert c.BITS_XOR == 0xF0
    assert c.BITS_SHIFT_LEFT == 16
    assert c.BITS_SHIFT_RIGHT == 16


def test_float_chain(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.FLOAT_A == 1.0
    assert c.FLOAT_B == 1.5
    assert c.FLOAT_C == 3.0
    assert c.FLOAT_D == 0.75


def test_enum_constant_reference(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.PRIORITY_VALUE == c.Priority.HIGH
    assert c.PRIORITY_CHAIN == c.Priority.HIGH


def test_parenthesized_expressions(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.PAREN_A == 30
    assert c.PAREN_B == 20
    assert c.PAREN_C == 25


def test_modulo_operations(generated_modules: dict[str, ModuleType]) -> None:
    c = generated_modules["constant_types"]
    assert c.MOD_A == 2
    assert c.MOD_B == 2


# Large integer type limit tests


def test_octet_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.OCTET_MAX == 255
    assert li.OCTET_MIN == 0


def test_short_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.SHORT_MAX == 32767
    assert li.SHORT_MIN == -32768


def test_ushort_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.USHORT_MAX == 65535
    assert li.USHORT_MIN == 0


def test_long_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.LONG_MAX == 2147483647
    assert li.LONG_MIN == -2147483648


def test_ulong_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.ULONG_MAX == 4294967295
    assert li.ULONG_MIN == 0


def test_longlong_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.LONGLONG_MAX == 9223372036854775807
    assert li.LONGLONG_MIN == -9223372036854775808


def test_ulonglong_limits(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.ULONGLONG_MAX == 18446744073709551615
    assert li.ULONGLONG_MIN == 0


def test_hex_literals(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.HEX_DEADBEEF == 0xDEADBEEF
    assert li.HEX_FFFFFFFF == 0xFFFFFFFF
    assert li.HEX_64_BIT == 0x123456789ABCDEF0


def test_octal_literals(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.OCTAL_777 == 0o777
    assert li.OCTAL_777 == 511


def test_large_int_struct(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    s = li.LargeIntFields(
        big_signed=9223372036854775807,
        big_unsigned=18446744073709551615,
    )
    assert s.big_signed == 9223372036854775807
    assert s.big_unsigned == 18446744073709551615


def test_derived_constants(generated_modules: dict[str, ModuleType]) -> None:
    li = generated_modules["large_integer_types"]
    assert li.LONG_MAX_MINUS_ONE == 2147483646
    assert li.LONGLONG_MAX_MINUS_ONE == 9223372036854775806
