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

#![allow(clippy::print_stderr)]

use std::collections::HashMap;
use std::fmt::Write;

use ic_cli::color::Colorize as _;
use ic_diagnostic::{Diag, DiagnosticEmitter, Label, error_span, warn_span};
use ic_lexer::token::Kind;
use ic_parse::Reason;
use ic_vfs::{SourceMap, Span};

use crate::util::Error;

fn format_slice<T: std::fmt::Display>(kind: &[T]) -> String {
    match kind.split_last() {
        Some((last, rest)) if !rest.is_empty() => {
            let body = rest
                .iter()
                .map(|v| format!("{v}").yellow().to_string())
                .collect::<Vec<_>>()
                .join(", ");

            format!("{body} or {}", format!("{last}").yellow())
        }
        Some((last, _)) => format!("{last}").yellow().to_string(),
        _ => String::new(),
    }
}

fn with_expansion(diag: Diag, span: Span, exp: &HashMap<Span, ic_preproc::ExpansionInfo>) -> Diag {
    if let Some(info) = exp.get(&span) {
        diag.label(
            Label::new(info.invocation_span)
                .message(format!("expanded from macro '{}'", info.macro_name))
                .color(ic_diagnostic::Color::Cyan),
        )
    } else {
        diag
    }
}

fn parse_error_to_diag(
    e: &ic_parse::Error,
    warn: bool,
    exp: &HashMap<Span, ic_preproc::ExpansionInfo>,
) -> Diag {
    let diag_fn = if warn { warn_span } else { error_span };

    let (msg, span, label): (String, Span, String) = match &e.reason {
        Reason::Unclosed { span, delimiter } => (
            format!("unclosed delimiter {delimiter}"),
            *span,
            "unclosed delimiter here".into(),
        ),
        Reason::Custom(message) => (
            message.clone(),
            e.span,
            e.label.unwrap_or("unexpected token").into(),
        ),
        Reason::Unexpected => {
            let cause = e
                .found
                .as_ref()
                .map_or("unexpected end of input".to_string(), |k| {
                    format!("unexpected {}", k.red())
                });

            let expected = if let Some(l) = e.label {
                l.yellow().to_string()
            } else if let Some(kinds) = &e.expected {
                if kinds.contains(&Kind::Eoi) {
                    "top-level definition".yellow().to_string()
                } else {
                    format_slice(kinds)
                }
            } else {
                "definition".yellow().to_string()
            };

            let found = e
                .found
                .as_ref()
                .map_or_else(|| "end of input".to_string(), ToString::to_string);

            (
                format!("{cause}, expected {expected}"),
                e.span,
                format!("unexpected {found}"),
            )
        }
    };

    with_expansion(diag_fn(msg, Label::new(span).message(label)), span, exp)
}

use crate::config::ErrorFormat;

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn fmt_errors(
    errors: &[Error],
    vfs: &SourceMap,
    exp: &HashMap<Span, ic_preproc::ExpansionInfo>,
    format: ErrorFormat,
) -> String {
    let mut buf = String::new();
    let prefix = "error:".red().bold();
    let mut emitter = DiagnosticEmitter::auto();

    for err in errors {
        if !buf.is_empty() && format == ErrorFormat::Human {
            _ = writeln!(&mut buf);
        }

        _ = match err {
            Error::Parse(e) => {
                let diag = parse_error_to_diag(e, false, exp);
                emit_diag(&mut emitter, &mut buf, vfs, &diag, format)
            }
            Error::Preproc(e) => {
                let diag = preproc_to_diag(e, vfs, false, exp);
                emit_diag(&mut emitter, &mut buf, vfs, &diag, format)
            }
            Error::Lower(diag) => emit_diag(&mut emitter, &mut buf, vfs, diag, format),
            Error::Io(e) => writeln!(&mut buf, "{prefix} {e}"),
        };
    }
    buf
}

fn emit_diag(
    emitter: &mut DiagnosticEmitter,
    buf: &mut String,
    vfs: &SourceMap,
    diag: &Diag,
    format: ErrorFormat,
) -> std::fmt::Result {
    match format {
        ErrorFormat::Human => emitter.emit(buf, vfs, diag),
        ErrorFormat::Short => emitter.emit_compact(buf, vfs, diag),
    }
}

fn preproc_to_diag(
    e: &ic_preproc::Error,
    vfs: &SourceMap,
    warn: bool,
    exp: &HashMap<Span, ic_preproc::ExpansionInfo>,
) -> Diag {
    let diag_fn = if warn { warn_span } else { error_span };
    let span = e.span();
    let (msg, label): (String, &str) = match e {
        ic_preproc::Error::Note { tokens, .. } => {
            let dir = if warn { "#warning" } else { "#error" };
            let msg = if tokens.is_empty() {
                format!("{dir} directive")
            } else {
                let text = tokens
                    .iter()
                    .map(|t| &vfs.source_str(t.span.start.file_id)[t.span.range()])
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{dir} directive: {text}")
            };
            (msg, "here")
        }
        ic_preproc::Error::Extraneous { directive, .. } => (
            format!("extra tokens after #{directive} directive"),
            "extraneous tokens",
        ),
        ic_preproc::Error::Syntax { message, .. } | ic_preproc::Error::Expr { message, .. } => {
            ((*message).to_string(), "here")
        }
    };

    with_expansion(diag_fn(msg, Label::new(span).message(label)), span, exp).code("preprocessor")
}

#[must_use]
pub fn fmt_warnings(warnings: &[Diag], vfs: &SourceMap, format: ErrorFormat) -> String {
    let mut buf = String::new();
    let mut emitter = DiagnosticEmitter::auto();
    for diag in warnings {
        if !buf.is_empty() && format == ErrorFormat::Human {
            _ = writeln!(&mut buf);
        }
        _ = emit_diag(&mut emitter, &mut buf, vfs, diag, format);
    }
    buf
}
