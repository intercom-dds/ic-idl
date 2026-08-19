mod common;

use ic_hir::hir::DefId;
use ic_hir::member_id::{Autoid, effective_autoid, member_ids};

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
    assert_eq!(member_ids(&hir.context, choice), [239_892_167, 256_044_424]);

    let hash_id = def_id(&hir, "HashId");
    assert_eq!(member_ids(&hir.context, hash_id), [31_773_853, 31_773_854]);

    let derived = def_id(&hir, "Derived");
    assert_eq!(
        member_ids(&hir.context, derived),
        [236_336_729, 236_336_730]
    );
}
