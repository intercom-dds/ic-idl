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

use ic_alloc::md5;
use ic_hir::hir;
use ic_hir::hir::{PrimitiveTy, TyKind};
use ic_omgidl::types::xtypes::{
    CompleteTypeObject, EK_BOTH, EK_COMPLETE, EK_MINIMAL, Empty, EquivalenceKind, MinimalAliasBody,
    MinimalAliasHeader, MinimalAliasType, MinimalAnnotationHeader, MinimalAnnotationParameter,
    MinimalAnnotationType, MinimalArrayHeader, MinimalArrayType, MinimalBitfield, MinimalBitflag,
    MinimalBitmaskHeader, MinimalBitmaskType, MinimalBitsetHeader, MinimalBitsetType,
    MinimalCollectionElement, MinimalCollectionHeader, MinimalDiscriminatorMember,
    MinimalEnumeratedHeader, MinimalEnumeratedLiteral, MinimalEnumeratedType, MinimalExtendedType,
    MinimalMapType, MinimalMemberDetail, MinimalSequenceType, MinimalStructHeader,
    MinimalStructMember, MinimalStructType, MinimalTypeDetail, MinimalTypeObject,
    MinimalUnionHeader, MinimalUnionMember, MinimalUnionType, NameHash, PlainArrayLElemDefn,
    PlainArraySElemDefn, PlainMapLTypeDefn, PlainMapSTypeDefn, PlainSequenceLElemDefn,
    PlainSequenceSElemDefn, StronglyConnectedComponentId, TK_BOOLEAN, TK_INT8, TK_INT16, TK_INT32,
    TK_INT64, TK_NONE, TK_UINT8, TK_UINT16, TK_UINT32, TK_UINT64, TypeIdentifier, TypeKind,
    TypeObject, TypeObjectHashId,
};

pub fn name_hash(name: &str) -> NameHash {
    let digest = md5::digest(name.as_bytes());
    let mut hash = NameHash::default();
    let len = hash.len();
    hash.copy_from_slice(&digest[..len]);
    hash
}

pub fn format_type_id(id: &TypeIdentifier) -> String {
    match id {
        TypeIdentifier::EkComplete(hash) => {
            format!("EkComplete({})", hex_hash(hash))
        }
        TypeIdentifier::EkMinimal(hash) => {
            format!("EkMinimal({})", hex_hash(hash))
        }
        TypeIdentifier::ScComponentId(sc) => {
            let hash = match &sc.sc_component_id {
                TypeObjectHashId::EkComplete(h) => format!("Complete({})", hex_hash(h)),
                TypeObjectHashId::EkMinimal(h) => format!("Minimal({})", hex_hash(h)),
                TypeObjectHashId::Null => "Null".to_string(),
            };
            format!("SCC({}, {}/{})", hash, sc.scc_index, sc.scc_length)
        }
        other => format!("{other:?}"),
    }
}

fn hex_hash(hash: &[u8; 14]) -> String {
    use std::fmt::Write;
    hash.iter().fold(String::with_capacity(28), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

pub fn complete_to_minimal(complete: CompleteTypeObject) -> MinimalTypeObject {
    match complete {
        CompleteTypeObject::AliasType(cmp) => MinimalTypeObject::AliasType(MinimalAliasType {
            alias_flags: cmp.alias_flags,
            header: MinimalAliasHeader {},
            body: MinimalAliasBody {
                common: cmp.body.common,
            },
        }),
        CompleteTypeObject::AnnotationType(cmp) => {
            MinimalTypeObject::AnnotationType(MinimalAnnotationType {
                annotation_flag: cmp.annotation_flag,
                header: MinimalAnnotationHeader {},
                member_seq: cmp
                    .member_seq
                    .into_iter()
                    .map(|v| MinimalAnnotationParameter {
                        common: v.common,
                        name_hash: name_hash(&v.name),
                        default_value: v.default_value,
                    })
                    .collect(),
            })
        }
        CompleteTypeObject::StructType(cmp) => MinimalTypeObject::StructType(MinimalStructType {
            struct_flags: cmp.struct_flags,
            header: MinimalStructHeader {
                base_type: cmp.header.base_type,
                detail: MinimalTypeDetail {},
            },
            member_seq: cmp
                .member_seq
                .into_iter()
                .map(|v| MinimalStructMember {
                    common: v.common,
                    detail: MinimalMemberDetail {
                        name_hash: name_hash(&v.detail.name),
                    },
                })
                .collect(),
        }),
        CompleteTypeObject::UnionType(cmp) => MinimalTypeObject::UnionType(MinimalUnionType {
            union_flags: cmp.union_flags,
            header: MinimalUnionHeader {
                base_type: cmp.header.base_type,
                detail: MinimalTypeDetail {},
            },
            discriminator: MinimalDiscriminatorMember {
                common: cmp.discriminator.common,
            },
            member_seq: cmp
                .member_seq
                .into_iter()
                .map(|v| MinimalUnionMember {
                    common: v.common,
                    detail: MinimalMemberDetail {
                        name_hash: name_hash(&v.detail.name),
                    },
                })
                .collect(),
        }),
        CompleteTypeObject::BitsetType(cmp) => MinimalTypeObject::BitsetType(MinimalBitsetType {
            bitset_flags: cmp.bitset_flags,
            header: MinimalBitsetHeader {},
            field_seq: cmp
                .field_seq
                .into_iter()
                .map(|v| MinimalBitfield {
                    common: v.common,
                    name_hash: name_hash(&v.detail.name),
                })
                .collect(),
        }),
        CompleteTypeObject::SequenceType(cmp) => {
            MinimalTypeObject::SequenceType(MinimalSequenceType {
                collection_flag: cmp.collection_flag,
                header: MinimalCollectionHeader {
                    common: cmp.header.common,
                },
                element: MinimalCollectionElement {
                    common: cmp.element.common,
                },
            })
        }
        CompleteTypeObject::ArrayType(cmp) => MinimalTypeObject::ArrayType(MinimalArrayType {
            collection_flag: cmp.collection_flag,
            header: MinimalArrayHeader {
                common: cmp.header.common,
            },
            element: MinimalCollectionElement {
                common: cmp.element.common,
            },
        }),
        CompleteTypeObject::MapType(cmp) => MinimalTypeObject::MapType(MinimalMapType {
            collection_flag: cmp.collection_flag,
            header: MinimalCollectionHeader {
                common: cmp.header.common,
            },
            key: MinimalCollectionElement {
                common: cmp.key.common,
            },
            element: MinimalCollectionElement {
                common: cmp.element.common,
            },
        }),
        CompleteTypeObject::EnumeratedType(cmp) => {
            MinimalTypeObject::EnumeratedType(MinimalEnumeratedType {
                enum_flags: cmp.enum_flags,
                header: MinimalEnumeratedHeader {
                    common: cmp.header.common,
                },
                literal_seq: cmp
                    .literal_seq
                    .into_iter()
                    .map(|v| MinimalEnumeratedLiteral {
                        common: v.common,
                        detail: MinimalMemberDetail {
                            name_hash: name_hash(&v.detail.name),
                        },
                    })
                    .collect(),
            })
        }
        CompleteTypeObject::BitmaskType(cmp) => {
            MinimalTypeObject::BitmaskType(MinimalBitmaskType {
                bitmask_flags: cmp.bitmask_flags,
                header: MinimalBitmaskHeader {
                    common: cmp.header.common,
                },
                flag_seq: cmp
                    .flag_seq
                    .into_iter()
                    .map(|v| MinimalBitflag {
                        common: v.common,
                        detail: MinimalMemberDetail {
                            name_hash: name_hash(&v.detail.name),
                        },
                    })
                    .collect(),
            })
        }
        CompleteTypeObject::ExtendedType(_) => {
            MinimalTypeObject::ExtendedType(MinimalExtendedType {})
        }
    }
}

pub fn equivalence_kind(ident: &TypeIdentifier) -> EquivalenceKind {
    match ident {
        TypeIdentifier::SeqSdefn(PlainSequenceSElemDefn { header, .. })
        | TypeIdentifier::SeqLdefn(PlainSequenceLElemDefn { header, .. })
        | TypeIdentifier::ArraySdefn(PlainArraySElemDefn { header, .. })
        | TypeIdentifier::ArrayLdefn(PlainArrayLElemDefn { header, .. })
        | TypeIdentifier::MapSdefn(PlainMapSTypeDefn { header, .. })
        | TypeIdentifier::MapLdefn(PlainMapLTypeDefn { header, .. }) => header.equiv_kind,
        TypeIdentifier::EkComplete(_) => EK_COMPLETE,
        TypeIdentifier::EkMinimal(_) => EK_MINIMAL,
        _ => EK_BOTH,
    }
}

pub fn type_object_size(obj: &TypeObject) -> usize {
    intercom_cts::cdr2::to_le_bytes(obj)
        .expect("failed to serialize TypeObject")
        .len()
}

pub fn get_holder_type(ty: &hir::Ty) -> TypeKind {
    match &ty.kind {
        TyKind::Primitive(prim) => match prim {
            PrimitiveTy::Bool => TK_BOOLEAN,
            PrimitiveTy::Int8 => TK_INT8,
            PrimitiveTy::UInt8 => TK_UINT8,
            PrimitiveTy::Int16 => TK_INT16,
            PrimitiveTy::UInt16 => TK_UINT16,
            PrimitiveTy::Int32 => TK_INT32,
            PrimitiveTy::UInt32 => TK_UINT32,
            PrimitiveTy::Int64 => TK_INT64,
            PrimitiveTy::UInt64 => TK_UINT64,
            _ => TK_NONE,
        },
        _ => TK_NONE,
    }
}

pub fn equivalence_hash(type_obj: &TypeObject) -> TypeIdentifier {
    let serialized =
        intercom_cts::cdr2::to_le_bytes(type_obj).expect("failed to serialize TypeObject");
    let digest = md5::digest(&serialized);

    let mut hash = [0u8; 14];
    hash.copy_from_slice(&digest[0..14]);

    match type_obj {
        TypeObject::Complete(_) => TypeIdentifier::EkComplete(hash),
        TypeObject::Minimal(_) => TypeIdentifier::EkMinimal(hash),
        TypeObject::Null => TypeIdentifier::TkNone(Empty {}),
    }
}

pub fn update_contained_identifiers(
    identifier: &mut TypeIdentifier,
    old_to_new: &std::collections::BTreeMap<TypeIdentifier, TypeIdentifier>,
) {
    if let Some(new_id) = old_to_new.get(identifier) {
        *identifier = new_id.clone();
        return;
    }

    match identifier {
        TypeIdentifier::SeqSdefn(defn) => {
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        TypeIdentifier::SeqLdefn(defn) => {
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        TypeIdentifier::MapSdefn(defn) => {
            update_contained_identifiers(&mut defn.key_identifier, old_to_new);
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        TypeIdentifier::MapLdefn(defn) => {
            update_contained_identifiers(&mut defn.key_identifier, old_to_new);
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        TypeIdentifier::ArraySdefn(defn) => {
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        TypeIdentifier::ArrayLdefn(defn) => {
            update_contained_identifiers(&mut defn.element_identifier, old_to_new);
            defn.header.equiv_kind = equivalence_kind(&defn.element_identifier);
        }
        _ => {}
    }
}

#[allow(clippy::cast_possible_wrap)]
pub fn scc_equivalence_hash(
    type_objects: &[TypeObject],
    kind: EquivalenceKind,
) -> StronglyConnectedComponentId {
    let serialized =
        intercom_cts::cdr1::to_le_bytes(type_objects).expect("failed to serialize TypeObject");
    let digest = md5::digest(&serialized);

    let mut hash = [0u8; 14];
    hash.copy_from_slice(&digest[0..14]);

    let sc_component_id = match kind {
        EK_COMPLETE => TypeObjectHashId::EkComplete(hash),
        EK_MINIMAL => TypeObjectHashId::EkMinimal(hash),
        _ => TypeObjectHashId::Null,
    };

    StronglyConnectedComponentId {
        sc_component_id,
        scc_length: type_objects.len() as i32,
        scc_index: 0,
    }
}

pub fn update_type_object_identifiers(
    type_obj: &mut TypeObject,
    old_to_new: &std::collections::BTreeMap<TypeIdentifier, TypeIdentifier>,
) {
    if let TypeObject::Complete(complete) = type_obj {
        update_complete_type_object_identifiers(complete, old_to_new);
    }
}

fn update_complete_type_object_identifiers(
    complete: &mut CompleteTypeObject,
    old_to_new: &std::collections::BTreeMap<TypeIdentifier, TypeIdentifier>,
) {
    match complete {
        CompleteTypeObject::AliasType(alias) => {
            update_contained_identifiers(&mut alias.body.common.related_type, old_to_new);
        }
        CompleteTypeObject::AnnotationType(ann) => {
            for param in &mut ann.member_seq {
                update_contained_identifiers(&mut param.common.member_type_id, old_to_new);
            }
        }
        CompleteTypeObject::StructType(struct_ty) => {
            update_contained_identifiers(&mut struct_ty.header.base_type, old_to_new);
            for member in &mut struct_ty.member_seq {
                update_contained_identifiers(&mut member.common.member_type_id, old_to_new);
            }
        }
        CompleteTypeObject::UnionType(union) => {
            update_contained_identifiers(&mut union.header.base_type, old_to_new);
            update_contained_identifiers(&mut union.discriminator.common.type_id, old_to_new);
            for member in &mut union.member_seq {
                update_contained_identifiers(&mut member.common.type_id, old_to_new);
            }
        }
        CompleteTypeObject::SequenceType(seq) => {
            update_contained_identifiers(&mut seq.element.common.type_, old_to_new);
        }
        CompleteTypeObject::ArrayType(arr) => {
            update_contained_identifiers(&mut arr.element.common.type_, old_to_new);
        }
        CompleteTypeObject::MapType(map) => {
            update_contained_identifiers(&mut map.element.common.type_, old_to_new);
            update_contained_identifiers(&mut map.key.common.type_, old_to_new);
        }
        _ => {}
    }
}
