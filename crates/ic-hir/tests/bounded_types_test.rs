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

#[test]
fn test_bounded_sequence() {
    let input = r"
        typedef sequence<long, 100> BoundedIntList;
        typedef sequence<long> UnboundedIntList;
        
        const long MAX_SIZE = 50;
        typedef sequence<string, MAX_SIZE> BoundedStrings;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Check that bounds were evaluated
    let bounded_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BoundedIntList")
        .expect("BoundedIntList not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &bounded_def.1.kind {
        if let ic_hir::hir::TyKind::Sequence { bound, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(100), "Expected bound of 100");
        } else {
            panic!("Expected sequence type, got: {:?}", alias_ty.ty.kind);
        }
    } else {
        panic!("Expected alias definition, got: {:?}", bounded_def.1.kind);
    }

    // Check computed bound
    let computed_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BoundedStrings")
        .expect("BoundedStrings not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &computed_def.1.kind {
        if let ic_hir::hir::TyKind::Sequence { bound, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(50), "Expected bound of 50");
        } else {
            panic!("Expected sequence type");
        }
    } else {
        panic!("Expected alias definition");
    }
}

#[test]
fn test_bounded_string() {
    let input = r"
        typedef string<128> ShortString;
        typedef wstring<1024> LongWideString;
        typedef string UnboundedString;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Check string bound
    let short_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ShortString")
        .expect("ShortString not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &short_def.1.kind {
        if let ic_hir::hir::TyKind::String { bound, wide, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(128), "Expected bound of 128");
            assert!(!wide, "Expected narrow string");
        } else {
            panic!("Expected string type");
        }
    } else {
        panic!("Expected alias definition");
    }

    // Check wide string bound
    let wide_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "LongWideString")
        .expect("LongWideString not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &wide_def.1.kind {
        if let ic_hir::hir::TyKind::String { bound, wide, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(1024), "Expected bound of 1024");
            assert!(wide, "Expected wide string");
        } else {
            panic!("Expected string type");
        }
    } else {
        panic!("Expected alias definition");
    }
}

#[test]
fn test_bounded_map() {
    let input = r"
        typedef map<string, long, 1000> BoundedMap;
        typedef map<string, long> UnboundedMap;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Check map bound
    let bounded_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BoundedMap")
        .expect("BoundedMap not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &bounded_def.1.kind {
        if let ic_hir::hir::TyKind::Map { bound, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(1000), "Expected bound of 1000");
        } else {
            panic!("Expected map type");
        }
    } else {
        panic!("Expected alias definition");
    }
}

#[test]
fn test_nested_bounded_types() {
    let input = r"
        typedef sequence<string<50>, 10> BoundedStringArray;
        typedef map<string<32>, sequence<long, 5>, 100> ComplexBoundedMap;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Check nested bounds in sequence
    let seq_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BoundedStringArray")
        .expect("BoundedStringArray not found");

    if let ic_hir::hir::DefKind::Alias(alias_ty) = &seq_def.1.kind {
        if let ic_hir::hir::TyKind::Sequence { ty, bound, .. } = &alias_ty.ty.kind {
            assert_eq!(*bound, Some(10), "Expected sequence bound of 10");

            // Check inner string bound
            if let ic_hir::hir::TyKind::String {
                bound: str_bound, ..
            } = &ty.kind
            {
                assert_eq!(*str_bound, Some(50), "Expected string bound of 50");
            } else {
                panic!("Expected string type inside sequence");
            }
        } else {
            panic!("Expected sequence type");
        }
    } else {
        panic!("Expected alias definition");
    }
}
