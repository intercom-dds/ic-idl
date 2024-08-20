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

use std::ffi::{self, CString};

use ic_syntax::{DeclKind, Declarator, Expr, Item, LitKind, Op, OpKind, ParamKind, Path, Type};

use crate::ptree::{self, ptree as node};

// TODO: refactor ptree to use span instead of position
const POS: ptree::position = ptree::position {
    line: -1,
    column: -1,
};

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
        OpKind::OpNot => b'|',
    };
    c as ffi::c_char
}

fn param_kind(kind: Option<ParamKind>) -> ffi::c_int {
    let c = match kind {
        Some(ParamKind::ParamOut) => ptree::OPT_OUT,
        Some(ParamKind::ParamInout) => ptree::OPT_INOUT,
        _ => ptree::OPT_IN,
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

unsafe fn create_ident(name: &str) -> ptree::identifier {
    let name = CString::new(name).unwrap();
    ptree::create_identifier(name.as_ptr())
}

unsafe fn create_decl(names: &[Declarator], _annotations: *mut node) -> *mut ptree::declarator {
    let mut list = std::ptr::null_mut();
    for name in names {
        let decl = match name {
            Declarator::Simple(v) => {
                let ident = create_ident(&v.name);
                ptree::create_decl(ident, std::ptr::null_mut())
            }
            Declarator::Array(v) => {
                let ident = create_ident(&v.ident.name);
                let mut decl = ptree::create_decl(ident, std::ptr::null_mut());
                for bound in &v.bounds {
                    let expr = lower_expr(bound);
                    decl = ptree::append_array_size(decl, expr);
                }
                decl
            }
        };
        list = ptree::append_decl(list, decl);
    }
    list
}

// TODO: accept closure instead? so we can replace logic in StructValue
unsafe fn lower_list(items: &[Item]) -> *mut node {
    let mut list = std::ptr::null_mut();
    for item in items {
        let item = lower_item(item);
        list = ptree::append_node(list, item);
    }
    list
}

unsafe fn lower_ty(ty: &Type) -> *mut node {
    match ty {
        Type::Any(_) => std::ptr::addr_of_mut!(ptree::any_type),
        Type::Fixed(_) => std::ptr::addr_of_mut!(ptree::fixed_type),
        Type::Sequence(v) => {
            let ty = lower_ty(&v.ty);
            ptree::create_sequence(ty, std::ptr::null_mut())
        }
        Type::String_(v) => match &v.bound {
            Some(_) if v.wide => todo!(),
            Some(_) => todo!(),
            None if v.wide => std::ptr::addr_of_mut!(ptree::unbounded_wstring_type),
            None => std::ptr::addr_of_mut!(ptree::unbounded_string_type),
        },
        Type::Map(v) => {
            let key = lower_ty(&v.key);
            let elem = lower_ty(&v.value);
            ptree::create_map(key, elem, std::ptr::null_mut())
        }
        Type::Path(v) => {
            let ident = create_ident(&path_str(v));
            ptree::lookup_type(ident)
        }
    }
}

unsafe fn lower_expr(num: &Expr) -> *const ptree::numeric {
    match num {
        Expr::Literal(v) => match v.kind.clone() {
            LitKind::LitBool(v) => ptree::create_bool(v as ffi::c_int),
            LitKind::LitInt(v) => ptree::create_i64(v as i64, 10),
            LitKind::LitFloat(v) => ptree::create_double(v),
            LitKind::LitChar(v) => ptree::create_char(v as ffi::c_char),
            LitKind::LitString(v) => {
                let str = CString::new(v).unwrap();
                ptree::create_str(str.as_ptr())
            }
        },
        Expr::Path(v) => {
            let ident = create_ident(&path_str(v));
            ptree::lookup_value(ident)
        }
        Expr::Unary(v) => {
            let op = op_kind(v.op);
            ptree::expr_unary(op, lower_expr(&v.expr))
        }
        Expr::Binary(v) => {
            let op = op_kind(v.op);
            ptree::expr_binary(op, lower_expr(&v.lhs), lower_expr(&v.rhs))
        }
        Expr::InitList(_) => {
            // create_const_node(decl, type_, value)
            todo!()
        }
    }
}

unsafe fn lower_item(item: &Item) -> *mut node {
    match item {
        Item::AnnotationValue(_) => todo!(),
        Item::ModuleValue(v) => {
            let ident = create_ident(&v.name.name);
            ptree::create_module_start(ident);

            let members = lower_list(&v.definitions);
            ptree::create_module_finish(members, POS)
        }
        Item::StructValue(v) => {
            let ident = create_ident(&v.name.name);
            // TODO: parent
            ptree::create_struct_start(ident, std::ptr::null_mut());
            let mut list = std::ptr::null_mut();

            for mem in &v.members {
                let ty = lower_ty(&mem.ty);
                let decls = create_decl(&mem.names, std::ptr::null_mut());
                let mem = ptree::create_member(decls, ty, std::ptr::null_mut());
                list = ptree::append_node(list, mem);
            }
            ptree::create_struct_finish(list, POS)
        }
        Item::UnionValue(_) => todo!(),
        Item::EnumValue(v) => {
            // TODO: what about collecting "into" a list?
            // let values = v
            //     .fields
            //     .iter()
            //     .map(|v| {
            //         let ident = create_ident(&v.name.name);
            //         // TODO: update value
            //         ptree::create_enum_value(ident, std::ptr::null_mut())
            //     })
            //     .collect();

            let mut values = std::ptr::null_mut();
            for field in &v.fields {
                let ident = create_ident(&field.name.name);
                let expr = field
                    .value
                    .as_ref()
                    .map_or(std::ptr::null(), |v| lower_expr(v));

                let val = ptree::create_enum_value(ident, expr);
                values = ptree::append_enum_node(values, val);
            }

            let ident = create_ident(&v.name.name);
            ptree::create_enum(ident, values, POS)
        }
        Item::ExceptionValue(v) => {
            let ident = create_ident(&v.name.name);
            ptree::create_exception_start(ident);
            let mut list = std::ptr::null_mut();

            for mem in &v.members {
                let ty = lower_ty(&mem.ty);
                let decls = create_decl(&mem.names, std::ptr::null_mut());
                let mem = ptree::create_member(decls, ty, std::ptr::null_mut());
                list = ptree::append_node(list, mem);
            }
            ptree::create_struct_finish(list, POS)
        }
        Item::BitmaskValue(v) => {
            let mut values = std::ptr::null_mut();
            for bit in &v.bits {
                let ident = create_ident(&bit.name.name);
                let expr = bit
                    .value
                    .as_ref()
                    .map_or(std::ptr::null(), |v| lower_expr(v));

                let val = ptree::create_bitmask_value(ident, expr);
                values = ptree::append_node(values, val);
            }

            let ident = create_ident(&v.name.name);
            ptree::create_bitmask(ident, values, POS)
        }
        Item::ConstValue(v) => {
            let ty = lower_ty(&v.ty);
            let decl = std::ptr::null_mut();
            let expr = lower_expr(&v.value);
            ptree::create_const_node(decl, ty, expr)
        }
        Item::TypedefValue(v) => {
            let ty = lower_ty(&v.ty);
            let _ident = create_ident(&v.name.name);
            // ...declarators...
            ptree::create_type(std::ptr::null_mut(), ty)
        }
        Item::DeclValue(v) => {
            let ident = create_ident(&v.name.name);
            match v.kind {
                DeclKind::DeclStruct => ptree::create_struct_dcl(ident),
                DeclKind::DeclUnion => ptree::create_union_dcl(ident),
                DeclKind::DeclNative => ptree::create_native_type(ident),
                DeclKind::DeclInterface => ptree::create_interface_dcl(ident, 0),
                DeclKind::DeclValuetype => ptree::create_valuetype_dcl(ident),
            }
        }
        Item::BitsetValue(v) => {
            let mut list = std::ptr::null_mut();
            for field in &v.fields {
                let decl = std::ptr::null_mut();
                let size = lower_expr(&field.size);
                let ty = std::ptr::null_mut();
                let bitfield = ptree::create_bitfield(decl, size, ty);
                list = ptree::append_node(list, bitfield);
            }

            let ident = create_ident(&v.name.name);
            ptree::create_bitset(ident, list, std::ptr::null_mut(), POS)
        }
        Item::InterfaceValue(v) => {
            // TODO: parents
            let ident = create_ident(&v.name.name);
            ptree::create_interface_start(
                ident,
                std::ptr::null_mut(),
                v.local.is_some() as ffi::c_int,
            );

            let mut list = std::ptr::null_mut();
            for proto in &v.prototypes {
                let ident = create_ident(&proto.name.name);
                let params = {
                    let mut params = std::ptr::null_mut();
                    for p in &proto.params {
                        let ty = lower_ty(&p.ty);
                        let ident = std::ptr::null_mut();
                        let kind = param_kind(p.kind);

                        // TODO: does this need to be a declarator?? can't it
                        // be an identifier??
                        let dcl = ptree::create_param_dcl(ident, ty, kind);
                        params = ptree::append_node(params, dcl);
                    }
                    params
                };

                // TODO: retval, raises
                let dcl = ptree::create_interface_op(
                    ident,
                    params,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                list = ptree::append_node(list, dcl);
            }
            list
        }
        Item::ValuetypeValue(_) => todo!(),
    }
}

pub extern "C" fn callback(ptr: *mut ffi::c_void) -> *mut node {
    unsafe {
        let ast = ptr.cast::<&[Item]>();

        // TODO: we need to add info about includes
        lower_list(*ast)
    }
}
