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
#include "cidl/ptree_builder.h"

uint32_t ic_error_count(const ic_parse_result_t* result) {
    return reinterpret_cast<const intercom::cidl::parse_result*>(result)->error_count;
}

const char* ic_parse_error(const ic_parse_result_t* result) {
    return reinterpret_cast<const intercom::cidl::parse_result*>(result)->msg.c_str();
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

void ic_codegen_cpp(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_dds_cplpl(res, destination);
}

void ic_codegen_python(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_python(res, destination);
}

void ic_codegen_idl(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_idl(res, destination);
}

void ic_codegen_json(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_json(res, destination);
}

void ic_codegen_json_schema(const ic_parse_result_t* result, const char* destination) {
    auto res = reinterpret_cast<const intercom::cidl::parse_result*>(result);
    intercom::cidl::code_gen_json_schema(res);
}

static void register_primitives(parser_state* state) {
    register_node(state, &any_type);
    register_node(state, &object_type);
    register_node(state, &boolean_type);
    register_node(state, &int8_type);
    register_node(state, &octet_type);
    register_node(state, &char_type);
    register_node(state, &wchar_type);
    register_node(state, &short_type);
    register_node(state, &ushort_type);
    register_node(state, &long_type);
    register_node(state, &ulong_type);
    register_node(state, &longlong_type);
    register_node(state, &ulonglong_type);
    register_node(state, &float_type);
    register_node(state, &double_type);
    register_node(state, &ldouble_type);
    register_node(state, &fixed_type);
    register_node(state, &unbounded_string_type);
    register_node(state, &unbounded_wstring_type);
}

ic_parse_result_t* ic_parse_w_state(ic_parser_callback_t callback, void* user_data) {
    intercom::cidl::IdlParser parser;
    parser.run([=](auto state) {
        register_primitives(state);
        return callback(state, user_data);
    });
    auto res = new intercom::cidl::parse_result(parser.result());
    return reinterpret_cast<ic_parse_result_t*>(res);
}
