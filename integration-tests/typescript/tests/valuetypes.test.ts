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
  Identifiable,
  Named,
  SimpleValue,
  DerivedValue,
  Empty,
  WithSequence,
  IdentifiableValue,
  NamedValue,
  FullValue,
  ValueWithPrivate,
} from "@generated/valuetype_types";

describe("valuetypes", () => {
  describe("SimpleValue", () => {
    test("instantiation", () => {
      const v: SimpleValue = { id: 1, name: "test" };
      expect(v.id).toBe(1);
      expect(v.name).toBe("test");
    });
  });

  describe("DerivedValue inheritance", () => {
    test("has fields from SimpleValue plus description", () => {
      const v: DerivedValue = { id: 1, name: "base", description: "derived" };
      expect(v.id).toBe(1);
      expect(v.name).toBe("base");
      expect(v.description).toBe("derived");
    });

    test("can be used where SimpleValue is expected", () => {
      const v: DerivedValue = { id: 1, name: "base", description: "derived" };
      const acceptsSimple = (s: SimpleValue) => s.id + s.name.length;
      expect(acceptsSimple(v)).toBe(5);
    });
  });

  describe("Empty valuetype", () => {
    test("can be instantiated", () => {
      const v: Empty = {};
      expect(v).toBeDefined();
    });
  });

  describe("WithSequence", () => {
    test("can hold sequences", () => {
      const v: WithSequence = { numbers: [1, 2, 3], names: ["a", "b"] };
      expect(v.numbers).toEqual([1, 2, 3]);
      expect(v.names).toEqual(["a", "b"]);
    });
  });

  describe("valuetype supports interface", () => {
    test("IdentifiableValue implements Identifiable", () => {
      const v: IdentifiableValue = { id: 42, data: "test" };
      expect(v.id).toBe(42);
      expect(v.data).toBe("test");
      const acceptsIdentifiable = (i: Identifiable) => i.id;
      expect(acceptsIdentifiable(v)).toBe(42);
    });

    test("NamedValue implements Named", () => {
      const v: NamedValue = { name: "foo", value: 100 };
      expect(v.name).toBe("foo");
      expect(v.value).toBe(100);
      const acceptsNamed = (n: Named) => n.name;
      expect(acceptsNamed(v)).toBe("foo");
    });
  });

  describe("valuetype inheritance and supports", () => {
    test("FullValue extends SimpleValue and implements Identifiable", () => {
      const v: FullValue = { id: 1, name: "base", extra: "more" };
      expect(v.id).toBe(1);
      expect(v.name).toBe("base");
      expect(v.extra).toBe("more");
      const acceptsSimple = (s: SimpleValue) => s.id;
      const acceptsIdentifiable = (i: Identifiable) => i.id;
      expect(acceptsSimple(v)).toBe(1);
      expect(acceptsIdentifiable(v)).toBe(1);
    });
  });

  describe("ValueWithPrivate", () => {
    test("private state members are accessible as regular fields", () => {
      const v: ValueWithPrivate = { label: "test", internal_id: 123 };
      expect(v.label).toBe("test");
      expect(v.internal_id).toBe(123);
    });
  });
});
