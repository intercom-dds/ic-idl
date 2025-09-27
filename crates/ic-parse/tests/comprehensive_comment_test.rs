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

use ic_parse::from_file;
use ic_preproc::ProcArgs;
use ic_syntax::{AnnotationAppl, Expr, Item, Literal, LiteralValue};
use ic_vfs::SourceMap;

fn get_doc_strings(annotations: &[AnnotationAppl]) -> Vec<String> {
    annotations
        .iter()
        .filter(|a| a.ident.segments[0].name == "doc")
        .map(|a| {
            if let Some(arg) = a.args.first() {
                if let Expr::Literal(Literal {
                    value: LiteralValue::String(s),
                    ..
                }) = &arg.value
                {
                    s.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        })
        .collect()
}

#[test]
fn test_struct_trailing_comment() {
    use ic_parse::from_file;
    use ic_preproc::ProcArgs;
    use ic_syntax::Item;
    use ic_vfs::SourceMap;

    let mut vfs = SourceMap::default();
    let content = r"/// Leading comment for MyStruct
struct MyStruct /** inline comment */ {
    ///< Trailing at start of struct
    /// Doc for field1
    string field1; ///< Trailing for field1
    /// Should be dropped
}; ///< Trailing after struct";

    let file = vfs.embed(content);
    let args = ProcArgs::default();
    let parse_result = from_file(file, args, &mut vfs);

    assert!(parse_result.errors.is_empty());

    // Debug assertions to verify comment positions
    assert!(content.contains("/// Leading comment"));
    assert!(content.contains("/** inline comment */"));
    assert!(content.contains("///< Trailing at start"));
    assert!(content.contains("/// Doc for field1"));
    assert!(content.contains("///< Trailing for field1"));
    assert!(content.contains("/// Should be dropped"));
    assert!(content.contains("///< Trailing after struct"));

    if let Item::StructValue(s) = &parse_result.tree[0] {
        let docs = get_doc_strings(&s.annotations);

        // Verify struct has expected number of doc comments
        assert!(!docs.is_empty(), "Struct should have doc comments");

        // The issue: we're expecting 4 comments but only getting 3
        // Missing: "< Trailing after struct"
        assert_eq!(
            docs.len(),
            4,
            "Expected 4 comments, got {}: {:?}",
            docs.len(),
            docs
        );
        assert!(
            docs.contains(&"Trailing after struct".to_string()),
            "Missing trailing comment after struct"
        );
    } else {
        panic!("Expected a struct");
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_comprehensive_comment_attachment() {
    let mut vfs = SourceMap::default();

    let content = r"/// Leading comment for MyStruct
struct MyStruct /** inline comment */ {
    ///< Trailing at start of struct
    /// Doc for field1
    string field1; ///< Trailing for field1
    /// Should be dropped
}; ///< Trailing after struct

/// Leading comment for MyUnion
union MyUnion /** inline union */ switch (int32) {
    ///< Trailing at start of union
    /// Doc for case 1
    case 1:
        string option1; ///< Trailing for option1
    /// Doc for case 2  
    case 2:
        int32 option2;
    /// Should be dropped
}; ///< Trailing after union

/// Leading comment for MyEnum  
enum MyEnum /** inline enum */ {
    ///< Trailing at start of enum
    /// Doc for VALUE1
    VALUE1, ///< Trailing for VALUE1
    /// Doc for VALUE2
    VALUE2 = 10 ///< Trailing for VALUE2
    /// Should be dropped
}; ///< Trailing after enum

/// Leading comment for MyException
exception MyException /** inline exception */ {
    ///< Trailing at start of exception
    /// Doc for message
    string message; ///< Trailing for message
    /// Doc for code
    int32 code;
    /// Should be dropped
}; ///< Trailing after exception

/// Leading comment for MyInterface
interface MyInterface /** inline interface */ {
    ///< Trailing at start of interface
    /// Doc for method1
    void method1(); ///< Trailing for method1
    /// Doc for method2
    string method2(int32 param);
    /// Should be dropped inside interface
}; ///< Trailing after interface

/// Leading comment for MyBitmask
bitmask MyBitmask /** inline bitmask */ {
    ///< Trailing at start of bitmask
    /// Doc for BIT1
    BIT1, ///< Trailing for BIT1
    /// Doc for BIT2
    BIT2 = 0x02 ///< Trailing for BIT2
}; ///< Trailing after bitmask

/// Leading comment for MyBitset
bitset MyBitset /** inline bitset */ {
    ///< Trailing at start of bitset
    /// Doc for field1
    bitfield<1> field1; ///< Trailing for field1
    /// Doc for field2  
    bitfield<3> field2;
    /// Should be dropped
}; ///< Trailing after bitset

/// Leading comment for MyValueType
valuetype MyValueType /** inline valuetype */ {
    ///< Trailing at start of valuetype
    /// Doc for public member
    public string value; ///< Trailing for value
    /// Should be dropped
}; ///< Trailing after valuetype

/// Leading comment for MyAnnotation
@annotation MyAnnotation /** inline annotation */ {
    ///< Trailing at start of annotation  
    /// Doc for param1
    string param1; ///< Trailing for param1
    /// Doc for param2
    int32 param2 default 42;
}; ///< Trailing after annotation

/// Leading comment for MyModule
module MyModule /** inline module */ {
    ///< Trailing at start of module
    
    /// Doc for const
    const int32 MY_CONST = 100; ///< Trailing for const
    
    /// Doc for alias  
    typedef string MyAlias; ///< Trailing for alias
    
    /// Nested struct
    struct NestedStruct {
        boolean flag;
    }; ///< Trailing for nested
    
    /// Should be dropped at end
}; ///< Trailing after module

/// Final leading comment
const float PI = 3.14159; ///< Trailing for PI
///< Extra trailing at end";

    let file = vfs.embed(content);

    // Parse using from_file
    let args = ProcArgs::default();
    let parse_result = from_file(file, args, &mut vfs);
    assert!(
        parse_result.errors.is_empty(),
        "Parse errors: {:?}",
        parse_result.errors
    );

    // We should have at least 10 top-level items
    assert!(
        parse_result.tree.len() >= 10,
        "Expected at least 10 top-level items, got {}",
        parse_result.tree.len()
    );

    let mut item_idx = 0;

    // Test MyStruct
    if let Item::StructValue(s) = &parse_result.tree[item_idx] {
        assert_eq!(s.ident.name, "MyStruct");
        let docs = get_doc_strings(&s.annotations);
        // Verify MyStruct has expected doc comments
        assert_eq!(docs.len(), 4, "MyStruct should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyStruct".to_string()));
        assert!(docs.contains(&"inline comment".to_string()));
        assert!(docs.contains(&"Trailing at start of struct".to_string()));
        assert!(docs.contains(&"Trailing after struct".to_string()));

        // Check field1
        assert_eq!(s.members.len(), 1, "MyStruct should have 1 field");
        let field1_docs = get_doc_strings(&s.members[0].annotations);
        assert_eq!(field1_docs.len(), 2, "field1 should have 2 doc comments");
        assert!(field1_docs.contains(&"Doc for field1".to_string()));
        assert!(field1_docs.contains(&"Trailing for field1".to_string()));
    } else {
        panic!("Expected MyStruct at index {item_idx}");
    }
    item_idx += 1;

    // Test MyUnion
    if let Item::UnionValue(u) = &parse_result.tree[item_idx] {
        assert_eq!(u.ident.name, "MyUnion");
        let docs = get_doc_strings(&u.annotations);
        assert_eq!(docs.len(), 4, "MyUnion should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyUnion".to_string()));
        assert!(docs.contains(&"inline union".to_string()));
        assert!(docs.contains(&"Trailing at start of union".to_string()));
        assert!(docs.contains(&"Trailing after union".to_string()));

        // Check union fields
        assert_eq!(u.fields.len(), 2);
        let field1_docs = get_doc_strings(&u.fields[0].annotations);
        assert_eq!(
            field1_docs.len(),
            2,
            "Union case 1 should have 2 doc comments"
        );
        assert!(field1_docs.contains(&"Doc for case 1".to_string()));
        assert!(field1_docs.contains(&"Trailing for option1".to_string()));
    } else {
        panic!("Expected MyUnion at index {item_idx}");
    }
    item_idx += 1;

    // Test MyEnum
    if let Item::EnumValue(e) = &parse_result.tree[item_idx] {
        assert_eq!(e.ident.name, "MyEnum");
        let docs = get_doc_strings(&e.annotations);
        assert_eq!(docs.len(), 4, "MyEnum should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyEnum".to_string()));
        assert!(docs.contains(&"inline enum".to_string()));
        assert!(docs.contains(&"Trailing at start of enum".to_string()));
        assert!(docs.contains(&"Trailing after enum".to_string()));

        // Check enum values
        assert_eq!(e.fields.len(), 2);
        let value1_docs = get_doc_strings(&e.fields[0].annotations);
        assert_eq!(value1_docs.len(), 2, "VALUE1 should have 2 doc comments");
        assert!(value1_docs.contains(&"Doc for VALUE1".to_string()));
        assert!(value1_docs.contains(&"Trailing for VALUE1".to_string()));
    } else {
        panic!("Expected MyEnum at index {item_idx}");
    }
    item_idx += 1;

    // Test MyException
    if let Item::ExceptionValue(e) = &parse_result.tree[item_idx] {
        assert_eq!(e.ident.name, "MyException");
        let docs = get_doc_strings(&e.annotations);
        assert_eq!(docs.len(), 4, "MyException should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyException".to_string()));
        assert!(docs.contains(&"inline exception".to_string()));
        assert!(docs.contains(&"Trailing at start of exception".to_string()));
        assert!(docs.contains(&"Trailing after exception".to_string()));

        // Check exception fields
        assert_eq!(e.members.len(), 2);
        let msg_docs = get_doc_strings(&e.members[0].annotations);
        assert_eq!(
            msg_docs.len(),
            2,
            "message field should have 2 doc comments"
        );
        assert!(msg_docs.contains(&"Doc for message".to_string()));
        assert!(msg_docs.contains(&"Trailing for message".to_string()));
    } else {
        panic!("Expected MyException at index {item_idx}");
    }
    item_idx += 1;

    // Test MyInterface
    if let Item::InterfaceValue(i) = &parse_result.tree[item_idx] {
        assert_eq!(i.ident.name, "MyInterface");
        let docs = get_doc_strings(&i.annotations);
        assert_eq!(docs.len(), 4, "MyInterface should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyInterface".to_string()));
        assert!(docs.contains(&"inline interface".to_string()));
        assert!(docs.contains(&"Trailing at start of interface".to_string()));
        assert!(docs.contains(&"Trailing after interface".to_string()));

        // Interface members (Prototype and Attribute) don't have annotations field
        // so we can't check method comments
    } else {
        panic!("Expected MyInterface at index {item_idx}");
    }
    item_idx += 1;

    // Test MyBitmask
    if let Item::BitmaskValue(b) = &parse_result.tree[item_idx] {
        assert_eq!(b.ident.name, "MyBitmask");
        let docs = get_doc_strings(&b.annotations);
        assert_eq!(docs.len(), 4, "MyBitmask should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyBitmask".to_string()));
        assert!(docs.contains(&"inline bitmask".to_string()));
        assert!(docs.contains(&"Trailing at start of bitmask".to_string()));
        assert!(docs.contains(&"Trailing after bitmask".to_string()));

        // Check bits
        assert_eq!(b.bits.len(), 2);
        let bit1_docs = get_doc_strings(&b.bits[0].annotations);
        assert_eq!(bit1_docs.len(), 2, "BIT1 should have 2 doc comments");
        assert!(bit1_docs.contains(&"Doc for BIT1".to_string()));
        assert!(bit1_docs.contains(&"Trailing for BIT1".to_string()));
    } else {
        panic!("Expected MyBitmask at index {item_idx}");
    }
    item_idx += 1;

    // Test MyBitset
    if let Item::BitsetValue(b) = &parse_result.tree[item_idx] {
        assert_eq!(b.ident.name, "MyBitset");
        let docs = get_doc_strings(&b.annotations);
        assert_eq!(docs.len(), 4, "MyBitset should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyBitset".to_string()));
        assert!(docs.contains(&"inline bitset".to_string()));
        assert!(docs.contains(&"Trailing at start of bitset".to_string()));
        assert!(docs.contains(&"Trailing after bitset".to_string()));

        // Check fields
        assert_eq!(b.fields.len(), 2);
        let field1_docs = get_doc_strings(&b.fields[0].annotations);
        assert_eq!(field1_docs.len(), 2, "bitfield1 should have 2 doc comments");
        assert!(field1_docs.contains(&"Doc for field1".to_string()));
        assert!(field1_docs.contains(&"Trailing for field1".to_string()));
    } else {
        panic!("Expected MyBitset at index {item_idx}");
    }
    item_idx += 1;

    // Test MyValueType
    if let Item::ValuetypeValue(v) = &parse_result.tree[item_idx] {
        assert_eq!(v.ident.name, "MyValueType");
        let docs = get_doc_strings(&v.annotations);
        assert_eq!(docs.len(), 4, "MyValueType should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyValueType".to_string()));
        assert!(docs.contains(&"inline valuetype".to_string()));
        assert!(docs.contains(&"Trailing at start of valuetype".to_string()));
        assert!(docs.contains(&"Trailing after valuetype".to_string()));

        // ValueMember elements don't have annotations field
    } else {
        panic!("Expected MyValueType at index {item_idx}");
    }
    item_idx += 1;

    // Test MyAnnotation
    if let Item::AnnotationValue(a) = &parse_result.tree[item_idx] {
        assert_eq!(a.ident.name, "MyAnnotation");
        let docs = get_doc_strings(&a.annotations);
        assert_eq!(docs.len(), 4, "MyAnnotation should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyAnnotation".to_string()));
        assert!(docs.contains(&"inline annotation".to_string()));
        assert!(docs.contains(&"Trailing at start of annotation".to_string()));
        assert!(docs.contains(&"Trailing after annotation".to_string()));

        // Check annotation params
        assert_eq!(a.params.len(), 2);
        if let ic_syntax::AnnotationField::Member(m) = &a.params[0] {
            let param1_docs = get_doc_strings(&m.annotations);
            assert_eq!(param1_docs.len(), 2, "param1 should have 2 doc comments");
            assert!(param1_docs.contains(&"Doc for param1".to_string()));
            assert!(param1_docs.contains(&"Trailing for param1".to_string()));
        } else {
            panic!("Expected annotation member");
        }
    } else {
        panic!("Expected MyAnnotation at index {item_idx}");
    }
    item_idx += 1;

    // Test MyModule
    if let Item::ModuleValue(m) = &parse_result.tree[item_idx] {
        assert_eq!(m.ident.name, "MyModule");
        let docs = get_doc_strings(&m.annotations);
        assert_eq!(docs.len(), 4, "MyModule should have 4 doc comments");
        assert!(docs.contains(&"Leading comment for MyModule".to_string()));
        assert!(docs.contains(&"inline module".to_string()));
        assert!(docs.contains(&"Trailing at start of module".to_string()));
        assert!(docs.contains(&"Trailing after module".to_string()));

        // Check module contents
        assert_eq!(m.definitions.len(), 3, "Module should have 3 definitions");

        // Check const
        if let Item::ConstValue(c) = &m.definitions[0] {
            let const_docs = get_doc_strings(&c.annotations);
            assert_eq!(const_docs.len(), 2, "MY_CONST should have 2 doc comments");
            assert!(const_docs.contains(&"Doc for const".to_string()));
            assert!(const_docs.contains(&"Trailing for const".to_string()));
        } else {
            panic!("Expected const in module");
        }

        // Check alias
        if let Item::AliasValue(a) = &m.definitions[1] {
            let alias_docs = get_doc_strings(&a.annotations);
            assert_eq!(alias_docs.len(), 2, "MyAlias should have 2 doc comments");
            assert!(alias_docs.contains(&"Doc for alias".to_string()));
            assert!(alias_docs.contains(&"Trailing for alias".to_string()));
        } else {
            panic!("Expected alias in module");
        }

        // Check nested struct
        if let Item::StructValue(s) = &m.definitions[2] {
            let struct_docs = get_doc_strings(&s.annotations);
            assert_eq!(
                struct_docs.len(),
                2,
                "NestedStruct should have 2 doc comments"
            );
            assert!(struct_docs.contains(&"Nested struct".to_string()));
            assert!(struct_docs.contains(&"Trailing for nested".to_string()));
        } else {
            panic!("Expected struct in module");
        }
    } else {
        panic!("Expected MyModule at index {item_idx}");
    }
}

#[test]
fn test_comment_scoping() {
    let mut vfs = SourceMap::default();

    // Test that comments don't leak between items
    let content = r"
/// Comment for Struct1
struct Struct1 {
    int32 field;
}; ///< Trailing for Struct1

/// This should ONLY attach to Struct2  
struct Struct2 {
    string value;
};
";

    let file = vfs.embed(content);
    let args = ProcArgs::default();
    let parse_result = from_file(file, args, &mut vfs);
    assert!(parse_result.errors.is_empty());

    assert_eq!(parse_result.tree.len(), 2);

    // Check Struct1
    if let Item::StructValue(s1) = &parse_result.tree[0] {
        let docs = get_doc_strings(&s1.annotations);
        // Verify Struct1 has expected doc comments
        assert_eq!(
            docs.len(),
            2,
            "Struct1 should have 2 comments, got: {docs:?}"
        );
        assert!(docs.contains(&"Comment for Struct1".to_string()));
        assert!(docs.contains(&"Trailing for Struct1".to_string()));
    } else {
        panic!("Expected Struct1");
    }

    // Check Struct2 - should only have its own comment
    if let Item::StructValue(s2) = &parse_result.tree[1] {
        let docs = get_doc_strings(&s2.annotations);
        assert_eq!(docs.len(), 1);
        assert!(docs.contains(&"This should ONLY attach to Struct2".to_string()));
    }
}

#[test]
fn test_empty_body_comments() {
    let mut vfs = SourceMap::default();

    let content = r"
/// Empty struct with inline comment
struct Empty /** inline */ {
    ///< Trailing at start
}; ///< Trailing after

/// Empty enum
enum EmptyEnum /** inline enum */ {
    ///< Trailing at start of enum
}; ///< Trailing after enum
";

    let file = vfs.embed(content);
    let args = ProcArgs::default();
    let parse_result = from_file(file, args, &mut vfs);
    assert!(parse_result.errors.is_empty());

    // Check empty struct
    if let Item::StructValue(s) = &parse_result.tree[0] {
        let docs = get_doc_strings(&s.annotations);
        // Verify empty struct has expected doc comments
        assert_eq!(
            docs.len(),
            4,
            "Empty struct should have 4 comments, got: {docs:?}"
        );
        assert!(docs.contains(&"Empty struct with inline comment".to_string()));
        assert!(docs.contains(&"inline".to_string()));
        assert!(docs.contains(&"Trailing at start".to_string()));
        assert!(docs.contains(&"Trailing after".to_string()));
    } else {
        panic!("Expected empty struct");
    }

    // Check empty enum
    if let Item::EnumValue(e) = &parse_result.tree[1] {
        let docs = get_doc_strings(&e.annotations);
        assert_eq!(docs.len(), 4);
        assert!(docs.contains(&"Empty enum".to_string()));
        assert!(docs.contains(&"inline enum".to_string()));
        assert!(docs.contains(&"Trailing at start of enum".to_string()));
        assert!(docs.contains(&"Trailing after enum".to_string()));
    }
}
