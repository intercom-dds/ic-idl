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

use crate::union_types;

#[test]
fn union_int_variant() {
    let u = union_types::IntOrString::IntVal(42);
    assert_eq!(u.disc(), 1);
    assert_eq!(u, union_types::IntOrString::IntVal(42));
}

#[test]
fn union_string_variant() {
    let u = union_types::IntOrString::StrVal("hello".into());
    assert_eq!(u.disc(), 2);
    assert_eq!(u, union_types::IntOrString::StrVal("hello".into()));
}

#[test]
fn union_wrong_variant_raises() {
    let mut u = union_types::IntOrString::IntVal(42);
    assert!(!matches!(u, union_types::IntOrString::StrVal(_)));

    u = union_types::IntOrString::StrVal("test".into());
    assert!(!matches!(u, union_types::IntOrString::IntVal(_)));
}

#[test]
fn union_enum_discriminator() {
    let tv = union_types::TypedValue::new();
    assert_eq!(
        std::any::type_name_of_val(&tv.disc()),
        std::any::type_name::<union_types::ValueKind>()
    );
}

#[test]
fn union_enum_string_variant() {
    let tv = union_types::TypedValue::StringValue("test string".into());
    assert_eq!(tv.disc(), union_types::ValueKind::StringKind);
    assert_eq!(
        tv,
        union_types::TypedValue::StringValue("test string".into())
    );
}

#[test]
fn union_bool_discriminator() {
    let mut bs = union_types::BoolSwitch::TrueVal(100);
    assert!(bs.disc());
    assert_eq!(bs, union_types::BoolSwitch::TrueVal(100));

    bs = union_types::BoolSwitch::FalseVal("false branch".into());
    assert!(!bs.disc());
    assert_eq!(bs, union_types::BoolSwitch::FalseVal("false branch".into()));
}

#[test]
fn union_multi_case() {
    let mut mc = union_types::MultiCase::SmallVal1(5);
    assert_eq!(mc.disc(), 1);

    mc = union_types::MultiCase::SmallVal2(5);
    assert_eq!(mc.disc(), 2);
    assert_eq!(mc, union_types::MultiCase::SmallVal2(5));
    mc = union_types::MultiCase::SmallVal2(10);
    assert_eq!(mc.disc(), 2);
    assert_eq!(mc, union_types::MultiCase::SmallVal2(10));

    mc = union_types::MultiCase::SmallVal3(10);
    assert_eq!(mc.disc(), 3);
    assert_eq!(mc, union_types::MultiCase::SmallVal3(10));

    mc = union_types::MultiCase::TextVal10("test".into());
    assert_eq!(mc.disc(), 10);
    assert_eq!(mc, union_types::MultiCase::TextVal10("test".into()));
}

#[test]
fn union_multi_case_roundtrip() {
    for value in [
        union_types::MultiCase::SmallVal1(1),
        union_types::MultiCase::SmallVal2(2),
        union_types::MultiCase::SmallVal3(3),
    ] {
        let json = intercom_cts::json::to_string(&value, false).unwrap();
        let decoded = intercom_cts::json::from_str(&json).unwrap();

        assert_eq!(value, decoded);
    }
}

#[test]
fn union_default_method() {
    #[allow(unused)]
    let mut u = union_types::IntOrString::StrVal("hello".into());
    u = union_types::IntOrString::DefaultVal(true);
    assert_ne!(u.disc(), 1);
    assert_ne!(u.disc(), 2);
    assert_eq!(u, union_types::IntOrString::DefaultVal(true));
}

#[test]
fn union_discriminator_property() {
    let mut u = union_types::IntOrString::IntVal(42);
    assert_eq!(u.disc(), 1);

    u = union_types::IntOrString::StrVal("test".into());
    assert_eq!(u.disc(), 2);

    let tv = union_types::TypedValue::IntValue(100);
    assert_eq!(tv.disc(), union_types::ValueKind::IntKind);
}

#[test]
fn union_equality() {
    let u1 = union_types::IntOrString::IntVal(42);
    let u2 = union_types::IntOrString::IntVal(42);
    let u3 = union_types::IntOrString::IntVal(99);

    assert_eq!(u1, u2);
    assert!(!(u1 == u3));
    assert!(!(u1 != u2));
    assert_ne!(u1, u3);
}

#[test]
fn union_default_constructor_uses_default_discriminator_case() {
    assert_eq!(
        union_types::DefaultDiscriminatorCase::new(),
        union_types::DefaultDiscriminatorCase::Value0(0)
    );
}

#[test]
fn union_default_constructor_with_default_case() {
    let u = union_types::IntOrString::new();
    assert_eq!(u.disc(), 0);
    assert_eq!(u, union_types::IntOrString::DefaultVal(false));
}

#[test]
fn union_default_constructor_without_default_case() {
    let mc = union_types::MultiCase::new();
    assert_eq!(mc.disc(), 0);
    assert_eq!(mc, union_types::MultiCase::Flag(false));
}

#[test]
fn union_default_constructor_enum_discriminator() {
    let tv = union_types::TypedValue::new();
    assert_eq!(tv.disc(), union_types::ValueKind::IntKind);
    assert_eq!(tv, union_types::TypedValue::IntValue(0));
}

#[test]
fn union_default_variant_sets_discriminator() {
    let mut u = union_types::IntOrString::IntVal(42);
    assert_eq!(u.disc(), 1);

    u = union_types::IntOrString::DefaultVal(true);
    assert_eq!(u.disc(), 0);
    assert_eq!(u, union_types::IntOrString::DefaultVal(true));
}

#[test]
fn union_swap() {
    let mut u1 = union_types::IntOrString::IntVal(42);
    let mut u2 = union_types::IntOrString::StrVal("hello".into());

    std::mem::swap(&mut u1, &mut u2);

    assert_eq!(u1.disc(), 2);
    assert_eq!(u1, union_types::IntOrString::StrVal("hello".into()));
    assert_eq!(u2.disc(), 1);
    assert_eq!(u2, union_types::IntOrString::IntVal(42));
}

#[test]
fn union_swap_same_discriminator() {
    let mut tv1 = union_types::TypedValue::IntValue(100);
    let mut tv2 = union_types::TypedValue::IntValue(200);

    std::mem::swap(&mut tv1, &mut tv2);

    assert_eq!(tv1.disc(), union_types::ValueKind::IntKind);
    assert_eq!(tv1, union_types::TypedValue::IntValue(200));
    assert_eq!(tv2.disc(), union_types::ValueKind::IntKind);
    assert_eq!(tv2, union_types::TypedValue::IntValue(100));
}
