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

#[test]
fn circular_struct_inheritance() {
    let output = common::compile_idl_with_warnings(
        r"
        struct A;
        struct B;
        
        struct A : B {
            long x;
        };
        
        struct B : A {
            long y;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn circular_interface_inheritance() {
    let output = common::compile_idl_with_warnings(
        r"
        interface Foo;
        interface Bar;
        
        interface Foo : Bar {
            void method1();
        };
        
        interface Bar : Foo {
            void method2();
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn longer_circular_chain() {
    let output = common::compile_idl_with_warnings(
        r"
        struct A;
        struct B;
        struct C;
        
        struct A : B {
            long a;
        };
        
        struct B : C {
            long b;
        };
        
        struct C : A {
            long c;
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn multiple_interface_inheritance() {
    let output = common::compile_idl_with_warnings(
        r"
        interface A;
        interface B;
        interface C;
        
        interface A : B, C {
            void methodA();
        };
        
        interface B {
            void methodB();
        };
        
        interface C : A {
            void methodC();
        };
        ",
    );
    assert_snapshot!(output);
}

#[test]
fn self_referential_interface() {
    let output = common::compile_idl_with_warnings(
        r"
        interface SelfRef;
        
        interface SelfRef : SelfRef {
            void method();
        };
        ",
    );
    assert_snapshot!(output);
}
