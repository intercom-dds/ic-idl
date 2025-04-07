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

#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]

use std::ffi::{self, CString};
use std::ptr;

use ic_parse::SourceMap;
use ic_ptree::{ParseResult, sys};
use ic_syntax::{
    AnnotationAppl, AnnotationArg, AnnotationField, AnnotationMember, DeclKind, Declarator, Expr,
    Field, InterfaceMember, Item, Label, LiteralValue, Op, OpKind, Param, ParamKind, Path, Type,
    UnionElement,
};

use crate::common::{self, NUM_UNDEF, collect_with, create_ident, path_str};

fn op_kind(op: Op) -> ffi::c_char {
    let c = match op.kind {
        OpKind::Add => b'+',
        OpKind::Sub => b'-',
        OpKind::Multiply => b'*',
        OpKind::Divide => b'/',
        OpKind::Modulo => b'%',
        OpKind::Lshift => b'<',
        OpKind::Rshift => b'>',
        OpKind::Or => b'|',
        OpKind::Xor => b'^',
        OpKind::And => b'&',
        OpKind::Not => b'~',
    };
    c as ffi::c_char
}

fn param_kind(kind: Option<ParamKind>) -> ffi::c_int {
    match kind {
        Some(v) => common::param_kind(v),
        None => sys::OPT_IN as ffi::c_int,
    }
}

unsafe fn path_or_null(state: *mut sys::parser_state, path: Option<&Path>) -> *mut sys::ptree {
    path.as_ref()
        .map_or(ptr::null_mut(), |p| lower_path(state, p))
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

unsafe fn decl_path_list(state: *mut sys::parser_state, names: &[Path]) -> *mut sys::declarator {
    let mut list = ptr::null_mut();
    for name in names {
        let ident = create_ident(&path_str(name));
        let decl = sys::create_decl(state, ident.as_ptr(), ptr::null_mut());
        list = sys::append_decl(list, decl);
    }
    list
}

unsafe fn lower_item_list(state: *mut sys::parser_state, items: &[Item]) -> *mut sys::ptree {
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
        Type::String(v) => match &v.bound {
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
        Type::Path(v) => {
            // FIXME(idarcar): this is just a hack to make void work
            if v.leading_colons.is_none() && v.segments.len() == 1 && v.segments[0].name == "void" {
                return std::ptr::null_mut();
            }
            lower_path(state, v)
        }
    }
}

unsafe fn lower_expr(state: *mut sys::parser_state, num: &Expr) -> *const sys::numeric {
    match num {
        Expr::Literal(v) => match v.value.clone() {
            LiteralValue::Null => NUM_UNDEF,
            LiteralValue::Bool(v) => sys::create_bool(state, ffi::c_int::from(v)),
            LiteralValue::Int(v) => sys::create_i64(state, v as i64, 10),
            LiteralValue::Char(v) => sys::create_char(state, v as ffi::c_char),
            LiteralValue::Float(v) => sys::create_double(state, v),
            LiteralValue::String(v) => {
                let str = CString::new(v).unwrap();
                sys::create_str(state, str.as_ptr())
            }
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

unsafe fn lower_path(state: *mut sys::parser_state, path: &Path) -> *mut sys::ptree {
    let ident = create_ident(&path_str(path));
    sys::lookup_type(state, ident.as_ptr())
}

unsafe fn lower_field(state: *mut sys::parser_state, field: &Field) -> *mut sys::ptree {
    let ty = lower_ty(state, &field.ty);
    let annotations = lower_applied_annotations(state, &field.annotations);
    let decls = create_decl_list(state, &field.names, ptr::null_mut());
    sys::create_member(state, decls, ty, annotations)
}

unsafe fn lower_annotation_member(
    state: *mut sys::parser_state,
    member: &AnnotationMember,
) -> *mut sys::ptree {
    let ty = lower_ty(state, &member.ty);
    let decl = create_decl(state, &member.decl, ptr::null_mut());
    let default = member
        .default
        .as_ref()
        .map_or(NUM_UNDEF, |v| lower_expr(state, v));

    sys::create_annotation_member(state, decl, ty, default)
}

unsafe fn lower_interface_member(
    state: *mut sys::parser_state,
    member: &InterfaceMember,
) -> *mut sys::ptree {
    let param = |param: &Param| {
        let ty = lower_ty(state, &param.ty);
        let kind = param_kind(param.kind);
        let decl = create_decl(state, &param.decl, ptr::null_mut());
        sys::create_param_dcl(state, decl, ty, kind)
    };

    match member {
        InterfaceMember::Attr(v) => {
            let decl = create_decl_list(state, &v.decl, ptr::null_mut());
            let ty = lower_ty(state, &v.ty);
            let getraises = decl_path_list(state, &v.getraises);
            let setraises = decl_path_list(state, &v.setraises);
            sys::create_attribute(
                state,
                decl,
                ty,
                getraises,
                setraises,
                ffi::c_int::from(v.readonly.is_some()),
            )
        }
        InterfaceMember::Proto(v) => {
            let ident = create_ident(&v.ident.name);
            let params = collect_with(state, sys::append_node, &v.params, param);
            let ret_ty = lower_ty(state, &v.ret);
            let raises = decl_path_list(state, &v.raises);
            sys::create_interface_op(state, ident.as_ptr(), params, ret_ty, raises)
        }
        InterfaceMember::Item(v) => lower_item(state, v),
    }
}

unsafe fn lower_annotation_arg(
    state: *mut sys::parser_state,
    arg: &AnnotationArg,
) -> *mut sys::ptree {
    let name = arg.ident.as_ref().map(|v| create_ident(&v.name));
    let name = name.as_ref().map_or(ptr::null(), |v| v.as_ptr());
    let value = lower_expr(state, &arg.value);
    sys::create_annotation_param(state, name, value)
}

unsafe fn lower_applied_annotation(
    state: *mut sys::parser_state,
    annotation: &AnnotationAppl,
) -> *mut sys::ptree {
    let name = format!("@{}", path_str(&annotation.ident));
    let ident = create_ident(&name);
    sys::create_annotation_start(state, ident.as_ptr());
    let params = collect_with(state, sys::append_node, &annotation.args, |arg| {
        lower_annotation_arg(state, arg)
    });
    sys::create_annotation_finish(state, params)
}

unsafe fn lower_applied_annotations(
    state: *mut sys::parser_state,
    annotations: &[AnnotationAppl],
) -> *mut sys::ptree {
    collect_with(state, sys::append_node, annotations, |ann| {
        lower_applied_annotation(state, ann)
    })
}

unsafe fn annotate(
    state: *mut sys::parser_state,
    node: *mut sys::ptree,
    annotations: &[AnnotationAppl],
) -> *mut sys::ptree {
    let anns = lower_applied_annotations(state, annotations);
    sys::annotate(state, node, anns)
}

unsafe fn lower_item(state: *mut sys::parser_state, item: &Item) -> *mut sys::ptree {
    match item {
        Item::AnnotationValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_annotation_dcl_start(state, ident.as_ptr());
            let params = collect_with(state, sys::append_node, &v.params, |param| match param {
                AnnotationField::Item(v) => lower_item(state, v),
                AnnotationField::Member(v) => lower_annotation_member(state, v),
            });
            sys::create_annotation_dcl_finish(state, params)
        }
        Item::ModuleValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_module_start(state, ident.as_ptr());
            let members = lower_item_list(state, &v.definitions);
            let ty = sys::create_module_finish(state, members);
            annotate(state, ty, &v.annotations)
        }
        Item::StructValue(v) => {
            let parent = v
                .parent
                .as_ref()
                .map_or(ptr::null_mut(), |v| lower_path(state, v));

            let ident = create_ident(&v.ident.name);
            sys::create_struct_start(state, ident.as_ptr(), parent);
            let members = collect_with(state, sys::append_node, &v.members, |v| {
                lower_field(state, v)
            });

            let ty = sys::create_struct_finish(state, members);
            annotate(state, ty, &v.annotations)
        }
        Item::ExceptionValue(v) => {
            let ident = create_ident(&v.ident.name);
            sys::create_exception_start(state, ident.as_ptr());
            let members = collect_with(state, sys::append_node, &v.members, |v| {
                lower_field(state, v)
            });

            let ty = sys::create_exception_finish(state, members);
            annotate(state, ty, &v.annotations)
        }
        Item::EnumValue(v) => {
            let values = collect_with(state, sys::append_enum_node, &v.fields, |field| {
                let ident = create_ident(&field.ident.name);
                let expr = field
                    .value
                    .as_ref()
                    .map_or(NUM_UNDEF, |v| lower_expr(state, v));

                let val = sys::create_enum_value(state, ident.as_ptr(), expr);
                annotate(state, val, &field.annotations)
            });

            let ident = create_ident(&v.ident.name);
            let ty = sys::create_enum(state, ident.as_ptr(), values);
            annotate(state, ty, &v.annotations)
        }
        Item::BitmaskValue(v) => {
            let values = collect_with(state, sys::append_enum_node, &v.bits, |bit| {
                let ident = create_ident(&bit.ident.name);
                let expr = bit
                    .value
                    .as_ref()
                    .map_or(NUM_UNDEF, |v| lower_expr(state, v));

                sys::create_bitmask_value(state, ident.as_ptr(), expr)
            });

            let ident = create_ident(&v.ident.name);
            let ty = sys::create_bitmask(state, ident.as_ptr(), values);
            annotate(state, ty, &v.annotations)
        }
        Item::ConstValue(v) => {
            let ty = lower_ty(state, &v.ty);
            let decl = create_decl(state, &v.decl, ptr::null_mut());
            let expr = lower_expr(state, &v.value);
            let ty = sys::create_const_node(state, decl, ty, expr);
            annotate(state, ty, &v.annotations)
        }
        Item::AliasValue(v) => {
            let ty = lower_ty(state, &v.ty);
            let decls = create_decl_list(state, &v.decl, ptr::null_mut());
            let ty = sys::create_type(state, decls, ty);
            annotate(state, ty, &v.annotations)
        }
        Item::DeclValue(v) => {
            let ident = create_ident(&v.ident.name);
            match v.kind {
                DeclKind::Struct => sys::create_struct_dcl(state, ident.as_ptr()),
                DeclKind::Union => sys::create_union_dcl(state, ident.as_ptr()),
                DeclKind::Native => sys::create_native_type(state, ident.as_ptr()),
                DeclKind::Interface => sys::create_interface_dcl(state, ident.as_ptr(), 0),
                DeclKind::Valuetype => sys::create_valuetype_dcl(state, ident.as_ptr()),
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

            let parent = path_or_null(state, v.parent.as_ref());
            let ident = create_ident(&v.ident.name);
            let ty = sys::create_bitset(state, ident.as_ptr(), bitfields, parent);
            annotate(state, ty, &v.annotations)
        }
        Item::InterfaceValue(v) => {
            let parents = decl_path_list(state, &v.inherits);
            let ident = create_ident(&v.ident.name);
            sys::create_interface_start(
                state,
                ident.as_ptr(),
                parents,
                ffi::c_int::from(v.local.is_some()),
            );
            let ty = sys::create_interface_finish(
                state,
                collect_with(state, sys::append_node, &v.members, |v| {
                    lower_interface_member(state, v)
                }),
            );
            annotate(state, ty, &v.annotations)
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
                let annotations = lower_applied_annotations(state, &var.annotations);
                let labels =
                    collect_with(state, sys::append_node, &var.labels, |v| label(state, v));

                let mem = match &var.field {
                    UnionElement::Member(v) => {
                        let ty = lower_ty(state, &v.ty);
                        let decl = create_decl(state, &v.decl, ptr::null_mut());
                        sys::create_member(state, decl, ty, ptr::null_mut())
                    }
                    UnionElement::Null(_) => sys::create_null_node(state),
                };
                sys::create_union_member(state, mem, labels, annotations)
            });

            let disc = sys::create_union_discriminator(
                state,
                lower_ty(state, &v.disc.ty),
                ptr::null_mut(),
            );
            let ty = sys::create_union_finish(state, disc, members);
            annotate(state, ty, &v.annotations)
        }
        Item::ValuetypeValue(v) => {
            let ident = create_ident(&v.ident.name);
            let parent = path_or_null(state, v.inherits.as_ref());
            let supports = path_or_null(state, v.supports.as_ref());
            sys::create_valuetype_start(state, ident.as_ptr(), parent, supports);
            // TODO: members
            let ty = sys::create_valuetype_finish(state, ptr::null_mut());
            annotate(state, ty, &v.annotations)
        }
    }
}

pub unsafe fn inject_builtin(state: *mut sys::parser_state) {
    let builtin = common::parse_builtin();

    // Discard the generated nodes -- we don't want to include the built-in
    // types in the tree. They just need to be registered in the symbol map with
    // their respective definitions.
    let ident = create_ident("<built-in>");
    sys::create_include_start(state, ident.as_ptr(), 0);
    let list = lower_item_list(state, &builtin.tree);
    sys::create_include_finish(state, list);
    assert!(!list.is_null());
}

unsafe fn lower_ast(
    state: *mut sys::parser_state,
    ast: &[Item],
    vfs: &SourceMap,
) -> *mut sys::ptree {
    inject_builtin(state);

    collect_with(state, sys::append_node, ast, |item| {
        let span = ic_syntax::util::item_span(item);
        let defined_in = format!("{}", vfs.name(span.start.file_id).display());
        let include = create_ident(&defined_in);

        sys::create_include_start(state, include.as_ptr(), 0);
        let node = lower_item(state, item);
        sys::create_include_finish(state, node)
    })
}

pub unsafe fn lower(ast: &[Item], vfs: &SourceMap) -> ParseResult {
    let result = unsafe {
        let state = sys::ic_parser_create();
        let tree = lower_ast(state, ast, vfs);
        let inner = sys::ic_parser_result(state, tree);
        ParseResult::from_raw(inner)
    };

    if let Some(err) = result.diagnostics() {
        debug_assert!(false, "{err}");
    }
    result
}
