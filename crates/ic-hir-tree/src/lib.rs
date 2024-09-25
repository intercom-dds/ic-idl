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

mod tree;

use std::fmt::Write;

use ic_cli::color::Colorize;
use ic_hir::hir::*;
use ic_hir::{Context, ResolvedGraph};

use crate::tree::Leaf;

fn emit_span(span: &Span) -> String {
    format!(
        "<{:?}:{}, {:?}:{}>",
        span.start.file_id, span.start.offset, span.end.file_id, span.end.offset,
    )
    .yellow()
}

fn emit_ty(context: &Context, ty: &Ty) -> String {
    let kind = match ty {
        Ty::Any => "any",
        Ty::Fixed => "fixed",
        Ty::Primitive(kind) => match kind {
            PrimitiveTy::Bool => "bool",
            PrimitiveTy::Char => "char",
            PrimitiveTy::WChar => "wchar",
            PrimitiveTy::Int8 => "int8",
            PrimitiveTy::UInt8 => "uint8",
            PrimitiveTy::Int16 => "int16",
            PrimitiveTy::UInt16 => "uint16",
            PrimitiveTy::Int32 => "int32",
            PrimitiveTy::UInt32 => "uint32",
            PrimitiveTy::Int64 => "int64",
            PrimitiveTy::UInt64 => "uint64",
            PrimitiveTy::Float => "float",
            PrimitiveTy::Double => "double",
            PrimitiveTy::String => "string",
            PrimitiveTy::WString => "wstring",
        },
        Ty::String { wide, bound } => {
            let prefix = if *wide { "w" } else { "" };
            let bound = if let Some(bound) = bound {
                format!("<{bound}>")
            } else {
                String::new()
            };
            return format!("{prefix}string{bound}").cyan();
        }
        Ty::Array { ty, len } => {
            let ty = emit_ty(context, ty);
            return format!("array<{ty}, {len}>").cyan();
        }
        Ty::Sequence { ty, bound } => {
            let ty = emit_ty(context, ty);
            let bound = bound.map(|v| format!(", {v}")).unwrap_or_default();
            return format!("sequence<{ty}{bound}>").cyan();
        }
        Ty::Map { key, elem, bound } => {
            let bound = bound.map(|v| format!(", {v}")).unwrap_or_default();
            return format!(
                "map<{}, {}{bound}>",
                emit_ty(context, key),
                emit_ty(context, elem)
            )
            .cyan();
        }
        Ty::Adt(id) => {
            let name = context.type_of(*id).ident.name.cyan();
            return format!(
                "{}({}{}, {name})",
                "adt".cyan(),
                "def=".clear(),
                format!("0x{id:#02?}").blue(),
            );
        }
    };

    kind.cyan()
}

fn emit_member(context: &Context, mem: &Member) -> Leaf<String> {
    let span = emit_span(&mem.ident.span);
    let ty = emit_ty(context, &mem.ty);
    let mut member = leaf!(
        "{} {span} {} emit",
        "member".green().bold(),
        mem.ident.name.cyan(),
    );
    member.push(leaf!("{} {ty}", "type".purple()));
    member
}

fn emit_variant(context: &Context, var: &Variant) -> Leaf<String> {
    let span = emit_span(&var.ident.span);
    let default = if var.is_default { "default" } else { "" };

    let mut node = leaf!(
        "{} {span} '{}' emit {default}",
        "variant".green().bold(),
        &var.ident.name.cyan(),
    );
    let ty = emit_ty(context, &var.ty);
    node.push(leaf!("{} {}", "type".purple(), ty));

    for label in &var.labels {
        node.push(leaf!("{} {label:?}", "label".green().bold()));
    }
    node
}

fn emit_def(context: &Context, id: DefId) -> Leaf<String> {
    let def = context.definitions.get(id);
    let kind = match def.kind {
        DefKind::Annotation(_) => "annotation",
        DefKind::Module(_) => "module",
        DefKind::Struct(_) => "struct",
        DefKind::Except(_) => "exception",
        DefKind::Union(_) => "union",
        DefKind::Enum(_) => "enum",
        DefKind::Const(_) => "const",
        DefKind::Bitmask(_) => "bitmask",
        DefKind::Alias(_) => "alias",
        DefKind::Interface(_) => "interface",
        DefKind::Valuetype(_) => "valuetype",
        DefKind::Decl { .. } => "decl",
    };

    let span = emit_span(&def.span);
    let mut node = leaf!(
        "{} def={} {span} {} emit",
        kind.green().bold(),
        format!("0x{id:02X?}").blue(),
        def.ident.name.cyan(),
    );

    match &def.kind {
        DefKind::Annotation(v) => {
            let nested = v.types.iter().map(|&v| emit_def(context, v));
            node.extend(nested);

            let members = v.members.iter().map(|v| emit_member(context, v));
            node.extend(members);
        }
        DefKind::Module(v) => {
            let nested = v.definitions.iter().map(|&v| emit_def(context, v));
            node.extend(nested);
        }
        DefKind::Struct(v) => {
            if let Some(_parent) = &v.parent {
                node.push(leaf!("{}", "parent".purple()));
            }
            let members = v.members.iter().map(|v| emit_member(context, v));
            node.extend(members);
        }
        DefKind::Except(v) => {
            let members = v.members.iter().map(|v| emit_member(context, v));
            node.extend(members);
        }
        DefKind::Union(v) => {
            let disc_ty = emit_ty(context, &v.disc);
            node.push(leaf!("{} {disc_ty}", "disc".purple()));

            let variants = v.variants.iter().map(|v| emit_variant(context, v));
            node.extend(variants);
        }
        DefKind::Enum(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty} builtin", "type".purple()));

            for var in &v.fields {
                let span = emit_span(&var.ident.span);
                node.push(leaf!(
                    "{} {span} {} {}",
                    "enumerator".green().bold(),
                    &var.ident.name.cyan(),
                    format!("'= {}'", var.value).purple(),
                ));
            }
        }
        DefKind::Const(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty} builtin", "type".purple()));
        }
        DefKind::Bitmask(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty} builtin", "type".purple()));

            for flag in &v.flags {
                let span = emit_span(&flag.ident.span);
                node.push(leaf!(
                    "{} {span} {} {}",
                    "flag".green().bold(),
                    &flag.ident.name.cyan(),
                    format!("'= {}'", flag.value).purple(),
                ));
            }
        }
        DefKind::Alias(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty}", "type".purple()));
        }
        DefKind::Interface(v) => {
            for def in &v.definitions {
                node.push(emit_def(context, *def));
            }
            for def in &v.prototypes {
                let span = emit_span(&def.ident.span);
                let mut proto = leaf!(
                    "{} {span} {} emit",
                    "prototype".green().bold(),
                    def.ident.name.cyan()
                );
                proto.push(leaf!("{} {}", "return".purple(), emit_ty(context, &def.ty)));
                for param in &def.params {
                    // TODO: in/inout/out
                    let mut arg = leaf!(
                        "{} {} inout",
                        "param".green().bold(),
                        param.ident.name.cyan()
                    );
                    arg.push(leaf!("{} {}", "type".purple(), emit_ty(context, &param.ty)));
                    proto.push(arg);
                }
                node.push(proto);
            }
            for _ in &v.attributes {
                node.push("attrib".green());
            }
        }
        DefKind::Valuetype(v) => {
            for def in &v.definitions {
                node.push(emit_def(context, *def));
            }
        }
        DefKind::Decl(kind) => {
            let kind = match kind {
                Decl::Struct => "struct",
                Decl::Union => "union",
                Decl::Native => "native",
                Decl::Interface => "interface",
                Decl::Valuetype => "valuetype",
            };
            node.push(leaf!("{} {}", "kind".green(), kind.cyan()));
        }
    }
    node
}

fn plural(word: &str, count: usize) -> String {
    let s = if count == 1 { "" } else { "s" };
    format!("{} {word}{s}", count.green())
}

pub fn emit_tree(result: &ResolvedGraph) {
    let leaves = result.order.iter().map(|id| emit_def(&result.context, *id));
    let mut root = leaf!("{}", ".".gray());
    root.extend(leaves);

    let mut buf = String::new();
    _ = writeln!(&mut buf, "{root}");
    _ = write!(
        &mut buf,
        "{}",
        plural("definition", result.context.definitions.len()),
    );
    println!("{buf}");
}
