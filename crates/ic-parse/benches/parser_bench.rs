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

#![feature(test)]
extern crate test;

use std::hint::black_box;

use ic_parse::from_str;
use test::Bencher;

/// A small but representative IDL sample with common constructs
const SMALL_IDL: &str = r"
module Example {
    struct Point {
        float x;
        float y;
        float z;
    };
    
    interface Calculator {
        float add(in float a, in float b);
        float multiply(in float a, in float b);
        sequence<Point> generatePoints(in long count);
    };
    
    const float PI = 3.14159265359;
    const long MAX_POINTS = 1000000;
    
    enum Status {
        OK,
        ERROR,
        PENDING
    };
    
    typedef sequence<Point> PointList;
    typedef map<string, Point> PointMap;
};
";

/// A medium-sized IDL with more complex types and nesting
const MEDIUM_IDL: &str = r"
module DDS {
    module XTypes {
        @extensibility(FINAL) @nested
        struct StringSTypeDefn {
            octet bound;
        };
        
        @extensibility(FINAL) @nested
        struct StringLTypeDefn {
            unsigned long bound;
        };
        
        @extensibility(APPENDABLE) @nested
        struct PlainCollectionHeader {
            octet equiv_kind;
            octet element_flags;
        };
        
        @extensibility(FINAL) @nested
        struct PlainSequenceSElemDefn {
            PlainCollectionHeader header;
            octet bound;
            octet element_identifier;
        };
        
        @extensibility(FINAL) @nested
        struct PlainSequenceLElemDefn {
            PlainCollectionHeader header;
            unsigned long bound;
            octet element_identifier;
        };
        
        @extensibility(FINAL) @nested
        struct PlainArraySElemDefn {
            PlainCollectionHeader header;
            sequence<octet> array_bound_seq;
            octet element_identifier;
        };
        
        @extensibility(FINAL) @nested
        struct PlainArrayLElemDefn {
            PlainCollectionHeader header;
            sequence<unsigned long> array_bound_seq;
            octet element_identifier;
        };
        
        @extensibility(FINAL) @nested
        struct PlainMapSTypeDefn {
            PlainCollectionHeader header;
            octet bound;
            octet element_identifier;
            octet key_flags;
            octet key_identifier;
        };
        
        @extensibility(FINAL) @nested
        struct PlainMapLTypeDefn {
            PlainCollectionHeader header;
            unsigned long bound;
            octet element_identifier;
            octet key_flags;
            octet key_identifier;
        };
        
        @extensibility(FINAL) @nested
        union TypeIdentifier switch (octet) {
            case TI_STRING8_SMALL:
            case TI_STRING16_SMALL:
                StringSTypeDefn string_sdefn;
            case TI_STRING8_LARGE:
            case TI_STRING16_LARGE:
                StringLTypeDefn string_ldefn;
            case TI_PLAIN_SEQUENCE_SMALL:
                PlainSequenceSElemDefn seq_sdefn;
            case TI_PLAIN_SEQUENCE_LARGE:
                PlainSequenceLElemDefn seq_ldefn;
            case TI_PLAIN_ARRAY_SMALL:
                PlainArraySElemDefn array_sdefn;
            case TI_PLAIN_ARRAY_LARGE:
                PlainArrayLElemDefn array_ldefn;
            case TI_PLAIN_MAP_SMALL:
                PlainMapSTypeDefn map_sdefn;
            case TI_PLAIN_MAP_LARGE:
                PlainMapLTypeDefn map_ldefn;
        };
        
        const octet TI_STRING8_SMALL = 0x70;
        const octet TI_STRING16_SMALL = 0x71;
        const octet TI_STRING8_LARGE = 0x72;
        const octet TI_STRING16_LARGE = 0x73;
        const octet TI_PLAIN_SEQUENCE_SMALL = 0x80;
        const octet TI_PLAIN_SEQUENCE_LARGE = 0x81;
        const octet TI_PLAIN_ARRAY_SMALL = 0x90;
        const octet TI_PLAIN_ARRAY_LARGE = 0x91;
        const octet TI_PLAIN_MAP_SMALL = 0xA0;
        const octet TI_PLAIN_MAP_LARGE = 0xA1;
    };
};
";

/// A large IDL with many definitions to stress test the parser
fn generate_large_idl() -> String {
    let mut idl = String::with_capacity(100_000);
    idl.push_str("module LargeTest {\n");

    // Generate many struct definitions
    for i in 0..100 {
        idl.push_str(&format!(
            r"
    @extensibility(APPENDABLE)
    struct TestStruct{i} {{
        long field1;
        float field2;
        double field3;
        string field4;
        boolean field5;
        octet field6;
        sequence<long> field7;
        sequence<string, 10> field8;
    }};
"
        ));
    }

    // Generate enum definitions
    for i in 0..20 {
        idl.push_str(&format!(
            r"
    enum TestEnum{i} {{
        VALUE_A_{i},
        VALUE_B_{i},
        VALUE_C_{i},
        VALUE_D_{i},
        VALUE_E_{i}
    }};
"
        ));
    }

    // Generate union definitions
    for i in 0..20 {
        idl.push_str(&format!(
            r"
    union TestUnion{i} switch (long) {{
        case 0: long int_val;
        case 1: float float_val;
        case 2: string string_val;
        default: octet default_val;
    }};
"
        ));
    }

    // Generate interface definitions
    for i in 0..10 {
        idl.push_str(&format!(
            r"
    interface TestInterface{i} {{
        void operation1(in long param1, in string param2);
        long operation2(in float param1, out string param2);
        TestStruct{i} operation3(inout sequence<long> param1);
        readonly attribute long attr1;
        attribute string attr2;
    }};
"
        ));
    }

    // Generate constants
    for i in 0..50 {
        idl.push_str(&format!("    const long CONST_{i} = {i};\n"));
    }

    // Generate typedefs
    for i in 0..30 {
        idl.push_str(&format!(
            "    typedef sequence<TestStruct{seq_i}> TestStructSeq{i};\n",
            seq_i = i % 100
        ));
    }

    idl.push_str("};\n");
    idl
}

#[bench]
fn bench_parse_small_idl(b: &mut Bencher) {
    b.iter(|| {
        let result = from_str(black_box(SMALL_IDL));
        black_box(result)
    });
}

#[bench]
fn bench_parse_medium_idl(b: &mut Bencher) {
    b.iter(|| {
        let result = from_str(black_box(MEDIUM_IDL));
        black_box(result)
    });
}

#[bench]
fn bench_parse_large_idl(b: &mut Bencher) {
    let large_idl = generate_large_idl();
    b.iter(|| {
        let result = from_str(black_box(&large_idl));
        black_box(result)
    });
}

/// Benchmark that measures throughput in bytes per second
#[bench]
fn bench_parse_throughput(b: &mut Bencher) {
    let large_idl = generate_large_idl();
    let bytes = large_idl.len();

    b.bytes = bytes as u64;
    b.iter(|| {
        let result = from_str(black_box(&large_idl));
        black_box(result)
    });
}

/// Benchmark preprocessing only (lexing + preprocessing)
#[bench]
fn bench_preproc_only(b: &mut Bencher) {
    use ic_lexer::token::Kind;
    use ic_parse::SourceMap;
    use ic_preproc::ProcArgs;

    let large_idl = generate_large_idl();
    let bytes = large_idl.len();

    b.bytes = bytes as u64;
    b.iter(|| {
        let mut vfs = SourceMap::default();
        let file_id = vfs.embed(black_box(&large_idl));

        let mut state = ic_preproc::State::new();
        let iter = ic_preproc::with_state(file_id, ProcArgs::default(), &mut state, &mut vfs);

        // Collect tokens, filtering out comments and newlines
        let tokens: Vec<_> = iter
            .filter(|t| !matches!(t.kind, Kind::Comment { .. } | Kind::Newline))
            .collect();

        black_box(tokens)
    });
}

// Benchmark parsing only (with pre-collected tokens) - disabled due to arena issues
// in repeated iterations
