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
use ic_syntax::Item;

fn assert_has_annotation(result: &ic_parse::ParseResult, name: &str) {
    assert!(
        result.errors.is_empty(),
        "parse errors: {:?}",
        result.errors
    );
    assert!(
        result.orphaned_annotations.is_empty(),
        "orphaned annotations: {:?}",
        result
            .orphaned_annotations
            .iter()
            .map(|a| &a.ident.segments[0].name)
            .collect::<Vec<_>>()
    );

    let item = &result.tree[0];
    let annotations = match item {
        Item::StructValue(s) => &s.annotations,
        Item::AliasValue(a) => &a.annotations,
        Item::ConstValue(c) => &c.annotations,
        Item::EnumValue(e) => &e.annotations,
        Item::UnionValue(u) => &u.annotations,
        Item::InterfaceValue(i) => &i.annotations,
        Item::ExceptionValue(e) => &e.annotations,
        Item::BitsetValue(b) => &b.annotations,
        Item::BitmaskValue(b) => &b.annotations,
        Item::DeclValue(d) => &d.annotations,
        Item::ModuleValue(m) => &m.annotations,
        Item::AnnotationValue(a) => &a.annotations,
        Item::ValuetypeValue(v) => &v.annotations,
    };

    assert!(
        annotations.iter().any(|a| a.ident.segments[0].name == name),
        "expected annotation @{} on {:?}, found {:?}",
        name,
        std::mem::discriminant(item),
        annotations
            .iter()
            .map(|a| &a.ident.segments[0].name)
            .collect::<Vec<_>>()
    );
}

fn assert_member_has_annotation(result: &ic_parse::ParseResult, member_idx: usize, name: &str) {
    assert!(
        result.errors.is_empty(),
        "parse errors: {:?}",
        result.errors
    );
    assert!(
        result.orphaned_annotations.is_empty(),
        "orphaned annotations: {:?}",
        result
            .orphaned_annotations
            .iter()
            .map(|a| &a.ident.segments[0].name)
            .collect::<Vec<_>>()
    );

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };

    let field = &s.members[member_idx];
    assert!(
        field
            .annotations
            .iter()
            .any(|a| a.ident.segments[0].name == name),
        "expected annotation @{} on member {}, found {:?}",
        name,
        member_idx,
        field
            .annotations
            .iter()
            .map(|a| &a.ident.segments[0].name)
            .collect::<Vec<_>>()
    );
}

// Struct members

#[test]
fn struct_member_annotation_before_type() {
    let result = from_str("struct Foo { @optional sequence<string> value; };");
    assert_member_has_annotation(&result, 0, "optional");
}

#[test]
fn struct_member_annotation_after_type() {
    let result = from_str("struct Foo { sequence<string> @optional value; };");
    assert_member_has_annotation(&result, 0, "optional");
}

#[test]
fn struct_member_annotation_after_declarator() {
    let result = from_str("struct Foo { sequence<string> value @optional; };");
    assert_member_has_annotation(&result, 0, "optional");
}

#[test]
fn struct_member_annotation_all_positions() {
    let result = from_str(
        "struct Foo {
            @a sequence<string> value1;
            sequence<string> @b value2;
            sequence<string> value3 @c;
        };",
    );
    assert_member_has_annotation(&result, 0, "a");
    assert_member_has_annotation(&result, 1, "b");
    assert_member_has_annotation(&result, 2, "c");
}

// Struct declaration

#[test]
fn struct_annotation_before_keyword() {
    let result = from_str("@optional struct Foo { long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn struct_annotation_after_keyword() {
    let result = from_str("struct @optional Foo { long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn struct_annotation_after_name() {
    let result = from_str("struct Foo @optional { long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn struct_annotation_after_body() {
    let result = from_str("struct Foo { long x; } @optional;");
    assert_has_annotation(&result, "optional");
}

// Typedef

#[test]
fn typedef_annotation_before_keyword() {
    let result = from_str("@optional typedef string MyString;");
    assert_has_annotation(&result, "optional");
}

#[test]
fn typedef_annotation_after_keyword() {
    let result = from_str("typedef @optional string MyString;");
    assert_has_annotation(&result, "optional");
}

#[test]
fn typedef_annotation_after_type() {
    let result = from_str("typedef string @optional MyString;");
    assert_has_annotation(&result, "optional");
}

#[test]
fn typedef_annotation_after_declarator() {
    let result = from_str("typedef string MyString @optional;");
    assert_has_annotation(&result, "optional");
}

// Const

#[test]
fn const_annotation_before_keyword() {
    let result = from_str("@optional const string MyConst = \"foo\";");
    assert_has_annotation(&result, "optional");
}

#[test]
fn const_annotation_after_keyword() {
    let result = from_str("const @optional string MyConst = \"foo\";");
    assert_has_annotation(&result, "optional");
}

#[test]
fn const_annotation_after_type() {
    let result = from_str("const string @optional MyConst = \"foo\";");
    assert_has_annotation(&result, "optional");
}

#[test]
fn const_annotation_after_declarator() {
    let result = from_str("const string MyConst @optional = \"foo\";");
    assert_has_annotation(&result, "optional");
}

#[test]
fn const_annotation_after_equals() {
    let result = from_str("const string MyConst = @optional \"foo\";");
    assert_has_annotation(&result, "optional");
}

#[test]
fn const_annotation_after_value() {
    let result = from_str("const string MyConst = \"foo\" @optional;");
    assert_has_annotation(&result, "optional");
}

// Enum

#[test]
fn enum_annotation_before_keyword() {
    let result = from_str("@optional enum MyEnum { ZERO };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn enum_annotation_after_keyword() {
    let result = from_str("enum @optional MyEnum { ZERO };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn enum_annotation_after_name() {
    let result = from_str("enum MyEnum @optional { ZERO };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn enum_annotation_after_body() {
    let result = from_str("enum MyEnum { ZERO } @optional;");
    assert_has_annotation(&result, "optional");
}

// Union

#[test]
fn union_annotation_before_keyword() {
    let result = from_str("@optional union MyUnion switch (long) { case 0: long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn union_annotation_after_keyword() {
    let result = from_str("union @optional MyUnion switch (long) { case 0: long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn union_annotation_after_name() {
    let result = from_str("union MyUnion @optional switch (long) { case 0: long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn union_annotation_after_body() {
    let result = from_str("union MyUnion switch (long) { case 0: long x; } @optional;");
    assert_has_annotation(&result, "optional");
}

// Interface

#[test]
fn interface_annotation_before_keyword() {
    let result = from_str("@optional interface MyInterface {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn interface_annotation_after_keyword() {
    let result = from_str("interface @optional MyInterface {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn interface_annotation_after_name() {
    let result = from_str("interface MyInterface @optional {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn interface_annotation_after_body() {
    let result = from_str("interface MyInterface {} @optional;");
    assert_has_annotation(&result, "optional");
}

// Exception

#[test]
fn exception_annotation_before_keyword() {
    let result = from_str("@optional exception MyException {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn exception_annotation_after_keyword() {
    let result = from_str("exception @optional MyException {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn exception_annotation_after_name() {
    let result = from_str("exception MyException @optional {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn exception_annotation_after_body() {
    let result = from_str("exception MyException {} @optional;");
    assert_has_annotation(&result, "optional");
}

// Bitmask

#[test]
fn bitmask_annotation_before_keyword() {
    let result = from_str("@optional bitmask MyBitmask { FLAG };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitmask_annotation_after_keyword() {
    let result = from_str("bitmask @optional MyBitmask { FLAG };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitmask_annotation_after_name() {
    let result = from_str("bitmask MyBitmask @optional { FLAG };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitmask_annotation_after_body() {
    let result = from_str("bitmask MyBitmask { FLAG } @optional;");
    assert_has_annotation(&result, "optional");
}

// Bitset

#[test]
fn bitset_annotation_before_keyword() {
    let result = from_str("@optional bitset MyBitset { bitfield<4> field; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitset_annotation_after_keyword() {
    let result = from_str("bitset @optional MyBitset { bitfield<4> field; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitset_annotation_after_name() {
    let result = from_str("bitset MyBitset @optional { bitfield<4> field; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn bitset_annotation_after_body() {
    let result = from_str("bitset MyBitset { bitfield<4> field; } @optional;");
    assert_has_annotation(&result, "optional");
}

// Module

#[test]
fn module_annotation_before_keyword() {
    let result = from_str("@optional module MyModule {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn module_annotation_after_keyword() {
    let result = from_str("module @optional MyModule {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn module_annotation_after_name() {
    let result = from_str("module MyModule @optional {};");
    assert_has_annotation(&result, "optional");
}

#[test]
fn module_annotation_after_body() {
    let result = from_str("module MyModule {} @optional;");
    assert_has_annotation(&result, "optional");
}

// Native

#[test]
fn native_annotation_before_keyword() {
    let result = from_str("@optional native MyNative;");
    assert_has_annotation(&result, "optional");
}

#[test]
fn native_annotation_after_keyword() {
    let result = from_str("native @optional MyNative;");
    assert_has_annotation(&result, "optional");
}

#[test]
fn native_annotation_after_name() {
    let result = from_str("native MyNative @optional;");
    assert_has_annotation(&result, "optional");
}

// Valuetype

#[test]
fn valuetype_annotation_before_keyword() {
    let result = from_str("@optional valuetype MyValue { public long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn valuetype_annotation_after_keyword() {
    let result = from_str("valuetype @optional MyValue { public long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn valuetype_annotation_after_name() {
    let result = from_str("valuetype MyValue @optional { public long x; };");
    assert_has_annotation(&result, "optional");
}

#[test]
fn valuetype_annotation_after_body() {
    let result = from_str("valuetype MyValue { public long x; } @optional;");
    assert_has_annotation(&result, "optional");
}

// Template types - annotations inside <> should go to element type

#[test]
fn sequence_element_annotation() {
    let result = from_str("struct Foo { sequence<@key string> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    assert!(field.annotations.is_empty());

    let ic_syntax::Type::Sequence(seq) = &field.ty else {
        panic!("expected sequence");
    };
    assert_eq!(seq.annotations.len(), 1);
    assert_eq!(seq.annotations[0].ident.segments[0].name, "key");
}

#[test]
fn map_key_annotation() {
    let result = from_str("struct Foo { map<@key string, long> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    assert!(field.annotations.is_empty());

    let ic_syntax::Type::Map(m) = &field.ty else {
        panic!("expected map");
    };
    assert_eq!(m.key_annotations.len(), 1);
    assert_eq!(m.key_annotations[0].ident.segments[0].name, "key");
    assert!(m.value_annotations.is_empty());
}

#[test]
fn map_value_annotation() {
    let result = from_str("struct Foo { map<string, @optional long> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    assert!(field.annotations.is_empty());

    let ic_syntax::Type::Map(m) = &field.ty else {
        panic!("expected map");
    };
    assert!(m.key_annotations.is_empty());
    assert_eq!(m.value_annotations.len(), 1);
    assert_eq!(m.value_annotations[0].ident.segments[0].name, "optional");
}

// Multiple annotations

#[test]
fn multiple_annotations_different_positions() {
    let result = from_str("@a struct @b Foo @c { long x; } @d;");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    assert_eq!(s.annotations.len(), 4);
    let names: Vec<_> = s
        .annotations
        .iter()
        .map(|a| a.ident.segments[0].name.as_str())
        .collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert!(names.contains(&"c"));
    assert!(names.contains(&"d"));
}

// Orphaned annotations - these should NOT attach to anything

fn assert_orphaned(result: &ic_parse::ParseResult, name: &str) {
    assert!(
        result.errors.is_empty(),
        "parse errors: {:?}",
        result.errors
    );
    assert!(
        result
            .orphaned_annotations
            .iter()
            .any(|a| a.ident.segments[0].name == name),
        "expected @{} to be orphaned, but orphaned list is: {:?}",
        name,
        result
            .orphaned_annotations
            .iter()
            .map(|a| &a.ident.segments[0].name)
            .collect::<Vec<_>>()
    );
}

#[test]
fn orphaned_annotation_after_struct_member_semi() {
    let result = from_str("struct Foo { string value; @orphan };");
    assert_orphaned(&result, "orphan");
}

#[test]
fn orphaned_annotation_inside_empty_struct() {
    let result = from_str("struct Foo { @orphan };");
    assert_orphaned(&result, "orphan");
}

#[test]
fn annotation_between_struct_members_attaches_to_next() {
    let result = from_str("struct Foo { long a; @attached long b; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    assert!(s.members[0].annotations.is_empty());
    assert_eq!(s.members[1].annotations.len(), 1);
    assert_eq!(
        s.members[1].annotations[0].ident.segments[0].name,
        "attached"
    );
}

#[test]
fn orphaned_annotation_inside_empty_enum() {
    let result = from_str("enum Foo { @orphan };");
    assert_orphaned(&result, "orphan");
}

#[test]
fn sequence_element_annotation_after_type() {
    let result = from_str("struct Foo { sequence<string @key> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    let ic_syntax::Type::Sequence(seq) = &field.ty else {
        panic!("expected sequence");
    };
    assert_eq!(seq.annotations.len(), 1);
    assert_eq!(seq.annotations[0].ident.segments[0].name, "key");
}

#[test]
fn map_value_annotation_after_type() {
    let result = from_str("struct Foo { map<string, long @optional> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    let ic_syntax::Type::Map(m) = &field.ty else {
        panic!("expected map");
    };
    assert_eq!(m.value_annotations.len(), 1);
    assert_eq!(m.value_annotations[0].ident.segments[0].name, "optional");
}

#[test]
fn map_key_annotation_after_type() {
    let result = from_str("struct Foo { map<string @key, long> value; };");
    assert!(result.errors.is_empty());
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    let ic_syntax::Type::Map(m) = &field.ty else {
        panic!("expected map");
    };
    assert_eq!(m.key_annotations.len(), 1);
    assert_eq!(m.key_annotations[0].ident.segments[0].name, "key");
}

#[test]
fn orphaned_annotation_inside_string_bounds() {
    let result = from_str("struct Foo { string<@orphan 10> value; };");
    assert_orphaned(&result, "orphan");
}

#[test]
fn nested_sequence_annotations() {
    let result =
        from_str("struct Foo { @outer sequence<@inner sequence<@innermost string>> value; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(result.orphaned_annotations.is_empty());

    let Item::StructValue(s) = &result.tree[0] else {
        panic!("expected struct");
    };
    let field = &s.members[0];
    assert_eq!(field.annotations.len(), 1);
    assert_eq!(field.annotations[0].ident.segments[0].name, "outer");

    let ic_syntax::Type::Sequence(outer_seq) = &field.ty else {
        panic!("expected outer sequence");
    };
    assert_eq!(outer_seq.annotations.len(), 1);
    assert_eq!(outer_seq.annotations[0].ident.segments[0].name, "inner");

    let ic_syntax::Type::Sequence(inner_seq) = outer_seq.ty.as_ref() else {
        panic!("expected inner sequence");
    };
    assert_eq!(inner_seq.annotations.len(), 1);
    assert_eq!(inner_seq.annotations[0].ident.segments[0].name, "innermost");
}
