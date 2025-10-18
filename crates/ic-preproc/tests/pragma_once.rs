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

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

#[test]
fn include_pragma_once() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
            #include "tests/pragma_once.idl"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert_eq!(state.warnings().len(), 1);
}

#[test]
fn pragma_once_recursive() {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(
        r#"
            #include "tests/pragma_once_recursive.idl"
        "#,
    );

    let args = ProcArgs::default();
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);

    assert!(state.errors().is_empty());
    assert!(state.warnings().is_empty());
    assert!(state.is_defined("SEEN_1"));
    assert!(state.is_defined("SEEN_2"));
}
