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

use ic_hir::hir::{DefKind, PrimitiveTy, TyKind};
use ic_hir_xform::strip_typedefs;

#[test]
fn test_simple_typedef_to_primitive() {
    let idl = r"
        typedef long MyInt;
        
        struct Example {
            MyInt value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the member type is now primitive
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        assert!(
            matches!(
                struct_ty.members[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Member type should be long (Int32), got {:?}",
            struct_ty.members[0].ty.kind
        );
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_chained_typedefs() {
    let idl = r"
        typedef long MyInt;
        typedef MyInt AnotherInt;
        typedef AnotherInt YetAnotherInt;
        
        struct Example {
            YetAnotherInt value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the member type is now primitive
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        assert!(
            matches!(
                struct_ty.members[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Member type should be long (Int32) after resolving chain"
        );
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_typedef_to_struct() {
    let idl = r"
        struct Foo {
            long x;
        };
        
        typedef Foo Bar;
        
        struct Example {
            Bar value;
        };
    ";

    let hir = common::parse_and_resolve(idl);

    // Get the Foo struct's DefId before transformation
    let foo_id = hir
        .order
        .iter()
        .find(|&&id| hir.context.type_of(id).ident.name == "Foo")
        .copied()
        .expect("Foo should exist");

    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the Example struct and verify the member type is now Foo directly
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        assert!(
            matches!(struct_ty.members[0].ty.kind, TyKind::Adt(id) if id == foo_id),
            "Member type should be Adt(Foo), got {:?}",
            struct_ty.members[0].ty.kind
        );
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_typedef_in_sequence() {
    let idl = r"
        typedef long MyInt;
        
        struct Example {
            sequence<MyInt> values;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the sequence element type is now primitive
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        if let TyKind::Sequence { ty: elem_ty, .. } = &struct_ty.members[0].ty.kind {
            assert!(
                matches!(elem_ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Sequence element type should be long (Int32)"
            );
        } else {
            panic!("Member should be a sequence");
        }
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_typedef_in_array() {
    let idl = r"
        typedef long MyInt;
        
        struct Example {
            MyInt values[10];
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the array element type is now primitive
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        if let TyKind::Array {
            ty: elem_ty, len, ..
        } = &struct_ty.members[0].ty.kind
        {
            assert_eq!(*len, 10);
            assert!(
                matches!(elem_ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Array element type should be long (Int32)"
            );
        } else {
            panic!("Member should be an array");
        }
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_typedef_in_map() {
    let idl = r"
        typedef long MyKey;
        typedef string MyValue;
        
        struct Example {
            map<MyKey, MyValue> data;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the map key/value types are now resolved
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        if let TyKind::Map { key, elem, .. } = &struct_ty.members[0].ty.kind {
            assert!(
                matches!(key.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Map key type should be long (Int32)"
            );
            assert!(
                matches!(elem.kind, TyKind::String { .. }),
                "Map value type should be string"
            );
        } else {
            panic!("Member should be a map");
        }
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_typedef_in_module() {
    let idl = r"
        module A {
            typedef long MyInt;
            
            struct Example {
                MyInt value;
            };
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // Find the module
    let module = transformed
        .iter()
        .find(|def| def.ident.name == "A")
        .expect("Module A should exist");

    if let DefKind::Module(module_ty) = &module.kind {
        // The module should contain only the struct, not the typedef
        assert_eq!(
            module_ty.definitions.len(),
            1,
            "Module should have only 1 definition (the struct)"
        );

        let child_def = transformed.context.type_of(module_ty.definitions[0]);
        assert_eq!(child_def.ident.name, "Example");

        if let DefKind::Struct(struct_ty) = &child_def.kind {
            assert!(
                matches!(
                    struct_ty.members[0].ty.kind,
                    TyKind::Primitive(PrimitiveTy::Int32)
                ),
                "Member type should be long (Int32)"
            );
        }
    } else {
        panic!("A should be a module");
    }
}

#[test]
fn test_typedef_in_union() {
    let idl = r"
        typedef long MyInt;
        
        union Example switch (long) {
            case 1: MyInt value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the union and verify the variant type is now primitive
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example union should exist");

    if let DefKind::Union(union_ty) = &example.kind {
        assert_eq!(union_ty.variants.len(), 1);
        assert!(
            matches!(
                union_ty.variants[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Variant type should be long (Int32)"
        );
    } else {
        panic!("Example should be a union");
    }
}

#[test]
fn test_typedef_in_interface() {
    let idl = r"
        typedef long MyInt;
        
        interface Example {
            MyInt getValue();
            void setValue(in MyInt v);
            attribute MyInt attr;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the interface and verify the types are resolved
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example interface should exist");

    if let DefKind::Interface(interface_ty) = &example.kind {
        // Check return type of getValue
        assert!(
            matches!(
                interface_ty.prototypes[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Return type should be long (Int32)"
        );

        // Check parameter type of setValue
        assert!(
            matches!(
                interface_ty.prototypes[1].params[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Parameter type should be long (Int32)"
        );

        // Check attribute type
        assert!(
            matches!(
                interface_ty.attributes[0].ty.kind,
                TyKind::Primitive(PrimitiveTy::Int32)
            ),
            "Attribute type should be long (Int32)"
        );
    } else {
        panic!("Example should be an interface");
    }
}

#[test]
fn test_typedef_of_sequence() {
    let idl = r"
        typedef sequence<long> IntSeq;
        
        struct Example {
            IntSeq values;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the type is now an inline sequence
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        if let TyKind::Sequence { ty: elem_ty, .. } = &struct_ty.members[0].ty.kind {
            assert!(
                matches!(elem_ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Sequence element type should be long (Int32)"
            );
        } else {
            panic!(
                "Member should be a sequence, got {:?}",
                struct_ty.members[0].ty.kind
            );
        }
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_preserve_non_typedef_definitions() {
    let idl = r"
        typedef long MyInt;
        
        struct Foo {
            long x;
        };
        
        enum Bar {
            A,
            B
        };
        
        struct Example {
            MyInt value;
            Foo foo;
            Bar bar;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // Count definitions by type
    let struct_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Struct(_)))
        .count();
    let enum_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Enum(_)))
        .count();
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();

    assert_eq!(struct_count, 2, "Should have 2 structs (Foo, Example)");
    assert_eq!(enum_count, 1, "Should have 1 enum (Bar)");
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");
}

#[test]
fn test_nested_typedef_in_sequence_of_sequence() {
    let idl = r"
        typedef long MyInt;
        typedef sequence<MyInt> IntSeq;
        
        struct Example {
            sequence<IntSeq> matrix;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the struct and verify the nested types are resolved
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example struct should exist");

    if let DefKind::Struct(struct_ty) = &example.kind {
        assert_eq!(struct_ty.members.len(), 1);
        if let TyKind::Sequence { ty: outer_elem, .. } = &struct_ty.members[0].ty.kind {
            if let TyKind::Sequence { ty: inner_elem, .. } = &outer_elem.kind {
                assert!(
                    matches!(inner_elem.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                    "Inner sequence element type should be long (Int32)"
                );
            } else {
                panic!("Outer sequence element should be a sequence");
            }
        } else {
            panic!("Member should be a sequence");
        }
    } else {
        panic!("Example should be a struct");
    }
}

#[test]
fn test_exception_with_typedef() {
    let idl = r"
        typedef string ErrorMsg;
        
        exception MyError {
            ErrorMsg message;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the exception and verify the member type is resolved
    let exception = transformed
        .iter()
        .find(|def| def.ident.name == "MyError")
        .expect("MyError exception should exist");

    if let DefKind::Except(except_ty) = &exception.kind {
        assert_eq!(except_ty.members.len(), 1);
        assert!(
            matches!(except_ty.members[0].ty.kind, TyKind::String { .. }),
            "Member type should be string"
        );
    } else {
        panic!("MyError should be an exception");
    }
}

#[test]
fn test_union_discriminator_typedef() {
    let idl = r"
        typedef long MyDisc;
        
        union Example switch (MyDisc) {
            case 1: string value;
        };
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the union and verify the discriminator type is resolved
    let example = transformed
        .iter()
        .find(|def| def.ident.name == "Example")
        .expect("Example union should exist");

    if let DefKind::Union(union_ty) = &example.kind {
        assert!(
            matches!(union_ty.disc.ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
            "Discriminator type should be long (Int32)"
        );
    } else {
        panic!("Example should be a union");
    }
}

#[test]
fn test_const_with_typedef() {
    let idl = r"
        typedef long MyInt;
        
        const MyInt MY_CONST = 42;
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the constant and verify its type is now primitive
    let my_const = transformed
        .iter()
        .find(|def| def.ident.name == "MY_CONST")
        .expect("MY_CONST should exist");

    if let DefKind::Const(const_ty) = &my_const.kind {
        assert!(
            matches!(const_ty.ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
            "Const type should be long (Int32), got {:?}",
            const_ty.ty.kind
        );
    } else {
        panic!("MY_CONST should be a const");
    }
}

#[test]
fn test_const_array_with_typedef() {
    use ic_hir::hir::Numeric;

    let idl = r"
        typedef long MyInt;
        
        const MyInt MY_ARRAY[3] = {1, 2, 3};
    ";

    let hir = common::parse_and_resolve(idl);
    let transformed = strip_typedefs::transform(hir);

    // There should be no typedefs in the result
    let typedef_count = transformed
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Alias(_)))
        .count();
    assert_eq!(typedef_count, 0, "Should have no typedefs after stripping");

    // Find the constant and verify the array element type in the Numeric is resolved
    let my_array = transformed
        .iter()
        .find(|def| def.ident.name == "MY_ARRAY")
        .expect("MY_ARRAY should exist");

    if let DefKind::Const(const_ty) = &my_array.kind {
        // Check the declared type
        if let TyKind::Array { ty: elem_ty, .. } = &const_ty.ty.kind {
            assert!(
                matches!(elem_ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Array element type should be long (Int32)"
            );
        } else {
            panic!("Const type should be an array");
        }

        // Check the Numeric value's type
        if let Numeric::Array { ty, .. } = &const_ty.value {
            assert!(
                matches!(ty.kind, TyKind::Primitive(PrimitiveTy::Int32)),
                "Numeric array element type should be long (Int32), got {:?}",
                ty.kind
            );
        } else {
            panic!("Const value should be a Numeric::Array");
        }
    } else {
        panic!("MY_ARRAY should be a const");
    }
}
