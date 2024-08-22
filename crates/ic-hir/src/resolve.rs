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

use std::collections::{HashMap, HashSet};

use ic_syntax::visit::Visitor;
use ic_syntax::*;
use visit::{visit_module, visit_struct, visit_struct_field};

pub struct SymbolTable {}

#[derive(Default, Debug)]
struct Scope {
    fwd_decls: HashMap<String, ()>,
    types: HashMap<String, ()>,
}

// TODO: Scope or absolute names?
#[derive(Default, Debug)]
pub struct Resolver {
    modules: HashSet<String>,
    fwd_decls: HashMap<String, ()>,
    types: HashMap<String, ()>,
    scopes: Vec<Scope>,
}

impl Resolver {
    fn is_defined(&self, ty: &Type) -> bool {
        let name = util::type_name(ty);
        self.types.contains_key(&name) || self.fwd_decls.contains_key(&name)
    }
}

impl<'a> Visitor<'a> for Resolver {
    fn visit_module(&mut self, module: &'a ModuleDef) {
        self.modules.insert(module.ident.name.clone());
        visit_module(self, module);
    }

    fn visit_decl(&mut self, decl: &'a Decl) {
        self.fwd_decls.insert(decl.ident.name.clone(), ());
    }

    fn visit_struct(&mut self, def: &'a StructDef) {
        self.types.insert(def.ident.name.clone(), ());
        visit_struct(self, def);
    }

    fn visit_struct_field(&mut self, def: &'a Field) {
        if !self.is_defined(&def.ty) {
            eprintln!("type not defined");
        }
        visit_struct_field(self, def);
    }
}
