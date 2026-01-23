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
import type { StructA1, StructA2, StructA3 } from "../generated/module_a";
import {
  CONST_A1,
  CONST_A2,
  CONST_A3,
  EnumA,
  EnumA2,
} from "../generated/module_a";
import type { StructB1, StructB2 } from "../generated/module_b";
import { CONST_B1, CONST_B2 } from "../generated/module_b";
import { C1, C2, C3 } from "../generated/constants_only";
import { Color, Size } from "../generated/enums_only";

describe("multi module", () => {
  describe("module_a", () => {
    test("first opening - StructA1 and CONST_A1", () => {
      const a1: StructA1 = { value: 10 };
      expect(a1.value).toBe(10);
      expect(CONST_A1).toBe(100);
      expect(EnumA.X).toBe(0);
      expect(EnumA.Y).toBe(1);
    });

    test("second opening - StructA2 and CONST_A2", () => {
      const a1: StructA1 = { value: 5 };
      const a2: StructA2 = { data: 3.14, ref_to_a1: a1 };
      expect(a2.data).toBe(3.14);
      expect(a2.ref_to_a1.value).toBe(5);
      expect(CONST_A2).toBe(101);
      expect(EnumA2.P).toBe(0);
      expect(EnumA2.Q).toBe(1);
      expect(EnumA2.R).toBe(2);
    });

    test("third opening - StructA3 and CONST_A3", () => {
      const a1: StructA1 = { value: 1 };
      const a2: StructA2 = { data: 2.0, ref_to_a1: a1 };
      const a3: StructA3 = { flag: true, a1, a2 };
      expect(a3.flag).toBe(true);
      expect(a3.a1.value).toBe(1);
      expect(a3.a2.data).toBe(2.0);
      expect(CONST_A3).toBe(102);
    });
  });

  describe("module_b", () => {
    test("both openings - StructB1, StructB2, and constants", () => {
      const b1: StructB1 = { name: "test" };
      const b2: StructB2 = { id: 42, refToB1: b1 };
      expect(b1.name).toBe("test");
      expect(b2.id).toBe(42);
      expect(b2.refToB1.name).toBe("test");
      expect(CONST_B1).toBe(200);
      expect(CONST_B2).toBe(201);
    });
  });

  describe("reopened module types can reference earlier", () => {
    test("StructA2 references StructA1", () => {
      const a1: StructA1 = { value: 10 };
      const a2: StructA2 = { data: 3.14, ref_to_a1: a1 };
      expect(a2.ref_to_a1.value).toBe(10);
    });
  });

  describe("reopened module chain", () => {
    test("StructA3 references both StructA1 and StructA2", () => {
      const a1: StructA1 = { value: 1 };
      const a2: StructA2 = { data: 2.0, refToA1: a1 };
      const a3: StructA3 = { flag: true, a1, a2 };
      expect(a3.flag).toBe(true);
      expect(a3.a1.value).toBe(1);
      expect(a3.a2.data).toBe(2.0);
    });
  });

  describe("constants only module", () => {
    test("constants are exported", () => {
      expect(C1).toBe(1);
      expect(C2).toBe(2);
      expect(C3).toBe(3);
    });
  });

  describe("enums only module", () => {
    test("enums are exported", () => {
      expect(Color.RED).toBe(0);
      expect(Color.GREEN).toBe(1);
      expect(Color.BLUE).toBe(2);
      expect(Size.SMALL).toBe(0);
      expect(Size.MEDIUM).toBe(1);
      expect(Size.LARGE).toBe(2);
    });
  });
});
