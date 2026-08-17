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
} from "@generated/enum_types";

describe("enums", () => {
  test("Color enum has correct members", () => {
    expect(Color.RED).toBe(0);
    expect(Color.GREEN).toBe(1);
    expect(Color.BLUE).toBe(2);
  });

  test("Status enum has explicit values", () => {
    expect(Status.OK).toBe(0);
    expect(Status.WARNING).toBe(100);
    expect(Status.ERROR).toBe(200);
  });

  test("GappedEnum has non-sequential values", () => {
    expect(GappedEnum.FIRST).toBe(0);
    expect(GappedEnum.SECOND).toBe(5);
    expect(GappedEnum.THIRD).toBe(10);
    expect(GappedEnum.FOURTH).toBe(100);
  });

  test("NegativeEnum supports negative values", () => {
    expect(NegativeEnum.NEG_TWO).toBe(-2);
    expect(NegativeEnum.NEG_ONE).toBe(-1);
    expect(NegativeEnum.ZERO).toBe(0);
    expect(NegativeEnum.POS_ONE).toBe(1);
  });

  test("MixedEnum has mixed auto and explicit values", () => {
    expect(MixedEnum.AUTO_FIRST).toBe(0);
    expect(MixedEnum.EXPLICIT_TEN).toBe(10);
    expect(MixedEnum.AUTO_ELEVEN).toBe(11);
    expect(MixedEnum.EXPLICIT_HUNDRED).toBe(100);
    expect(MixedEnum.AUTO_HUNDRED_ONE).toBe(101);
  });

  test("ENUM_CONST references enum value", () => {
    expect(ENUM_CONST).toBe(Status.WARNING);
  });
});
