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
  ContainsAny,
  MultipleAny,
  AnyWithOtherFields,
  SequenceOfAny,
  MapWithAny,
  OptionalAny,
  AnyAlias,
  UsingAnyAlias,
} from "../generated/anyTypes";

describe("any type", () => {
  test("any accepts undefined", () => {
    const c: ContainsAny = { value: undefined };
    expect(c.value).toBeUndefined();
  });

  test("any accepts int", () => {
    const c: ContainsAny = { value: 42 };
    expect(c.value).toBe(42);
  });

  test("any accepts string", () => {
    const c: ContainsAny = { value: "hello" };
    expect(c.value).toBe("hello");
  });

  test("any accepts list", () => {
    const c: ContainsAny = { value: [1, 2, 3] };
    expect(c.value).toEqual([1, 2, 3]);
  });

  test("any accepts dict", () => {
    const c: ContainsAny = { value: { key: "value" } };
    expect(c.value).toEqual({ key: "value" });
  });

  test("any accepts nested struct", () => {
    const inner: ContainsAny = { value: "nested" };
    const outer: ContainsAny = { value: inner };
    expect((outer.value as ContainsAny).value).toBe("nested");
  });

  test("multiple any fields", () => {
    const m: MultipleAny = {
      first: 1,
      second: "two",
      third: [3.0],
    };
    expect(m.first).toBe(1);
    expect(m.second).toBe("two");
    expect(m.third).toEqual([3.0]);
  });

  test("any with other fields", () => {
    const a: AnyWithOtherFields = {
      id: 123,
      name: "test",
      payload: { data: [1, 2, 3] },
    };
    expect(a.id).toBe(123);
    expect(a.name).toBe("test");
    expect(a.payload).toEqual({ data: [1, 2, 3] });
  });

  test("sequence of any", () => {
    const s: SequenceOfAny = {
      items: [1, "two", 3.0, null, { key: "value" }],
    };
    expect(s.items.length).toBe(5);
    expect(s.items[1]).toBe("two");
    expect(s.items[4]).toEqual({ key: "value" });
  });

  test("map with any", () => {
    const m: MapWithAny = {
      properties: {
        int: 1,
        str: "hello",
        list: [1, 2, 3],
      },
    };
    expect(m.properties.int).toBe(1);
    expect(m.properties.str).toBe("hello");
    expect(m.properties.list).toEqual([1, 2, 3]);
  });

  test("optional any default", () => {
    const o: OptionalAny = {};
    expect(o.maybeValue).toBeUndefined();
  });

  test("optional any with value", () => {
    const o: OptionalAny = { maybeValue: { nested: "data" } };
    expect(o.maybeValue).toEqual({ nested: "data" });
  });

  test("any alias typedef exists", () => {
    const val: AnyAlias = "anything";
    expect(val).toBe("anything");
  });

  test("using any alias", () => {
    const u: UsingAnyAlias = { data: "aliased value" };
    expect(u.data).toBe("aliased value");
  });

  test("any can be reassigned", () => {
    const c: ContainsAny = { value: 1 };
    expect(c.value).toBe(1);
    c.value = "now a string";
    expect(c.value).toBe("now a string");
    c.value = [1, 2, 3];
    expect(c.value).toEqual([1, 2, 3]);
  });
});
