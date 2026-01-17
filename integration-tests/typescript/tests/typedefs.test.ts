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
  Integer,
  UnsignedInteger,
  Real,
  Text,
  Flag,
  Byte,
  IntList,
  StringList,
  RealList,
  Count,
  Label,
  Level1,
  Level2,
  Level3,
  Level4,
  Level5,
  SeqLevel1,
  SeqLevel2,
  SeqLevel3,
  MapLevel1,
  MapLevel2,
  MapLevel3,
  StringIntMap,
  StringStringMap,
  LongArray,
  Point,
  Person,
  Container,
  Measurement,
  WithArrayTypedef,
  DeepChainStruct,
} from "../generated/typedefTypes";

describe("typedefs", () => {
  describe("primitive typedefs", () => {
    test("Integer is number", () => {
      const val: Integer = 42;
      expect(val).toBe(42);
    });

    test("UnsignedInteger is number", () => {
      const val: UnsignedInteger = 100;
      expect(val).toBe(100);
    });

    test("Real is number", () => {
      const val: Real = 3.14;
      expect(val).toBeCloseTo(3.14);
    });

    test("Text is string", () => {
      const val: Text = "hello";
      expect(val).toBe("hello");
    });

    test("Flag is boolean", () => {
      const val: Flag = true;
      expect(val).toBe(true);
    });

    test("Byte is number", () => {
      const val: Byte = 255;
      expect(val).toBe(255);
    });
  });

  describe("sequence typedefs", () => {
    test("IntList", () => {
      const val: IntList = [1, 2, 3];
      expect(val).toEqual([1, 2, 3]);
    });

    test("StringList", () => {
      const val: StringList = ["a", "b", "c"];
      expect(val).toEqual(["a", "b", "c"]);
    });

    test("RealList", () => {
      const val: RealList = [1.1, 2.2, 3.3];
      expect(val.length).toBe(3);
    });
  });

  describe("nested typedefs (typedef of typedef)", () => {
    test("Count is Integer is number", () => {
      const val: Count = 100;
      expect(val).toBe(100);
    });

    test("Label is Text is string", () => {
      const val: Label = "label";
      expect(val).toBe("label");
    });
  });

  describe("deep typedef chains", () => {
    test("5-level integer chain", () => {
      const l1: Level1 = 1;
      const l2: Level2 = 2;
      const l3: Level3 = 3;
      const l4: Level4 = 4;
      const l5: Level5 = 5;
      expect(l1 + l2 + l3 + l4 + l5).toBe(15);
    });

    test("3-level sequence chain", () => {
      const s1: SeqLevel1 = [1, 2, 3];
      const s2: SeqLevel2 = [4, 5, 6];
      const s3: SeqLevel3 = [7, 8, 9];
      expect(s1.length + s2.length + s3.length).toBe(9);
    });

    test("3-level map chain", () => {
      const m1: MapLevel1 = { a: 1 };
      const m2: MapLevel2 = { b: 2 };
      const m3: MapLevel3 = { c: 3 };
      expect(m1.a + m2.b + m3.c).toBe(6);
    });
  });

  describe("map typedefs", () => {
    test("StringIntMap", () => {
      const val: StringIntMap = { one: 1, two: 2 };
      expect(val.one).toBe(1);
    });

    test("StringStringMap", () => {
      const val: StringStringMap = { key: "value" };
      expect(val.key).toBe("value");
    });
  });

  describe("array typedef", () => {
    test("LongArray", () => {
      const val: LongArray = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
      expect(val.length).toBe(10);
    });
  });

  describe("structs using typedefs", () => {
    test("Point uses Real typedef", () => {
      const p: Point = { x: 1.5, y: 2.5 };
      expect(p.x).toBe(1.5);
      expect(p.y).toBe(2.5);
    });

    test("Person uses Text, Integer, Flag typedefs", () => {
      const person: Person = {
        name: "John",
        age: 30,
        active: true,
      };
      expect(person.name).toBe("John");
      expect(person.age).toBe(30);
      expect(person.active).toBe(true);
    });

    test("Container uses IntList, StringList, StringIntMap", () => {
      const container: Container = {
        numbers: [1, 2, 3],
        labels: ["a", "b"],
        lookup: { key: 42 },
      };
      expect(container.numbers.length).toBe(3);
      expect(container.labels.length).toBe(2);
      expect(container.lookup.key).toBe(42);
    });

    test("Measurement uses Label and Count", () => {
      const m: Measurement = {
        name: "temperature",
        value: 25,
      };
      expect(m.name).toBe("temperature");
      expect(m.value).toBe(25);
    });

    test("WithArrayTypedef", () => {
      const w: WithArrayTypedef = {
        values: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
      };
      expect(w.values.length).toBe(10);
    });

    test("DeepChainStruct uses deep typedef chains", () => {
      const d: DeepChainStruct = {
        deepInt: 42,
        deepSeq: [1, 2, 3],
        deepMap: { key: 100 },
      };
      expect(d.deepInt).toBe(42);
      expect(d.deepSeq).toEqual([1, 2, 3]);
      expect(d.deepMap.key).toBe(100);
    });
  });
});
