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

use std::collections::HashMap;

use ic_hir::hir::{Decl, DefKind};
use ic_hir_xform::flatten;

#[test]
fn flattens_reopened_modules_in_source_order() {
    let hir = common::parse_and_resolve(
        r"
        struct Before {};

        module alpha {
            struct First {};

            module beta {
                struct Second {};
            };

            struct Third {};
        };

        struct Middle {};

        module alpha {
            struct Fourth {};
        };

        struct After {};
        ",
    );

    let flattened = flatten::transform(hir, "_");
    let names: Vec<_> = flattened
        .hir
        .order
        .iter()
        .map(|&id| flattened.hir.context.type_of(id).ident.name.as_str())
        .collect();

    assert_eq!(
        names,
        [
            "Before",
            "alpha_First",
            "alpha_beta_Second",
            "alpha_Third",
            "Middle",
            "alpha_Fourth",
            "After",
        ]
    );
    assert!(
        flattened
            .hir
            .order
            .iter()
            .all(|&id| !matches!(flattened.hir.context.type_of(id).kind, DefKind::Module(_)))
    );
}

#[test]
fn declares_interfaces_and_valuetypes_before_their_nested_definitions() {
    let hir = common::parse_and_resolve(
        r"
        module example {
            interface Service {
                struct Request {};
            };

            valuetype Value {
                typedef Value Self;
                struct State {};
            };
        };
        ",
    );

    let flattened = flatten::transform(hir, "_");
    let definitions: Vec<_> = flattened
        .hir
        .order
        .iter()
        .map(|&id| flattened.hir.context.type_of(id))
        .collect();
    let names: Vec<_> = definitions
        .iter()
        .map(|def| def.ident.name.as_str())
        .collect();

    assert_eq!(
        names,
        [
            "example_Service",
            "example_Service_Request",
            "example_Service",
            "example_Value",
            "example_Value_Self",
            "example_Value_State",
            "example_Value",
        ]
    );
    assert!(matches!(
        definitions[0].kind,
        DefKind::Decl(Decl::Interface)
    ));
    assert!(matches!(definitions[2].kind, DefKind::Interface(_)));
    assert!(matches!(
        definitions[3].kind,
        DefKind::Decl(Decl::Valuetype)
    ));
    assert!(matches!(definitions[6].kind, DefKind::Valuetype(_)));
}

#[test]
fn clears_nested_definition_lists_after_flattening() {
    let hir = common::parse_and_resolve(
        r"
        module example {
            interface Service {
                struct Request {};
            };

            valuetype Value {
                struct State {};
            };
        };
        ",
    );

    let flattened = flatten::transform(hir, "_");

    for (_, def) in &flattened.hir.context.definitions {
        let definitions = match &def.kind {
            DefKind::Module(module) => Some(&module.definitions),
            DefKind::Interface(interface) => Some(&interface.definitions),
            DefKind::Valuetype(valuetype) => Some(&valuetype.definitions),
            _ => None,
        };

        if let Some(definitions) = definitions {
            assert!(
                definitions.is_empty(),
                "{} still has definitions",
                def.ident.name
            );
        }
    }
}

#[test]
fn preserves_parent_metadata() {
    let hir = common::parse_and_resolve(
        r"
        module outer {
            module inner {
                struct Item {};
            };

            interface Service {
                struct Request {};
            };
        };
        ",
    );
    let parents: HashMap<_, _> = hir
        .context
        .definitions
        .iter()
        .map(|(id, def)| (id, def.parent))
        .collect();

    let flattened = flatten::transform(hir, "_");

    for (id, parent) in parents {
        assert_eq!(flattened.hir.context.type_of(id).parent, parent);
    }
}
