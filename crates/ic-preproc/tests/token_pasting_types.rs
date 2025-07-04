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

use ic_lexer::token::{Base, Kind};
use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

#[test]
fn token_pasting_produces_correct_types() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r"
            #define PASTE(a, b) a##b
            
            // Should produce identifiers
            PASTE(foo, bar)
            PASTE(get_, value)
            
            // Should produce numbers
            PASTE(123, 456)
            PASTE(0x, FF)
            
            // Should produce operators
            PASTE(+, +)
            PASTE(=, =)
            PASTE(<, =)
            PASTE(>, =)
            PASTE(<, <)
            PASTE(>, >)
            PASTE(&, &)
            PASTE(|, |)
            
            // Should produce keywords
            PASTE(inter, face)
            PASTE(mod, ule)
        ",
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
    
    // Filter out newlines for easier checking
    let tokens: Vec<_> = tokens.into_iter()
        .filter(|t| t.kind != Kind::Newline)
        .collect();
    
    // Check the produced tokens
    let mut i = 0;
    
    // foobar - identifier
    assert_eq!(tokens[i].kind, Kind::Ident);
    assert_eq!(&vfs.source_str(tokens[i].span.start.file_id)[tokens[i].span.range()], "foobar");
    i += 1;
    
    // get_value - identifier
    assert_eq!(tokens[i].kind, Kind::Ident);
    assert_eq!(&vfs.source_str(tokens[i].span.start.file_id)[tokens[i].span.range()], "get_value");
    i += 1;
    
    // 123456 - number
    assert_eq!(tokens[i].kind, Kind::Number { base: Base::Decimal });
    assert_eq!(&vfs.source_str(tokens[i].span.start.file_id)[tokens[i].span.range()], "123456");
    i += 1;
    
    // 0xFF - hex number
    assert_eq!(tokens[i].kind, Kind::Number { base: Base::Hexadecimal });
    assert_eq!(&vfs.source_str(tokens[i].span.start.file_id)[tokens[i].span.range()], "0xFF");
    i += 1;
    
    // ++ - identifier (not a single token in IDL)
    assert_eq!(tokens[i].kind, Kind::Ident);
    assert_eq!(&vfs.source_str(tokens[i].span.start.file_id)[tokens[i].span.range()], "++");
    i += 1;
    
    // == - operator
    assert_eq!(tokens[i].kind, Kind::EqEq);
    i += 1;
    
    // <= - operator
    assert_eq!(tokens[i].kind, Kind::LtEq);
    i += 1;
    
    // >= - operator
    assert_eq!(tokens[i].kind, Kind::GtEq);
    i += 1;
    
    // << - operator
    assert_eq!(tokens[i].kind, Kind::LShift);
    i += 1;
    
    // >> - operator
    assert_eq!(tokens[i].kind, Kind::RShift);
    i += 1;
    
    // && - operator
    assert_eq!(tokens[i].kind, Kind::And);
    i += 1;
    
    // || - operator
    assert_eq!(tokens[i].kind, Kind::Or);
    i += 1;
    
    // interface - keyword
    assert!(matches!(tokens[i].kind, Kind::Keyword(_)));
    i += 1;
    
    // module - keyword
    assert!(matches!(tokens[i].kind, Kind::Keyword(_)));
}