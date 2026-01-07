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

use ic_hir::ResolvedGraph;
use ic_hir_lower::AstInput;
use ic_ptree_lower::from_hir;
use ic_vfs::SourceMap;

fn parse_idl(idl: &str) -> (ResolvedGraph, SourceMap) {
    let mut vfs = SourceMap::default();
    let file_id = vfs.embed(idl);
    let parsed = ic_parse::from_file(file_id, &vfs);
    assert!(parsed.errors.is_empty());

    let hir = ic_hir_lower::from_ast(AstInput::User(parsed.tree));
    assert!(hir.errors.is_empty());
    (hir, vfs)
}

#[test]
fn test_empty_idl() {
    let (hir, vfs) = parse_idl("");
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_deeply_nested_types() {
    let idl = r"
        typedef sequence<sequence<sequence<sequence<long>>>> Deep4;
        typedef map<string, map<string, map<string, long>>> DeepMap;

        struct DeepNesting {
            Deep4 nested_sequences;
            DeepMap nested_maps;
            sequence<map<string, sequence<long>>> mixed;
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_mutually_recursive_structs() {
    let idl = r"
        // Forward declaration
        struct A;

        struct B {
            @shared A a;
        };

        struct A {
            @shared B b;
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_empty_containers() {
    let idl = r"
        struct EmptyStruct {
        };

        exception EmptyException {
        };

        interface EmptyInterface {
        };

        module EmptyModule {
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_numeric_literals() {
    let idl = r#"
        const long DEC = 42;
        const long HEX = 0x2A;
        const long OCT = 052;
        const unsigned long BIG = 4294967295;
        const long long HUGE = 9223372036854775807;
        const float F1 = 3.14;
        const float F2 = 3.14e10;
        const float F3 = 3.14E-10;
        const double D1 = 2.718281828;
        const char C1 = 'A';
        const char C2 = '\n';
        const char C3 = '\x41';
        const string S1 = "Hello, World!";
        const string S2 = "Line 1\nLine 2";
        const boolean B1 = TRUE;
        const boolean B2 = FALSE;
    "#;

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_union_edge_cases() {
    let idl = r"
        // Union with single case
        union SingleCase switch (long) {
            case 1:
                string s;
        };

        // Union with only default
        union OnlyDefault switch (long) {
            default:
                long value;
        };

        // Union with multiple labels per case
        union MultiLabel switch (long) {
            case 1:
            case 2:
            case 3:
            case 4:
            case 5:
                string many;
            default:
                null;
        };

        // Union with enum discriminator
        enum Color { RED, GREEN, BLUE };
        union ColorUnion switch (Color) {
            case RED:
                long r;
            case GREEN:
                long g;
            case BLUE:
                long b;
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_interface_edge_cases() {
    let idl = r"
        // Interface with no return value
        interface VoidMethods {
            void method1();
            void method2(in long x);
        };

        // Interface with complex parameter types
        typedef long Matrix[10][10];
        typedef string StringArray[100];

        interface ComplexParams {
            void processMatrix(in Matrix matrix);
            sequence<string> getStrings(in sequence<long> indices);
            map<string, long> countWords(in StringArray text);
        };

        // Interface with all parameter directions
        interface ParamDirections {
            void allTypes(in long input, out long output, inout long both);
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_annotation_edge_cases() {
    let idl = r"
        // Using various built-in annotations
        @optional
        @deprecated
        struct AnnotatedStruct {
            @range(min = 0, max = 100)
            long field1;

            @optional
            string field2;

            @bit(5)
            octet field3;
        };

        // Multiple annotations on same element
        @deprecated
        @optional
        interface OldInterface {
            @oneway
            void notifyEvent();
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_type_bounds() {
    let idl = r"
        struct Bounded {
            string<0> empty_string;
            string<1> single_char;
            string<65535> max_string;
            sequence<long, 0> empty_seq;
            sequence<long, 1> single_seq;
            sequence<double, 1000000> large_seq;
            map<string, long, 0> empty_map;
            map<string, long, 1> single_map;
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_scoped_references() {
    let idl = r"
        module A {
            struct S1 { long x; };

            module B {
                struct S2 { S1 s; };
                typedef S1 S1Alias;

                module C {
                    struct S3 {
                        S1 s1;
                        S2 s2;
                        ::A::S1 abs_s1;
                        ::A::B::S2 abs_s2;
                    };
                };
            };
        };

        struct Global {
            A::S1 s1;
            A::B::S2 s2;
            A::B::C::S3 s3;
        };
    ";

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_const_expression_references() {
    let idl = r#"
        const long A = 10;
        const long B = A;
        const long C = A + B;
        const long D = C * 2;

        struct ConstArrays {
            long arr1[A];
            long arr2[B];
            long arr3[C];
            long arr4[D];
        };

        const string PREFIX = "ID_";
        const long VERSION = 1;
    "#;

    let (hir, vfs) = parse_idl(idl);
    let ptree = from_hir(&hir, &vfs);
    assert!(ptree.diagnostics().is_none());
}
