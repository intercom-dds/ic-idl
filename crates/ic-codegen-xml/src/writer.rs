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

use ic_emit::printer::PrettyPrinter;

pub struct XmlWriter {
    printer: PrettyPrinter,
}

impl XmlWriter {
    pub fn new() -> Self {
        Self {
            printer: PrettyPrinter::new(),
        }
    }

    pub fn declaration(&mut self, version: &str, encoding: &str) {
        self.printer
            .text("<?xml version=\"")
            .text(version)
            .text("\" encoding=\"")
            .text(encoding)
            .text("\"?>")
            .endl();
    }

    pub fn element<F>(&mut self, name: &str, attrs: &[(&str, &str)], f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.printer.text("<").text(name);

        for (key, value) in attrs {
            self.printer.text(" ");
            self.write_escaped_attr(key);
            self.printer.text("=\"");
            self.write_escaped_attr(value);
            self.printer.text("\"");
        }

        self.printer.text(">").endl().indent();

        f(self);

        self.printer.dedent().text("</").text(name).text(">").endl();
    }

    pub fn empty_element(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.printer.text("<").text(name);

        for (key, value) in attrs {
            self.printer.text(" ");
            self.write_escaped_attr(key);
            self.printer.text("=\"");
            self.write_escaped_attr(value);
            self.printer.text("\"");
        }

        self.printer.text("/>").endl();
    }

    fn write_escaped_attr(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '&' => self.printer.text("&amp;"),
                '"' => self.printer.text("&quot;"),
                '\'' => self.printer.text("&apos;"),
                '<' => self.printer.text("&lt;"),
                '>' => self.printer.text("&gt;"),
                _ => self.printer.text(c),
            };
        }
    }

    pub fn finish(self) -> String {
        self.printer.finish()
    }
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}
