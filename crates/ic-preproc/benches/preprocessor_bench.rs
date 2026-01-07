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

use std::fmt::Write;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ic_preproc::{ProcArgs, State, with_state};
use ic_vfs::SourceMap;

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
        _ = writeln!(input, "int result{i} = MAX({i}, SQUARE({}));", i + 1);
        _ = writeln!(input, "int min{i} = MIN({}, {});", i * 2, i * 3);
        _ = writeln!(input, "const char* str{i} = STRINGIFY(value_{i});");
        _ = writeln!(input, "int CONCAT(var_, {i}) = {};", i * 10);
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
        _ = writeln!(input, "#if FEATURE_A && VERSION > {}", i % 10);
        _ = writeln!(input, "    int feature_a_{i} = {i};");
        input.push_str("#elif FEATURE_B\n");
        _ = writeln!(input, "    int feature_b_{i} = {};", i * 2);
        input.push_str("#else\n");
        _ = writeln!(input, "    int default_{i} = {};", i * 3);
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
        _ = writeln!(input, "MAKE_FUNC(test_{i})");
        _ = writeln!(input, "DECLARE_VAR(int, var_{i}, {})", i * 100);
    }

    input
}

// Helper to create input with nested macro expansion
fn create_nested_macro_input() -> String {
    let mut input = String::new();

    // Create nested macro definitions
    input.push_str("#define A(x) B(x)\n");
    input.push_str("#define B(x) C(x)\n");
    input.push_str("#define C(x) D(x)\n");
    input.push_str("#define D(x) ((x) * (x) * (x))\n");

    // Use the nested macros
    for i in 0..200 {
        _ = writeln!(input, "int result{i} = A({i});");
    }

    input
}

// Helper to create input simulating include guard patterns
fn create_include_guard_input() -> String {
    let mut input = String::new();

    for i in 0..50 {
        _ = writeln!(input, "#define HEADER_{i}_H");
        _ = writeln!(input, "#ifndef HEADER_{i}_H");
        _ = writeln!(input, "struct Data{i} {{ int value; }};");
        input.push_str("#endif\n");
    }

    input
}

fn bench_simple_tokenization(c: &mut Criterion) {
    let input = "int main() { return 0; }\n".repeat(1000);

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("simple_tokenization", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_macro_expansion(c: &mut Criterion) {
    let input = create_macro_heavy_input();

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("macro_expansion", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_conditional_processing(c: &mut Criterion) {
    let input = create_conditional_heavy_input();

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("conditionals", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_token_manipulation(c: &mut Criterion) {
    let input = create_token_manipulation_input();

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("token_manipulation", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_nested_macros(c: &mut Criterion) {
    let input = create_nested_macro_input();

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("nested_macros", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_include_guards(c: &mut Criterion) {
    let input = create_include_guard_input();

    let mut group = c.benchmark_group("preproc");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("include_guards", |b| {
        b.iter(|| {
            let mut vfs = SourceMap::default();
            let id = vfs.embed(&input);
            let args = ProcArgs::default();
            let mut state = State::new();

            let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
            std::hint::black_box(tokens)
        });
    });

    group.finish();
}

fn bench_all_workloads(c: &mut Criterion) {
    let simple = "int main() { return 0; }\n".repeat(1000);
    let macros = create_macro_heavy_input();
    let conditionals = create_conditional_heavy_input();
    let token_manip = create_token_manipulation_input();
    let nested = create_nested_macro_input();
    let guards = create_include_guard_input();

    let mut group = c.benchmark_group("preproc_comparison");

    for (name, input) in [
        ("simple", simple.as_str()),
        ("macros", macros.as_str()),
        ("conditionals", conditionals.as_str()),
        ("token_manipulation", token_manip.as_str()),
        ("nested_macros", nested.as_str()),
        ("include_guards", guards.as_str()),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), input, |b, input| {
            b.iter(|| {
                let mut vfs = SourceMap::default();
                let id = vfs.embed(input);
                let args = ProcArgs::default();
                let mut state = State::new();

                let tokens: Vec<_> = with_state(id, args, &mut state, &mut vfs).collect();
                std::hint::black_box(tokens)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_tokenization,
    bench_macro_expansion,
    bench_conditional_processing,
    bench_token_manipulation,
    bench_nested_macros,
    bench_include_guards,
    bench_all_workloads,
);
criterion_main!(benches);
