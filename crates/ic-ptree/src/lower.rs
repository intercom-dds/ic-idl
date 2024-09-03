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

#![allow(clippy::cast_possible_wrap, clippy::too_many_lines)]

use std::ffi::{self, CString};
use std::ptr;

use ic_syntax::{
    AnnotationField, DeclKind, Declarator, Expr, Field, InterfaceMember, Item, Label, LiteralValue,
    Op, OpKind, Param, ParamKind, Path, Type, UnionElement,
};

use crate::sys;

const BUILTIN_ANNOTATIONS: &str = include_str!("../idl/annotations.idl");

#[allow(unused_unsafe)]
static mut NUM_UNDEF: *const sys::numeric = unsafe { ptr::addr_of!(sys::num_undef) };

fn op_kind(op: Op) -> ffi::c_char {
    let c = match op.kind {
        OpKind::OpAdd => b'+',
        OpKind::OpSub => b'-',
        OpKind::OpMultiply => b'*',
        OpKind::OpDivide => b'/',
        OpKind::OpModulo => b'%',
        OpKind::OpLshift => b'<',
        OpKind::OpRshift => b'>',
        OpKind::OpOr => b'|',
        OpKind::OpXor => b'^',
        OpKind::OpAnd => b'&',
        OpKind::OpNot => b'~',
    };
    c as ffi::c_char
}

fn param_kind(kind: Option<ParamKind>) -> ffi::c_int {
    let c = match kind {
        Some(ParamKind::ParamOut) => sys::OPT_OUT,
        Some(ParamKind::ParamInout) => sys::OPT_INOUT,
        _ => sys::OPT_IN,
    };
    c as ffi::c_int
}

fn path_str(path: &Path) -> String {
    let mut str = if path.leading_colons.is_some() {
        vec!["::"]
    } else {
        vec![]
    };
    str.extend(path.segments.iter().map(|v| v.name.as_str()));
    str.join("")
}

fn path_or_null(state: *mut sys::parser_state, path: &Option<Path>) -> *mut sys::ptree {
    path.as_ref()
        .map_or(ptr::null_mut(), |p| lower_path(state, p))
}

fn create_ident(name: &str) -> CString {
    CString::new(name).unwrap()
}

unsafe fn create_decl(
    state: *mut sys::parser_state,
    name: &Declarator,
    _annotations: *mut sys::ptree,
) -> *mut sys::declarator {
    match name {
        Declarator::Simple(v) => {
            let ident = create_ident(&v.name);
            sys::create_decl(state, ident.as_ptr(), ptr::null_mut())
        }
        Declarator::Array(v) => {
            let ident = create_ident(&v.ident.name);
            let mut decl = sys::create_decl(state, ident.as_ptr(), ptr::null_mut());
            for bound in &v.bounds {
                let expr = lower_expr(state, bound);
                decl = sys::append_array_size(state, decl, expr);
            }
            decl
        }
    }
}

unsafe fn create_decl_list(
    state: *mut sys::parser_state,
    names: &[Declarator],
    annotations: *mut sys::ptree,
) -> *mut sys::declarator {
    let mut list = ptr::null_mut();
    for name in names {
        let decl = create_decl(state, name, annotations);
        list = sys::append_decl(list, decl);
    }
    list
}

unsafe fn lower_item_list(state: *mut sys::parser_state, items: &[Item]) -> *mut sys::ptree {
    // TODO: we need to add info about includes
    collect_with(state, sys::append_node, items, |v| unsafe {
        lower_item(state, v)
    })
}

unsafe fn lower_ty(state: *mut sys::parser_state, ty: &Type) -> *mut sys::ptree {
    match ty {
        Type::Any(_) => ptr::addr_of_mut!(sys::any_type),
        Type::Fixed(_) => ptr::addr_of_mut!(sys::fixed_type),
        Type::Sequence(v) => {
            let ty = lower_ty(state, &v.ty);
            let bound = v.bound.as_ref().map_or(NUM_UNDEF, |e| lower_expr(state, e));
            sys::create_sequence(state, ty, bound)
        }
        Type::String_(v) => match &v.bound {
            Some(bound) => {
                let bound = lower_expr(state, bound);
                if v.wide {
                    sys::create_wstring(state, bound)
                } else {
                    sys::create_string(state, bound)
                }
            }
            None if v.wide => ptr::addr_of_mut!(sys::unbounded_wstring_type),
            None => ptr::addr_of_mut!(sys::unbounded_string_type),
        },
        Type::Map(v) => {
            let key = lower_ty(state, &v.key);
            let elem = lower_ty(state, &v.value);
            let bound = v.bound.as_ref().map_or(NUM_UNDEF, |e| lower_expr(state, e));
            sys::create_map(state, key, elem, bound)
        }
        Type::Path(v) => lower_path(state, v),
    }
}

unsafe fn lower_expr(state: *mut sys::parser_state, num: &Expr) -> *const sys::numeric {
    match num {
        Expr::Literal(v) => match v.value.clone() {
            LiteralValue::Bool(v) => sys::create_bool(state, ffi::c_int::from(v.value)),
            LiteralValue::Int(v) => sys::create_i64(state, v as i64, 10),
            // LiteralValue::Int(v) => sys::create_double(v),
            LiteralValue::Char(v) => sys::create_char(state, v as ffi::c_char),
            LiteralValue::String_(v) => {
                let str = CString::new(v).unwrap();
                sys::create_str(state, str.as_ptr())
            }
            LiteralValue::Null => todo!(),
        },
        Expr::Path(v) => {
            let ident = create_ident(&path_str(v));
            sys::lookup_value(state, ident.as_ptr())
        }
        Expr::Unary(v) => {
            let op = op_kind(v.op);
            sys::expr_unary(state, op, lower_expr(state, &v.expr))
        }
        Expr::Binary(v) => {
            let op = op_kind(v.op);
            sys::expr_binary(
                state,
                op,
                lower_expr(state, &v.lhs),
                lower_expr(state, &v.rhs),
            )
        }
        Expr::InitList(v) => {
            let mut list = ptr::null_mut();
            for expr in &v.values {
                let declarator = expr.ident.as_ref().map_or(ptr::null_mut(), |ident| {
                    let ident = create_ident(&ident.name);
                    sys::create_decl(state, ident.as_ptr(), ptr::null_mut())
                });

                let val = sys::create_const_node(
                    state,
                    declarator,
                    ptr::null_mut(),
                    lower_expr(state, &expr.value),
                );
                list = sys::append_node(state, list, val);
            }
            sys::create_value_node(state, NUM_UNDEF, list)
        }
    }
}

fn lower_path(state: *mut sys::parser_state, path: &Path) -> *mut sys::ptree {
    unsafe {
        let ident = create_ident(&path_str(path));
        sys::lookup_type(state, ident.as_ptr())
    }
}

fn lower_field(state: *mut sys::parser_state, field: &Field) -> *mut sys::ptree {
    unsafe {
        let ty = lower_ty(state, &field.ty);
        let decls = create_decl_list(state, &field.names, ptr::null_mut());
        sys::create_member(state, decls, ty, ptr::null_mut())
    }
}

fn lower_interface_member(
    state: *mut sys::parser_state,
    member: &InterfaceMember,
) -> *mut sys::ptree {
    unsafe {
        let param = |param: &Param| {
            let ty = lower_ty(state, &param.ty);
            let kind = param_kind(param.kind);
            let decl = {
                let ident = create_ident(&param.ident.name);
                sys::create_decl(state, ident.as_ptr(), ptr::null_mut())
            };
            sys::create_param_dcl(state, decl, ty, kind)
        };

        match member {
            InterfaceMember::Attr(_) => todo!(),
            InterfaceMember::Proto(v) => {
                let ident = create_ident(&v.ident.name);
                let params = collect_with(state, sys::append_node, &v.params, param);
                sys::create_interface_op(
                    state,
                    ident.as_ptr(),
                    params,
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            }
            InterfaceMember::Item(v) => lower_item(state, v),
        }
    }
}

unsafe fn lower_item(state: *mut sys::parser_state, item: &Item) -> *mut sys::ptree {
    match item {
        Item::AnnotationValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_annotation_dcl_start(state, ident.as_ptr());
            let params = collect_with(state, sys::append_node, &v.params, |param| match param {
                AnnotationField::Item(v) => lower_item(state, v),
                AnnotationField::Member(v) => lower_field(state, v),
            });
            sys::create_annotation_finish(state, params)
        }
        Item::ModuleValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_module_start(state, ident.as_ptr());
            let members = lower_item_list(state, &v.definitions);
            sys::create_module_finish(state, members)
        }
        Item::StructValue(v) => {
            // TODO: parent
            let ident = create_ident(&v.ident.name);
            sys::create_struct_start(state, ident.as_ptr(), ptr::null_mut());
            let members = collect_with(state, sys::append_node, &v.members, |v| {
                lower_field(state, v)
            });
            sys::create_struct_finish(state, members)
        }
        Item::ExceptionValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_exception_start(state, ident.as_ptr());
            let members = collect_with(state, sys::append_node, &v.members, |v| {
                lower_field(state, v)
            });
            sys::create_exception_finish(state, members)
        }
        Item::EnumValue(v) => {
            let values = collect_with(state, sys::append_enum_node, &v.fields, |field| {
                let ident = create_ident(&field.ident.name);
                let expr = field
                    .value
                    .as_ref()
                    .map_or(NUM_UNDEF, |v| lower_expr(state, v));
                sys::create_enum_value(state, ident.as_ptr(), expr)
            });

            let ident = create_ident(&v.ident.name);
            sys::create_enum(state, ident.as_ptr(), values)
        }
        Item::BitmaskValue(v) => {
            let values = collect_with(state, sys::append_node, &v.bits, |bit| {
                let ident = create_ident(&bit.ident.name);
                let expr = bit
                    .value
                    .as_ref()
                    .map_or(NUM_UNDEF, |v| lower_expr(state, v));

                sys::create_bitmask_value(state, ident.as_ptr(), expr)
            });

            let ident = create_ident(&v.ident.name);
            sys::create_bitmask(state, ident.as_ptr(), values)
        }
        Item::ConstValue(v) => {
            let ty = lower_ty(state, &v.ty);
            let decl = create_decl(state, &v.decl, ptr::null_mut());
            let expr = lower_expr(state, &v.value);
            sys::create_const_node(state, decl, ty, expr)
        }
        Item::AliasValue(v) => {
            let ty = lower_ty(state, &v.ty);
            let decls = create_decl_list(state, &v.decl, ptr::null_mut());
            sys::create_type(state, decls, ty)
        }
        Item::DeclValue(v) => {
            let ident = create_ident(&v.ident.name);
            match v.kind {
                DeclKind::DeclStruct => sys::create_struct_dcl(state, ident.as_ptr()),
                DeclKind::DeclUnion => sys::create_union_dcl(state, ident.as_ptr()),
                DeclKind::DeclNative => sys::create_native_type(state, ident.as_ptr()),
                DeclKind::DeclInterface => sys::create_interface_dcl(state, ident.as_ptr(), 0),
                DeclKind::DeclValuetype => sys::create_valuetype_dcl(state, ident.as_ptr()),
            }
        }
        Item::BitsetValue(v) => {
            let bitfields = collect_with(state, sys::append_node, &v.fields, |field| {
                let ident = create_ident(&v.ident.name);
                let size = lower_expr(state, &field.size);
                let ty = field
                    .ty
                    .as_ref()
                    .map_or(ptr::null_mut(), |v| lower_ty(state, v));
                sys::create_bitfield(state, ident.as_ptr(), size, ty)
            });

            let parent = path_or_null(state, &v.parent);
            let ident = create_ident(&v.ident.name);
            sys::create_bitset(state, ident.as_ptr(), bitfields, parent)
        }
        Item::InterfaceValue(v) => {
            // TODO: parents
            let ident = create_ident(&v.ident.name);
            sys::create_interface_start(
                state,
                ident.as_ptr(),
                ptr::null_mut(),
                ffi::c_int::from(v.local.is_some()),
            );
            sys::create_interface_finish(
                state,
                collect_with(state, sys::append_node, &v.members, |v| {
                    lower_interface_member(state, v)
                }),
            )
        }
        Item::UnionValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_union_start(state, ident.as_ptr());

            let label = |state, label: &Label| match label {
                Label::Case(v) => {
                    let expr = lower_expr(state, v);
                    sys::create_case_label(state, expr)
                }
                Label::Default(_) => sys::create_default_case(state),
            };

            let members = collect_with(state, sys::append_node, &v.fields, |var| {
                let mem = match &var.field {
                    UnionElement::Member(v) => {
                        let ty = lower_ty(state, &v.ty);
                        let decl = create_decl(state, &v.decl, ptr::null_mut());
                        sys::create_member(state, decl, ty, ptr::null_mut())
                    }
                    UnionElement::Null(_) => sys::create_null_node(state),
                };

                let labels =
                    collect_with(state, sys::append_node, &var.labels, |v| label(state, v));
                sys::create_union_member(state, mem, labels, ptr::null_mut())
            });

            let ident = create_ident("_d");
            let decl = sys::create_decl(state, ident.as_ptr(), ptr::null_mut());
            let disc =
                sys::create_member(state, decl, lower_ty(state, &v.disc.ty), ptr::null_mut());
            sys::create_union_finish(state, disc, members)
        }
        Item::ValuetypeValue(v) => {
            let ident = create_ident(&v.ident.name);
            let parent = path_or_null(state, &v.inherits);
            let supports = path_or_null(state, &v.supports);
            sys::create_valuetype_start(state, ident.as_ptr(), parent, supports);
            // TODO: members
            sys::create_valuetype_finish(state, ptr::null_mut())
        }
    }
}

type Appender = unsafe extern "C" fn(
    *mut sys::parser_state,
    *mut sys::ptree,
    *mut sys::ptree,
) -> *mut sys::ptree;

unsafe fn collect_with<I, C, T>(
    state: *mut sys::parser_state,
    appender: Appender,
    iter: I,
    cb: C,
) -> *mut sys::ptree
where
    I: IntoIterator<Item = T>,
    C: Fn(T) -> *mut sys::ptree,
{
    let mut list = ptr::null_mut();
    unsafe {
        for elem in iter {
            let node = cb(elem);
            list = appender(state, list, node);
        }
    }
    list
}

unsafe fn inject_builtin(state: *mut sys::parser_state) {
    let builtin =
        ic_parse::from_str(BUILTIN_ANNOTATIONS).expect("failed to parse built-in annotations");

    // Discard the generated nodes -- we don't want to include the built-in
    // types in the tree. They just need to be registered in the symbol map with
    // their respective definitions.
    lower_item_list(state, &builtin.tree);
}

pub extern "C" fn callback(
    state: *mut sys::parser_state,
    ptr: *mut ffi::c_void,
) -> *mut sys::ptree {
    let ast = ptr.cast::<&[Item]>();
    unsafe {
        inject_builtin(state);
        lower_item_list(state, *ast)
    }
}
