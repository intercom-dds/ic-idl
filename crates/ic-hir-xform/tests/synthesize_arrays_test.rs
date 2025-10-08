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

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefKind, TyKind};
use ic_hir_xform::synthesize_arrays;

fn count_array_aliases(hir: &ResolvedGraph) -> usize {
    hir.order
        .iter()
        .filter(|&&def_id| {
            let def = hir.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Array { .. }))
        })
        .count()
}

fn count_inline_arrays(hir: &ResolvedGraph) -> usize {
    let mut count = 0;
    for &def_id in &hir.order {
        let def = hir.context.definitions.get(def_id);
        if let DefKind::Struct(struct_ty) = &def.kind {
            for member in &struct_ty.members {
                if matches!(member.ty.kind, TyKind::Array { .. }) {
                    count += 1;
                }
            }
        }
    }
    count
}

#[test]
fn test_basic_array_synthesis() {
    let source = r"
        struct Test {
            long a[10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize one array alias"
    );
    assert_eq!(
        count_inline_arrays(&transformed),
        0,
        "Should replace all inline arrays"
    );
}

#[test]
fn test_array_reuse() {
    let source = r"
        struct Test {
            long a[10];
            long b[10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should reuse the same array alias"
    );
}

#[test]
fn test_different_lengths_different_aliases() {
    let source = r"
        struct Test {
            long a[10];
            long b[20];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Different lengths should create different aliases"
    );
}

#[test]
fn test_different_types_different_aliases() {
    let source = r"
        struct Test {
            long a[10];
            short b[10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Different element types should create different aliases"
    );
}

#[test]
fn test_adt_collision_prevention() {
    let source = r"
        struct Foo { long x; };
        struct Bar { long y; };
        struct Test {
            Foo a[10];
            Bar b[10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Different ADT types should not collide"
    );
}

#[test]
fn test_string_types_no_collision() {
    let source = r"
        struct Test {
            string a[4];
            wstring b[4];
            string<10> c[4];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        3,
        "Different string types should not collide"
    );
}

#[test]
fn test_module_nesting() {
    let source = r"
        module M {
            struct Test {
                long a[10];
            };
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    let module_def_id = transformed
        .order
        .iter()
        .find(|&&id| {
            matches!(
                transformed.context.definitions.get(id).kind,
                DefKind::Module(_)
            )
        })
        .expect("Should have a module");

    let module_def = transformed.context.definitions.get(*module_def_id);
    if let DefKind::Module(module_ty) = &module_def.kind {
        let has_array_alias = module_ty.definitions.iter().any(|&child_id| {
            let def = transformed.context.definitions.get(child_id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Array { .. }))
        });
        assert!(has_array_alias, "Array alias should be inside the module");
    }
}

#[test]
fn test_typedef_array() {
    let source = r"
        typedef long LongArray[5];
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize array for typedef"
    );
}

#[test]
fn test_union_arrays() {
    let source = r"
        union Test switch(long) {
            case 1: long a[10];
            case 2: short b[5];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Should synthesize arrays in union variants"
    );
}

#[test]
fn test_exception_arrays() {
    let source = r"
        exception TestEx {
            long data[100];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize arrays in exceptions"
    );
}

#[test]
fn test_sequence_bound_no_collision() {
    let source = r"
        struct Test {
            sequence<long> a[5];
            sequence<long, 10> b[5];
            sequence<long, 20> c[5];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        3,
        "Different sequence bounds should not collide"
    );
}

#[test]
fn test_map_types_no_collision() {
    let source = r"
        struct Test {
            map<string, long> a[4];
            map<string, string> b[4];
            map<long, string> c[4];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        3,
        "Different map types should not collide"
    );
}

#[test]
fn test_multidimensional_arrays() {
    let source = r"
        struct Test {
            long matrix[5][10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize one array type for multidimensional array"
    );

    let alias_def = transformed
        .order
        .iter()
        .find(|&&def_id| {
            let def = transformed.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Array { .. }))
        })
        .expect("Should have an array alias");

    let alias = transformed.context.definitions.get(*alias_def);
    if let DefKind::Alias(alias_ty) = &alias.kind {
        if let TyKind::Array {
            ty: elem_ty, len, ..
        } = &alias_ty.ty.kind
        {
            assert_eq!(*len, 5, "Outer array should have length 5");
            assert!(
                matches!(elem_ty.kind, TyKind::Array { .. }),
                "Element should be an array"
            );
            if let TyKind::Array { len: inner_len, .. } = elem_ty.kind {
                assert_eq!(inner_len, 10, "Inner array should have length 10");
            }
        }
    }
}

#[test]
fn test_interface_operation_arrays() {
    let source = r"
        typedef long LongArray10[10];
        typedef long LongArray5[5];
        interface Test {
            LongArray10 operation(in LongArray5 param);
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Should synthesize arrays for typedef used in operations"
    );
}

#[test]
fn test_interface_attribute_arrays() {
    let source = r"
        typedef long LongArray20[20];
        interface Test {
            attribute LongArray20 data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_arrays::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize array for typedef used in attributes"
    );
}
