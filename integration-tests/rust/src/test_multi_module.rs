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

use crate::{constants_only, enums_only, module_a, module_b};

#[test]
fn module_a_exists() {
    let s1 = module_a::StructA1 { value: 42 };
    assert_eq!(s1.value, 42);
}

#[test]
fn module_b_exists() {
    let s1 = module_b::StructB1 {
        name: "test".into(),
    };
    assert_eq!(s1.name, "test");
}

#[test]
fn module_a_first_opening() {
    let s1 = module_a::StructA1 { value: 10 };
    assert_eq!(s1.value, 10);
    assert_eq!(module_a::CONST_A1, 100);
    assert_eq!(module_a::EnumA::X as i32, 0);
    assert_eq!(module_a::EnumA::Y as i32, 1);
}

#[test]
fn module_a_second_opening() {
    let a1 = module_a::StructA1 { value: 5 };
    let s2 = module_a::StructA2 {
        data: 3.14,
        ref_to_a1: a1,
    };
    assert_eq!(s2.data, 3.14);
    assert_eq!(s2.ref_to_a1.value, 5);
    assert_eq!(module_a::CONST_A2, 101);
    assert_eq!(module_a::EnumA2::P as i32, 0);
    assert_eq!(module_a::EnumA2::Q as i32, 1);
    assert_eq!(module_a::EnumA2::R as i32, 2);
}

#[test]
fn module_a_third_opening() {
    let a1 = module_a::StructA1 { value: 1 };
    let a2 = module_a::StructA2 {
        data: 2.0,
        ref_to_a1: a1,
    };
    let s3 = module_a::StructA3 { flag: true, a1, a2 };
    assert!(s3.flag);
    assert_eq!(s3.a1.value, 1);
    assert_eq!(s3.a2.data, 2.0);
    assert_eq!(module_a::CONST_A3, 102);
}

#[test]
fn module_b_both_openings() {
    let b1 = module_b::StructB1 {
        name: "first".into(),
    };
    assert_eq!(b1.name, "first");
    assert_eq!(module_b::CONST_B1, 200);

    let b2 = module_b::StructB2 {
        id: 42,
        ref_to_b1: b1,
    };
    assert_eq!(b2.id, 42);
    assert_eq!(b2.ref_to_b1.name, "first");
    assert_eq!(module_b::CONST_B2, 201);
}

#[test]
fn reopened_module_types_can_reference_earlier() {
    let a1 = module_a::StructA1 { value: 100 };
    let a2 = module_a::StructA2 {
        data: 99.5,
        ref_to_a1: a1,
    };
    assert_eq!(a2.ref_to_a1.value, 100);
    assert_eq!(a2.data, 99.5);
}

#[test]
fn reopened_module_chain() {
    let a1 = module_a::StructA1 { value: 10 };
    let a2 = module_a::StructA2 {
        data: 20.0,
        ref_to_a1: a1,
    };
    let a3 = module_a::StructA3 {
        flag: false,
        a1,
        a2,
    };
    assert_eq!(a3.a1.value, 10);
    assert_eq!(a3.a2.data, 20.0);
    assert_eq!(a3.a2.ref_to_a1.value, 10);
    assert_eq!(a3.flag, false);
}

#[test]
fn constants_only_module() {
    assert_eq!(constants_only::C1, 1);
    assert_eq!(constants_only::C2, 2);
    assert_eq!(constants_only::C3, 3);
}

#[test]
fn enums_only_module() {
    assert_eq!(enums_only::Color::Red as i32, 0);
    assert_eq!(enums_only::Color::Green as i32, 1);
    assert_eq!(enums_only::Color::Blue as i32, 2);
    assert_eq!(enums_only::Size::Small as i32, 0);
    assert_eq!(enums_only::Size::Medium as i32, 1);
    assert_eq!(enums_only::Size::Large as i32, 2);
}

#[test]
fn cross_module_references() {
    let a1 = module_a::StructA1 { value: 50 };
    let b1 = module_b::StructB1 {
        name: "cross".into(),
    };
    assert_eq!(a1.value, 50);
    assert_eq!(b1.name, "cross");
}
