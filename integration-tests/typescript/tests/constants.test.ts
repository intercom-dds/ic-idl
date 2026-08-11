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

import { describe, expect, test } from "bun:test";
import {
  INT_CONST,
  UINT_CONST,
  SHORT_CONST,
  LONGLONG_CONST,
  DOUBLE_CONST,
  FLOAT_CONST,
  STRING_CONST,
  BOOL_TRUE,
  BOOL_FALSE,
  OCTET_CONST,
  CHAIN_1,
  CHAIN_2,
  CHAIN_3,
  CHAIN_4,
  CHAIN_5,
  ARITH_BASE,
  ARITH_DOUBLED,
  ARITH_QUADRUPLED,
  ARITH_OCTUPLED,
  NEGATIVE,
  NEGATED,
  DOUBLE_NEGATED,
  BITS_A,
  BITS_B,
  BITS_OR,
  BITS_AND,
  BITS_XOR,
  BITS_SHIFT_LEFT,
  BITS_SHIFT_RIGHT,
  FLOAT_A,
  FLOAT_B,
  FLOAT_C,
  FLOAT_D,
  Priority,
  PRIORITY_VALUE,
  PRIORITY_CHAIN,
  GREETING,
  FAREWELL,
  PAREN_A,
  PAREN_B,
  PAREN_C,
  MOD_A,
  MOD_B,
} from "../generated/constant_types";
import {
  OCTET_MAX,
  OCTET_MIN,
  SHORT_MAX,
  SHORT_MIN,
  USHORT_MAX,
  USHORT_MIN,
  IDL_LONG_MAX,
  IDL_LONG_MIN,
  IDL_ULONG_MAX,
  IDL_ULONG_MIN,
  LONGLONG_MAX,
  LONGLONG_MIN,
  ULONGLONG_MAX,
  ULONGLONG_MIN,
  HEX_DEADBEEF,
  HEX_FFFFFFFF,
  HEX_64BIT,
  OCTAL_777,
  LONG_MAX_MINUS_ONE,
  LONGLONG_MAX_MINUS_ONE,
} from "../generated/large_integer_types";
import type { LargeIntFields } from "../generated/large_integer_types";

describe("constants", () => {
  describe("primitive constants", () => {
    test("integer constants", () => {
      expect(INT_CONST).toBe(42);
      expect(UINT_CONST).toBe(100);
      expect(SHORT_CONST).toBe(-10);
      expect(OCTET_CONST).toBe(255);
    });

    test("large integer constant", () => {
      expect(LONGLONG_CONST).toBe(9999999999);
    });

    test("floating point constants", () => {
      expect(DOUBLE_CONST).toBeCloseTo(3.14159, 5);
      expect(FLOAT_CONST).toBe(2.5);
    });

    test("string constants", () => {
      expect(STRING_CONST).toBe("hello world");
      expect(GREETING).toBe("Hello");
      expect(FAREWELL).toBe("Goodbye");
    });

    test("boolean constants", () => {
      expect(BOOL_TRUE).toBe(true);
      expect(BOOL_FALSE).toBe(false);
    });
  });

  describe("constant chains", () => {
    test("constants can reference other constants", () => {
      expect(CHAIN_1).toBe(10);
      expect(CHAIN_2).toBe(CHAIN_1);
      expect(CHAIN_3).toBe(CHAIN_2);
      expect(CHAIN_4).toBe(CHAIN_3);
      expect(CHAIN_5).toBe(CHAIN_4);
    });
  });

  describe("arithmetic constants", () => {
    test("multiplication chain", () => {
      expect(ARITH_BASE).toBe(100);
      expect(ARITH_DOUBLED).toBe(200);
      expect(ARITH_QUADRUPLED).toBe(400);
      expect(ARITH_OCTUPLED).toBe(800);
    });

    test("negation", () => {
      expect(NEGATIVE).toBe(-50);
      expect(NEGATED).toBe(50);
      expect(DOUBLE_NEGATED).toBe(-50);
    });

    test("parenthesized expressions", () => {
      expect(PAREN_A).toBe(30);
      expect(PAREN_B).toBe(20);
      expect(PAREN_C).toBe(25);
    });

    test("modulo", () => {
      expect(MOD_A).toBe(2);
      expect(MOD_B).toBe(2);
    });
  });

  describe("bitwise constants", () => {
    test("hex values", () => {
      expect(BITS_A).toBe(0x0f);
      expect(BITS_B).toBe(0xf0);
    });

    test("bitwise operations", () => {
      expect(BITS_OR).toBe(0xff);
      expect(BITS_AND).toBe(0xff);
      expect(BITS_XOR).toBe(0xf0);
    });

    test("shifts", () => {
      expect(BITS_SHIFT_LEFT).toBe(16);
      expect(BITS_SHIFT_RIGHT).toBe(16);
    });
  });

  describe("float arithmetic", () => {
    test("float chain", () => {
      expect(FLOAT_A).toBe(1);
      expect(FLOAT_B).toBe(1.5);
      expect(FLOAT_C).toBe(3);
      expect(FLOAT_D).toBe(0.75);
    });
  });

  describe("enum constants", () => {
    test("enum values", () => {
      expect(Priority.LOW).toBe(0);
      expect(Priority.MEDIUM).toBe(50);
      expect(Priority.HIGH).toBe(100);
    });

    test("constant from enum", () => {
      expect(PRIORITY_VALUE).toBe(Priority.HIGH);
      expect(PRIORITY_CHAIN).toBe(PRIORITY_VALUE);
    });
  });
});

describe("large integer constants", () => {
  describe("integer limits", () => {
    test("octet limits", () => {
      expect(OCTET_MAX).toBe(255);
      expect(OCTET_MIN).toBe(0);
    });

    test("short limits", () => {
      expect(SHORT_MAX).toBe(32767);
      expect(SHORT_MIN).toBe(-32768);
    });

    test("ushort limits", () => {
      expect(USHORT_MAX).toBe(65535);
      expect(USHORT_MIN).toBe(0);
    });

    test("long limits", () => {
      expect(IDL_LONG_MAX).toBe(2147483647);
      expect(IDL_LONG_MIN).toBe(-2147483648);
    });

    test("ulong limits", () => {
      expect(IDL_ULONG_MAX).toBe(4294967295);
      expect(IDL_ULONG_MIN).toBe(0);
    });

    test("longlong limits (as strings due to JS number limits)", () => {
      expect(LONGLONG_MAX).toBe("9223372036854775807");
      expect(LONGLONG_MIN).toBe("-9223372036854775808");
    });

    test("ulonglong limits (as strings due to JS number limits)", () => {
      expect(ULONGLONG_MAX).toBe("18446744073709551615");
      expect(ULONGLONG_MIN).toBe(0);
    });
  });

  describe("hex literals", () => {
    test("32-bit hex values", () => {
      expect(HEX_DEADBEEF).toBe(0xdeadbeef);
      expect(HEX_FFFFFFFF).toBe(0xffffffff);
    });

    test("64-bit hex (as string)", () => {
      expect(HEX_64BIT).toBe("1311768467463790320");
    });
  });

  describe("octal literals", () => {
    test("octal 777", () => {
      expect(OCTAL_777).toBe(0o777);
      expect(OCTAL_777).toBe(511);
    });
  });

  describe("large int struct", () => {
    test("can hold large integers", () => {
      const s: LargeIntFields = {
        big_signed: "9223372036854775807",
        big_unsigned: "18446744073709551615",
      };
      expect(s.big_signed).toBe("9223372036854775807");
      expect(s.big_unsigned).toBe("18446744073709551615");
    });
  });

  describe("derived constants", () => {
    test("constants derived from limits", () => {
      expect(LONG_MAX_MINUS_ONE).toBe(2147483646);
      expect(LONGLONG_MAX_MINUS_ONE).toBe("9223372036854775806");
    });
  });
});
