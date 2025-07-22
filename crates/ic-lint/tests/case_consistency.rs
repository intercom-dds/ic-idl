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
use common::test_lint_hir;

#[test]
fn consistent_capitalization() {
    let source = r"
module MyModule {
    struct MyStruct {
        long field1;
    };
    
    struct AnotherStruct {
        MyStruct myField;  // Consistent capitalization
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn inconsistent_struct_reference() {
    let source = r"
module MyModule {
    struct MyStruct {
        long field1;
    };
    
    struct AnotherStruct {
        mystruct myField;  // Inconsistent capitalization
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn inconsistent_interface_reference() {
    let source = r"
module MyModule {
    interface MyInterface {
        void doSomething();
    };
    
    interface AnotherInterface : myinterface {  // Inconsistent capitalization
        void doMore();
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn inconsistent_enum_reference() {
    let source = r"
module MyModule {
    enum Color {
        RED,
        GREEN,
        BLUE
    };
    
    struct ColorHolder {
        color myColor;  // Inconsistent capitalization
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_inconsistent_references() {
    let source = r"
module MyModule {
    struct Point {
        long x;
        long y;
    };
    
    struct Line {
        point start;    // Inconsistent
        POINT end;      // Also inconsistent  
    };
    
    struct Polygon {
        sequence<Point> vertices;  // Consistent
        point center;             // Inconsistent
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn case_insensitive_names() {
    let source = r"
module MyModule {
    struct FOO {
        long value;
    };
    
    struct Container {
        foo field1;    // Should warn - inconsistent with FOO
        FOO field2;    // Should be ok - matches original
        Foo field3;    // Should warn - inconsistent with FOO
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn union_discriminator_case() {
    let source = r"
module MyModule {
    enum DiscriminatorType {
        TYPE_A,
        TYPE_B
    };
    
    union MyUnion switch (discriminatortype) {  // Inconsistent capitalization
        case TYPE_A: long a;
        case TYPE_B: string b;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}
