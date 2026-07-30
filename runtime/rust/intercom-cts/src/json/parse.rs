// Copyright 2023 KONGSBERG
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
        if let Some(next) = self.skip()
            && c == next
        {
            self.advance();
            return Ok(());
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

    fn parse_hex4(&mut self) -> Result<u16> {
        let mut value = 0;
        for _ in 0..4 {
            let c = self
                .next()
                .ok_or_else(|| error!(self, "incomplete unicode escape"))?;

            let d = c
                .to_digit(16)
                .ok_or_else(|| error!(self, "invalid unicode escape"))?;
            value = (value << 4) | d;
        }
        Ok(value as u16)
    }

    fn string(&mut self) -> Result<String> {
        let mut str = String::new();
        while let Some(c) = self.next() {
            if c == '"' {
                break;
            }

            if c == '\\' {
                let escape = self
                    .next()
                    .ok_or_else(|| error!(self, "unterminated string"))?;

                match escape {
                    '"' => str.push('"'),
                    '\\' => str.push('\\'),
                    '/' => str.push('/'),
                    'b' => str.push('\x08'),
                    'f' => str.push('\x0c'),
                    'n' => str.push('\n'),
                    'r' => str.push('\r'),
                    't' => str.push('\t'),
                    'u' => {
                        let code = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&code) {
                            if self.next() != Some('\\') || self.next() != Some('u') {
                                return Err(error!(
                                    self,
                                    "expected low surrogate after high surrogate"
                                ));
                            }

                            let code2 = self.parse_hex4()?;
                            if !(0xDC00..=0xDFFF).contains(&code2) {
                                return Err(error!(self, "invalid low surrogate"));
                            }

                            let combined = 0x10000
                                + (((code as u32 - 0xD800) << 10) | (code2 as u32 - 0xDC00));

                            if let Some(c) = char::from_u32(combined) {
                                str.push(c);
                            } else {
                                return Err(error!(self, "invalid unicode codepoint"));
                            }
                        } else if let Some(c) = char::from_u32(code as u32) {
                            str.push(c);
                        } else {
                            return Err(error!(self, "invalid unicode codepoint"));
                        }
                    }
                    c => return Err(error!(self, "invalid escape character '{c}'")),
                }
            } else {
                str.push(c);
            }
        }

        self.expect('"')
            .map_err(|_| error!(self, "unterminated string"))?;
        Ok(str)
    }

    fn number(&mut self) -> Result<Number> {
        let mut num_str = String::new();

        if let Some(c) = self.get() {
            if c == '-' {
                num_str.push('-');
                self.advance();
            }
        } else {
            return Err(error!(self, "unexpected end of file"));
        }

        let mut has_digits = false;
        while let Some(c) = self.get() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
                has_digits = true;
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(error!(self, "invalid number"));
        }

        let mut is_float = false;
        if self.get() == Some('.') {
            is_float = true;
            num_str.push('.');
            self.advance();
            while let Some(c) = self.get() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if let Some(c) = self.get()
            && (c == 'e' || c == 'E')
        {
            is_float = true;
            num_str.push(c);
            self.advance();

            if let Some(sign) = self.get()
                && (sign == '+' || sign == '-')
            {
                num_str.push(sign);
                self.advance();
            }

            while let Some(c) = self.get() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if is_float {
            let f = num_str
                .parse::<f64>()
                .map_err(|_| error!(self, "invalid float"))?;
            Ok(Number::Float(f))
        } else if num_str.starts_with('-') {
            let i = num_str
                .parse::<i64>()
                .map_err(|_| error!(self, "invalid integer"))?;
            Ok(Number::Signed(i))
        } else {
            let u = num_str
                .parse::<u64>()
                .map_err(|_| error!(self, "invalid integer"))?;
            Ok(Number::Unsigned(u))
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
