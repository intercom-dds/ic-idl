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
use ic_hir::hir::{
    Decl, DefFlags, DefId, DefKind, Label, Member, Numeric, ParamKind, Span, Ty, TyKind, Variant,
};
use ic_hir::{Context, ResolvedGraph};

use crate::tree::Leaf;

fn emit_span(span: &Span) -> String {
    format!(
        "<{:?}:{}, {:?}:{}>",
        span.start.file_id, span.start.offset, span.end.file_id, span.end.offset,
    )
    .yellow()
}

fn emit_ann_arg(arg: &ic_hir::hir::AnnArg) -> String {
    format!("{} = {}", arg.ident.name, emit_numeric(&arg.value))
}

fn emit_ann_node(ann: &ic_hir::hir::Ann) -> Leaf<String> {
    let ann_str = if ann.args.is_empty() {
        format!("@{}", ann.ident.name)
    } else {
        let args = ann
            .args
            .iter()
            .map(emit_ann_arg)
            .collect::<Vec<_>>()
            .join(", ");
        format!("@{}({})", ann.ident.name, args)
    };

    let span = emit_span(&ann.ident.span);
    leaf!(
        "{} adt(def={}) {span} {}",
        "ann".purple(),
        format!("{:#02X?}", ann.def_id).blue(),
        ann_str.cyan(),
    )
}

fn emit_param_kind(kind: ParamKind) -> &'static str {
    match kind {
        ParamKind::In => "in",
        ParamKind::Out => "out",
        ParamKind::Inout => "inout",
    }
}

fn emit_flags(flags: DefFlags) -> String {
    let mut buf = vec![];
    if flags.contains(DefFlags::IS_CIRCULAR) {
        buf.push("circular");
    }
    if flags.contains(DefFlags::IS_TRIVIAL) {
        buf.push("trivial");
    }
    if flags.contains(DefFlags::IS_BUILTIN) {
        buf.push("builtin");
    }
    if flags.contains(DefFlags::IS_SYNTHESIZED) {
        buf.push("synth");
    }
    if flags.contains(DefFlags::IS_INCOMPLETE) {
        buf.push("incomplete");
    }
    if flags.contains(DefFlags::IS_EMIT) {
        buf.push("emit");
    }
    if flags.contains(DefFlags::TOTAL_ORDER) {
        buf.push("ord");
    }
    buf.join(" ")
}

fn emit_ty(context: &Context, ty: &Ty) -> String {
    let kind = match &ty.kind {
        TyKind::Any => "any",
        TyKind::Fixed => "fixed",
        TyKind::Null => "null",
        TyKind::Primitive(kind) => {
            return kind.to_string().to_ascii_lowercase().cyan();
        }
        TyKind::String { wide, bound, .. } => {
            let prefix = if *wide { "w" } else { "" };
            let bound = if let Some(bound) = bound {
                format!("<{bound}>")
            } else {
                String::new()
            };
            return format!("{prefix}string{bound}").cyan();
        }
        TyKind::Array { ty, len, .. } => {
            let ty = emit_ty(context, ty);
            return format!("array<{ty}, {len}>").cyan();
        }
        TyKind::Sequence { ty, bound, .. } => {
            let ty = emit_ty(context, ty);
            let bound = bound.map(|v| format!(", {v}")).unwrap_or_default();
            return format!("sequence<{ty}{bound}>").cyan();
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            let bound = bound.map(|v| format!(", {v}")).unwrap_or_default();
            return format!(
                "map<{}, {}{bound}>",
                emit_ty(context, key),
                emit_ty(context, elem)
            )
            .cyan();
        }
        TyKind::Adt(id) => {
            let name = context.type_of(*id).ident.name.cyan();
            return format!(
                "{}({}{}, {name})",
                "adt".cyan(),
                "def=".clear(),
                format!("{id:#02X?}").blue(),
            );
        }
    };

    kind.cyan()
}

fn emit_numeric(val: &Numeric) -> String {
    match val {
        Numeric::Null => "null".to_string(),
        Numeric::Bool(b) => b.to_string().to_uppercase(),
        Numeric::Char(c) => format!("'{c}'"),
        Numeric::Int8(i) => i.to_string(),
        Numeric::Octet(o) => o.to_string(),
        Numeric::Int16(i) => i.to_string(),
        Numeric::UInt16(u) => u.to_string(),
        Numeric::Int32(i) => i.to_string(),
        Numeric::UInt32(u) => u.to_string(),
        Numeric::Int64(i) => i.to_string(),
        Numeric::UInt64(u) => u.to_string(),
        Numeric::Float(f) => f.to_string(),
        Numeric::Double(d) => d.to_string(),
        Numeric::String(s) => format!("\"{s}\""),
        Numeric::Const(def_id) => format!("<const {def_id:#02X?}>"),
        Numeric::Array { .. } => "<array>".to_string(),
        Numeric::Sequence { .. } => "<sequence>".to_string(),
        Numeric::Map { .. } => "<map>".to_string(),
        Numeric::Struct { .. } => "<struct>".to_string(),
        Numeric::Union { .. } => "<union>".to_string(),
    }
}

fn emit_ann_param(context: &Context, param: &ic_hir::hir::AnnParam) -> Leaf<String> {
    let span = emit_span(&param.ident.span);
    let ty = emit_ty(context, &param.ty);

    let mut member = leaf!(
        "{} {span} {} emit",
        "param".green().bold(),
        param.ident.name.cyan(),
    );

    if let Some(ref default) = param.default {
        let default_str = emit_numeric(default);
        member.push(leaf!("{} {}", "default".purple(), default_str.yellow()));
    }

    member.push(ty);

    member
}

fn emit_member(context: &Context, mem: &Member) -> Leaf<String> {
    let span = emit_span(&mem.ident.span);
    let ty = emit_ty(context, &mem.ty);

    let mut member = leaf!(
        "{} {span} {} emit",
        "member".green().bold(),
        mem.ident.name.cyan(),
    );

    // Add annotation nodes
    for ann in &mem.annotations {
        member.push(emit_ann_node(ann));
    }

    member.push(leaf!("{} {ty}", "type".purple()));

    member
}

fn emit_label(context: &Context, label: &Label) -> Leaf<String> {
    let span = emit_span(&label.span);
    let value_str = if let Numeric::Const(def_id) = &label.value {
        let def = context.type_of(*def_id);
        let qualified_name = context.qualified_name(*def_id);
        let value = if let DefKind::Const(const_ty) = &def.kind {
            format!(" '= {}'", emit_numeric(&const_ty.value)).purple()
        } else {
            emit_numeric(&label.value)
        };
        format!("{}{}", qualified_name.cyan(), value)
    } else {
        format!("' = {}'", emit_numeric(&label.value)).purple()
    };

    leaf!("{} {span} {}", "label".green().bold(), value_str)
}

fn emit_variant(context: &Context, var: &Variant) -> Leaf<String> {
    let span = emit_span(&var.ident.span);
    let default = if var.is_default { "default" } else { "" };

    let mut node = leaf!(
        "{} {span} '{}' emit {default}",
        "variant".green().bold(),
        &var.ident.name.cyan(),
    );

    // Add annotation nodes
    for ann in &var.annotations {
        node.push(emit_ann_node(ann));
    }

    let ty = emit_ty(context, &var.ty);
    node.push(leaf!("{} {}", "type".purple(), ty));

    for label in &var.labels {
        node.push(emit_label(context, label));
    }
    node
}

#[allow(clippy::too_many_lines)]
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
        DefKind::Bitset(_) => "bitset",
        DefKind::Alias(_) => "alias",
        DefKind::Interface(_) => "interface",
        DefKind::Valuetype(_) => "valuetype",
        DefKind::Decl { .. } => "decl",
    };

    let span = emit_span(&def.span);

    let mut node = leaf!(
        "{} def={} {span} {} {}",
        kind.green().bold(),
        format!("{id:#02X?}").blue(),
        def.ident.name.cyan(),
        emit_flags(def.flags),
    );

    // Add annotation nodes first
    for ann in &def.annotations {
        node.push(emit_ann_node(ann));
    }

    match &def.kind {
        DefKind::Annotation(v) => {
            let nested = v.types.iter().map(|&v| emit_def(context, v));
            node.extend(nested);

            let params = v.params.iter().map(|v| emit_ann_param(context, v));
            node.extend(params);
        }
        DefKind::Module(v) => {
            let nested = v.definitions.iter().map(|&v| emit_def(context, v));
            node.extend(nested);
        }
        DefKind::Struct(v) => {
            if let Some(parent) = v.parent {
                let parent = &context.type_of(parent).ident.name;
                node.push(leaf!("{} {}", "parent".purple(), parent.cyan()));
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

            for &var_id in &v.fields {
                let var_def = context.definitions.get(var_id);
                let span = emit_span(&var_def.ident.span);

                if let DefKind::Const(const_ty) = &var_def.kind {
                    let value = match const_ty.value {
                        Numeric::Int32(v) => v.to_string(),
                        Numeric::Int64(v) => v.to_string(),
                        _ => "?".to_string(),
                    };

                    let mut enum_node = leaf!(
                        "{} {span} {} {}",
                        "enumerator".green().bold(),
                        &var_def.ident.name.cyan(),
                        format!("'= {value}'").purple(),
                    );

                    // Add annotation nodes
                    for ann in &var_def.annotations {
                        enum_node.push(emit_ann_node(ann));
                    }

                    node.push(enum_node);
                }
            }
        }
        DefKind::Const(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty} builtin", "type".purple()));
            node.push(leaf!("{} {:?}", "value".purple(), v.value));
        }
        DefKind::Bitmask(v) => {
            let ty = emit_ty(context, &v.ty);
            node.push(leaf!("{} {ty} builtin", "type".purple()));

            for flag in &v.flags {
                let span = emit_span(&flag.ident.span);

                let mut flag_node = leaf!(
                    "{} {span} {} {}",
                    "flag".green().bold(),
                    &flag.ident.name.cyan(),
                    format!("'= {}'", flag.value).purple(),
                );

                // Add annotation nodes
                for ann in &flag.annotations {
                    flag_node.push(emit_ann_node(ann));
                }

                node.push(flag_node);
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
                    "{} {span} {}",
                    "prototype".green().bold(),
                    def.ident.name.cyan(),
                );
                proto.push(leaf!("{} {}", "return".purple(), emit_ty(context, &def.ty)));
                for param in &def.params {
                    let mut arg = leaf!(
                        "{} {} {}",
                        "param".green().bold(),
                        param.ident.name.cyan(),
                        emit_param_kind(param.kind),
                    );
                    arg.push(leaf!("{} {}", "type".purple(), emit_ty(context, &param.ty)));
                    proto.push(arg);
                }
                node.push(proto);
            }
            for attr in &v.attributes {
                let span = emit_span(&attr.ident.span);
                let ty = emit_ty(context, &attr.ty);
                let readonly = if attr.is_readonly { "readonly " } else { "" };

                let mut attr_node = leaf!(
                    "{} {span} {} {readonly}emit",
                    "attribute".green().bold(),
                    attr.ident.name.cyan(),
                );

                attr_node.push(leaf!("{} {}", "type".purple(), ty));

                if !attr.getraises.is_empty() {
                    let raises = attr
                        .getraises
                        .iter()
                        .map(|&id| context.type_of(id).ident.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    attr_node.push(leaf!("{} {}", "getraises".purple(), raises.cyan()));
                }

                if !attr.setraises.is_empty() {
                    let raises = attr
                        .setraises
                        .iter()
                        .map(|&id| context.type_of(id).ident.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    attr_node.push(leaf!("{} {}", "setraises".purple(), raises.cyan()));
                }

                node.push(attr_node);
            }
        }
        DefKind::Valuetype(v) => {
            for def in &v.definitions {
                node.push(emit_def(context, *def));
            }
        }
        DefKind::Bitset(v) => {
            if let Some(parent) = v.parent {
                let parent = &context.type_of(parent).ident.name;
                node.push(leaf!("{} {}", "parent".purple(), parent.cyan()));
            }

            for field in &v.fields {
                let span = emit_span(&field.ident.span);

                let mut field_node = leaf!(
                    "{} {span} {} size={}",
                    "bitfield".green().bold(),
                    &field.ident.name.cyan(),
                    field.size.to_string().purple(),
                );

                // Add annotation nodes
                for ann in &field.annotations {
                    field_node.push(emit_ann_node(ann));
                }

                field_node.push(leaf!("{} {}", "type".purple(), emit_ty(context, &field.ty)));
                node.push(field_node);
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
            node.push(leaf!("{} {}", "kind".purple(), kind.cyan()));
        }
    }
    node
}

fn plural(word: &str, count: usize) -> String {
    let s = if count == 1 { "" } else { "s" };
    format!("{} {word}{s}", count.green())
}

#[must_use]
pub fn emit_tree(result: &ResolvedGraph) -> String {
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
    buf
}
