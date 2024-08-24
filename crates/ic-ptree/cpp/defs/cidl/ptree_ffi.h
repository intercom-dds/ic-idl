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

#include "cidl/ptree_builder.h"
#include "stdint.h"

#ifdef __cplusplus
extern "C" {
#endif

struct ic_parse_result_t;

struct ptree;

typedef struct ptree* (*ic_parser_callback_t)(struct parser_state*, void*);

struct ic_parse_result_t* ic_parse_w_state(ic_parser_callback_t callback, void* user_data);

uint32_t ic_error_count(const struct ic_parse_result_t* result);

const char* ic_parse_error(const struct ic_parse_result_t* result);

void ic_parse_free(struct ic_parse_result_t* result);

struct ic_parse_result_t* ic_ptree_merge(const struct ic_parse_result_t** result);

void ic_ptree_dump(const struct ic_parse_result_t* result);

void ic_codegen_proto(const struct ic_parse_result_t* result, const char* destination);

void ic_codegen_cpp(const struct ic_parse_result_t* result, const char* destination);

void ic_codegen_python(const struct ic_parse_result_t* result, const char* destination);

void ic_codegen_rust(const struct ic_parse_result_t* result, const char* destination);

void ic_codegen_idl(const struct ic_parse_result_t* result, const char* destination);

void ic_codegen_json(const struct ic_parse_result_t* result, const char* destination);

#ifdef __cplusplus
}
#endif
