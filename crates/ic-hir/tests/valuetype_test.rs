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

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn valuetype_enum_values_assigned() {
    let idl = r"
        valuetype Foo {
            enum Status {
                ACTIVE,
                INACTIVE,
                PENDING
            };
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the enum values - they are stored as const definitions
    let mut active_value = None;
    let mut inactive_value = None;
    let mut pending_value = None;

    for (_, def) in &result.context.definitions {
        if def.ident.name == "ACTIVE" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    active_value = Some(val);
                }
            }
        } else if def.ident.name == "INACTIVE" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    inactive_value = Some(val);
                }
            }
        } else if def.ident.name == "PENDING" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    pending_value = Some(val);
                }
            }
        }
    }

    // Check that enum values are properly assigned
    assert_eq!(active_value, Some(0));
    assert_eq!(inactive_value, Some(1));
    assert_eq!(pending_value, Some(2));
}

#[test]
fn valuetype_nested_types_resolved() {
    let idl = r"
        valuetype Example {
            enum Status {
                ACTIVE,
                INACTIVE
            };
            
            struct Data {
                string name;
                long value;
            };
            
            public Status current_status;
            public Data info;
            
            Status getStatus();
            void updateData(in Data new_data);
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the valuetype definition
    let valuetype_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Valuetype(v) => {
                if def.ident.name == "Example" {
                    Some(v)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find Example valuetype");

    // Check that members have resolved types
    assert_eq!(valuetype_def.members.len(), 2);
    assert_eq!(valuetype_def.members[0].ident.name, "current_status");
    assert_eq!(valuetype_def.members[1].ident.name, "info");

    // Check operations
    assert_eq!(valuetype_def.prototypes.len(), 2);
    assert_eq!(valuetype_def.prototypes[0].ident.name, "getStatus");
    assert_eq!(valuetype_def.prototypes[1].ident.name, "updateData");
}

#[test]
fn valuetype_with_multiple_nested_types() {
    let idl = r"
        valuetype Complex {
            struct Point {
                float x;
                float y;
            };
            
            enum Color {
                RED,
                GREEN,
                BLUE
            };
            
            typedef sequence<Point> PointList;
            
            public Point origin;
            public Color primary_color;
            public PointList vertices;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the valuetype definition
    let valuetype_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Valuetype(v) => {
                if def.ident.name == "Complex" {
                    Some(v)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find Complex valuetype");

    // Check that all members are present
    assert_eq!(valuetype_def.members.len(), 3);

    // Verify member names
    assert_eq!(valuetype_def.members[0].ident.name, "origin");
    assert_eq!(valuetype_def.members[1].ident.name, "primary_color");
    assert_eq!(valuetype_def.members[2].ident.name, "vertices");
}

#[test]
fn valuetype_operations_with_nested_types() {
    let idl = r"
        valuetype MyService {
            struct Request {
                string id;
                long timestamp;
            };
            
            struct Response {
                boolean success;
                string message;
            };
            
            Response process(in Request req);
            void log(in Request req, in Response resp);
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the valuetype definition
    let valuetype_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Valuetype(v) => {
                if def.ident.name == "MyService" {
                    Some(v)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find MyService valuetype");

    // Check operations
    assert_eq!(valuetype_def.prototypes.len(), 2);

    // Check first operation
    let process_op = &valuetype_def.prototypes[0];
    assert_eq!(process_op.ident.name, "process");

    // Check parameter
    assert_eq!(process_op.params.len(), 1);
    assert_eq!(process_op.params[0].ident.name, "req");
}

#[test]
fn valuetype_enum_explicit_values() {
    let idl = r"
        valuetype Config {
            enum Level {
                DEBUG = 10,
                INFO = 20,
                WARN = 30,
                ERROR = 40
            };
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the enum values
    let mut debug_value = None;
    let mut info_value = None;
    let mut warn_value = None;
    let mut error_value = None;

    for (_, def) in &result.context.definitions {
        if def.ident.name == "DEBUG" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    debug_value = Some(val);
                }
            }
        } else if def.ident.name == "INFO" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    info_value = Some(val);
                }
            }
        } else if def.ident.name == "WARN" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    warn_value = Some(val);
                }
            }
        } else if def.ident.name == "ERROR" {
            if let DefKind::Const(const_ty) = &def.kind {
                if let Numeric::Int32(val) = const_ty.value {
                    error_value = Some(val);
                }
            }
        }
    }

    // Check explicit values
    assert_eq!(debug_value, Some(10));
    assert_eq!(info_value, Some(20));
    assert_eq!(warn_value, Some(30));
    assert_eq!(error_value, Some(40));
}

#[test]
fn valuetype_declaration_order_matters() {
    let idl = r"
        valuetype OrderTest {
            struct Data {
                string value;
            };
            
            // This should work - Data is defined above
            public Data valid_member;
            
            // Define another type after member
            enum Status {
                OK,
                ERROR
            };
            
            // This should also work - Status is defined above
            public Status current_status;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the valuetype definition
    let valuetype_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Valuetype(v) => {
                if def.ident.name == "OrderTest" {
                    Some(v)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find OrderTest valuetype");

    // Both members should be successfully resolved
    assert_eq!(valuetype_def.members.len(), 2);
    assert_eq!(valuetype_def.members[0].ident.name, "valid_member");
    assert_eq!(valuetype_def.members[1].ident.name, "current_status");
}

#[test]
fn valuetype_with_attributes() {
    let idl = r"
        valuetype PropertyBag {
            struct Item {
                string key;
                any value;
            };
            
            attribute Item primary_item;
            readonly attribute long count;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the valuetype definition
    let valuetype_def = result
        .context
        .definitions
        .iter()
        .find_map(|(_, def)| match &def.kind {
            DefKind::Valuetype(v) => {
                if def.ident.name == "PropertyBag" {
                    Some(v)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("Should find PropertyBag valuetype");

    // Check attributes
    assert_eq!(valuetype_def.attributes.len(), 2);

    // Check first attribute
    assert_eq!(valuetype_def.attributes[0].ident.name, "primary_item");
    assert!(!valuetype_def.attributes[0].is_readonly);

    // Check second attribute
    assert_eq!(valuetype_def.attributes[1].ident.name, "count");
    assert!(valuetype_def.attributes[1].is_readonly);
}

#[test]
fn valuetype_type_before_definition_error() {
    let idl = r"
        valuetype BadOrder {
            // Error: Status is used before it's defined
            public Status current_status;
            
            enum Status {
                ACTIVE,
                INACTIVE
            };
            
            // This would also fail
            Data getData();
            
            struct Data {
                string value;
            };
        };
    ";

    let (result, _source_map, rendered) = common::parse_and_resolve(idl);

    // Should have errors
    assert!(!result.errors.is_empty());

    // Snapshot test for the error output
    insta::assert_snapshot!(rendered);
}
