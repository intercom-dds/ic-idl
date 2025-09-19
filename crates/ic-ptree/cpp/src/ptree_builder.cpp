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

#include "cidl/ptree_builder.h"

#include <algorithm>
#include <cstddef>
#include <cstring>
#include <iostream>
#include <map>
#include <set>
#include <string>
#include <utility>
#include <vector>

#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

extern std::map<std::string, ptree**> g_builtin_annotation_map;

static ptree create_primitive_node(const char* name, numeric_kind num_kind) {
    ptree node;
    node.kind = N_PRIMITIVE;
    node.name = name;
    node.value.val._d(num_kind);
    node.flags |= OPT_BUILTIN;
    node.file_name = "<built-in>";
    return node;
}

static ptree create_interface_node(const char* name, numeric_kind num_kind) {
    ptree node = create_primitive_node(name, num_kind);
    node.kind = N_INTERFACE;
    return node;
}

static ptree create_string_node(const char* name, ptree* element_type) {
    ptree node = create_primitive_node(name, UNDEF_KIND);
    node.kind = N_STRING;
    node.element_type = element_type;
    node.flags |= OPT_BUILTIN;
    return node;
}

extern "C" {
node_kind ANY_KIND[] = {N_UNDEF};

ptree boolean_type = create_primitive_node("boolean", BOOLEAN_KIND);
ptree int8_type = create_primitive_node("int8", INT8_KIND);
ptree octet_type = create_primitive_node("uint8", OCTET_KIND);
ptree char_type = create_primitive_node("char", CHAR_KIND);
ptree wchar_type = create_primitive_node("wchar", CHAR_KIND);
ptree short_type = create_primitive_node("int16", SHORT_KIND);
ptree ushort_type = create_primitive_node("uint16", USHORT_KIND);
ptree long_type = create_primitive_node("int32", LONG_KIND);
ptree ulong_type = create_primitive_node("uint32", ULONG_KIND);
ptree longlong_type = create_primitive_node("int64", LONGLONG_KIND);
ptree ulonglong_type = create_primitive_node("uint64", ULONGLONG_KIND);
ptree float_type = create_primitive_node("float", FLOAT_KIND);
ptree double_type = create_primitive_node("double", DOUBLE_KIND);
ptree ldouble_type = create_primitive_node("long double", DOUBLE_KIND);
ptree fixed_type = create_primitive_node("fixed", LONGLONG_KIND);
ptree unbounded_string_type = create_string_node("string", &char_type);
ptree unbounded_wstring_type = create_string_node("wstring", &wchar_type);

// Any and Object are remnants from CORBA and not used by DDS.
// They are here so we can parse old IDLs with CORBA interfaces.
ptree any_type = create_primitive_node("any", UNDEF_KIND);
ptree object_type = create_interface_node("Object", UNDEF_KIND);
}

static const node_kind TYPE_KIND[] = {
    N_PRIMITIVE,
    N_NATIVE,
    N_STRUCT,
    N_UNION,
    N_VALUETYPE,
    N_INTERFACE,
    N_EXCEPTION,
    N_ENUM,
    N_BITSET,
    N_BITMASK,
    N_SEQUENCE,
    N_MAP,
    N_ARRAY,
    N_STRING,
    N_FIXED,
    N_ALIAS,
    N_UNDEF,
};

static ptree* value_type(const numeric& value) {
    switch (value.kind()) {
    case BOOLEAN_KIND:
        return &boolean_type;
    case INT8_KIND:
    case OCTET_KIND:
        return &octet_type;
    case SHORT_KIND:
        return &short_type;
    case USHORT_KIND:
        return &ushort_type;
    case LONG_KIND:
        return &long_type;
    case ULONG_KIND:
        return &ulong_type;
    case LONGLONG_KIND:
        return &longlong_type;
    case ULONGLONG_KIND:
        return &ulonglong_type;
    case FLOAT_KIND:
        return &float_type;
    case DOUBLE_KIND:
        return &double_type;
    case CHAR_KIND:
        return &char_type;
    case STRING_KIND:
        return &unbounded_string_type;
    case PTREE_KIND:
        return value.val.node()->type;
    default:
        return &any_type;
    }
}

static bool is_of_type(const ptree* node, const node_kind kind[]) {
    if (!node) {
        return false;
    }
    if (kind == ANY_KIND) {
        return true;
    }
    for (int i = 0; kind[i] != N_UNDEF; ++i) {
        if (node->kind == kind[i]) {
            return true;
        }
    }
    return false;
}

static ptree* lookup_name(
    parser_state* state,
    const std::string& name,
    const std::map<std::string, ptree*>& lookup,
    const node_kind kind[],
    size_t level
) {
    ptree* res = nullptr;
    std::map<std::string, ptree*>::const_iterator it;
    if (name[0] == ':') {
        it = lookup.find(name);
        if (it != lookup.end() && is_of_type(it->second, kind)) {
            res = it->second;
        }
    } else {
        if (level > 0) {
            for (size_t i = 0; !res && i < state->context[level - 1].size(); ++i) {
                res = lookup_name(
                    state,
                    lc_scoped_name(state->context[level - 1][i]) + "::" + name,
                    lookup,
                    kind,
                    level - 1
                );
            }
            if (res == nullptr) {
                res = lookup_name(state, name, lookup, kind, level - 1);
            }
        } else {
            it = lookup.find("::" + name);
            if (it != lookup.end() && is_of_type(it->second, kind)) {
                res = it->second;
            }
        }
    }
    return res;
}

template <typename T>
static T* append_to_list(T* list, T* node) {
    if (!list) {
        return node;
    }
    if (!node) {
        return list;
    }

    T* last = list;
    while (last->next) {
        last = last->next;
    }
    last->next = node;
    return list;
}

static ptree* create_context_node(
    parser_state* state,
    node_kind kind,
    const char* ident,
    const std::vector<ptree*>& parents = std::vector<ptree*>()
) {
    ptree* node = create_node(state, kind, ident);
    node->parents = parents;
    register_node(state, node);
    push_context(state, node);
    return node;
}

static std::vector<ptree*> create_node_list(parser_state* state, declarator* decl, node_kind kind) {
    int len = 0;
    for (declarator* d = decl; d; d = d->next) {
        ++len;
    }
    std::vector<ptree*> res;
    if (len > 0) {
        for (declarator* d = decl; d; d = d->next) {
            ptree* node = lookup_node(state, d->ident.c_str());
            if (!node || node->kind != kind) {
                state->error() << "invalid parent type in node " << node;
                return {};
            }
            res.push_back(node);
        }
    }
    return res;
}

static bool is_int(const std::string& str) {
    int is_negative = !str.empty() && str.front() == '-';
    return std::all_of(str.begin() + is_negative, str.end(), isdigit);
}

static numeric lookup_member_value(parser_state* state, const numeric& value, const ptree* type) {
    for (ptree* member : type->members) {
        // Match if name is equal or numeric value is equal
        bool is_enum = value.kind() == STRING_KIND && member->name == value.val.str();
        bool is_num = value.kind() != STRING_KIND || is_int(value.val.str());
        if (is_enum || (is_num && integer_value(member->value) == integer_value(value))) {
            numeric num;
            num.base = 10;
            num.val.node(member);
            return num;
        }
    }
    state->error() << "invalid parent type in node " << type;
    return value;
}

static ptree* find_member(ptree* node, const char* name) {
    ptree* member = node ? node->members : nullptr;
    while (member) {
        if (member->name == name) {
            return member;
        }
        member = member->next;
    }
    return member;
}

static std::vector<std::string> split_doc_lines(const char* text, int placement) {
    std::vector<std::string> lines;
    if (!text || !text[0]) {
        return lines;
    }
    std::size_t whitespace = std::string::npos;
    std::istringstream stream(text);
    std::string tmp;
    while (std::getline(stream, tmp)) {
        if (!lines.empty() || !tmp.empty()) {
            if (placement == AFTER_DECLARATION && tmp[0] == '<') {
                tmp = tmp.substr(1);
            }
            auto non_blank = tmp.find_first_not_of(' ');
            if (non_blank != std::string::npos && tmp[non_blank] == '*') {
                if (non_blank + 1 < tmp.size()) {
                    non_blank += tmp.substr(non_blank + 1).find_first_not_of(' ');
                } else {
                    non_blank = std::string::npos;
                }
            }
            if (non_blank != std::string::npos &&
                (whitespace == std::string::npos || non_blank < whitespace)) {
                whitespace = non_blank;
            }
            lines.emplace_back(tmp);
        }
    }
    if (whitespace == std::string::npos) {
        whitespace = 0;
    }
    for (auto& line : lines) {
        if (line.size() > whitespace) {
            line = line.substr(whitespace);
        } else {
            line.clear();
        }
        if (placement == AFTER_DECLARATION) {
            line = "< " + line;
        }
    }
    if (placement != AFTER_DECLARATION) {
        // Break long detail strings after the next space.
        const size_t line_length = 120;
        std::vector<std::string> wrapped_lines;
        for (auto line : lines) {
            size_t position = 0;
            while (position != std::string::npos) {
                if (line.size() <= line_length) {
                    wrapped_lines.emplace_back(line);
                    position = std::string::npos;
                } else {
                    auto end = line.find_first_of(' ', position + line_length);
                    wrapped_lines.emplace_back(line.substr(0, end));
                    position = end;
                    if (position != std::string::npos) {
                        position = line.find_first_not_of(' ', position);
                    }
                    if (position != std::string::npos) {
                        line = line.substr(position);
                    }
                }
            }
        }
        lines = wrapped_lines;
    }

    while (!lines.empty() && lines.back().empty()) {
        lines.pop_back();
    }
    return lines;
}

static std::string format_docstring(const char* text, int placement) {
    std::stringstream res;
    for (const auto& line : split_doc_lines(text, placement)) {
        res << line << std::endl;
    }
    return res.str();
}

static ptree*
deep_clone_node(parser_state* state, const ptree* node, std::map<const ptree*, ptree*>& alloc);

static numeric
clone_numeric(parser_state* state, const numeric& num, std::map<const ptree*, ptree*>& alloc) {
    numeric clone = num;
    if (clone.kind() == PTREE_KIND) {
        clone.val.node(deep_clone_node(state, clone.val.node(), alloc));
    }
    return clone;
}

static ptree*
deep_clone_node(parser_state* state, const ptree* node, std::map<const ptree*, ptree*>& alloc) {
    if (!node || (node->flags & OPT_BUILTIN) != 0 || node->kind == N_ANNOTATION_DEF) {
        return const_cast<ptree*>(node);
    }
    auto it = alloc.find(node);
    if (it != alloc.end()) {
        return it->second;
    }

    // We don't use the copy constructor of `ptree` here to prevent accidentally
    // copying pointers from the other tree.
    auto p = std::make_shared<ptree>();
    alloc[node] = p.get();
    alloc[p.get()] = p.get();
    state->allocated_nodes.emplace_back(p);

    p->kind = node->kind;
    p->name = node->name;
    p->flags = node->flags;
    p->file_name = node->file_name;
    p->original = node;

    p->value = clone_numeric(state, node->value, alloc);
    p->super = deep_clone_node(state, node->super, alloc);
    p->scope = deep_clone_node(state, node->scope, alloc);
    p->type = deep_clone_node(state, node->type, alloc);
    p->element_type = deep_clone_node(state, node->element_type, alloc);
    p->key_type = deep_clone_node(state, node->key_type, alloc);
    p->discriminator = deep_clone_node(state, node->discriminator, alloc);
    p->included_from = node->included_from;

    for (const auto& bound : node->bounds) {
        p->bounds.emplace_back(clone_numeric(state, bound, alloc));
    }
    for (auto mem : node->members) {
        p->members = append_to_list(p->members, deep_clone_node(state, mem, alloc));
    }
    for (auto ann : node->annotations) {
        p->annotations = append_to_list(p->annotations, deep_clone_node(state, ann, alloc));
    }
    for (auto gen : node->generated) {
        p->generated = append_to_list(p->generated, deep_clone_node(state, gen, alloc));
    }
    for (auto parent : node->parents) {
        p->parents.emplace_back(deep_clone_node(state, parent, alloc));
    }
    for (auto raise : node->getraises) {
        p->getraises.emplace_back(deep_clone_node(state, raise, alloc));
    }
    for (auto raise : node->setraises) {
        p->setraises.emplace_back(deep_clone_node(state, raise, alloc));
    }

    auto scoped_name = lc_scoped_name(p.get());
    if ((p->flags & OPT_DECLARATION) == 0) {
        state->type_map[scoped_name] = p.get();
    }
    state->type_dcl_map[scoped_name] = p.get();
    return p.get();
}

static ptree* create_or_lookup_type(parser_state* state, node_kind kind, const char* ident) {
    std::string lc_name = "::" + tolower(ident);
    if (!state->context.empty()) {
        lc_name = lc_scoped_name(state->context[state->context.size() - 1][0]) + lc_name;
    }
    if (state->type_map.find(lc_name) == state->type_map.end()) {
        state->type_map[lc_name] = create_node(state, kind, ident);
    }
    return state->type_map[lc_name];
}

static ptree* create_sub_array_value_type(parser_state* state, const ptree* array, uint32_t depth) {
    declarator sub_array_type_decl;
    sub_array_type_decl.bounds = array->bounds;
    sub_array_type_decl.bounds.erase(
        sub_array_type_decl.bounds.begin(), sub_array_type_decl.bounds.begin() + depth
    );
    ptree* sub_arr = create_array_type(state, &sub_array_type_decl, array->element_type);
    return sub_arr;
}

static bool is_ref(const numeric& value) {
    return value.kind() == PTREE_KIND && value.val.node()->kind == N_CONST &&
           !value.val.node()->name.empty();
}

template <typename T, typename Pred>
static bool all_in_range(T begin, const T& end, const Pred& pred) {
    for (; begin != end; begin++) {
        if (!pred(begin)) {
            return false;
        }
    }
    return true;
}

static bool has_all_type_values(ptree* type, const std::set<int>& values) {
    if (type->kind == N_ALIAS) {
        return has_all_type_values(type->type, values);
    }
    const auto in_values = [&values](const int& i) { return values.find(i) != values.end(); };
    if (type->kind == N_ENUM) {
        return std::all_of(begin(type->members), end(type->members), [&in_values](const ptree* m) {
            return in_values(integer_value(m->value));
        });
    }
    if (type == &boolean_type) {
        return all_in_range(0, 2, in_values);
    }
    if (type == &octet_type || type == &char_type) {
        return all_in_range(0, 256, in_values);
    }
    return false;
}

static ptree* assign_members(parser_state* state, ptree* node, ptree* members) {
    node->members = members;

    // Apply any trailing doxy annotation at the head of the member list to the node.
    while (node->members && node->members->kind == N_ANNOTATION &&
           (is_doc_with_placement(node->members, AFTER_DECLARATION) ||
            (node->members->type == annotation_type_doc && node->members->next == nullptr))) {
        auto ann = node->members;
        node->members = node->members->next;
        ann->next = nullptr;
        annotate(state, node, ann);
    }
    return node;
}

extern "C" {

ptree* duplicate_node(parser_state* state, const ptree* node) {
    std::shared_ptr<ptree> p(new ptree);
    state->allocated_nodes.push_back(p);
    *p = *node;
    return p.get();
}

ptree* create_node(parser_state* state, node_kind kind, const char* ident) {
    std::shared_ptr<ptree> p(new ptree);
    p->kind = kind;
    if (ident) {
        p->name = ident;
    }
    p->super = state->context.empty() ? nullptr : state->context[state->context.size() - 1][0];
    p->scope = p->super;
    p->flags |= OPT_EMIT_CODE;

    if (!state->include_context.empty()) {
        const auto& [name, node] = state->include_context[state->include_context.size() - 1];
        p->included_from = node;
        p->file_name = name;
    }

    state->allocated_nodes.push_back(p);
    return p.get();
}

// Duplicates an entire tree, creating new nodes for all types and values.
ptree* duplicate_tree(parser_state* state, const ptree* node) {
    ptree* dup = nullptr;
    std::map<const ptree*, ptree*> allocated;
    for (; node; node = node->next) {
        dup = append_to_list(dup, deep_clone_node(state, node, allocated));
    }
    return dup;
}

ptree* try_lookup_node(parser_state* state, const char* name, const node_kind kind[]) {
    std::string lc_name = tolower(name);
    ptree* type = lookup_name(state, lc_name, state->type_map, kind, state->context.size());
    if (type == nullptr) {
        type = lookup_name(state, lc_name, state->type_dcl_map, kind, state->context.size());
    }
    return type;
}

void clear_namespace_nodes(parser_state* state) {
    auto it = state->type_map.begin();
    while (it != state->type_map.end()) {
        if (it->second->kind == N_MODULE) {
            state->type_map.erase(it++);
        } else {
            ++it;
        }
    }
}

void set_node_flags(ptree* p, ptree_opts flags) {
    p->flags = flags;
}

ptree* append_node(ptree* list, ptree* node) {
    if (list == node) {
        return list;
    }
    if (!node) {
        return list;
    }

    if (!list) {
        return node;
    }

    // If node is also a list, append each node individually to the list
    if (node->next) {
        auto next = node->next;
        node->next = nullptr;
        return append_node(append_node(list, node), next);
    }

    // Find last node in list
    auto last = list;
    while (last->next) {
        last = last->next;
    }

    last->next = node;
    return list;
}

ptree* remove_node(ptree* list, ptree* node) {
    while (list == node) {
        list = list->next;
    }
    for (auto a : list) {
        if (a->next == node) {
            a->next = node->next;
        }
    }
    return list;
}

declarator* append_decl(declarator* list, declarator* decl) {
    return append_to_list(list, decl);
}

declarator* create_decl(parser_state* state, const char* ident, ptree* annotations) {
    std::shared_ptr<declarator> decl(new declarator);
    decl->ident = ident;
    decl->annotations = annotations;
    state->allocated_decl.push_back(decl);
    return decl.get();
}

int register_node(parser_state* state, ptree* p) {
    std::string lc_name = lc_scoped_name(p);
    if (state->type_map.find(lc_name) != state->type_map.end()) {
        state->error() << "duplicate registration of name \"" << idl_scoped_name(p, nullptr)
                       << "\"";
        return false;
    }
    if (state->type_dcl_map.find(lc_name) != state->type_dcl_map.end() &&
        state->type_dcl_map[lc_name]->kind != p->kind) {
        state->error() << "inconsistent kind for previously declared type \""
                       << idl_scoped_name(p, nullptr) << "\" ";
        return false;
    }
    state->type_map[lc_name] = p;
    if (state->type_dcl_map.find(lc_name) != state->type_dcl_map.end()) {
        ptree* dcl = state->type_dcl_map[lc_name];
        dcl->type = p;
    }
    return true;
}

int register_node_dcl(parser_state* state, ptree* p) {
    std::string lc_name = lc_scoped_name(p);
    if (state->type_dcl_map.find(lc_name) == state->type_dcl_map.end()) {
        state->type_dcl_map[lc_name] = p;
    }
    return true;
}

ptree* lookup_node(parser_state* state, const char* ident) {
    ptree* type = try_lookup_node(state, ident, ANY_KIND);
    if (!type) {
        state->error() << "unknown node \"" << ident << "\"";
    }
    return type;
}

void add_context_parent_lookup(parser_state* state, ptree* p) {
    if (p) {
        for (auto& parent : p->parents) {
            state->context[state->context.size() - 1].push_back(parent);
            add_context_parent_lookup(state, parent);
        }
    }
}

void push_context(parser_state* state, ptree* p) {
    std::vector<ptree*> vec;
    vec.push_back(p);
    state->context.push_back(vec);
    add_context_parent_lookup(state, p);
}

ptree* pop_context(parser_state* state) {
    ptree* p = nullptr;
    if (!state->context.empty()) {
        p = state->context[state->context.size() - 1][0];
        state->context.pop_back();
    }
    return p;
}

ptree* peek_context(parser_state* state) {
    ptree* p = nullptr;
    if (!state->context.empty()) {
        p = state->context[state->context.size() - 1][0];
    }
    return p;
}

static std::string array_name(ptree* element_type, declarator* decl) {
    std::stringstream str;
    str << element_type->name;
    for (auto& bound : decl->bounds) {
        str << "[" << string_value(bound) << "]";
    }
    return str.str();
}

ptree* create_array_type(parser_state* state, declarator* declarator, ptree* type) {
    ptree* res = create_or_lookup_type(state, N_ARRAY, array_name(type, declarator).c_str());
    res->element_type = type;
    res->bounds = declarator->bounds;
    res->annotations = declarator->annotations;
    return res;
}

static std::string sequence_name(ptree* element_type, const numeric& bound) {
    std::stringstream str;
    int val = integer_value(bound);
    str << "sequence<" << element_type->name;
    if (val > 0) {
        str << "," << val;
    }
    str << ">";
    return str.str();
}

static std::string map_name(ptree* key_type, ptree* element_type, const numeric& bound) {
    std::stringstream str;
    int val = integer_value(bound);
    str << "map<" << key_type->name << "," << element_type->name;
    if (val > 0) {
        str << "," << val;
    }
    str << ">";
    return str.str();
}

static std::string string_name(ptree* element_type, const numeric& bound) {
    std::stringstream str;
    int val = integer_value(bound);
    if (element_type == &wchar_type) {
        str << "w";
    }
    str << "string";
    if (val > 0) {
        str << "<" << val << ">";
    }
    return str.str();
}

void create_include_start(parser_state* state, const char* ident, int is_system_include) {
    ptree* p = nullptr;
    std::string scoped_name = std::string("::<") + ident;
    auto it = state->type_map.find(scoped_name);
    if (it != state->type_map.end()) {
        p = it->second;
    }
    if (!p) {
        p = create_node(state, N_INCLUDE, ident);
        p->flags |= is_system_include ? OPT_SYSTEM_INCLUDE : 0;
        state->type_map[scoped_name] = p;
    }
    state->include_context.emplace_back(ident, p);
}

ptree* create_include_finish(parser_state* state, ptree* def) {
    state->include_context.pop_back();
    return def;
}

void create_module_start(parser_state* state, const char* ident) {
    const node_kind module_kind[] = {N_MODULE};
    ptree* p = create_node(state, N_MODULE, ident);
    ptree* prev = try_lookup_node(state, lc_scoped_name(p).c_str(), module_kind);
    if (!prev) {
        register_node(state, p);
    }
    push_context(state, p);
}

ptree* create_module_finish(parser_state* state, ptree* def) {
    ptree* p = pop_context(state);
    assign_members(state, p, def);
    return p;
}

const numeric* lookup_value(parser_state* state, const char* ident) {
    ptree* p = try_lookup_node(state, ident, ANY_KIND);
    if (p) {
        auto n = new_numeric(state, PTREE_KIND);
        n->val.node(p);
        return n;
    }
    if (!state->context.empty() &&
        state->context[state->context.size() - 1][0]->kind == N_ANNOTATION) {
        auto n = new_numeric(state, PTREE_KIND);
        n->val.str(ident);
        return n;
    }
    state->error() << "unknown value \"" << ident << "\"";
    return new_numeric(state, UNDEF_KIND);
}

const numeric* create_value_node(parser_state* state, const numeric* value, ptree* members) {
    auto num = new_numeric(state, value->kind());
    *num = *value;
    if (num->kind() == UNDEF_KIND) {
        const char* ident = {nullptr};
        ptree* node = create_node(state, N_CONST, ident);
        assign_members(state, node, members);
        node->flags |= OPT_CONST_VALUE;
        for (auto elem : members) {
            elem->flags |= OPT_CONST_VALUE;
        }
        num->val.node(node);
    }
    return num;
}

static void
validate_const_value_type(parser_state* state, const char* ident, const ptree* complex_value) {
    for (const auto node : complex_value) {
        if (node->kind == N_CONST && node->value->_d() == PTREE_KIND) {
            auto val = node->value->node();
            if (!is_primitive(val) && val->kind != N_STRING && val->kind != N_CONST) {
                state->error() << "Cannot assign " << val << " of type " << val->kind
                               << " to const " << ident;
            }
        }
        validate_const_value_type(state, ident, node->members);
    }
}

ptree* create_const_node(parser_state* state, declarator* decl, ptree* type, const numeric* value) {
    numeric num(*value);
    const char* ident = {nullptr};
    if (decl) {
        ident = decl->ident.c_str();
    }
    ptree* p = create_node(state, N_CONST, ident);
    if (type && decl && !decl->bounds.empty()) {
        type = create_array_type(state, decl, type);
    }
    p->type = type ? type : value_type(num);
    p->value = num;
    if (num.kind() == UNDEF_KIND) {
        p->flags |= OPT_DECLARATION;
    }
    if (num.kind() == PTREE_KIND) {
        validate_const_value_type(state, ident, num->node());
    }
    if (type && decl) {
        register_node(state, p);
    }
    return p;
}

ptree* add_bounds(ptree* type, const numeric* bound) {
    if (bound->kind() != UNDEF_KIND) {
        type->bounds.push_back(*bound);
    }
    return type;
}

ptree* create_sequence(parser_state* state, ptree* element_type, const numeric* bound) {
    if (!element_type) {
        return nullptr;
    }
    ptree* p =
        create_or_lookup_type(state, N_SEQUENCE, sequence_name(element_type, *bound).c_str());
    p->element_type = element_type;
    add_bounds(p, bound);
    return p;
}

ptree* create_string(parser_state* state, const numeric* bound) {
    ptree* p;
    if (bound->kind() == UNDEF_KIND) {
        p = &unbounded_string_type;
    } else {
        p = create_or_lookup_type(state, N_STRING, string_name(&char_type, *bound).c_str());
        p->element_type = &char_type;
        add_bounds(p, bound);
    }
    return p;
}

ptree* create_wstring(parser_state* state, const numeric* bound) {
    ptree* p;
    if (bound->kind() == UNDEF_KIND) {
        p = &unbounded_wstring_type;
    } else {
        p = create_or_lookup_type(state, N_STRING, string_name(&wchar_type, *bound).c_str());
        p->element_type = &wchar_type;
        add_bounds(p, bound);
    }
    return p;
}

ptree* create_fixed(parser_state* state, const numeric* bound1, const numeric* bound2) {
    ptree* p = create_or_lookup_type(state, N_FIXED, fixed_name(*bound1, *bound2).c_str());
    p->element_type = &long_type;

    p->bounds.push_back(*bound1);
    p->bounds.push_back(*bound2);
    return p;
}

numeric* new_numeric(parser_state* state, numeric_kind kind) {
    numeric n;
    n.val._d(kind);
    state->numeric_map.emplace_back(n);
    return &state->numeric_map.back();
}

const numeric* create_bool(parser_state* state, int value) {
    auto n = new_numeric(state, BOOLEAN_KIND);
    n->val.b(value != 0);
    return n;
}

const numeric* create_char(parser_state* state, char value) {
    auto n = new_numeric(state, CHAR_KIND);
    n->val.c(value);
    return n;
}

const numeric* create_i64(parser_state* state, int64_t value, int base) {
    auto n = new_numeric(state, LONGLONG_KIND);
    n->base = base;
    n->val.ll(value);
    return n;
}

const numeric* create_u64(parser_state* state, uint64_t value, int base) {
    auto n = new_numeric(state, ULONGLONG_KIND);
    n->base = base;
    n->val.ull(value);
    return n;
}

const numeric* create_str(parser_state* state, const char* value) {
    auto n = new_numeric(state, STRING_KIND);
    n->val.str(value);
    return n;
}

const numeric* create_float(parser_state* state, float value) {
    auto n = new_numeric(state, FLOAT_KIND);
    n->val.f(value);
    return n;
}

const numeric* create_double(parser_state* state, double value) {
    auto n = new_numeric(state, DOUBLE_KIND);
    n->val.d(value);
    return n;
}

const numeric* create_numeric_node(parser_state* state, ptree* node) {
    auto n = new_numeric(state, PTREE_KIND);
    n->val.node(node);
    return n;
}

ptree* create_struct_start(parser_state* state, const char* ident, ptree* parent) {
    std::vector<ptree*> parents;
    if (parent) {
        parent->flags |= OPT_HAS_CHILDREN;
        parents.push_back(parent);
    }
    return create_context_node(state, N_STRUCT, ident, parents);
}

ptree* create_struct_finish(parser_state* state, ptree* members) {
    ptree* p = pop_context(state);
    assign_members(state, p, members);
    return p;
}

ptree* create_struct_dcl(parser_state* state, const char* ident) {
    ptree* p = create_node(state, N_STRUCT, ident);
    register_node_dcl(state, p);
    p->flags |= OPT_DECLARATION;
    return p;
}

ptree* create_union_start(parser_state* state, const char* ident) {
    return create_context_node(state, N_UNION, ident);
}

ptree* create_union_discriminator(parser_state* state, ptree* type, ptree* annotations) {
    auto decl = create_decl(state, "_d", nullptr);
    return create_member(state, decl, type, annotations);
}

ptree* create_union_finish(parser_state* state, ptree* discriminator, ptree* members) {
    if (discriminator) {
        ptree* prev_case = nullptr;
        ptree* default_case = nullptr;
        int label_group = 0;
        int default_label_group = 0;

        create_annotation_start(state, "@must_understand", annotation_type_must_understand);
        discriminator = annotate(state, discriminator, create_annotation_finish(state, nullptr));
        std::set<int> case_values;

        for (auto mem : members) {
            for (auto c : mem->members) {
                c->type = discriminator->type;
                // default:
                if (c->flags & OPT_DEFAULT) {
                    default_case = c;
                    default_label_group = label_group;
                    continue;
                }
                // case:
                prev_case = c;
                if (c->type->kind == N_ENUM && c->value.kind() != PTREE_KIND) {
                    c->value = lookup_member_value(state, c->value, discriminator->type);
                }
                if (c->value.kind() != PTREE_KIND) {
                    c->value = *expr_convert(state, &c->value, c->type->value.kind());
                }
                if (c->value.kind() != UNDEF_KIND) {
                    case_values.insert(integer_value(c->value));
                }
                if (prev_case && default_case && default_case->value.kind() == UNDEF_KIND &&
                    label_group == default_label_group) {
                    default_case->value = prev_case->value;
                }
            }
            ++label_group;
        }
        if (!default_case && !has_all_type_values(discriminator->type, case_values)) {
            default_case = create_default_case(state);
            default_case->type = discriminator->type;
            auto default_member =
                create_union_member(state, create_null_node(state), default_case, nullptr);
            append_node(members, default_member);
        }
        if (default_case && default_case->value.kind() == UNDEF_KIND) {
            if (default_case->type->kind == N_ENUM) {
                for (auto elem : default_case->type->members) {
                    if (case_values.find(integer_value(elem->value)) == case_values.end()) {
                        default_case->value.val.node(elem);
                        break;
                    }
                }
            } else {
                for (int i = 0; default_case->value.kind() == UNDEF_KIND; ++i) {
                    if (case_values.find(i) == case_values.end()) {
                        default_case->value.val.l(i);
                        default_case->value = *expr_convert(
                            state,
                            &default_case->value,
                            base_type_of(discriminator->type)->value.kind()
                        );
                    }
                }
            }
        }
        // Coalesce multiple null members into one
        ptree* null_case = nullptr;
        ptree* last = nullptr;
        for (ptree* mem = members; mem; mem = mem->next) {
            if (mem->kind == N_NULL) {
                if (null_case) {
                    for (auto c : mem->members) {
                        c->super = null_case;
                    }
                    null_case->members = append_node(null_case->members, mem->members);
                    last->next = mem->next;
                } else {
                    null_case = mem;
                    last = mem;
                }
            } else {
                last = mem;
            }
        }

        // Mark the member that contains the default case as default as well
        for (ptree* mem = members; mem; mem = mem->next) {
            if (has_default_case(mem)) {
                mem->flags |= OPT_DEFAULT;
                break;
            }
        }
    }

    ptree* p = pop_context(state);
    p->discriminator = discriminator;
    assign_members(state, p, members);
    return p;
}

ptree* create_union_dcl(parser_state* state, const char* ident) {
    ptree* p = create_node(state, N_UNION, ident);
    register_node_dcl(state, p);
    p->flags |= OPT_DECLARATION;
    return p;
}

ptree* create_union_member(parser_state* state, ptree* value, ptree* cases, ptree* annotations) {
    if (value) {
        value->members = append_node(value->members, cases);

        for (auto cas = value->members; cas; cas = cas->next) {
            cas->super = value;

            if (cas->next && cas->next->kind == N_ANNOTATION) {
                annotate(state, value, cas->next);
                cas->next = nullptr;
                break;
            }
        }
        annotate(state, value, annotations);
    }
    return value;
}

ptree*
create_member(parser_state* state, declarator* declarators, ptree* type, ptree* annotations) {
    ptree* res = nullptr;
    if (type) {
        while (declarators) {
            ptree* node = create_node(state, N_MEMBER, declarators->ident.c_str());
            register_node(state, node);
            node->type =
                !declarators->bounds.empty() ? create_array_type(state, declarators, type) : type;
            annotate(state, node, append_node(declarators->annotations, annotations));
            res = append_node(res, node);
            declarators = declarators->next;
        }
    } else {
        state->error() << "unknown type for member " << declarators->ident;
    }
    return res;
}

ptree* create_case_label(parser_state* state, const numeric* value) {
    ptree* p = create_node(state, N_CASE, string_value(*value).c_str());
    p->value = *value;
    return p;
}

ptree* create_default_case(parser_state* state) {
    ptree* p = create_node(state, N_CASE, "default");
    p->flags |= OPT_DEFAULT;
    return p;
}

ptree* create_null_node(parser_state* state) {
    ptree* p = create_node(state, N_NULL, "null");
    return p;
}

ptree* create_type(parser_state* state, declarator* declarators, ptree* type) {
    if (!type) {
        return nullptr;
    }

    ptree* res = nullptr;
    ptree* scope = state->context.empty() ? nullptr : state->context[state->context.size() - 1][0];

    if (type->super == scope && type->next == nullptr) {
        if (type->name[0] == '<') {
            res = type;
            res->name = declarators->ident;
            register_node(state, res);
            declarators = declarators->next;
        }
    }
    while (declarators) {
        ptree* t = type;
        if (!declarators->bounds.empty()) {
            t = create_array_type(state, declarators, type);
        }
        ptree* node = create_node(state, N_ALIAS, declarators->ident.c_str());
        node->type = t;
        annotate(state, node, declarators->annotations);
        register_node(state, node);
        res = append_node(res, node);
        declarators = declarators->next;
    }
    return res;
}

ptree* create_native_type(parser_state* state, const char* ident) {
    ptree* node = create_node(state, N_NATIVE, ident);
    register_node(state, node);
    return node;
}

ptree* create_exception_start(parser_state* state, const char* ident) {
    return create_context_node(state, N_EXCEPTION, ident);
}

ptree* create_exception_finish(parser_state* state, ptree* members) {
    ptree* node = pop_context(state);
    assign_members(state, node, members);
    return node;
}

ptree* create_interface_dcl(parser_state* state, const char* ident, int is_local) {
    ptree* node = create_node(state, N_INTERFACE, ident);
    register_node_dcl(state, node);
    node->flags |= OPT_DECLARATION;
    if (is_local) {
        node->flags |= OPT_LOCAL;
    }
    return node;
}

ptree*
create_interface_start(parser_state* state, const char* ident, declarator* parents, int is_local) {
    auto node = create_context_node(
        state, N_INTERFACE, ident, create_node_list(state, parents, N_INTERFACE)
    );
    if (is_local) {
        peek_context(state)->flags |= OPT_LOCAL;
    }
    return node;
}

ptree* create_interface_finish(parser_state* state, ptree* members) {
    ptree* node = pop_context(state);
    assign_members(state, node, members);
    for (auto& parent : node->parents) {
        parent->flags |= OPT_HAS_CHILDREN;
    }

    // IDL interfaces do not form lexical scopes
    for (auto m : node->members) {
        m->scope = node->scope;
    }
    return node;
}

ptree* annotate(parser_state* state, ptree* node, ptree* annotations) {
    if (node) {
        ptree* ann = annotations;
        while (ann) {
            for (auto m : ann->members) {
                ptree* type_member = find_member(ann->type, m->name.c_str());
                if (type_member && type_member->type == &any_type) {
                    node_kind lookup_kinds[] = {N_CONST, N_UNDEF};
                    if (m->value.kind() == STRING_KIND &&
                        try_lookup_node(state, m->value.val.str().c_str(), lookup_kinds)) {
                        m->value = *lookup_value(state, m->value.val.str().c_str());
                    } else if (m->value.kind() != PTREE_KIND) {
                        m->value =
                            *expr_convert(state, &m->value, base_type_of(node)->value.kind());
                        m->type = base_type_of(node);
                    }
                    if (m->value.kind() == PTREE_KIND) {
                        m->type = m->value.val.node()->type;
                    }
                }
            }
            if (ann->type == annotation_type_ext_suppress) {
                if (integer_value(ann->value) == 1) {
                    node->flags &= ~OPT_EMIT_CODE;
                } else {
                    node->flags |= OPT_EMIT_CODE;
                }
            }
            if (ann->type == annotation_type_bit_bound) {
                int bits = integer_value(ann->value);
                if (node->kind == N_ENUM) {
                    if (bits <= 8) {
                        node->element_type = &int8_type;
                        for (auto m : node->members) {
                            if (long_long_value(m->value) > std::numeric_limits<int8_t>::max()) {
                                node->element_type = &octet_type;
                            }
                        }
                    } else if (bits <= 16) {
                        node->element_type = &short_type;
                        for (auto m : node->members) {
                            if (long_long_value(m->value) > std::numeric_limits<int16_t>::max()) {
                                node->element_type = &ushort_type;
                            }
                        }
                    } else {
                        node->element_type = &long_type;
                        for (auto m : node->members) {
                            if (long_long_value(m->value) > std::numeric_limits<int16_t>::max()) {
                                node->element_type = &ulong_type;
                            }
                        }
                    }
                    node->value = node->element_type->value;
                    for (auto m : node->members) {
                        auto new_value =
                            *expr_convert(state, &m->value, node->element_type->value.kind());
                        if (string_value(new_value) == string_value(m->value)) {
                            m->value = new_value;
                        }
                    }
                } else if (node->kind == N_BITMASK) {
                    if (bits <= 8) {
                        node->element_type = &octet_type;
                    } else if (bits <= 16) {
                        node->element_type = &ushort_type;
                    } else if (bits <= 32) {
                        node->element_type = &ulong_type;
                    } else {
                        node->element_type = &ulonglong_type;
                    }
                    node->value = node->element_type->value;
                    for (auto m : node->members) {
                        auto v = value<uint64_t>(m->value);
                        while (v) {
                            v >>= 1;
                        }
                        m->value =
                            *expr_convert(state, &m->value, node->element_type->value.kind());
                    }
                }
            }
            if (ann->type == annotation_type_ext_repeat_count) {
                if (ann->value.kind() == PTREE_KIND) {
                    const_cast<ptree*>(ann->value.val.node())->flags |= OPT_SEQUENCE_LENGTH;
                }
            }
            if (ann->type == annotation_type_default) {
                ann->members->value = ann->value;
            }
            ann->super = ann->scope = node;
            ann = ann->next;
        }

        node->annotations = append_node(node->annotations, annotations);

        if (node->kind == N_ANNOTATION) {
            append_to_list(node, node->annotations);
            node->annotations = nullptr;
        }
    }
    return node;
}

ptree* annotate_list(parser_state* state, ptree* node, ptree* annotations) {
    ptree* n = node;
    while (n) {
        annotate(state, n, annotations);
        n = n->next;
    }
    return node;
}

ptree* annotate_last(parser_state* state, ptree* node, ptree* annotations) {
    if (node) {
        ptree* n = node;
        while (n->next) {
            n = n->next;
        }
        annotate(state, n, annotations);
    }
    return node;
}

ptree* annotate_alias(parser_state* state, ptree* node, ptree* annotations) {
    ptree* res = node;
    if (node && annotations) {
        std::stringstream name;
        name << node->name;
        for (auto ann : annotations) {
            name << "_" << ann->name;
            for (auto member : ann->members) {
                name << "_" << string_value(member->value);
            }
        }
        ptree* existing = try_lookup_node(state, name.str().c_str(), ANY_KIND);
        if (existing) {
            res = existing;
        } else {
            res = create_node(state, N_ALIAS, name.str().c_str());
            res->type = node;
            res->flags |= OPT_ANONYMOUS_ALIAS;
            res = annotate(state, res, annotations);
        }
    }
    return res;
}

ptree* create_interface_op(
    parser_state* state,
    const char* ident,
    ptree* params,
    ptree* retval,
    declarator* raises
) {
    ptree* node = create_node(state, N_PROTOTYPE, ident);
    register_node(state, node);
    assign_members(state, node, params);
    node->type = retval;
    for (auto p : params) {
        p->super = node;
        p->scope = node->scope;
    }
    if (raises) {
        node->getraises = create_node_list(state, raises, N_EXCEPTION);
    }
    return node;
}

ptree* create_param_dcl(parser_state* state, declarator* decl, ptree* type, int kind) {
    ptree* node = create_node(state, N_MEMBER, decl->ident.c_str());
    node->type = type;
    node->flags |= kind;
    return node;
}

ptree* create_attribute(
    parser_state* state,
    declarator* decl,
    ptree* type,
    declarator* getraises,
    declarator* setraises,
    int readonly
) {
    ptree* node = create_node(state, N_MEMBER, decl->ident.c_str());
    register_node(state, node);
    node->type = type;
    node->annotations = decl->annotations;
    if (readonly) {
        node->flags |= OPT_READONLY;
    }
    if (setraises) {
        node->setraises = create_node_list(state, setraises, N_EXCEPTION);
    }
    if (getraises) {
        node->getraises = create_node_list(state, getraises, N_EXCEPTION);
    }
    return node;
}

ptree* create_map(parser_state* state, ptree* key_type, ptree* element_type, const numeric* bound) {
    if (!key_type || !element_type) {
        return nullptr;
    }
    ptree* p =
        create_or_lookup_type(state, N_MAP, map_name(key_type, element_type, *bound).c_str());
    p->element_type = element_type;
    p->key_type = key_type;
    add_bounds(p, bound);
    return p;
}

ptree* create_bitset(parser_state* state, const char* ident, ptree* fields, ptree* parent) {
    ptree* node = create_node(state, N_BITSET, ident);
    register_node(state, node);
    assign_members(state, node, fields);
    if (parent) {
        parent->flags |= OPT_HAS_CHILDREN;
        node->parents.push_back(parent);
    }
    for (auto m : fields) {
        m->super = node;
        m->scope = node->scope;
        register_node(state, m);
    }
    return node;
}

ptree* create_bitfield(parser_state* state, const char* ident, const numeric* bits, ptree* type) {
    ptree* node = create_node(state, N_CONST, ident);
    node->value = *bits;
    node->type = type ? type : &long_type;
    return node;
}

ptree* create_enum(parser_state* state, const char* ident, ptree* type, ptree* values) {
    ptree* node = create_node(state, N_ENUM, ident);
    register_node(state, node);

    node->element_type = type;
    node->value = ulong_type.value;
    for (ptree* val : values) {
        val->super = node;
        val->scope = node->scope;
        // Register value inside enum scope too. IDL spec says register it
        // outside (and we do in create_enum_value), but this is consistent
        // with bitset and languages with scoped enums.
        register_node(state, val);
    }
    assign_members(state, node, values);
    return node;
}

ptree* create_enum_value(parser_state* state, const char* ident, const numeric* value) {
    ptree* p = create_node(state, N_CONST, ident);
    register_node(state, p);
    p->value = *value;
    return p;
}

ptree* create_bitmask(parser_state* state, const char* ident, ptree* type, ptree* values) {
    ptree* node = create_node(state, N_BITMASK, ident);
    register_node(state, node);
    for (ptree* val : values) {
        val->super = node;
        val->scope = node->scope;
        register_node(state, val);
    }
    assign_members(state, node, values);
    node->element_type = type;
    node->value = ulong_type.value;
    return node;
}

ptree* create_bitmask_value(parser_state* state, const char* ident, const numeric* value) {
    ptree* node = create_node(state, N_CONST, ident);
    register_node(state, node);
    node->value = *value;
    return node;
}

void create_annotation_dcl_start(parser_state* state, const char* ident) {
    create_context_node(state, N_ANNOTATION_DEF, ident);
}

ptree* create_annotation_dcl_finish(parser_state* state, ptree* members) {
    ptree* node = pop_context(state);
    assign_members(state, node, members);
    auto builtin_it = g_builtin_annotation_map.find(idl_scoped_name(node, nullptr));
    if (builtin_it != g_builtin_annotation_map.end()) {
        *builtin_it->second = node;
    }
    return node;
}

ptree* create_annotation_member(
    parser_state* state,
    declarator* decl,
    ptree* type,
    const numeric* default_value
) {
    ptree* node = create_node(state, N_MEMBER, decl->ident.c_str());
    node->type = type;
    node->value = *default_value;
    return node;
}

void create_annotation_start(parser_state* state, const char* ident, ptree* annotation_def) {
    ptree* node;
    if (annotation_def && annotation_def->kind == N_ANNOTATION_DEF) {
        node = create_node(state, N_ANNOTATION, annotation_def->name.c_str());
        node->type = annotation_def;
        node->super = annotation_def->super;
        node->scope = annotation_def->scope;
    } else {
        node = create_node(state, N_ANNOTATION, ident + 1);  // +1 to skip '@'
        node->type = annotation_def;
    }
    push_context(state, node);
}

ptree* create_annotation_finish(parser_state* state, ptree* params) {
    ptree* node = pop_context(state);
    if (node->type == nullptr) {
        return nullptr;
    }
    if (node->type == annotation_type_bit_bound_old) {
        node->type = annotation_type_bit_bound;
        node->name = node->type->name;
    }
    if (node->type == annotation_type_ext_doc) {
        node->type = annotation_type_doc;
    }
    if (node->type == annotation_type_must_understand_old) {
        node->type = annotation_type_must_understand;
        node->name = node->type->name;
    }
    if (node->type == annotation_type_minimum_type_check_old) {
        node->type = annotation_type_ext_minimum_type_check;
        node->name = node->type->name;
    }
    if (node->type == annotation_type_ext_no_serializer) {
        node->type = annotation_type_non_serialized;
        node->name = node->type->name;
    }
    if (node->type == annotation_type_final) {
        node->type = annotation_type_extensibility;
        node->name = node->type->name;
        numeric value;
        value.val.node(try_lookup_node(state, "Extensibility::FINAL", ANY_KIND));
        params = create_annotation_param(state, "value", &value);
    }
    if (node->type == annotation_type_mutable) {
        node->type = annotation_type_extensibility;
        node->name = node->type->name;
        numeric value;
        value.val.node(try_lookup_node(state, "Extensibility::MUTABLE", ANY_KIND));
        params = create_annotation_param(state, "value", &value);
    }
    if (node->type == annotation_type_appendable) {
        node->type = annotation_type_extensibility;
        node->name = node->type->name;
        numeric value;
        value.val.node(try_lookup_node(state, "Extensibility::APPENDABLE", ANY_KIND));
        params = create_annotation_param(state, "value", &value);
    }
    node->scope = state->context.empty() ? nullptr : state->context[state->context.size() - 1][0];
    node->super = node->scope;
    ptree* default_value = nullptr;
    ptree* first_value = nullptr;
    int member_count = 0;
    std::map<std::string, ptree*> arguments;
    for (auto el : node->type->members) {
        if (el->kind == N_MEMBER) {
            if (member_count == 0) {
                first_value = el;
            }
            if (el->value.kind() == UNDEF_KIND) {
                if (default_value && default_value->value.kind() == UNDEF_KIND) {
                    default_value = nullptr;
                    break;
                }
                default_value = el;
            }
            ++member_count;
        }
    }
    if (member_count == 1) {
        default_value = first_value;
    }
    for (auto el : params) {
        if (el->name.empty()) {
            if (default_value) {
                el->name = default_value->name;
            } else {
                return nullptr;
            }
        }
        arguments[el->name] = el;
    }
    for (auto el : node->type->members) {
        if (el->kind == N_MEMBER) {
            auto it = arguments.find(el->name);
            ptree* arg = nullptr;
            if (it == arguments.end()) {
                arg = create_annotation_param(state, el->name.c_str(), &el->value);
                params = append_node(params, arg);
                arg->type = el->type;
            } else {
                arg = it->second;
                if (it->second->value.kind() == UNDEF_KIND) {
                    arg->value = el->value;
                    arg->type = el->type;
                } else if (arg->value.kind() == PTREE_KIND &&
                           arg->value->node()->type == el->type) {
                    // Do nothing, const value type same as member type
                } else if (el->type != &any_type) {
                    arg->value = *expr_convert(state, &arg->value, el->value.kind());
                    arg->type = el->type;
                }
            }
        }
    }
    if (member_count == 1 && params) {
        node->value = params->value;
    }
    assign_members(state, node, params);
    return node;
}

ptree* create_annotation_param(parser_state* state, const char* ident, const numeric* value) {
    ptree* node = create_node(state, N_CONST, ident);
    node->value = *value;
    node->type = value_type(*value);
    return node;
}

ptree* create_valuetype_dcl(parser_state* state, const char* ident) {
    ptree* p = create_node(state, N_VALUETYPE, ident);
    register_node_dcl(state, p);
    p->flags |= OPT_DECLARATION;
    return p;
}

ptree*
create_valuetype_start(parser_state* state, const char* ident, ptree* parent, ptree* interface) {
    std::vector<ptree*> parents;
    if (parent) {
        parent->flags |= OPT_HAS_CHILDREN;
        parents.push_back(parent);
    }
    ptree* node = create_context_node(state, N_VALUETYPE, ident, parents);
    node->type = interface;
    return node;
}

ptree* create_valuetype_finish(parser_state* state, ptree* members) {
    ptree* node = pop_context(state);
    assign_members(state, node, members);
    return node;
}

declarator* append_array_size(parser_state* state, declarator* decl, const numeric* value) {
    if (!decl) {
        decl = create_decl(state, "", nullptr);
    }
    decl->bounds.push_back(*value);
    return decl;
}

void validate_node(parser_state* state, ptree* node) {
    node_kind has_members[] = {
        N_MODULE,
        N_INCLUDE,
        N_STRUCT,
        N_UNION,
        N_MEMBER,
        N_NULL,
        N_VALUETYPE,
        N_INTERFACE,
        N_PROTOTYPE,
        N_EXCEPTION,
        N_ENUM,
        N_BITSET,
        N_BITMASK,
        N_ANNOTATION_DEF,
        N_ANNOTATION,
        N_UNDEF
    };
    node_kind has_subtype[] = {
        N_SEQUENCE, N_MAP, N_ARRAY, N_STRING, N_FIXED, N_ENUM, N_BITMASK, N_UNDEF
    };
    node_kind has_type[] = {N_ALIAS, N_CONST, N_MEMBER, N_CASE, N_PROTOTYPE, N_ANNOTATION, N_UNDEF};
    node_kind can_declare[] = {N_STRUCT, N_UNION, N_VALUETYPE, N_INTERFACE, N_CONST, N_UNDEF};
    node_kind is_member[] = {N_MEMBER, N_CASE, N_PROTOTYPE, N_UNDEF};
    node_kind illegal_types[] = {N_MODULE, N_INCLUDE, N_CONST, N_MEMBER, N_CASE, N_UNDEF};
    if (node) {
        // All nodes have names
        if (node->name.empty()) {
            state->error() << "Unnamed node in scope " << node->super;
        }

        if ((!node->included_from && (node->flags & OPT_BUILTIN) == 0) || node->file_name.empty()) {
            state->error() << "Node is missing a file name";
        }

        // If node has members, it must be a type with members
        if (node->members && !is_of_type(node, has_members)) {
            state->error() << "Unexpected members in node " << node << " with kind " << node->kind;
        }

        // Only nodes with subtype shall have an element type, and they must have one
        if (node->element_type && !is_of_type(node, has_subtype)) {
            state->error() << "Unexpected element type in node " << node << " with kind "
                           << node->kind;
        } else if (!node->element_type && is_of_type(node, has_subtype)) {
            state->error() << "Missing element type in node " << node << " with kind "
                           << node->kind;
        }

        // Only declarable nodes can have a declaration
        if ((node->flags & OPT_DECLARATION) && !is_of_type(node, can_declare)) {
            state->error() << "Unexpected declaration of " << node << " with kind " << node->kind;
        }

        if (node->type) {
            // All types have names
            if (node->type->name.empty()) {
                state->error() << "Unnamed type for node " << node;
            }

            // Declarations points to their definition through type.
            if (!is_of_type(node, has_type) && !(node->flags & OPT_DECLARATION)) {
                state->error() << "Unexpected type in node " << node << " with kind " << node->kind;
            }

            // Type shall never point to a declaration
            if (node->type->flags & OPT_DECLARATION) {
                state->error() << "Type " << node->type << " for node " << node << " with kind "
                               << node->kind << " only declared, not defined";
            }

            // Some kinds (such as include and module) cannot be a type
            if (is_of_type(node->type, illegal_types)) {
                state->error() << "Type " << node->type << " with kind " << node->type->kind
                               << " for node " << node << " with kind " << node->kind
                               << " is not a legal type kind";
            }
        }
        // Prototypes may have a null (return) type, others must have a non-null type
        else if (is_of_type(node, has_type) && node->kind != N_PROTOTYPE) {
            state->error() << "Missing type in node " << node << " with kind " << node->kind;
        }

        // Members must be scoped inside a node that can hold members
        if (is_of_type(node, is_member) && !is_of_type(node->super, has_members)) {
            state->error() << "Unexpected scope " << node->super << " for member " << node;
        }

        if (node->kind == N_CONST) {
            // All constants must have a defined value
            if (!(node->flags & OPT_DECLARATION) &&
                (node->value.kind() == UNDEF_KIND ||
                 (node->value.kind() == PTREE_KIND && node->value.val.node() == nullptr))) {
                state->error() << "Undefined constant value " << node;
            }
        }

        // All annotations must be N_ANNOTATION
        for (auto ann : node->annotations) {
            if (ann->kind != N_ANNOTATION) {
                state->error() << "Illegal annotation " << ann << " on node " << node;
            }
        }

        // All exceptions must be of N_EXCEPTION kind
        for (auto except = node->getraises.begin(); except != node->getraises.end(); ++except) {
            if ((*except)->kind != N_EXCEPTION) {
                state->error() << "Illegal exception " << (*except) << " on node " << node;
            }
        }
        for (auto except = node->setraises.begin(); except != node->setraises.end(); ++except) {
            if ((*except)->kind != N_EXCEPTION) {
                state->error() << "Illegal exception " << (*except) << " on node " << node;
            }
        }
        for (auto parent = node->parents.begin(); parent != node->parents.end(); ++parent) {
            // All parents must be of same kind as child
            if ((*parent)->kind != node->kind) {
                state->error() << "Illegal parent " << (*parent) << " for node " << node;
            }

            // All parents must have the same extensibility
            if (get_extensibility(*parent) != get_extensibility(node)) {
                state->error() << "Illegal extensibility on " << (*parent) << " for node " << node
                               << ": derived types may not differ in extensibility. Parent is "
                               << get_extensibility(*parent) << ", child is "
                               << get_extensibility(node);
            }
        }

        validate_node(state, node->type);
        validate_node(state, node->key_type);
        validate_node(state, node->element_type);
    }
}

void validate_tree(parser_state* state, ptree* node) {
    node_kind tree_types[] = {
        N_ANNOTATION_DEF,
        N_MODULE,
        N_STRUCT,
        N_UNION,
        N_VALUETYPE,
        N_INTERFACE,
        N_PROTOTYPE,
        N_EXCEPTION,
        N_ENUM,
        N_BITSET,
        N_BITMASK,
        N_CASE,
        N_CONST,
        N_MEMBER,
        N_ALIAS,
        N_NULL,
        N_ANNOTATION,
        N_UNDEF
    };

    while (node) {
        // Node found in traversal must be one of the valid tree types
        if (!is_of_type(node, tree_types)) {
            state->error() << "Unexpected node " << node << " of type " << node->kind << " in tree";
        }

        validate_node(state, node);
        validate_tree(state, node->members);
        node = node->next;
    }
}

void format_doxy_comments(parser_state* state, ptree* tree) {
    while (tree) {
        format_doxy_comments(state, tree->members);
        for (auto ann : tree->annotations) {
            if (ann->type == annotation_type_doc) {
                for (auto text : ann->members) {
                    if (std::string_view(text->name) == "text") {
                        text->value.val.str(format_docstring(
                            text->value.val.str().c_str(),
                            value<int32_t>(get_annotation_value(ann, "placement"))
                        ));
                    }
                }
            }
        }
        tree = tree->next;
    }
}
}

ptree* parser_state::lookup_node(const char* a_name) const {
    auto it = type_map.find(std::string("::") + tolower(a_name));
    return it != type_map.end() ? it->second : nullptr;
}

parser_state::error_stream parser_state::error() {
    return parser_state::error_stream(this);
}
