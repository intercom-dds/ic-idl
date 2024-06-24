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

//! Functionality for lowering an AST into a HIR.

// pub fn from_ast(ast: Vec<Definition>, interner: Interner) {
//     // TODO: we need to (somehow) insert the context into each node, while
//     // still retaining mutable access to it...
//     let ctx = Context {
//         arena: Arena::default(),
//         interner,
//     };
//
//     let mut hir = vec![];
//     let mut ty_stack = vec![];
//
//     for node in ast {
//         match node.kind {
//             ic_syntax::ItemKind::Annotation(_) => todo!(),
//             ic_syntax::ItemKind::Struct(_) => todo!(),
//             ic_syntax::ItemKind::Union(_) => todo!(),
//             ic_syntax::ItemKind::Enum(_) => todo!(),
//             ic_syntax::ItemKind::Exception(_) => todo!(),
//             ic_syntax::ItemKind::Bitmask(_) => todo!(),
//             ic_syntax::ItemKind::Bitset(_) => todo!(),
//             ic_syntax::ItemKind::Const(_) => todo!(),
//             ic_syntax::ItemKind::Typedef(_) => todo!(),
//             ic_syntax::ItemKind::Interface(_) => todo!(),
//             ic_syntax::ItemKind::Valuetype(_) => todo!(),
//             ic_syntax::ItemKind::Decl(_) => todo!(),
//             ic_syntax::ItemKind::Module(v) => {
//                 let scope = ty_stack.last().copied();
//                 let module = Module {
//                     name: node.name.name,
//                     annotations: vec![],
//                     scope,
//                     ctx: todo!(),
//                     data: ModuleInner::default(),
//                 };
//                 hir.push(Type::Module(module));
//             }
//         }
//     }
// }

