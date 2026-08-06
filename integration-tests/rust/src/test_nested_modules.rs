// Copyright 2026 KONGSBERG
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

use crate::nested_module_types;

#[test]
fn top_level_types_exist() {
    let s = nested_module_types::TopLevelStruct { value: 42 };
    assert_eq!(s.value, 42);
    assert_eq!(nested_module_types::TopLevelEnum::First as i32, 0);
    assert_eq!(nested_module_types::TopLevelEnum::Second as i32, 1);
}

#[test]
fn nested_module_level1_exists() {
    let top = nested_module_types::TopLevelStruct { value: 10 };
    let s = nested_module_types::level1::Level1Struct {
        data: 20,
        parent_ref: top,
    };
    assert_eq!(s.data, 20);
    assert_eq!(s.parent_ref.value, 10);
    assert_eq!(nested_module_types::level1::Level1Enum::A as i32, 0);
    assert_eq!(nested_module_types::level1::Level1Enum::B as i32, 1);
    assert_eq!(nested_module_types::level1::Level1Enum::C as i32, 2);
}

#[test]
fn nested_module_level2_exists() {
    let top = nested_module_types::TopLevelStruct { value: 1 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 2,
        parent_ref: top,
    };
    let s = nested_module_types::level1::level2::Level2Struct {
        name: "test".into(),
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(s.name, "test");
    assert_eq!(s.level1_ref.data, 2);
    assert_eq!(s.top_ref.value, 1);
}

#[test]
fn nested_module_level3_exists() {
    let top = nested_module_types::TopLevelStruct { value: 1 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 2,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "level2".into(),
        level1_ref: l1,
        top_ref: top,
    };
    let s = nested_module_types::level1::level2::level3::Level3Struct {
        id: 99,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(s.id, 99);
    assert_eq!(s.level2_ref.name, "level2");
    assert_eq!(s.level1_ref.data, 2);
    assert_eq!(s.top_ref.value, 1);
    assert_eq!(nested_module_types::level1::level2::level3::DEEP_CONST, 42);
}

#[test]
fn sibling_module_exists() {
    let s = nested_module_types::sibling::SiblingStruct { id: 123 };
    assert_eq!(s.id, 123);
}

#[test]
fn top_level_struct_instantiation() {
    let s = nested_module_types::TopLevelStruct { value: 100 };
    assert_eq!(s.value, 100);
}

#[test]
fn level1_struct_with_parent_ref() {
    let parent = nested_module_types::TopLevelStruct { value: 50 };
    let s = nested_module_types::level1::Level1Struct {
        data: 75,
        parent_ref: parent,
    };
    assert_eq!(s.data, 75);
    assert_eq!(s.parent_ref.value, 50);
}

#[test]
fn level2_struct_with_refs() {
    let top = nested_module_types::TopLevelStruct { value: 10 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 20,
        parent_ref: top,
    };
    let s = nested_module_types::level1::level2::Level2Struct {
        name: "hello".into(),
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(s.name, "hello");
    assert_eq!(s.level1_ref.data, 20);
    assert_eq!(s.level1_ref.parent_ref.value, 10);
    assert_eq!(s.top_ref.value, 10);
}

#[test]
fn level3_struct_with_all_refs() {
    let top = nested_module_types::TopLevelStruct { value: 1 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 2,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "level2".into(),
        level1_ref: l1,
        top_ref: top,
    };
    let l3 = nested_module_types::level1::level2::level3::Level3Struct {
        id: 3,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(l3.id, 3);
    assert_eq!(l3.level2_ref.name, "level2");
    assert_eq!(l3.level1_ref.data, 2);
    assert_eq!(l3.top_ref.value, 1);
}

#[test]
fn deep_constant() {
    assert_eq!(nested_module_types::level1::level2::level3::DEEP_CONST, 42);
}

#[test]
fn sibling_cross_ref_struct() {
    let top = nested_module_types::TopLevelStruct { value: 100 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 200,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "cross".into(),
        level1_ref: l1,
        top_ref: top,
    };
    let l3 = nested_module_types::level1::level2::level3::Level3Struct {
        id: 300,
        level2_ref: l2.clone(),
        level1_ref: l1,
        top_ref: top,
    };
    let cr = nested_module_types::sibling::CrossRef {
        from_level1: l1,
        from_level2: l2,
        from_level3: l3,
    };
    assert_eq!(cr.from_level1.data, 200);
    assert_eq!(cr.from_level2.name, "cross");
    assert_eq!(cr.from_level3.id, 300);
}

#[test]
fn top_using_nested_struct() {
    let top = nested_module_types::TopLevelStruct { value: 1 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 2,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "test".into(),
        level1_ref: l1,
        top_ref: top,
    };
    let l3 = nested_module_types::level1::level2::level3::Level3Struct {
        id: 3,
        level2_ref: l2.clone(),
        level1_ref: l1,
        top_ref: top,
    };
    let sib = nested_module_types::sibling::SiblingStruct { id: 4 };
    let tun = nested_module_types::TopUsingNested { l1, l2, l3, sib };
    assert_eq!(tun.l1.data, 2);
    assert_eq!(tun.l2.name, "test");
    assert_eq!(tun.l3.id, 3);
    assert_eq!(tun.sib.id, 4);
}

#[test]
fn level1_enum() {
    assert_eq!(nested_module_types::level1::Level1Enum::A as i32, 0);
    assert_eq!(nested_module_types::level1::Level1Enum::B as i32, 1);
    assert_eq!(nested_module_types::level1::Level1Enum::C as i32, 2);
}

#[test]
fn namespace_hierarchy_level1() {
    let top = nested_module_types::TopLevelStruct { value: 10 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 20,
        parent_ref: top,
    };
    assert_eq!(l1.data, 20);
}

#[test]
fn namespace_hierarchy_level2() {
    let top = nested_module_types::TopLevelStruct { value: 10 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 20,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "nested".into(),
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(l2.name, "nested");
}

#[test]
fn namespace_hierarchy_level3() {
    let top = nested_module_types::TopLevelStruct { value: 10 };
    let l1 = nested_module_types::level1::Level1Struct {
        data: 20,
        parent_ref: top,
    };
    let l2 = nested_module_types::level1::level2::Level2Struct {
        name: "nested".into(),
        level1_ref: l1,
        top_ref: top,
    };
    let l3 = nested_module_types::level1::level2::level3::Level3Struct {
        id: 30,
        level2_ref: l2,
        level1_ref: l1,
        top_ref: top,
    };
    assert_eq!(l3.id, 30);
}
