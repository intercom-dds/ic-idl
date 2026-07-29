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

pub fn expand_marshal(attrs: &TypeAttrs, variants: &[VariantAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let discriminants = assign_variant_discriminants(variants);

    let match_arms: Vec<_> = variants
        .iter()
        .zip(&discriminants)
        .enumerate()
        .map(|(idx, (variant, disc))| {
            let variant_ident = &variant.ident;

            quote! {
                Self::#variant_ident => state.encode_variant::<i32>(&MEMBER_INFO[#idx], #disc),
            }
        })
        .collect();

    quote! {
        impl ::intercom_cts::Marshal for #ident {
            fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::intercom_cts::encode::Serializer<'a>,
            {
                use ::intercom_cts::encode::EnumSerializer as _;

                let state = ar.encode_enum(&TYPE_INFO)?;
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

pub fn expand_unmarshal(attrs: &TypeAttrs, _variants: &[VariantAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;

    quote! {
        impl ::intercom_cts::Unmarshal for #ident {
            fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
            where
                D: ::intercom_cts::decode::Deserializer<'a>,
            {
                use ::intercom_cts::decode::EnumDeserializer as _;

                let state = ar.decode_enum(&TYPE_INFO)?;
                *self = state.decode_enumerator(*self)?;
                Ok(())
            }
        }
    }
}

pub fn expand_enum_visitor(
    attrs: &TypeAttrs,
    variants: &[VariantAttrs],
) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let discriminants = assign_variant_discriminants(variants);

    let member_id_arms: Vec<_> = variants
        .iter()
        .zip(&discriminants)
        .map(|(variant, disc)| {
            let variant_ident = &variant.ident;

            quote! {
                #disc => Self::#variant_ident,
            }
        })
        .collect();

    let member_field_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = variant.variant_name();

            quote! {
                #variant_name => Self::#variant_ident,
            }
        })
        .collect();

    quote! {
        impl ::intercom_cts::decode::EnumVisitor for #ident {
            fn member_id<'a, D>(self, de: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::intercom_cts::decode::Deserializer<'a>,
            {
                use ::intercom_cts::error::Error as _;

                let value = match de.decode_i32()? {
                    #(#member_id_arms)*
                    _ => return Err(D::Error::custom("Invalid enum value")),
                };
                Ok(value)
            }

            fn member_field<'a, D>(self, name: &str) -> ::std::result::Result<Self, D::Error>
            where
                D: ::intercom_cts::decode::Deserializer<'a>,
            {
                use ::intercom_cts::error::Error as _;

                let value = match name {
                    #(#member_field_arms)*
                    _ => return Err(D::Error::custom("Invalid enum value")),
                };
                Ok(value)
            }
        }
    }
}
