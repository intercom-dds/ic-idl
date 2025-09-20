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

use ic_hir::hir::DefKind;
use ic_hir_xform::enum_prefix;

#[test]
fn test_enum_prefix_stripping() {
    let idl = r#"
        enum Color {
            COLOR_RED,
            COLOR_GREEN,
            COLOR_BLUE
        };
        
        enum Status {
            STATUS_OK,
            STATUS_ERROR,
            STATUS_PENDING
        };
        
        // Should not be stripped - prefix doesn't match enum name
        enum Type {
            UNKNOWN_TYPE,
            INTEGER_TYPE,
            STRING_TYPE
        };
        
        // Should not be stripped - not all have same prefix
        enum Mixed {
            MIXED_ONE,
            OTHER_TWO,
            MIXED_THREE
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = enum_prefix::transform(hir);

    for def in transformed.iter() {
        match &def.ident.name[..] {
            "Color" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    assert_eq!(const_names, vec!["BLUE", "GREEN", "RED"]);
                }
            }
            "Status" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    assert_eq!(const_names, vec!["ERROR", "OK", "PENDING"]);
                }
            }
            "Type" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    // Should not be stripped
                    assert_eq!(
                        const_names,
                        vec!["INTEGER_TYPE", "STRING_TYPE", "UNKNOWN_TYPE"]
                    );
                }
            }
            "Mixed" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    // Should not be stripped
                    assert_eq!(const_names, vec!["MIXED_ONE", "MIXED_THREE", "OTHER_TWO"]);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_bitmask_prefix_stripping() {
    let idl = r#"
        bitmask Flags {
            FLAGS_READABLE,
            FLAGS_WRITABLE,
            FLAGS_EXECUTABLE
        };
        
        bitmask Options {
            OPTIONS_VERBOSE,
            OPTIONS_DEBUG
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = enum_prefix::transform(hir);

    for def in transformed.iter() {
        match &def.ident.name[..] {
            "Flags" => {
                if let DefKind::Bitmask(b) = &def.kind {
                    let mut flag_names: Vec<_> =
                        b.flags.iter().map(|f| f.ident.name.clone()).collect();
                    flag_names.sort();
                    assert_eq!(flag_names, vec!["EXECUTABLE", "READABLE", "WRITABLE"]);
                }
            }
            "Options" => {
                if let DefKind::Bitmask(b) = &def.kind {
                    let mut flag_names: Vec<_> =
                        b.flags.iter().map(|f| f.ident.name.clone()).collect();
                    flag_names.sort();
                    assert_eq!(flag_names, vec!["DEBUG", "VERBOSE"]);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_camel_case_prefix_stripping() {
    let idl = r#"
        enum ColorType {
            ColorTypeRed,
            ColorTypeGreen,
            ColorTypeBlue
        };
        
        // Different case style in enum name
        enum color_type {
            ColorTypeAlpha,
            ColorTypeBeta
        };
    "#;

    let hir = common::parse_and_resolve(idl);
    let transformed = enum_prefix::transform(hir);

    for def in transformed.iter() {
        match &def.ident.name[..] {
            "ColorType" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    assert_eq!(const_names, vec!["Blue", "Green", "Red"]);
                }
            }
            "color_type" => {
                if let DefKind::Enum(e) = &def.kind {
                    let mut const_names: Vec<_> = e
                        .fields
                        .iter()
                        .map(|&id| transformed.context.type_of(id).ident.name.clone())
                        .collect();
                    const_names.sort();
                    assert_eq!(const_names, vec!["Alpha", "Beta"]);
                }
            }
            _ => {}
        }
    }
}
