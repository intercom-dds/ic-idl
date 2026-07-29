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

use crate::attrs::{TypeAttrs, VariantAttrs};
use crate::utils::assign_variant_discriminants;

pub fn expand_disc_method(
    attrs: &TypeAttrs,
    variants: &[VariantAttrs],
) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let discriminants = assign_variant_discriminants(variants);

    let match_arms: Vec<_> = variants
        .iter()
        .zip(&discriminants)
        .map(|(variant, disc)| {
            let variant_ident = &variant.ident;

            quote! {
                Self::#variant_ident(_) => #disc,
            }
        })
        .collect();

    quote! {
        impl #ident {
            pub const fn disc(&self) -> i32 {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

pub fn expand_marshal(attrs: &TypeAttrs, variants: &[VariantAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;

    let match_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let variant_ident = &variant.ident;

            quote! {
                Self::#variant_ident(v) => state.encode_variant(&MEMBER_INFO[#idx], v),
            }
        })
        .collect();

    quote! {
        impl ::intercom_cts::Marshal for #ident {
            fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::intercom_cts::encode::Serializer<'a>,
            {
                use ::intercom_cts::encode::UnionSerializer as _;

                let mut state = ar.encode_union(&TYPE_INFO)?;
                state.encode_discriminant(&self.disc())?;
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

pub fn expand_unmarshal(attrs: &TypeAttrs, variants: &[VariantAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let discriminants = assign_variant_discriminants(variants);

    let match_arms: Vec<_> = variants
        .iter()
        .zip(&discriminants)
        .enumerate()
        .map(|(idx, (variant, disc))| {
            let variant_ident = &variant.ident;

            if let Some(field) = variant.fields.first() {
                let field_ty = &field.ty;

                quote! {
                    #disc => {
                        let mut value = <#field_ty>::default();
                        state.decode_variant(&MEMBER_INFO[#idx], &mut value)?;
                        Self::#variant_ident(value)
                    }
                }
            } else {
                syn::Error::new_spanned(
                    &variant.ident,
                    "union variants must have exactly one field",
                )
                .to_compile_error()
            }
        })
        .collect();

    let first_variant = &variants[0].ident;
    let first_field_ty = if let Some(field) = variants[0].fields.first() {
        &field.ty
    } else {
        return syn::Error::new_spanned(
            &variants[0].ident,
            "union variants must have exactly one field",
        )
        .to_compile_error();
    };

    quote! {
        impl ::intercom_cts::Unmarshal for #ident {
            fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
            where
                D: ::intercom_cts::decode::Deserializer<'a>,
            {
                use ::intercom_cts::decode::UnionDeserializer as _;

                let mut state = ar.decode_union(&TYPE_INFO)?;
                let mut disc = i32::default();
                state.decode_discriminant(&mut disc)?;
                *self = match disc {
                    #(#match_arms,)*
                    _ => {
                        let mut value = <#first_field_ty>::default();
                        state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                        Self::#first_variant(value)
                    }
                };
                Ok(())
            }
        }
    }
}
