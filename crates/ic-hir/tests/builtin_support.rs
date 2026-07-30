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

use common::parse_with_custom_builtins;
use ic_hir::hir::DefFlags;

#[test]
fn test_builtin_types_not_in_output() {
    let builtin_src = r"
        struct BuiltinType {
            long value;
        };
    ";

    let user_src = r"
        struct UserType {
            BuiltinType data;
        };
    ";

    let (result, _, diagnostics) = parse_with_custom_builtins(builtin_src, user_src, false);

    assert!(result.errors.is_empty(), "Unexpected errors: {diagnostics}");

    // User type should be in order
    assert_eq!(result.order.len(), 1);
    let user_def = result.context.type_of(result.order[0]);
    assert_eq!(user_def.ident.name, "UserType");

    // Builtin type should be in builtin_order
    assert_eq!(result.builtin_order.len(), 1);
    let builtin_def = result.context.type_of(result.builtin_order[0]);
    assert_eq!(builtin_def.ident.name, "BuiltinType");
    assert!(builtin_def.flags.contains(DefFlags::IS_BUILTIN));
}

#[test]
fn test_builtin_types_included_in_output() {
    let builtin_src = r"
        struct BuiltinType {
            long value;
        };
    ";

    let user_src = r"
        struct UserType {
            BuiltinType data;
        };
    ";

    let (result, _, diagnostics) = parse_with_custom_builtins(builtin_src, user_src, true);

    assert!(result.errors.is_empty(), "Unexpected errors: {diagnostics}");

    // Both types should be in order, with builtin first
    assert_eq!(result.order.len(), 2);

    let first_def = result.context.type_of(result.order[0]);
    assert_eq!(first_def.ident.name, "BuiltinType");
    assert!(first_def.flags.contains(DefFlags::IS_BUILTIN));

    let second_def = result.context.type_of(result.order[1]);
    assert_eq!(second_def.ident.name, "UserType");
    assert!(!second_def.flags.contains(DefFlags::IS_BUILTIN));
}

#[test]
fn test_nested_builtin_definitions_are_marked_builtin() {
    let builtin_src = r"
        module ext {
            @annotation nested {};
            struct NestedType {
                long value;
            };
        };
    ";
    let user_src = "struct UserType {};";

    let (result, _, diagnostics) = parse_with_custom_builtins(builtin_src, user_src, false);

    assert!(result.errors.is_empty(), "Unexpected errors: {diagnostics}");

    for (_, def) in &result.context.definitions {
        if def.ident.name == "UserType" {
            assert!(!def.flags.contains(DefFlags::IS_BUILTIN));
        } else {
            assert!(
                def.flags.contains(DefFlags::IS_BUILTIN),
                "{} should be marked as built-in",
                def.ident.name
            );
        }
    }
}
