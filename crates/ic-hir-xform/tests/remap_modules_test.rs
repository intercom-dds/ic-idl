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

mod common;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefFlags, DefId, DefKind};
use ic_hir_lower::AstInput;
use ic_hir_xform::normalize;
use ic_hir_xform::remap_modules::{self, ModuleMap};
use ic_vfs::SourceMap;

fn children(hir: &ResolvedGraph, id: DefId) -> &[DefId] {
    let DefKind::Module(module) = &hir.context.type_of(id).kind else {
        panic!("expected a module");
    };

    &module.definitions
}

#[test]
fn renamed_reopened_modules_keep_their_positions() {
    let hir = common::parse_and_resolve(
        r"
        module DDS {
            struct First {};
        };

        struct Between {};

        module DDS {
            struct Second {};
        };
        ",
    );

    let order = hir.order.clone();
    let transformed = remap_modules::transform(
        hir,
        &[ModuleMap {
            from: vec!["DDS".into()],
            to: vec!["Foo".into()],
        }],
    );

    normalize::normalize(&transformed);
    assert_eq!(transformed.order, order);
    assert_eq!(transformed.context.type_of(order[0]).ident.name, "Foo");
    assert_eq!(transformed.context.type_of(order[2]).ident.name, "Foo");
}

#[test]
fn destination_chains_preserve_each_files_modules_and_metadata() {
    let mut source_map = SourceMap::default();
    let first_file = source_map.embed_with_name(
        "first.idl",
        r"
        module DDS {
            struct First {};
        };
        ",
    );
    let second_file = source_map.embed_with_name(
        "second.idl",
        r"
        module DDS {
            struct Second {};
        };
        ",
    );

    let mut tree = Vec::new();
    for file in [first_file, second_file] {
        let parsed = ic_parse::from_file(file, &source_map);
        assert!(parsed.errors.is_empty());
        tree.extend(parsed.tree);
    }

    let mut hir = ic_hir_lower::from_ast(AstInput::User(tree));
    assert!(hir.errors.is_empty());

    let originals = hir.order.clone();
    hir.context.definitions.get_mut(originals[0]).flags |= DefFlags::IS_INCLUDED;

    let transformed = remap_modules::transform(
        hir,
        &[ModuleMap {
            from: vec!["DDS".into()],
            to: vec!["Outer".into(), "Inner".into(), "Foo".into()],
        }],
    );

    normalize::normalize(&transformed);
    assert_eq!(transformed.order.len(), 2);

    for (index, file) in [first_file, second_file].into_iter().enumerate() {
        let original = transformed.context.type_of(originals[index]);
        let expected_flags = DefFlags::IS_SYNTHESIZED | (original.flags & DefFlags::IS_INCLUDED);

        let mut id = transformed.order[index];
        for name in ["Outer", "Inner"] {
            let def = transformed.context.type_of(id);
            assert_eq!(def.ident.name, name);
            assert_eq!(def.span, original.span);
            assert_eq!(def.ident.span.start.file_id, file);
            assert_eq!(def.flags.bits(), expected_flags.bits());

            let contents = children(&transformed, id);
            assert_eq!(contents.len(), 1);

            id = contents[0];
        }

        assert_eq!(id, originals[index]);
        assert_eq!(original.ident.name, "Foo");
        assert_eq!(children(&transformed, id).len(), 1);
    }
}

#[test]
fn destination_chains_keep_reopened_blocks_in_source_order() {
    let hir = common::parse_and_resolve(
        r"
        module DDS {
            struct First {};
        };

        struct Between {};

        module DDS {
            struct Second {};
        };
        ",
    );

    let originals = hir.order.clone();
    let transformed = remap_modules::transform(
        hir,
        &[ModuleMap {
            from: vec!["DDS".into()],
            to: vec!["Outer".into(), "Foo".into()],
        }],
    );

    normalize::normalize(&transformed);
    assert_eq!(transformed.order.len(), 3);
    assert_eq!(transformed.order[1], originals[1]);
    assert_eq!(
        children(&transformed, transformed.order[0]),
        &[originals[0]]
    );
    assert_eq!(
        children(&transformed, transformed.order[2]),
        &[originals[2]]
    );
}

#[test]
fn nested_reopened_modules_remap_separately_and_prune_empty_parents() {
    let hir = common::parse_and_resolve(
        r"
        module Old {
            module DDS {
                struct First {};
            };
        };

        struct Between {};

        module Old {
            module DDS {
                struct Second {};
            };
        };
        ",
    );
    let originals = hir.order.clone();
    let modules = [
        children(&hir, originals[0])[0],
        children(&hir, originals[2])[0],
    ];

    let transformed = remap_modules::transform(
        hir,
        &[ModuleMap {
            from: vec!["Old".into(), "DDS".into()],
            to: vec!["Outer".into(), "Foo".into()],
        }],
    );

    normalize::normalize(&transformed);
    assert_eq!(transformed.order.len(), 3);
    assert_eq!(transformed.order[1], originals[1]);
    assert_eq!(children(&transformed, transformed.order[0]), &[modules[0]]);
    assert_eq!(children(&transformed, transformed.order[2]), &[modules[1]]);
    assert_eq!(children(&transformed, originals[0]), []);
    assert_eq!(children(&transformed, originals[2]), []);
}
