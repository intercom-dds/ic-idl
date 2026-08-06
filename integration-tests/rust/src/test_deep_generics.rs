// Copyright 2026 KONGSBERG
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

use std::collections::BTreeMap;

use crate::deep_generic_types;

#[test]
fn two_level_seq() {
    let matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let seq = deep_generic_types::TwoLevelSeq { matrix };
    assert_eq!(seq.matrix.len(), 2);
    assert_eq!(seq.matrix[0].len(), 3);
    assert_eq!(seq.matrix[0][0], 1);
    assert_eq!(seq.matrix[1][2], 6);
}

#[test]
fn three_level_seq() {
    let cube = vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]];
    let seq = deep_generic_types::ThreeLevelSeq { cube };
    assert_eq!(seq.cube.len(), 2);
    assert_eq!(seq.cube[0].len(), 2);
    assert_eq!(seq.cube[0][0].len(), 2);
    assert_eq!(seq.cube[0][0][0], 1);
    assert_eq!(seq.cube[1][1][1], 8);
}

#[test]
fn four_level_deep() {
    let hypercube = vec![vec![vec![vec![1, 2]]]];
    let deep = deep_generic_types::FourLevelDeep { hypercube };
    assert_eq!(deep.hypercube.len(), 1);
    assert_eq!(deep.hypercube[0][0][0][0], 1);
    assert_eq!(deep.hypercube[0][0][0][1], 2);
}

#[test]
fn map_of_seq() {
    let indexed_lists = BTreeMap::from([
        ("first".into(), vec![1, 2, 3]),
        ("second".into(), vec![4, 5]),
    ]);
    let seq = deep_generic_types::MapOfSeq { indexed_lists };

    assert_eq!(seq.indexed_lists.len(), 2);
    assert_eq!(seq.indexed_lists["first"].len(), 3);
    assert_eq!(seq.indexed_lists["first"][0], 1);
    assert_eq!(seq.indexed_lists["second"][1], 5);
}

#[test]
fn seq_of_map() {
    let list_of_dicts = vec![
        BTreeMap::from([("a".into(), 1), ("b".into(), 2)]),
        BTreeMap::from([("c".into(), 3)]),
    ];
    let seq = deep_generic_types::SeqOfMap { list_of_dicts };
    assert_eq!(seq.list_of_dicts.len(), 2);
    assert_eq!(seq.list_of_dicts[0]["a"], 1);
    assert_eq!(seq.list_of_dicts[1]["c"], 3);
}

#[test]
fn map_of_map() {
    let nested_dict = BTreeMap::from([("outer".into(), BTreeMap::from([("inner".into(), 42)]))]);
    let map = deep_generic_types::MapOfMap { nested_dict };
    assert_eq!(map.nested_dict.len(), 1);
    assert_eq!(map.nested_dict["outer"]["inner"], 42);
}

#[test]
fn map_seq_map() {
    let complex_structure = BTreeMap::from([(
        "key".into(),
        vec![BTreeMap::from([("a".into(), 1), ("b".into(), 2)])],
    )]);

    let seq = deep_generic_types::MapSeqMap { complex_structure };
    assert_eq!(seq.complex_structure.len(), 1);
    assert_eq!(seq.complex_structure["key"][0]["a"], 1);
}

#[test]
fn seq_map_seq() {
    let inverse_structure = vec![BTreeMap::from([("key".into(), vec![1, 2, 3])])];
    let seq = deep_generic_types::SeqMapSeq { inverse_structure };
    assert_eq!(seq.inverse_structure.len(), 1);
    assert_eq!(seq.inverse_structure[0]["key"][1], 2);
}

#[test]
fn point_struct() {
    let p = deep_generic_types::Point { x: 10, y: 20 };
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);
}

#[test]
fn seq_of_points() {
    let points = vec![
        deep_generic_types::Point { x: 1, y: 2 },
        deep_generic_types::Point { x: 3, y: 4 },
    ];
    let seq = deep_generic_types::SeqOfPoints { points };
    assert_eq!(seq.points.len(), 2);
    assert_eq!(seq.points[0].x, 1);
    assert_eq!(seq.points[1].y, 4);
}

#[test]
fn map_of_points() {
    let named_points = BTreeMap::from([
        ("origin".into(), deep_generic_types::Point { x: 0, y: 0 }),
        ("unit".into(), deep_generic_types::Point { x: 1, y: 1 }),
    ]);
    let map = deep_generic_types::MapOfPoints { named_points };

    assert_eq!(map.named_points.len(), 2);
    assert_eq!(map.named_points["origin"].x, 0);
    assert_eq!(map.named_points["unit"].y, 1);
}

#[test]
fn seq_of_seq_of_points() {
    let point_matrix = vec![
        vec![
            deep_generic_types::Point { x: 1, y: 2 },
            deep_generic_types::Point { x: 3, y: 4 },
        ],
        vec![deep_generic_types::Point { x: 5, y: 6 }],
    ];

    let seq = deep_generic_types::SeqOfSeqOfPoints { point_matrix };
    assert_eq!(seq.point_matrix.len(), 2);
    assert_eq!(seq.point_matrix[0].len(), 2);
    assert_eq!(seq.point_matrix[0][0].x, 1);
    assert_eq!(seq.point_matrix[1][0].y, 6);
}

#[test]
fn map_of_seq_of_points() {
    let point_lists = BTreeMap::from([
        (
            "path1".into(),
            vec![
                deep_generic_types::Point { x: 0, y: 0 },
                deep_generic_types::Point { x: 1, y: 1 },
            ],
        ),
        (
            "path2".into(),
            vec![deep_generic_types::Point { x: 2, y: 2 }],
        ),
    ]);

    let map = deep_generic_types::MapOfSeqOfPoints { point_lists };
    assert_eq!(map.point_lists.len(), 2);
    assert_eq!(map.point_lists["path1"].len(), 2);
    assert_eq!(map.point_lists["path1"][0].x, 0);
    assert_eq!(map.point_lists["path2"][0].y, 2);
}

#[test]
fn typedef_aliases_exist() {
    let list: deep_generic_types::IntList = vec![1, 2, 3];
    assert_eq!(list.len(), 3);

    let matrix: deep_generic_types::IntMatrix = vec![vec![1, 2], vec![3, 4]];
    assert_eq!(matrix.len(), 2);

    let named: deep_generic_types::NamedMatrices =
        BTreeMap::from([("mat1".into(), vec![vec![1, 2], vec![3, 4]])]);
    assert_eq!(named.len(), 1);
}

#[test]
fn using_typedef_chain() {
    let data: deep_generic_types::NamedMatrices = BTreeMap::from([
        ("matrix1".into(), vec![vec![1, 2, 3], vec![4, 5, 6]]),
        ("matrix2".into(), vec![vec![7, 8]]),
    ]);
    let chain = deep_generic_types::UsingTypedefChain { data };

    assert_eq!(chain.data.len(), 2);
    assert_eq!(chain.data["matrix1"].len(), 2);
    assert_eq!(chain.data["matrix1"][0][2], 3);
}

#[test]
fn array_of_seq() {
    let items: [Vec<i32>; 3] = [vec![1, 2], vec![3, 4, 5], vec![6]];
    let seq = deep_generic_types::ArrayOfSeq { items };
    assert_eq!(seq.items.len(), 3);
    assert_eq!(seq.items[0].len(), 2);
    assert_eq!(seq.items[1].len(), 3);
    assert_eq!(seq.items[0][0], 1);
    assert_eq!(seq.items[2][0], 6);
}

#[test]
fn three_ints_typedef() {
    let triple: deep_generic_types::ThreeInts = [1, 2, 3];
    assert_eq!(triple.len(), 3);
    assert_eq!(triple[0], 1);
    assert_eq!(triple[2], 3);
}

#[test]
fn seq_of_array() {
    let fixed_triples: Vec<deep_generic_types::ThreeInts> = vec![[1, 2, 3], [4, 5, 6]];
    let seq = deep_generic_types::SeqOfArray { fixed_triples };
    assert_eq!(seq.fixed_triples.len(), 2);
    assert_eq!(seq.fixed_triples[0][0], 1);
    assert_eq!(seq.fixed_triples[1][2], 6);
}

#[test]
fn map_of_array() {
    let named_triples: BTreeMap<String, deep_generic_types::ThreeInts> =
        BTreeMap::from([("rgb".into(), [255, 128, 64]), ("xyz".into(), [1, 2, 3])]);

    let map = deep_generic_types::MapOfArray { named_triples };
    assert_eq!(map.named_triples.len(), 2);
    assert_eq!(map.named_triples["rgb"][0], 255);
    assert_eq!(map.named_triples["xyz"][2], 3);
}

#[test]
fn empty_nested_structures() {
    let empty = vec![];
    let seq = deep_generic_types::TwoLevelSeq { matrix: empty };
    assert_eq!(seq.matrix.len(), 0);
}

#[test]
fn deeply_nested_point_lists() {
    let point_matrix = vec![
        vec![
            deep_generic_types::Point { x: 0, y: 0 },
            deep_generic_types::Point { x: 0, y: 1 },
        ],
        vec![
            deep_generic_types::Point { x: 1, y: 0 },
            deep_generic_types::Point { x: 1, y: 1 },
        ],
    ];

    let matrix = deep_generic_types::SeqOfSeqOfPoints { point_matrix };
    assert_eq!(matrix.point_matrix.len(), 2);
    assert_eq!(matrix.point_matrix[1][1].x, 1);
    assert_eq!(matrix.point_matrix[1][1].y, 1);
}
