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

use ic_emit::printer::PrettyPrinter;

pub struct PyWriter {
    printer: PrettyPrinter,
}

impl PyWriter {
    pub fn new() -> Self {
        Self {
            printer: PrettyPrinter::new(),
        }
    }

    pub fn write(&mut self, args: &[&dyn ToString]) {
        for arg in args {
            let s = arg.to_string();
            for ch in s.chars() {
                match ch {
                    '\n' => {
                        self.printer.endl();
                    }
                    _ => {
                        self.printer.text(ch);
                    }
                }
            }
        }
    }

    pub fn indent(&mut self) {
        self.printer.indent();
    }

    pub fn dedent(&mut self) {
        self.printer.dedent();
    }

    pub fn finish(self) -> String {
        let mut result = self
            .printer
            .finish()
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");

        result.truncate(result.trim_end().len());
        result.push('\n');
        result
    }
}

impl Default for PyWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! py {
    ($writer:expr, $($arg:expr),+ $(,)?) => {
        {
            let args: &[&dyn std::string::ToString] = &[$(&$arg),+];
            $writer.write(args);
        }
    };
}
