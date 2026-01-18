# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

from types import ModuleType


def test_char_letter(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_A == "A"


def test_char_digit(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_DIGIT == "5"


def test_char_space(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_SPACE == " "


def test_char_newline(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_NEWLINE == "\n"


def test_char_tab(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_TAB == "\t"


def test_char_quote(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_QUOTE == "'"


def test_char_backslash(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.CHAR_BACKSLASH == "\\"


def test_wchar_ascii(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WCHAR_A == "A"


def test_wchar_omega(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WCHAR_OMEGA == "Ω"
    assert cw.WCHAR_OMEGA == chr(937)


def test_wchar_chinese(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WCHAR_CHINESE == "中"
    assert cw.WCHAR_CHINESE == chr(20013)


def test_wstring_hello(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WSTRING_HELLO == "Hello"


def test_wstring_unicode(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WSTRING_UNICODE == "日本語テスト"


def test_wstring_emoji(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WSTRING_EMOJI == "🎉🚀"


def test_wstring_empty(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    assert cw.WSTRING_EMPTY == ""


def test_char_fields_struct(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    cf = cw.CharFields(single_char="X", wide_char="Ω")
    assert cf.single_char == "X"
    assert cf.wide_char == "Ω"


def test_wstring_fields_struct(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    ws = cw.WstringFields(wide_text="日本語", narrow_text="ASCII")
    assert ws.wide_text == "日本語"
    assert ws.narrow_text == "ASCII"


def test_char_sequences(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    cs = cw.CharSequences(char_seq=["a", "b", "c"], wchar_seq=["α", "β", "γ"])  # noqa: RUF001
    assert cs.char_seq == ["a", "b", "c"]
    assert cs.wchar_seq == ["α", "β", "γ"]  # noqa: RUF001


def test_mixed_char_types(generated_modules: dict[str, ModuleType]) -> None:
    cw = generated_modules["char_wstring_types"]
    m = cw.MixedCharTypes(letter="A", wide_letter="Ω", text="hello", wide_text="世界")
    assert m.letter == "A"
    assert m.wide_letter == "Ω"
    assert m.text == "hello"
    assert m.wide_text == "世界"


def test_french_accents(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.FRENCH == "café résumé naïve"


def test_german_umlauts(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.GERMAN == "größe über müde"


def test_spanish_accents(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.SPANISH == "señor mañana niño"


def test_norwegian_characters(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.NORWEGIAN == "blåbær ærlig øl"


def test_swedish_characters(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.SWEDISH == "smörgås älg ö"


def test_greek(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.GREEK == "αβγδ Ωμέγα"


def test_russian(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.RUSSIAN == "привет мир"


def test_chinese(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.CHINESE == "你好世界"


def test_japanese(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.JAPANESE == "こんにちは世界"


def test_korean(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.KOREAN == "안녕하세요"


def test_emoji(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.EMOJI == "🎉🚀💻🔥"


def test_mixed_scripts(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.MIXED == "Hello 世界 🌍 café"


def test_escaped_quotes(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.QUOTES == 'He said "hello"'


def test_escaped_backslash(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.BACKSLASH == "path\\to\\file"


def test_newline_in_string(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.NEWLINE == "line1\nline2"
    assert len(ut.NEWLINE.split("\n")) == 2


def test_tab_in_string(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.TAB == "col1\tcol2"
    assert len(ut.TAB.split("\t")) == 2


def test_empty_string(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.EMPTY == ""
    assert len(ut.EMPTY) == 0


def test_spaces(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert ut.SPACES == "   "
    assert len(ut.SPACES) == 3


def test_unicode_spaces(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert "\u00a0" in ut.UNICODE_SPACES


def test_math_symbols(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert "∑" in ut.MATH
    assert "∞" in ut.MATH
    assert "√" in ut.MATH


def test_currency_symbols(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert "€" in ut.CURRENCY
    assert "£" in ut.CURRENCY
    assert "¥" in ut.CURRENCY


def test_arrows(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    assert "→" in ut.ARROWS
    assert "←" in ut.ARROWS


def test_unicode_struct_fields(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["unicode_types"]
    data = ut.UnicodeData(label="日本語", description="Ελληνικά 🎉")
    assert data.label == "日本語"
    assert data.description == "Ελληνικά 🎉"
