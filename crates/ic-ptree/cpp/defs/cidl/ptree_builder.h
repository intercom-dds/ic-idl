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

#include <stdint.h>

#include "cidl/constants.h"

#ifdef __cplusplus
extern "C" {
#endif

struct parser_state;

const char* get_symbol(struct parser_state*, const char* name);

struct identifier create_identifier(struct parser_state*, const char* name);

struct ptree* append_node(struct parser_state*, struct ptree* list, struct ptree* node);

struct ptree* append_enum_node(struct parser_state*, struct ptree* list, struct ptree* node);

struct declarator* append_decl(struct declarator* list, struct declarator* decl);

struct declarator*
create_decl(struct parser_state*, struct identifier ident, struct ptree* annotations);

void push_context(struct parser_state*, struct ptree* p);

int register_node(struct parser_state*, struct ptree* p);

int register_node_dcl(struct parser_state*, struct ptree* p);

struct ptree* lookup_node(struct parser_state*, struct identifier ident);

struct ptree* lookup_type(struct parser_state*, struct identifier ident);

struct ptree* pop_context(struct parser_state*);

struct ptree* duplicate_node(struct parser_state*, const struct ptree* node);

struct ptree* duplicate_tree(struct parser_state*, const struct ptree* node);

void create_include_start(struct parser_state*, struct identifier ident);

struct ptree*
create_array_type(struct parser_state*, struct declarator* declarator, struct ptree* type);

struct ptree* create_include_finish(struct parser_state*, struct ptree* members);

void create_module_start(struct parser_state*, struct identifier ident);

struct ptree* create_module_finish(struct parser_state*, struct ptree* members);

const struct numeric* lookup_value(struct parser_state*, struct identifier ident);

const struct numeric*
create_value_node(struct parser_state*, const struct numeric* value, struct ptree* members);

struct ptree* create_const_node(
    struct parser_state*,
    struct declarator* decl,
    struct ptree* type,
    const struct numeric* value
);

struct ptree*
create_sequence(struct parser_state*, struct ptree* element_type, const struct numeric* bound);

struct ptree* create_string(struct parser_state*, const struct numeric* bound);

struct ptree* create_wstring(struct parser_state*, const struct numeric* bound);

struct ptree*
create_fixed(struct parser_state*, const struct numeric* bound1, const struct numeric* bound2);

struct numeric* new_numeric(struct parser_state*, enum numeric_kind kind);

const struct numeric* create_bool(struct parser_state*, int value);

const struct numeric* create_char(struct parser_state*, char value);

const struct numeric* create_i64(struct parser_state*, int64_t value, int base);

const struct numeric* create_u64(struct parser_state*, uint64_t value, int base);

const struct numeric* create_str(struct parser_state*, const char* value);

const struct numeric* create_float(struct parser_state*, float value);

const struct numeric* create_double(struct parser_state*, double value);

struct ptree*
create_struct_start(struct parser_state*, struct identifier ident, struct ptree* parent);

struct ptree* create_struct_finish(struct parser_state*, struct ptree* members);

struct ptree* create_struct_dcl(struct parser_state*, struct identifier ident);

struct ptree* create_union_start(struct parser_state*, struct identifier ident);

struct ptree*
create_union_finish(struct parser_state*, struct ptree* discriminator, struct ptree* members);

struct ptree* create_union_dcl(struct parser_state*, struct identifier ident);

struct ptree* create_member(
    struct parser_state*,
    struct declarator* declarators,
    struct ptree* type,
    struct ptree* annotations
);

struct ptree* create_union_member(
    struct parser_state*,
    struct ptree* value,
    struct ptree* cases,
    struct ptree* annotations
);

struct ptree* create_case_label(struct parser_state*, const struct numeric* value);

struct ptree* create_default_case(struct parser_state*);

struct ptree* create_null_node(struct parser_state*);

struct ptree* create_enum(struct parser_state*, struct identifier ident, struct ptree* values);

struct ptree*
create_enum_value(struct parser_state*, struct identifier ident, const struct numeric* value);

struct ptree* create_type(struct parser_state*, struct declarator* declarators, struct ptree* type);

struct ptree* create_native_type(struct parser_state*, struct identifier ident);

void create_exception_start(struct parser_state*, struct identifier ident);

struct ptree* create_exception_finish(struct parser_state*, struct ptree* members);

struct ptree* create_interface_dcl(struct parser_state*, struct identifier ident, int is_local);

void create_interface_start(
    struct parser_state*,
    struct identifier ident,
    struct declarator* parents,
    int is_local
);

struct ptree* create_interface_finish(struct parser_state*, struct ptree* members);

struct ptree* annotate(struct parser_state*, struct ptree* node, struct ptree* annotations);

struct ptree* annotate_alias(struct parser_state*, struct ptree* node, struct ptree* annotations);

struct ptree* annotate_list(struct parser_state*, struct ptree* node, struct ptree* annotations);

struct ptree* annotate_last(struct parser_state*, struct ptree* node, struct ptree* annotations);

struct ptree* create_interface_op(
    struct parser_state*,
    struct identifier ident,
    struct ptree* params,
    struct ptree* retval,
    struct declarator* raises
);

struct ptree*
create_param_dcl(struct parser_state*, struct declarator* decl, struct ptree* type, int kind);

struct ptree* create_attribute(
    struct parser_state*,
    struct declarator* decl,
    struct ptree* type,
    struct declarator* getraises,
    struct declarator* setraises,
    int readonly
);

struct ptree* create_map(
    struct parser_state*,
    struct ptree* key_type,
    struct ptree* element_type,
    const struct numeric* bound
);

struct ptree* create_bitset(
    struct parser_state*,
    struct identifier ident,
    struct ptree* fields,
    struct ptree* type
);

struct ptree* create_bitfield(
    struct parser_state*,
    struct declarator* declarators,
    const struct numeric* bits,
    struct ptree* type
);

struct ptree* create_bitmask(struct parser_state*, struct identifier ident, struct ptree* values);

struct ptree*
create_bitmask_value(struct parser_state*, struct identifier ident, const struct numeric* value);

void create_annotation_dcl_start(struct parser_state*, struct identifier ident);

struct ptree* create_annotation_dcl_finish(struct parser_state*, struct ptree* members);

struct ptree* create_annotation_member(
    struct parser_state*,
    struct declarator* decl,
    struct ptree* type,
    const struct numeric* default_value
);

void create_annotation_start(struct parser_state*, struct identifier ident);

struct ptree* create_annotation_finish(struct parser_state*, struct ptree* params);

struct ptree*
create_annotation_param(struct parser_state*, struct identifier ident, const struct numeric* value);

struct ptree* create_valuetype_dcl(struct parser_state*, struct identifier ident);

struct ptree*
create_valuetype_start(struct parser_state*, struct identifier ident, struct ptree* parent, struct ptree* interface);

struct ptree* create_valuetype_finish(struct parser_state*, struct ptree* members);

struct ptree* create_valuetype_factory(
    struct parser_state*,
    struct identifier ident,
    struct ptree* params,
    struct declarator* raises
);

struct ptree*
create_valuetype_factory_param(struct parser_state*, struct declarator* decl, struct ptree* type);

struct ptree* create_valuetype_member(
    struct parser_state*,
    struct declarator* declarators,
    struct ptree* type,
    int is_public
);

struct declarator*
append_array_size(struct parser_state*, struct declarator* decl, const struct numeric* value);

struct declarator*
set_array_bounds(struct parser_state*, struct declarator* decl, struct declarator* bounds);

void validate_tree(struct parser_state*, struct ptree* node);

void format_doxy_comments(struct parser_state*, struct ptree* tree);

struct ptree* try_lookup_node(struct parser_state*, const char* name, const enum node_kind kind[]);

struct ptree* create_node(struct parser_state*, enum node_kind kind, struct identifier ident);

struct ptree* create_doc(struct parser_state*, struct identifier ident, int post_comment);

struct ptree* merge_members(struct parser_state*, struct ptree* node, struct ptree* members);

#ifdef __cplusplus
}

#  include <iostream>
#  include <list>
#  include <map>
#  include <memory>
#  include <set>
#  include <string>
#  include <vector>

#  include "cidl/ptree.h"

extern "C" struct parser_state {
    ptree* lookup_node(const char* name) const;

    long long enum_counter{0};
    std::vector<std::vector<ptree*>> context;
    std::vector<ptree*> include_context;
    std::map<std::string, ptree*> type_map;
    std::map<std::string, ptree*> type_dcl_map;
    std::vector<std::shared_ptr<ptree>> allocated_nodes;
    std::vector<std::shared_ptr<declarator>> allocated_decl;
    std::set<std::string> symbol_map;
    std::list<numeric> numeric_map;
    ptree top_level;
};

#  define ERR std::cerr

#endif
