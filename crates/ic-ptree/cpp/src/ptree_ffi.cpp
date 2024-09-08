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

uint32_t ic_error_count(const ic_parse_result_t* result) {
    return reinterpret_cast<const parse_result*>(result)->error_count;
}

const char* ic_parse_error(const ic_parse_result_t* result) {
    return reinterpret_cast<const parse_result*>(result)->msg.c_str();
}

void ic_parse_free(ic_parse_result_t* result) {
    delete reinterpret_cast<parse_result*>(result);
}

ic_parse_result_t* ic_ptree_merge(const ic_parse_result_t** result) {
    std::vector<parse_result> to_merge;
    for (auto tree = result; *tree; ++tree) {
        auto native = reinterpret_cast<const parse_result*>(*tree);
        to_merge.emplace_back(*native);
    }

    auto merged = new parse_result(intercom::cidl::merge_results(to_merge));
    return reinterpret_cast<ic_parse_result_t*>(merged);
}

void ic_ptree_dump(const ic_parse_result_t* result) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::ptree_dump(res);
}

void ic_codegen_proto(const ic_parse_result_t* result, ic_list_t* list) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_proto(res, list);
}

void ic_codegen_cpp(const ic_parse_result_t* result, struct cpp_options_t options) {
    intercom::cidl::Config config;
    if (options.header_ext) {
        config.header_subfolder = options.header_ext;
    }
    if (options.header_postfix) {
        config.cpp_header_postfix = options.header_postfix;
    }
    if (options.dll_export) {
        config.dll_exp_sym = options.dll_export;
    }
    config.cpp_scoped_enums = options.scoped_enums;
    config.cpp_access_functions = options.access_functions;
    config.cpp_no_stream_op = options.no_stream_op;
    config.use_fmtlib = options.use_fmt;

    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_dds_cplpl(res, config);
}

void ic_codegen_python(const ic_parse_result_t* result, ic_list_t* list) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_python(res, list);
}

void ic_codegen_rust(const ic_parse_result_t* result, ic_list_t* list) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_rust(res, list);
}

void ic_codegen_idl(const ic_parse_result_t* result, ic_list_t* list) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_idl(res, list);
}

void ic_codegen_json(const ic_parse_result_t* result, ic_list_t* list) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_json(res, list);
}

void ic_codegen_json_schema(const ic_parse_result_t* result, const char*) {
    auto res = reinterpret_cast<const parse_result*>(result);
    intercom::cidl::code_gen_json_schema(res);
}
