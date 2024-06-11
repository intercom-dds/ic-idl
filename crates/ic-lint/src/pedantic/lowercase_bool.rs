// Copyright 2024 KONGSBERG
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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_syntax::visit::Visitor;

/// Lint that checks for uses of lowercase `true` or `false`, neither of which
/// are standard IDL. Only `TRUE` and `FALSE` are specified in the standard.
pub struct LowercaseBool<'a>(&'a str);

impl<'a> Visitor<'a> for LowercaseBool<'a> {
    fn visit_literal(&mut self, num: &'a ic_syntax::Literal) {
        if let ic_syntax::LitKind::LitBool = &num.kind {
            let range = num.span.start as usize..num.span.start as usize;
            if let Some(span) = self.0.get(range) {
                if span.chars().any(char::is_lowercase) {
                    eprintln!(
                        "{}:{}: boolean literals must be written in uppercase",
                        num.span.start,
                        num.span.end + num.span.start,
                    );
                    eprintln!(" = help: lowercase literals are an InterCOM extension");
                    eprintln!(" = note: warning produced by -Wpedantic");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ic_syntax::*;

    use super::*;

    #[test]
    fn lowercase_lit() {
        let ident = Literal {
            kind: LitKind::Bool,
            span: Span::default(),
        };
        let mut lint = LowercaseBool("true");
        lint.visit_literal(&ident);

        let ident = Literal {
            kind: LitKind::Bool,
            span: Span::default(),
        };
        let mut lint = LowercaseBool("false");
        lint.visit_literal(&ident);
    }

    #[test]
    fn uppercase_lit() {
        // complies with the standard so no warning produced
        let num = Literal {
            kind: LitKind::Bool,
            span: Span::default(),
        };
        let mut lint = LowercaseBool("TRUE");
        lint.visit_literal(&num);

        let num = Literal {
            kind: LitKind::Bool,
            span: Span::default(),
        };
        let mut lint = LowercaseBool("FALSE");
        lint.visit_literal(&num);
    }
}
