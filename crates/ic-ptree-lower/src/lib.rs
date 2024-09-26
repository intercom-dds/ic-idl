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

use core::ffi;
use std::ffi::CString;
use std::ptr;

use ic_hir::Context;
use ic_hir::hir::*;
use ic_ptree::{ParseResult, sys};

#[allow(unused_unsafe)]
static mut NUM_UNDEF: *const sys::numeric = unsafe { ptr::addr_of!(sys::num_undef) };

/// Lowers the HIR into a `ptree`. This process should be infallible, as
/// everything should have been resolved and type checked prior to this.
pub fn from_hir(tree: &ic_hir::ResolvedGraph) -> ParseResult {
    let state = unsafe { sys::ic_parser_create() };
    let builder = TreeBuilder {
        state,
        ctx: &tree.context,
    };

    let mut list = ptr::null_mut();
    for id in &tree.order {
        unsafe {
            let node = builder.lower_def(*id);
            list = sys::append_node(state, list, node);
        }
    }

    unsafe {
        let result = sys::ic_parser_result(builder.state, list);
        ParseResult::from_raw(result)
    }
}

fn create_ident(name: &str) -> CString {
    CString::new(name).unwrap()
}

struct TreeBuilder<'a> {
    ctx: &'a Context,
    state: *mut sys::parser_state,
}

impl TreeBuilder<'_> {
    unsafe fn lower_ty(&self, ty: &Ty) -> *mut sys::ptree {
        match ty {
            Ty::Any => ptr::addr_of_mut!(sys::any_type),
            Ty::Fixed => ptr::addr_of_mut!(sys::fixed_type),
            Ty::Primitive(kind) => match kind {
                PrimitiveTy::Bool => ptr::addr_of_mut!(sys::boolean_type),
                PrimitiveTy::Char => ptr::addr_of_mut!(sys::char_type),
                PrimitiveTy::WChar => ptr::addr_of_mut!(sys::wchar_type),
                PrimitiveTy::Int8 => ptr::addr_of_mut!(sys::int8_type),
                PrimitiveTy::UInt8 => ptr::addr_of_mut!(sys::octet_type),
                PrimitiveTy::Int16 => ptr::addr_of_mut!(sys::short_type),
                PrimitiveTy::UInt16 => ptr::addr_of_mut!(sys::ushort_type),
                PrimitiveTy::Int32 => ptr::addr_of_mut!(sys::long_type),
                PrimitiveTy::UInt32 => ptr::addr_of_mut!(sys::ulong_type),
                PrimitiveTy::Int64 => ptr::addr_of_mut!(sys::longlong_type),
                PrimitiveTy::UInt64 => ptr::addr_of_mut!(sys::ulonglong_type),
                PrimitiveTy::Float32 => ptr::addr_of_mut!(sys::float_type),
                PrimitiveTy::Float64 => ptr::addr_of_mut!(sys::double_type),
                PrimitiveTy::Float128 => ptr::addr_of_mut!(sys::ldouble_type),
            },
            // Ty::Array { .. } => (),
            // Ty::Sequence { .. } => (),
            Ty::String { wide, .. } => {
                if *wide {
                    sys::create_wstring(self.state, NUM_UNDEF)
                } else {
                    sys::create_string(self.state, NUM_UNDEF)
                }
            }
            // Ty::Map { .. } => (),
            Ty::Adt(id) => {
                let ty = self.ctx.type_of(*id);
                // TODO: we have to provide a qualified name here, probably.
                // or just cache the types and do the lookup ourselves.
                let ident = create_ident(&ty.ident.name);
                sys::lookup_type(self.state, ident.as_ptr())
            }
            _ => todo!(),
        }
    }

    unsafe fn lower_decl(&self, ident: &Ident) -> *mut sys::declarator {
        let ident = create_ident(&ident.name);
        sys::create_decl(self.state, ident.as_ptr(), ptr::null_mut())
    }

    unsafe fn lower_def(&self, id: DefId) -> *mut sys::ptree {
        let def = self.ctx.type_of(id);
        let ident = create_ident(&def.ident.name);
        let ident = ident.as_ptr();

        match &def.kind {
            DefKind::Annotation(v) => {
                sys::create_annotation_dcl_start(self.state, ident);
                let mut members = ptr::null_mut();
                for id in &v.types {
                    let node = self.lower_def(*id);
                    members = sys::append_node(self.state, members, node);
                }
                for mem in &v.members {
                    let ty = self.lower_ty(&mem.ty);
                    let decl = self.lower_decl(&mem.ident);
                    let node = sys::create_annotation_member(self.state, decl, ty, NUM_UNDEF);
                    members = sys::append_node(self.state, members, node);
                }
                sys::create_annotation_dcl_finish(self.state, members)
            }
            DefKind::Module(v) => {
                sys::create_module_start(self.state, ident);
                let mut members = ptr::null_mut();
                for id in &v.definitions {
                    let node = self.lower_def(*id);
                    members = sys::append_node(self.state, members, node);
                }
                sys::create_module_finish(self.state, members)
            }
            DefKind::Struct(v) => {
                let parent = v
                    .parent
                    .map(|id| self.lower_def(id))
                    .unwrap_or(ptr::null_mut());

                sys::create_struct_start(self.state, ident, parent);
                let mut members = ptr::null_mut();
                for mem in &v.members {
                    let ty = self.lower_ty(&mem.ty);
                    let decl = self.lower_decl(&mem.ident);
                    let node = sys::create_member(self.state, decl, ty, ptr::null_mut());
                    members = sys::append_node(self.state, members, node);
                }
                sys::create_struct_finish(self.state, members)
            }
            DefKind::Except(v) => {
                sys::create_exception_start(self.state, ident);
                let mut members = ptr::null_mut();
                for mem in &v.members {
                    let ty = self.lower_ty(&mem.ty);
                    let decl = self.lower_decl(&mem.ident);
                    let node = sys::create_member(self.state, decl, ty, ptr::null_mut());
                    members = sys::append_node(self.state, members, node);
                }
                sys::create_exception_finish(self.state, members)
            }
            DefKind::Union(v) => {
                sys::create_union_start(self.state, ident);
                let mut variants = ptr::null_mut();
                for var in &v.variants {
                    let mut cases = ptr::null_mut();
                    for _ in &var.labels {
                        let label = sys::create_case_label(self.state, NUM_UNDEF);
                        cases = sys::append_node(self.state, cases, label);
                    }

                    let decl = self.lower_decl(&var.ident);
                    let ty = self.lower_ty(&var.ty);
                    let mem = sys::create_member(self.state, decl, ty, ptr::null_mut());
                    let mem = sys::create_union_member(self.state, mem, cases, ptr::null_mut());
                    variants = sys::append_node(self.state, variants, mem);
                }

                let ty = self.lower_ty(&v.disc);
                let disc = sys::create_union_discriminator(self.state, ty, ptr::null_mut());
                sys::create_union_finish(self.state, disc, variants)
            }
            DefKind::Enum(v) => {
                let mut values = ptr::null_mut();
                for var in &v.fields {
                    let name = create_ident(&var.ident.name);
                    let node = sys::create_enum_value(self.state, name.as_ptr(), NUM_UNDEF);
                    values = sys::append_enum_node(self.state, values, node);
                }
                sys::create_enum(self.state, ident, values)
            }
            DefKind::Const(v) => {
                let ty = self.lower_ty(&v.ty);
                let decl = self.lower_decl(&def.ident);
                sys::create_const_node(self.state, decl, ty, NUM_UNDEF)
            }
            DefKind::Bitmask(v) => {
                let mut values = ptr::null_mut();
                for var in &v.flags {
                    let name = create_ident(&var.ident.name);
                    let node = sys::create_bitmask_value(self.state, name.as_ptr(), NUM_UNDEF);
                    values = sys::append_enum_node(self.state, values, node);
                }
                sys::create_bitmask(self.state, ident, values)
            }
            DefKind::Alias(v) => {
                let ty = self.lower_ty(&v.ty);
                let decl = self.lower_decl(&def.ident);
                sys::create_type(self.state, decl, ty)
            }
            DefKind::Interface(v) => {
                sys::create_interface_start(self.state, ident, ptr::null_mut(), 0);
                let mut members = ptr::null_mut();
                for proto in &v.prototypes {
                    let mut params = ptr::null_mut();
                    for param in &proto.params {
                        let ty = self.lower_ty(&param.ty);
                        let kind = match param.kind {
                            ParamKind::In => sys::OPT_IN,
                            ParamKind::Out => sys::OPT_OUT,
                            ParamKind::Inout => sys::OPT_INOUT,
                        };
                        let decl = self.lower_decl(&param.ident);
                        let param = sys::create_param_dcl(self.state, decl, ty, kind as ffi::c_int);
                        params = sys::append_node(self.state, params, param);
                    }

                    let ident = create_ident(&proto.ident.name);
                    let ret = self.lower_ty(&proto.ty);
                    let op = sys::create_interface_op(
                        self.state,
                        ident.as_ptr(),
                        params,
                        ret,
                        ptr::null_mut(),
                    );
                    members = sys::append_node(self.state, members, op);
                }
                sys::create_interface_finish(self.state, members)
            }
            DefKind::Valuetype(_) => {
                sys::create_valuetype_start(self.state, ident, ptr::null_mut(), ptr::null_mut());
                sys::create_valuetype_finish(self.state, ptr::null_mut())
            }
            DefKind::Decl(v) => match v {
                Decl::Struct => sys::create_struct_dcl(self.state, ident),
                Decl::Union => sys::create_union_dcl(self.state, ident),
                Decl::Native => sys::create_native_type(self.state, ident),
                Decl::Interface => sys::create_interface_dcl(self.state, ident, 0),
                Decl::Valuetype => sys::create_valuetype_dcl(self.state, ident),
            },
        }
    }
}
