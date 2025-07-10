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

use ic_parse::from_file;
use ic_preproc::{ProcArgs, preprocess};
use ic_syntax::Item;
use ic_vfs::SourceMap;

#[test]
fn test_trailing_comments_in_tokens() {
    let mut vfs = SourceMap::default();

    // Simple test case with trailing comment
    let content = "int value; /// trailing comment";
    let file = vfs.embed(content);

    // Run preprocessor to get tokens
    let args = ProcArgs::default();
    let token_iter = preprocess(file, args, &mut vfs);
    let tokens: Vec<_> = token_iter.collect();

    // Check that we have the expected tokens
    let comment_tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| matches!(t.kind, ic_lexer::token::Kind::Comment { .. }))
        .collect();

    assert_eq!(comment_tokens.len(), 1);

    // Verify the comment is marked as trailing
    if let ic_lexer::token::Kind::Comment { trailing } = comment_tokens[0].kind {
        assert!(trailing, "Comment should be marked as trailing");
    } else {
        panic!("Expected Comment token");
    }
}

#[test]
fn test_leading_vs_trailing_comments() {
    let mut vfs = SourceMap::default();

    let content = r#"/// Leading comment
int a;
int b; /// Trailing comment
    /// Another leading comment
int c;"#;

    let file = vfs.embed(content);

    // Run preprocessor to get tokens
    let args = ProcArgs::default();
    let token_iter = preprocess(file, args, &mut vfs);
    let tokens: Vec<_> = token_iter.collect();

    // Collect all comment tokens with their trailing flag
    let comments: Vec<bool> = tokens
        .into_iter()
        .filter_map(|t| match t.kind {
            ic_lexer::token::Kind::Comment { trailing } => Some(trailing),
            _ => None,
        })
        .collect();

    assert_eq!(comments.len(), 3);
    assert_eq!(comments[0], false); // Leading comment
    assert_eq!(comments[1], true); // Trailing comment
    assert_eq!(comments[2], false); // Leading comment (with indentation)
}

#[test]
fn test_struct_field_trailing_comments() {
    let mut vfs = SourceMap::default();

    let content = r#"
struct Example {
    long field1; /// This is a trailing comment for field1
    string field2; /// This is a trailing comment for field2
};"#;

    let file = vfs.embed(content);

    // Parse using from_file
    let args = ProcArgs::default();
    let (parse_result, errors) = from_file(file, args, &mut vfs);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert_eq!(parse_result.tree.len(), 1);

    // Check that the struct has the expected fields with annotations
    if let Item::StructValue(s) = &parse_result.tree[0] {
        assert_eq!(s.members.len(), 2);

        // Check field1 has a trailing comment annotation
        let field1 = &s.members[0];
        assert_eq!(field1.annotations.len(), 1);
        assert_eq!(field1.annotations[0].ident.segments[0].name, "doc");

        // Check field2 has a trailing comment annotation
        let field2 = &s.members[1];
        assert_eq!(field2.annotations.len(), 1);
        assert_eq!(field2.annotations[0].ident.segments[0].name, "doc");
    } else {
        panic!("Expected a struct item");
    }
}

#[test]
fn test_leading_vs_trailing_struct_comments() {
    let mut vfs = SourceMap::default();

    let content = r#"
struct MixedComments {
    /// Leading comment for field1
    long field1;
    long field2; /// Trailing comment for field2
    /// Leading comment for field3
    /// Another line of leading comment
    string field3;
};"#;

    let file = vfs.embed(content);

    // Parse using from_file
    let args = ProcArgs::default();
    let (parse_result, errors) = from_file(file, args, &mut vfs);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert_eq!(parse_result.tree.len(), 1);

    // Check that the struct has the expected fields with annotations
    if let Item::StructValue(s) = &parse_result.tree[0] {
        assert_eq!(s.members.len(), 3);

        // field1 should have 1 leading comment
        let field1 = &s.members[0];
        assert_eq!(field1.annotations.len(), 1);

        // field2 should have 1 trailing comment
        let field2 = &s.members[1];
        assert_eq!(field2.annotations.len(), 1);

        // field3 should have 2 leading comments
        let field3 = &s.members[2];
        assert_eq!(field3.annotations.len(), 2);
    } else {
        panic!("Expected a struct item");
    }
}
