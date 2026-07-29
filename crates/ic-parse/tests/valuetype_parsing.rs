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
use ic_syntax::{DeclKind, Item, ValueMember};

#[test]
fn parse_empty_valuetype() {
    let result = from_str("valuetype MyValue { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.name.name, "MyValue");
            assert!(def.members.is_empty());
            assert!(def.inherits.is_none());
            assert!(def.supports.is_empty());
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_public_member() {
    let result = from_str("valuetype MyValue { public long id; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.members.len(), 1);
            match &def.members[0] {
                ValueMember::State(member) => {
                    assert_eq!(member.visibility.value, ic_syntax::Visibility::Public);
                }
                _ => panic!("expected state member"),
            }
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_private_member() {
    let result = from_str("valuetype MyValue { private string secret; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::State(member) => {
                assert_ne!(member.visibility.value, ic_syntax::Visibility::Public);
            }
            _ => panic!("expected state member"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_mixed_visibility() {
    let result = from_str(
        "valuetype MyValue {
            public long id;
            private string secret;
            public string name;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.members.len(), 3);
            match &def.members[0] {
                ValueMember::State(m) => {
                    assert_eq!(m.visibility.value, ic_syntax::Visibility::Public);
                }
                _ => panic!("expected state"),
            }
            match &def.members[1] {
                ValueMember::State(m) => {
                    assert_ne!(m.visibility.value, ic_syntax::Visibility::Public);
                }
                _ => panic!("expected state"),
            }
            match &def.members[2] {
                ValueMember::State(m) => {
                    assert_eq!(m.visibility.value, ic_syntax::Visibility::Public);
                }
                _ => panic!("expected state"),
            }
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_inheritance() {
    let result = from_str("valuetype Child : Parent { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.name.name, "Child");
            assert!(def.inherits.is_some());
            let parent = def.inherits.as_ref().unwrap();
            assert_eq!(parent.segments[0].name, "Parent");
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_supports() {
    let result = from_str("valuetype MyValue supports MyInterface { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert!(!def.supports.is_empty());
            let iface = &def.supports[0];
            assert_eq!(iface.segments[0].name, "MyInterface");
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_inheritance_and_supports() {
    let result = from_str("valuetype Child : Parent supports MyInterface { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert!(def.inherits.is_some());
            assert!(!def.supports.is_empty());
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_forward_declaration() {
    let result = from_str("valuetype Forward;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.name.name, "Forward");
            assert_eq!(decl.kind, DeclKind::Valuetype);
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn parse_valuetype_with_operation() {
    let result = from_str(
        "valuetype MyValue {
            public long id;
            long getId();
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.members.len(), 2);
            match &def.members[1] {
                ValueMember::Proto(proto) => {
                    assert_eq!(proto.name.name, "getId");
                }
                _ => panic!("expected prototype"),
            }
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_oneway_operation() {
    let result = from_str(
        "valuetype MyValue {
            oneway void notify(in string msg);
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Proto(proto) => {
                assert!(proto.oneway.is_some());
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_attribute() {
    let result = from_str(
        "valuetype MyValue {
            attribute long count;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Attribute(attr) => {
                assert!(attr.readonly.is_none());
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_readonly_attribute() {
    let result = from_str(
        "valuetype MyValue {
            readonly attribute string name;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Attribute(attr) => {
                assert!(attr.readonly.is_some());
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_typedef() {
    let result = from_str(
        "valuetype MyValue {
            typedef long Count;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Item(item) => {
                assert!(matches!(item, Item::Alias(_)));
            }
            _ => panic!("expected item"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_const() {
    let result = from_str(
        "valuetype MyValue {
            const long MAX = 100;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Item(item) => {
                assert!(matches!(item, Item::Const(_)));
            }
            _ => panic!("expected item"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_struct() {
    let result = from_str(
        "valuetype MyValue {
            struct Inner { long x; };
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => match &def.members[0] {
            ValueMember::Item(item) => {
                assert!(matches!(item, Item::Struct(_)));
            }
            _ => panic!("expected item"),
        },
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_with_annotation() {
    let result = from_str("@custom valuetype MyValue { };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "custom");
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_complex_valuetype() {
    let result = from_str(
        "valuetype Person : Entity supports Serializable {
            public long id;
            public string name;
            private string ssn;
            
            typedef sequence<string> StringList;
            const long MAX_AGE = 150;
            
            attribute long age;
            readonly attribute string fullName;
            
            long getId();
            void setName(in string name);
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Valuetype(def) => {
            assert!(def.inherits.is_some());
            assert!(!def.supports.is_empty());
            // 3 state members + 2 items + 2 attrs + 2 protos = 9
            assert_eq!(def.members.len(), 9);
        }
        _ => panic!("expected valuetype"),
    }
}

#[test]
fn parse_valuetype_in_module() {
    let result = from_str(
        "module MyModule {
            valuetype MyValue { public long id; };
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Module(module) => {
            assert_eq!(module.items.len(), 1);
            assert!(matches!(&module.items[0], Item::Valuetype(_)));
        }
        _ => panic!("expected module"),
    }
}
