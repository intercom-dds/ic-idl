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

fn main() {
    cc::Build::new()
        .cpp(true)
        .includes([
            "../ic-ptree/cpp/defs",
            "../../external/fmt/defs",
            "../../external/utils/defs",
            "../../library/cpp/defs",
        ])
        .define("FMT_HEADER_ONLY", "1")
        .define("FMT_CONSTEVAL", "")
        .files([
            "cpp/src/code_gen.cpp",
            "cpp/src/commandline.cpp",
            "cpp/src/constants.cpp",
            "cpp/src/idl_parser.cpp",
            "cpp/src/memf.cpp",
            "cpp/src/ptree_builder.cpp",
            "cpp/src/ptree_dump.cpp",
            "cpp/src/ptree_ffi.cpp",
            "cpp/src/ptree_helpers.cpp",
            "cpp/src/symbols.cpp",
        ])
        .compile("ic_codegen_ptree");
}
