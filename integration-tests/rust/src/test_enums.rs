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

use crate::enum_types;

#[test]
fn enum_members_exist() {
    let _red = enum_types::Color::Red;
    let _green = enum_types::Color::Green;
    let _blue = enum_types::Color::Blue;
}

#[test]
fn enum_is_enum_type() {
    assert_eq!(
        intercom_cts::type_info::<enum_types::Color>().kind,
        intercom_cts::TypeKind::Enum
    );
    assert_eq!(
        intercom_cts::type_info::<enum_types::Status>().kind,
        intercom_cts::TypeKind::Enum
    );
    assert_eq!(
        intercom_cts::type_info::<enum_types::GappedEnum>().kind,
        intercom_cts::TypeKind::Enum
    );
    assert_eq!(
        intercom_cts::type_info::<enum_types::NegativeEnum>().kind,
        intercom_cts::TypeKind::Enum
    );
    assert_eq!(
        intercom_cts::type_info::<enum_types::MixedEnum>().kind,
        intercom_cts::TypeKind::Enum
    );
}

#[test]
fn enum_auto_values() {
    assert_eq!(enum_types::Color::Red as i32, 0);
    assert_eq!(enum_types::Color::Green as i32, 1);
    assert_eq!(enum_types::Color::Blue as i32, 2);
}

#[test]
fn enum_explicit_values() {
    assert_eq!(enum_types::Status::Ok as i32, 0);
    assert_eq!(enum_types::Status::Warning as i32, 100);
    assert_eq!(enum_types::Status::Error as i32, 200);
}

#[test]
fn enum_comparison() {
    assert_eq!(enum_types::Color::Red, enum_types::Color::Red);
    assert!(!(enum_types::Color::Red == enum_types::Color::Blue));
    assert!(enum_types::Color::Red != enum_types::Color::Blue);
    assert!(!(enum_types::Color::Red != enum_types::Color::Red));

    assert_eq!(enum_types::Status::Warning, enum_types::Status::Warning);
    assert!(enum_types::Status::Ok != enum_types::Status::Error);
}

#[test]
fn enum_by_value() {
    let c0 = unsafe { std::mem::transmute::<i32, enum_types::Color>(0) };
    let c1 = unsafe { std::mem::transmute::<i32, enum_types::Color>(1) };
    let c2 = unsafe { std::mem::transmute::<i32, enum_types::Color>(2) };

    assert_eq!(c0, enum_types::Color::Red);
    assert_eq!(c1, enum_types::Color::Green);
    assert_eq!(c2, enum_types::Color::Blue);

    let s100 = unsafe { std::mem::transmute::<i32, enum_types::Status>(100) };
    assert_eq!(s100, enum_types::Status::Warning);
}

#[test]
fn enum_gapped_values() {
    assert_eq!(enum_types::GappedEnum::First as i32, 0);
    assert_eq!(enum_types::GappedEnum::Second as i32, 5);
    assert_eq!(enum_types::GappedEnum::Third as i32, 10);
    assert_eq!(enum_types::GappedEnum::Fourth as i32, 100);

    let g5 = unsafe { std::mem::transmute::<i32, enum_types::GappedEnum>(5) };
    assert_eq!(g5, enum_types::GappedEnum::Second);

    let g100 = unsafe { std::mem::transmute::<i32, enum_types::GappedEnum>(100) };
    assert_eq!(g100, enum_types::GappedEnum::Fourth);
}

#[test]
fn enum_negative_values() {
    assert_eq!(enum_types::NegativeEnum::NegTwo as i32, -2);
    assert_eq!(enum_types::NegativeEnum::NegOne as i32, -1);
    assert_eq!(enum_types::NegativeEnum::Zero as i32, 0);
    assert_eq!(enum_types::NegativeEnum::PosOne as i32, 1);

    let neg = unsafe { std::mem::transmute::<i32, enum_types::NegativeEnum>(-2) };
    assert_eq!(neg, enum_types::NegativeEnum::NegTwo);

    let zero = unsafe { std::mem::transmute::<i32, enum_types::NegativeEnum>(0) };
    assert_eq!(zero, enum_types::NegativeEnum::Zero);
}

#[test]
fn enum_const_from_enum_value() {
    assert_eq!(enum_types::ENUM_CONST as i32, 100);
    assert_eq!(enum_types::ENUM_CONST, enum_types::Status::Warning);
}

#[test]
fn enum_mixed_explicit_auto() {
    assert_eq!(enum_types::MixedEnum::AutoFirst as i32, 0);
    assert_eq!(enum_types::MixedEnum::ExplicitTen as i32, 10);
    assert_eq!(enum_types::MixedEnum::AutoEleven as i32, 11);
    assert_eq!(enum_types::MixedEnum::ExplicitHundred as i32, 100);
    assert_eq!(enum_types::MixedEnum::AutoHundredOne as i32, 101);

    let m11 = unsafe { std::mem::transmute::<i32, enum_types::MixedEnum>(11) };
    assert_eq!(m11, enum_types::MixedEnum::AutoEleven);

    let m101 = unsafe { std::mem::transmute::<i32, enum_types::MixedEnum>(101) };
    assert_eq!(m101, enum_types::MixedEnum::AutoHundredOne);
}
