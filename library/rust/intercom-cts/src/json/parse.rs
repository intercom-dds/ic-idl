// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

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

    fn string(&mut self) -> Result<String> {
        let mut str = String::new();
        while let Some(c) = self.next() {
            if c == '"' {
                break;
            }
            str.push(c);
        }

        self.expect('"')
            .map_err(|_| error!(self, "unterminated string"))?;
        Ok(str)
    }

    // TODO: leading zeroes are not permitted
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
                Ok(Number::Signed((value as i64).wrapping_neg()))
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
