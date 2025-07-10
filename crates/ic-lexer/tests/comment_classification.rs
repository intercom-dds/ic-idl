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

use ic_lexer::cursor::Cursor;
use ic_lexer::token::{Kind, Token};
use ic_vfs::SourceMap;

fn scan(input: &str) -> Vec<Token> {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(input);
    let src = vfs.source(id);
    let mut cursor = Cursor::new(src, id);

    let mut tokens = vec![];
    while let Some(t) = cursor.next() {
        tokens.push(t);
    }
    tokens
}

#[test]
fn test_regular_vs_doc_comments() {
    // Regular comments are consumed without producing tokens
    assert_eq!(scan("/* regular */").len(), 0);
    assert_eq!(scan("/**/").len(), 0);
    assert_eq!(scan("/***/").len(), 0);
    assert_eq!(scan("/****/").len(), 0);
    assert_eq!(scan("/*****/").len(), 0);

    // Documentation comments produce Comment tokens
    let doc_comments = vec![
        "/** doc comment */",
        "/** This is documentation */",
        "/*! doc comment */",
        "/*! This is documentation */",
        "/**\n * Multi-line\n * doc comment\n */",
        "/*!\n * Multi-line\n * doc comment\n */",
    ];

    for comment in doc_comments {
        let tokens = scan(comment);
        assert_eq!(
            tokens.len(),
            1,
            "Doc comment '{}' should produce 1 token",
            comment
        );
        assert_eq!(tokens[0].kind, Kind::Comment);
    }
}

#[test]
fn test_comment_edge_cases() {
    // Comments followed by code
    assert_eq!(scan("/**/ text").len(), 1); // Just the 'text' identifier

    // Multiple comments
    assert_eq!(scan("/**/ /**/").len(), 0); // Two regular comments
    assert_eq!(scan("/** doc */ /** doc */").len(), 2); // Two doc comments

    // Block comments don't nest in C-style languages
    assert_eq!(scan("/* /* */ */").len(), 2); // First */ ends comment, second */ tokenized
    assert_eq!(scan("/** /** */ */").len(), 3); // Doc comment, then */ tokenized
}

#[test]
fn test_many_stars() {
    // Verify that comments with many stars are still regular comments
    for stars in [10, 50, 100, 500] {
        let pattern = format!("/*{}*/", "*".repeat(stars));
        assert_eq!(scan(&pattern).len(), 0);
    }
}
