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

use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use ic_cli::color::Colorize;
use ic_diagnostic::{Diag, Label, error_span, warn_span};
use ic_parse::Reason;
use ic_parse::lexer::Kind;
use ic_vfs::{SourceMap, Span};

use crate::util::Error;

fn rel_path(path: &Path) -> PathBuf {
    std::env::current_dir()
        .map_or(path, |v| path.strip_prefix(v).unwrap_or(path))
        .to_path_buf()
}

fn format_slice<T: std::fmt::Display>(kind: &[T]) -> String {
    match kind.split_last() {
        Some((last, rest)) if !rest.is_empty() => {
            let body = rest
                .iter()
                .map(Colorize::yellow)
                .collect::<Vec<_>>()
                .join(", ");

            format!("{body} or {}", last.yellow())
        }
        Some((last, _)) => last.yellow(),
        _ => String::new(),
    }
}

fn parse_error_to_diag(error: &ic_parse::Error, is_warning: bool) -> Diag {
    parse_error_to_diag_with_expansion(error, is_warning, &HashMap::new())
}

fn parse_error_to_diag_with_expansion(
    error: &ic_parse::Error,
    is_warning: bool,
    expansion_info: &HashMap<Span, ic_preproc::ExpansionInfo>,
) -> Diag {
    let diag_fn = if is_warning { warn_span } else { error_span };

    match &error.reason {
        Reason::Unclosed { span, delimiter } => {
            // Check if this error occurred in a macro expansion
            if let Some(info) = expansion_info.get(span) {
                // Primary error points to macro invocation
                let mut diag = diag_fn(
                    format!("unclosed delimiter {delimiter}"),
                    Label::new(info.invocation_span).message("unclosed delimiter here"),
                );
                // Secondary label shows where in the macro definition
                diag = diag.label(
                    Label::new(*span)
                        .message(format!("expanded from macro '{}'", info.macro_name))
                        .color(ic_diagnostic::Color::Cyan),
                );
                diag
            } else {
                diag_fn(
                    format!("unclosed delimiter {delimiter}"),
                    Label::new(*span).message("unclosed delimiter here"),
                )
            }
        }

        Reason::Custom(message) => {
            // Check if this error occurred in a macro expansion
            if let Some(info) = expansion_info.get(&error.span) {
                // Primary error points to macro invocation
                let mut diag = diag_fn(
                    message.clone(),
                    Label::new(info.invocation_span).message("unexpected token"),
                );
                // Secondary label shows where in the macro definition
                diag = diag.label(
                    Label::new(error.span)
                        .message(format!("expanded from macro '{}'", info.macro_name))
                        .color(ic_diagnostic::Color::Cyan),
                );
                diag
            } else {
                diag_fn(
                    message.clone(),
                    Label::new(error.span).message("unexpected token"),
                )
            }
        }

        Reason::Unexpected => {
            let cause = if let Some(e) = &error.found {
                format!("unexpected {}", e.to_string().red())
            } else {
                "unexpected end of input".to_string()
            };

            let expected = if let Some(e) = error.label {
                e.yellow()
            } else if let Some(e) = &error.expected {
                if e.contains(&Kind::Eoi) {
                    "top-level definition".yellow()
                } else {
                    format_slice(e)
                }
            } else {
                "definition".yellow()
            };

            let found = error
                .found
                .as_ref()
                .map_or_else(|| "end of input".to_string(), ToString::to_string);

            // Check if this error occurred in a macro expansion
            if let Some(info) = expansion_info.get(&error.span) {
                // Primary error points to macro invocation
                let mut diag = diag_fn(
                    format!("{cause}, expected {expected}"),
                    Label::new(info.invocation_span).message(format!("unexpected {found}")),
                );
                // Secondary label shows where in the macro definition
                diag = diag.label(
                    Label::new(error.span)
                        .message(format!("expanded from macro '{}'", info.macro_name))
                        .color(ic_diagnostic::Color::Cyan),
                );
                diag
            } else {
                diag_fn(
                    format!("{cause}, expected {expected}"),
                    Label::new(error.span).message(format!("unexpected {found}")),
                )
            }
        }
    }
}

fn emit_error(error: &ic_parse::Error, vfs: &SourceMap, buf: &mut dyn fmt::Write) -> fmt::Result {
    emit_error_with_expansion(error, vfs, buf, &HashMap::new())
}

fn emit_error_with_expansion(
    error: &ic_parse::Error,
    vfs: &SourceMap,
    buf: &mut dyn fmt::Write,
    expansion_info: &HashMap<Span, ic_preproc::ExpansionInfo>,
) -> fmt::Result {
    let diag = parse_error_to_diag_with_expansion(error, false, expansion_info);
    let file = vfs.file_info(error.span.start.file_id);
    let relative = rel_path(&file.path).to_string_lossy().to_string();
    ic_diagnostic::emit_with_source(buf, &relative, &file.source, &diag)
}

pub fn emit_errors(errors: &[Error], vfs: &SourceMap) {
    emit_errors_with_expansion(errors, vfs, &HashMap::new());
}

pub fn emit_errors_with_expansion(
    errors: &[Error],
    vfs: &SourceMap,
    expansion_info: &HashMap<Span, ic_preproc::ExpansionInfo>,
) {
    let mut buf = String::new();
    let prefix = "error:".red().bold();

    for err in errors {
        if !buf.is_empty() {
            _ = writeln!(&mut buf);
        }

        _ = match err {
            Error::Preproc(e) => writeln!(&mut buf, "{prefix} {e}"),
            Error::Io(e) => writeln!(&mut buf, "{prefix} {e}"),
            Error::Custom(e) => writeln!(&mut buf, "{prefix} {e}"),
            Error::Parse(e) => emit_error_with_expansion(e, vfs, &mut buf, expansion_info),
            Error::Diagnostic(diag) => ic_diagnostic::emit_diagnostic(&mut buf, vfs, diag),
        };
    }
    eprintln!("{buf}");
}

pub fn emit_warnings(warnings: &[Diag], vfs: &SourceMap) {
    let mut buf = String::new();
    for diag in warnings {
        if !buf.is_empty() {
            _ = writeln!(&mut buf);
        }
        _ = ic_diagnostic::emit_diagnostic(&mut buf, vfs, diag);
    }

    if !buf.is_empty() {
        eprintln!("{buf}");
    }
}

pub fn parse_error_to_warning(error: &ic_parse::Error) -> Diag {
    parse_error_to_diag(error, true)
}
