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
  Point,
  Point3D,
  Point4D,
  Rectangle,
  AllPrimitives,
  WithSequence,
  WithArray,
  WithMap,
  Empty,
} from "../generated/structTypes";

describe("structs", () => {
  describe("Point", () => {
    test("instantiation", () => {
      const p: Point = { x: 10, y: 20 };
      expect(p.x).toBe(10);
      expect(p.y).toBe(20);
    });

    test("field modification", () => {
      const p: Point = { x: 5, y: 10 };
      p.x = 100;
      p.y = 200;
      expect(p.x).toBe(100);
      expect(p.y).toBe(200);
    });
  });

  describe("Point3D inheritance", () => {
    test("has all fields from Point plus z", () => {
      const p3d: Point3D = { x: 1, y: 2, z: 3 };
      expect(p3d.x).toBe(1);
      expect(p3d.y).toBe(2);
      expect(p3d.z).toBe(3);
    });

    test("can be used where Point is expected", () => {
      const p3d: Point3D = { x: 1, y: 2, z: 3 };
      const acceptsPoint = (p: Point) => p.x + p.y;
      expect(acceptsPoint(p3d)).toBe(3);
    });
  });

  describe("Point4D multi-level inheritance", () => {
    test("has all fields from Point, Point3D, plus w", () => {
      const p4d: Point4D = { x: 1, y: 2, z: 3, w: 4 };
      expect(p4d.x).toBe(1);
      expect(p4d.y).toBe(2);
      expect(p4d.z).toBe(3);
      expect(p4d.w).toBe(4);
    });

    test("can be used where Point3D is expected", () => {
      const p4d: Point4D = { x: 1, y: 2, z: 3, w: 4 };
      const acceptsPoint3D = (p: Point3D) => p.x + p.y + p.z;
      expect(acceptsPoint3D(p4d)).toBe(6);
    });

    test("can be used where Point is expected", () => {
      const p4d: Point4D = { x: 1, y: 2, z: 3, w: 4 };
      const acceptsPoint = (p: Point) => p.x + p.y;
      expect(acceptsPoint(p4d)).toBe(3);
    });
  });

  describe("nested struct (Rectangle)", () => {
    test("can contain other structs", () => {
      const tl: Point = { x: 0, y: 0 };
      const br: Point = { x: 100, y: 100 };
      const rect: Rectangle = { topLeft: tl, bottomRight: br };
      expect(rect.topLeft.x).toBe(0);
      expect(rect.bottomRight.y).toBe(100);
    });
  });

  describe("AllPrimitives", () => {
    test("can hold all primitive types", () => {
      const p: AllPrimitives = {
        boolVal: true,
        byteVal: 255,
        shortVal: -100,
        ushortVal: 1000,
        longVal: -50000,
        ulongVal: 100000,
        longlongVal: -9999999999,
        ulonglongVal: 9999999999,
        floatVal: 3.14,
        doubleVal: 2.71828,
        stringVal: "hello",
      };
      expect(p.boolVal).toBe(true);
      expect(p.byteVal).toBe(255);
      expect(p.shortVal).toBe(-100);
      expect(p.ushortVal).toBe(1000);
      expect(p.longVal).toBe(-50000);
      expect(p.ulongVal).toBe(100000);
      expect(p.longlongVal).toBe(-9999999999);
      expect(p.ulonglongVal).toBe(9999999999);
      expect(p.floatVal).toBeCloseTo(3.14);
      expect(p.doubleVal).toBeCloseTo(2.71828);
      expect(p.stringVal).toBe("hello");
    });

    test("primitive types are correct JavaScript types", () => {
      const p: AllPrimitives = {
        boolVal: true,
        byteVal: 255,
        shortVal: -100,
        ushortVal: 1000,
        longVal: -50000,
        ulongVal: 100000,
        longlongVal: -9999999999,
        ulonglongVal: 9999999999,
        floatVal: 3.14,
        doubleVal: 2.71828,
        stringVal: "hello",
      };
      expect(typeof p.boolVal).toBe("boolean");
      expect(typeof p.byteVal).toBe("number");
      expect(typeof p.shortVal).toBe("number");
      expect(typeof p.ushortVal).toBe("number");
      expect(typeof p.longVal).toBe("number");
      expect(typeof p.ulongVal).toBe("number");
      expect(typeof p.longlongVal).toBe("number");
      expect(typeof p.ulonglongVal).toBe("number");
      expect(typeof p.floatVal).toBe("number");
      expect(typeof p.doubleVal).toBe("number");
      expect(typeof p.stringVal).toBe("string");
    });
  });

  describe("WithSequence", () => {
    test("can hold sequences (arrays)", () => {
      const s: WithSequence = { numbers: [1, 2, 3], names: ["a", "b"] };
      expect(s.numbers).toEqual([1, 2, 3]);
      expect(s.names).toEqual(["a", "b"]);
    });
  });

  describe("WithArray", () => {
    test("can hold fixed arrays", () => {
      const s: WithArray = { fixedNumbers: [1, 2, 3, 4, 5] };
      expect(s.fixedNumbers.length).toBe(5);
      expect(s.fixedNumbers[0]).toBe(1);
    });
  });

  describe("WithMap", () => {
    test("can hold maps (Records)", () => {
      const s: WithMap = { stringToInt: { one: 1, two: 2 } };
      expect(s.stringToInt.one).toBe(1);
      expect(s.stringToInt.two).toBe(2);
    });
  });

  describe("Empty struct", () => {
    test("can be instantiated", () => {
      const e: Empty = {};
      expect(e).toBeDefined();
    });
  });
});
