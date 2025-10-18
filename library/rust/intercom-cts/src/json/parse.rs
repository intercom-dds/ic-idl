// Copyright 2025 KONGSBERG
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

use std::collections::BTreeMap;

use crate::json::error::Error;
use crate::json::{Number, Result, Value};

macro_rules! error {
    ($self:ident, $($arg:tt)*) => {
        Error {
            msg: format!($($arg)*),
            line: $self.line,
            column: $self.column,
        }
    }
}

struct JsonParser<I: Iterator<Item = char>> {
    iter: I,
    curr: Option<char>,
    line: usize,
    column: usize,
}

impl<I> JsonParser<I>
where
    I: Iterator<Item = char>,
{
    fn new(mut iter: I) -> Self {
        let curr = iter.next();
        Self {
            iter,
            curr,
            line: 1,
            column: 1,
        }
    }

    fn get(&mut self) -> Option<char> {
        self.curr
    }

    fn skip(&mut self) -> Option<char> {
        while let Some(c) = self.get() {
            if !c.is_ascii_whitespace() {
                break;
            }
            self.advance();
        }
        self.get()
    }

    fn advance(&mut self) {
        self.curr = self.iter.next();
        if let Some(c) = self.curr {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn expect(&mut self, c: char) -> Result<()> {
        if let Some(next) = self.skip() {
            if c == next {
                self.advance();
                return Ok(());
            }
        }
        Err(error!(self, "expected '{c}'"))
    }

    fn next(&mut self) -> Option<char> {
        self.advance();
        self.get()
    }

    fn any(&mut self) -> Result<Value> {
        match self
            .skip()
            .ok_or_else(|| error!(self, "unexpected end of file"))?
        {
            'n' => self.null(),
            't' => self.bool(true),
            'f' => self.bool(false),
            '{' => self.object().map(Value::Object),
            '[' => self.array().map(Value::Array),
            '"' => self.string().map(Value::String),
            '-' | '0'..='9' => self.number().map(Value::Number),
            c => Err(error!(self, "unexpected character: '{c}'")),
        }
    }

    fn ident(&mut self, ident: &'static str) -> bool {
        if ident.chars().skip(1).all(|c| self.next() == Some(c)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn null(&mut self) -> Result<Value> {
        self.ident("null")
            .then_some(Value::Null)
            .ok_or_else(|| error!(self, "expected 'null'"))
    }

    fn bool(&mut self, value: bool) -> Result<Value> {
        let res = if value {
            self.ident("true").then_some(Value::Bool(true))
        } else {
            self.ident("false").then_some(Value::Bool(false))
        };
        res.ok_or_else(|| error!(self, "expected boolean"))
    }

    fn object(&mut self) -> Result<BTreeMap<String, Value>> {
        let mut values = BTreeMap::new();
        self.expect('{')?;

        while let Some(c) = self.skip() {
            if c == '}' {
                break;
            }

            let key = self.string()?;
            self.expect(':')?;
            let value = self.any()?;
            values.insert(key, value);

            if self.skip() != Some('}') {
                self.expect(',')?;
            }
        }

        self.expect('}')?;
        Ok(values)
    }

    fn array(&mut self) -> Result<Vec<Value>> {
        let mut values = vec![];
        self.expect('[')?;

        while self.skip() != Some(']') {
            if !values.is_empty() {
                self.expect(',')?;
            }
            values.push(self.any()?);
        }
        self.expect(']')?;
        Ok(values)
    }

    fn string(&mut self) -> Result<String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            let c = self
                .get()
                .ok_or_else(|| error!(self, "unterminated string"))?;
            match c {
                '"' => {
                    self.advance();
                    break;
                }
                '\\' => {
                    out.push(self.read_escape()?);
                    self.advance();
                }
                c if c < '\u{20}' => return Err(error!(self, "unescaped control character")),
                _ => {
                    out.push(c);
                    self.advance();
                }
            }
        }

        Ok(out)
    }

    fn read_escape(&mut self) -> Result<char> {
        let e = self
            .next()
            .ok_or_else(|| error!(self, "incomplete escape"))?;
        Ok(match e {
            '"' | '\\' | '/' => e,
            'b' => '\u{0008}',
            'f' => '\u{000C}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => return self.read_unicode_escape(),
            _ => return Err(error!(self, "invalid escape")),
        })
    }

    fn read_unicode_escape(&mut self) -> Result<char> {
        let hi = self.read_hex4()? as u32;

        // High surrogate -> must be followed by \uDC00–\uDFFF
        if (0xD800..=0xDBFF).contains(&hi) {
            self.expect('\\')?;
            self.expect('u')?;
            let lo = self.read_hex4()? as u32;
            if !(0xDC00..=0xDFFF).contains(&lo) {
                return Err(error!(self, "invalid low surrogate"));
            }
            let cp = 0x10000 + (((hi - 0xD800) << 10) | (lo - 0xDC00));
            return char::from_u32(cp).ok_or_else(|| error!(self, "invalid code point"));
        }

        // Lone low surrogate is invalid
        if (0xDC00..=0xDFFF).contains(&hi) {
            return Err(error!(self, "unexpected low surrogate"));
        }

        char::from_u32(hi).ok_or_else(|| error!(self, "invalid code point"))
    }

    fn read_hex4(&mut self) -> Result<u16> {
        let mut v: u16 = 0;
        for _ in 0..4 {
            let c = self
                .next()
                .ok_or_else(|| error!(self, "incomplete unicode escape"))?;
            let d = c
                .to_digit(16)
                .ok_or_else(|| error!(self, "invalid unicode escape"))?;
            v = (v << 4) | (d as u16);
        }
        Ok(v)
    }

    fn integer(&mut self) -> Option<u64> {
        let mut value: u64 = 0;
        while let Some(c) = self.get() {
            match c {
                c @ '0'..='9' => {
                    value = value.checked_mul(10)?;
                    value = value.checked_add(c as u64 - ('0' as u64))?;
                    self.advance();
                }
                _ => break,
            }
        }
        Some(value)
    }

    #[allow(clippy::cast_precision_loss)]
    fn number(&mut self) -> Result<Number> {
        let neg = if self.get() == Some('-') {
            self.advance();
            true
        } else {
            false
        };

        let value = self
            .integer()
            .ok_or_else(|| error!(self, "invalid number"))?;

        if self.get() == Some('.') {
            self.advance();

            let mut dec = 1.0;
            let mut frac = 0.0;
            while let Some(c) = self.get() {
                match c {
                    c @ '0'..='9' => {
                        dec /= 10.0;
                        frac += ((c as u64) - ('0' as u64)) as f64 * dec;
                        self.advance();
                    }
                    _ => break,
                }
            }
            return Ok(Number::Float(value as f64 + frac));
        }

        if neg {
            if value > i64::MAX as u64 + 1 {
                Err(error!(self, "invalid number"))
            } else {
                Ok(Number::Signed(i64::try_from(value).unwrap().wrapping_neg()))
            }
        } else {
            Ok(Number::Unsigned(value))
        }
    }

    fn parse(&mut self) -> Result<Value> {
        let value = self.any()?;
        if let Some(c) = self.skip() {
            Err(error!(self, "unexpected character '{c}'"))
        } else {
            Ok(value)
        }
    }
}

pub fn parse(input: &str) -> Result<Value> {
    let mut parser = JsonParser::new(input.chars());
    parser.parse()
}
