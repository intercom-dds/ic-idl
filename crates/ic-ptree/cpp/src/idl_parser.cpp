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

#include "cidl/idl_parser.h"

#include <algorithm>
#include <cassert>
#include <filesystem>
#include <functional>
#include <iostream>
#include <memory>
#include <set>
#include <sstream>
#include <string>

#include "cidl/constants.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_ffi.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

extern "C" {

struct ptree* annotation_type_id;
struct ptree* annotation_type_autoid;
struct ptree* annotation_type_optional;
struct ptree* annotation_type_position;
struct ptree* annotation_type_value;
struct ptree* annotation_type_empty;
struct ptree* annotation_type_extensibility;
struct ptree* annotation_type_final;
struct ptree* annotation_type_mutable;
struct ptree* annotation_type_appendable;
struct ptree* annotation_type_shared;
struct ptree* annotation_type_key;
struct ptree* annotation_type_must_understand;
struct ptree* annotation_type_default;
struct ptree* annotation_type_default_literal;
struct ptree* annotation_type_range;
struct ptree* annotation_type_min;
struct ptree* annotation_type_max;
struct ptree* annotation_type_unit;
struct ptree* annotation_type_bit_bound;
struct ptree* annotation_type_external;
struct ptree* annotation_type_nested;
struct ptree* annotation_type_verbatim;
struct ptree* annotation_type_service;
struct ptree* annotation_type_topic;
struct ptree* annotation_type_dds_service;
struct ptree* annotation_type_dds_request_topic;
struct ptree* annotation_type_dds_reply_topic;
struct ptree* annotation_type_oneway;
struct ptree* annotation_type_ami;
struct ptree* annotation_type_bitset_old;
struct ptree* annotation_type_bit_bound_old;
struct ptree* annotation_type_must_understand_old;
struct ptree* annotation_type_minimum_type_check_old;
struct ptree* annotation_type_hashid;
struct ptree* annotation_type_default_nested;
struct ptree* annotation_type_ignore_literal_names;
struct ptree* annotation_type_try_construct;
struct ptree* annotation_type_non_serialized;
struct ptree* annotation_type_data_representation;
struct ptree* annotation_type_doc;
struct ptree* annotation_type_merge;
struct ptree* annotation_type_const;
struct ptree* annotation_type_static;
struct ptree* annotation_type_derive;
struct ptree* annotation_type_ext_rename;
struct ptree* annotation_type_ext_builder;
struct ptree* annotation_type_ext_doc;
struct ptree* annotation_type_ext_minimum_type_check;
struct ptree* annotation_type_ext_suppress;
struct ptree* annotation_type_ext_no_constructor;
struct ptree* annotation_type_ext_no_serializer;
struct ptree* annotation_type_ext_listener;
struct ptree* annotation_type_ext_length_bit_bound;
struct ptree* annotation_type_ext_value_offset;
struct ptree* annotation_type_ext_length_value_offset;
struct ptree* annotation_type_ext_repeat_count;
struct ptree* annotation_type_ext_vmf_xri;
struct ptree* annotation_type_ext_vmf_decimal;
struct ptree* annotation_type_ext_string_constants;
struct ptree* annotation_type_ext_jaus_presence_vector;
struct ptree* annotation_type_ext_jaus_integer;
struct ptree* annotation_type_ext_jaus_integer_function;
struct ptree* annotation_type_ext_protobuf_type;
struct ptree* annotation_type_jaus;

const char* node_kind_str(node_kind kind) {
    switch (kind) {
    case N_UNDEF:
        return "N_UNDEF";
    case N_INCLUDE:
        return "N_INCLUDE";
    case N_PRIMITIVE:
        return "N_PRIMITIVE";
    case N_NATIVE:
        return "N_NATIVE";
    case N_MODULE:
        return "N_MODULE";
    case N_STRUCT:
        return "N_STRUCT";
    case N_UNION:
        return "N_UNION";
    case N_VALUETYPE:
        return "N_VALUETYPE";
    case N_INTERFACE:
        return "N_INTERFACE";
    case N_EXCEPTION:
        return "N_EXCEPTION";
    case N_ENUM:
        return "N_ENUM";
    case N_BITSET:
        return "N_BITSET";
    case N_BITMASK:
        return "N_BITMASK";
    case N_CASE:
        return "N_CASE";
    case N_NULL:
        return "N_NULL";
    case N_MEMBER:
        return "N_MEMBER";
    case N_PROTOTYPE:
        return "N_PROTOTYPE";
    case N_SEQUENCE:
        return "N_SEQUENCE";
    case N_MAP:
        return "N_MAP";
    case N_ARRAY:
        return "N_ARRAY";
    case N_STRING:
        return "N_STRING";
    case N_FIXED:
        return "N_FIXED";
    case N_ALIAS:
        return "N_ALIAS";
    case N_CONST:
        return "N_CONST";
    case N_ANNOTATION_DEF:
        return "N_ANNOTATION_DEF";
    case N_ANNOTATION:
        return "N_ANNOTATION";
    }
    return "";
}

const char* numeric_kind_str(numeric_kind kind) {
    switch (kind) {
    case UNDEF_KIND:
        return "UNDEF_KIND";
    case BOOLEAN_KIND:
        return "BOOLEAN_KIND";
    case INT8_KIND:
        return "INT8_KIND";
    case OCTET_KIND:
        return "OCTET_KIND";
    case SHORT_KIND:
        return "SHORT_KIND";
    case USHORT_KIND:
        return "USHORT_KIND";
    case LONG_KIND:
        return "LONG_KIND";
    case ULONG_KIND:
        return "ULONG_KIND";
    case LONGLONG_KIND:
        return "LONGLONG_KIND";
    case ULONGLONG_KIND:
        return "ULONGLONG_KIND";
    case FLOAT_KIND:
        return "FLOAT_KIND";
    case DOUBLE_KIND:
        return "DOUBLE_KIND";
    case CHAR_KIND:
        return "CHAR_KIND";
    case STRING_KIND:
        return "STRING_KIND";
    case PTREE_KIND:
        return "PTREE_KIND";
    }
    return "";
}
}

using FileList = std::vector<std::pair<std::filesystem::path, std::filesystem::path>>;

static std::map<std::string, ptree**> initialize_builtin_annotation_map() {
    std::map<std::string, ptree**> res;

    res["id"] = &annotation_type_id;
    res["autoid"] = &annotation_type_autoid;
    res["optional"] = &annotation_type_optional;
    res["position"] = &annotation_type_position;
    res["value"] = &annotation_type_value;
    res["empty"] = &annotation_type_empty;
    res["extensibility"] = &annotation_type_extensibility;
    res["final"] = &annotation_type_final;
    res["mutable"] = &annotation_type_mutable;
    res["appendable"] = &annotation_type_appendable;
    res["shared"] = &annotation_type_shared;
    res["key"] = &annotation_type_key;
    res["must_understand"] = &annotation_type_must_understand;
    res["default"] = &annotation_type_default;
    res["default_literal"] = &annotation_type_default_literal;
    res["range"] = &annotation_type_range;
    res["min"] = &annotation_type_min;
    res["max"] = &annotation_type_max;
    res["unit"] = &annotation_type_unit;
    res["bit_bound"] = &annotation_type_bit_bound;
    res["external"] = &annotation_type_external;
    res["nested"] = &annotation_type_nested;
    res["verbatim"] = &annotation_type_verbatim;
    res["service"] = &annotation_type_service;
    res["topic"] = &annotation_type_topic;
    res["DDSService"] = &annotation_type_dds_service;
    res["DDSRequestTopic"] = &annotation_type_dds_request_topic;
    res["DDSReplyTopic"] = &annotation_type_dds_reply_topic;
    res["oneway"] = &annotation_type_oneway;
    res["ami"] = &annotation_type_ami;
    res["bitset"] = &annotation_type_bitset_old;
    res["bitbound"] = &annotation_type_bit_bound_old;
    res["mustunderstand"] = &annotation_type_must_understand_old;
    res["hashid"] = &annotation_type_hashid;
    res["doc"] = &annotation_type_doc;
    res["merge"] = &annotation_type_merge;
    res["default_nested"] = &annotation_type_default_nested;
    res["ignore_literal_names"] = &annotation_type_ignore_literal_names;
    res["try_construct"] = &annotation_type_try_construct;
    res["non_serialized"] = &annotation_type_non_serialized;
    res["data_representation"] = &annotation_type_data_representation;
    res["const"] = &annotation_type_const;
    res["static"] = &annotation_type_static;
    res["derive"] = &annotation_type_derive;
    res["ext::rename"] = &annotation_type_ext_rename;
    res["ext::builder"] = &annotation_type_ext_builder;
    res["ext::minimum_type_check"] = &annotation_type_minimum_type_check_old;
    res["ext::doc"] = &annotation_type_ext_doc;
    res["ext::minimum_type_check"] = &annotation_type_ext_minimum_type_check;
    res["ext::suppress"] = &annotation_type_ext_suppress;
    res["ext::no_constructor"] = &annotation_type_ext_no_constructor;
    res["ext::no_serializer"] = &annotation_type_ext_no_serializer;
    res["ext::listener"] = &annotation_type_ext_listener;
    res["ext::length_bit_bound"] = &annotation_type_ext_length_bit_bound;
    res["ext::value_offset"] = &annotation_type_ext_value_offset;
    res["ext::length_value_offset"] = &annotation_type_ext_length_value_offset;
    res["ext::repeat_count"] = &annotation_type_ext_repeat_count;
    res["ext::vmf_xri"] = &annotation_type_ext_vmf_xri;
    res["ext::vmf_decimal"] = &annotation_type_ext_vmf_decimal;
    res["ext::string_constants"] = &annotation_type_ext_string_constants;
    res["ext::jaus_presence_vector"] = &annotation_type_ext_jaus_presence_vector;
    res["ext::jaus_integer"] = &annotation_type_ext_jaus_integer;
    res["ext::jaus_integer_function"] = &annotation_type_ext_jaus_integer_function;
    res["ext::protobuf_type"] = &annotation_type_ext_protobuf_type;
    res["jaus"] = &annotation_type_jaus;

    return res;
}

std::map<std::string, ptree**> g_builtin_annotation_map = initialize_builtin_annotation_map();

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
    state->type_map["::octet"] = &octet_type;
}

static void init_parser_state(parser_state* state) {
    static auto s_initial_state = []() -> parser_state {
        parser_state initial;
        register_primitives(&initial);
        clear_namespace_nodes(&initial);

        // Everything created up until this point is builtin types
        for (const auto& node : initial.allocated_nodes) {
            node->flags |= OPT_BUILTIN;
        }

        return initial;
    }();
    *state = s_initial_state;
}

static void update_incomplete_type(parser_state* state, struct ptree* node, struct ptree*& type) {
    if (type) {
        if (type->flags & OPT_DECLARATION) {
            if (type->type) {
                type = type->type;
                node->flags |= OPT_CIRCULAR;
            } else {
                std::stringstream stream;
                stream << "type \"" << type->name << "\" declared only (as \"" << node->name
                       << "\")";
                state->error() << stream.str().c_str();
            }
        }
        update_incomplete_type(state, node, type->type);
        update_incomplete_type(state, node, type->element_type);
        update_incomplete_type(state, node, type->key_type);
    }
}

static void resolve_incomplete_types(parser_state* state, struct ptree* node) {
    while (node) {
        update_incomplete_type(state, node, node->type);
        resolve_incomplete_types(state, node->members);
        node = node->next;
    }
}

static ptree* prune_annotations(struct ptree* node, struct ptree* super = nullptr) {
    if (!node) {
        return nullptr;
    }
    if (node->kind == N_ANNOTATION &&
        (node->type != annotation_type_doc || super == nullptr || super->kind != N_MODULE)) {
        return node->next;
    }
    node->next = prune_annotations(node->next, super);
    node->members = prune_annotations(node->members, node);
    return node;
}

static void generate_code(parser_state* state, struct ptree* node) {
    while (node) {
        state->include_context.emplace_back(node->file_name, node->included_from);
        push_context(state, node);
        generate_code(state, node->members);
        state->include_context.pop_back();
        pop_context(state);
        node = node->next;
    }
}

static void tree_modules_add(const ptree* tree, std::set<std::string>& modules) {
    while (tree) {
        if (is_emit(tree, LANG_NONE)) {
            modules.insert(module_name(tree));
            tree_modules_add(tree->members, modules);
        }
        tree = tree->next;
    }
}

static void tree_modules_prune(const ptree* tree, std::set<std::string>& modules) {
    while (tree) {
        // Exclude modules that are not emit
        if (!is_emit(tree, LANG_NONE)) {
            modules.erase(module_name(tree));
        }
        // ...but keep modules that directly contain emittable modules
        if (is_emit(tree, LANG_NONE) && tree->kind != N_MODULE && tree->scope &&
            tree->scope->kind == N_MODULE) {
            modules.insert(module_name(tree));
        }
        tree_modules_prune(tree->members, modules);
        tree = tree->next;
    }
}

static void tree_modules(const ptree* tree, std::set<std::string>& modules) {
    // Add all modules that are OPT_EMIT to set
    tree_modules_add(tree, modules);
    // Remove module names that are not OPT_EMIT somewhere in the tree.
    // This is done to avoid emitting parent modules in Ada when the
    // --no-header-follow option is used and the contents of the parent
    // module is in an included file while a sub-module is in the main file
    tree_modules_prune(tree, modules);
}

static void tree_includes(const ptree* tree, std::set<const ptree*>& includes) {
    while (tree) {
        if ((tree->flags & OPT_EMIT_CODE) != 0) {
            if (tree->included_from != nullptr) {
                includes.insert(tree->included_from);
            }
            tree_includes(tree->members, includes);
        }
        tree = tree->next;
    }
}

static void register_node_in_scope(parser_state* state, ptree* node, ptree* scp) {
    std::swap(node->super, scp);
    register_node(state, node);
    std::swap(node->super, scp);
}

static void register_inherited_nodes(parser_state* state, ptree* node) {
    if (node->type || (node->kind != N_STRUCT && node->kind != N_INTERFACE)) {
        return;
    }
    for (ptree* parent = node; !parent->parents.empty();) {
        parent = base_type_of(parent->parents.front());
        for (ptree* elem : parent->members) {
            register_node_in_scope(state, elem, node);
        }
    }
}

static parse_result get_parse_result(parser_state* state) {
    resolve_incomplete_types(state, state->top_level.next);
    state->top_level.next = prune_annotations(state->top_level.next);
    format_doxy_comments(state, state->top_level.next);
    generate_code(state, state->top_level.next);
    for (std::shared_ptr<ptree>& node : state->allocated_nodes) {
        register_inherited_nodes(state, node.get());
    }
    validate_tree(state, state->top_level.next);

    if (state->top_level.next) {
        state->numeric_map.clear();
    }

    parse_result result;
    result.tree = state->top_level.next;
    result.state.reset(state);
    result.error_count = state->errors.size();

    std::stringstream msg;
    for (const auto& err : state->errors) {
        msg << err << '\n';
    }
    result.msg = msg.str();

    tree_modules(state->top_level.next, result.modules);
    tree_includes(state->top_level.next, result.includes);
    return result;
}

static void suppress_content_from_includes(parse_result& result, const FileList& input_files) {
    std::set<std::string> input_file_set;
    for (auto& file : input_files) {
        input_file_set.insert(std::filesystem::canonical(file.first).string());
    }
    std::function<void(ptree*)> filter = [&](ptree* tree) {
        if (!tree) {
            return;
        }
        for (auto node : tree) {
            if (input_file_set.find(node->file_name) == input_file_set.end()) {
                node->flags &= ~OPT_EMIT_CODE;
            }
            filter(node->members);
        }
    };
    filter(const_cast<ptree*>(result.tree));
    result.includes.clear();
    tree_includes(result.tree, result.includes);
    tree_modules(result.tree, result.modules);
}

static void update_include_paths(parse_result& result, const FileList& input_files) {
    std::map<std::string, std::string> path_map;
    for (auto& file : input_files) {
        path_map.emplace(std::filesystem::canonical(file.first).string(), file.second.string());
    }
    std::function<void(ptree*)> filter = [&](ptree* tree) {
        if (!tree) {
            return;
        }
        for (auto node : tree) {
            auto it = path_map.find(node->file_name);
            if (it != path_map.end()) {
                node->included_from->name = it->second;
            }
            filter(node->members);
        }
    };
    filter(const_cast<ptree*>(result.tree));
}

static void validate_consistent_types(
    const ptree* tree,
    std::map<std::string, const ptree*>& type_map,
    parse_result& result
) {
    auto validate_type = [&](const ptree* type) {
        if (type && (type->flags & OPT_DECLARATION) == 0) {
            auto name = lc_scoped_name(type);
            auto it = type_map.find(name);
            if (it != type_map.end() && it->second != type) {
                std::stringstream err;
                err << "Inconsistent type for node " << name << " of kind " << type->kind;
                if (!result.msg.empty()) {
                    result.msg += "\n";
                }
                result.msg += err.str();
                result.error_count++;
            } else {
                type_map[name] = type;
            }
        }
    };
    auto validate_node = [&](const ptree* node) {
        if (node->kind == N_CONST && (node->flags & OPT_CONST_VALUE) == 0) {
            validate_type(node);
        }
        if (node->kind == N_ALIAS || node->kind == N_STRUCT || node->kind == N_UNION ||
            node->kind == N_VALUETYPE || node->kind == N_INTERFACE) {
            validate_type(node);
        }
        validate_type(node->type);
        validate_type(node->element_type);
        validate_type(node->key_type);
    };
    for (auto node : tree) {
        validate_node(node);
        validate_consistent_types(node->members, type_map, result);
        if (node->value->_d() == PTREE_KIND) {
            validate_node(node->value->node());
            validate_consistent_types(node->value->node()->members, type_map, result);
        }
    }
}

// Update type pointers in tree to point into the main
// tree structure if they are defined there
void update_ptree_types_after_merge(parse_result& result) {
    // Update type map with pointers from merged tree
    std::function<void(ptree*)> update_type_map = [&](ptree* tree) {
        for (auto node : tree) {
            auto name = lc_scoped_name(node);
            auto& map = (node->flags & OPT_DECLARATION) != 0 ? result.state->type_dcl_map
                                                             : result.state->type_map;
            auto it = map.find(name);
            if (it != map.end() && it->second->kind == node->kind) {
                it->second = node;
            }
            update_type_map(node->members);
            update_type_map(node->generated);
        }
    };

    // Return node from type map if it exists, otherwise return input argument
    std::function<ptree*(ptree*)> lookup_type = [&](ptree* node) {
        if (node) {
            auto it = result.state->type_map.find(lc_scoped_name(node));
            if (it != result.state->type_map.end()) {
                return it->second;
            }
        }
        return node;
    };

    // Update tree with type pointers from type map if they exist
    std::set<const ptree*> seen;
    std::function<void(ptree*)> update_types = [&](ptree* tree) {
        if (seen.find(tree) != seen.end()) {
            return;
        }
        for (auto node : tree) {
            seen.insert(node);
            node->type = lookup_type(node->type);
            node->key_type = lookup_type(node->key_type);
            node->element_type = lookup_type(node->element_type);
            for (auto& parent : node->parents) {
                parent = lookup_type(parent);
                update_types(parent);
            }
            for (auto& getraise : node->getraises) {
                getraise = lookup_type(getraise);
                update_types(getraise);
            }
            for (auto& setraise : node->setraises) {
                setraise = lookup_type(setraise);
                update_types(setraise);
            }
            for (auto& bound : node->bounds) {
                if (bound->_d() == PTREE_KIND) {
                    bound.val.node(lookup_type(const_cast<ptree*>(bound.val.node())));
                }
            }
            update_types(node->type);
            update_types(node->key_type);
            update_types(node->element_type);

            update_types(node->members);
            update_types(node->generated);
            update_types(node->discriminator);
            update_types(node->annotations);
            if (node->value->_d() == PTREE_KIND) {
                auto updated = const_cast<ptree*>(node->value->node());
                if (updated->kind != N_CONST || (updated->flags & OPT_CONST_VALUE) == 0) {
                    updated = lookup_type(updated);
                }
                update_types(updated);
                node->value.val.node(updated);
            }
            std::for_each(node->parents.begin(), node->parents.end(), update_types);
            std::for_each(node->getraises.begin(), node->getraises.end(), update_types);
            std::for_each(node->setraises.begin(), node->setraises.end(), update_types);
        }
    };

    auto tree = const_cast<ptree*>(result.tree);
    update_type_map(tree);
    update_types(tree);

    // Check that there is only a single definition for each type in the tree.
    // Errors here should never happen and could be asserted, but add it to user error messages
    // to help debug any follow-on errors in the backend in release builds.
    std::map<std::string, const ptree*> type_map;
    validate_consistent_types(tree, type_map, result);
}

namespace intercom::cidl {

parse_result merge_results(std::vector<parse_result>& to_merge) {
    parse_result out;
    out.state = std::make_shared<parser_state>();
    ptree* new_tree = nullptr;

    // Inject the current built-in annotations into the new state. There may be
    // multiple definitions of the same type, so we resort to using the
    // last-registered definitions.
    for (auto node : g_builtin_annotation_map) {
        auto name = lc_scoped_name(*node.second);
        out.state->type_map[name] = *node.second;
    }

    std::map<std::string, const ptree*> seen_includes;

    // Filter nodes so that a file is only defined once
    std::function<ptree*(ptree*)> filter_includes = [&](ptree* node) -> ptree* {
        if (!node) {
            return nullptr;
        }
        ptree* prev = nullptr;
        for (auto n : node) {
            if (seen_includes.find(n->file_name) == seen_includes.end()) {
                seen_includes[n->file_name] = n->included_from;
            }
            if (seen_includes[n->file_name] != n->included_from) {
                if (prev) {
                    prev->next = n->next;
                } else {
                    node = n->next;
                }
            } else {
                prev = n;
            }
        }
        for (auto n : node) {
            n->members = filter_includes(n->members);
        }
        return node;
    };

    for (auto& to_merge_result : to_merge) {
        if (to_merge_result.tree) {
            // Take ownership of nodes
            out.state->allocated_nodes.insert(
                out.state->allocated_nodes.end(),
                to_merge_result.state->allocated_nodes.begin(),
                to_merge_result.state->allocated_nodes.end()
            );
            out.state->allocated_decl.insert(
                out.state->allocated_decl.end(),
                to_merge_result.state->allocated_decl.begin(),
                to_merge_result.state->allocated_decl.end()
            );
            out.state->type_map.insert(
                to_merge_result.state->type_map.begin(), to_merge_result.state->type_map.end()
            );
            out.state->type_dcl_map.insert(
                to_merge_result.state->type_dcl_map.begin(),
                to_merge_result.state->type_dcl_map.end()
            );

            // Update ptree and add it to the merged tree
            auto to_merge_tree = const_cast<ptree*>(to_merge_result.tree);
            to_merge_tree = filter_includes(to_merge_tree);
            new_tree = append_node(out.state.get(), to_merge_tree, new_tree);
        }

        // Merge errors
        out.error_count += to_merge_result.error_count;
        if (!to_merge_result.msg.empty()) {
            if (!out.msg.empty()) {
                out.msg += "\n";
            }
            out.msg += to_merge_result.msg;
        }
    }
    out.tree = new_tree;
    tree_modules(out.tree, out.modules);
    tree_includes(out.tree, out.includes);

    to_merge.clear();
    return out;
}
}  // namespace intercom::cidl

parser_state* ic_parser_create() {
    auto state = new parser_state();
    init_parser_state(state);
    return state;
}

parse_result* ic_parser_result(parser_state* state, ptree* tree) {
    state->top_level.next = tree;
    return new parse_result(get_parse_result(state));
}

parse_result* ic_ptree_merge(const parse_result** result) {
    std::vector<parse_result> to_merge;
    for (auto tree = result; *tree; ++tree) {
        to_merge.emplace_back(**tree);
    }

    auto merged = new parse_result(intercom::cidl::merge_results(to_merge));
    // TODO(idarcar):
    // if (no_header_follow != 0) {
    //     suppress_content_from_includes(result, expanded_files);
    // }
    update_ptree_types_after_merge(*merged);
    return merged;
}
