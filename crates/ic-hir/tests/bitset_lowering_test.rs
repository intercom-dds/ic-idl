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

mod common;

#[test]
fn test_bitset_lowering_simple() {
    let idl = r#"
        bitset SimpleFlags {
            bitfield<1> flag1;
            bitfield<1> flag2;
            bitfield<6> reserved;
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Verify the bitset was processed correctly
    let bitset = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "SimpleFlags")
        .expect("Bitset not found");

    if let ic_hir::hir::DefKind::Bitset(bs) = &bitset.1.kind {
        assert_eq!(bs.fields.len(), 3);
        assert_eq!(bs.fields[0].ident.name, "flag1");
        assert_eq!(bs.fields[0].size, 1);
        assert_eq!(bs.fields[1].ident.name, "flag2");
        assert_eq!(bs.fields[1].size, 1);
        assert_eq!(bs.fields[2].ident.name, "reserved");
        assert_eq!(bs.fields[2].size, 6);
    } else {
        panic!("Expected bitset");
    }
}

#[test]
fn test_bitset_with_inheritance() {
    let idl = r#"
        bitset BaseFlags {
            bitfield<1> enabled;
            bitfield<1> active;
            bitfield<6> reserved;
        };
        
        bitset ExtendedFlags : BaseFlags {
            bitfield<1> extra1;
            bitfield<1> extra2;
            bitfield<6> padding;
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the extended bitset
    let extended = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ExtendedFlags")
        .expect("ExtendedFlags not found");

    if let ic_hir::hir::DefKind::Bitset(bs) = &extended.1.kind {
        assert!(bs.parent.is_some());
        assert_eq!(bs.fields.len(), 3);
    } else {
        panic!("Expected bitset");
    }
}

#[test]
fn test_bitset_with_complex_types() {
    let idl = r#"
        enum StatusCode {
            OK = 0,
            ERROR = 1,
            PENDING = 2
        };
        
        bitset StatusFlags {
            bitfield<16, StatusCode> status;
            bitfield<1> is_final;
            bitfield<31, uint32> timestamp;
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Verify the bitset fields have correct types
    let bitset = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "StatusFlags")
        .expect("StatusFlags not found");

    if let ic_hir::hir::DefKind::Bitset(bs) = &bitset.1.kind {
        assert_eq!(bs.fields.len(), 3);

        // First field should reference the enum type
        if let ic_hir::hir::TyKind::Adt(def_id) = &bs.fields[0].ty.kind {
            let enum_def = result.context.type_of(*def_id);
            assert_eq!(enum_def.ident.name, "StatusCode");
        } else {
            panic!("Expected ADT type for status field");
        }
    } else {
        panic!("Expected bitset");
    }
}
