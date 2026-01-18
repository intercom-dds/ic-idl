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

#include <gtest/gtest.h>

#include <type_traits>

#include "generated/strings.h"

namespace {

TEST(StringsTest, test_char_letter) {
    EXPECT_EQ(char_wstring_types::CHAR_A, 'A');
}

TEST(StringsTest, test_char_digit) {
    EXPECT_EQ(char_wstring_types::CHAR_DIGIT, '5');
}

TEST(StringsTest, test_char_space) {
    EXPECT_EQ(char_wstring_types::CHAR_SPACE, ' ');
}

TEST(StringsTest, test_char_newline) {
    EXPECT_EQ(char_wstring_types::CHAR_NEWLINE, '\n');
}

TEST(StringsTest, test_char_tab) {
    EXPECT_EQ(char_wstring_types::CHAR_TAB, '\t');
}

TEST(StringsTest, test_char_quote) {
    EXPECT_EQ(char_wstring_types::CHAR_QUOTE, '\'');
}

TEST(StringsTest, test_char_backslash) {
    EXPECT_EQ(char_wstring_types::CHAR_BACKSLASH, '\\');
}

TEST(StringsTest, test_wchar_ascii) {
    EXPECT_EQ(char_wstring_types::WCHAR_A, u'A');
}

TEST(StringsTest, test_wchar_omega) {
    EXPECT_EQ(char_wstring_types::WCHAR_OMEGA, u'Ω');
}

TEST(StringsTest, test_wchar_chinese) {
    EXPECT_EQ(char_wstring_types::WCHAR_CHINESE, u'中');
}

TEST(StringsTest, test_wstring_hello) {
    EXPECT_EQ(std::u16string_view(char_wstring_types::WSTRING_HELLO), u"Hello");
}

TEST(StringsTest, test_wstring_unicode) {
    EXPECT_EQ(std::u16string_view(char_wstring_types::WSTRING_UNICODE), u"日本語テスト");
}

TEST(StringsTest, test_wstring_emoji) {
    EXPECT_EQ(std::u16string_view(char_wstring_types::WSTRING_EMOJI), u"🎉🚀");
}

TEST(StringsTest, test_wstring_empty) {
    EXPECT_EQ(std::u16string_view(char_wstring_types::WSTRING_EMPTY), u"");
}

TEST(StringsTest, test_char_fields_struct) {
    char_wstring_types::CharFields cf('X', u'Y');
    EXPECT_EQ(cf.single_char, 'X');
    EXPECT_EQ(cf.wide_char, u'Y');
}

TEST(StringsTest, test_wstring_fields_struct) {
    char_wstring_types::WstringFields wf(L"Wide", "Narrow");
    EXPECT_EQ(wf.wide_text, L"Wide");
    EXPECT_EQ(wf.narrow_text, "Narrow");
}

TEST(StringsTest, test_char_sequences) {
    std::vector<char> chars = {'a', 'b', 'c'};
    std::vector<char16_t> wchars = {u'x', u'y', u'z'};
    char_wstring_types::CharSequences cs(chars, wchars);
    EXPECT_EQ(cs.char_seq.size(), 3);
    EXPECT_EQ(cs.wchar_seq.size(), 3);
    EXPECT_EQ(cs.char_seq[0], 'a');
    EXPECT_EQ(cs.wchar_seq[2], u'z');
}

TEST(StringsTest, test_mixed_char_types) {
    char_wstring_types::MixedCharTypes mct('A', u'Ω', "text", L"wide");
    EXPECT_EQ(mct.letter, 'A');
    EXPECT_EQ(mct.wide_letter, u'Ω');
    EXPECT_EQ(mct.text, "text");
    EXPECT_EQ(mct.wide_text, L"wide");
}

TEST(StringsTest, test_unicode_french) {
    EXPECT_STREQ(unicode_types::FRENCH, "café résumé naïve");
}

TEST(StringsTest, test_unicode_german) {
    EXPECT_STREQ(unicode_types::GERMAN, "größe über müde");
}

TEST(StringsTest, test_unicode_spanish) {
    EXPECT_STREQ(unicode_types::SPANISH, "señor mañana niño");
}

TEST(StringsTest, test_unicode_norwegian) {
    EXPECT_STREQ(unicode_types::NORWEGIAN, "blåbær ærlig øl");
}

TEST(StringsTest, test_unicode_swedish) {
    EXPECT_STREQ(unicode_types::SWEDISH, "smörgås älg ö");
}

TEST(StringsTest, test_unicode_greek) {
    EXPECT_STREQ(unicode_types::GREEK, "αβγδ Ωμέγα");
}

TEST(StringsTest, test_unicode_russian) {
    EXPECT_STREQ(unicode_types::RUSSIAN, "привет мир");
}

TEST(StringsTest, test_unicode_chinese) {
    EXPECT_STREQ(unicode_types::CHINESE, "你好世界");
}

TEST(StringsTest, test_unicode_japanese) {
    EXPECT_STREQ(unicode_types::JAPANESE, "こんにちは世界");
}

TEST(StringsTest, test_unicode_korean) {
    EXPECT_STREQ(unicode_types::KOREAN, "안녕하세요");
}

TEST(StringsTest, test_unicode_emoji) {
    EXPECT_STREQ(unicode_types::EMOJI, "🎉🚀💻🔥");
}

TEST(StringsTest, test_unicode_mixed) {
    EXPECT_STREQ(unicode_types::MIXED, "Hello 世界 🌍 café");
}

TEST(StringsTest, test_unicode_quotes) {
    EXPECT_STREQ(unicode_types::QUOTES, "He said \"hello\"");
}

TEST(StringsTest, test_unicode_backslash) {
    EXPECT_STREQ(unicode_types::BACKSLASH, "path\\to\\file");
}

TEST(StringsTest, test_unicode_newline) {
    EXPECT_STREQ(unicode_types::NEWLINE, "line1\nline2");
}

TEST(StringsTest, test_unicode_tab) {
    EXPECT_STREQ(unicode_types::TAB, "col1\tcol2");
}

TEST(StringsTest, test_unicode_empty) {
    EXPECT_STREQ(unicode_types::EMPTY, "");
}

TEST(StringsTest, test_unicode_spaces) {
    EXPECT_STREQ(unicode_types::SPACES, "   ");
}

TEST(StringsTest, test_unicode_unicode_spaces) {
    EXPECT_STREQ(unicode_types::UNICODE_SPACES, " \u00A0 ");
}

TEST(StringsTest, test_unicode_math) {
    EXPECT_STREQ(unicode_types::MATH, "∑∏∫√∞≠≤≥");
}

TEST(StringsTest, test_unicode_currency) {
    EXPECT_STREQ(unicode_types::CURRENCY, "$ € £ ¥ ₹ ₽");
}

TEST(StringsTest, test_unicode_arrows) {
    EXPECT_STREQ(unicode_types::ARROWS, "← → ↑ ↓ ↔ ⇒");
}

TEST(StringsTest, test_unicode_data_struct) {
    unicode_types::UnicodeData ud("Label", "描述");
    EXPECT_EQ(ud.label, "Label");
    EXPECT_EQ(ud.description, "描述");
}

TEST(StringsTest, test_unicode_data_with_emoji) {
    unicode_types::UnicodeData ud("🎉", "Celebration");
    EXPECT_EQ(ud.label, "🎉");
    EXPECT_EQ(ud.description, "Celebration");
}

TEST(StringsTest, test_unicode_data_mixed_scripts) {
    unicode_types::UnicodeData ud("Hello世界", "Mixed script text");
    EXPECT_EQ(ud.label, "Hello世界");
    EXPECT_EQ(ud.description, "Mixed script text");
}

TEST(StringsTest, test_char_type_annotations) {
    EXPECT_TRUE((std::is_same<decltype(char_wstring_types::CHAR_A), const char>::value));
    EXPECT_TRUE((std::is_same<decltype(char_wstring_types::WCHAR_A), const char16_t>::value));
}

TEST(StringsTest, test_string_type_annotations) {
    EXPECT_TRUE((std::is_same_v<decltype(char_wstring_types::WSTRING_HELLO), const char16_t* const>)
    );
    EXPECT_TRUE((std::is_same_v<decltype(unicode_types::FRENCH), const char* const>));
}

TEST(StringsTest, test_char_fields_type_annotations) {
    EXPECT_TRUE((std::is_same_v<decltype(char_wstring_types::CharFields::single_char), char>));
    EXPECT_TRUE((std::is_same_v<decltype(char_wstring_types::CharFields::wide_char), char16_t>));
}

TEST(StringsTest, test_wstring_fields_type_annotations) {
    EXPECT_TRUE(
        (std::is_same_v<decltype(char_wstring_types::WstringFields::wide_text), std::wstring>)
    );
    EXPECT_TRUE(
        (std::is_same_v<decltype(char_wstring_types::WstringFields::narrow_text), std::string>)
    );
}

TEST(StringsTest, test_unicode_data_type_annotations) {
    EXPECT_TRUE((std::is_same_v<decltype(unicode_types::UnicodeData::label), std::string>));
    EXPECT_TRUE((std::is_same_v<decltype(unicode_types::UnicodeData::description), std::string>));
}

} // namespace
