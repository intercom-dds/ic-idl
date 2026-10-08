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

use ic_codegen_rust::{RustOptions, codegen_rust};
use ic_emit::File as EmitFile;
use ic_hir_lower::{AstInput, from_ast};
use ic_parse::from_str;

const BUILTIN_ANNOTATIONS: &str = include_str!("../../ic-idl/idl/annotations.idl");

/// Compile an IDL source string through the full pipeline and return the
/// generated Rust source code (contents of `lib.rs`).
fn generate_rust(idl: &str) -> String {
    let builtins = from_str(BUILTIN_ANNOTATIONS);
    assert!(
        builtins.errors.is_empty(),
        "builtin parse errors: {:?}",
        builtins.errors
    );

    let user = from_str(idl);
    assert!(
        user.errors.is_empty(),
        "user parse errors: {:?}",
        user.errors
    );

    let hir = from_ast(AstInput::WithBuiltins {
        builtins: builtins.tree,
        user: user.tree,
        include_in_output: false,
    });

    let hir = ic_hir_xform::type_flags::transform(hir);

    let files = codegen_rust(&hir, RustOptions::default());

    files
        .into_iter()
        .find_map(|f| match f {
            EmitFile::Generated { source, .. } => Some(source),
            EmitFile::Dep(_) => None,
        })
        .expect("expected at least one generated file")
}

/// Extract all `///` doc-comment lines from generated source, stripping
/// leading code indentation so assertions can compare the `///` content.
fn doc_lines(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("///").then(|| trimmed.to_string())
        })
        .collect()
}

/// Line comments with indented continuation lines should preserve the indentation so that markdown list continuations are valid.
#[test]
fn line_comment_list_continuation_preserved() {
    let idl = r"
/// Options:
/// - first option
///   with details
///   more details
/// - second option
struct Config {
    long value;
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// Options:"));
    assert!(docs.iter().any(|d| d == "/// - first option"));
    assert!(
        docs.iter().any(|d| d == "///   with details"),
        "expected indented continuation, got: {docs:?}"
    );
    assert!(
        docs.iter().any(|d| d == "///   more details"),
        "expected indented continuation, got: {docs:?}"
    );
    assert!(docs.iter().any(|d| d == "/// - second option"));
}

/// Block comments with `*` prefix should strip the `*` but preserve additional indentation.
#[test]
fn block_comment_star_prefix_stripped() {
    let idl = r"
/**
 * A struct.
 * - list item
 *   continuation
 */
struct S {
    long f;
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// A struct."));
    assert!(docs.iter().any(|d| d == "/// - list item"));
    assert!(
        docs.iter().any(|d| d == "///   continuation"),
        "expected indented continuation, got: {docs:?}"
    );
}

/// Doc comments on struct members should preserve indentation.
#[test]
fn member_doc_preserves_indentation() {
    let idl = r"
struct S {
    /// Field:
    ///   - detail 1
    ///   - detail 2
    long f;
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// Field:"));
    assert!(
        docs.iter().any(|d| d == "///   - detail 1"),
        "expected indented list item, got: {docs:?}"
    );
    assert!(
        docs.iter().any(|d| d == "///   - detail 2"),
        "expected indented list item, got: {docs:?}"
    );
}

/// Empty doc-comment lines should produce bare `///` (not `/// `).
#[test]
fn empty_doc_line() {
    let idl = r"
/// First paragraph.
///
/// Second paragraph.
struct S {
    long f;
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// First paragraph."));
    assert!(
        docs.iter().any(|d| d == "///"),
        "expected bare '///' for empty line, got: {docs:?}"
    );
    assert!(docs.iter().any(|d| d == "/// Second paragraph."));
}

/// Trailing doc comments (`///<`) should also preserve indentation.
#[test]
fn trailing_comment_preserves_indentation() {
    let idl = r"
struct S {
    long f; ///< The field
            ///<   with details
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// The field"));
    assert!(
        docs.iter().any(|d| d == "///   with details"),
        "expected indented trailing comment, got: {docs:?}"
    );
}

/// A single space after `///` is the conventional separator and should be
/// stripped, so `/// text` produces `/// text` (not `///  text`).
#[test]
fn single_space_separator() {
    let idl = r"
/// Hello world
struct S { long f; };
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(
        docs.iter().any(|d| d == "/// Hello world"),
        "expected single-spaced output, got: {docs:?}"
    );
}

/// Code block in doc comment should preserve indentation.
#[test]
fn code_block_preserves_indentation() {
    let idl = r"
/// Example:
/// ```text
///   indented code
/// ```
struct S {
    long f;
};
";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    assert!(docs.iter().any(|d| d == "/// Example:"));
    assert!(docs.iter().any(|d| d == "/// ```text"));
    assert!(
        docs.iter().any(|d| d == "///   indented code"),
        "expected indented code block, got: {docs:?}"
    );
    assert!(docs.iter().any(|d| d == "/// ```"));
}

/// Doc comment block indentation should be removed.
#[test]
fn comment_block_is_unindented() {
    let idl = r"/**
            Example:
            This comment is indented in the IDL file.
            The indentation should be removed.
            But in the following list the indentation should not be removed:
             - Bulletpoint
               - Sub-Bulletpoint
        */
        struct S {
            long f;
        };
    ";
    let source = generate_rust(idl);
    let docs = doc_lines(&source);

    let expected_lines = [
        "/// Example:",
        "/// This comment is indented in the IDL file.",
        "/// The indentation should be removed.",
        "/// But in the following list the indentation should not be removed:",
        "///  - Bulletpoint",
        "///    - Sub-Bulletpoint",
    ];

    assert_eq!(docs.len(), expected_lines.len());
    for (actual, expected) in docs.iter().zip(expected_lines.iter()) {
        assert_eq!(actual, expected);
    }
}
