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
use ic_syntax::{Declarator, Item, Type};

#[test]
fn parse_simple_typedef() {
    let result = from_str("typedef long MyInt;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    let Item::Alias(def) = &result.tree[0] else {
        panic!("expected AliasValue")
    };
    assert_eq!(def.declarators.len(), 1);
    let Declarator::Name(ident) = &def.declarators[0] else {
        panic!("expected simple declarator")
    };
    assert_eq!(ident.name, "MyInt");
    let Type::Named(path) = &def.ty else {
        panic!("expected path type")
    };
    assert_eq!(path.segments[0].name, "int32");
}

#[test]
fn parse_typedef_multiple_declarators() {
    let result = from_str("typedef long A, B, C;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(result.tree.len(), 1);

    match &result.tree[0] {
        Item::Alias(def) => {
            assert_eq!(def.declarators.len(), 3);
            let names: Vec<_> = def
                .declarators
                .iter()
                .map(|d| match d {
                    Declarator::Name(ident) => ident.name.as_str(),
                    Declarator::Array(arr) => arr.name.name.as_str(),
                })
                .collect();
            assert_eq!(names, vec!["A", "B", "C"]);
        }
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_array() {
    let result = from_str("typedef long Arr[10];");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Alias(def) = &result.tree[0] else {
        panic!("expected AliasValue")
    };
    assert_eq!(def.declarators.len(), 1);
    let Declarator::Array(arr) = &def.declarators[0] else {
        panic!("expected array declarator")
    };
    assert_eq!(arr.name.name, "Arr");
    assert_eq!(arr.bounds.len(), 1);
}

#[test]
fn parse_typedef_multidim_array() {
    let result = from_str("typedef long Matrix[10][20][30];");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Alias(def) = &result.tree[0] else {
        panic!("expected AliasValue")
    };
    let Declarator::Array(arr) = &def.declarators[0] else {
        panic!("expected array declarator")
    };
    assert_eq!(arr.name.name, "Matrix");
    assert_eq!(arr.bounds.len(), 3);
}

#[test]
fn parse_typedef_with_annotation() {
    let result = from_str("@custom typedef long MyInt;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => {
            assert_eq!(def.meta.annotations.len(), 1);
            assert_eq!(def.meta.annotations[0].path.segments[0].name, "custom");
        }
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_user_type() {
    let result = from_str("typedef MyModule::MyType AliasName;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Named(path) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].name, "MyModule");
                assert_eq!(path.segments[1].name, "MyType");
            }
            _ => panic!("expected path type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_sequence_unbounded() {
    let result = from_str("typedef sequence<long> LongSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Sequence(seq) => {
                assert!(seq.bound.is_none());
                match &seq.element {
                    Type::Named(path) => assert_eq!(path.segments[0].name, "int32"),
                    _ => panic!("expected path type inside sequence"),
                }
            }
            _ => panic!("expected sequence type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_sequence_bounded() {
    let result = from_str("typedef sequence<long, 100> BoundedSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Sequence(seq) => {
                assert!(seq.bound.is_some());
            }
            _ => panic!("expected sequence type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_sequence_nested() {
    let result = from_str("typedef sequence<sequence<long>> NestedSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Sequence(outer) => match &outer.element {
                Type::Sequence(inner) => match &inner.element {
                    Type::Named(path) => assert_eq!(path.segments[0].name, "int32"),
                    _ => panic!("expected path type"),
                },
                _ => panic!("expected nested sequence"),
            },
            _ => panic!("expected sequence type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_sequence_with_annotation() {
    let result = from_str("typedef sequence<@key long> AnnotatedSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Sequence(seq) => {
                assert_eq!(seq.element_annotations.len(), 1);
                assert_eq!(seq.element_annotations[0].path.segments[0].name, "key");
            }
            _ => panic!("expected sequence type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_string_unbounded() {
    let result = from_str("typedef string MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::String(s) => {
                assert_ne!(s.kind, ic_syntax::StringKind::Wide);
                assert!(s.bound.is_none());
            }
            _ => panic!("expected string type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_string_bounded() {
    let result = from_str("typedef string<256> ShortString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::String(s) => {
                assert_ne!(s.kind, ic_syntax::StringKind::Wide);
                assert!(s.bound.is_some());
            }
            _ => panic!("expected string type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_wstring_unbounded() {
    let result = from_str("typedef wstring WideString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::String(s) => {
                assert_eq!(s.kind, ic_syntax::StringKind::Wide);
                assert!(s.bound.is_none());
            }
            _ => panic!("expected string type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_wstring_bounded() {
    let result = from_str("typedef wstring<100> BoundedWide;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::String(s) => {
                assert_eq!(s.kind, ic_syntax::StringKind::Wide);
                assert!(s.bound.is_some());
            }
            _ => panic!("expected string type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_fixed() {
    let result = from_str("typedef fixed<10, 2> Money;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Fixed(f) => {
                assert!(f.bounds.is_some());
            }
            _ => panic!("expected fixed type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_map_basic() {
    let result = from_str("typedef map<string, long> StrToLong;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Map(m) => {
                assert!(m.bound.is_none());
                match &m.key {
                    Type::String(s) => assert_ne!(s.kind, ic_syntax::StringKind::Wide),
                    _ => panic!("expected string key type"),
                }
                match &m.value {
                    Type::Named(path) => assert_eq!(path.segments[0].name, "int32"),
                    _ => panic!("expected path value type"),
                }
            }
            _ => panic!("expected map type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_map_bounded() {
    let result = from_str("typedef map<string, long, 50> BoundedMap;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Map(m) => {
                assert!(m.bound.is_some());
            }
            _ => panic!("expected map type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_map_with_annotations() {
    let result = from_str("typedef map<@key string, @value long> AnnotatedMap;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Map(m) => {
                assert_eq!(m.key_annotations.len(), 1);
                assert_eq!(m.key_annotations[0].path.segments[0].name, "key");
                assert_eq!(m.value_annotations.len(), 1);
                assert_eq!(m.value_annotations[0].path.segments[0].name, "value");
            }
            _ => panic!("expected map type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_map_nested_sequence() {
    let result = from_str("typedef map<string, sequence<long>> MapOfSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Map(m) => match m.value {
                Type::Sequence(_) => {}
                _ => panic!("expected sequence value type"),
            },
            _ => panic!("expected map type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_sequence_of_string() {
    let result = from_str("typedef sequence<string<100>, 50> BoundedStrSeq;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Alias(def) => match &def.ty {
            Type::Sequence(seq) => {
                assert!(seq.bound.is_some());
                match &seq.element {
                    Type::String(s) => {
                        assert_ne!(s.kind, ic_syntax::StringKind::Wide);
                        assert!(s.bound.is_some());
                    }
                    _ => panic!("expected string type inside sequence"),
                }
            }
            _ => panic!("expected sequence type"),
        },
        _ => panic!("expected AliasValue"),
    }
}

#[test]
fn parse_typedef_in_module() {
    let result = from_str("module Types { typedef long MyInt; typedef string MyStr; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Module(def) => {
            assert_eq!(def.items.len(), 2);
            assert!(matches!(&def.items[0], Item::Alias(_)));
            assert!(matches!(&def.items[1], Item::Alias(_)));
        }
        _ => panic!("expected module"),
    }
}

#[test]
fn parse_typedef_missing_semicolon() {
    let result = from_str("typedef long MyInt");
    assert!(!result.errors.is_empty());
}

#[test]
fn parse_typedef_missing_declarator() {
    let result = from_str("typedef long ;");
    assert!(!result.errors.is_empty());
}

#[test]
fn parse_typedef_missing_type() {
    let result = from_str("typedef MyInt;");
    // This actually parses as typedef with user-defined type "MyInt" as the type
    // and expects a declarator. So it should fail because there's no declarator.
    assert!(!result.errors.is_empty());
}
