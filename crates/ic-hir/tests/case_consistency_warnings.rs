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

use insta::assert_snapshot;

mod common;
use common::compile_idl_with_warnings;

#[test]
fn test_simple_type_case_inconsistency() {
    let idl = r"
        struct MyStruct {
            long value;
        };
        
        struct Test {
            mystruct field1;
            MYSTRUCT field2;
            MyStruct field3; // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_qualified_path_case_inconsistency() {
    let idl = r"
        module foo {
            struct Bar {
                long value;
            };
        };
        
        struct Test {
            foo::bar field1;   // Type name wrong
            FOO::Bar field2;   // Module name wrong
            FOO::bar field3;   // Both wrong
            foo::Bar field4;   // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_nested_module_case_inconsistency() {
    let idl = r"
        module Outer {
            module Inner {
                struct Type {
                    long value;
                };
            };
        };
        
        struct Test {
            outer::inner::type field1;  // All wrong
            Outer::INNER::Type field2;  // Middle wrong
            OUTER::Inner::TYPE field3;  // First and last wrong
            Outer::Inner::Type field4;  // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_interface_type_case_inconsistency() {
    let idl = r"
        interface MyInterface {
            struct NestedType {
                long value;
            };
        };
        
        struct Test {
            myinterface::nestedtype field1;
            MyInterface::NESTEDTYPE field2;
            MyInterface::NestedType field3; // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_enum_case_inconsistency() {
    let idl = r"
        enum Status {
            ACTIVE,
            INACTIVE
        };
        
        struct Test {
            status field1;
            STATUS field2;
            Status field3; // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_union_case_inconsistency() {
    let idl = r"
        enum Kind {
            INT,
            STRING
        };
        
        union MyUnion switch (kind) {  // Discriminator type wrong
            case INT: long intValue;
            case STRING: string strValue;
        };
        
        struct Test {
            myunion field1;
            MYUNION field2;
            MyUnion field3; // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_typedef_case_inconsistency() {
    let idl = r"
        typedef long MyLong;
        
        struct Test {
            mylong field1;
            MYLONG field2;
            MyLong field3; // Correct
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}

#[test]
fn test_no_warnings_when_consistent() {
    let idl = r"
        module MyModule {
            struct MyStruct {
                long value;
            };
            
            interface MyInterface {
                struct InnerType {
                    string name;
                };
            };
        };
        
        struct Test {
            MyModule::MyStruct field1;
            MyModule::MyInterface::InnerType field2;
        };
    ";

    assert_snapshot!(compile_idl_with_warnings(idl));
}
