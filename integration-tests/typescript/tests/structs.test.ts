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
} from "@generated/struct_types";

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
      const rect: Rectangle = { top_left: tl, bottom_right: br };
      expect(rect.top_left.x).toBe(0);
      expect(rect.bottom_right.y).toBe(100);
    });
  });

  describe("AllPrimitives", () => {
    test("can hold all primitive types", () => {
      const p: AllPrimitives = {
        bool_val: true,
        byte_val: 255,
        short_val: -100,
        ushort_val: 1000,
        long_val: -50000,
        ulong_val: 100000,
        longlong_val: -9999999999,
        ulonglong_val: 9999999999,
        float_val: 3.14,
        double_val: 2.71828,
        string_val: "hello",
      };
      expect(p.bool_val).toBe(true);
      expect(p.byte_val).toBe(255);
      expect(p.short_val).toBe(-100);
      expect(p.ushort_val).toBe(1000);
      expect(p.long_val).toBe(-50000);
      expect(p.ulong_val).toBe(100000);
      expect(p.longlong_val).toBe(-9999999999);
      expect(p.ulonglong_val).toBe(9999999999);
      expect(p.float_val).toBeCloseTo(3.14);
      expect(p.double_val).toBeCloseTo(2.71828);
      expect(p.string_val).toBe("hello");
    });

    test("primitive types are correct JavaScript types", () => {
      const p: AllPrimitives = {
        bool_val: true,
        byte_val: 255,
        short_val: -100,
        ushort_val: 1000,
        long_val: -50000,
        ulong_val: 100000,
        longlong_val: -9999999999,
        ulonglong_val: 9999999999,
        float_val: 3.14,
        double_val: 2.71828,
        string_val: "hello",
      };
      expect(typeof p.bool_val).toBe("boolean");
      expect(typeof p.byte_val).toBe("number");
      expect(typeof p.short_val).toBe("number");
      expect(typeof p.ushort_val).toBe("number");
      expect(typeof p.long_val).toBe("number");
      expect(typeof p.ulong_val).toBe("number");
      expect(typeof p.longlong_val).toBe("number");
      expect(typeof p.ulonglong_val).toBe("number");
      expect(typeof p.float_val).toBe("number");
      expect(typeof p.double_val).toBe("number");
      expect(typeof p.string_val).toBe("string");
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
      const s: WithArray = { fixed_numbers: [1, 2, 3, 4, 5] };
      expect(s.fixed_numbers.length).toBe(5);
      expect(s.fixed_numbers[0]).toBe(1);
    });
  });

  describe("WithMap", () => {
    test("can hold maps (Records)", () => {
      const s: WithMap = { string_to_int: { one: 1, two: 2 } };
      expect(s.string_to_int.one).toBe(1);
      expect(s.string_to_int.two).toBe(2);
    });
  });

  describe("Empty struct", () => {
    test("can be instantiated", () => {
      const e: Empty = {};
      expect(e).toBeDefined();
    });
  });
});
