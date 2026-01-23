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
  TopLevelStruct,
  TopUsingNested,
} from "../generated/nested_module_types";
import { TopLevelEnum, level1, sibling } from "../generated/nested_module_types";

describe("nested modules", () => {
  describe("top level types", () => {
    test("TopLevelStruct instantiation", () => {
      const s: TopLevelStruct = { value: 42 };
      expect(s.value).toBe(42);
    });

    test("TopLevelEnum exists", () => {
      expect(TopLevelEnum.FIRST).toBe(0);
      expect(TopLevelEnum.SECOND).toBe(1);
    });
  });

  describe("level1 module", () => {
    test("Level1Struct exists", () => {
      const parent: TopLevelStruct = { value: 1 };
      const s: level1.Level1Struct = { data: 10, parent_ref: parent };
      expect(s.data).toBe(10);
      expect(s.parent_ref.value).toBe(1);
    });

    test("Level1Enum exists", () => {
      expect(level1.Level1Enum.A).toBe(0);
      expect(level1.Level1Enum.B).toBe(1);
      expect(level1.Level1Enum.C).toBe(2);
    });
  });

  describe("level2 module", () => {
    test("Level2Struct exists", () => {
      const top: TopLevelStruct = { value: 1 };
      const l1: level1.Level1Struct = { data: 2, parent_ref: top };
      const l2: level1.level2.Level2Struct = {
        name: "test",
        level1_ref: l1,
        top_ref: top,
      };
      expect(l2.name).toBe("test");
      expect(l2.level1_ref.data).toBe(2);
      expect(l2.top_ref.value).toBe(1);
    });
  });

  describe("level3 module", () => {
    test("Level3Struct exists", () => {
      const top: TopLevelStruct = { value: 1 };
      const l1: level1.Level1Struct = { data: 2, parent_ref: top };
      const l2: level1.level2.Level2Struct = {
        name: "l2",
        level1_ref: l1,
        top_ref: top,
      };
      const l3: level1.level2.level3.Level3Struct = {
        id: 100,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
      };
      expect(l3.id).toBe(100);
      expect(l3.level2_ref.name).toBe("l2");
      expect(l3.level1_ref.data).toBe(2);
      expect(l3.top_ref.value).toBe(1);
    });

    test("DEEP_CONST exists", () => {
      expect(level1.level2.level3.DEEP_CONST).toBe(42);
    });
  });

  describe("sibling module", () => {
    test("SiblingStruct exists", () => {
      const s: sibling.SiblingStruct = { id: 4 };
      expect(s.id).toBe(4);
    });

    test("CrossRef can reference types from sibling modules", () => {
      const top: TopLevelStruct = { value: 1 };
      const l1: level1.Level1Struct = { data: 2, parent_ref: top };
      const l2: level1.level2.Level2Struct = {
        name: "l2",
        level1_ref: l1,
        top_ref: top,
      };
      const l3: level1.level2.level3.Level3Struct = {
        id: 3,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
      };
      const cross: sibling.CrossRef = {
        from_level1: l1,
        from_level2: l2,
        from_level3: l3,
      };
      expect(cross.from_level1.data).toBe(2);
      expect(cross.from_level2.name).toBe("l2");
      expect(cross.from_level3.id).toBe(3);
    });
  });

  describe("TopUsingNested", () => {
    test("can reference deeply nested types", () => {
      const top: TopLevelStruct = { value: 1 };
      const l1: level1.Level1Struct = { data: 2, parent_ref: top };
      const l2: level1.level2.Level2Struct = {
        name: "l2",
        level1_ref: l1,
        top_ref: top,
      };
      const l3: level1.level2.level3.Level3Struct = {
        id: 3,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
      };
      const sib: sibling.SiblingStruct = { id: 4 };
      const using: TopUsingNested = { l1, l2, l3, sib };
      expect(using.l1.data).toBe(2);
      expect(using.l2.name).toBe("l2");
      expect(using.l3.id).toBe(3);
      expect(using.sib.id).toBe(4);
    });
  });
});
