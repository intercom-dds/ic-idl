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

use std::path::{Path, PathBuf};

use ic_idl::{CompileDiagnostics, CompileError, Compiler, CompilerOptions, ErrorFormat};
use ic_vfs::{Include, LoadError, Span};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/invalid_utf8")
}

fn first_invalid_utf8(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    std::str::from_utf8(&bytes).unwrap_err().valid_up_to()
}

fn first_nul(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    bytes.iter().position(|&byte| byte == 0).unwrap()
}

fn compile_errors(compiler: &mut Compiler) -> CompileDiagnostics {
    match compiler.compile() {
        Err(CompileError::Diagnostics(diagnostics)) => diagnostics,
        Err(error) => panic!("expected diagnostics, got {error}"),
        Ok(_) => panic!("expected compilation to fail"),
    }
}

fn error_span(diagnostics: &CompileDiagnostics) -> Span {
    assert_eq!(diagnostics.errors.len(), 1);
    let ic_idl::DiagnosticError::Encoding(error) = &diagnostics.errors[0] else {
        panic!("expected an encoding diagnostic");
    };

    error.span
}

fn compiler_for(path: &Path) -> Compiler {
    let mut compiler = Compiler::new(CompilerOptions::default());
    compiler.add_file(path.to_path_buf());
    compiler
}

#[test]
fn invalid_utf8_diagnostic_formats() {
    let path = fixtures().join("invalid.idl");
    let offset = first_invalid_utf8(&path);
    let mut compiler = compiler_for(&path);
    let diagnostics = compile_errors(&mut compiler);
    let span = error_span(&diagnostics);

    assert_eq!(span.start.offset as usize, offset);
    assert_eq!(
        span.end.offset as usize,
        offset + char::REPLACEMENT_CHARACTER.len_utf8()
    );
    assert_eq!(compiler.source_map().path(span.start.file_id), path);

    for (name, format) in [
        ("detailed", ErrorFormat::Detailed),
        ("short", ErrorFormat::Short),
        ("json", ErrorFormat::Json),
    ] {
        let output = diagnostics
            .format(compiler.source_map(), format)
            .replace(fixtures().to_str().unwrap(), "<test>");
        insta::assert_snapshot!(name, output);
    }
}

#[test]
fn invalid_utf8_in_included_file() {
    let path = fixtures().join("main.idl");
    let included = fixtures().join("included.idl");
    let mut compiler = compiler_for(&path);
    let diagnostics = compile_errors(&mut compiler);
    let span = error_span(&diagnostics);

    assert_eq!(compiler.source_map().path(span.start.file_id), included);
    assert_eq!(span.start.offset as usize, first_invalid_utf8(&included));
    let output = diagnostics
        .format(compiler.source_map(), ErrorFormat::Detailed)
        .replace(fixtures().to_str().unwrap(), "<test>");
    insta::assert_snapshot!(output);
}

#[test]
fn invalid_utf8_sequences() {
    for name in [
        "sequence_invalid_byte.idl",
        "sequence_truncated.idl",
        "sequence_incomplete.idl",
        "sequence_after_utf8.idl",
        "sequence_crlf.idl",
        "sequence_utf8_bom.idl",
    ] {
        let path = fixtures().join(name);
        let offset = first_invalid_utf8(&path);
        let mut compiler = compiler_for(&path);
        let diagnostics = compile_errors(&mut compiler);
        let span = error_span(&diagnostics);

        assert_eq!(span.start.offset as usize, offset);
        assert_eq!(
            span.end.offset as usize,
            offset + char::REPLACEMENT_CHARACTER.len_utf8()
        );
        assert!(
            diagnostics
                .format(compiler.source_map(), ErrorFormat::Detailed)
                .contains("invalid UTF-8 sequence")
        );
    }
}

#[test]
fn utf16_byte_order_mark() {
    for name in ["utf16le.idl", "utf16be.idl"] {
        let path = fixtures().join(name);
        let mut compiler = compiler_for(&path);
        let diagnostics = compile_errors(&mut compiler);
        let span = error_span(&diagnostics);

        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 6);

        let output = diagnostics.format(compiler.source_map(), ErrorFormat::Short);
        assert!(output.contains("file is UTF-16 encoded; only UTF-8 is supported"));
    }
}

#[test]
fn utf32_byte_order_mark() {
    for name in ["utf32le.idl", "utf32be.idl"] {
        let path = fixtures().join(name);
        let mut compiler = compiler_for(&path);
        let diagnostics = compile_errors(&mut compiler);
        let span = error_span(&diagnostics);

        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 8);

        let output = diagnostics.format(compiler.source_map(), ErrorFormat::Short);
        assert!(output.contains("file is UTF-32 encoded; only UTF-8 is supported"));
    }
}

#[test]
fn nul_byte() {
    for name in [
        "nul_utf16.idl",
        "nul_block_comment.idl",
        "nul_line_comment.idl",
        "nul_before_invalid.idl",
    ] {
        let path = fixtures().join(name);
        let offset = first_nul(&path);
        let mut compiler = compiler_for(&path);
        let diagnostics = compile_errors(&mut compiler);
        let span = error_span(&diagnostics);

        assert_eq!(span.start.offset as usize, offset);
        assert_eq!(span.end.offset as usize, offset + 1);

        let output = diagnostics.format(compiler.source_map(), ErrorFormat::Short);
        assert!(output.contains("file contains NUL bytes; it may be UTF-16 encoded"));
    }

    let path = fixtures().join("nul_after_invalid.idl");
    let mut compiler = compiler_for(&path);
    let diagnostics = compile_errors(&mut compiler);
    let span = error_span(&diagnostics);

    assert_eq!(span.start.offset as usize, first_invalid_utf8(&path));

    let output = diagnostics.format(compiler.source_map(), ErrorFormat::Short);
    assert!(output.contains("file contains invalid UTF-8"));
}

#[test]
fn cached_invalid_utf8_remains_an_error() {
    let path = fixtures().join("invalid.idl");
    let mut compiler = compiler_for(&path);
    let first = error_span(&compile_errors(&mut compiler));
    let second = error_span(&compile_errors(&mut compiler));

    assert_eq!(first, second);
    assert!(matches!(
        compiler.source_map_mut().open(&path, Include::Static),
        Err(LoadError::InvalidUtf8(error)) if error.span == first
    ));
}

#[test]
fn valid_utf8_replacement_character_is_accepted() {
    let mut compiler = compiler_for(&fixtures().join("valid.idl"));

    assert!(compiler.compile().is_ok());
}

#[test]
fn missing_file_remains_an_io_error() {
    let mut compiler = compiler_for(&fixtures().join("missing.idl"));

    assert!(matches!(
        compiler.compile(),
        Err(CompileError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound
    ));
}
