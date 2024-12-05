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

#[allow(clippy::wildcard_imports)]
use crate::syntax::*;

pub trait Visitor<'a> {
    fn visit_definition(&mut self, def: &'a Definition) {}

    fn visit_annotation_def(&mut self, def: &'a AnnotationDef) {}

    fn visit_annotation_field(&mut self, def: &'a AnnotationField) {}

    fn visit_annotation_appl(&mut self, def: &'a AnnotationAppl) {}

    fn visit_annotation_arg(&mut self, def: &'a AnnotationArg) {}

    fn visit_module(&mut self, module: &'a ModuleDef) {
        for def in &module.defs {
            self.visit_definition(def);
        }
    }

    fn visit_struct(&mut self, def: &'a StructDef) {
        for mem in &def.members {
            self.visit_struct_field(mem);
        }
    }

    fn visit_struct_field(&mut self, def: &'a Field) {
        self.visit_type(&def.ty);
        self.visit_ident(&def.name);
    }

    fn visit_union(&mut self, def: &'a UnionDef) {
        self.visit_discriminant(&def.disc);
        for var in &def.fields {
            self.visit_union_variant(var);
        }
    }

    fn visit_discriminant(&mut self, def: &'a Discriminator) {}

    fn visit_union_variant(&mut self, def: &'a UnionField) {
        for label in &def.labels {
            self.visit_union_label(label);
        }
    }

    fn visit_union_label(&mut self, def: &'a Label) {}

    fn visit_enum(&mut self, def: &'a EnumDef) {
        for var in &def.fields {
            self.visit_enum_variant(var);
        }
    }

    fn visit_enum_variant(&mut self, def: &'a Enumerator) {
        for ann in &def.annotations {
            self.visit_annotation_appl(ann);
        }
        self.visit_ident(&def.name);
    }

    fn visit_numeric(&mut self, num: &'a Numeric) {}

    fn visit_bitmask(&mut self, bitmask: &'a BitmaskDef) {
        for ann in &bitmask.annotations {
            self.visit_annotation_appl(ann);
        }
        for bit in &bitmask.bits {
            self.visit_bitmask_bit(bit);
        }
    }

    fn visit_bitmask_bit(&mut self, bit: &'a Bit) {}

    fn visit_ident(&mut self, ident: &'a Ident) {}

    fn visit_type(&mut self, ident: &'a Type) {}
}

pub trait Visit {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V);
}

impl<T: Visit> Visit for Option<T> {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V) {
        if let Some(v) = self {
            v.visit(visitor);
        }
    }
}
