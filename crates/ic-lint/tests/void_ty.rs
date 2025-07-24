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
fn valid_void_return_type() {
    let source = r"
interface TestInterface {
    void doSomething();
    void doAnotherThing(in long value);
    void processData(in string data, out long result);
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty());
}

#[test]
fn void_as_parameter_type() {
    let source = r"
interface TestInterface {
    void processVoid(in void param);
    long calculate(in void input, out void output);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_struct_member() {
    let source = r"
struct InvalidStruct {
    void invalidField;
    long validField;
    void anotherInvalid;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_exception_member() {
    let source = r"
exception InvalidException {
    void errorField;
    string message;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_union_variant() {
    let source = r"
union InvalidUnion switch (long) {
    case 1: void voidVariant;
    case 2: string stringVariant;
    case 3: void anotherVoid;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_typedef() {
    let source = r"
typedef void VoidAlias;
typedef sequence<void> VoidSequence;
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_array() {
    let source = r"
struct ArrayStruct {
    void invalidArray[10];
};

typedef void VoidArray[5];
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_sequence() {
    let source = r"
struct SequenceStruct {
    sequence<void> voidSeq;
    sequence<sequence<void>> nestedVoidSeq;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_map() {
    let source = r"
struct MapStruct {
    map<string, void> stringToVoid;
    map<void, long> voidToLong;
    map<void, void> voidToVoid;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_const() {
    let source = r"
const void INVALID_CONST = 0;
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_annotation_member() {
    let source = r"
@annotation TestAnnotation {
    void invalidMember;
    string validMember;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn mixed_valid_invalid_void() {
    let source = r"
interface MixedInterface {
    // Valid use of void
    void validVoidReturn();
    
    // Invalid uses of void
    void invalidMethod(in void param);
    
    // Valid non-void method
    long calculate(in long x, in long y);
};

struct MixedStruct {
    void invalidField;
    long validField;
};

// Valid use in another interface
interface AnotherInterface {
    void shutdown();
    oneway void notify(in string message);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_in_nested_types() {
    let source = r"
struct NestedInvalid {
    sequence<sequence<void>> deeplyNested;
    map<string, sequence<void>> mapOfVoidSeq;
};

interface NestedInterface {
    // Valid void return
    void process();

    // Invalid void in nested parameter type
    void handleData(in sequence<void> data);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn void_with_oneway() {
    let source = r"
interface OnewayInterface {
    // Valid: oneway operations must return void
    oneway void notify(in string message);
    oneway void shutdown();
    
    // Still invalid: void as parameter
    oneway void invalid(in void param);
};
";

    assert_snapshot!(test_lint_hir(source));
}
