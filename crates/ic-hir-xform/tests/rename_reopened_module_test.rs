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

use ic_emit::case::Case;
use ic_hir::hir::DefKind;
use ic_hir_xform::{Target, rename};

#[test]
fn test_reopened_modules_keep_same_name() {
    let idl = r"
        module FooBar {
            struct Baz {};
        };
        
        module FooBar {
            struct Qux {};
        };
    ";

    let hir = common::parse_and_resolve(idl);

    // Apply rename transformation to convert to snake_case
    let target = Target {
        module: Some(Case::Snake),
        struct_type: Some(Case::Pascal),
        ..Default::default()
    };

    let transformed = rename::transform(hir, &target);

    // Both foo_bar modules should remain as foo_bar (not foo_bar and foo_bar_)
    let module_names: Vec<_> = transformed
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if matches!(def.kind, DefKind::Module(_)) {
                Some(def.ident.name.clone())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(module_names.len(), 2, "Should have two module instances");
    assert!(
        module_names.iter().all(|name| name == "foo_bar"),
        "Both modules should be named 'foo_bar', but got: {module_names:?}"
    );
}

#[test]
fn test_nested_reopened_modules() {
    let idl = r"
        module A {
            module B {
                struct Foo {};
            };
        };
        
        module A {
            module B {
                struct Bar {};
            };
        };
    ";

    let hir = common::parse_and_resolve(idl);

    // Apply rename transformation (no changes expected for PascalCase modules)
    let target = Target {
        module: Some(Case::Pascal),
        struct_type: Some(Case::Pascal),
        ..Default::default()
    };

    let transformed = rename::transform(hir, &target);

    // Check that all A modules are still named A
    let a_modules: Vec<_> = transformed
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if matches!(def.kind, DefKind::Module(_)) && def.ident.name == "A" {
                Some(def.ident.name.clone())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(a_modules.len(), 2, "Should have two A module instances");
    assert!(
        a_modules.iter().all(|name| name == "A"),
        "All A modules should remain named 'A', but got: {a_modules:?}"
    );

    // Check that all B modules are still named B
    let b_modules: Vec<_> = transformed
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if matches!(def.kind, DefKind::Module(_)) && def.ident.name == "B" {
                Some(def.ident.name.clone())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(b_modules.len(), 2, "Should have two B module instances");
    assert!(
        b_modules.iter().all(|name| name == "B"),
        "All B modules should remain named 'B', but got: {b_modules:?}"
    );
}

#[test]
fn test_module_with_similar_name_not_blocked() {
    let idl = r"
        module A {
            struct Foo {};
        };
        
        module A {
            struct Bar {};
        };
        
        module A__ {
            struct Baz {};
        };
    ";

    let hir = common::parse_and_resolve(idl);

    // Apply rename transformation to convert to snake_case
    let target = Target {
        module: Some(Case::Snake),
        struct_type: Some(Case::Pascal),
        ..Default::default()
    };

    let transformed = rename::transform(hir, &target);

    // Collect all module names
    let module_names: Vec<_> = transformed
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if matches!(def.kind, DefKind::Module(_)) {
                Some((def.ident.name.clone(), def.ident.name.clone()))
            } else {
                None
            }
        })
        .collect();

    // Debug: print what we got

    // Count occurrences
    let a_count = module_names.iter().filter(|(name, _)| name == "a").count();
    let a_underscore_count = module_names.iter().filter(|(name, _)| name == "a_").count();

    assert_eq!(
        a_count, 2,
        "Should have two 'a' modules (the reopened module A)"
    );
    assert_eq!(
        a_underscore_count, 1,
        "Should have one 'a_' module (from A__)"
    );
}
