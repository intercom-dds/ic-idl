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
use syn::{Data, DeriveInput, Fields, Ident, Lit, Type, Variant};

#[derive(Debug, Clone)]
pub enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

impl Extensibility {
    pub fn as_flag_expr(&self) -> proc_macro2::TokenStream {
        match self {
            Extensibility::Final => quote!(::intercom_cts::TypeFlag::IS_FINAL),
            Extensibility::Appendable => quote!(::intercom_cts::TypeFlag::IS_APPENDABLE),
            Extensibility::Mutable => quote!(::intercom_cts::TypeFlag::IS_MUTABLE),
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "final" => Some(Extensibility::Final),
            "appendable" => Some(Extensibility::Appendable),
            "mutable" => Some(Extensibility::Mutable),
            _ => None,
        }
    }
}

pub struct TypeAttrs {
    pub ident: Ident,
    pub data: TypeData,
    pub name: Option<String>,
    pub extensibility: Option<Extensibility>,
    pub final_: bool,
    pub appendable_: bool,
    pub mutable_: bool,
    pub nested: bool,
}

pub enum TypeData {
    Struct(Vec<FieldAttrs>),
    Enum(Vec<VariantAttrs>),
}

impl TypeAttrs {
    pub fn from_derive_input(input: &DeriveInput) -> syn::Result<Self> {
        let mut name = None;
        let mut extensibility = None;
        let mut final_ = false;
        let mut appendable_ = false;
        let mut mutable_ = false;
        let mut nested = false;

        for attr in &input.attrs {
            if !attr.path().is_ident("cts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = s {
                        name = Some(lit_str.value());
                    }
                } else if meta.path.is_ident("extensibility") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = s {
                        extensibility = Extensibility::from_str(&lit_str.value());
                    }
                } else if meta.path.is_ident("final") || meta.path.is_ident("r#final") {
                    final_ = true;
                } else if meta.path.is_ident("appendable") {
                    appendable_ = true;
                } else if meta.path.is_ident("mutable") {
                    mutable_ = true;
                } else if meta.path.is_ident("nested") {
                    nested = true;
                }
                Ok(())
            })?;
        }

        let data = match &input.data {
            Data::Struct(data_struct) => {
                let fields = match &data_struct.fields {
                    Fields::Named(fields) => fields
                        .named
                        .iter()
                        .map(FieldAttrs::from_field)
                        .collect::<syn::Result<_>>()?,
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() != 1 {
                            return Err(syn::Error::new_spanned(
                                input,
                                "only newtype tuple structs (single field) are supported",
                            ));
                        }
                        fields
                            .unnamed
                            .iter()
                            .map(FieldAttrs::from_field)
                            .collect::<syn::Result<_>>()?
                    }
                    Fields::Unit => {
                        return Err(syn::Error::new_spanned(
                            input,
                            "unit structs are not supported",
                        ));
                    }
                };
                TypeData::Struct(fields)
            }
            Data::Enum(data_enum) => {
                let variants = data_enum
                    .variants
                    .iter()
                    .map(VariantAttrs::from_variant)
                    .collect::<syn::Result<_>>()?;
                TypeData::Enum(variants)
            }
            Data::Union(_) => {
                return Err(syn::Error::new_spanned(input, "unions are not supported"));
            }
        };

        Ok(TypeAttrs {
            ident: input.ident.clone(),
            data,
            name,
            extensibility,
            final_,
            appendable_,
            mutable_,
            nested,
        })
    }

    pub fn type_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.ident.to_string())
    }

    pub fn extensibility(&self) -> syn::Result<Extensibility> {
        if let Some(ext) = &self.extensibility {
            if self.final_ || self.appendable_ || self.mutable_ {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "cannot specify both `extensibility = \"...\"` and shorthand flags like \
                     `r#final`",
                ));
            }
            return Ok(ext.clone());
        }

        let count = [self.final_, self.appendable_, self.mutable_]
            .iter()
            .filter(|&&x| x)
            .count();

        if count > 1 {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "can only specify one of `r#final`, `appendable`, or `mutable`",
            ));
        }

        if self.final_ {
            Ok(Extensibility::Final)
        } else if self.mutable_ {
            Ok(Extensibility::Mutable)
        } else {
            Ok(Extensibility::Appendable)
        }
    }

    pub fn type_flags(&self) -> syn::Result<proc_macro2::TokenStream> {
        let ext = self.extensibility()?;
        let mut flag_expr = ext.as_flag_expr();

        if self.nested {
            flag_expr = quote!(#flag_expr.union(::intercom_cts::TypeFlag::IS_NESTED));
        }

        if self.is_keyed() {
            flag_expr = quote!(#flag_expr.union(::intercom_cts::TypeFlag::IS_KEYED));
        }

        Ok(flag_expr)
    }

    pub fn is_newtype(&self) -> bool {
        if let TypeData::Struct(fields) = &self.data {
            fields.len() == 1 && fields[0].ident.is_none()
        } else {
            false
        }
    }

    pub fn is_keyed(&self) -> bool {
        if let TypeData::Struct(fields) = &self.data {
            fields.iter().any(|f| f.key)
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct FieldAttrs {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub name: Option<String>,
    pub id: Option<u32>,
    pub key: bool,
    pub must_understand: bool,
    pub non_serialized: bool,
}

impl FieldAttrs {
    fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut name = None;
        let mut id = None;
        let mut key = false;
        let mut must_understand = false;
        let mut non_serialized = false;

        for attr in &field.attrs {
            if !attr.path().is_ident("cts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = s {
                        name = Some(lit_str.value());
                    }
                } else if meta.path.is_ident("id") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Int(lit_int) = lit {
                        id = Some(lit_int.base10_parse()?);
                    }
                } else if meta.path.is_ident("key") {
                    key = true;
                } else if meta.path.is_ident("must_understand") {
                    must_understand = true;
                } else if meta.path.is_ident("non_serialized") {
                    non_serialized = true;
                }
                Ok(())
            })?;
        }

        Ok(FieldAttrs {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            name,
            id,
            key,
            must_understand,
            non_serialized,
        })
    }

    pub fn field_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.ident
                .as_ref()
                .map_or_else(|| "unnamed".to_string(), std::string::ToString::to_string)
        })
    }

    pub fn is_option_type(&self) -> bool {
        if let Type::Path(type_path) = &self.ty
            && let Some(segment) = type_path.path.segments.last()
        {
            return segment.ident == "Option";
        }
        false
    }

    pub fn is_optional(&self) -> bool {
        self.is_option_type()
    }

    pub fn member_flags(&self) -> proc_macro2::TokenStream {
        let mut flag_expr = quote!(::intercom_cts::MemberFlag::nil());

        if self.key {
            flag_expr = quote!(#flag_expr.union(::intercom_cts::MemberFlag::IS_KEY));
        }

        if self.must_understand {
            flag_expr = quote!(#flag_expr.union(::intercom_cts::MemberFlag::IS_MUST_UNDERSTAND));
        }

        if self.is_optional() {
            flag_expr = quote!(#flag_expr.union(::intercom_cts::MemberFlag::IS_OPTIONAL));
        }

        flag_expr
    }
}

pub struct VariantAttrs {
    pub ident: Ident,
    pub fields: Vec<FieldAttrs>,
    pub name: Option<String>,
    pub disc: Option<syn::Expr>,
}

impl VariantAttrs {
    fn from_variant(variant: &Variant) -> syn::Result<Self> {
        let mut name = None;
        let mut disc = None;

        for attr in &variant.attrs {
            if !attr.path().is_ident("cts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = s {
                        name = Some(lit_str.value());
                    }
                } else if meta.path.is_ident("disc") {
                    let value = meta.value()?;
                    disc = Some(value.parse()?);
                }
                Ok(())
            })?;
        }

        let fields = match &variant.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(FieldAttrs::from_field)
                .collect::<syn::Result<_>>()?,
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(FieldAttrs::from_field)
                .collect::<syn::Result<_>>()?,
            Fields::Unit => Vec::new(),
        };

        Ok(VariantAttrs {
            ident: variant.ident.clone(),
            fields,
            name,
            disc,
        })
    }

    pub fn variant_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.ident.to_string())
    }

    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }
}
