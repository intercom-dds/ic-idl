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
use ic_parse::from_str;
use ic_ptree_lower::from_hir;
use ic_vfs::SourceMap;

fn parse_and_lower_hir(idl: &str) -> (ic_ptree::ParseResult, ResolvedGraph) {
    let vfs = SourceMap::default();
    let parsed = from_str(idl);
    assert!(parsed.errors.is_empty(), "Parse errors: {:?}", parsed.errors);
    
    let hir = ic_hir::from_ast(parsed.tree);
    assert!(hir.errors.is_empty(), "HIR errors: {:?}", hir.errors);
    
    let ptree = from_hir(&hir, &vfs);
    (ptree, hir)
}

#[test]
fn test_primitive_types() {
    let idl = r#"
        struct PrimitiveTypes {
            boolean b;
            char c;
            wchar wc;
            octet o;
            short s;
            unsigned short us;
            long l;
            unsigned long ul;
            long long ll;
            unsigned long long ull;
            float f;
            double d;
            long double ld;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_string_types() {
    let idl = r#"
        struct StringTypes {
            string s1;
            string<100> s2;
            wstring ws1;
            wstring<200> ws2;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_sequence_types() {
    let idl = r#"
        struct SequenceTypes {
            sequence<long> s1;
            sequence<long, 10> s2;
            sequence<string> s3;
            sequence<sequence<double>> nested;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_array_types() {
    let idl = r#"
        struct ArrayTypes {
            long a1[10];
            double a2[5][3];
            string a3[100];
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_map_types() {
    let idl = r#"
        struct MapTypes {
            map<string, long> m1;
            map<long, string, 100> m2;
            map<string, sequence<double>> m3;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_enum_lowering() {
    let idl = r#"
        enum Color {
            RED,
            GREEN = 5,
            BLUE
        };
        
        enum Status {
            @value(100)
            SUCCESS,
            @value(200)
            FAILURE
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_bitmask_lowering() {
    let idl = r#"
        bitmask Flags {
            FLAG_A,
            FLAG_B,
            @value(8)
            FLAG_C
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_struct_inheritance() {
    let idl = r#"
        struct Base {
            long id;
        };
        
        struct Derived : Base {
            string name;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_union_lowering() {
    let idl = r#"
        union MyUnion switch (long) {
            case 1:
                long l;
            case 2:
            case 3:
                string s;
            default:
                double d;
        };
        
        union EnumUnion switch (Color) {
            case RED:
                long red_value;
            case GREEN:
                string green_value;
        };
        
        enum Color {
            RED, GREEN, BLUE
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_exception_lowering() {
    let idl = r#"
        exception MyError {
            string message;
            long code;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_interface_lowering() {
    let idl = r#"
        interface Calculator {
            long add(in long a, in long b);
            void divide(in double a, in double b, out double result);
            string format(in string fmt, inout long value);
        };
        
        local interface LocalService {
            void process();
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_const_lowering() {
    let idl = r#"
        const long MAX_SIZE = 100;
        const double PI = 3.14159;
        const string NAME = "test";
        const boolean ENABLED = TRUE;
        const char DELIMITER = ',';
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_typedef_lowering() {
    let idl = r#"
        typedef long Id;
        typedef sequence<string> StringList;
        typedef string Name[50];
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_module_lowering() {
    let idl = r#"
        module Outer {
            struct Point {
                double x;
                double y;
            };
            
            module Inner {
                interface Service {
                    Point getOrigin();
                };
            };
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_forward_declarations() {
    let idl = r#"
        struct Forward;
        
        struct Container {
            Forward f;
        };
        
        struct Forward {
            long value;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_annotations() {
    let idl = r#"
        @unit("meters")
        struct Distance {
            @range(min = 0, max = 1000)
            double value;
        };
        
        @deprecated
        interface OldService {
            void oldMethod();
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_annotation_declarations() {
    let idl = r#"
        @annotation
        struct custom {
            string description;
            long priority = 0;
        };
        
        @custom(description = "Test struct", priority = 5)
        struct TestStruct {
            long id;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_valuetype_lowering() {
    let idl = r#"
        valuetype Money {
            public double amount;
            public string currency;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_native_type() {
    let idl = r#"
        native Handle;
        
        struct Container {
            Handle h;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_self_referential_struct() {
    let idl = r#"
        struct Node {
            long value;
            Node next;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_const_references() {
    let idl = r#"
        const long SIZE = 10;
        const long DOUBLE_SIZE = SIZE * 2;
        
        struct Buffer {
            octet data[SIZE];
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_complex_nested_types() {
    let idl = r#"
        typedef sequence<string> StringSeq;
        typedef map<string, StringSeq> StringMap;
        
        struct Complex {
            StringMap data;
            sequence<map<long, sequence<double>>> nested;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_any_and_fixed_types() {
    let idl = r#"
        struct SpecialTypes {
            any a;
            fixed<10, 2> price;
        };
    "#;
    
    let (ptree, _) = parse_and_lower_hir(idl);
    assert!(ptree.diagnostics().is_none());
}