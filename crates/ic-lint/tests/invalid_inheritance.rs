// Copyright 2025 KONGSBERG
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

mod common;

#[test]
fn struct_inheriting_from_enum() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        struct MyStruct : Color {
            long field;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn struct_inheriting_from_union() {
    let idl = r"
        union MyUnion switch(long) {
            case 1: string x;
            case 2: long y;
        };
        struct BadStruct : MyUnion {
            long field;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn struct_inheriting_from_interface() {
    let idl = r"
        interface IBase {
            void method();
        };
        struct BadStruct : IBase {
            long field;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn interface_inheriting_from_struct() {
    let idl = r"
        struct BaseStruct {
            long x;
        };
        interface BadInterface : BaseStruct {
            void method();
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn interface_inheriting_from_enum() {
    let idl = r"
        enum Status { ACTIVE, INACTIVE };
        interface BadInterface : Status {
            void method();
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_inheriting_from_struct() {
    let idl = r"
        struct BaseStruct {
            long x;
        };
        valuetype BadValue : BaseStruct {
            public long y;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_inheriting_from_enum() {
    let idl = r"
        enum Priority { LOW, MEDIUM, HIGH };
        valuetype BadValue : Priority {
            public long x;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_supporting_non_interface() {
    let idl = r"
        struct SomeStruct {
            long x;
        };
        valuetype BadValue supports SomeStruct {
            public long y;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_supporting_enum() {
    let idl = r"
        enum State { ON, OFF };
        valuetype BadValue supports State {
            public long x;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn bitset_inheriting_from_struct() {
    let idl = r"
        struct BaseStruct {
            long x;
        };
        bitset BadBits : BaseStruct {
            bitfield<1> BIT0;
            bitfield<1> BIT1;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn bitset_inheriting_from_enum() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        bitset BadBits : Color {
            bitfield<1> BIT0;
            bitfield<1> BIT1;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valid_inheritance() {
    let idl = r"
        // Valid struct inheritance
        struct BaseStruct {
            long x;
        };
        struct DerivedStruct : BaseStruct {
            long y;
        };

        // Valid interface inheritance
        interface IBase {
            void method1();
        };
        interface IDerived : IBase {
            void method2();
        };

        // Valid valuetype inheritance and supports
        valuetype BaseValue {
            public long x;
        };
        valuetype DerivedValue : BaseValue {
            public long y;
        };
        valuetype ValueWithSupports supports IBase {
            public long z;
        };

        // Valid bitset inheritance
        bitset BaseBits {
            bitfield<1> BIT0;
            bitfield<1> BIT1;
        };
        bitset DerivedBits : BaseBits {
            bitfield<1> BIT2;
            bitfield<1> BIT3;
        };
    ";
    let report = common::lint_hir(idl);
    assert!(
        report.errors.is_empty(),
        "Expected no errors, but got: {:?}",
        report.errors
    );
}

#[test]
fn multiple_invalid_inheritance() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        union Status switch(long) { case 1: string s; };

        // Multiple inheritance violations in one file
        struct BadStruct1 : Color { long x; };
        struct BadStruct2 : Status { long y; };

        interface BadInterface : Color { void method(); };

        valuetype BadValue : Color { public long z; };
        valuetype BadValue2 supports Status { public long w; };

        bitset BadBits : Status { bitfield<1> BIT0; };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn struct_inheriting_from_typedef_to_enum() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        typedef Color ColorRef;
        struct BadStruct : ColorRef {
            long field;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn interface_inheriting_from_typedef_to_struct() {
    let idl = r"
        struct BaseStruct {
            long x;
        };
        typedef BaseStruct StructRef;
        interface BadInterface : StructRef {
            void method();
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_inheriting_from_typedef_to_struct() {
    let idl = r"
        struct BaseStruct {
            long x;
        };
        typedef BaseStruct StructRef;
        valuetype BadValue : StructRef {
            public long y;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn valuetype_supporting_typedef_to_struct() {
    let idl = r"
        struct SomeStruct {
            long x;
        };
        typedef SomeStruct StructRef;
        valuetype BadValue supports StructRef {
            public long y;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn bitset_inheriting_from_typedef_to_enum() {
    let idl = r"
        enum Color { RED, GREEN, BLUE };
        typedef Color ColorRef;
        bitset BadBits : ColorRef {
            bitfield<1> BIT0;
            bitfield<1> BIT1;
        };
    ";
    insta::assert_snapshot!(common::test_lint_hir(idl));
}

#[test]
fn inheritance_through_typedef() {
    let idl = r"
        struct BaseStruct {
            long x;
        };

        typedef BaseStruct StructRef;
        struct DerivedStruct : StructRef {
            long y;
        };

        interface IBase {
            void method1();
        };

        typedef IBase IBaseRef;
        interface IDerived : IBaseRef {
            void method2();
        };

        valuetype BaseValue {
            public long x;
        };

        typedef BaseValue ValueRef;
        valuetype DerivedValue : ValueRef {
            public long y;
        };

        valuetype ValueWithSupports supports IBaseRef {
            public long z;
        };

        bitset BaseBits {
            bitfield<1> BIT0;
            bitfield<1> BIT1;
        };

        typedef BaseBits BitsRef;
        typedef BitsRef BitsRefRef;
        bitset DerivedBits : BitsRefRef {
            bitfield<1> BIT2;
            bitfield<1> BIT3;
        };
    ";

    let report = common::lint_hir(idl);
    assert!(
        report.errors.is_empty(),
        "Expected no errors, but got: {:?}",
        report.errors
    );
}
