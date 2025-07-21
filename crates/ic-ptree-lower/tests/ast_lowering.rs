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

use ic_parse::from_str;
use ic_ptree_lower::from_ast;
use ic_vfs::SourceMap;

fn parse_and_lower_ast(idl: &str) -> ic_ptree::ParseResult {
    let vfs = SourceMap::default();
    let parsed = from_str(idl);
    assert!(parsed.errors.is_empty(), "Parse errors: {:?}", parsed.errors);
    
    from_ast(&parsed, &vfs)
}

#[test]
fn test_basic_struct_ast() {
    let idl = r#"
        struct Point {
            double x;
            double y;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_interface_with_methods_ast() {
    let idl = r#"
        interface Service {
            void process();
            long calculate(in long a, in long b);
            string getName();
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_union_with_default_ast() {
    let idl = r#"
        union Result switch (long) {
            case 0:
                string error;
            case 1:
                long value;
            default:
                boolean dummy;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_nested_modules_ast() {
    let idl = r#"
        module A {
            module B {
                module C {
                    struct Deep {
                        long value;
                    };
                };
            };
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_annotations_ast() {
    let idl = r#"
        @unit("seconds")
        @range(min = 0, max = 3600)
        typedef long Duration;
        
        @deprecated
        struct OldStruct {
            @key
            long id;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_exception_ast() {
    let idl = r#"
        exception NetworkError {
            string message;
            long code;
        };
        
        interface Network {
            void connect() raises (NetworkError);
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_complex_types_ast() {
    let idl = r#"
        typedef sequence<sequence<long>> Matrix;
        typedef map<string, Matrix> NamedMatrices;
        
        struct DataContainer {
            NamedMatrices matrices;
            string labels[10][20];
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_enum_with_annotations_ast() {
    let idl = r#"
        enum Priority {
            @value(0)
            LOW,
            @value(50)
            MEDIUM,
            @value(100)
            HIGH
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_bitmask_ast() {
    let idl = r#"
        bitmask Permissions {
            READ,
            WRITE,
            EXECUTE,
            @value(16)
            DELETE
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_const_expressions_ast() {
    let idl = r#"
        const long BASE = 10;
        const long MULTIPLIER = 5;
        const long RESULT = BASE * MULTIPLIER;
        const string PREFIX = "ID_";
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_valuetype_ast() {
    let idl = r#"
        valuetype Coordinate {
            public double x;
            public double y;
            public double z;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_native_and_forward_decl_ast() {
    let idl = r#"
        native Opaque;
        struct Forward;
        interface IForward;
        
        struct Container {
            Opaque handle;
            Forward fwd;
        };
        
        struct Forward {
            long data;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_interface_inheritance_ast() {
    let idl = r#"
        interface Base {
            void baseMethod();
        };
        
        interface Derived : Base {
            void derivedMethod();
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_any_type_ast() {
    let idl = r#"
        struct Variant {
            any value;
            string type_name;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_fixed_type_ast() {
    let idl = r#"
        struct Financial {
            fixed<12, 2> amount;
            string currency;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_multidimensional_arrays_ast() {
    let idl = r#"
        struct Image {
            octet pixels[1024][768][3];
            string format;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_scoped_names_ast() {
    let idl = r#"
        module Graphics {
            struct Color {
                octet r, g, b;
            };
        };
        
        module UI {
            struct Button {
                Graphics::Color background;
                string label;
            };
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_local_interface_ast() {
    let idl = r#"
        local interface LocalCache {
            void store(in string key, in any value);
            any retrieve(in string key);
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}

#[test]
fn test_custom_annotations_ast() {
    let idl = r#"
        struct validation {
            string pattern;
            long min_length = 0;
            long max_length = 255;
        };
        
        struct User {
            @validation(pattern = "^[a-zA-Z]+$", min_length = 3)
            string username;
        };
    "#;
    
    let ptree = parse_and_lower_ast(idl);
    assert!(ptree.diagnostics().is_none());
}