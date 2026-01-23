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
  SimpleError,
  EmptyError,
  DetailedError,
  ValidationError,
} from "../generated/exception_types";

describe("exceptions", () => {
  test("exception inherits from Error", () => {
    const e = new SimpleError(404, "Not found");
    expect(e instanceof Error).toBe(true);
  });

  test("exception instantiation", () => {
    const e = new SimpleError(404, "Not found");
    expect(e.error_code).toBe(404);
    expect(e.message).toBe("Not found");
  });

  test("exception raise and catch", () => {
    let caught: SimpleError | null = null;
    try {
      throw new SimpleError(500, "Internal error");
    } catch (e) {
      if (e instanceof SimpleError) {
        caught = e;
      }
    }
    expect(caught).not.toBeNull();
    expect(caught?.error_code).toBe(500);
    expect(caught?.message).toBe("Internal error");
  });

  test("exception catch as base Error", () => {
    let caught = false;
    try {
      throw new SimpleError(400, "Bad request");
    } catch (e) {
      if (e instanceof Error) {
        caught = true;
      }
    }
    expect(caught).toBe(true);
  });

  test("empty exception", () => {
    const e = new EmptyError();
    expect(e instanceof Error).toBe(true);
    expect(e.name).toBe("EmptyError");
  });

  test("detailed exception fields", () => {
    const e = new DetailedError(
      123,
      "Something went wrong",
      "Additional context here",
      true,
    );
    expect(e.code).toBe(123);
    expect(e.message).toBe("Something went wrong");
    expect(e.details).toBe("Additional context here");
    expect(e.recoverable).toBe(true);
  });

  test("validation error", () => {
    const e = new ValidationError("email", "Invalid email format", 10);
    expect(e.field_name).toBe("email");
    expect(e.error_message).toBe("Invalid email format");
    expect(e.position).toBe(10);
  });
});
