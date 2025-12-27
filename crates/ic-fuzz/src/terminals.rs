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

use rand::Rng;

use crate::grammar::{IntegerBase, TerminalSpec};

pub struct TerminalGenerator<'a, R> {
    rng: &'a mut R,
}

impl<'a, R: Rng> TerminalGenerator<'a, R> {
    pub fn new(rng: &'a mut R) -> Self {
        Self { rng }
    }

    pub fn generate(&mut self, spec: &TerminalSpec) -> String {
        match spec {
            TerminalSpec::Identifier => self.generate_identifier(),
            TerminalSpec::Integer { bases } => self.generate_integer(bases),
            TerminalSpec::Float => self.generate_float(),
            TerminalSpec::String => self.generate_string(),
            TerminalSpec::Char => self.generate_char(),
        }
    }

    pub fn generate_identifier(&mut self) -> String {
        let prefixes = ["x", "y", "z", "foo", "bar", "baz", "my", "the", "a"];
        let suffixes = ["", "1", "2", "_val", "_type", "_data"];

        let prefix = prefixes[self.rng.gen_range(0..prefixes.len())];
        let suffix = suffixes[self.rng.gen_range(0..suffixes.len())];

        if self.rng.gen_bool(0.3) {
            format!("{prefix}{}{suffix}", self.rng.gen_range(0..100))
        } else {
            format!("{prefix}{suffix}")
        }
    }

    fn generate_integer(&mut self, bases: &[IntegerBase]) -> String {
        let base = if bases.is_empty() {
            IntegerBase::Decimal
        } else {
            bases[self.rng.gen_range(0..bases.len())]
        };

        let value: u64 = if self.rng.gen_bool(0.7) {
            self.rng.gen_range(0..1000)
        } else {
            self.rng.gen_range(0..1_000_000)
        };

        match base {
            IntegerBase::Decimal => format!("{value}"),
            IntegerBase::Hex => format!("0x{value:X}"),
            IntegerBase::Octal => format!("0{value:o}"),
        }
    }

    fn generate_float(&mut self) -> String {
        let formats = [
            |rng: &mut R| {
                let int_part: u32 = rng.gen_range(0..1000);
                let frac_part: u32 = rng.gen_range(0..1000);
                format!("{int_part}.{frac_part}")
            },
            |rng: &mut R| {
                let mantissa: f64 = rng.gen_range(1.0..10.0);
                let exp: i32 = rng.gen_range(-10..10);
                format!("{mantissa:.2}e{exp}")
            },
            |rng: &mut R| {
                let value: u32 = rng.gen_range(1..100);
                let exp: i32 = rng.gen_range(-5..5);
                format!("{value}E{exp}")
            },
        ];

        let format_fn = formats[self.rng.gen_range(0..formats.len())];
        format_fn(self.rng)
    }

    fn generate_string(&mut self) -> String {
        let contents = [
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

        let content = contents[self.rng.gen_range(0..contents.len())];
        format!("\"{content}\"")
    }

    fn generate_char(&mut self) -> String {
        let chars = [
            'a', 'b', 'c', 'x', 'y', 'z', 'A', 'B', 'C', 'X', 'Y', 'Z', '0', '1', '9', ' ', '!',
            '@',
        ];

        let c = chars[self.rng.gen_range(0..chars.len())];
        format!("'{c}'")
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

    #[test]
    fn test_generate_identifier() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        for _ in 0..10 {
            let ident = gen.generate(&TerminalSpec::Identifier);
            assert!(!ident.is_empty());
            assert!(ident.chars().next().unwrap().is_ascii_alphabetic());
        }
    }

    #[test]
    fn test_generate_integer_decimal() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        let spec = TerminalSpec::Integer {
            bases: vec![IntegerBase::Decimal],
        };
        for _ in 0..10 {
            let num = gen.generate(&spec);
            assert!(num.parse::<u64>().is_ok(), "failed to parse: {num}");
        }
    }

    #[test]
    fn test_generate_integer_hex() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        let spec = TerminalSpec::Integer {
            bases: vec![IntegerBase::Hex],
        };
        for _ in 0..10 {
            let num = gen.generate(&spec);
            assert!(num.starts_with("0x"), "expected hex prefix: {num}");
        }
    }

    #[test]
    fn test_generate_float() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        for _ in 0..10 {
            let f = gen.generate(&TerminalSpec::Float);
            assert!(
                f.contains('.') || f.contains('e') || f.contains('E'),
                "expected float format: {f}"
            );
        }
    }

    #[test]
    fn test_generate_string() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        for _ in 0..10 {
            let s = gen.generate(&TerminalSpec::String);
            assert!(
                s.starts_with('"') && s.ends_with('"'),
                "expected quoted string: {s}"
            );
        }
    }

    #[test]
    fn test_generate_char() {
        let mut rng = make_rng();
        let mut gen = TerminalGenerator::new(&mut rng);

        for _ in 0..10 {
            let c = gen.generate(&TerminalSpec::Char);
            assert!(
                c.starts_with('\'') && c.ends_with('\''),
                "expected char literal: {c}"
            );
            assert_eq!(c.len(), 3, "char literal should be 3 chars: {c}");
        }
    }
}
