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

use std::error::Error;

use ic_hir::hir::{DefKind, Numeric};

mod common;

#[test]
fn test_mixed_int_float_multiplication() -> Result<(), Box<dyn Error>> {
    let idl = r#"
        const uint32 FOO = 3 * 1.5;
    "#;

    let (hir, _source_map, _diagnostics) = common::parse_and_resolve(idl);

    // Find the constant FOO
    let (_, foo_const) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "FOO")
        .expect("FOO constant not found");

    if let DefKind::Const(const_ty) = &foo_const.kind {
        assert_eq!(const_ty.value, Numeric::UInt32(4));
    } else {
        panic!("FOO is not a constant");
    }

    Ok(())
}

#[test]
fn test_mixed_int_float_division() -> Result<(), Box<dyn Error>> {
    let idl = r#"
        const uint32 BAR = 10 / 3.0;
    "#;

    let (hir, _source_map, _diagnostics) = common::parse_and_resolve(idl);

    let (_, bar_const) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BAR")
        .expect("BAR constant not found");

    if let DefKind::Const(const_ty) = &bar_const.kind {
        assert_eq!(const_ty.value, Numeric::UInt32(3));
    } else {
        panic!("BAR is not a constant");
    }

    Ok(())
}

#[test]
fn test_mixed_float_int_addition() -> Result<(), Box<dyn Error>> {
    let idl = r#"
        const int32 BAZ = 5.7 + 2;
    "#;

    let (hir, _source_map, _diagnostics) = common::parse_and_resolve(idl);

    let (_, baz_const) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BAZ")
        .expect("BAZ constant not found");

    if let DefKind::Const(const_ty) = &baz_const.kind {
        assert_eq!(const_ty.value, Numeric::Int32(7));
    } else {
        panic!("BAZ is not a constant");
    }

    Ok(())
}

#[test]
fn test_mixed_int_float_subtraction() -> Result<(), Box<dyn Error>> {
    let idl = r#"
        const uint16 QUX = 100 - 20.5;
    "#;

    let (hir, _source_map, _diagnostics) = common::parse_and_resolve(idl);

    let (_, qux_const) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "QUX")
        .expect("QUX constant not found");

    if let DefKind::Const(const_ty) = &qux_const.kind {
        assert_eq!(const_ty.value, Numeric::UInt16(79));
    } else {
        panic!("QUX is not a constant");
    }

    Ok(())
}

#[test]
fn test_float_to_integer_conversion() -> Result<(), Box<dyn Error>> {
    let idl = r#"
        const uint32 FLOAT_TO_UINT = 4.5;
        const int32 FLOAT_TO_INT = -3.7;
        const uint8 LARGE_FLOAT = 255.9;
    "#;

    let (hir, _source_map, _diagnostics) = common::parse_and_resolve(idl);

    // Check FLOAT_TO_UINT
    let (_, float_to_uint) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "FLOAT_TO_UINT")
        .expect("FLOAT_TO_UINT constant not found");

    if let DefKind::Const(const_ty) = &float_to_uint.kind {
        assert_eq!(const_ty.value, Numeric::UInt32(4));
    } else {
        panic!("FLOAT_TO_UINT is not a constant");
    }

    // Check FLOAT_TO_INT
    let (_, float_to_int) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "FLOAT_TO_INT")
        .expect("FLOAT_TO_INT constant not found");

    if let DefKind::Const(const_ty) = &float_to_int.kind {
        assert_eq!(const_ty.value, Numeric::Int32(-3));
    } else {
        panic!("FLOAT_TO_INT is not a constant");
    }

    // Check LARGE_FLOAT
    let (_, large_float) = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "LARGE_FLOAT")
        .expect("LARGE_FLOAT constant not found");

    if let DefKind::Const(const_ty) = &large_float.kind {
        assert_eq!(const_ty.value, Numeric::Octet(255));
    } else {
        panic!("LARGE_FLOAT is not a constant");
    }

    Ok(())
}
