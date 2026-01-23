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
import type {
  IntOrString,
  TypedValue,
  BoolSwitch,
  MultiCase,
} from "../generated/union_types";
import { ValueKind } from "../generated/union_types";

describe("unions", () => {
  describe("IntOrString", () => {
    test("integer variant", () => {
      const val: IntOrString = { $discriminator: 1, int_val: 42 };
      expect(val.$discriminator).toBe(1);
      if (val.$discriminator === 1) {
        expect(val.int_val).toBe(42);
      }
    });

    test("string variant", () => {
      const val: IntOrString = { $discriminator: 2, str_val: "hello" };
      expect(val.$discriminator).toBe(2);
      if (val.$discriminator === 2) {
        expect(val.str_val).toBe("hello");
      }
    });

    test("default variant", () => {
      const val: IntOrString = { $discriminator: 99, default_val: true };
      expect(val.$discriminator).toBe(99);
      if (val.$discriminator !== 1 && val.$discriminator !== 2) {
        expect(val.default_val).toBe(true);
      }
    });
  });

  describe("TypedValue with enum discriminator", () => {
    test("IntKind variant", () => {
      const val: TypedValue = {
        $discriminator: ValueKind.INT_KIND,
        int_value: 123,
      };
      expect(val.$discriminator).toBe(ValueKind.INT_KIND);
      if (val.$discriminator === ValueKind.INT_KIND) {
        expect(val.int_value).toBe(123);
      }
    });

    test("FloatKind variant", () => {
      const val: TypedValue = {
        $discriminator: ValueKind.FLOAT_KIND,
        float_value: 3.14,
      };
      expect(val.$discriminator).toBe(ValueKind.FLOAT_KIND);
      if (val.$discriminator === ValueKind.FLOAT_KIND) {
        expect(val.float_value).toBeCloseTo(3.14);
      }
    });

    test("StringKind variant", () => {
      const val: TypedValue = {
        $discriminator: ValueKind.STRING_KIND,
        string_value: "test",
      };
      expect(val.$discriminator).toBe(ValueKind.STRING_KIND);
      if (val.$discriminator === ValueKind.STRING_KIND) {
        expect(val.string_value).toBe("test");
      }
    });
  });

  describe("BoolSwitch", () => {
    test("true variant", () => {
      const val: BoolSwitch = { $discriminator: true, true_val: 100 };
      expect(val.$discriminator).toBe(true);
      if (val.$discriminator === true) {
        expect(val.true_val).toBe(100);
      }
    });

    test("false variant", () => {
      const val: BoolSwitch = { $discriminator: false, false_val: "off" };
      expect(val.$discriminator).toBe(false);
      if (val.$discriminator === false) {
        expect(val.false_val).toBe("off");
      }
    });
  });

  describe("MultiCase", () => {
    test("case 1, 2, 3 share same variant", () => {
      const val1: MultiCase = { $discriminator: 1, small_val: 10 };
      const val2: MultiCase = { $discriminator: 2, small_val: 20 };
      const val3: MultiCase = { $discriminator: 3, small_val: 30 };

      expect(val1.small_val).toBe(10);
      expect(val2.small_val).toBe(20);
      expect(val3.small_val).toBe(30);
    });

    test("case 10, 20 share same variant", () => {
      const val10: MultiCase = { $discriminator: 10, text_val: "ten" };
      const val20: MultiCase = { $discriminator: 20, text_val: "twenty" };

      expect(val10.text_val).toBe("ten");
      expect(val20.text_val).toBe("twenty");
    });

    test("default case", () => {
      const val: MultiCase = { $discriminator: 999, flag: false };
      expect(val.flag).toBe(false);
    });
  });
});
