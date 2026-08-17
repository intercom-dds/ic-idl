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
  CHAR_A,
  CHAR_DIGIT,
  CHAR_SPACE,
  CHAR_NEWLINE,
  CHAR_TAB,
  CHAR_QUOTE,
  CHAR_BACKSLASH,
  WCHAR_A,
  WCHAR_OMEGA,
  WCHAR_CHINESE,
  WSTRING_HELLO,
  WSTRING_UNICODE,
  WSTRING_EMOJI,
  WSTRING_EMPTY,
} from "@generated/char_wstring_types";
import type {
  CharFields,
  WstringFields,
  CharSequences,
  MixedCharTypes,
} from "@generated/char_wstring_types";
import {
  FRENCH,
  GERMAN,
  SPANISH,
  NORWEGIAN,
  SWEDISH,
  GREEK,
  RUSSIAN,
  CHINESE,
  JAPANESE,
  KOREAN,
  EMOJI,
  MIXED,
  QUOTES,
  BACKSLASH,
  NEWLINE,
  TAB,
  EMPTY,
  SPACES,
  UNICODE_SPACES,
  MATH,
  CURRENCY,
  ARROWS,
} from "@generated/unicode_types";
import type { UnicodeData } from "@generated/unicode_types";

describe("char and wstring types", () => {
  describe("char constants", () => {
    test("char letter", () => {
      expect(CHAR_A).toBe("A");
    });

    test("char digit", () => {
      expect(CHAR_DIGIT).toBe("5");
    });

    test("char space", () => {
      expect(CHAR_SPACE).toBe(" ");
    });

    test("char newline", () => {
      expect(CHAR_NEWLINE).toBe("\n");
    });

    test("char tab", () => {
      expect(CHAR_TAB).toBe("\t");
    });

    test("char quote", () => {
      expect(CHAR_QUOTE).toBe("'");
    });

    test("char backslash", () => {
      expect(CHAR_BACKSLASH).toBe("\\");
    });
  });

  describe("wchar constants", () => {
    test("wchar ASCII", () => {
      expect(WCHAR_A).toBe("A");
    });

    test("wchar omega", () => {
      expect(WCHAR_OMEGA).toBe("Ω");
      expect(WCHAR_OMEGA).toBe(String.fromCharCode(937));
    });

    test("wchar Chinese", () => {
      expect(WCHAR_CHINESE).toBe("中");
      expect(WCHAR_CHINESE).toBe(String.fromCharCode(20013));
    });
  });

  describe("wstring constants", () => {
    test("wstring hello", () => {
      expect(WSTRING_HELLO).toBe("Hello");
    });

    test("wstring unicode", () => {
      expect(WSTRING_UNICODE).toBe("日本語テスト");
    });

    test("wstring emoji", () => {
      expect(WSTRING_EMOJI).toBe("🎉🚀");
    });

    test("wstring empty", () => {
      expect(WSTRING_EMPTY).toBe("");
    });
  });

  describe("char structs", () => {
    test("CharFields", () => {
      const cf: CharFields = { single_char: "X", wide_char: "Ω" };
      expect(cf.single_char).toBe("X");
      expect(cf.wide_char).toBe("Ω");
    });

    test("WstringFields", () => {
      const ws: WstringFields = { wide_text: "日本語", narrow_text: "ASCII" };
      expect(ws.wide_text).toBe("日本語");
      expect(ws.narrow_text).toBe("ASCII");
    });

    test("CharSequences", () => {
      const cs: CharSequences = {
        char_seq: ["a", "b", "c"],
        wchar_seq: ["α", "β", "γ"],
      };
      expect(cs.char_seq).toEqual(["a", "b", "c"]);
      expect(cs.wchar_seq).toEqual(["α", "β", "γ"]);
    });

    test("MixedCharTypes", () => {
      const m: MixedCharTypes = {
        letter: "A",
        wide_letter: "Ω",
        text: "hello",
        wide_text: "世界",
      };
      expect(m.letter).toBe("A");
      expect(m.wide_letter).toBe("Ω");
      expect(m.text).toBe("hello");
      expect(m.wide_text).toBe("世界");
    });
  });
});

describe("unicode strings", () => {
  describe("latin with accents", () => {
    test("French", () => {
      expect(FRENCH).toBe("café résumé naïve");
    });

    test("German", () => {
      expect(GERMAN).toBe("größe über müde");
    });

    test("Spanish", () => {
      expect(SPANISH).toBe("señor mañana niño");
    });

    test("Norwegian", () => {
      expect(NORWEGIAN).toBe("blåbær ærlig øl");
    });

    test("Swedish", () => {
      expect(SWEDISH).toBe("smörgås älg ö");
    });
  });

  describe("non-latin scripts", () => {
    test("Greek", () => {
      expect(GREEK).toBe("αβγδ Ωμέγα");
    });

    test("Russian", () => {
      expect(RUSSIAN).toBe("привет мир");
    });

    test("Chinese", () => {
      expect(CHINESE).toBe("你好世界");
    });

    test("Japanese", () => {
      expect(JAPANESE).toBe("こんにちは世界");
    });

    test("Korean", () => {
      expect(KOREAN).toBe("안녕하세요");
    });

    test("Emoji", () => {
      expect(EMOJI).toBe("🎉🚀💻🔥");
    });

    test("Mixed scripts", () => {
      expect(MIXED).toBe("Hello 世界 🌍 café");
    });
  });

  describe("escape sequences", () => {
    test("quotes", () => {
      expect(QUOTES).toBe('He said "hello"');
    });

    test("backslash", () => {
      expect(BACKSLASH).toBe("path\\to\\file");
    });

    test("newline", () => {
      expect(NEWLINE).toBe("line1\nline2");
    });

    test("tab", () => {
      expect(TAB).toBe("col1\tcol2");
    });
  });

  describe("whitespace", () => {
    test("empty string", () => {
      expect(EMPTY).toBe("");
      expect(EMPTY.length).toBe(0);
    });

    test("spaces", () => {
      expect(SPACES).toBe("   ");
      expect(SPACES.length).toBe(3);
    });

    test("unicode spaces (including non-breaking space)", () => {
      expect(UNICODE_SPACES.length).toBe(3);
      expect(UNICODE_SPACES.charCodeAt(1)).toBe(0xa0);
    });
  });

  describe("symbols", () => {
    test("math symbols", () => {
      expect(MATH).toBe("∑∏∫√∞≠≤≥");
    });

    test("currency symbols", () => {
      expect(CURRENCY).toBe("$ € £ ¥ ₹ ₽");
    });

    test("arrows", () => {
      expect(ARROWS).toBe("← → ↑ ↓ ↔ ⇒");
    });
  });

  describe("UnicodeData struct", () => {
    test("can hold unicode strings", () => {
      const data: UnicodeData = {
        label: "日本語",
        description: "Japanese text with emoji 🇯🇵",
      };
      expect(data.label).toBe("日本語");
      expect(data.description).toContain("🇯🇵");
    });
  });
});
