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

use quote::quote;

use crate::attrs::{FieldAttrs, VariantAttrs};

pub enum TypeKind {
    Enum,
    Union,
}

pub fn determine_type_kind(variants: &[VariantAttrs]) -> TypeKind {
    if variants.iter().any(VariantAttrs::has_fields) {
        TypeKind::Union
    } else {
        TypeKind::Enum
    }
}

pub fn assign_member_ids(fields: &[FieldAttrs]) -> Vec<u32> {
    let mut next_id = 0u32;
    fields
        .iter()
        .map(|field| {
            if let Some(id) = field.id {
                next_id = id + 1;
                id
            } else {
                let id = next_id;
                next_id += 1;
                id
            }
        })
        .collect()
}

pub fn assign_variant_discriminants(variants: &[VariantAttrs]) -> Vec<proc_macro2::TokenStream> {
    let mut next_disc = 0i32;
    variants
        .iter()
        .map(|variant| {
            if let Some(disc) = &variant.disc {
                quote!(#disc)
            } else {
                let disc = next_disc;
                next_disc += 1;
                quote!(#disc)
            }
        })
        .collect()
}
