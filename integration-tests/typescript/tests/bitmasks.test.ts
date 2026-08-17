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
  Permissions,
  ExplicitFlags,
  GappedFlags,
  SingleFlag,
  MixedFlags,
  type FileInfo,
} from "@generated/bitmask_types";

describe("bitmasks", () => {
  test("Permissions has power-of-2 values", () => {
    expect(Permissions.READ).toBe(1);
    expect(Permissions.WRITE).toBe(2);
    expect(Permissions.EXECUTE).toBe(4);
    expect(Permissions.DELETE).toBe(8);
  });

  test("Permissions can be combined with bitwise OR", () => {
    const readWrite = Permissions.READ | Permissions.WRITE;
    expect(readWrite).toBe(3);
    expect(readWrite & Permissions.READ).toBe(Permissions.READ);
    expect(readWrite & Permissions.WRITE).toBe(Permissions.WRITE);
    expect(readWrite & Permissions.EXECUTE).toBe(0);
  });

  test("FileInfo struct uses Permissions", () => {
    const file: FileInfo = {
      path: "/tmp/test.txt",
      perms: Permissions.READ | Permissions.WRITE,
    };
    expect(file.path).toBe("/tmp/test.txt");
    expect(file.perms & Permissions.READ).toBeTruthy();
  });

  test("ExplicitFlags has explicit values", () => {
    expect(ExplicitFlags.FLAG_A).toBe(2);
    expect(ExplicitFlags.FLAG_B).toBe(4);
    expect(ExplicitFlags.FLAG_C).toBe(16);
    expect(ExplicitFlags.FLAG_D).toBe(256);
  });

  test("GappedFlags has non-contiguous values", () => {
    expect(GappedFlags.LOW).toBe(1);
    expect(GappedFlags.HIGH).toBe(128);
  });

  test("SingleFlag has one value", () => {
    expect(SingleFlag.ONLY).toBe(1);
  });

  test("MixedFlags has mixed auto and explicit values", () => {
    expect(MixedFlags.AUTO_FIRST).toBe(1);
    expect(MixedFlags.EXPLICIT_FOUR).toBe(16);
    expect(MixedFlags.AUTO_FIVE).toBe(32);
    expect(MixedFlags.AUTO_SIX).toBe(64);
  });
});
