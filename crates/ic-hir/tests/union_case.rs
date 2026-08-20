mod common;

use ic_hir::hir::{DefId, DefKind, Numeric};
use ic_hir::union_case::{
    default_discriminator, default_union_case, union_case, unused_discriminator,
};

fn def_id(hir: &ic_hir::ResolvedGraph, name: &str) -> DefId {
    hir.context
        .definitions
        .iter()
        .find_map(|(id, def)| (def.ident.name == name).then_some(id))
        .unwrap()
}

fn union_ty<'a>(hir: &'a ic_hir::ResolvedGraph, name: &str) -> &'a ic_hir::hir::UnionTy {
    let def = hir.context.type_of(def_id(hir, name));
    let DefKind::Union(union_ty) = &def.kind else {
        panic!("{name} is not a union");
    };

    union_ty
}

#[test]
fn resolves_union_discriminator_cases() {
    let hir = common::parse_and_resolve_successfully(
        r#"
        typedef long Disc;

        union PrimitiveChoice switch (Disc) {
            case 1:
            case 0:
                long selected;
            default:
                boolean fallback;
        };

        enum Kind {
            FIRST,
            @default_literal SECOND,
            THIRD
        };

        union EnumChoice switch (Kind) {
            case SECOND:
                long second;
            case FIRST:
                long first;
            default:
                boolean fallback;
        };

        union BoolChoice switch (boolean) {
            case FALSE:
                long false_value;
            default:
                long true_value;
        };
        "#,
    );

    let primitive = union_ty(&hir, "PrimitiveChoice");
    assert_eq!(
        default_discriminator(&hir.context, primitive),
        Numeric::Int32(0)
    );
    let default_case = default_union_case(&hir.context, primitive);
    assert_eq!(default_case.variant.ident.name, "selected");
    assert_eq!(
        default_case
            .label
            .map(|label| hir.context.integer_value(&label.value)),
        Some(0)
    );

    let fallback_case = union_case(&hir.context, primitive, &Numeric::Int32(42)).unwrap();
    assert_eq!(fallback_case.variant.ident.name, "fallback");
    assert!(fallback_case.label.is_none());
    assert_eq!(
        unused_discriminator(&hir.context, primitive),
        Some(Numeric::Int32(2))
    );

    let enum_choice = union_ty(&hir, "EnumChoice");
    assert_eq!(
        hir.context
            .integer_value(&default_discriminator(&hir.context, enum_choice)),
        1
    );
    let default_case = default_union_case(&hir.context, enum_choice);
    assert_eq!(default_case.variant.ident.name, "second");
    assert_eq!(
        default_case
            .label
            .map(|label| hir.context.integer_value(&label.value)),
        Some(1)
    );
    assert_eq!(
        hir.context
            .integer_value(&unused_discriminator(&hir.context, enum_choice).unwrap()),
        2
    );

    let bool_choice = union_ty(&hir, "BoolChoice");
    let default_case = default_union_case(&hir.context, bool_choice);
    assert_eq!(default_case.variant.ident.name, "false_value");
    assert_eq!(
        default_case.label.map(|label| &label.value),
        Some(&Numeric::Bool(false))
    );
    assert_eq!(
        unused_discriminator(&hir.context, bool_choice),
        Some(Numeric::Bool(true))
    );
}
