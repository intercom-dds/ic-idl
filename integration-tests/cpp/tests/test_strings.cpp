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

#include <doctest/doctest.h>

#include <type_traits>

#include "string_types.h"

TEST_CASE("char_letter" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_A == 'A');
}

TEST_CASE("char_digit" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_DIGIT == '5');
}

TEST_CASE("char_space" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_SPACE == ' ');
}

TEST_CASE("char_newline" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_NEWLINE == '\n');
}

TEST_CASE("char_tab" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_TAB == '\t');
}

TEST_CASE("char_quote" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_QUOTE == '\'');
}

TEST_CASE("char_backslash" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::CHAR_BACKSLASH == '\\');
}

TEST_CASE("wchar_ascii" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::WCHAR_A == u'A');
}

TEST_CASE("wchar_omega" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::WCHAR_OMEGA == u'Ω');
}

TEST_CASE("wchar_chinese" * doctest::test_suite("strings")) {
    CHECK(char_wstring_types::WCHAR_CHINESE == u'中');
}

TEST_CASE("wstring_hello" * doctest::test_suite("strings")) {
    CHECK(std::u16string_view(char_wstring_types::WSTRING_HELLO) == u"Hello");
}

TEST_CASE("wstring_unicode" * doctest::test_suite("strings")) {
    CHECK(std::u16string_view(char_wstring_types::WSTRING_UNICODE) == u"日本語テスト");
}

TEST_CASE("wstring_emoji" * doctest::test_suite("strings")) {
    CHECK(std::u16string_view(char_wstring_types::WSTRING_EMOJI) == u"🎉🚀");
}

TEST_CASE("wstring_empty" * doctest::test_suite("strings")) {
    CHECK(std::u16string_view(char_wstring_types::WSTRING_EMPTY) == u"");
}

TEST_CASE("char_fields_struct" * doctest::test_suite("strings")) {
    char_wstring_types::CharFields cf('X', u'Y');
    CHECK(cf.single_char == 'X');
    CHECK(cf.wide_char == u'Y');
}

TEST_CASE("wstring_fields_struct" * doctest::test_suite("strings")) {
    char_wstring_types::WstringFields wf(L"Wide", "Narrow");
    CHECK(wf.wide_text == L"Wide");
    CHECK(wf.narrow_text == "Narrow");
}

TEST_CASE("char_sequences" * doctest::test_suite("strings")) {
    std::vector<char> chars = {'a', 'b', 'c'};
    std::vector<char16_t> wchars = {u'x', u'y', u'z'};
    char_wstring_types::CharSequences cs(chars, wchars);
    CHECK(cs.char_seq.size() == 3);
    CHECK(cs.wchar_seq.size() == 3);
    CHECK(cs.char_seq[0] == 'a');
    CHECK(cs.wchar_seq[2] == u'z');
}

TEST_CASE("mixed_char_types" * doctest::test_suite("strings")) {
    char_wstring_types::MixedCharTypes mct('A', u'Ω', "text", L"wide");
    CHECK(mct.letter == 'A');
    CHECK(mct.wide_letter == u'Ω');
    CHECK(mct.text == "text");
    CHECK(mct.wide_text == L"wide");
}

TEST_CASE("unicode_french" * doctest::test_suite("strings")) {
    CHECK(unicode_types::FRENCH == "café résumé naïve");
}

TEST_CASE("unicode_german" * doctest::test_suite("strings")) {
    CHECK(unicode_types::GERMAN == "größe über müde");
}

TEST_CASE("unicode_spanish" * doctest::test_suite("strings")) {
    CHECK(unicode_types::SPANISH == "señor mañana niño");
}

TEST_CASE("unicode_norwegian" * doctest::test_suite("strings")) {
    CHECK(unicode_types::NORWEGIAN == "blåbær ærlig øl");
}

TEST_CASE("unicode_swedish" * doctest::test_suite("strings")) {
    CHECK(unicode_types::SWEDISH == "smörgås älg ö");
}

TEST_CASE("unicode_greek" * doctest::test_suite("strings")) {
    CHECK(unicode_types::GREEK == "αβγδ Ωμέγα");
}

TEST_CASE("unicode_russian" * doctest::test_suite("strings")) {
    CHECK(unicode_types::RUSSIAN == "привет мир");
}

TEST_CASE("unicode_chinese" * doctest::test_suite("strings")) {
    CHECK(unicode_types::CHINESE == "你好世界");
}

TEST_CASE("unicode_japanese" * doctest::test_suite("strings")) {
    CHECK(unicode_types::JAPANESE == "こんにちは世界");
}

TEST_CASE("unicode_korean" * doctest::test_suite("strings")) {
    CHECK(unicode_types::KOREAN == "안녕하세요");
}

TEST_CASE("unicode_emoji" * doctest::test_suite("strings")) {
    CHECK(unicode_types::EMOJI == "🎉🚀💻🔥");
}

TEST_CASE("unicode_mixed" * doctest::test_suite("strings")) {
    CHECK(unicode_types::MIXED == "Hello 世界 🌍 café");
}

TEST_CASE("unicode_quotes" * doctest::test_suite("strings")) {
    CHECK(unicode_types::QUOTES == "He said \"hello\"");
}

TEST_CASE("unicode_backslash" * doctest::test_suite("strings")) {
    CHECK(unicode_types::BACKSLASH == "path\\to\\file");
}

TEST_CASE("unicode_newline" * doctest::test_suite("strings")) {
    CHECK(unicode_types::NEWLINE == "line1\nline2");
}

TEST_CASE("unicode_tab" * doctest::test_suite("strings")) {
    CHECK(unicode_types::TAB == "col1\tcol2");
}

TEST_CASE("unicode_empty" * doctest::test_suite("strings")) {
    CHECK(unicode_types::EMPTY == "");
}

TEST_CASE("unicode_spaces" * doctest::test_suite("strings")) {
    CHECK(unicode_types::SPACES == "   ");
}

TEST_CASE("unicode_unicode_spaces" * doctest::test_suite("strings")) {
    CHECK(unicode_types::UNICODE_SPACES == " \u00A0 ");
}

TEST_CASE("unicode_math" * doctest::test_suite("strings")) {
    CHECK(unicode_types::MATH == "∑∏∫√∞≠≤≥");
}

TEST_CASE("unicode_currency" * doctest::test_suite("strings")) {
    CHECK(unicode_types::CURRENCY == "$ € £ ¥ ₹ ₽");
}

TEST_CASE("unicode_arrows" * doctest::test_suite("strings")) {
    CHECK(unicode_types::ARROWS == "← → ↑ ↓ ↔ ⇒");
}

TEST_CASE("unicode_data_struct" * doctest::test_suite("strings")) {
    unicode_types::UnicodeData ud("Label", "描述");
    CHECK(ud.label == "Label");
    CHECK(ud.description == "描述");
}

TEST_CASE("unicode_data_with_emoji" * doctest::test_suite("strings")) {
    unicode_types::UnicodeData ud("🎉", "Celebration");
    CHECK(ud.label == "🎉");
    CHECK(ud.description == "Celebration");
}

TEST_CASE("unicode_data_mixed_scripts" * doctest::test_suite("strings")) {
    unicode_types::UnicodeData ud("Hello世界", "Mixed script text");
    CHECK(ud.label == "Hello世界");
    CHECK(ud.description == "Mixed script text");
}

TEST_CASE("char_type_annotations" * doctest::test_suite("strings")) {
    CHECK((std::is_same<decltype(char_wstring_types::CHAR_A), const char>::value));
    CHECK((std::is_same<decltype(char_wstring_types::WCHAR_A), const char16_t>::value));
}

TEST_CASE("string_type_annotations" * doctest::test_suite("strings")) {
    CHECK((std::is_same_v<decltype(char_wstring_types::WSTRING_HELLO), const char16_t* const>));
    CHECK((std::is_same_v<decltype(unicode_types::FRENCH), const char* const>));
}

TEST_CASE("char_fields_type_annotations" * doctest::test_suite("strings")) {
    CHECK((std::is_same_v<decltype(char_wstring_types::CharFields::single_char), char>));
    CHECK((std::is_same_v<decltype(char_wstring_types::CharFields::wide_char), char16_t>));
}

TEST_CASE("wstring_fields_type_annotations" * doctest::test_suite("strings")) {
    CHECK((std::is_same_v<decltype(char_wstring_types::WstringFields::wide_text), std::wstring>));
    CHECK((std::is_same_v<decltype(char_wstring_types::WstringFields::narrow_text), std::string>));
}

TEST_CASE("unicode_data_type_annotations" * doctest::test_suite("strings")) {
    CHECK((std::is_same_v<decltype(unicode_types::UnicodeData::label), std::string>));
    CHECK((std::is_same_v<decltype(unicode_types::UnicodeData::description), std::string>));
}
