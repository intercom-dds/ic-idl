// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

use std::rc::Rc;

use ic_lexer::cursor::Cursor;
use ic_lexer::token::Kind;
use ic_vfs::SourceMap;

#[test]
fn test_no_shift_tokens() {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed("a >> b");
    let source = vfs.source_str(file_id);
    let mut cursor = Cursor::new(Rc::from(source), file_id);

    let tok1 = cursor.advance().unwrap();
    assert_eq!(tok1.kind, Kind::Ident);

    let tok2 = cursor.advance().unwrap();
    assert_eq!(tok2.kind, Kind::Gt);

    let tok3 = cursor.advance().unwrap();
    assert_eq!(tok3.kind, Kind::Gt);

    let tok4 = cursor.advance().unwrap();
    assert_eq!(tok4.kind, Kind::Ident);
}

#[test]
fn test_left_shift_as_two_tokens() {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed("x << y");
    let source = vfs.source_str(file_id);
    let mut cursor = Cursor::new(Rc::from(source), file_id);

    let tok1 = cursor.advance().unwrap();
    assert_eq!(tok1.kind, Kind::Ident);

    let tok2 = cursor.advance().unwrap();
    assert_eq!(tok2.kind, Kind::Lt);

    let tok3 = cursor.advance().unwrap();
    assert_eq!(tok3.kind, Kind::Lt);

    let tok4 = cursor.advance().unwrap();
    assert_eq!(tok4.kind, Kind::Ident);
}

#[test]
fn test_template_closing() {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed("sequence<sequence<T>>");
    let source = vfs.source_str(file_id);
    let mut cursor = Cursor::new(Rc::from(source), file_id);

    let tokens: Vec<_> = std::iter::from_fn(|| cursor.advance())
        .map(|t| t.kind)
        .collect();

    assert_eq!(
        tokens,
        vec![
            Kind::Keyword(ic_lexer::token::Kw::Sequence),
            Kind::Lt,
            Kind::Keyword(ic_lexer::token::Kw::Sequence),
            Kind::Lt,
            Kind::Ident,
            Kind::Gt,
            Kind::Gt,
        ]
    );
}

#[test]
fn test_comparison_operators() {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed("a <= b >= c");
    let source = vfs.source_str(file_id);
    let mut cursor = Cursor::new(Rc::from(source), file_id);

    let tok1 = cursor.advance().unwrap();
    assert_eq!(tok1.kind, Kind::Ident);

    let tok2 = cursor.advance().unwrap();
    assert_eq!(tok2.kind, Kind::LtEq);

    let tok3 = cursor.advance().unwrap();
    assert_eq!(tok3.kind, Kind::Ident);

    let tok4 = cursor.advance().unwrap();
    assert_eq!(tok4.kind, Kind::GtEq);

    let tok5 = cursor.advance().unwrap();
    assert_eq!(tok5.kind, Kind::Ident);
}
