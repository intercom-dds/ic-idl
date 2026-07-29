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
use ic_syntax::{DeclKind, InterfaceMember, Item, Type};

#[test]
fn interface_operation_annotations_preserved() {
    let result = from_str(
        "interface Foo {
            @deprecated
            void oldMethod();
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert_eq!(proto.meta.annotations.len(), 1);
                assert_eq!(
                    proto.meta.annotations[0].path.segments[0].name,
                    "deprecated"
                );
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn interface_operation_doc_comment_preserved() {
    let result = from_str(
        "interface Foo {
            /// This method is old
            void oldMethod();
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert!(
                    !proto.meta.annotations.is_empty(),
                    "expected doc comment to be captured as annotation"
                );
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn interface_attribute_annotations_preserved() {
    let result = from_str(
        "interface Foo {
            @key
            attribute long id;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Attribute(attr) => {
                assert_eq!(attr.meta.annotations.len(), 1);
                assert_eq!(attr.meta.annotations[0].path.segments[0].name, "key");
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn interface_readonly_attribute_annotations_preserved() {
    let result = from_str(
        "interface Foo {
            @optional
            readonly attribute string name;
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Attribute(attr) => {
                assert_eq!(attr.meta.annotations.len(), 1);
                assert_eq!(attr.meta.annotations[0].path.segments[0].name, "optional");
            }
            _ => panic!("expected attribute"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn interface_oneway_operation_annotations_preserved() {
    let result = from_str(
        "interface Foo {
            @custom
            oneway void notify();
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Proto(proto) => {
                assert!(proto.oneway.is_some());
                assert_eq!(proto.meta.annotations.len(), 1);
                assert_eq!(proto.meta.annotations[0].path.segments[0].name, "custom");
            }
            _ => panic!("expected prototype"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn interface_nested_struct_annotations_preserved() {
    let result = from_str(
        "interface Foo {
            @nested
            struct Bar { long x; };
        };",
    );
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Interface(def) => match &def.members[0] {
            InterfaceMember::Item(Item::Struct(s)) => {
                assert_eq!(s.meta.annotations.len(), 1);
                assert_eq!(s.meta.annotations[0].path.segments[0].name, "nested");
            }
            _ => panic!("expected nested struct"),
        },
        _ => panic!("expected interface"),
    }
}

#[test]
fn native_declaration_trailing_comment() {
    let result = from_str("native Handle; /// A native handle type");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.kind, DeclKind::Native);
            assert!(
                !decl.meta.annotations.is_empty(),
                "expected trailing comment to be captured"
            );
        }
        _ => panic!("expected native declaration"),
    }
}

#[test]
fn interface_forward_declaration_trailing_comment() {
    let result = from_str("interface Forward; /// Forward declaration");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.kind, DeclKind::Interface);
            assert!(
                !decl.meta.annotations.is_empty(),
                "expected trailing comment to be captured"
            );
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn valuetype_forward_declaration_trailing_comment() {
    let result = from_str("valuetype Forward; /// Forward declaration");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    match &result.tree[0] {
        Item::Decl(decl) => {
            assert_eq!(decl.kind, DeclKind::Valuetype);
            assert!(
                !decl.meta.annotations.is_empty(),
                "expected trailing comment to be captured"
            );
        }
        _ => panic!("expected forward declaration"),
    }
}

#[test]
fn invalid_octal_literal_reports_error() {
    let result = from_str("const long x = 08;");
    assert!(
        !result.errors.is_empty(),
        "expected error for invalid octal literal"
    );
}

#[test]
fn empty_hex_literal_reports_error() {
    let result = from_str("const long x = 0x;");
    assert!(
        !result.errors.is_empty(),
        "expected error for empty hex literal"
    );
}

#[test]
fn valid_octal_literal_parses() {
    let result = from_str("const long x = 07;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn valid_hex_literal_parses() {
    let result = from_str("const long x = 0xFF;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn valid_decimal_literal_parses() {
    let result = from_str("const long x = 123;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn invalid_float_literal_reports_error() {
    let result = from_str("const double x = 1e;");
    assert!(
        !result.errors.is_empty(),
        "expected error for invalid float literal"
    );
}

#[test]
fn valid_float_literal_parses() {
    let result = from_str("const double x = 1.5e10;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn annotation_on_sequence_member_not_element() {
    let result = from_str("struct Foo { @optional sequence<string> value; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.fields[0];

    assert_eq!(
        field.meta.annotations.len(),
        1,
        "annotation should be on field"
    );
    assert_eq!(field.meta.annotations[0].path.segments[0].name, "optional");

    let Type::Sequence(seq) = &field.ty else {
        panic!("expected sequence type");
    };
    assert!(
        seq.element_annotations.is_empty(),
        "sequence element should have no annotations"
    );
}

#[test]
fn annotation_inside_sequence_on_element() {
    let result = from_str("struct Foo { sequence<@key string> value; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.fields[0];

    assert!(
        field.meta.annotations.is_empty(),
        "field should have no annotations"
    );

    let Type::Sequence(seq) = &field.ty else {
        panic!("expected sequence type");
    };
    assert_eq!(
        seq.element_annotations.len(),
        1,
        "annotation should be on sequence element"
    );
    assert_eq!(seq.element_annotations[0].path.segments[0].name, "key");
}

#[test]
fn annotation_after_type_on_member() {
    let result = from_str("struct Foo { sequence<string> @optional value; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.fields[0];

    assert_eq!(
        field.meta.annotations.len(),
        1,
        "annotation should be on field"
    );
    assert_eq!(field.meta.annotations[0].path.segments[0].name, "optional");
}

#[test]
fn annotation_after_declarator_on_member() {
    let result = from_str("struct Foo { sequence<string> value @optional; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.fields[0];

    assert_eq!(
        field.meta.annotations.len(),
        1,
        "annotation should be on field"
    );
    assert_eq!(field.meta.annotations[0].path.segments[0].name, "optional");
}

#[test]
fn annotation_between_long_and_double() {
    // Annotations can appear anywhere, including between multi-token types like "long double"
    let result = from_str("const long @optional double my82 = 1;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    // The type should be "long double", not "long"
    let Type::Named(p) = &c.ty else {
        panic!("expected path type");
    };
    assert_eq!(p.segments[0].name, "long double");
}

#[test]
fn annotation_between_long_and_long() {
    // Annotations between "long" and "long" for "long long" type
    let result = from_str("const long @foo long x = 1;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    let Type::Named(p) = &c.ty else {
        panic!("expected path type");
    };
    assert_eq!(p.segments[0].name, "int64"); // long long is represented as int64
}

#[test]
fn annotation_between_unsigned_and_long() {
    // Annotations between "unsigned" and "long"
    let result = from_str("const unsigned @foo long x = 1;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    let Type::Named(p) = &c.ty else {
        panic!("expected path type");
    };
    assert_eq!(p.segments[0].name, "uint32");
}

#[test]
fn annotation_between_unsigned_long_and_long() {
    // Annotations between "unsigned long" and second "long"
    let result = from_str("const unsigned long @foo long x = 1;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    let Type::Named(p) = &c.ty else {
        panic!("expected path type");
    };
    assert_eq!(p.segments[0].name, "uint64"); // unsigned long long
}

#[test]
fn annotation_before_closing_template_bracket_with_rshift() {
    // Annotation between expression and closing > in template with >> operator
    // The >> should be recognized as right-shift, not two closing brackets
    let result = from_str("typedef wstring<1 >> 2 @foo> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Alias(a) = &result.tree[0] else {
        panic!("expected typedef");
    };
    let Type::String(s) = &a.ty else {
        panic!("expected string type");
    };
    assert!(s.bound.is_some(), "expected bounded string");
}

#[test]
fn annotation_in_rshift_expression_with_identifier() {
    // Annotation after identifier in >> expression inside template
    let result = from_str("typedef wstring<x >> y @foo> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn annotation_before_rhs_in_rshift_expression() {
    // Annotation before RHS operand in >> expression inside template
    // e.g., `1 >> @foo 2` - the @foo is before the 2
    let result = from_str("typedef wstring<1 >> @foo 2> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Alias(a) = &result.tree[0] else {
        panic!("expected typedef");
    };
    let Type::String(s) = &a.ty else {
        panic!("expected string type");
    };
    assert!(s.bound.is_some(), "expected bounded string");
}

#[test]
fn annotation_qualified_before_rhs_in_rshift_expression() {
    // Qualified annotation before RHS operand in >> expression
    let result = from_str("typedef wstring<1 >> @foo::bar 2> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn annotation_global_qualified_before_rhs_in_rshift_expression() {
    // Globally qualified annotation before RHS operand in >> expression
    let result = from_str("typedef wstring<1 >> @::foo 2> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn annotation_with_space_before_scoped_name_in_rshift() {
    // Space before :: means ::bar is a separate scoped name, not part of @foo
    // Input: `1 >> @foo ::bar` where ::bar is the RHS expression
    let result = from_str("typedef wstring<1 >> @foo ::bar> MyString;");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn string_literal_escape_sequences() {
    // Test that escape sequences in strings are properly unescaped
    let result = from_str(r#"const string x = "hello\nworld";"#);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    // The value should contain an actual newline, not backslash-n
    match &c.value.value {
        ic_syntax::ExprKind::Literal(lit) => match &lit {
            ic_syntax::Literal::String(s) => {
                assert_eq!(s, "hello\nworld", "expected unescaped newline");
            }
            _ => panic!("expected string literal"),
        },
        _ => panic!("expected literal expression"),
    }
}

#[test]
fn string_literal_hex_escape() {
    let result = from_str(r#"const string x = "\x41\x42\x43";"#);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    match &c.value.value {
        ic_syntax::ExprKind::Literal(lit) => match &lit {
            ic_syntax::Literal::String(s) => {
                assert_eq!(s, "ABC", "expected hex-escaped ABC");
            }
            _ => panic!("expected string literal"),
        },
        _ => panic!("expected literal expression"),
    }
}

#[test]
fn string_literal_invalid_escape_reports_error() {
    // \z is not a valid escape sequence
    let result = from_str(r#"const string x = "hello\zworld";"#);
    assert!(
        !result.errors.is_empty(),
        "expected error for invalid escape"
    );
}

#[test]
fn string_literal_escaped_quotes() {
    let result = from_str(r#"const string x = "say \"hello\"";"#);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Const(c) = &result.tree[0] else {
        panic!("expected const");
    };
    match &c.value.value {
        ic_syntax::ExprKind::Literal(lit) => match &lit {
            ic_syntax::Literal::String(s) => {
                assert_eq!(s, r#"say "hello""#);
            }
            _ => panic!("expected string literal"),
        },
        _ => panic!("expected literal expression"),
    }
}

#[test]
fn unterminated_block_comment_reports_error() {
    // Unterminated block comments should report an error, not panic
    let result = from_str("/*!<!");
    assert!(
        !result.errors.is_empty(),
        "expected error for unterminated comment"
    );

    let result = from_str("/**<");
    assert!(
        !result.errors.is_empty(),
        "expected error for unterminated comment"
    );

    let result = from_str("/*!");
    assert!(
        !result.errors.is_empty(),
        "expected error for unterminated comment"
    );

    let result = from_str("/**");
    assert!(
        !result.errors.is_empty(),
        "expected error for unterminated comment"
    );

    let result = from_str("/*");
    assert!(
        !result.errors.is_empty(),
        "expected error for unterminated comment"
    );
}
