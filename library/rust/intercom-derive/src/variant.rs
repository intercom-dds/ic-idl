// Copyright 2024 KONGSBERG
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

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{DeriveInput, Error, Ident, Result};

use crate::attr::{self, is_unique};
use crate::{Marshal, Unmarshal};

struct Variant {
    id: usize,
    name: String,
    member: Ident,
    non_serialized: bool,
}

impl Variant {
    fn new(id: usize, variant: &syn::Variant) -> Result<Self> {
        let attrs = attr::field(&variant.attrs)?;
        let id = attrs.id.unwrap_or(id);
        let name = attrs.rename.unwrap_or_else(|| {
            variant
                .ident
                .to_string()
                .trim_start_matches("r#")
                .to_owned()
        });

        Ok(Self {
            id,
            name,
            member: variant.ident.clone(),
            non_serialized: attrs.non_serialized.unwrap_or(false),
        })
    }
}

fn variants(input: &DeriveInput, data: &Punctuated<syn::Variant, Comma>) -> Result<Vec<Variant>> {
    let ident = &input.ident;
    let mut next_id = 0;
    let variants = data
        .into_iter()
        .map(|field| {
            let var = Variant::new(next_id, field)?;
            next_id = var.id + 1;
            Ok(var)
        })
        .collect::<Result<Vec<_>>>()?;

    // Check for duplicate member IDs
    if !is_unique(&variants, |v| v.id) {
        return Err(Error::new(
            Span::call_site(),
            format!("Enum `{ident}` has variants with duplicate IDs"),
        ));
    }

    // Ensure all serialized names are unique
    if !is_unique(&variants, |v| &v.name) {
        return Err(Error::new(
            Span::call_site(),
            format!("Enum `{ident}` has variants with duplicate names"),
        ));
    }

    let variants = variants.into_iter().filter(|v| !v.non_serialized).collect();
    Ok(variants)
}

impl ToTokens for Marshal<Variant> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let variant_id = self.0.id;
        let variant_name = &self.0.name;
        let variant_field = &self.0.member;

        let expanded = quote! {
            Self::#variant_field(ref v) => {
                ::intercom_cts::encode::UnionSerializer::encode_discriminant(
                    &mut state,
                    &#variant_id
                )?;
                ::intercom_cts::encode::UnionSerializer::encode_variant(
                    state,
                    #variant_id,
                    #variant_name,
                    v,
                )
            }
        };
        expanded.to_tokens(stream);
    }
}

pub fn marshal(
    input: &DeriveInput,
    data: &Punctuated<syn::Variant, Comma>,
) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let variants: Vec<_> = variants(input, data)?.into_iter().map(Marshal).collect();

    Ok(quote! {
        let mut state = ::intercom_cts::encode::Serializer::encode_union(
            archive,
            stringify!(#ident),
        )?;
        match self {
            #(#variants)*
        }
    })
}

impl ToTokens for Unmarshal<Variant> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let variant_id = self.0.id;
        let variant_name = &self.0.name;
        let variant_field = &self.0.member;

        let expanded = quote! {
            #variant_id => {
                let mut value = ::std::default::Default::default();
                ::intercom_cts::decode::UnionDeserializer::decode_variant(
                    state,
                    #variant_id,
                    #variant_name,
                    &mut value,
                )?;
                Self::#variant_field(value)
            }
        };
        expanded.to_tokens(stream);
    }
}

pub fn unmarshal(
    input: &DeriveInput,
    data: &Punctuated<syn::Variant, Comma>,
) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let variants: Vec<_> = variants(input, data)?.into_iter().map(Unmarshal).collect();

    Ok(quote! {
        let mut state = ::intercom_cts::decode::Deserializer::decode_union(
            archive,
            stringify!(#ident),
        )?;
        let mut disc = 0_usize;
        ::intercom_cts::decode::UnionDeserializer::decode_discriminant(
            &mut state,
            &mut disc,
        )?;
        *self = match disc {
            #(#variants)*
            _ => {
                return ::std::result::Result::Err(
                    <__D::Error as ::intercom_cts::error::Error>::custom(
                        "Unknown discriminant"
                    )
                );
            }
        };
        Ok(())
    })
}
