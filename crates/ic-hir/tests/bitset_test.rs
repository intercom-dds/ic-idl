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

use ic_hir::hir::{DefKind, PrimitiveTy, TyKind};

mod common;

#[test]
fn test_bitset_basic() {
    let input = r"
        bitset Status {
            bitfield<1> active;
            bitfield<3> state;
            bitfield<4, uint8> code;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the bitset
    let status = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Status")
        .expect("Status bitset not found");

    match &status.1.kind {
        DefKind::Bitset(bitset) => {
            assert_eq!(bitset.fields.len(), 3);

            // Check first field - should get bool type
            assert_eq!(bitset.fields[0].ident.name, "active");
            assert_eq!(bitset.fields[0].size, 1);
            match &bitset.fields[0].ty.kind {
                TyKind::Primitive(PrimitiveTy::Bool) => {}
                _ => panic!("Expected bool type for 1-bit field"),
            }

            // Check second field - should get uint8
            assert_eq!(bitset.fields[1].ident.name, "state");
            assert_eq!(bitset.fields[1].size, 3);
            match &bitset.fields[1].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt8) => {}
                _ => panic!("Expected uint8 type for 3-bit field"),
            }

            // Check third field - explicit uint8
            assert_eq!(bitset.fields[2].ident.name, "code");
            assert_eq!(bitset.fields[2].size, 4);
            match &bitset.fields[2].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt8) => {}
                _ => panic!("Expected uint8 type for explicitly typed field"),
            }
        }
        _ => panic!("Expected bitset definition"),
    }
}

#[test]
fn test_bitset_inheritance() {
    let input = r"
        bitset Base {
            bitfield<8> version;
        };
        
        bitset Extended : Base {
            bitfield<16> extra;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the extended bitset
    let extended = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Extended")
        .expect("Extended bitset not found");

    match &extended.1.kind {
        DefKind::Bitset(bitset) => {
            // Check parent is resolved
            assert!(bitset.parent.is_some(), "Parent should be resolved");

            // Check field
            assert_eq!(bitset.fields.len(), 1);
            assert_eq!(bitset.fields[0].ident.name, "extra");
            assert_eq!(bitset.fields[0].size, 16);
            match &bitset.fields[0].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt16) => {}
                _ => panic!("Expected uint16 type for 16-bit field"),
            }
        }
        _ => panic!("Expected bitset definition"),
    }
}

#[test]
fn test_bitset_size_expressions() {
    let input = r"
        const uint32 BYTE_SIZE = 8;
        
        bitset Dynamic {
            bitfield<BYTE_SIZE> one_byte;
            bitfield<BYTE_SIZE * 2> two_bytes;
            bitfield<BYTE_SIZE + 4> twelve_bits;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    let dynamic = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Dynamic")
        .expect("Dynamic bitset not found");

    match &dynamic.1.kind {
        DefKind::Bitset(bitset) => {
            assert_eq!(bitset.fields.len(), 3);

            // Check evaluated sizes
            assert_eq!(bitset.fields[0].size, 8);
            assert_eq!(bitset.fields[1].size, 16);
            assert_eq!(bitset.fields[2].size, 12);

            // Check assigned types
            match &bitset.fields[0].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt8) => {}
                _ => panic!("Expected uint8 for 8-bit field"),
            }
            match &bitset.fields[1].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt16) => {}
                _ => panic!("Expected uint16 for 16-bit field"),
            }
            match &bitset.fields[2].ty.kind {
                TyKind::Primitive(PrimitiveTy::UInt16) => {}
                _ => panic!("Expected uint16 for 12-bit field"),
            }
        }
        _ => panic!("Expected bitset definition"),
    }
}

#[test]
fn test_bitset_automatic_types() {
    let input = r"
        bitset AutoTypes {
            bitfield<1> flag;          // bool
            bitfield<8> byte;          // uint8
            bitfield<16> word;         // uint16
            bitfield<32> dword;        // uint32
            bitfield<64> qword;        // uint64
            bitfield<17> odd_size;     // uint32 (next size up)
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    let auto_types = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "AutoTypes")
        .expect("AutoTypes bitset not found");

    match &auto_types.1.kind {
        DefKind::Bitset(bitset) => {
            let expected_types = [
                (1, "boolean"), // PrimitiveTy::Bool -> "boolean"
                (8, "uint8"),
                (16, "uint16"),
                (32, "uint32"),
                (64, "uint64"),
                (17, "uint32"),
            ];

            for (i, (size, expected)) in expected_types.iter().enumerate() {
                assert_eq!(bitset.fields[i].size, *size);
                let actual = match &bitset.fields[i].ty.kind {
                    TyKind::Primitive(p) => p.name(),
                    _ => panic!("Expected primitive type"),
                };
                assert_eq!(actual, *expected, "Field {i} type mismatch");
            }
        }
        _ => panic!("Expected bitset definition"),
    }
}

#[test]
fn test_bitfield_type_determination() {
    let idl = r"
bitset TestBitset {
    bitfield<1> flag1;         // Should be bool
    bitfield<1, boolean> flag2; // Explicitly bool
    bitfield<3> small_val;     // Should be uint8
    bitfield<3, int8> small_int; // Explicitly int8
    bitfield<9> medium_val;    // Should be uint16
    bitfield<17> large_val;    // Should be uint32
    bitfield<33> xlarge_val;   // Should be uint64
};
";

    let (hir, _, _) = common::parse_and_resolve(idl);

    assert!(
        hir.errors.is_empty(),
        "Expected no errors, but got: {:?}",
        hir.errors
    );

    // Find the TestBitset definition
    let bitset_def = hir
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestBitset")
        .expect("TestBitset not found")
        .1;

    if let DefKind::Bitset(bitset) = &bitset_def.kind {
        assert_eq!(bitset.fields.len(), 7, "Expected 7 fields");

        // Check field types
        assert_matches!(
            &bitset.fields[0].ty.kind,
            TyKind::Primitive(PrimitiveTy::Bool),
            "flag1 should be bool"
        );
        assert_matches!(
            &bitset.fields[1].ty.kind,
            TyKind::Primitive(PrimitiveTy::Bool),
            "flag2 should be bool"
        );
        assert_matches!(
            &bitset.fields[2].ty.kind,
            TyKind::Primitive(PrimitiveTy::UInt8),
            "small_val should be uint8"
        );
        assert_matches!(
            &bitset.fields[3].ty.kind,
            TyKind::Primitive(PrimitiveTy::Int8),
            "small_int should be int8"
        );
        assert_matches!(
            &bitset.fields[4].ty.kind,
            TyKind::Primitive(PrimitiveTy::UInt16),
            "medium_val should be uint16"
        );
        assert_matches!(
            &bitset.fields[5].ty.kind,
            TyKind::Primitive(PrimitiveTy::UInt32),
            "large_val should be uint32"
        );
        assert_matches!(
            &bitset.fields[6].ty.kind,
            TyKind::Primitive(PrimitiveTy::UInt64),
            "xlarge_val should be uint64"
        );
    } else {
        panic!("TestBitset is not a bitset");
    }
}

// Helper macro for checking field types
#[macro_export]
macro_rules! assert_matches {
    ($expr:expr, $pattern:pat, $msg:expr) => {
        match $expr {
            $pattern => {}
            _ => panic!("{}", $msg),
        }
    };
}
