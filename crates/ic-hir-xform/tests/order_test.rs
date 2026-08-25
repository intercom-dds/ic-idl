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

mod common;

use std::collections::{HashMap, HashSet};

use ic_hir::ResolvedGraph;
use ic_hir::hir::{AliasTy, Def, DefFlags, DefId, DefKind, Ident, PrimitiveTy, Span, Ty, TyKind};
use ic_hir_xform::order;
use ic_vfs::{FileId, SourceMap};

#[derive(Clone, Debug)]
struct Entry {
    def_id: DefId,
    path: Vec<String>,
    name: String,
    is_decl: bool,
}

impl Entry {
    fn qualified(&self) -> String {
        if self.path.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.path.join("::"), self.name)
        }
    }

    fn rendered(&self) -> String {
        if self.is_decl {
            format!("decl {}", self.qualified())
        } else {
            self.qualified()
        }
    }
}

fn walk(hir: &ResolvedGraph, order: &[DefId], path: &[String], out: &mut Vec<Entry>) {
    for &def_id in order {
        let def = hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Module(module) => {
                let mut nested = path.to_vec();
                nested.push(def.ident.name.clone());
                walk(hir, &module.definitions, &nested, out);
            }
            kind => out.push(Entry {
                def_id,
                path: path.to_vec(),
                name: def.ident.name.clone(),
                is_decl: matches!(kind, DefKind::Decl(_)),
            }),
        }
    }
}

fn entries(hir: &ResolvedGraph) -> Vec<Entry> {
    let mut out = Vec::new();
    walk(hir, &hir.order, &[], &mut out);
    out
}

fn emission(hir: &ResolvedGraph) -> Vec<String> {
    entries(hir).iter().map(Entry::rendered).collect()
}

fn decl_names(hir: &ResolvedGraph) -> Vec<String> {
    entries(hir)
        .iter()
        .filter(|entry| entry.is_decl)
        .map(Entry::qualified)
        .collect()
}

fn block_tree(hir: &ResolvedGraph) -> Vec<String> {
    fn walk(hir: &ResolvedGraph, order: &[DefId], path: &[String], out: &mut Vec<String>) {
        for &def_id in order {
            let def = hir.context.definitions.get(def_id);

            let DefKind::Module(module) = &def.kind else {
                continue;
            };

            let mut nested = path.to_vec();
            nested.push(def.ident.name.clone());

            out.push(nested.join("::"));

            let children = module.definitions.clone();
            walk(hir, &children, &nested, out);
        }
    }

    let mut out = Vec::new();
    walk(hir, &hir.order, &[], &mut out);
    out
}

fn top_level_modules(hir: &ResolvedGraph) -> Vec<String> {
    hir.order
        .iter()
        .filter_map(|&def_id| {
            let def = hir.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Module(_)).then(|| def.ident.name.clone())
        })
        .collect()
}

fn external_sites(hir: &ResolvedGraph) -> Vec<String> {
    let mut out = Vec::new();

    for entry in entries(hir) {
        let def = hir.context.definitions.get(entry.def_id);

        if ic_hir_analysis::annotation::is_external(&hir.context, def) {
            out.push(entry.qualified());
        }

        match &def.kind {
            DefKind::Struct(s) => {
                for member in &s.members {
                    if ic_hir_analysis::annotation::is_external(&hir.context, member) {
                        out.push(format!("{}.{}", entry.qualified(), member.ident.name));
                    }
                }
            }
            DefKind::Valuetype(v) => {
                for member in &v.members {
                    if ic_hir_analysis::annotation::is_external(&hir.context, member) {
                        out.push(format!("{}.{}", entry.qualified(), member.ident.name));
                    }
                }
            }
            DefKind::Except(e) => {
                for member in &e.members {
                    if ic_hir_analysis::annotation::is_external(&hir.context, member) {
                        out.push(format!("{}.{}", entry.qualified(), member.ident.name));
                    }
                }
            }
            DefKind::Union(u) => {
                for variant in &u.variants {
                    if ic_hir_analysis::annotation::is_external(&hir.context, variant) {
                        out.push(format!("{}.{}", entry.qualified(), variant.ident.name));
                    }
                }
            }
            _ => {}
        }
    }

    out
}

fn decl_kinds(hir: &ResolvedGraph) -> Vec<String> {
    entries(hir)
        .iter()
        .filter_map(|entry| {
            let DefKind::Decl(decl) = hir.context.definitions.get(entry.def_id).kind else {
                return None;
            };

            Some(format!("{decl:?} {}", entry.qualified()))
        })
        .collect()
}

fn strip_decls(hir: &mut ResolvedGraph) {
    let kept: Vec<_> = hir
        .order
        .iter()
        .copied()
        .filter(|&id| !matches!(hir.context.definitions.get(id).kind, DefKind::Decl(_)))
        .collect();
    hir.order = kept;

    let mut stack = hir.order.clone();

    while let Some(def_id) = stack.pop() {
        let DefKind::Module(module) = &hir.context.definitions.get(def_id).kind else {
            continue;
        };

        let kept: Vec<_> = module
            .definitions
            .iter()
            .copied()
            .filter(|&id| !matches!(hir.context.definitions.get(id).kind, DefKind::Decl(_)))
            .collect();

        stack.extend(kept.iter().copied());

        if let DefKind::Module(module) = &mut hir.context.definitions.get_mut(def_id).kind {
            module.definitions = kept;
        }
    }
}

fn reverse_order(hir: &mut ResolvedGraph) {
    hir.order.reverse();

    let mut stack = hir.order.clone();

    while let Some(def_id) = stack.pop() {
        let DefKind::Module(module) = &mut hir.context.definitions.get_mut(def_id).kind else {
            continue;
        };

        module.definitions.reverse();
        let children = module.definitions.clone();
        stack.extend(children);
    }
}

fn leaf_ids(hir: &ResolvedGraph) -> Vec<DefId> {
    entries(hir).into_iter().map(|entry| entry.def_id).collect()
}

fn graph_input(mut hir: ResolvedGraph) -> ResolvedGraph {
    strip_decls(&mut hir);

    let leaves = leaf_ids(&hir);
    let mut offsets: Vec<u32> = leaves
        .iter()
        .map(|&id| hir.context.definitions.get(id).span.start.offset)
        .collect();
    offsets.reverse();

    for (&def_id, offset) in leaves.iter().zip(offsets) {
        hir.context.definitions.get_mut(def_id).span.start.offset = offset;
    }

    reverse_order(&mut hir);
    hir
}

fn assert_declare_before_use(hir: &ResolvedGraph) {
    let all = entries(hir);

    let present: HashSet<DefId> = all
        .iter()
        .filter(|entry| !entry.is_decl)
        .map(|entry| entry.def_id)
        .collect();

    let site: HashMap<DefId, (Vec<String>, String)> = all
        .iter()
        .filter(|entry| !entry.is_decl)
        .map(|entry| (entry.def_id, (entry.path.clone(), entry.name.clone())))
        .collect();

    let mut declared: HashSet<(Vec<String>, String)> = HashSet::new();
    let mut emitted: HashSet<DefId> = HashSet::new();

    for entry in &all {
        if entry.is_decl {
            declared.insert((entry.path.clone(), entry.name.clone()));
            continue;
        }

        let mut deps: Vec<DefId> = hir.context.deps(entry.def_id).into_iter().collect();
        deps.sort_unstable();

        for dep in deps {
            if dep == entry.def_id || !present.contains(&dep) {
                continue;
            }

            let known = emitted.contains(&dep) || declared.contains(&site[&dep]);

            assert!(
                known,
                "{} is used by {} before it is defined or declared; emission order was {:?}",
                hir.context.qualified_name(dep),
                entry.qualified(),
                emission(hir)
            );
        }

        emitted.insert(entry.def_id);
    }
}

fn alloc_alias(hir: &mut ResolvedGraph, name: &str, span: Span, ty: Ty) -> DefId {
    hir.context.definitions.alloc_with_id(|id| Def {
        id,
        ident: Ident {
            name: name.to_string(),
            span,
        },
        parent: None,
        annotations: Vec::new(),
        span,
        kind: DefKind::Alias(AliasTy { ty }),
        flags: DefFlags::nil(),
    })
}

fn set_alias_ty(hir: &mut ResolvedGraph, def_id: DefId, ty: Ty) {
    let DefKind::Alias(alias) = &mut hir.context.definitions.get_mut(def_id).kind else {
        panic!("expected an alias definition");
    };

    alias.ty = ty;
}

#[test]
fn acyclic_single_module_is_reordered_dependency_first() {
    let idl = r"
        module M {
            struct A { long x; };
            struct B { A a; };
            struct C { B b; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(
        emission(&input),
        vec!["M::C", "M::B", "M::A"],
        "the constructed input must be in dependent-before-dependency order"
    );

    let ordered = order::apply(input);
    assert_eq!(emission(&ordered), vec!["M::A", "M::B", "M::C"]);
    assert_eq!(top_level_modules(&ordered), vec!["M"]);
    assert_declare_before_use(&ordered);
}

#[test]
fn acyclic_multiple_modules_group_into_maximal_runs() {
    let idl = r"
        module Aye {
            struct A1 { long x; };
            struct A2 { long y; };
        };

        module Bee {
            struct B1 { Aye::A1 a; Aye::A2 b; };
            struct B2 { Aye::A1 c; Aye::A2 d; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(
        emission(&input),
        vec!["Bee::B2", "Bee::B1", "Aye::A2", "Aye::A1"]
    );

    let ordered = order::apply(input);
    assert_eq!(
        emission(&ordered),
        vec!["Aye::A2", "Aye::A1", "Bee::B2", "Bee::B1"]
    );
    assert_eq!(top_level_modules(&ordered), vec!["Aye", "Bee"]);
    assert_declare_before_use(&ordered);
}

#[test]
fn intra_module_sequence_cycle_gets_a_forward_declaration() {
    let idl = r"
        module M {
            struct Bar;
            struct Foo { sequence<Bar> bars; };
            struct Bar { sequence<Foo> foos; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(emission(&input), vec!["M::Bar", "M::Foo"]);

    let ordered = order::apply(input);

    assert_eq!(emission(&ordered), vec!["decl M::Foo", "M::Bar", "M::Foo"]);
    assert_eq!(top_level_modules(&ordered), vec!["M"]);
    assert_declare_before_use(&ordered);

    let names: Vec<String> = entries(&ordered)
        .iter()
        .filter(|entry| !entry.is_decl)
        .map(Entry::qualified)
        .collect();

    assert_eq!(names, vec!["M::Bar", "M::Foo"]);
    assert_eq!(decl_names(&ordered), vec!["M::Foo"]);
}

#[test]
fn cross_module_cycle_declares_into_the_declared_types_module() {
    let idl = r"
        module Bee { struct Bar; };
        module Aye { struct Foo { sequence<Bee::Bar> bars; }; };
        module Bee { struct Bar { sequence<Aye::Foo> foos; }; };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(emission(&input), vec!["Bee::Bar", "Aye::Foo"]);

    let ordered = order::apply(input);
    assert_eq!(
        emission(&ordered),
        vec!["decl Aye::Foo", "Bee::Bar", "Aye::Foo"]
    );

    let decl = entries(&ordered)
        .into_iter()
        .find(|entry| entry.is_decl)
        .expect("a forward declaration must be inserted");

    assert_eq!(decl.name, "Foo");
    assert_eq!(decl.path, vec!["Aye".to_string()]);
    assert_eq!(top_level_modules(&ordered), vec!["Aye", "Bee", "Aye"]);

    assert_declare_before_use(&ordered);
}

#[test]
fn all_direct_union_cycle_marks_one_variant_external() {
    let idl = r"
        module M {
            union B;
            union A switch (long) { case 0: B b; };
            union B switch (long) { case 0: A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);
    assert_eq!(emission(&ordered), vec!["decl M::A", "M::B", "M::A"]);
    assert_eq!(external_sites(&ordered), vec!["M::B.a"]);
}

#[test]
fn all_direct_struct_cycle_marks_one_member_external() {
    let idl = r"
        module M {
            struct B;
            struct A { B b; };
            struct B { A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);
    assert_eq!(emission(&ordered), vec!["decl M::A", "M::B", "M::A"]);
    assert_eq!(external_sites(&ordered), vec!["M::B.a"]);
}

#[test]
fn all_direct_valuetype_cycle_marks_one_member_external() {
    let idl = r"
        module M {
            valuetype B;
            valuetype A { public B b; };
            valuetype B { public A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);
    assert_eq!(emission(&ordered), vec!["decl M::A", "M::B", "M::A"]);
    assert_eq!(external_sites(&ordered), vec!["M::B.a"]);

    let again = order::apply(ordered.clone());
    assert_eq!(
        external_sites(&again),
        external_sites(&ordered),
        "no direct layout cycle may survive the pass"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn a_struct_valuetype_direct_cycle_is_broken_at_a_member() {
    let idl = r"
        module M {
            valuetype V;
            struct S { V v; };
            valuetype V { public S s; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);
    assert_eq!(
        external_sites(&ordered),
        vec!["M::V.s"],
        "a valuetype member is a direct layout edge, so the cycle must be seen and broken"
    );

    let again = order::apply(ordered.clone());
    assert_eq!(
        external_sites(&again),
        external_sites(&ordered),
        "no direct layout cycle may survive the pass"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn a_valuetype_member_outranks_a_union_variant() {
    let idl = r"
        module M {
            valuetype V;
            union U switch (boolean) { case TRUE: V v; case FALSE: long x; };
            valuetype V { public U u; };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert_eq!(
        external_sites(&ordered),
        vec!["M::V.u"],
        "a member site outranks a variant site whatever kind owns the member"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn an_exception_member_is_never_marked_external() {
    let idl = r"
        module M {
            struct S;
            exception E { S s; };
            struct S { E e; };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert!(external_sites(&ordered).is_empty());
}

#[test]
fn cycle_through_alias_declares_the_struct_and_orders_the_alias_first() {
    let idl = r"
        module M {
            struct S;
            typedef S T;
            struct S { sequence<T> ts; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(emission(&input), vec!["M::S", "M::T"]);

    let ordered = order::apply(input);
    assert_eq!(emission(&ordered), vec!["decl M::S", "M::T", "M::S"]);
    assert_eq!(
        decl_names(&ordered),
        vec!["M::S"],
        "the declaration must name the struct at the end of the alias chain"
    );

    let rendered = emission(&ordered);
    let alias = rendered
        .iter()
        .position(|name| name == "M::T")
        .expect("the alias must be emitted");
    let definition = rendered
        .iter()
        .position(|name| name == "M::S")
        .expect("the struct must be emitted");

    assert!(
        alias < definition,
        "an alias cannot be declared ahead, so it must be emitted before its user"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn indirect_edge_suppresses_external_annotation() {
    let idl = r"
        module M {
            struct B;
            struct A { map<long, B> m; };
            struct B { A a; };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));
    assert_eq!(decl_names(&ordered), vec!["M::A"]);
    assert!(external_sites(&ordered).is_empty());
}

#[test]
fn interface_cycle_is_forward_declared_as_an_interface() {
    let idl = r"
        module M {
            interface B;
            interface A { void f(in B b); };
            interface B { void g(in A a); };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert_eq!(decl_names(&ordered), vec!["M::A"]);
    assert_eq!(
        decl_kinds(&ordered),
        vec!["Interface M::A"],
        "an interface must be forward declared as an interface"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn valuetype_cycle_is_forward_declared_as_a_valuetype() {
    let idl = r"
        module M {
            valuetype B;
            valuetype A { public B b; };
            valuetype B { public A a; };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert_eq!(decl_names(&ordered), vec!["M::A"]);
    assert_eq!(
        decl_kinds(&ordered),
        vec!["Valuetype M::A"],
        "a valuetype must be forward declared as a valuetype"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn an_exception_cycle_is_not_forward_declared() {
    let idl = r"
        module M {
            struct S;
            exception E { sequence<S> ss; };
            struct S { sequence<E> es; };
        };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert!(
        !decl_kinds(&ordered)
            .iter()
            .any(|decl| decl.ends_with("M::E")),
        "an exception has no forward declaration in IDL, so none may be emitted; got {:?}",
        decl_kinds(&ordered)
    );
}

#[test]
fn every_direct_cycle_in_one_component_is_broken() {
    let idl = r"
        module M {
            struct B;
            struct C;
            struct A { B b; C c; };
            struct B { A a; };
            struct C { A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);

    assert_eq!(external_sites(&ordered), vec!["M::C.a", "M::B.a"]);

    let again = order::apply(ordered.clone());
    assert_eq!(
        external_sites(&again),
        external_sites(&ordered),
        "no direct layout cycle may survive the pass"
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn a_direct_cycle_beside_an_indirect_one_is_still_broken() {
    let idl = r"
        module M {
            struct B;
            struct C;
            struct A { sequence<B> bs; C c; };
            struct B { A a; };
            struct C { A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    assert!(external_sites(&input).is_empty());

    let ordered = order::apply(input);

    assert_eq!(external_sites(&ordered), vec!["M::C.a"]);

    let again = order::apply(ordered.clone());
    assert_eq!(
        external_sites(&again),
        external_sites(&ordered),
        "an indirect edge elsewhere in the component must not suppress the repair"
    );
}

#[test]
fn apply_is_deterministic() {
    let idl = r"
        module Bee { struct Bar; };
        module Aye {
            struct Foo { sequence<Bee::Bar> bars; };
            struct Other { Foo f; };
        };
        module Bee {
            struct Bar { sequence<Aye::Foo> foos; };
            union Pick;
            union Sel switch (long) { case 0: Pick p; };
            union Pick switch (long) { case 0: Sel s; };
        };
    ";

    let first = order::apply(graph_input(common::parse_with_builtins(idl)));
    let second = order::apply(graph_input(common::parse_with_builtins(idl)));

    assert!(!decl_names(&first).is_empty());
    assert!(!external_sites(&first).is_empty());
    assert_eq!(emission(&first), emission(&second));
    assert_eq!(decl_names(&first), decl_names(&second));
    assert_eq!(top_level_modules(&first), top_level_modules(&second));
    assert_eq!(external_sites(&first), external_sites(&second));
}

#[test]
fn every_definition_survives_exactly_once() {
    let idl = r"
        module Aye {
            struct A1 { long x; };
            typedef sequence<A1> A1Seq;
            enum E { E_ONE, E_TWO };
        };
        module Bee {
            struct Bar;
            struct Foo { Aye::A1Seq s; sequence<Bar> bars; Aye::E e; };
            struct Bar { sequence<Foo> foos; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));

    let mut before = leaf_ids(&input);
    before.sort_unstable();

    let ordered = order::apply(input);
    let mut after: Vec<DefId> = entries(&ordered)
        .into_iter()
        .filter(|entry| !entry.is_decl)
        .map(|entry| entry.def_id)
        .collect();

    after.sort_unstable();
    assert_eq!(before, after);

    let unique: HashSet<DefId> = after.iter().copied().collect();
    assert_eq!(unique.len(), after.len(), "a definition was emitted twice");
    assert_declare_before_use(&ordered);
}

#[test]
fn cyclic_alias_chain_terminates() {
    let mut hir = common::parse_with_builtins("struct Anchor { long x; };");
    let span = hir.context.definitions.get(hir.order[0]).span;

    let placeholder = Ty {
        span,
        kind: TyKind::Primitive(PrimitiveTy::Int32),
    };

    let p = alloc_alias(&mut hir, "P", span, placeholder.clone());
    let q = alloc_alias(
        &mut hir,
        "Q",
        span,
        Ty {
            span,
            kind: TyKind::Adt(p),
        },
    );
    set_alias_ty(
        &mut hir,
        p,
        Ty {
            span,
            kind: TyKind::Adt(q),
        },
    );
    hir.order = vec![p, q];

    let ordered = order::apply(hir);
    assert_eq!(emission(&ordered).len(), 2);
    assert!(
        decl_names(&ordered).is_empty(),
        "an alias is never forward declared"
    );

    let mut hir = common::parse_with_builtins("struct Anchor { long x; };");
    let span = hir.context.definitions.get(hir.order[0]).span;

    let t = alloc_alias(&mut hir, "T", span, placeholder);
    let u = alloc_alias(
        &mut hir,
        "U",
        span,
        Ty {
            span,
            kind: TyKind::Sequence {
                ty: Box::new(Ty {
                    span,
                    kind: TyKind::Adt(t),
                }),
                bound: None,
                bound_span: None,
            },
        },
    );
    set_alias_ty(
        &mut hir,
        t,
        Ty {
            span,
            kind: TyKind::Adt(u),
        },
    );
    hir.order = vec![t, u];

    let ordered = order::apply(hir);
    assert_eq!(emission(&ordered).len(), 2);
    assert!(
        decl_names(&ordered).is_empty(),
        "an alias is never forward declared"
    );
}

fn module_docs(hir: &ResolvedGraph) -> Vec<String> {
    fn walk(hir: &ResolvedGraph, order: &[DefId], path: &[String], out: &mut Vec<String>) {
        for &def_id in order {
            let def = hir.context.definitions.get(def_id);

            let DefKind::Module(module) = &def.kind else {
                continue;
            };

            let mut nested = path.to_vec();
            nested.push(def.ident.name.clone());

            let doc = def
                .annotations
                .iter()
                .find_map(|ann| ic_hir_analysis::annotation::doc(&hir.context, ann))
                .unwrap_or_default();

            out.push(format!("{}={}", nested.join("::"), doc));

            walk(hir, &module.definitions, &nested, out);
        }
    }

    let mut out = Vec::new();
    walk(hir, &hir.order, &[], &mut out);
    out
}

fn module_flags(hir: &ResolvedGraph) -> Vec<u32> {
    hir.order
        .iter()
        .filter_map(|&def_id| {
            let def = hir.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Module(_)).then(|| def.flags.bits())
        })
        .collect()
}

#[test]
fn module_doc_survives_reordering() {
    let idl = r#"
        @doc(text = "the M module")
        module M {
            struct A { long x; };
            struct B { A a; };
        };
    "#;

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(module_docs(&input), vec!["M=the M module"]);

    let ordered = order::apply(input);

    assert_eq!(emission(&ordered), vec!["M::A", "M::B"]);
    assert_eq!(module_docs(&ordered), vec!["M=the M module"]);
}

#[test]
fn nested_modules_keep_their_own_docs() {
    let idl = r#"
        @doc(text = "the outer module")
        module IC {
            @doc(text = "the inner module")
            module Management {
                struct A { long x; };
                struct B { A a; };
            };
        };
    "#;

    let input = graph_input(common::parse_with_builtins(idl));

    let ordered = order::apply(input);

    assert_eq!(
        emission(&ordered),
        vec!["IC::Management::A", "IC::Management::B"]
    );
    assert_eq!(
        module_docs(&ordered),
        vec!["IC=the outer module", "IC::Management=the inner module"]
    );
}

#[test]
fn module_flags_survive_reordering() {
    let idl = r"
        module M {
            struct A { long x; };
        };
    ";

    let mut input = graph_input(common::parse_with_builtins(idl));

    let module_id = *input
        .order
        .iter()
        .find(|&&def_id| {
            matches!(
                input.context.definitions.get(def_id).kind,
                DefKind::Module(_)
            )
        })
        .expect("the fixture must declare a module");

    input
        .context
        .definitions
        .get_mut(module_id)
        .flags
        .set(DefFlags::IS_SYNTHESIZED);

    let expected = input.context.definitions.get(module_id).flags.bits();
    assert_ne!(expected, 0, "the fixture must set a distinguishable flag");

    let ordered = order::apply(input);
    assert_eq!(
        module_flags(&ordered),
        vec![expected],
        "a synthesized module def must carry the original module def's flags"
    );
}

#[test]
fn reopened_module_carries_its_doc_only_once() {
    let idl = r#"
        module Bee { struct Bar; };

        @doc(text = "the Aye module")
        module Aye {
            struct Foo { sequence<Bee::Bar> bars; };
        };

        module Bee { struct Bar { sequence<Aye::Foo> foos; }; };
    "#;

    let input = graph_input(common::parse_with_builtins(idl));
    assert_eq!(
        module_docs(&input),
        vec!["Bee=", "Aye=the Aye module", "Bee="]
    );

    let ordered = order::apply(input);
    assert_eq!(top_level_modules(&ordered), vec!["Aye", "Bee", "Aye"]);
    assert_eq!(
        module_docs(&ordered),
        vec!["Aye=the Aye module", "Bee=", "Aye="],
    );
}

fn emitted_defs(hir: &ResolvedGraph) -> Vec<DefId> {
    fn walk(hir: &ResolvedGraph, order: &[DefId], out: &mut Vec<DefId>) {
        for &def_id in order {
            out.push(def_id);

            if let DefKind::Module(module) = &hir.context.definitions.get(def_id).kind {
                let children = module.definitions.clone();
                walk(hir, &children, out);
            }
        }
    }

    let mut out = Vec::new();
    walk(hir, &hir.order, &mut out);
    out
}

fn live_defs(hir: &ResolvedGraph) -> HashSet<DefId> {
    let mut live: HashSet<DefId> = emitted_defs(hir).into_iter().collect();
    let mut stack = hir.builtin_order.clone();

    while let Some(def_id) = stack.pop() {
        live.insert(def_id);

        if let DefKind::Module(module) = &hir.context.definitions.get(def_id).kind {
            stack.extend(module.definitions.iter().copied());
        }
    }

    live
}

fn dropped_module_contents(hir: &ResolvedGraph) -> Vec<String> {
    let live = live_defs(hir);

    hir.context
        .definitions
        .iter()
        .filter(|(id, _)| !live.contains(id))
        .filter_map(|(id, def)| match &def.kind {
            DefKind::Module(module) if !module.definitions.is_empty() => Some(format!(
                "{:?} `{}` still holds {} definitions",
                id,
                hir.context.qualified_name(id),
                module.definitions.len()
            )),
            _ => None,
        })
        .collect()
}

fn unresolvable_defs(hir: &ResolvedGraph) -> Vec<String> {
    let emitted: HashSet<DefId> = emitted_defs(hir).into_iter().collect();
    let mut out = Vec::new();

    for def_id in emitted_defs(hir) {
        let name = hir.context.qualified_name(def_id);
        let found = hir.context.lookup_symbol(&name);

        if !found.is_some_and(|id| emitted.contains(&id)) {
            out.push(format!("{name} resolved to {found:?}"));
        }
    }

    out
}

#[test]
fn the_first_block_of_a_module_is_the_original_module_def() {
    let idl = r"
        module M {
            struct A { long x; };
            struct B { A a; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    let original = *input
        .order
        .iter()
        .find(|&&def_id| {
            matches!(
                input.context.definitions.get(def_id).kind,
                DefKind::Module(_)
            )
        })
        .expect("the fixture must declare a module");

    let ordered = order::apply(input);
    assert_eq!(ordered.order, vec![original],);
    assert!(dropped_module_contents(&ordered).is_empty());
    assert!(unresolvable_defs(&ordered).is_empty());
}

#[test]
fn a_reopened_module_keeps_its_name_bound_to_an_emitted_block() {
    let idl = r"
        module Bee {
            struct Bar;
            struct Baz { long x; };
        };
        module Aye { struct Foo { sequence<Bee::Bar> bars; }; };
        module Bee { struct Bar { sequence<Aye::Foo> foos; }; };
    ";

    let ordered = order::apply(graph_input(common::parse_with_builtins(idl)));
    let blocks = top_level_modules(&ordered);
    assert!(blocks.len() > blocks.iter().collect::<HashSet<_>>().len(),);
    assert!(dropped_module_contents(&ordered).is_empty(),);
    assert!(unresolvable_defs(&ordered).is_empty());
}

fn place_in_file(hir: &mut ResolvedGraph, def_id: DefId, file_id: FileId) {
    let def = hir.context.definitions.get_mut(def_id);
    def.span.start.file_id = file_id;
    def.span.end.file_id = file_id;
    def.ident.span.start.file_id = file_id;
    def.ident.span.end.file_id = file_id;
}

fn blocks_by_file(hir: &ResolvedGraph) -> Vec<(FileId, Vec<String>)> {
    hir.order
        .iter()
        .map(|&def_id| {
            let def = hir.context.definitions.get(def_id);

            let children: &[DefId] = match &def.kind {
                DefKind::Module(module) => &module.definitions,
                _ => &[],
            };

            let names = children
                .iter()
                .map(|&child| hir.context.definitions.get(child).ident.name.clone())
                .collect();

            (def.ident.span.start.file_id, names)
        })
        .collect()
}

#[test]
fn a_module_split_across_files_is_emitted_as_one_block_per_file() {
    let idl = r"
        module M {
            struct A { long x; };
            struct B { long y; };
        };
    ";

    let mut input = graph_input(common::parse_with_builtins(idl));
    let mut source_map = SourceMap::default();
    let first = source_map.embed_with_name("first.idl", "");
    let second = source_map.embed_with_name("second.idl", "");
    let elsewhere = source_map.embed_with_name("elsewhere.idl", "");
    assert!(
        first != second && second != elsewhere,
        "the fixture must mint three distinct files"
    );

    let module_id = *input
        .order
        .iter()
        .find(|&&def_id| {
            matches!(
                input.context.definitions.get(def_id).kind,
                DefKind::Module(_)
            )
        })
        .expect("the fixture must declare a module");

    for def_id in leaf_ids(&input) {
        let name = input.context.definitions.get(def_id).ident.name.clone();

        let file_id = match name.as_str() {
            "A" => first,
            "B" => second,
            other => panic!("the fixture must hold only A and B, found `{other}`"),
        };

        place_in_file(&mut input, def_id, file_id);
    }

    place_in_file(&mut input, module_id, elsewhere);

    let ordered = order::apply(input);
    assert_eq!(
        blocks_by_file(&ordered),
        vec![
            (first, vec!["A".to_string()]),
            (second, vec!["B".to_string()]),
        ],
    );
    assert!(dropped_module_contents(&ordered).is_empty());
    assert!(unresolvable_defs(&ordered).is_empty());
}

fn set_offsets(hir: &mut ResolvedGraph, offsets: &[(&str, u32)]) {
    for entry in entries(hir) {
        let Some(&(_, offset)) = offsets.iter().find(|(name, _)| *name == entry.name) else {
            panic!("the fixture must give `{}` an offset", entry.name);
        };

        hir.context
            .definitions
            .get_mut(entry.def_id)
            .span
            .start
            .offset = offset;
    }
}

#[test]
fn sibling_modules_share_one_enclosing_block() {
    let idl = r"
        module IC {
            module Common { struct C1 { long x; }; };
            module Management { struct M1 { IC::Common::C1 c; }; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    let ordered = order::apply(input);

    assert_eq!(
        emission(&ordered),
        vec!["IC::Common::C1", "IC::Management::M1"]
    );
    assert_eq!(
        block_tree(&ordered),
        vec!["IC", "IC::Common", "IC::Management"],
        "changing the inner module must not reopen the enclosing module"
    );
    assert_eq!(top_level_modules(&ordered), vec!["IC"]);
    assert_declare_before_use(&ordered);
}

#[test]
fn a_nested_module_run_stays_inside_the_enclosing_block() {
    let idl = r"
        module IC {
            module Common { struct C1 { long x; }; };
            module Management {
                struct M1 { IC::Common::C1 c; };
                module UCM { struct U1 { IC::Management::M1 m; }; };
                struct M2 { IC::Management::UCM::U1 u; };
            };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    let ordered = order::apply(input);
    assert_eq!(
        emission(&ordered),
        vec![
            "IC::Common::C1",
            "IC::Management::M1",
            "IC::Management::UCM::U1",
            "IC::Management::M2",
        ],
    );
    assert_eq!(
        block_tree(&ordered),
        vec!["IC", "IC::Common", "IC::Management", "IC::Management::UCM"],
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn a_ready_sibling_is_emitted_before_leaving_its_module() {
    let idl = r"
        module IC {
            module Common {
                struct C1 { long x; };
                struct C2 { long y; };
            };
            module Management { struct M1 { IC::Common::C1 c; }; };
        };
    ";

    let mut input = common::parse_with_builtins(idl);
    set_offsets(&mut input, &[("C1", 10), ("M1", 20), ("C2", 30)]);

    let ordered = order::apply(input);
    assert_eq!(
        emission(&ordered),
        vec!["IC::Common::C1", "IC::Common::C2", "IC::Management::M1"],
    );
    assert_eq!(
        block_tree(&ordered),
        vec!["IC", "IC::Common", "IC::Management"]
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn a_required_reopen_happens_inside_one_enclosing_block() {
    let idl = r"
        module IC {
            module Bee { struct Bar; struct Baz { long x; }; };
            module Aye { struct Foo { sequence<IC::Bee::Bar> bars; }; };
            module Bee { struct Bar { sequence<IC::Aye::Foo> foos; }; };
        };
    ";

    let input = graph_input(common::parse_with_builtins(idl));
    let ordered = order::apply(input);
    let blocks = block_tree(&ordered);
    assert_eq!(blocks.iter().filter(|name| *name == "IC").count(), 1);

    let inner: Vec<_> = blocks.iter().filter(|name| *name != "IC").collect();
    assert!(inner.len() > inner.iter().collect::<HashSet<_>>().len());
    assert!(dropped_module_contents(&ordered).is_empty());
    assert!(unresolvable_defs(&ordered).is_empty(),);
    assert_declare_before_use(&ordered);
}

#[test]
fn a_singleton_sorting_between_cycle_members_is_emitted_between_them() {
    let idl = r"
        module M {
            struct Cee;
            struct Aye { long x; };
            struct Bee { sequence<Cee> cs; };
            struct Cee { sequence<Bee> bs; };
        };
    ";

    let mut input = common::parse_with_builtins(idl);
    strip_decls(&mut input);
    set_offsets(&mut input, &[("Bee", 10), ("Aye", 20), ("Cee", 30)]);

    let ordered = order::apply(input);

    assert_eq!(
        emission(&ordered),
        vec!["decl M::Cee", "M::Bee", "M::Aye", "M::Cee"],
    );
    assert_declare_before_use(&ordered);
}

#[test]
fn sharing_the_current_file_outranks_sharing_a_longer_module_path() {
    let idl = r"
        module IC {
            module Common {
                struct C1 { long x; };
                struct C2 { long y; };
            };
            module Other { struct O1 { long z; }; };
        };
    ";

    let mut input = common::parse_with_builtins(idl);
    set_offsets(&mut input, &[("C1", 10), ("C2", 20), ("O1", 30)]);

    let mut source_map = SourceMap::default();
    let first = source_map.embed_with_name("first.idl", "");
    let second = source_map.embed_with_name("second.idl", "");
    assert!(first != second, "the fixture must mint two distinct files");

    for def_id in leaf_ids(&input) {
        let name = input.context.definitions.get(def_id).ident.name.clone();

        let file_id = match name.as_str() {
            "C1" | "O1" => first,
            "C2" => second,
            other => panic!("the fixture must hold only C1, C2 and O1, found `{other}`"),
        };

        place_in_file(&mut input, def_id, file_id);
    }

    let ordered = order::apply(input);

    assert_eq!(
        emission(&ordered),
        vec!["IC::Common::C1", "IC::Other::O1", "IC::Common::C2"],
    );
    assert_declare_before_use(&ordered);
}
