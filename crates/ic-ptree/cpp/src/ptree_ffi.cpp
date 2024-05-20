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

#include "cidl/ptree_ffi.h"

#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"

ic_parse_result_t* ic_parse_idl(const char* input) {
    intercom::cidl::IdlParser idl_parser;
    idl_parser.run(input);

    auto result = new intercom::cidl::parse_result(idl_parser.result());
    return reinterpret_cast<ic_parse_result_t*>(result);
}

void ic_parse_free(ic_parse_result_t* result) {
    delete reinterpret_cast<intercom::cidl::parse_result*>(result);
}

ic_parse_result_t* ic_ptree_merge(const ic_parse_result_t** result) {
    std::vector<intercom::cidl::parse_result> to_merge;
    for (auto tree = result; *tree; ++tree) {
        auto native = reinterpret_cast<const intercom::cidl::parse_result*>(*tree);
        to_merge.emplace_back(*native);
    }

    auto merged = new intercom::cidl::parse_result(intercom::cidl::merge_results(to_merge));
    return reinterpret_cast<ic_parse_result_t*>(merged);
}

void ic_ast_dump(const ic_parse_result_t* result) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    ast_dump(res);
}

void ic_codegen_proto(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_proto(res, destination);
}

void ic_codegen_java(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_java(res, destination);
}
