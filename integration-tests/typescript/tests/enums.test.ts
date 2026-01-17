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
  Color,
  Status,
  GappedEnum,
  NegativeEnum,
  MixedEnum,
  ENUM_CONST,
} from "../generated/enumTypes";

describe("enums", () => {
  test("Color enum has correct members", () => {
    expect(Color.Red).toBe(0);
    expect(Color.Green).toBe(1);
    expect(Color.Blue).toBe(2);
  });

  test("Status enum has explicit values", () => {
    expect(Status.Ok).toBe(0);
    expect(Status.Warning).toBe(100);
    expect(Status.Error).toBe(200);
  });

  test("GappedEnum has non-sequential values", () => {
    expect(GappedEnum.First).toBe(0);
    expect(GappedEnum.Second).toBe(5);
    expect(GappedEnum.Third).toBe(10);
    expect(GappedEnum.Fourth).toBe(100);
  });

  test("NegativeEnum supports negative values", () => {
    expect(NegativeEnum.NegTwo).toBe(-2);
    expect(NegativeEnum.NegOne).toBe(-1);
    expect(NegativeEnum.Zero).toBe(0);
    expect(NegativeEnum.PosOne).toBe(1);
  });

  test("MixedEnum has mixed auto and explicit values", () => {
    expect(MixedEnum.AutoFirst).toBe(0);
    expect(MixedEnum.ExplicitTen).toBe(10);
    expect(MixedEnum.AutoEleven).toBe(11);
    expect(MixedEnum.ExplicitHundred).toBe(100);
    expect(MixedEnum.AutoHundredOne).toBe(101);
  });

  test("ENUM_CONST references enum value", () => {
    expect(ENUM_CONST).toBe(Status.Warning);
  });
});
