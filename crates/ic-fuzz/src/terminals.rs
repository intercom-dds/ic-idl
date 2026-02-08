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

use std::fmt::Write;

use rand::{Rng, RngExt};

use crate::grammar::{IntegerBase, TerminalSpec};

pub struct TerminalGenerator<'a, R> {
    rng: &'a mut R,
}

impl<'a, R: Rng> TerminalGenerator<'a, R> {
    pub fn new(rng: &'a mut R) -> Self {
        Self { rng }
    }

    pub fn write(&mut self, spec: &TerminalSpec, out: &mut impl Write) {
        match spec {
            TerminalSpec::Identifier => self.write_identifier(out),
            TerminalSpec::Integer { bases } => self.write_integer(bases, out),
            TerminalSpec::Float => self.write_float(out),
            TerminalSpec::String => self.write_string(out),
            TerminalSpec::Char => self.write_char(out),
        }
    }

    pub fn write_identifier(&mut self, out: &mut impl Write) {
        const PREFIXES: [&str; 9] = ["x", "y", "z", "foo", "bar", "baz", "my", "the", "a"];
        const SUFFIXES: [&str; 6] = ["", "1", "2", "_val", "_type", "_data"];

        let prefix = PREFIXES[self.rng.random_range(0..PREFIXES.len())];
        let suffix = SUFFIXES[self.rng.random_range(0..SUFFIXES.len())];

        _ = out.write_str(prefix);
        if self.rng.random_bool(0.3) {
            _ = write!(out, "{}", self.rng.random_range(0..100u32));
        }
        _ = out.write_str(suffix);
    }

    fn write_integer(&mut self, bases: &[IntegerBase], out: &mut impl Write) {
        let base = if bases.is_empty() {
            IntegerBase::Decimal
        } else {
            bases[self.rng.random_range(0..bases.len())]
        };

        let value: u64 = if self.rng.random_bool(0.7) {
            self.rng.random_range(0..1000)
        } else {
            self.rng.random_range(0..1_000_000)
        };

        match base {
            IntegerBase::Decimal => {
                _ = write!(out, "{value}");
            }
            IntegerBase::Hex => {
                _ = write!(out, "0x{value:X}");
            }
            IntegerBase::Octal => {
                _ = write!(out, "0{value:o}");
            }
        }
    }

    fn write_float(&mut self, out: &mut impl Write) {
        match self.rng.random_range(0..3) {
            0 => {
                let int_part: u32 = self.rng.random_range(0..1000);
                let frac_part: u32 = self.rng.random_range(0..1000);
                _ = write!(out, "{int_part}.{frac_part}");
            }
            1 => {
                let mantissa: f64 = self.rng.random_range(1.0..10.0);
                let exp: i32 = self.rng.random_range(-10..10);
                _ = write!(out, "{mantissa:.2}e{exp}");
            }
            _ => {
                let value: u32 = self.rng.random_range(1..100);
                let exp: i32 = self.rng.random_range(-5..5);
                _ = write!(out, "{value}E{exp}");
            }
        }
    }

    fn write_string(&mut self, out: &mut impl Write) {
        const CONTENTS: [&str; 10] = [
            "",
            "hello",
            "world",
            "test string",
            "foo bar",
            "123",
            "special: !@#$%",
            "with\ttab",
            "line1\\nline2",
            "quote: \\\"nested\\\"",
        ];

        let content = CONTENTS[self.rng.random_range(0..CONTENTS.len())];
        _ = out.write_char('"');
        _ = out.write_str(content);
        _ = out.write_char('"');
    }

    fn write_char(&mut self, out: &mut impl Write) {
        const CHARS: [char; 18] = [
            'a', 'b', 'c', 'x', 'y', 'z', 'A', 'B', 'C', 'X', 'Y', 'Z', '0', '1', '9', ' ', '!',
            '@',
        ];

        let c = CHARS[self.rng.random_range(0..CHARS.len())];
        _ = out.write_char('\'');
        _ = out.write_char(c);
        _ = out.write_char('\'');
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    fn make_rng() -> StdRng {
        StdRng::seed_from_u64(12345)
    }

    fn generate(rng: &mut StdRng, spec: &TerminalSpec) -> String {
        let mut out = String::new();
        TerminalGenerator::new(rng).write(spec, &mut out);
        out
    }

    #[test]
    fn test_generate_identifier() {
        let mut rng = make_rng();

        for _ in 0..10 {
            let ident = generate(&mut rng, &TerminalSpec::Identifier);
            assert!(!ident.is_empty());
            assert!(ident.chars().next().unwrap().is_ascii_alphabetic());
        }
    }

    #[test]
    fn test_generate_integer_decimal() {
        let mut rng = make_rng();

        let spec = TerminalSpec::Integer {
            bases: vec![IntegerBase::Decimal],
        };
        for _ in 0..10 {
            let num = generate(&mut rng, &spec);
            assert!(num.parse::<u64>().is_ok(), "failed to parse: {num}");
        }
    }

    #[test]
    fn test_generate_integer_hex() {
        let mut rng = make_rng();

        let spec = TerminalSpec::Integer {
            bases: vec![IntegerBase::Hex],
        };
        for _ in 0..10 {
            let num = generate(&mut rng, &spec);
            assert!(num.starts_with("0x"), "expected hex prefix: {num}");
        }
    }

    #[test]
    fn test_generate_float() {
        let mut rng = make_rng();

        for _ in 0..10 {
            let f = generate(&mut rng, &TerminalSpec::Float);
            assert!(
                f.contains('.') || f.contains('e') || f.contains('E'),
                "expected float format: {f}"
            );
        }
    }

    #[test]
    fn test_generate_string() {
        let mut rng = make_rng();

        for _ in 0..10 {
            let s = generate(&mut rng, &TerminalSpec::String);
            assert!(
                s.starts_with('"') && s.ends_with('"'),
                "expected quoted string: {s}"
            );
        }
    }

    #[test]
    fn test_generate_char() {
        let mut rng = make_rng();

        for _ in 0..10 {
            let c = generate(&mut rng, &TerminalSpec::Char);
            assert!(
                c.starts_with('\'') && c.ends_with('\''),
                "expected char literal: {c}"
            );
            assert_eq!(c.len(), 3, "char literal should be 3 chars: {c}");
        }
    }
}
