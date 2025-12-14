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
use ic_hir_xform::synthesize_collections;

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        2,
        "Should synthesize two array types for multidimensional array (inner and outer)"
    );

    let outer_alias_def = transformed
        .order
        .iter()
        .rfind(|&&def_id| {
            let def = transformed.context.definitions.get(def_id);
            if let DefKind::Alias(alias_ty) = &def.kind
                && let TyKind::Array { len, .. } = &alias_ty.ty.kind
            {
                return *len == 5;
            }
            false
        })
        .expect("Should have outer array alias with length 5");

    let outer_alias = transformed.context.definitions.get(*outer_alias_def);
    if let DefKind::Alias(alias_ty) = &outer_alias.kind
        && let TyKind::Array {
            ty: elem_ty, len, ..
        } = &alias_ty.ty.kind
    {
        assert_eq!(*len, 5, "Outer array should have length 5");
        assert!(
            matches!(elem_ty.kind, TyKind::Adt(_)),
            "Element should reference the inner array typedef, but got: {:?}",
            elem_ty.kind
        );
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
    let transformed = synthesize_collections::transform(hir);

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
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize array for typedef used in attributes"
    );
}

fn count_sequence_aliases(hir: &ResolvedGraph) -> usize {
    hir.order
        .iter()
        .filter(|&&def_id| {
            let def = hir.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Sequence { .. }))
        })
        .count()
}

fn count_map_aliases(hir: &ResolvedGraph) -> usize {
    hir.order
        .iter()
        .filter(|&&def_id| {
            let def = hir.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Map { .. }))
        })
        .count()
}

#[test]
fn test_basic_sequence_synthesis() {
    let source = r"
        struct Test {
            sequence<long> numbers;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one sequence alias"
    );
}

#[test]
fn test_bounded_sequence_synthesis() {
    let source = r"
        struct Test {
            sequence<long, 100> numbers;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one bounded sequence alias"
    );
}

#[test]
fn test_sequence_reuse() {
    let source = r"
        struct Test {
            sequence<long> a;
            sequence<long> b;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should reuse the same sequence alias"
    );
}

#[test]
fn test_different_sequence_bounds_different_aliases() {
    let source = r"
        struct Test {
            sequence<long, 10> a;
            sequence<long, 20> b;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        2,
        "Different bounds should create different aliases"
    );
}

#[test]
fn test_bounded_unbounded_sequences_different_aliases() {
    let source = r"
        struct Test {
            sequence<long> unbounded;
            sequence<long, 10> bounded;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        2,
        "Bounded and unbounded sequences should be different"
    );
}

#[test]
fn test_different_sequence_element_types() {
    let source = r"
        struct Test {
            sequence<long> longs;
            sequence<short> shorts;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        2,
        "Different element types should create different aliases"
    );
}

#[test]
fn test_nested_sequences() {
    let source = r"
        struct Test {
            sequence<sequence<long>> nested;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        2,
        "Should synthesize both inner and outer sequence aliases"
    );
}

#[test]
fn test_basic_map_synthesis() {
    let source = r"
        struct Test {
            map<string, long> data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one map alias"
    );
}

#[test]
fn test_bounded_map_synthesis() {
    let source = r"
        struct Test {
            map<string, long, 100> data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one bounded map alias"
    );
}

#[test]
fn test_map_reuse() {
    let source = r"
        struct Test {
            map<string, long> a;
            map<string, long> b;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should reuse the same map alias"
    );
}

#[test]
fn test_different_map_key_types() {
    let source = r"
        struct Test {
            map<string, long> string_keys;
            map<long, long> long_keys;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        2,
        "Different key types should create different aliases"
    );
}

#[test]
fn test_different_map_value_types() {
    let source = r"
        struct Test {
            map<string, long> long_values;
            map<string, string> string_values;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        2,
        "Different value types should create different aliases"
    );
}

#[test]
fn test_different_map_bounds() {
    let source = r"
        struct Test {
            map<string, long, 10> a;
            map<string, long, 20> b;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        2,
        "Different bounds should create different aliases"
    );
}

#[test]
fn test_bounded_unbounded_maps_different_aliases() {
    let source = r"
        struct Test {
            map<string, long> unbounded;
            map<string, long, 10> bounded;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        2,
        "Bounded and unbounded maps should be different"
    );
}

#[test]
fn test_nested_maps() {
    let source = r"
        struct Test {
            map<string, map<string, long>> nested;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_map_aliases(&transformed),
        2,
        "Should synthesize both inner and outer map aliases"
    );
}

#[test]
fn test_mixed_collections() {
    let source = r"
        struct Test {
            sequence<long> seq;
            map<string, long> data;
            long array[10];
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one sequence alias"
    );
    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one map alias"
    );
    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize one array alias"
    );
}

#[test]
fn test_sequence_of_arrays() {
    let source = r"
        typedef long LongArray[10];
        struct Test {
            sequence<LongArray> seq_of_arrays;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should have one array alias (user-defined)"
    );
    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one sequence alias"
    );
}

#[test]
fn test_map_with_array_values() {
    let source = r"
        typedef long LongArray[5];
        struct Test {
            map<string, LongArray> map_of_arrays;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should have one array alias (user-defined)"
    );
    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one map alias"
    );
}

#[test]
fn test_typedef_with_nested_collections() {
    let source = r"
        typedef long LongArray[10];
        typedef sequence<LongArray> SeqOfArrays;
        struct Test {
            SeqOfArrays data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should have array typedef"
    );
    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize sequence from typedef"
    );
}

#[test]
fn test_valuetype_operations() {
    let source = r"
        valuetype Test {
            sequence<long> get_data();
            void set_data(in sequence<long> data);
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize sequence from valuetype operations"
    );
}

#[test]
fn test_valuetype_attributes() {
    let source = r"
        valuetype Test {
            attribute sequence<string> names;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize sequence from valuetype attributes"
    );
}

#[test]
fn test_valuetype_members_operations_and_attributes() {
    let source = r"
        valuetype Test {
            public long data[10];
            sequence<string> get_names();
            attribute map<string, long> mapping;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_array_aliases(&transformed),
        1,
        "Should synthesize array from valuetype member"
    );
    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize sequence from valuetype operation"
    );
    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize map from valuetype attribute"
    );
}

#[test]
fn test_nested_collection_ordering() {
    let source = r"
        struct Test {
            map<string, sequence<long, 5>> data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one sequence"
    );
    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one map"
    );

    let seq_def_id = transformed
        .order
        .iter()
        .find(|&&id| {
            let def = transformed.context.definitions.get(id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Sequence { .. }))
        })
        .expect("Should have sequence typedef");

    let map_def_id = transformed
        .order
        .iter()
        .find(|&&id| {
            let def = transformed.context.definitions.get(id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Map { .. }))
        })
        .expect("Should have map typedef");

    let seq_pos = transformed
        .order
        .iter()
        .position(|&id| id == *seq_def_id)
        .unwrap();
    let map_pos = transformed
        .order
        .iter()
        .position(|&id| id == *map_def_id)
        .unwrap();

    assert!(
        seq_pos < map_pos,
        "Sequence typedef must come before map typedef that references it"
    );
}

#[test]
fn test_sequence_of_map_ordering() {
    let source = r"
        struct Test {
            sequence<map<string, long>> data;
        };
    ";

    let hir = common::parse_and_resolve(source);
    let transformed = synthesize_collections::transform(hir);

    assert_eq!(
        count_sequence_aliases(&transformed),
        1,
        "Should synthesize one sequence"
    );
    assert_eq!(
        count_map_aliases(&transformed),
        1,
        "Should synthesize one map"
    );

    let map_def_id = transformed
        .order
        .iter()
        .find(|&&id| {
            let def = transformed.context.definitions.get(id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Map { .. }))
        })
        .expect("Should have map typedef");

    let seq_def_id = transformed
        .order
        .iter()
        .find(|&&id| {
            let def = transformed.context.definitions.get(id);
            matches!(def.kind, DefKind::Alias(ref alias) if matches!(alias.ty.kind, TyKind::Sequence { .. }))
        })
        .expect("Should have sequence typedef");

    let map_pos = transformed
        .order
        .iter()
        .position(|&id| id == *map_def_id)
        .unwrap();
    let seq_pos = transformed
        .order
        .iter()
        .position(|&id| id == *seq_def_id)
        .unwrap();

    assert!(
        map_pos < seq_pos,
        "Map typedef must come before sequence typedef that references it (map at {map_pos}, seq \
         at {seq_pos})",
    );
}
