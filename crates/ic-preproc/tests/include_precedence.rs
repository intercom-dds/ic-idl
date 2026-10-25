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

use std::path::PathBuf;

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/include_precedence")
}

fn preprocess(source: &str, args: ProcArgs) -> State {
    let mut vfs = SourceMap::default();
    let path = fixtures().join("main.idl");
    let id = vfs.embed_with_name(path.to_str().unwrap(), source);
    let mut state = State::new();
    with_state(id, args, &mut state, &mut vfs).for_each(drop);
    state
}

#[test]
fn quoted_includes_prefer_including_directory() {
    let args = ProcArgs::default().include(fixtures().join("first"));
    let state = preprocess("#include \"common.idl\"\n", args);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("LOCAL_INCLUDED"));
    assert!(!state.is_defined("FIRST_INCLUDED"));
}

#[test]
fn angle_includes_skip_including_directory() {
    let args = ProcArgs::default().include(fixtures().join("first"));
    let state = preprocess("#include <common.idl>\n", args);

    assert!(state.errors().is_empty());
    assert!(state.is_defined("FIRST_INCLUDED"));
    assert!(!state.is_defined("LOCAL_INCLUDED"));

    let state = preprocess("#include <common.idl>\n", ProcArgs::default());
    assert_eq!(state.errors().len(), 1);
    assert!(!state.is_defined("LOCAL_INCLUDED"));
}

#[test]
fn include_directories_preserve_order_with_duplicates() {
    for (first, second, expected, unexpected) in [
        ("first", "second", "FIRST_INCLUDED", "SECOND_INCLUDED"),
        ("second", "first", "SECOND_INCLUDED", "FIRST_INCLUDED"),
    ] {
        for source in ["#include <fallback.idl>\n", "#include \"fallback.idl\"\n"] {
            let args = ProcArgs::default()
                .include(fixtures().join(first))
                .includes([fixtures().join(second), fixtures().join(first)])
                .include(fixtures().join(second));
            let state = preprocess(source, args);

            assert!(state.errors().is_empty());
            assert!(state.is_defined(expected));
            assert!(!state.is_defined(unexpected));
        }
    }
}

#[test]
fn quoted_include_cache_uses_including_directory() {
    let source = "#include \"common.idl\"\n#include \"nested/entry.idl\"\n";
    let state = preprocess(source, ProcArgs::default());

    assert!(state.errors().is_empty());
    assert!(state.is_defined("LOCAL_INCLUDED"));
    assert!(state.is_defined("NESTED_INCLUDED"));
}

#[test]
fn quoted_include_cache_does_not_reuse_misses_across_directories() {
    let source = "#include \"only.idl\"\n#include \"nested/entry.idl\"\n";
    let state = preprocess(source, ProcArgs::default());

    assert_eq!(state.errors().len(), 1);
    assert!(state.is_defined("ONLY_INCLUDED"));
}
