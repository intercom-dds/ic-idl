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
  ShortString,
  MediumString,
  LongString,
  SmallIntList,
  StringList100,
  LargeDoubleList,
  BoundedFields,
  NestedBounded,
  Name,
  NameList,
  NameMap,
  MixedBounds,
} from "../generated/boundedTypes";

describe("bounded types", () => {
  describe("bounded string typedefs", () => {
    test("ShortString is string", () => {
      const s: ShortString = "test";
      expect(s).toBe("test");
    });

    test("MediumString is string", () => {
      const s: MediumString = "medium length string";
      expect(s).toBe("medium length string");
    });

    test("LongString is string", () => {
      const s: LongString = "a".repeat(4096);
      expect(s.length).toBe(4096);
    });
  });

  describe("bounded sequence typedefs", () => {
    test("SmallIntList is number array", () => {
      const list: SmallIntList = [1, 2, 3, 4, 5];
      expect(list).toEqual([1, 2, 3, 4, 5]);
    });

    test("StringList100 is string array", () => {
      const list: StringList100 = ["a", "b", "c"];
      expect(list).toEqual(["a", "b", "c"]);
    });

    test("LargeDoubleList is number array", () => {
      const list: LargeDoubleList = [1.1, 2.2, 3.3];
      expect(list).toEqual([1.1, 2.2, 3.3]);
    });
  });

  describe("bounded fields struct", () => {
    test("BoundedFields can be created", () => {
      const s: BoundedFields = {
        name: "test",
        description: "A longer description",
        values: [1, 2, 3],
        tags: ["a", "b"],
      };
      expect(s.name).toBe("test");
      expect(s.description).toBe("A longer description");
      expect(s.values).toEqual([1, 2, 3]);
      expect(s.tags).toEqual(["a", "b"]);
    });
  });

  describe("nested bounded struct", () => {
    test("NestedBounded can be created", () => {
      const s: NestedBounded = {
        matrix: [
          [1, 2],
          [3, 4],
        ],
        indexedLists: { a: [1, 2, 3], b: [4, 5, 6] },
      };
      expect(s.matrix).toEqual([
        [1, 2],
        [3, 4],
      ]);
      expect(s.indexedLists.a).toEqual([1, 2, 3]);
    });
  });

  describe("typedef chain with bounds", () => {
    test("Name is string", () => {
      const n: Name = "test name";
      expect(n).toBe("test name");
    });

    test("NameList is string array", () => {
      const list: NameList = ["name1", "name2"];
      expect(list).toEqual(["name1", "name2"]);
    });

    test("NameMap is record of string to string array", () => {
      const map: NameMap = {
        group1: ["name1", "name2"],
        group2: ["name3"],
      };
      expect(map.group1).toEqual(["name1", "name2"]);
    });
  });

  describe("mixed bounds struct", () => {
    test("MixedBounds can be created", () => {
      const s: MixedBounds = {
        boundedString: "bounded",
        unboundedString: "unbounded".repeat(100),
        boundedSeq: [1, 2, 3],
        unboundedSeq: Array.from({ length: 1000 }, (_, i) => i),
      };
      expect(s.boundedString).toBe("bounded");
      expect(s.unboundedString.length).toBe(900);
      expect(s.boundedSeq).toEqual([1, 2, 3]);
      expect(s.unboundedSeq.length).toBe(1000);
    });
  });

  describe("bounds not enforced at runtime", () => {
    test("values exceeding bounds are accepted", () => {
      const s: BoundedFields = {
        name: "x".repeat(1000),
        description: "y".repeat(10000),
        values: Array.from({ length: 1000 }, (_, i) => i),
        tags: Array.from({ length: 500 }, () => "tag"),
      };
      expect(s.name.length).toBe(1000);
      expect(s.description.length).toBe(10000);
      expect(s.values.length).toBe(1000);
      expect(s.tags.length).toBe(500);
    });
  });
});
