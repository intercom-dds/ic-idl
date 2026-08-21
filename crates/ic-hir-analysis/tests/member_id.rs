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

#[path = "../../ic-hir/tests/common/mod.rs"]
mod common;

use ic_hir::hir::DefId;
use ic_hir_analysis::member_id::{Autoid, effective_autoid, member_ids};

fn def_id(hir: &ic_hir::ResolvedGraph, name: &str) -> DefId {
    hir.context
        .definitions
        .iter()
        .find_map(|(id, def)| (def.ident.name == name).then_some(id))
        .unwrap()
}

#[test]
fn assigns_effective_member_ids() {
    let hir = common::parse_and_resolve_successfully(
        r#"
        const string HASH_NAME = "custom_hash";
        const unsigned long MEMBER_ID = 42;

        @autoid(HASH)
        module hashed {
            struct Hashed {
                long x;
                @hashid(HASH_NAME) long y;
                @id(MEMBER_ID) long z;
            };

            @autoid(SEQUENTIAL)
            struct Sequential {
                long x;
                @id(42) long y;
                long z;
            };

            union Choice switch (long) {
                case 0: long member1;
                default: long member2;
            };
        };

        struct HashId {
            @hashid long x;
            long y;
        };

        @autoid(HASH)
        struct Base {
            long base;
        };

        struct Derived : Base {
            long child;
        };
        "#,
    );

    let hashed = def_id(&hir, "Hashed");
    assert_eq!(effective_autoid(&hir.context, hashed), Autoid::Hash);
    assert_eq!(
        member_ids(&hir.context, hashed),
        [31_773_853, 37_920_031, 42]
    );

    let sequential = def_id(&hir, "Sequential");
    assert_eq!(
        effective_autoid(&hir.context, sequential),
        Autoid::Sequential
    );
    assert_eq!(member_ids(&hir.context, sequential), [0, 42, 43]);

    let choice = def_id(&hir, "Choice");
    assert_eq!(effective_autoid(&hir.context, choice), Autoid::Hash);
    assert_eq!(
        member_ids(&hir.context, choice),
        [0, 239_892_167, 256_044_424]
    );

    let hash_id = def_id(&hir, "HashId");
    assert_eq!(member_ids(&hir.context, hash_id), [31_773_853, 31_773_854]);

    let derived = def_id(&hir, "Derived");
    assert_eq!(
        member_ids(&hir.context, derived),
        [236_336_729, 236_336_730]
    );
}
