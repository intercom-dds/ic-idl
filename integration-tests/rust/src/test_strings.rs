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

use crate::{char_wstring_types, unicode_types};

#[test]
fn char_letter() {
    assert_eq!(char_wstring_types::CHAR_A, 'A');
}

#[test]
fn char_digit() {
    assert_eq!(char_wstring_types::CHAR_DIGIT, '5');
}

#[test]
fn char_space() {
    assert_eq!(char_wstring_types::CHAR_SPACE, ' ');
}

#[test]
fn char_newline() {
    assert_eq!(char_wstring_types::CHAR_NEWLINE, '\n');
}

#[test]
fn char_tab() {
    assert_eq!(char_wstring_types::CHAR_TAB, '\t');
}

#[test]
fn char_quote() {
    assert_eq!(char_wstring_types::CHAR_QUOTE, '\'');
}

#[test]
fn char_backslash() {
    assert_eq!(char_wstring_types::CHAR_BACKSLASH, '\\');
}

#[test]
fn wchar_ascii() {
    assert_eq!(char_wstring_types::WCHAR_A, 'A');
}

#[test]
fn wchar_omega() {
    assert_eq!(char_wstring_types::WCHAR_OMEGA, 'Ω');
}

#[test]
fn wchar_chinese() {
    assert_eq!(char_wstring_types::WCHAR_CHINESE, '中');
}

#[test]
fn wstring_hello() {
    assert_eq!(char_wstring_types::WSTRING_HELLO, "Hello");
}

#[test]
fn wstring_unicode() {
    assert_eq!(char_wstring_types::WSTRING_UNICODE, "日本語テスト");
}

#[test]
fn wstring_emoji() {
    assert_eq!(char_wstring_types::WSTRING_EMOJI, "🎉🚀");
}

#[test]
fn wstring_empty() {
    assert_eq!(char_wstring_types::WSTRING_EMPTY, "");
}

#[test]
fn char_fields_struct() {
    let cf = char_wstring_types::CharFields {
        single_char: 'X',
        wide_char: 'Y',
    };
    assert_eq!(cf.single_char, 'X');
    assert_eq!(cf.wide_char, 'Y');
}

#[test]
fn wstring_fields_struct() {
    let wf = char_wstring_types::WstringFields {
        wide_text: "Wide".into(),
        narrow_text: "Narrow".into(),
    };
    assert_eq!(wf.wide_text, "Wide");
    assert_eq!(wf.narrow_text, "Narrow");
}

#[test]
fn char_sequences() {
    let chars = vec!['a', 'b', 'c'];
    let wchars = vec!['x', 'y', 'z'];
    let cs = char_wstring_types::CharSequences {
        char_seq: chars,
        wchar_seq: wchars,
    };
    assert_eq!(cs.char_seq.len(), 3);
    assert_eq!(cs.wchar_seq.len(), 3);
    assert_eq!(cs.char_seq[0], 'a');
    assert_eq!(cs.wchar_seq[2], 'z');
}

#[test]
fn mixed_char_types() {
    let mct = char_wstring_types::MixedCharTypes {
        letter: 'A',
        wide_letter: 'Ω',
        text: "text".into(),
        wide_text: "wide".into(),
    };
    assert_eq!(mct.letter, 'A');
    assert_eq!(mct.wide_letter, 'Ω');
    assert_eq!(mct.text, "text");
    assert_eq!(mct.wide_text, "wide");
}

#[test]
fn unicode_french() {
    assert_eq!(unicode_types::FRENCH, "café résumé naïve");
}

#[test]
fn unicode_german() {
    assert_eq!(unicode_types::GERMAN, "größe über müde");
}

#[test]
fn unicode_spanish() {
    assert_eq!(unicode_types::SPANISH, "señor mañana niño");
}

#[test]
fn unicode_norwegian() {
    assert_eq!(unicode_types::NORWEGIAN, "blåbær ærlig øl");
}

#[test]
fn unicode_swedish() {
    assert_eq!(unicode_types::SWEDISH, "smörgås älg ö");
}

#[test]
fn unicode_greek() {
    assert_eq!(unicode_types::GREEK, "αβγδ Ωμέγα");
}

#[test]
fn unicode_russian() {
    assert_eq!(unicode_types::RUSSIAN, "привет мир");
}

#[test]
fn unicode_chinese() {
    assert_eq!(unicode_types::CHINESE, "你好世界");
}

#[test]
fn unicode_japanese() {
    assert_eq!(unicode_types::JAPANESE, "こんにちは世界");
}

#[test]
fn unicode_korean() {
    assert_eq!(unicode_types::KOREAN, "안녕하세요");
}

#[test]
fn unicode_emoji() {
    assert_eq!(unicode_types::EMOJI, "🎉🚀💻🔥");
}

#[test]
fn unicode_mixed() {
    assert_eq!(unicode_types::MIXED, "Hello 世界 🌍 café");
}

#[test]
fn unicode_quotes() {
    assert_eq!(unicode_types::QUOTES, "He said \"hello\"");
}

#[test]
fn unicode_backslash() {
    assert_eq!(unicode_types::BACKSLASH, "path\\to\\file");
}

#[test]
fn unicode_newline() {
    assert_eq!(unicode_types::NEWLINE, "line1\nline2");
}

#[test]
fn unicode_tab() {
    assert_eq!(unicode_types::TAB, "col1\tcol2");
}

#[test]
fn unicode_empty() {
    assert_eq!(unicode_types::EMPTY, "");
}

#[test]
fn unicode_spaces() {
    assert_eq!(unicode_types::SPACES, "   ");
}

#[test]
fn unicode_unicode_spaces() {
    assert_eq!(unicode_types::UNICODE_SPACES, " \u{a0} ");
}

#[test]
fn unicode_math() {
    assert_eq!(unicode_types::MATH, "∑∏∫√∞≠≤≥");
}

#[test]
fn unicode_currency() {
    assert_eq!(unicode_types::CURRENCY, "$ € £ ¥ ₹ ₽");
}

#[test]
fn unicode_arrows() {
    assert_eq!(unicode_types::ARROWS, "← → ↑ ↓ ↔ ⇒");
}

#[test]
fn unicode_data_struct() {
    let ud = unicode_types::UnicodeData {
        label: "Label".into(),
        description: "描述".into(),
    };
    assert_eq!(ud.label, "Label");
    assert_eq!(ud.description, "描述");
}

#[test]
fn unicode_data_with_emoji() {
    let ud = unicode_types::UnicodeData {
        label: "🎉".into(),
        description: "Celebration".into(),
    };
    assert_eq!(ud.label, "🎉");
    assert_eq!(ud.description, "Celebration");
}

#[test]
fn unicode_data_mixed_scripts() {
    let ud = unicode_types::UnicodeData {
        label: "Hello世界".into(),
        description: "Mixed script text".into(),
    };
    assert_eq!(ud.label, "Hello世界");
    assert_eq!(ud.description, "Mixed script text");
}

#[test]
fn char_type_annotations() {
    let _: char = char_wstring_types::CHAR_A;
    let _: char = char_wstring_types::WCHAR_A;
    assert!(true);
}

#[test]
fn string_type_annotations() {
    let _: &str = &char_wstring_types::WSTRING_HELLO;
    let _: &str = &unicode_types::FRENCH;
    assert!(true);
}

#[test]
fn char_fields_type_annotations() {
    let cf = char_wstring_types::CharFields::new();
    let _: char = cf.single_char;
    let _: char = cf.wide_char;
    assert!(true);
}

#[test]
fn wstring_fields_type_annotations() {
    let wf = char_wstring_types::WstringFields::new();
    let _: String = wf.wide_text;
    let _: String = wf.narrow_text;
    assert!(true);
}

#[test]
fn unicode_data_type_annotations() {
    let ud = unicode_types::UnicodeData::new();
    let _: String = ud.label;
    let _: String = ud.description;
    assert!(true);
}
