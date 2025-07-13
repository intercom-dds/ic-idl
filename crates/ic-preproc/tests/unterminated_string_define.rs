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

use ic_preproc::{Error, ProcArgs, State};
use ic_vfs::SourceMap;

#[test]
fn test_unterminated_string_in_define() {
    // Test unterminated string in macro definition
    let input = r#"#define MSG "hello world
const string s = MSG;"#;

    let mut vfs = SourceMap::default();
    let mut state = State::new();
    let file_id = vfs.embed(input);

    let iter = ic_preproc::with_state(file_id, ProcArgs::default(), &mut state, &mut vfs);
    
    // Collect all tokens - this should not panic
    let tokens: Vec<_> = iter.collect();
    
    println!("Warnings: {:?}", state.warnings());
    println!("Errors: {:?}", state.errors());
    println!("Token count: {}", tokens.len());
    
    // For now, just check that we don't panic
    println!("Test completed without panic");
}

#[test]
fn test_unterminated_string_macro_expansion() {
    // Test expanding a macro with unterminated string
    let input = r#"#define MSG "hello
const string s = MSG;"#;

    let mut vfs = SourceMap::default();
    let mut state = State::new();
    let file_id = vfs.embed(input);

    let iter = ic_preproc::with_state(file_id, ProcArgs::default(), &mut state, &mut vfs);
    
    // This might be where the panic happens - during expansion
    let tokens: Vec<_> = iter.collect();
    
    println!("Warnings: {:?}", state.warnings());
    println!("Token count: {}", tokens.len());
    println!("Test completed without panic");
}