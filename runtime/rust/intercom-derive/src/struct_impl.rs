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

use crate::attrs::{FieldAttrs, TypeAttrs};
use crate::utils::assign_member_ids;

pub fn expand_marshal(attrs: &TypeAttrs, fields: &[FieldAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;

    // For newtype (transparent), delegate to inner type
    if attrs.is_newtype() {
        let inner_ty = &fields[0].ty;
        return quote! {
            impl ::intercom_cts::Marshal for #ident {
                fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: ::intercom_cts::encode::Serializer<'a>,
                {
                    <#inner_ty as ::intercom_cts::Marshal>::marshal(&self.0, ar)
                }
            }
        };
    }

    let serialized_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.non_serialized)
        .cloned()
        .collect();
    let member_ids = assign_member_ids(&serialized_fields);

    let encode_fields: Vec<_> = serialized_fields
        .iter()
        .zip(&member_ids)
        .enumerate()
        .map(|(idx, (field, _))| {
            let field_ident = field.ident.as_ref().expect("named field");
            let is_optional = field.is_option_type();

            if is_optional {
                quote! {
                    state.encode_optional(&MEMBER_INFO[#idx], &self.#field_ident)?;
                }
            } else {
                quote! {
                    state.encode_field(&MEMBER_INFO[#idx], &self.#field_ident)?;
                }
            }
        })
        .collect();

    quote! {
        impl ::intercom_cts::Marshal for #ident {
            fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::intercom_cts::encode::Serializer<'a>,
            {
                use ::intercom_cts::encode::StructSerializer as _;

                let mut state = ar.encode_struct(&TYPE_INFO)?;
                #(#encode_fields)*
                state.end()
            }
        }
    }
}

pub fn expand_unmarshal(attrs: &TypeAttrs, fields: &[FieldAttrs]) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;

    // For newtype (transparent), delegate to inner type
    if attrs.is_newtype() {
        let inner_ty = &fields[0].ty;
        return quote! {
            impl ::intercom_cts::Unmarshal for #ident {
                fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
                where
                    D: ::intercom_cts::decode::Deserializer<'a>,
                {
                    <#inner_ty as ::intercom_cts::Unmarshal>::unmarshal_mut(&mut self.0, ar)
                }
            }
        };
    }

    let serialized_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.non_serialized)
        .cloned()
        .collect();
    let member_ids = assign_member_ids(&serialized_fields);

    let decode_fields: Vec<_> = serialized_fields
        .iter()
        .zip(&member_ids)
        .enumerate()
        .map(|(idx, (field, _))| {
            let field_ident = field.ident.as_ref().expect("named field");

            quote! {
                state.decode_field(&MEMBER_INFO[#idx], &mut self.#field_ident)?;
            }
        })
        .collect();

    quote! {
        impl ::intercom_cts::Unmarshal for #ident {
            fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
            where
                D: ::intercom_cts::decode::Deserializer<'a>,
            {
                use ::intercom_cts::decode::StructDeserializer as _;

                let mut state = ar.decode_struct(&TYPE_INFO)?;
                #(#decode_fields)*
                state.end()?;
                Ok(())
            }
        }
    }
}
