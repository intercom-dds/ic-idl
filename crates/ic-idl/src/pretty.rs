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

use ic_cli::color::Colorize;
use ic_diagnostic::{error_span, Diag, Label};
use ic_parse::{Error, Reason};

fn emit_error(input: &str, error: &Error) {
    let diag = match &error.reason {
        Reason::Unclosed { span, delimiter } => error_span(
            format!("unclosed delimiter {delimiter}"),
            Label::new(*span).message("unclosed delimiter here"),
        ),

        Reason::Custom(message) => {
            error_span(message, Label::new(error.span).message("unexpected token"))
        }

        Reason::Unexpected => {
            let cause = if let Some(e) = &error.found {
                format!("unexpected {}", e.to_string().red())
            } else {
                "unexpected end of input".to_string()
            };

            let expected = if let Some(e) = &error.expected {
                e.iter()
                    .map(|v| v.to_string().yellow().bold())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "definition".to_string()
            };

            let found = error
                .found
                .as_ref()
                .map_or_else(|| "end of input".to_string(), ToString::to_string);

            error_span(
                format!("{cause}, expected {expected}"),
                Label::new(error.span).message(format!("unexpected {found}")),
            )
        }
    };

    let mut buf = String::new();
    let _ = ic_diagnostic::emit_diagnostic(&mut buf, input, &diag);
    eprintln!("{buf}");
}

pub fn emit_errors(input: &str, errors: &[Error]) {
    for diag in errors {
        emit_error(input, diag);
    }
}
