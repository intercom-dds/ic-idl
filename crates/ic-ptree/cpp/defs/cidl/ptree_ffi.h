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

#include "stdint.h"

#ifdef __cplusplus
extern "C" {
#endif

struct ic_parse_result_t;

ic_parse_result_t* ic_parse_idl(const char* input);

uint32_t ic_warning_count(const ic_parse_result_t* result);

uint32_t ic_error_count(const ic_parse_result_t* result);

const char* ic_parse_error(const ic_parse_result_t* result);

void ic_parse_free(ic_parse_result_t* result);

ic_parse_result_t* ic_ptree_merge(const ic_parse_result_t** result);

void ic_ast_dump(const ic_parse_result_t* result);

void ic_codegen_proto(const ic_parse_result_t* result, const char* destination);

void ic_codegen_java(const ic_parse_result_t* result, const char* destination);

void ic_codegen_csharp(const ic_parse_result_t* result, const char* destination);

void ic_codegen_cpp(const ic_parse_result_t* result, const char* destination);

void ic_codegen_python(const ic_parse_result_t* result, const char* destination);

#ifdef __cplusplus
}
#endif
