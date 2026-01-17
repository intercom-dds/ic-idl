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

package integrationtests;

import static org.junit.jupiter.api.Assertions.*;

import char_wstring_types.*;
import org.junit.jupiter.api.Test;
import unicode_types.*;

class StringsTests {
    @Test
    void char_letter() {
        assertEquals('A', CHAR_A.value);
    }

    @Test
    void char_digit() {
        assertEquals('5', CHAR_DIGIT.value);
    }

    @Test
    void char_space() {
        assertEquals(' ', CHAR_SPACE.value);
    }

    @Test
    void char_newline() {
        assertEquals('\n', CHAR_NEWLINE.value);
    }

    @Test
    void char_tab() {
        assertEquals('\t', CHAR_TAB.value);
    }

    @Test
    void char_quote() {
        assertEquals('\'', CHAR_QUOTE.value);
    }

    @Test
    void char_backslash() {
        assertEquals('\\', CHAR_BACKSLASH.value);
    }

    @Test
    void wchar_ascii() {
        assertEquals('A', WCHAR_A.value);
    }

    @Test
    void wchar_omega() {
        assertEquals('\u03A9', WCHAR_OMEGA.value);
    }

    @Test
    void wchar_chinese() {
        assertEquals('\u4E2D', WCHAR_CHINESE.value);
    }

    @Test
    void wstring_hello() {
        assertEquals("Hello", WSTRING_HELLO.value);
    }

    @Test
    void wstring_unicode() {
        assertEquals("日本語テスト", WSTRING_UNICODE.value);
    }

    @Test
    void wstring_emoji() {
        assertNotNull(WSTRING_EMOJI.value);
        assertTrue(WSTRING_EMOJI.value.codePoints().anyMatch(cp -> cp > 0xFFFF));
    }

    @Test
    void wstring_empty() {
        assertEquals("", WSTRING_EMPTY.value);
    }

    @Test
    void char_fields_struct() {
        var c = new CharFields();
        c.setSingleChar('X');
        c.setWideChar('\u03B1'); // alpha
        assertEquals('X', c.getSingleChar());
        assertEquals('\u03B1', c.getWideChar());
    }

    @Test
    void wstring_fields_struct() {
        var w = new WstringFields();
        w.setWideText("wide text");
        w.setNarrowText("narrow text");
        assertEquals("wide text", w.getWideText());
        assertEquals("narrow text", w.getNarrowText());
    }

    @Test
    void char_sequences() {
        var c = new CharSequences();
        c.getCharSeq().add('a');
        c.getCharSeq().add('b');
        c.getWcharSeq().add('\u03B1');
        assertEquals(2, c.getCharSeq().size());
        assertEquals(1, c.getWcharSeq().size());
    }

    @Test
    void mixed_char_types() {
        var m = new MixedCharTypes();
        m.setLetter('Z');
        m.setWideLetter('\u4E2D');
        m.setText("narrow");
        m.setWideText("wide");
        assertEquals('Z', m.getLetter());
        assertEquals('\u4E2D', m.getWideLetter());
        assertEquals("narrow", m.getText());
        assertEquals("wide", m.getWideText());
    }

    @Test
    void french_accents() {
        assertEquals("café résumé naïve", FRENCH.value);
    }

    @Test
    void german_umlauts() {
        assertEquals("größe über müde", GERMAN.value);
    }

    @Test
    void spanish_accents() {
        assertEquals("señor mañana niño", SPANISH.value);
    }

    @Test
    void norwegian_characters() {
        assertEquals("blåbær ærlig øl", NORWEGIAN.value);
    }

    @Test
    void swedish_characters() {
        assertEquals("smörgås älg ö", SWEDISH.value);
    }

    @Test
    void greek() {
        assertEquals("αβγδ Ωμέγα", GREEK.value);
    }

    @Test
    void russian() {
        assertEquals("привет мир", RUSSIAN.value);
    }

    @Test
    void chinese() {
        assertEquals("你好世界", CHINESE.value);
    }

    @Test
    void japanese() {
        assertEquals("こんにちは世界", JAPANESE.value);
    }

    @Test
    void korean() {
        assertEquals("안녕하세요", KOREAN.value);
    }

    @Test
    void emoji() {
        assertNotNull(EMOJI.value);
        assertTrue(EMOJI.value.codePoints().anyMatch(cp -> cp > 0xFFFF));
    }

    @Test
    void mixed_scripts() {
        assertNotNull(MIXED.value);
        assertFalse(MIXED.value.isEmpty());
    }

    @Test
    void escaped_quotes() {
        assertTrue(QUOTES.value.contains("\""));
    }

    @Test
    void escaped_backslash() {
        assertTrue(BACKSLASH.value.contains("\\"));
    }

    @Test
    void newline_in_string() {
        assertTrue(NEWLINE.value.contains("\n"));
    }

    @Test
    void tab_in_string() {
        assertTrue(TAB.value.contains("\t"));
    }

    @Test
    void empty_string() {
        assertEquals("", EMPTY.value);
    }

    @Test
    void spaces() {
        assertTrue(SPACES.value.contains(" "));
    }

    @Test
    void unicode_spaces() {
        assertFalse(UNICODE_SPACES.value.isEmpty());
        assertTrue(UNICODE_SPACES.value.contains("\u00A0") || UNICODE_SPACES.value.contains("\u2003"));
    }

    @Test
    void math_symbols() {
        assertFalse(MATH.value.isEmpty());
        assertTrue(MATH.value.contains("∑") || MATH.value.matches(".*[\\u2200-\\u22FF].*"));
    }

    @Test
    void currency_symbols() {
        assertFalse(CURRENCY.value.isEmpty());
        assertTrue(CURRENCY.value.contains("€") || CURRENCY.value.contains("¥") || CURRENCY.value.contains("$"));
    }

    @Test
    void arrows() {
        assertFalse(ARROWS.value.isEmpty());
        assertTrue(ARROWS.value.contains("→") || ARROWS.value.contains("↔")
                || ARROWS.value.matches(".*[\\u2190-\\u21FF].*"));
    }

    @Test
    void unicode_struct_fields() {
        var u = new UnicodeData();
        u.setLabel("日本語");
        u.setDescription("αβγ");
        assertEquals("日本語", u.getLabel());
        assertEquals("αβγ", u.getDescription());
    }
}
