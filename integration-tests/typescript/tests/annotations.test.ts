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
} from "../generated/annotationTypes";

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
      requiredField: 100,
    };
    expect(minimal.requiredField).toBe(100);
    expect(minimal.optionalInt).toBeUndefined();

    const full: OptionalStruct = {
      requiredField: 100,
      optionalInt: 42,
      optionalString: "hello",
      optionalSeq: [1, 2, 3],
    };
    expect(full.optionalInt).toBe(42);
    expect(full.optionalString).toBe("hello");
    expect(full.optionalSeq).toEqual([1, 2, 3]);
  });

  test("NestedStruct can be used in other structs", () => {
    const nested: NestedStruct = { x: 10, y: 20 };
    const shared: SharedRefs = {
      sharedString: "test",
      sharedStruct: nested,
    };
    expect(shared.sharedStruct.x).toBe(10);
    expect(shared.sharedStruct.y).toBe(20);
  });

  test("CombinedAnnotations has key and optional fields", () => {
    const obj: CombinedAnnotations = {
      id: 1,
    };
    expect(obj.id).toBe(1);
    expect(obj.maybeSharedName).toBeUndefined();

    const withOptional: CombinedAnnotations = {
      id: 2,
      maybeSharedName: "shared",
    };
    expect(withOptional.maybeSharedName).toBe("shared");
  });

  test("TopicMessage has all required fields", () => {
    const msg: TopicMessage = {
      messageId: 12345,
      payload: "data",
      timestamp: Date.now(),
    };
    expect(msg.messageId).toBe(12345);
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
      fixedField: 999,
    };
    expect(obj.fixedField).toBe(999);
  });

  test("AnnotatedInterface can be implemented", () => {
    const impl: AnnotatedInterface = {
      fireAndForget: (_message: string) => {},
      getValue: () => 42,
      setValue: (_value: number) => {},
    };
    expect(impl.getValue()).toBe(42);
  });
});
