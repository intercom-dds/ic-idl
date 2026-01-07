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

use std::fmt::Write;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ic_parse::from_str;

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

/// Generate a large IDL with many definitions
fn generate_large_idl() -> String {
    let mut idl = String::with_capacity(100_000);
    idl.push_str("module LargeTest {\n");

    // Generate many struct definitions
    for i in 0..100 {
        _ = write!(
            idl,
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
        );
    }

    // Generate enum definitions
    for i in 0..20 {
        _ = write!(
            idl,
            r"
    enum TestEnum{i} {{
        VALUE_A_{i},
        VALUE_B_{i},
        VALUE_C_{i},
        VALUE_D_{i},
        VALUE_E_{i}
    }};
"
        );
    }

    // Generate union definitions
    for i in 0..20 {
        _ = write!(
            idl,
            r"
    union TestUnion{i} switch (long) {{
        case 0: long int_val;
        case 1: float float_val;
        case 2: string string_val;
        default: octet default_val;
    }};
"
        );
    }

    // Generate interface definitions
    for i in 0..10 {
        _ = write!(
            idl,
            r"
    interface TestInterface{i} {{
        void operation1(in long param1, in string param2);
        long operation2(in float param1, out string param2);
        TestStruct{i} operation3(inout sequence<long> param1);
        readonly attribute long attr1;
        attribute string attr2;
    }};
"
        );
    }

    // Generate constants
    for i in 0..50 {
        _ = writeln!(idl, "    const long CONST_{i} = {i};");
    }

    // Generate typedefs
    for i in 0..30 {
        _ = writeln!(
            idl,
            "    typedef sequence<TestStruct{seq_i}> TestStructSeq{i};",
            seq_i = i % 100
        );
    }

    idl.push_str("};\n");
    idl
}

/// Generate IDL with many annotations (tests annotation parsing performance)
fn generate_annotation_heavy_idl() -> String {
    let mut idl = String::with_capacity(50_000);
    idl.push_str("module Annotated {\n");

    for i in 0..50 {
        _ = write!(
            idl,
            r#"
    @extensibility(MUTABLE)
    @nested
    @id({i})
    @topic
    @autoid(HASH)
    struct AnnotatedStruct{i} {{
        @id(1) @key @optional long id;
        @id(2) @must_understand string name;
        @id(3) @range(min = 0, max = 100) long value;
        @id(4) @unit("meters") double distance;
        @id(5) @default(42) long defaulted;
    }};
"#
        );
    }

    idl.push_str("};\n");
    idl
}

/// Generate IDL with complex constant expressions
fn generate_expression_heavy_idl() -> String {
    let mut idl = String::with_capacity(20_000);
    idl.push_str("module Expressions {\n");

    // Base constants
    idl.push_str("    const long BASE = 100;\n");
    idl.push_str("    const long MULTIPLIER = 10;\n");
    idl.push_str("    const long OFFSET = 5;\n");

    // Complex expressions
    for i in 0..100 {
        _ = writeln!(
            idl,
            "    const long EXPR_{i} = (BASE + {i}) * MULTIPLIER - OFFSET + ({i} << 2) | ({i} & \
             0xFF);"
        );
    }

    // Nested arithmetic
    for i in 0..50 {
        _ = writeln!(
            idl,
            "    const long NESTED_{i} = ((({i} + 1) * 2) + 3) * 4 + 5;"
        );
    }

    idl.push_str("};\n");
    idl
}

/// Generate IDL with deeply nested types
fn generate_nested_types_idl() -> String {
    let mut idl = String::with_capacity(30_000);
    idl.push_str("module NestedTypes {\n");

    // Deeply nested sequences and maps
    for i in 0..30 {
        _ = writeln!(
            idl,
            "    typedef sequence<sequence<sequence<long>>> DeepSeq{i};"
        );
        _ = writeln!(
            idl,
            "    typedef map<string, map<string, map<string, long>>> DeepMap{i};"
        );
        _ = writeln!(
            idl,
            "    typedef sequence<map<string, sequence<long, 10>, 20>, 30> MixedNested{i};"
        );
    }

    // Structs with nested generic types
    for i in 0..20 {
        _ = write!(
            idl,
            r"
    struct NestedStruct{i} {{
        sequence<sequence<long, 10>, 20> nested_seq;
        map<string, sequence<map<long, string>>> complex_map;
        sequence<map<string, sequence<long>>> mixed;
    }};
"
        );
    }

    idl.push_str("};\n");
    idl
}

/// Generate IDL with many union case labels
fn generate_union_heavy_idl() -> String {
    let mut idl = String::with_capacity(30_000);
    idl.push_str("module Unions {\n");

    for i in 0..20 {
        _ = writeln!(idl, "    union ManyCase{i} switch (long) {{");
        for j in 0..20 {
            _ = writeln!(idl, "        case {j}: long val_{j};");
        }
        idl.push_str("        default: octet other;\n");
        idl.push_str("    };\n");
    }

    // Unions with enum discriminators
    for i in 0..10 {
        _ = write!(
            idl,
            r"
    enum Disc{i} {{ A_{i}, B_{i}, C_{i}, D_{i}, E_{i} }};
    union EnumUnion{i} switch (Disc{i}) {{
        case A_{i}: long a_val;
        case B_{i}: float b_val;
        case C_{i}: string c_val;
        case D_{i}: double d_val;
        default: octet other;
    }};
"
        );
    }

    idl.push_str("};\n");
    idl
}

// Benchmark parsing different input sizes
fn bench_parse_sizes(c: &mut Criterion) {
    let large_idl = generate_large_idl();

    let mut group = c.benchmark_group("parser/size");

    // Small IDL
    group.throughput(Throughput::Bytes(SMALL_IDL.len() as u64));
    group.bench_with_input(BenchmarkId::new("parse", "small"), SMALL_IDL, |b, input| {
        b.iter(|| from_str(std::hint::black_box(input)));
    });

    // Medium IDL
    group.throughput(Throughput::Bytes(MEDIUM_IDL.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "medium"),
        MEDIUM_IDL,
        |b, input| {
            b.iter(|| from_str(std::hint::black_box(input)));
        },
    );

    // Large IDL
    group.throughput(Throughput::Bytes(large_idl.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "large"),
        &large_idl,
        |b, input| {
            b.iter(|| from_str(std::hint::black_box(input)));
        },
    );

    group.finish();
}

// Benchmark parsing specific constructs
fn bench_parse_constructs(c: &mut Criterion) {
    let annotations = generate_annotation_heavy_idl();
    let expressions = generate_expression_heavy_idl();
    let nested = generate_nested_types_idl();
    let unions = generate_union_heavy_idl();

    let mut group = c.benchmark_group("parser/construct");

    group.throughput(Throughput::Bytes(annotations.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "annotations"),
        &annotations,
        |b, input| {
            b.iter(|| from_str(std::hint::black_box(input)));
        },
    );

    group.throughput(Throughput::Bytes(expressions.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "expressions"),
        &expressions,
        |b, input| {
            b.iter(|| from_str(std::hint::black_box(input)));
        },
    );

    group.throughput(Throughput::Bytes(nested.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("parse", "nested_types"),
        &nested,
        |b, input| {
            b.iter(|| from_str(std::hint::black_box(input)));
        },
    );

    group.throughput(Throughput::Bytes(unions.len() as u64));
    group.bench_with_input(BenchmarkId::new("parse", "unions"), &unions, |b, input| {
        b.iter(|| from_str(std::hint::black_box(input)));
    });

    group.finish();
}

criterion_group!(benches, bench_parse_sizes, bench_parse_constructs);
criterion_main!(benches);
