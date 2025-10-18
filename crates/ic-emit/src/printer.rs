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

#![allow(clippy::needless_pass_by_value)]

const INDENT: &str = "    ";

#[derive(Debug, Clone)]
enum Token {
    Text(String),
    BlockStart(String),
    BlockEnd(String),
    Newline,
    Indent,
    Dedent,
    Tab,
}

pub struct PrettyPrinter {
    tokens: Vec<Token>,
    indent_str: &'static str,
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyPrinter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            indent_str: "    ",
        }
    }

    pub fn text(&mut self, s: impl ToString) -> &mut Self {
        self.tokens.push(Token::Text(s.to_string()));
        self
    }

    pub fn begin(&mut self, brace: impl ToString) -> &mut Self {
        self.tokens.push(Token::BlockStart(brace.to_string()));
        self
    }

    pub fn end(&mut self, brace: impl ToString) -> &mut Self {
        self.tokens.push(Token::BlockEnd(brace.to_string()));
        self
    }

    pub fn endl(&mut self) -> &mut Self {
        self.tokens.push(Token::Newline);
        self
    }

    pub fn tab(&mut self) -> &mut Self {
        self.tokens.push(Token::Tab);
        self
    }

    pub fn blank(&mut self) -> &mut Self {
        if !self.tokens.is_empty() {
            if let Some(Token::Newline) = self.tokens.last() {
                self.tokens.push(Token::Newline);
            } else {
                self.tokens.push(Token::Newline);
                self.tokens.push(Token::Newline);
            }
        }
        self
    }

    pub fn indent(&mut self) -> &mut Self {
        self.tokens.push(Token::Indent);
        self
    }

    pub fn dedent(&mut self) -> &mut Self {
        self.tokens.push(Token::Dedent);
        self
    }

    fn render(&self) -> String {
        let mut output = String::new();
        let mut indent_level = 0usize;
        let mut prev_was_newline = false;

        for token in &self.tokens {
            match token {
                Token::Text(text) => {
                    if prev_was_newline {
                        for _ in 0..indent_level {
                            output.push_str(self.indent_str);
                        }
                        prev_was_newline = false;
                    }
                    output.push_str(text);
                }
                Token::BlockStart(brace) => {
                    if prev_was_newline {
                        for _ in 0..indent_level {
                            output.push_str(self.indent_str);
                        }
                        prev_was_newline = false;
                    }
                    output.push_str(brace);
                    indent_level += 1;
                }
                Token::BlockEnd(brace) => {
                    indent_level = indent_level.saturating_sub(1);
                    if prev_was_newline {
                        for _ in 0..indent_level {
                            output.push_str(self.indent_str);
                        }
                        prev_was_newline = false;
                    }
                    output.push_str(brace);
                }
                Token::Newline => {
                    output.push('\n');
                    prev_was_newline = true;
                }
                Token::Indent => {
                    indent_level += 1;
                }
                Token::Dedent => {
                    indent_level = indent_level.saturating_sub(1);
                }
                Token::Tab => {
                    output.push_str(INDENT);
                }
            }
        }

        output
    }

    #[must_use]
    pub fn finish(self) -> String {
        self.render()
    }
}

pub struct Twine {
    writer: PrettyPrinter,
}

impl Default for Twine {
    fn default() -> Self {
        Self::new()
    }
}

impl Twine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            writer: PrettyPrinter::new(),
        }
    }

    pub fn write(&mut self, args: &[&dyn ToString]) {
        for arg in args {
            let s = arg.to_string();
            for ch in s.chars() {
                match ch {
                    '{' | '[' | '(' => {
                        self.writer.begin(ch);
                    }
                    '}' | ']' | ')' => {
                        self.writer.end(ch);
                    }
                    '\n' => {
                        self.writer.endl();
                    }
                    '\t' => {
                        self.writer.text("    ");
                    }
                    _ => {
                        self.writer.text(ch);
                    }
                }
            }
        }
    }

    pub fn blank(&mut self) {
        self.writer.blank();
    }

    pub fn indent(&mut self) {
        self.writer.indent();
    }

    pub fn dedent(&mut self) {
        self.writer.dedent();
    }

    #[must_use]
    pub fn finish(self) -> String {
        self.writer.finish()
    }
}

#[macro_export]
macro_rules! w {
    ($twine:expr, $($arg:expr),+ $(,)?) => {
        {
            let args: &[&dyn std::string::ToString] = &[$(&$arg),+];
            $twine.write(args);
        }
    };
}
pub use w;
