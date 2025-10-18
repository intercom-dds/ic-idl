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

//! Provides derive macros for serialization and deserialization of Rust types.
//!
//! Refer to the [`intercom-cts`] crate for documentation.
//!
//! # Example
//!
//! ```rust,ignore
//! #[derive(Marshal, Unmarshal)]
//! struct Sheep {
//!     tag: usize,
//!     weight: usize,
//! }
//!
//! let data = Sheep { tag: 3, weight: 190 };
//! let json = intercom::cts::json::to_string(&data, false);
//! assert!(json.is_ok());
//! ````
//!
//! [`intercom-cts`]: ../intercom_cts/index.html

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, parse_macro_input};

mod attr;
mod field;
mod scalar;
mod variant;

struct Marshal<T>(T);

struct Unmarshal<T>(T);

#[proc_macro_derive(Marshal, attributes(cts))]
pub fn derive_marshal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    marshal(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(Unmarshal, attributes(cts))]
pub fn derive_unmarshal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    unmarshal(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn marshal(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let body = match &input.data {
        Data::Struct(v) => field::marshal(input, &v.fields),
        Data::Enum(v) if is_scalar(v) => scalar::marshal(input, &v.variants),
        Data::Enum(v) => variant::marshal(input, &v.variants),
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "Rust `union`s are not supported. Use `enum`s instead.",
        )),
    }?;

    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let wrapped = quote! {
        const _: () = {
            impl #impl_generics ::intercom_cts::Marshal for #ident #ty_generics #where_clause {
                #[allow(unused_variables)]
                fn marshal<__S>(&self, archive: __S) -> ::std::result::Result<__S::Ok, __S::Error>
                where
                    __S: ::intercom_cts::encode::Serializer,
                {
                    #body
                }
            }
        };
    };
    Ok(wrapped)
}

fn unmarshal(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let body = match &input.data {
        Data::Struct(v) => field::unmarshal(input, &v.fields),
        Data::Enum(v) if is_scalar(v) => scalar::unmarshal(input, &v.variants),
        Data::Enum(v) => variant::unmarshal(input, &v.variants),
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "Rust `union`s are not supported. Use `enum`s instead.",
        )),
    }?;

    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let wrapped = quote! {
        const _: () = {
            impl #impl_generics ::intercom_cts::Unmarshal for #ident #ty_generics #where_clause {
                #[allow(unused_variables)]
                fn unmarshal_mut<__D>(
                    &mut self,
                    archive: __D,
                ) -> ::std::result::Result<(), __D::Error>
                where
                    __D: ::intercom_cts::decode::Deserializer,
                {
                    #body
                }
            }
        };
    };
    Ok(wrapped)
}

fn is_scalar(data: &DataEnum) -> bool {
    !data.variants.iter().any(|v| !v.fields.is_empty())
}
