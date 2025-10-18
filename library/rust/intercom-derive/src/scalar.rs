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
use syn::{DeriveInput, Error, ExprLit, Ident, Result, Variant};

use crate::Marshal;
use crate::attr::is_unique;

struct Scalar {
    value: u32,
    name: String,
    member: Ident,
}

impl Scalar {
    fn new(value: u32, variant: &Variant) -> Result<Self> {
        let attrs = crate::attr::field(&variant.attrs)?;
        let name = attrs.rename.unwrap_or_else(|| variant.ident.to_string());
        let value = variant.discriminant.as_ref().map_or_else(
            || Ok(value),
            |v| {
                if let syn::Expr::Lit(ExprLit {
                    lit: syn::Lit::Int(v),
                    ..
                }) = &v.1
                {
                    v.base10_parse()
                } else {
                    Err(Error::new(Span::call_site(), "Invalid enumerator value"))
                }
            },
        )?;

        Ok(Self {
            value,
            name,
            member: variant.ident.clone(),
        })
    }

    fn from_iter<'a, I>(iter: I) -> Result<Vec<Self>>
    where
        I: IntoIterator<Item = &'a Variant>,
    {
        let mut next_id = 0;
        let mut variants = vec![];

        for var in iter {
            let var = Self::new(next_id, var)?;
            next_id = var.value + 1;
            variants.push(var);
        }
        Ok(variants)
    }
}

impl ToTokens for Marshal<Scalar> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let variant_id = self.0.value;
        let variant_name = &self.0.name;
        let variant_field = &self.0.member;

        let expanded = quote! {
            Self::#variant_field => {
                ::intercom_cts::encode::EnumSerializer::encode_variant(
                    state,
                    #variant_name,
                    #variant_id,
                )
            }
        };
        expanded.to_tokens(stream);
    }
}

pub fn marshal(
    input: &DeriveInput,
    data: &Punctuated<Variant, Comma>,
) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let variants = Scalar::from_iter(data)?;

    // Check for duplicate values
    if !is_unique(&variants, |v| v.value) {
        return Err(Error::new(
            Span::call_site(),
            format!("Enum `{ident}` has enumerators with duplicate values"),
        ));
    }

    let variants: Vec<_> = variants.into_iter().map(Marshal).collect();
    Ok(quote! {
        let state = ::intercom_cts::encode::Serializer::encode_enum(
            archive,
            stringify!(#ident),
        )?;
        match *self {
            #(#variants)*
        }
    })
}

pub fn unmarshal(
    input: &DeriveInput,
    data: &Punctuated<Variant, Comma>,
) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let variants = Scalar::from_iter(data)?;

    let var_name = variants.iter().map(|v| &v.name);
    let var_value = variants.iter().map(|v| v.value);
    let var_member: Vec<_> = variants.iter().map(|v| &v.member).collect();
    Ok(quote! {
        impl ::intercom_cts::decode::EnumVisitor for #ident {
            fn member_id<D>(self, archive: D) -> Result<Self, D::Error>
            where
                D: ::intercom_cts::decode::Deserializer,
            {
                let disc = ::intercom_cts::decode::Deserializer::decode_u32(archive)?;
                let value = match disc {
                    #(
                        #var_value => #ident::#var_member,
                    )*
                    _ => {
                        return ::std::result::Result::Err(
                            <D::Error as ::intercom_cts::error::Error>::custom(
                                "Invalid enum value",
                            )
                        );
                    }
                };
                Ok(value)
            }

            fn member_field<D>(self, name: &str) -> Result<Self, D::Error>
            where
                D: ::intercom_cts::decode::Deserializer,
            {
                let value = match name {
                    #(
                        #var_name => #ident::#var_member,
                    )*
                    _ => {
                        return ::std::result::Result::Err(
                            <D::Error as ::intercom_cts::error::Error>::custom(
                                "Invalid enum value",
                            )
                        );
                    }
                };
                Ok(value)
            }
        }

        let state = ::intercom_cts::decode::Deserializer::decode_enum(
            archive,
            stringify!(#ident),
        )?;
        *self = ::intercom_cts::decode::EnumDeserializer::decode_enumerator(
            state,
            *self,
        )?;
        Ok(())
    })
}
