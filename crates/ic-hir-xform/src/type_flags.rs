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

//! Marks types with `IS_TRIVIAL` and `TOTAL_ORDER` flags.
//!
//! This transformation analyzes types to determine:
//! - `IS_TRIVIAL`: Types that consist only of primitive types and arrays
//! - `TOTAL_ORDER`: Types whose members can form a well-ordered set

use std::collections::{HashMap, HashSet};

use ic_hir::hir::{Def, DefFlags, DefId, DefKind, PrimitiveTy, Ty, TyKind};
use ic_hir::{Context, ResolvedGraph};

/// Result of analyzing a type for flags
#[derive(Debug, Clone, Copy)]
struct TypeFlags {
    is_trivial: bool,
    total_order: bool,
}

impl TypeFlags {
    fn new() -> Self {
        // Start by assuming types are trivial and have total order
        Self {
            is_trivial: true,
            total_order: true,
        }
    }

    fn combine(&mut self, other: TypeFlags) {
        // A type is only trivial if all its components are trivial
        self.is_trivial &= other.is_trivial;
        // A type only has total order if all its components do
        self.total_order &= other.total_order;
    }
}

/// Analyzes and marks types with `IS_TRIVIAL` and `TOTAL_ORDER` flags
pub struct TypeFlagsAnalyzer {
    /// Cache of already analyzed types to handle recursion
    cache: HashMap<DefId, TypeFlags>,
    /// Set to track types currently being analyzed (for cycle detection)
    analyzing: HashSet<DefId>,
}

impl TypeFlagsAnalyzer {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            analyzing: HashSet::new(),
        }
    }

    /// Analyze all types in the HIR and set their flags
    pub fn analyze(mut self, context: &mut Context) {
        // First pass: analyze all types
        let def_ids: Vec<DefId> = context.definitions.iter().map(|(id, _)| id).collect();

        for def_id in def_ids {
            self.analyze_def(def_id, context);
        }

        // Second pass: apply the analyzed flags
        for (def_id, flags) in self.cache {
            let def = context.definitions.get_mut(def_id);

            if flags.is_trivial {
                def.flags.set(DefFlags::IS_TRIVIAL);
            } else {
                def.flags.unset(DefFlags::IS_TRIVIAL);
            }

            if flags.total_order {
                def.flags.set(DefFlags::TOTAL_ORDER);
            } else {
                def.flags.unset(DefFlags::TOTAL_ORDER);
            }
        }
    }

    fn analyze_def(&mut self, def_id: DefId, context: &Context) -> TypeFlags {
        // Check cache first
        if let Some(&flags) = self.cache.get(&def_id) {
            return flags;
        }

        // Check if we're in a cycle
        if self.analyzing.contains(&def_id) {
            // Recursive types are never trivial
            return TypeFlags {
                is_trivial: false,
                total_order: true,
            };
        }

        // Mark as analyzing
        self.analyzing.insert(def_id);

        let def = context.definitions.get(def_id);
        let mut flags = TypeFlags::new();

        // Check if type is already marked as circular
        if def.flags.contains(DefFlags::IS_CIRCULAR) {
            flags.is_trivial = false;
        }

        // Analyze based on definition kind
        match &def.kind {
            DefKind::Module(_) | DefKind::Interface(_) => {
                // Modules and interfaces are never trivial
                flags.is_trivial = false;
            }

            DefKind::Struct(s) => {
                if let Some(parent_id) = s.parent {
                    let parent_def = context.definitions.get(parent_id);
                    if !parent_def.flags.contains(DefFlags::IS_TRIVIAL) {
                        flags.is_trivial = false;
                    }
                    if !parent_def.flags.contains(DefFlags::TOTAL_ORDER) {
                        flags.total_order = false;
                    }
                }
                for member in &s.members {
                    let member_flags = self.analyze_type(&member.ty, context);
                    flags.combine(member_flags);
                }
            }

            DefKind::Union(u) => {
                // A union is trivial if all its variants are trivial
                for variant in &u.variants {
                    let variant_flags = self.analyze_type(&variant.ty, context);
                    flags.combine(variant_flags);
                }
            }

            DefKind::Valuetype(v) => {
                if let Some(parent_id) = v.parent {
                    let parent_def = context.definitions.get(parent_id);
                    if !parent_def.flags.contains(DefFlags::IS_TRIVIAL) {
                        flags.is_trivial = false;
                    }
                    if !parent_def.flags.contains(DefFlags::TOTAL_ORDER) {
                        flags.total_order = false;
                    }
                }
                for member in &v.members {
                    let member_flags = self.analyze_type(&member.ty, context);
                    flags.combine(member_flags);
                }
            }

            DefKind::Alias(a) => {
                // An alias has the same properties as its target type
                flags = self.analyze_type(&a.ty, context);
            }

            DefKind::Except(e) => {
                // Exceptions are like structs
                for member in &e.members {
                    let member_flags = self.analyze_type(&member.ty, context);
                    flags.combine(member_flags);
                }
            }

            DefKind::Enum(_)
            | DefKind::Bitmask(_)
            | DefKind::Const(_)
            | DefKind::Annotation(_)
            | DefKind::Decl(_)
            | DefKind::Bitset(_) => {
                // These are trivial and/or don't affect type properties
            }
        }

        // Remove from analyzing set
        self.analyzing.remove(&def_id);

        // Cache the result
        self.cache.insert(def_id, flags);

        flags
    }

    fn analyze_type(&mut self, ty: &Ty, context: &Context) -> TypeFlags {
        let mut flags = TypeFlags::new();

        match &ty.kind {
            TyKind::Primitive(prim) => {
                match prim {
                    PrimitiveTy::Float32 | PrimitiveTy::Float64 => {
                        // Floating point types don't have total order (NaN)
                        flags.total_order = false;
                    }
                    _ => {
                        // Other primitives are trivial and have total order
                    }
                }
            }

            TyKind::String { .. } => {
                // Strings are not trivial (heap allocated)
                flags.is_trivial = false;
            }

            TyKind::Sequence { ty: inner, .. } => {
                // Sequences are not trivial (heap allocated)
                flags.is_trivial = false;
                // Check element type
                let elem_flags = self.analyze_type(inner, context);
                flags.combine(elem_flags);
            }

            TyKind::Array { ty: inner, .. } => {
                // Arrays might be trivial if their element type is
                // Check element type
                let elem_flags = self.analyze_type(inner, context);
                flags.combine(elem_flags);
            }

            TyKind::Map { key, elem, .. } => {
                // Maps are never trivial
                flags.is_trivial = false;
                // Check key and value types for total order
                let key_flags = self.analyze_type(key, context);
                let val_flags = self.analyze_type(elem, context);
                flags.combine(key_flags);
                flags.combine(val_flags);
            }

            TyKind::Adt(def_id) => {
                // Analyze the referenced type
                flags = self.analyze_def(*def_id, context);
            }

            TyKind::Any | TyKind::Fixed | TyKind::Null => {
                // These types are not trivial
                flags.is_trivial = false;
            }
        }

        flags
    }
}

/// Analyzes and marks types with `IS_TRIVIAL` and `TOTAL_ORDER` flags.
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let analyzer = TypeFlagsAnalyzer::new();
    analyzer.analyze(&mut hir.context);
    hir
}
