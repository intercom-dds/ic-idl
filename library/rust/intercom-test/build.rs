// Copyright 2026 KONGSBERG
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

use intercom_build::Codegen;

fn main() {
    Codegen::new("test_idl")
        .input([
            "idl/bitmask.idl",
            "idl/bounds.idl",
            "idl/cdr1.idl",
            "idl/cdr2.idl",
            "idl/cdr2_corner_cases.idl",
            // "idl/coalesce.idl",  // TODO: codegen bug - non-exhaustive match for union
            "idl/collision.idl",
            // "idl/complex.idl",  // TODO: 'any' and 'Object' CORBA types not supported
            "idl/crab.idl",
            "idl/default.idl",
            "idl/derive.idl",
            // "idl/dynamic.idl",  // TODO: codegen bug - @bit_bound(64) generates u32 instead of u64
            "idl/escaped.idl",
            "idl/inc/a.idl",
            "idl/interface.idl",
            "idl/modules.idl",
            "idl/name.with.dot.idl",
            "idl/nested.idl",
            "idl/plain.idl",
            "idl/repository.idl",
            // "idl/scoped.idl",  // TODO: codegen bug - union variant naming
            "idl/string.idl",
        ])
        .generate()
        .unwrap();

    Codegen::new("test_no_rename")
        .input(["idl/no_rename.idl"])
        .no_rename(true)
        .generate()
        .unwrap();
}
