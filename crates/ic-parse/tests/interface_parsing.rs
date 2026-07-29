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

use ic_parse::from_str;
use ic_syntax::{InterfaceMember, Item, ParamKind};

#[test]
fn parse_empty_interface() {
    let result = from_str("interface Foo { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.name.name, "Foo");
            assert!(def.members.is_empty());
            assert!(def.inherits.is_empty());
            assert!(matches!(def.kind, ic_syntax::InterfaceKind::Regular));
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_operation() {
    let result = from_str("interface Foo { void bar(); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.members.len(), 1);
            match &def.members[0] {
                InterfaceMember::Proto(proto) => {
                    assert_eq!(proto.name.name, "bar");
                    assert!(proto.parameters.is_empty());
                    assert!(proto.raises.is_empty());
                    assert!(proto.oneway.is_none());
                }
                _ => panic!("expected prototype"),
            }
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_params() {
    let result = from_str("interface Foo { long add(in long a, in long b); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.name.name, "add");
                assert_eq!(proto.parameters.len(), 2);
                assert_eq!(proto.parameters[0].kind, Some(ParamKind::In));
                assert_eq!(proto.parameters[1].kind, Some(ParamKind::In));
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_out_params() {
    let result = from_str("interface Foo { void get(out long result); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.parameters.len(), 1);
                assert_eq!(proto.parameters[0].kind, Some(ParamKind::Out));
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_inout_params() {
    let result = from_str("interface Foo { void update(inout long value); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.parameters.len(), 1);
                assert_eq!(proto.parameters[0].kind, Some(ParamKind::InOut));
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_raises() {
    let result = from_str("interface Foo { void risky() raises (Error1, Error2); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.raises.len(), 2);
                assert_eq!(proto.raises[0].segments[0].name, "Error1");
                assert_eq!(proto.raises[1].segments[0].name, "Error2");
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_oneway() {
    let result = from_str("interface Foo { oneway void notify(in string msg); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.name.name, "notify");
                assert!(proto.oneway.is_some());
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_attribute() {
    let result = from_str("interface Foo { attribute long count; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Attribute(attr) => {
                assert!(attr.readonly.is_none());
                assert_eq!(attr.declarators.len(), 1);
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_readonly_attribute() {
    let result = from_str("interface Foo { readonly attribute string name; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Attribute(attr) => {
                assert!(attr.readonly.is_some());
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_single_inheritance() {
    let result = from_str("interface Child : Parent { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.name.name, "Child");
            assert_eq!(def.inherits.len(), 1);
            assert_eq!(def.inherits[0].segments[0].name, "Parent");
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_multiple_inheritance() {
    let result = from_str("interface Child : Parent1, Parent2, Parent3 { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.inherits.len(), 3);
            assert_eq!(def.inherits[0].segments[0].name, "Parent1");
            assert_eq!(def.inherits[1].segments[0].name, "Parent2");
            assert_eq!(def.inherits[2].segments[0].name, "Parent3");
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_local_interface() {
    let result = from_str("local interface Callback { void onEvent(); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.name.name, "Callback");
            assert!(matches!(def.kind, ic_syntax::InterfaceKind::Local(_)));
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_local_interface_with_inheritance() {
    let result = from_str("local interface LocalChild : LocalBase { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert!(matches!(def.kind, ic_syntax::InterfaceKind::Local(_)));
            assert_eq!(def.inherits.len(), 1);
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_forward_declaration() {
    let result = from_str("interface Forward;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.name.name, "Forward");
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn parse_interface_with_multiple_members() {
    let result = from_str(
        "interface Service {
            readonly attribute long count;
            attribute string name;
            long getValue(in long x);
            void setValue(in long x);
            oneway void notify(in string msg);
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.members.len(), 5);
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_nested_types() {
    let result = from_str(
        "interface Container {
            typedef long Count;
            const long MAX = 100;
            struct Item { long id; string name; };
            exception Error { string message; };
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.members.len(), 4);
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_scoped_types() {
    let result = from_str("interface Foo { Module::Type getValue(); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => match &proto.return_type {
                ic_syntax::Type::Named(path) => {
                    assert_eq!(path.segments.len(), 2);
                    assert_eq!(path.segments[0].name, "Module");
                    assert_eq!(path.segments[1].name, "Type");
                }
                _ => panic!("expected path type"),
            },
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn parse_interface_with_annotations() {
    let result = from_str("@custom interface Foo { @deprecated void oldMethod(); };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "custom");
        }
        _ => panic!("expected interface"),
    }
}
