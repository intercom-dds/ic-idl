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

#![allow(clippy::cast_possible_truncation)]

use quote::quote;

use crate::attrs::TypeAttrs;
use crate::utils::{assign_member_ids, assign_variant_discriminants};

pub fn expand_struct_contents(
    attrs: &TypeAttrs,
    fields: &[crate::attrs::FieldAttrs],
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = attrs.type_name();
    let type_flags = attrs.type_flags()?;
    Ok(expand_struct(attrs, &type_name, &type_flags, fields))
}

pub fn expand_enum_contents(
    attrs: &TypeAttrs,
    variants: &[crate::attrs::VariantAttrs],
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = attrs.type_name();
    let type_flags = attrs.type_flags()?;
    Ok(expand_enum(attrs, &type_name, &type_flags, variants))
}

pub fn expand_union_contents(
    attrs: &TypeAttrs,
    variants: &[crate::attrs::VariantAttrs],
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = attrs.type_name();
    let type_flags = attrs.type_flags()?;
    Ok(expand_union(attrs, &type_name, &type_flags, variants))
}

fn expand_struct(
    attrs: &TypeAttrs,
    type_name: &str,
    type_flags: &proc_macro2::TokenStream,
    fields: &[crate::attrs::FieldAttrs],
) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;

    // For newtype (transparent), delegate to inner type
    if attrs.is_newtype() {
        let inner_ty = &fields[0].ty;
        return quote! {
            impl ::intercom_cts::type_info::TypeDescriptor for #ident {
                const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> =
                    <#inner_ty as ::intercom_cts::type_info::TypeDescriptor>::TYPE_INFO;
                const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] =
                    <#inner_ty as ::intercom_cts::type_info::TypeDescriptor>::MEMBER_INFO;
            }
        };
    }

    let serialized_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.non_serialized)
        .cloned()
        .collect();
    let member_ids = assign_member_ids(&serialized_fields);

    let member_info_entries: Vec<_> = serialized_fields
        .iter()
        .zip(&member_ids)
        .map(|(field, &member_id)| {
            let field_name = field.field_name();
            let field_ty = &field.ty;
            let member_flags = field.member_flags();

            quote! {
                ::intercom_cts::MemberInfo {
                    name: #field_name,
                    member_id: #member_id,
                    flags: #member_flags,
                    type_info: ::intercom_cts::type_info::<#field_ty>(),
                }
            }
        })
        .collect();

    quote! {
        static TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
            name: #type_name,
            flags: #type_flags,
            kind: ::intercom_cts::TypeKind::Struct,
            key_info: None,
            element_info: None,
        };

        static MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
            #(#member_info_entries),*
        ];

        impl ::intercom_cts::type_info::TypeDescriptor for #ident {
            const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
            const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = MEMBER_INFO;
        }
    }
}

fn expand_enum(
    attrs: &TypeAttrs,
    type_name: &str,
    type_flags: &proc_macro2::TokenStream,
    variants: &[crate::attrs::VariantAttrs],
) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let discriminants = assign_variant_discriminants(variants);

    let member_info_entries: Vec<_> = variants
        .iter()
        .zip(&discriminants)
        .map(|(variant, disc)| {
            let variant_name = variant.variant_name();

            quote! {
                ::intercom_cts::MemberInfo {
                    name: #variant_name,
                    member_id: #disc as u32,
                    flags: ::intercom_cts::MemberFlag::nil(),
                    type_info: ::intercom_cts::type_info::<#ident>(),
                }
            }
        })
        .collect();

    quote! {
        const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
            name: #type_name,
            flags: #type_flags,
            kind: ::intercom_cts::TypeKind::Enum,
            key_info: None,
            element_info: Some(::intercom_cts::type_info::<i32>()),
        };

        const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
            #(#member_info_entries),*
        ];

        impl ::intercom_cts::type_info::TypeDescriptor for #ident {
            const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
            const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = MEMBER_INFO;
        }
    }
}

fn expand_union(
    attrs: &TypeAttrs,
    type_name: &str,
    type_flags: &proc_macro2::TokenStream,
    variants: &[crate::attrs::VariantAttrs],
) -> proc_macro2::TokenStream {
    let ident = &attrs.ident;
    let _discriminants = assign_variant_discriminants(variants);

    let member_info_entries: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let variant_name = variant.variant_name();
            let member_id = idx as u32 + 1;

            let field_ty = if let Some(field) = variant.fields.first() {
                &field.ty
            } else {
                return syn::Error::new_spanned(
                    &variant.ident,
                    "union variants must have exactly one field",
                )
                .to_compile_error();
            };

            quote! {
                ::intercom_cts::MemberInfo {
                    name: #variant_name,
                    member_id: #member_id,
                    flags: ::intercom_cts::MemberFlag::nil(),
                    type_info: ::intercom_cts::type_info::<#field_ty>(),
                }
            }
        })
        .collect();

    quote! {
        const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
            name: #type_name,
            flags: #type_flags,
            kind: ::intercom_cts::TypeKind::Union,
            key_info: None,
            element_info: None,
        };

        const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
            #(#member_info_entries),*
        ];

        impl ::intercom_cts::type_info::TypeDescriptor for #ident {
            const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
            const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = MEMBER_INFO;
        }
    }
}
