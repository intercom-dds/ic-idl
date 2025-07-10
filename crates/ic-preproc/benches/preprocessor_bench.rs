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

#![feature(test)]

extern crate test;

use std::hint::black_box;

use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;
use test::Bencher;

// Helper to create test input with macros
fn create_macro_heavy_input() -> String {
    let mut input = String::new();

    // Define some macros
    input.push_str("#define MAX(a, b) ((a) > (b) ? (a) : (b))\n");
    input.push_str("#define MIN(a, b) ((a) < (b) ? (a) : (b))\n");
    input.push_str("#define SQUARE(x) ((x) * (x))\n");
    input.push_str("#define STRINGIFY(x) #x\n");
    input.push_str("#define CONCAT(a, b) a ## b\n");
    input.push_str("#define DEBUG_PRINT(fmt, ...) printf(fmt, __VA_ARGS__)\n");

    // Use macros repeatedly
    for i in 0..100 {
        input.push_str(&format!(
            "int result{} = MAX({}, SQUARE({}));\n",
            i,
            i,
            i + 1
        ));
        input.push_str(&format!("int min{} = MIN({}, {});\n", i, i * 2, i * 3));
        input.push_str(&format!("const char* str{} = STRINGIFY(value_{});\n", i, i));
        input.push_str(&format!("int CONCAT(var_, {}) = {};\n", i, i * 10));
    }

    input
}

// Helper to create input with nested conditionals
fn create_conditional_heavy_input() -> String {
    let mut input = String::new();

    input.push_str("#define FEATURE_A 1\n");
    input.push_str("#define FEATURE_B 0\n");
    input.push_str("#define VERSION 5\n");

    for i in 0..50 {
        input.push_str(&format!("#if FEATURE_A && VERSION > {}\n", i % 10));
        input.push_str(&format!("    int feature_a_{} = {};\n", i, i));
        input.push_str("#elif FEATURE_B\n");
        input.push_str(&format!("    int feature_b_{} = {};\n", i, i * 2));
        input.push_str("#else\n");
        input.push_str(&format!("    int default_{} = {};\n", i, i * 3));
        input.push_str("#endif\n");
    }

    input
}

// Helper to create input with token pasting and stringification
fn create_token_manipulation_input() -> String {
    let mut input = String::new();

    input.push_str("#define MAKE_FUNC(name) \\\n");
    input.push_str("    int func_ ## name(int x) { \\\n");
    input.push_str("        return x * x; \\\n");
    input.push_str("    }\n");

    input.push_str("#define DECLARE_VAR(type, name, value) \\\n");
    input.push_str("    type name = value; \\\n");
    input.push_str("    const char* name ## _str = #name;\n");

    for i in 0..100 {
        input.push_str(&format!("MAKE_FUNC(test_{})\n", i));
        input.push_str(&format!("DECLARE_VAR(int, var_{}, {})\n", i, i * 100));
    }

    input
}

#[bench]
fn bench_simple_tokenization(b: &mut Bencher) {
    let input = "int main() { return 0; }\n".repeat(1000);

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}

#[bench]
fn bench_macro_expansion(b: &mut Bencher) {
    let input = create_macro_heavy_input();

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}

#[bench]
fn bench_conditional_processing(b: &mut Bencher) {
    let input = create_conditional_heavy_input();

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}

#[bench]
fn bench_token_manipulation(b: &mut Bencher) {
    let input = create_token_manipulation_input();

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}

#[bench]
fn bench_nested_macro_expansion(b: &mut Bencher) {
    let mut input = String::new();

    // Create nested macro definitions
    input.push_str("#define A(x) B(x)\n");
    input.push_str("#define B(x) C(x)\n");
    input.push_str("#define C(x) D(x)\n");
    input.push_str("#define D(x) ((x) * (x) * (x))\n");

    // Use the nested macros
    for i in 0..200 {
        input.push_str(&format!("int result{} = A({});\n", i, i));
    }

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}

#[bench]
fn bench_include_processing(b: &mut Bencher) {
    // This benchmark simulates the overhead of include directives
    // without actual file I/O
    let mut input = String::new();

    for i in 0..50 {
        input.push_str(&format!("#define HEADER_{}_H\n", i));
        input.push_str(&format!("#ifndef HEADER_{}_H\n", i));
        input.push_str(&format!("struct Data{} {{ int value; }};\n", i));
        input.push_str("#endif\n");
    }

    b.iter(|| {
        let mut vfs = SourceMap::default();
        let id = vfs.embed(&input);
        let args = ProcArgs::default();
        let mut state = State::new();

        let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
        black_box(tokens);
    });
}
