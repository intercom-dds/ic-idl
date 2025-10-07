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

#[test]
fn test_bitmask_flag_annotations() {
    let idl = r#"
        @annotation FlagInfo {
            string description;
            boolean deprecated default FALSE;
        };
        
        bitmask Status {
            @FlagInfo("Active status")
            ACTIVE = 0,
            
            @FlagInfo("Inactive status", true)
            INACTIVE = 1,
            
            @FlagInfo("Pending review")
            PENDING
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the bitmask definition
    let bitmask_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Bitmask(b) => {
                if def.ident.name == "Status" {
                    Some(b)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find Status bitmask");

    // Check that flags have annotations
    assert_eq!(bitmask_def.flags.len(), 3);

    // Check ACTIVE flag
    let active_flag_def = result.context.definitions.get(bitmask_def.flags[0]);
    assert_eq!(active_flag_def.ident.name, "ACTIVE");
    assert_eq!(active_flag_def.annotations.len(), 1);
    assert_eq!(active_flag_def.annotations[0].ident.name, "FlagInfo");

    // Check INACTIVE flag
    let inactive_flag_def = result.context.definitions.get(bitmask_def.flags[1]);
    assert_eq!(inactive_flag_def.ident.name, "INACTIVE");
    assert_eq!(inactive_flag_def.annotations.len(), 1);
    assert_eq!(inactive_flag_def.annotations[0].ident.name, "FlagInfo");

    // Check PENDING flag
    let pending_flag_def = result.context.definitions.get(bitmask_def.flags[2]);
    assert_eq!(pending_flag_def.ident.name, "PENDING");
    assert_eq!(pending_flag_def.annotations.len(), 1);
    assert_eq!(pending_flag_def.annotations[0].ident.name, "FlagInfo");
}

#[test]
fn test_bitset_field_annotations() {
    let idl = r#"
        @annotation FieldInfo {
            string description;
            long maxValue;
        };
        
        bitset Configuration {
            @FieldInfo("System mode", 3)
            bitfield<3, unsigned short> mode;
            
            @FieldInfo("Feature flags", 255)
            bitfield<8, unsigned short> features;
            
            @FieldInfo("Reserved for future use", 31)
            bitfield<5, unsigned short> reserved;
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the bitset definition
    let bitset_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Bitset(b) => {
                if def.ident.name == "Configuration" {
                    Some(b)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find Configuration bitset");

    // Check that fields have annotations
    assert_eq!(bitset_def.fields.len(), 3);

    // Check mode field
    let mode_field = &bitset_def.fields[0];
    assert_eq!(mode_field.ident.name, "mode");
    assert_eq!(mode_field.annotations.len(), 1);
    assert_eq!(mode_field.annotations[0].ident.name, "FieldInfo");

    // Check features field
    let features_field = &bitset_def.fields[1];
    assert_eq!(features_field.ident.name, "features");
    assert_eq!(features_field.annotations.len(), 1);
    assert_eq!(features_field.annotations[0].ident.name, "FieldInfo");

    // Check reserved field
    let reserved_field = &bitset_def.fields[2];
    assert_eq!(reserved_field.ident.name, "reserved");
    assert_eq!(reserved_field.annotations.len(), 1);
    assert_eq!(reserved_field.annotations[0].ident.name, "FieldInfo");
}

#[test]
fn test_bitmask_with_multiple_annotations() {
    let idl = r#"
        @annotation Deprecated {
            string reason;
        };
        
        @annotation Since {
            string version;
        };
        
        bitmask Permissions {
            @Since("1.0")
            READ = 0,
            
            @Since("1.0")
            WRITE = 1,
            
            @Deprecated("Use WRITE instead")
            @Since("1.0")
            MODIFY = 2,
            
            @Since("1.5")
            DELETE = 3
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the bitmask definition
    let bitmask_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Bitmask(b) => {
                if def.ident.name == "Permissions" {
                    Some(b)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find Permissions bitmask");

    // Check MODIFY flag has multiple annotations
    let modify_flag_def = result.context.definitions.get(bitmask_def.flags[2]);
    assert_eq!(modify_flag_def.ident.name, "MODIFY");
    assert_eq!(modify_flag_def.annotations.len(), 2);

    // Check annotation names (order might vary)
    let ann_names: Vec<&str> = modify_flag_def
        .annotations
        .iter()
        .map(|a| a.ident.name.as_str())
        .collect();
    assert!(ann_names.contains(&"Deprecated"));
    assert!(ann_names.contains(&"Since"));
}
