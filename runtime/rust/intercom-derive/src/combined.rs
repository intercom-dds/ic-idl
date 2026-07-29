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
use syn::DeriveInput;

use crate::attrs::{TypeAttrs, TypeData};
use crate::utils::{TypeKind, determine_type_kind};
use crate::{enum_impl, struct_impl, type_descriptor, union_impl};

pub fn expand_all(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = TypeAttrs::from_derive_input(input)?;

    let (type_info_impl, marshal_impl, unmarshal_impl, extra_impl) = match &attrs.data {
        TypeData::Struct(fields) => {
            let type_info = type_descriptor::expand_struct_contents(&attrs, fields)?;
            let marshal = struct_impl::expand_marshal(&attrs, fields);
            let unmarshal = struct_impl::expand_unmarshal(&attrs, fields);
            (type_info, marshal, unmarshal, quote! {})
        }
        TypeData::Enum(variants) => {
            let kind = determine_type_kind(variants);
            match kind {
                TypeKind::Enum => {
                    let type_info = type_descriptor::expand_enum_contents(&attrs, variants)?;
                    let marshal = enum_impl::expand_marshal(&attrs, variants);
                    let unmarshal = enum_impl::expand_unmarshal(&attrs, variants);
                    let visitor = enum_impl::expand_enum_visitor(&attrs, variants);
                    (
                        type_info,
                        marshal,
                        quote! { #unmarshal #visitor },
                        quote! {},
                    )
                }
                TypeKind::Union => {
                    let type_info = type_descriptor::expand_union_contents(&attrs, variants)?;
                    let marshal = union_impl::expand_marshal(&attrs, variants);
                    let unmarshal = union_impl::expand_unmarshal(&attrs, variants);
                    let disc_method = union_impl::expand_disc_method(&attrs, variants);
                    (type_info, marshal, unmarshal, disc_method)
                }
            }
        }
    };

    let output = quote! {
        #extra_impl

        const _: () = {
            #type_info_impl
            #marshal_impl
            #unmarshal_impl
        };
    };

    Ok(output)
}
