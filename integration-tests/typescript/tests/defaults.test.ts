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
import {
  DEFAULT_NAME,
  DEFAULT_COUNT,
  DEFAULT_RATE,
  DEFAULT_INNER,
  NESTED_INNER,
  Priority,
} from "../generated/defaultTypes";
import type { Inner, OptionalFields } from "../generated/defaultTypes";

describe("defaults", () => {
  describe("constants", () => {
    test("string constant values", () => {
      expect(DEFAULT_NAME).toBe("unnamed");
    });

    test("numeric constant values", () => {
      expect(DEFAULT_COUNT).toBe(100);
      expect(DEFAULT_RATE).toBeCloseTo(0.5);
    });

    test("struct const initializer", () => {
      expect(DEFAULT_INNER.x).toBe(10);
      expect(DEFAULT_INNER.y).toBe("default");
      expect(NESTED_INNER.x).toBe(99);
      expect(NESTED_INNER.y).toBe("nested");
    });
  });

  describe("optional fields", () => {
    test("optional fields can be omitted", () => {
      const opt: OptionalFields = {};
      expect(opt.maybeInt).toBeUndefined();
      expect(opt.maybeString).toBeUndefined();
      expect(opt.maybeStruct).toBeUndefined();
    });

    test("optional fields can be set", () => {
      const inner: Inner = { x: 5, y: "test" };
      const opt: OptionalFields = {
        maybeInt: 42,
        maybeString: "hello",
        maybeStruct: inner,
      };
      expect(opt.maybeInt).toBe(42);
      expect(opt.maybeString).toBe("hello");
      expect(opt.maybeStruct?.x).toBe(5);
    });
  });

  describe("enums", () => {
    test("Priority enum exists with default_literal", () => {
      expect(Priority.Low).toBe(0);
      expect(Priority.Medium).toBe(1);
      expect(Priority.High).toBe(2);
    });
  });
});
