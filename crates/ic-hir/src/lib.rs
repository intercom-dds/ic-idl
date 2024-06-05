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

#![allow(unused, dead_code)]

use ic_alloc::arena::Arena;
use ic_alloc::interner::Interner;

// mod annotation;
pub mod visit;

// TODO: some id that identifies the source file this belongs to
pub type NodeId = ic_alloc::arena::Id<Type>;

intercom_cts::bitmask! {
    #[derive(Copy, Clone)]
    pub StructFlags: u32 {
        IS_KEY = 0,
        IS_MUST_UNDERSTAND = 1,
    }
}

intercom_cts::bitmask! {
    #[derive(Copy, Clone)]
    pub ModuleFlags: u32 {}
}

intercom_cts::bitmask! {
    #[derive(Copy, Clone)]
    pub EnumFlags: u32 {}
}

pub struct SourceLocation {}

pub struct Pool {
    arena: Arena<Type>,
    interner: Interner,
}

pub enum Type {
    Struct(Struct),
    Module(Module),
}

pub struct ModuleInner {
    flags: ModuleFlags,
    definitions: Vec<Type>,
}

// Maybe?
pub type Module = Node<ModuleInner>;

// pub struct Module(Node<ModuleInner>);

impl Module {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn annotations(&self) -> &[()] {
        &self.annotations
    }

    pub fn flags(&self) -> ModuleFlags {
        self.data.flags
    }
}

pub struct Node<K> {
    name: String,
    annotations: Vec<()>,
    scope: Option<NodeId>,
    data: K,
}

struct EnumInner {
    pub flags: EnumFlags,
    pub value: (),
}

pub struct Enum(Node<EnumInner>);

pub struct Union {
    pub name: String,
    pub annotations: Vec<()>,
    pub scope: Option<NodeId>,
    pub flags: StructFlags,
    pub discriminator: NodeId,
}

pub struct Variant {
    pub name: String,
    pub annotations: Vec<()>,
    pub scope: Option<NodeId>,
    pub ty: NodeId,
}

pub struct Struct {
    pub name: String,
    pub annotations: Vec<()>,
    pub parent: Option<NodeId>,
    pub scope: Option<NodeId>,
    pub flags: StructFlags,
}

impl Struct {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent(&self) -> Option<&Struct> {
        None
    }

    pub fn annotations(&self) -> &[()] {
        &self.annotations
    }

    pub fn flags(&self) -> StructFlags {
        self.flags
    }
}

pub struct StructMember {
    pub name: String,
    pub ty: NodeId,
    pub annotations: Vec<()>,
}
