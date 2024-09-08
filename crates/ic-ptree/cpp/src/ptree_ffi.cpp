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

uint32_t ic_error_count(const parse_result* result) {
    return result->error_count;
}

const char* ic_parse_error(const parse_result* result) {
    return result->msg.c_str();
}

void ic_parse_free(parse_result* result) {
    delete result;
}

parse_result* ic_ptree_merge(const parse_result** result) {
    std::vector<parse_result> to_merge;
    for (auto tree = result; *tree; ++tree) {
        to_merge.emplace_back(**tree);
    }
    return new parse_result(intercom::cidl::merge_results(to_merge));
}

void ic_ptree_dump(const parse_result* result) {
    intercom::cidl::ptree_dump(result);
}

void ic_codegen_proto(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_proto(result, list);
}

void ic_codegen_cpp(const parse_result* result, struct cpp_options_t options) {
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

    intercom::cidl::code_gen_dds_cplpl(result, config);
}

void ic_codegen_python(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_python(result, list);
}

void ic_codegen_rust(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_rust(result, list);
}

void ic_codegen_idl(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_idl(result, list);
}

void ic_codegen_json(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_json(result, list);
}

void ic_codegen_json_schema(const parse_result* result, const char*) {
    intercom::cidl::code_gen_json_schema(result);
}
