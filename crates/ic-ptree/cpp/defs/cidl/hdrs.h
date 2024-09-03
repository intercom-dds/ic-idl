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

#pragma once

#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <string_view>
#include <vector>

#include "cidl/commandline.h"
#include "cidl/idl_parser.h"
#include "cidl/ptree_ffi.h"

namespace intercom::cidl {

enum ExtensibilityKind {
    FINAL_EXTENSIBILITY,
    EXTENSIBLE_EXTENSIBILITY,
    MUTABLE_EXTENSIBILITY,
};

std::string trim_include_name(std::filesystem::path name, bool trim_absolute);

void ptree_dump(const parse_result* result);

inline void code_gen_json_schema(const parse_result*) {}

void code_gen_json(const parse_result* result, ic_list_t* list);

void code_gen_dds_cplpl(const parse_result* result);

void code_gen_dds_cplpl(const parse_result* result, const Config& options, const char* destination);

void code_gen_python(const parse_result* result, ic_list_t* list);

void code_gen_rust(const parse_result* result, ic_list_t* list);

void code_gen_proto(const parse_result* result, ic_list_t* list);

void code_gen_idl(const parse_result* result, ic_list_t* list);

void generate_json_type(std::ostream& stream, const ptree* tree);

void emit_docs(struct memf* f, const ptree* obj);

void emit_post_docs(struct memf* f, const ptree* obj);

std::string cpp_type_name(const ptree* node, const ptree* context);

void gen_cpp_type_info(struct memf* memf, const ptree* obj, std::string_view funcname);

void get_type_library(const ptree* obj, unsigned char** cdr_typedef, size_t* len);

std::string get_type_id(const ptree* obj);

using PtreeSeq = std::vector<const ptree*>;
using PtreeSeqSeq = std::vector<PtreeSeq>;

PtreeSeqSeq ptree_build_order(const ptree* obj);

/// Duplicates the entire tree and all related state.
parse_result clone_tree(const parse_result* result);

std::string copyright_header(const std::string& comment_str = "//");

std::string tolower(std::string res);

}  // namespace intercom::cidl
