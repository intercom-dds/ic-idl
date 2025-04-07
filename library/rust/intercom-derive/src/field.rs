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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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
use syn::{DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed, Ident, Result};

use crate::attr::{self, is_unique};
use crate::{Marshal, Unmarshal};

#[derive(Clone)]
struct StructMember {
    id: usize,
    name: String,
    member: Ident,
    non_serialized: bool,
}

impl StructMember {
    fn new(id: usize, field: &syn::Field) -> Result<Self> {
        let attrs = attr::field(&field.attrs)?;
        let id = attrs.id.unwrap_or(id);
        let name = attrs.rename.unwrap_or_else(|| {
            field.ident.as_ref().map_or_else(
                || id.to_string(),
                |v| v.to_string().trim_start_matches("r#").to_owned(),
            )
        });

        Ok(Self {
            id,
            name,
            member: field.ident.clone().unwrap(),
            non_serialized: attrs.non_serialized.unwrap_or(false),
        })
    }

    fn from_iter<'a, I>(iter: I) -> Result<Vec<Self>>
    where
        I: IntoIterator<Item = &'a syn::Field>,
    {
        let mut next_id = 0;
        let mut variants = vec![];

        for var in iter {
            let var = Self::new(next_id, var)?;
            next_id = var.id + 1;
            variants.push(var);
        }
        Ok(variants)
    }
}

fn fields(input: &DeriveInput, data: &Fields) -> Result<Vec<StructMember>> {
    let ident = &input.ident;
    let fields: Vec<_> = match data {
        Fields::Named(FieldsNamed { named, .. }) => named.into_iter().collect(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed.into_iter().collect(),
        Fields::Unit => vec![],
    };
    let mut fields = StructMember::from_iter(fields)?;

    // Sort the members by their ID
    fields.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

    // Check for duplicate member IDs
    if !is_unique(&fields, |v| v.id) {
        return Err(Error::new(
            Span::call_site(),
            format!("Struct `{ident}` has members with duplicate IDs"),
        ));
    }

    // Ensure all serialized names are unique
    if !is_unique(&fields, |v| &v.name) {
        return Err(Error::new(
            Span::call_site(),
            format!("Struct `{ident}` has members with duplicate member names"),
        ));
    }

    // Don't serialize members annotated with non_serialized
    let fields = fields.into_iter().filter(|v| !v.non_serialized).collect();
    Ok(fields)
}

impl ToTokens for Marshal<StructMember> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let member_id = self.0.id;
        let member_name = &self.0.name;
        let member_field = &self.0.member;

        let expanded = quote! {
            ::intercom_cts::encode::FieldSerializer::encode_field(
                &mut state,
                #member_id,
                #member_name,
                &self.#member_field,
            )?;
        };
        expanded.to_tokens(stream);
    }
}

pub fn marshal(input: &DeriveInput, data: &Fields) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let fields: Vec<_> = fields(input, data)?.into_iter().map(Marshal).collect();

    Ok(quote! {
        let mut state = ::intercom_cts::encode::Serializer::encode_struct(
            archive,
            stringify!(#ident),
        )?;
        #(#fields)*
        ::intercom_cts::encode::FieldSerializer::end(state)
    })
}

impl ToTokens for Unmarshal<StructMember> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let member_id = self.0.id;
        let member_name = &self.0.name;
        let member_field = &self.0.member;

        let expanded = quote! {
            ::intercom_cts::decode::StructDeserializer::decode_field(
                &mut state,
                #member_id,
                #member_name,
                &mut self.#member_field,
            )?;
        };
        expanded.to_tokens(stream);
    }
}

pub fn unmarshal(input: &DeriveInput, data: &Fields) -> Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let fields: Vec<_> = fields(input, data)?.into_iter().map(Unmarshal).collect();

    Ok(quote! {
        let mut state = ::intercom_cts::decode::Deserializer::decode_struct(
            archive,
            stringify!(#ident),
        )?;
        #(#fields)*
        Ok(())
    })
}
