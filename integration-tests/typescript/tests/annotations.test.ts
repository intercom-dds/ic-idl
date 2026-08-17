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
  KeyedStruct,
  MultiKeyStruct,
  OptionalStruct,
  NestedStruct,
  SharedRefs,
  CombinedAnnotations,
  AnnotatedInterface,
  TopicMessage,
  MutableStruct,
  FinalStruct,
} from "@generated/annotation_types";

describe("annotations", () => {
  test("KeyedStruct can be instantiated", () => {
    const obj: KeyedStruct = {
      id: 42,
      name: "test",
      value: 3.14,
    };
    expect(obj.id).toBe(42);
    expect(obj.name).toBe("test");
    expect(obj.value).toBe(3.14);
  });

  test("MultiKeyStruct handles reserved word renaming", () => {
    const obj: MultiKeyStruct = {
      namespace_: "ns",
      id: 1,
      data: "payload",
    };
    expect(obj.namespace_).toBe("ns");
  });

  test("OptionalStruct supports optional fields", () => {
    const minimal: OptionalStruct = {
      required_field: 100,
    };
    expect(minimal.required_field).toBe(100);
    expect(minimal.optional_int).toBeUndefined();

    const full: OptionalStruct = {
      required_field: 100,
      optional_int: 42,
      optional_string: "hello",
      optional_seq: [1, 2, 3],
    };
    expect(full.optional_int).toBe(42);
    expect(full.optional_string).toBe("hello");
    expect(full.optional_seq).toEqual([1, 2, 3]);
  });

  test("NestedStruct can be used in other structs", () => {
    const nested: NestedStruct = { x: 10, y: 20 };
    const shared: SharedRefs = {
      shared_string: "test",
      shared_struct: nested,
    };
    expect(shared.shared_struct.x).toBe(10);
    expect(shared.shared_struct.y).toBe(20);
  });

  test("CombinedAnnotations has key and optional fields", () => {
    const obj: CombinedAnnotations = {
      id: 1,
    };
    expect(obj.id).toBe(1);
    expect(obj.maybe_shared_name).toBeUndefined();

    const withOptional: CombinedAnnotations = {
      id: 2,
      maybe_shared_name: "shared",
    };
    expect(withOptional.maybe_shared_name).toBe("shared");
  });

  test("TopicMessage has all required fields", () => {
    const msg: TopicMessage = {
      message_id: 12345,
      payload: "data",
      timestamp: Date.now(),
    };
    expect(msg.message_id).toBe(12345);
    expect(msg.payload).toBe("data");
  });

  test("MutableStruct is a normal struct", () => {
    const obj: MutableStruct = {
      version: 1,
      data: "content",
    };
    expect(obj.version).toBe(1);
  });

  test("FinalStruct is a normal struct", () => {
    const obj: FinalStruct = {
      fixed_field: 999,
    };
    expect(obj.fixed_field).toBe(999);
  });

  test("AnnotatedInterface can be implemented", () => {
    const impl: AnnotatedInterface = {
      fire_and_forget: (_message: string) => {},
      get_value: () => 42,
      set_value: (_value: number) => {},
    };
    expect(impl.get_value()).toBe(42);
  });
});
