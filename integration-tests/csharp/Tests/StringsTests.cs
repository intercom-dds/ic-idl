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

using Xunit;
using CharWstringTypes;
using UnicodeTypes;

namespace IntegrationTests;

public class StringsTests
{
    [Fact]
    public void CharConstants_BasicValues()
    {
        Assert.Equal('A', CharWstringTypes.Constants.CharA);
        Assert.Equal('5', CharWstringTypes.Constants.CharDigit);
        Assert.Equal(' ', CharWstringTypes.Constants.CharSpace);
    }

    [Fact]
    public void CharConstants_EscapeSequences()
    {
        Assert.Equal('\n', CharWstringTypes.Constants.CharNewline);
        Assert.Equal('\t', CharWstringTypes.Constants.CharTab);
        Assert.Equal('\'', CharWstringTypes.Constants.CharQuote);
        Assert.Equal('\\', CharWstringTypes.Constants.CharBackslash);
    }

    [Fact]
    public void WcharConstants_BasicValues()
    {
        Assert.Equal('A', CharWstringTypes.Constants.WcharA);
    }

    [Fact]
    public void WcharConstants_UnicodeValues()
    {
        Assert.Equal('\u03A9', CharWstringTypes.Constants.WcharOmega);
        Assert.Equal('\u4E2D', CharWstringTypes.Constants.WcharChinese);
    }

    [Fact]
    public void WstringConstants_BasicValues()
    {
        Assert.Equal("Hello", CharWstringTypes.Constants.WstringHello);
        Assert.Equal("", CharWstringTypes.Constants.WstringEmpty);
    }

    [Fact]
    public void WstringConstants_UnicodeValues()
    {
        Assert.Equal("日本語テスト", CharWstringTypes.Constants.WstringUnicode);
    }

    [Fact]
    public void CharFields_Instantiation()
    {
        var c = new CharFields();
        Assert.Equal('\0', c.SingleChar);
        Assert.Equal('\0', c.WideChar);
    }

    [Fact]
    public void CharFields_CanSetValues()
    {
        var c = new CharFields('X', '\u03A9');
        Assert.Equal('X', c.SingleChar);
        Assert.Equal('\u03A9', c.WideChar);
    }

    [Fact]
    public void WstringFields_Instantiation()
    {
        var w = new WstringFields();
        Assert.Equal("", w.WideText);
        Assert.Equal("", w.NarrowText);
    }

    [Fact]
    public void WstringFields_CanSetValues()
    {
        var w = new WstringFields("こんにちは", "Hello");
        Assert.Equal("こんにちは", w.WideText);
        Assert.Equal("Hello", w.NarrowText);
    }

    [Fact]
    public void CharSequences_EmptyByDefault()
    {
        var c = new CharSequences();
        Assert.Empty(c.CharSeq);
        Assert.Empty(c.WcharSeq);
    }

    [Fact]
    public void CharSequences_CanAddChars()
    {
        var c = new CharSequences();
        c.CharSeq.Add('a');
        c.CharSeq.Add('b');
        c.WcharSeq.Add('\u03B1');
        c.WcharSeq.Add('\u03B2');

        Assert.Equal(2, c.CharSeq.Count);
        Assert.Equal(2, c.WcharSeq.Count);
    }

    [Fact]
    public void MixedCharTypes_Instantiation()
    {
        var m = new MixedCharTypes();
        Assert.Equal('\0', m.Letter);
        Assert.Equal('\0', m.WideLetter);
        Assert.Equal("", m.Text);
        Assert.Equal("", m.WideText);
    }

    [Fact]
    public void MixedCharTypes_CanSetAllFields()
    {
        var m = new MixedCharTypes('A', '\u03A9', "Hello", "世界");
        Assert.Equal('A', m.Letter);
        Assert.Equal('\u03A9', m.WideLetter);
        Assert.Equal("Hello", m.Text);
        Assert.Equal("世界", m.WideText);
    }

    [Fact]
    public void UnicodeConstants_European()
    {
        Assert.Contains("é", UnicodeTypes.Constants.French);
        Assert.Contains("ö", UnicodeTypes.Constants.German);
        Assert.Contains("ñ", UnicodeTypes.Constants.Spanish);
        Assert.Contains("å", UnicodeTypes.Constants.Norwegian);
        Assert.Contains("ö", UnicodeTypes.Constants.Swedish);
    }

    [Fact]
    public void UnicodeConstants_Asian()
    {
        Assert.Equal("你好世界", UnicodeTypes.Constants.Chinese);
        Assert.Contains("こんにちは", UnicodeTypes.Constants.Japanese);
        Assert.Equal("안녕하세요", UnicodeTypes.Constants.Korean);
    }

    [Fact]
    public void UnicodeConstants_Greek()
    {
        Assert.Contains("α", UnicodeTypes.Constants.Greek);
        Assert.Contains("Ω", UnicodeTypes.Constants.Greek);
    }

    [Fact]
    public void UnicodeConstants_Russian()
    {
        Assert.Contains("п", UnicodeTypes.Constants.Russian);
    }

    [Fact]
    public void UnicodeConstants_SpecialChars()
    {
        Assert.Equal("He said \"hello\"", UnicodeTypes.Constants.Quotes);
        Assert.Equal("path\\to\\file", UnicodeTypes.Constants.Backslash);
        Assert.Equal("line1\nline2", UnicodeTypes.Constants.Newline);
        Assert.Equal("col1\tcol2", UnicodeTypes.Constants.Tab);
    }

    [Fact]
    public void UnicodeConstants_Whitespace()
    {
        Assert.Equal("", UnicodeTypes.Constants.Empty);
        Assert.Equal("   ", UnicodeTypes.Constants.Spaces);
    }

    [Fact]
    public void UnicodeConstants_Symbols()
    {
        Assert.Contains("∑", UnicodeTypes.Constants.Math);
        Assert.Contains("€", UnicodeTypes.Constants.Currency);
        Assert.Contains("←", UnicodeTypes.Constants.Arrows);
    }

    [Fact]
    public void UnicodeConstants_Mixed()
    {
        var mixed = UnicodeTypes.Constants.Mixed;
        Assert.Contains("Hello", mixed);
        Assert.Contains("世界", mixed);
        Assert.Contains("é", mixed);
    }

    [Fact]
    public void UnicodeData_Instantiation()
    {
        var u = new UnicodeData();
        Assert.Equal("", u.Label);
        Assert.Equal("", u.Description);
    }

    [Fact]
    public void UnicodeData_CanSetUnicodeValues()
    {
        var u = new UnicodeData("日本語", "Japanese language text");
        Assert.Equal("日本語", u.Label);
        Assert.Equal("Japanese language text", u.Description);
    }

    [Fact]
    public void UnicodeData_Equality()
    {
        var u1 = new UnicodeData("こんにちは", "Hello");
        var u2 = new UnicodeData("こんにちは", "Hello");
        var u3 = new UnicodeData("さようなら", "Goodbye");

        Assert.Equal(u1, u2);
        Assert.NotEqual(u1, u3);
    }

    [Fact]
    public void CharFields_Equality()
    {
        var c1 = new CharFields('A', '\u03A9');
        var c2 = new CharFields('A', '\u03A9');
        var c3 = new CharFields('B', '\u03A9');

        Assert.Equal(c1, c2);
        Assert.NotEqual(c1, c3);
    }

    [Fact]
    public void WstringFields_Equality()
    {
        var w1 = new WstringFields("Hello", "World");
        var w2 = new WstringFields("Hello", "World");
        var w3 = new WstringFields("Goodbye", "World");

        Assert.Equal(w1, w2);
        Assert.NotEqual(w1, w3);
    }

    [Fact]
    public void MixedCharTypes_Equality()
    {
        var m1 = new MixedCharTypes('A', '\u03A9', "Hello", "世界");
        var m2 = new MixedCharTypes('A', '\u03A9', "Hello", "世界");
        var m3 = new MixedCharTypes('B', '\u03A9', "Hello", "世界");

        Assert.Equal(m1, m2);
        Assert.NotEqual(m1, m3);
    }

    [Fact]
    public void CharFields_FieldTypes()
    {
        Assert.Equal(typeof(char), typeof(CharFields).GetProperty("SingleChar")!.PropertyType);
        Assert.Equal(typeof(char), typeof(CharFields).GetProperty("WideChar")!.PropertyType);
    }

    [Fact]
    public void WstringFields_FieldTypes()
    {
        Assert.Equal(typeof(string), typeof(WstringFields).GetProperty("WideText")!.PropertyType);
        Assert.Equal(typeof(string), typeof(WstringFields).GetProperty("NarrowText")!.PropertyType);
    }
}
