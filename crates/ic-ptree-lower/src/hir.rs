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

#![allow(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::ffi::CString;
use std::{ffi, ptr};

use ic_hir::hir::{
    Ann, Decl, DefId, DefKind, Ident, Numeric, ParamKind, PrimitiveTy, ProtoTy, Ty, TyKind, Variant,
};
use ic_hir::{Context, ResolvedGraph};
use ic_ptree::{ParseResult, sys};
use ic_vfs::SourceMap;

// SAFETY: Taking the address of a static variable is safe
#[allow(unused_unsafe)]
pub static mut NUM_UNDEF: *const sys::numeric = unsafe { std::ptr::addr_of!(sys::num_undef) };

struct TreeBuilder<'a> {
    ctx: &'a Context,
    state: *mut sys::parser_state,
    lowered: HashMap<DefId, *mut sys::ptree>,
}

impl<'a> TreeBuilder<'a> {
    fn new(state: *mut sys::parser_state, tree: &'a ResolvedGraph) -> Self {
        Self {
            state,
            ctx: &tree.context,
            lowered: HashMap::new(),
        }
    }

    unsafe fn lower_bound(&self, bound: Option<usize>) -> *const sys::numeric {
        bound.map_or(NUM_UNDEF, |bound| {
            sys::create_u64(self.state, bound as u64, 10)
        })
    }

    unsafe fn lower_ty(&mut self, ty: &Ty) -> *mut sys::ptree {
        match &ty.kind {
            TyKind::Any => ptr::addr_of_mut!(sys::any_type),
            TyKind::Fixed => ptr::addr_of_mut!(sys::fixed_type),
            TyKind::Primitive(kind) => match kind {
                PrimitiveTy::Void => std::ptr::null_mut(),
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
            TyKind::Array { ty, len, .. } => {
                let ty = self.lower_ty(ty);
                let bound = sys::create_u64(self.state, *len as u64, 10);
                let decl = sys::append_array_size(self.state, ptr::null_mut(), bound);
                sys::create_array_type(self.state, decl, ty)
            }
            TyKind::Sequence { ty, bound, .. } => {
                let ty = self.lower_ty(ty);
                let bound = self.lower_bound(*bound);
                sys::create_sequence(self.state, ty, bound)
            }
            TyKind::String { wide, bound, .. } => {
                let bound = self.lower_bound(*bound);
                if *wide {
                    sys::create_wstring(self.state, bound)
                } else {
                    sys::create_string(self.state, bound)
                }
            }
            TyKind::Map {
                key, elem, bound, ..
            } => {
                let key = self.lower_ty(key);
                let elem = self.lower_ty(elem);
                let bound = self.lower_bound(*bound);
                sys::create_map(self.state, key, elem, bound)
            }
            TyKind::Adt(id) => self.lookup_node(*id),
            TyKind::Null => ptr::null_mut(), // Null type has no ptree representation
        }
    }

    unsafe fn lower_decl(&self, ident: &Ident) -> *mut sys::declarator {
        let ident = create_ident(&ident.name);
        sys::create_decl(self.state, ident.as_ptr(), ptr::null_mut())
    }

    unsafe fn lower_proto(&mut self, proto: &ProtoTy) -> *mut sys::ptree {
        let params = collect_with(self.state, sys::append_node, &proto.params, |param| {
            let ty = self.lower_ty(&param.ty);
            let kind = param_kind(param.kind);
            let decl = self.lower_decl(&param.ident);
            sys::create_param_dcl(self.state, decl, ty, kind as ffi::c_int)
        });

        let ident = create_ident(&proto.ident.name);
        let ret = self.lower_ty(&proto.ty);
        sys::create_interface_op(self.state, ident.as_ptr(), params, ret, ptr::null_mut())
    }

    unsafe fn lower_variant(&mut self, var: &Variant) -> *mut sys::ptree {
        let cases = collect_with(self.state, sys::append_node, &var.labels, |label| {
            let value = self.lower_numeric(&label.value);
            sys::create_case_label(self.state, value)
        });

        // Check if this is a null case
        if matches!(var.ty.kind, TyKind::Null) {
            let null_node = sys::create_null_node(self.state);
            let annotations = self.lower_annotations(&var.annotations);
            sys::create_union_member(self.state, null_node, cases, annotations)
        } else {
            let decl = self.lower_decl(&var.ident);
            let ty = self.lower_ty(&var.ty);
            let mem = sys::create_member(self.state, decl, ty, ptr::null_mut());
            let annotations = self.lower_annotations(&var.annotations);
            sys::create_union_member(self.state, mem, cases, annotations)
        }
    }

    unsafe fn lower_numeric(&mut self, num: &Numeric) -> *const sys::numeric {
        match num {
            Numeric::Bool(v) => sys::create_bool(self.state, ffi::c_int::from(*v)),
            Numeric::Char(v) => sys::create_char(self.state, *v as ffi::c_char),
            Numeric::Int8(v) => sys::create_i64(self.state, i64::from(*v), 10),
            Numeric::Octet(v) => sys::create_u64(self.state, u64::from(*v), 10),
            Numeric::Int16(v) => sys::create_i64(self.state, i64::from(*v), 10),
            Numeric::UInt16(v) => sys::create_u64(self.state, u64::from(*v), 10),
            Numeric::Int32(v) => sys::create_i64(self.state, i64::from(*v), 10),
            Numeric::UInt32(v) => sys::create_u64(self.state, u64::from(*v), 10),
            Numeric::Int64(v) => sys::create_i64(self.state, *v, 10),
            Numeric::UInt64(v) => sys::create_u64(self.state, *v, 10),
            Numeric::Float(v) => sys::create_float(self.state, *v),
            Numeric::Double(v) => sys::create_double(self.state, *v),
            Numeric::String(v) => {
                let str = CString::new(v.clone()).unwrap();
                sys::create_str(self.state, str.as_ptr())
            }
            Numeric::Const(v) => {
                let node = self.lookup_node(*v);
                assert!(!node.is_null());
                let numeric = sys::create_numeric_node(self.state, node);
                sys::create_value_node(self.state, numeric, ptr::null_mut())
            }
            Numeric::Struct { fields, .. } => {
                let values = collect_with(self.state, sys::append_node, fields, |(ident, num)| {
                    let num = self.lower_numeric(num);
                    let decl = self.lower_decl(ident);
                    sys::create_const_node(self.state, decl, std::ptr::null_mut(), num)
                });
                sys::create_value_node(self.state, NUM_UNDEF, values)
            }
            Numeric::Sequence { values, .. } | Numeric::Array { values, .. } => {
                let values = collect_with(self.state, sys::append_node, values, |num| {
                    let num = self.lower_numeric(num);
                    sys::create_const_node(
                        self.state,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        num,
                    )
                });
                sys::create_value_node(self.state, NUM_UNDEF, values)
            }
            Numeric::Null | Numeric::Map { .. } | Numeric::Union { .. } => NUM_UNDEF,
        }
    }

    unsafe fn lower_annotation(&mut self, ann: &Ann) -> *mut sys::ptree {
        let name = format!("@{}", ann.ident.name);
        let ident = create_ident(&name);

        sys::create_annotation_start(self.state, ident.as_ptr());
        let params = collect_with(self.state, sys::append_node, &ann.args, |arg| {
            let decl = create_ident(&arg.ident.name);
            let val = self.lower_numeric(&arg.value);
            sys::create_annotation_param(self.state, decl.as_ptr(), val)
        });
        sys::create_annotation_finish(self.state, params)
    }

    unsafe fn lower_annotations(&mut self, anns: &[Ann]) -> *mut sys::ptree {
        collect_with(self.state, sys::append_node, anns, |ann| {
            self.lower_annotation(ann)
        })
    }

    unsafe fn annotate(&mut self, node: *mut sys::ptree, anns: &[Ann]) -> *mut sys::ptree {
        let annotations = self.lower_annotations(anns);
        sys::annotate(self.state, node, annotations);
        node
    }

    unsafe fn lookup_node(&self, id: DefId) -> *mut sys::ptree {
        let name = self.ctx.qualified_name(id);
        let ident = create_ident(&name);
        unsafe { sys::lookup_node(self.state, ident.as_ptr()) }
    }

    #[allow(clippy::too_many_lines)]
    unsafe fn lower_def(&mut self, id: DefId) -> *mut sys::ptree {
        // If this has been lowered before, return the corresponding node
        if let Some(v) = self.lowered.get(&id) {
            return *v;
        }

        let def = self.ctx.type_of(id);
        let ident = create_ident(&def.ident.name);
        let ident = ident.as_ptr();

        let node = match &def.kind {
            DefKind::Annotation(v) => {
                sys::create_annotation_dcl_start(self.state, ident);
                let types = collect_with(self.state, sys::append_node, &v.types, |id| {
                    self.lower_def(*id)
                });
                let fields = collect_with(self.state, sys::append_node, &v.params, |param| {
                    let ty = self.lower_ty(&param.ty);
                    let decl = self.lower_decl(&param.ident);
                    let default = param
                        .default
                        .as_ref()
                        .map_or(NUM_UNDEF, |v| self.lower_numeric(v));
                    sys::create_annotation_member(self.state, decl, ty, default)
                });
                let members = sys::append_node(self.state, types, fields);
                sys::create_annotation_dcl_finish(self.state, members)
            }
            DefKind::Module(v) => {
                sys::create_module_start(self.state, ident);
                let members = collect_with(self.state, sys::append_node, &v.definitions, |id| {
                    self.lower_def(*id)
                });
                sys::create_module_finish(self.state, members)
            }
            DefKind::Struct(v) => {
                let parent = v.parent.map_or(ptr::null_mut(), |id| self.lower_def(id));
                let ty = sys::create_struct_start(self.state, ident, parent);

                // Structs may be self-referential so we need to cache the node
                // before lowering any of its members.
                self.lowered.insert(id, ty);

                let members = collect_with(self.state, sys::append_node, &v.members, |mem| {
                    let ty = self.lower_ty(&mem.ty);
                    let decl = self.lower_decl(&mem.ident);
                    let ann = self.lower_annotations(&mem.annotations);
                    sys::create_member(self.state, decl, ty, ann)
                });
                sys::create_struct_finish(self.state, members)
            }
            DefKind::Except(v) => {
                let ty = sys::create_exception_start(self.state, ident);
                self.lowered.insert(id, ty);

                let members = collect_with(self.state, sys::append_node, &v.members, |mem| {
                    let ty = self.lower_ty(&mem.ty);
                    let decl = self.lower_decl(&mem.ident);
                    let ann = self.lower_annotations(&mem.annotations);
                    sys::create_member(self.state, decl, ty, ann)
                });
                sys::create_exception_finish(self.state, members)
            }
            DefKind::Union(v) => {
                let ty = sys::create_union_start(self.state, ident);
                self.lowered.insert(id, ty);

                let variants = collect_with(self.state, sys::append_node, &v.variants, |var| {
                    self.lower_variant(var)
                });

                let ty = self.lower_ty(&v.disc);
                let disc = sys::create_union_discriminator(self.state, ty, ptr::null_mut());
                sys::create_union_finish(self.state, disc, variants)
            }
            DefKind::Enum(v) => {
                let values = collect_with(self.state, sys::append_enum_node, &v.fields, |&var| {
                    let var = self.ctx.type_of(var);
                    let name = create_ident(&var.ident.name);
                    if let DefKind::Const(const_ty) = &var.kind {
                        let value = self.lower_numeric(&const_ty.value);
                        let node = sys::create_enum_value(self.state, name.as_ptr(), value);
                        self.annotate(node, &var.annotations)
                    } else {
                        std::ptr::null_mut()
                    }
                });
                sys::create_enum(self.state, ident, values)
            }
            DefKind::Const(v) => {
                let ty = self.lower_ty(&v.ty);
                let decl = self.lower_decl(&def.ident);
                let value = self.lower_numeric(&v.value);
                sys::create_const_node(self.state, decl, ty, value)
            }
            DefKind::Bitmask(v) => {
                let values = collect_with(self.state, sys::append_enum_node, &v.flags, |flag| {
                    let name = create_ident(&flag.ident.name);
                    let value = sys::create_u64(self.state, flag.value as u64, 10);
                    let node = sys::create_bitmask_value(self.state, name.as_ptr(), value);
                    self.annotate(node, &flag.annotations)
                });
                sys::create_bitmask(self.state, ident, values)
            }
            DefKind::Alias(v) => {
                let ty = self.lower_ty(&v.ty);
                let decl = self.lower_decl(&def.ident);
                sys::create_type(self.state, decl, ty)
            }
            DefKind::Interface(v) => {
                let ty = sys::create_interface_start(
                    self.state,
                    ident,
                    ptr::null_mut(),
                    ffi::c_int::from(v.is_local),
                );
                self.lowered.insert(id, ty);

                let members = collect_with(self.state, sys::append_node, &v.prototypes, |proto| {
                    self.lower_proto(proto)
                });
                sys::create_interface_finish(self.state, members)
            }
            DefKind::Valuetype(_) => {
                let ty = sys::create_valuetype_start(
                    self.state,
                    ident,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
                self.lowered.insert(id, ty);
                sys::create_valuetype_finish(self.state, ptr::null_mut())
            }
            DefKind::Bitset(_) => {
                // TODO: Implement bitset lowering when ptree supports it
                std::ptr::null_mut()
            }
            DefKind::Decl(v) => match v {
                Decl::Struct => sys::create_struct_dcl(self.state, ident),
                Decl::Union => sys::create_union_dcl(self.state, ident),
                Decl::Native => sys::create_native_type(self.state, ident),
                Decl::Interface => sys::create_interface_dcl(self.state, ident, 0),
                Decl::Valuetype => sys::create_valuetype_dcl(self.state, ident),
            },
        };

        // Apply annotations
        self.annotate(node, &def.annotations);
        self.lowered.insert(id, node);
        node
    }
}

#[must_use]
fn create_ident(name: &str) -> CString {
    CString::new(name).unwrap()
}

#[allow(clippy::cast_possible_wrap)]
fn param_kind(kind: ParamKind) -> ffi::c_int {
    let c = match kind {
        ParamKind::In => sys::OPT_IN,
        ParamKind::Out => sys::OPT_OUT,
        ParamKind::Inout => sys::OPT_INOUT,
    };
    c as ffi::c_int
}

type Appender = unsafe extern "C" fn(
    *mut sys::parser_state,
    *mut sys::ptree,
    *mut sys::ptree,
) -> *mut sys::ptree;

#[must_use]
unsafe fn collect_with<I, C, T>(
    state: *mut sys::parser_state,
    appender: Appender,
    iter: I,
    mut cb: C,
) -> *mut sys::ptree
where
    I: IntoIterator<Item = T>,
    C: FnMut(T) -> *mut sys::ptree,
{
    let mut list = std::ptr::null_mut();

    // SAFETY: The appender function is expected to handle the pointers correctly
    // and the callback is responsible for returning valid pointers
    unsafe {
        for elem in iter {
            let node = cb(elem);
            list = appender(state, list, node);
        }
    }
    list
}

pub unsafe fn lower(hir: &ResolvedGraph, vfs: &SourceMap) -> ParseResult {
    let state = unsafe { sys::ic_parser_create() };

    // Lower the tree
    let mut builder = TreeBuilder::new(state, hir);

    // Lower built-in definitions, but discard the nodes
    if !hir.builtin_order.is_empty() {
        let include = create_ident("<builtin-annotations>");
        sys::create_include_start(state, include.as_ptr(), 0);
        let nodes = collect_with(state, sys::append_node, &hir.builtin_order, |id| {
            builder.lower_def(*id)
        });
        sys::create_include_finish(state, nodes);
    }

    // Lower user definitions
    let tree = collect_with(state, sys::append_node, &hir.order, |id| {
        let def = builder.ctx.definitions.get(id);
        let defined_in = format!("{}", vfs.name(def.span.start.file_id).display());
        let include = create_ident(&defined_in);

        sys::create_include_start(state, include.as_ptr(), 0);
        let node = builder.lower_def(*id);
        sys::create_include_finish(state, node)
    });

    let result = {
        let inner = sys::ic_parser_result(state, tree);
        ParseResult::from_raw(inner)
    };

    if let Some(err) = result.diagnostics() {
        panic!("ptree lowering failed: {err}");
    }
    result
}
