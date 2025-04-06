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

#include <iostream>
#include <optional>

#include "cidl/commandline.h"
#include "cidl/idl_parser.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "icgen/template/casing.h"
#include "rust_common.h"

using namespace intercom::cidl;
using namespace intercom::rust;

static ptree* append_to_list(ptree* list, ptree* node) {
    if (!node || node == list) {
        return list;
    }
    if (!list) {
        return node;
    }

    auto last = list;
    while (last->next) {
        last = last->next;
    }
    last->next = node;
    return list;
}

static ptree* remove_node(ptree* list, ptree* node) {
    while (list == node) {
        list = list->next;
    }
    for (auto a : list) {
        if (a->next == node) {
            a->next = node->next;
        }
    }
    node->next = nullptr;
    return list;
}

static bool can_have_subtype(const ptree* node) {
    return node->kind == N_MODULE || node->kind == N_INTERFACE || node->kind == N_VALUETYPE ||
           node->kind == N_STRUCT || node->kind == N_EXCEPTION;
}

static void flag_trivial_ord(ptree* node, std::set<ptree*>& seen) {
    if (!node || !seen.insert(node).second) {
        return;
    }

    // Treat the type as trivial and ordinary until proven otherwise.
    // This is necessary to properly mark recursive types.
    node->flags |= OPT_RUST_TRIVIAL | OPT_RUST_TOTAL_ORDER;

    if (node == &float_type || node == &double_type) {
        node->flags &= ~OPT_RUST_TOTAL_ORDER;
    }
    if (is_shared(node) || (node->flags & OPT_CIRCULAR) != 0 || node->kind == N_MAP ||
        node->kind == N_SEQUENCE || node->kind == N_PROTOTYPE || node->kind == N_STRING ||
        node->kind == N_STRING || node->kind == N_INTERFACE || node->kind == N_MODULE) {
        node->flags &= ~OPT_RUST_TRIVIAL;
    }

    auto check_node = [&](ptree* obj) {
        flag_trivial_ord(obj, seen);
        if (obj) {
            if ((obj->flags & OPT_RUST_TRIVIAL) == 0) {
                node->flags &= ~OPT_RUST_TRIVIAL;
            }
            if ((obj->flags & OPT_RUST_TOTAL_ORDER) == 0) {
                node->flags &= ~OPT_RUST_TOTAL_ORDER;
            }
        }
    };

    check_node(node->type);
    check_node(node->key_type);
    check_node(node->element_type);
    std::for_each(node->parents.begin(), node->parents.end(), check_node);
    for (auto mem : node->members) {
        check_node(mem);
    }
}

/// Flags trivial types, and types that can form a total order. This is
/// necessary for Rust to determine the correct attributes to place on a type.
static void flag_trivial_ord(ptree* node) {
    std::set<ptree*> seen;
    for (; node; node = node->next) {
        flag_trivial_ord(node, seen);
    }
}

/// Object/any can't be serialized, so we mark the relevant members as
/// @non_serialized to avoid build errors.
static void annotate_any(parser_state* state, ptree* node) {
    for (; node; node = node->next) {
        if (node->kind == N_MEMBER) {
            auto base = base_type_of(node->type);
            if (base == &object_type || base == &any_type) {
                create_annotation_start(state, "@non_serialized");
                annotate(state, node, create_annotation_finish(state, nullptr));
            }
        } else {
            annotate_any(state, node->members);
        }
    }
}

/// Removes duplicate modules and squashes them into one. This will also remove
/// any non-emit modules from the tree.
static ptree*
squash_modules(parser_state* state, ptree* node, std::map<std::string, ptree*>& modules) {
    ptree* list = node;
    while (node) {
        ptree* next = node->next;
        if (node->kind == N_MODULE) {
            node->members = squash_modules(state, node->members, modules);

            auto it = modules.find(lc_scoped_name(node));
            if (it == modules.end()) {
                modules.emplace(lc_scoped_name(node), node);
            } else {
                list = remove_node(list, node);
                ptree* next_mem = nullptr;
                auto target = it->second;

                for (auto mem = node->members; mem; mem = next_mem) {
                    next_mem = mem->next;
                    mem->next = nullptr;
                    mem->scope = mem->super = target;
                    target->members = append_node(state, target->members, mem);
                }
            }
        }
        node = next;
    }
    return list;
}

static void move_nested(parser_state* state, ptree* node, ptree* scope, std::set<ptree*>& moved) {
    if (can_have_subtype(node) && node->kind != N_MODULE) {
        for (auto mem = node->members; mem;) {
            ptree* next = mem->next;
            if (mem->kind != N_MEMBER && mem->kind != N_PROTOTYPE) {
                // 1. Detach the member from the list
                node->members = remove_node(node->members, mem);

                // 2. Create an appropriate module for the type and rescope the type
                create_module_start(state, mod_name(node).c_str());
                auto mod = create_module_finish(state, mem);
                mem->scope = mem->super = mod;

                // 3. Move the newly created module to the parent scope
                if (scope->kind == N_MODULE) {
                    mod->scope = mod->super = scope;
                    scope->members = append_to_list(scope->members, mod);
                } else {
                    mod->scope = mod->super = nullptr;
                    scope = append_to_list(scope, mod);
                }

                // 4. Continue traversal the moved node
                move_nested(state, mem, mod, moved);
                moved.insert(mem);
            }
            mem = next;
        }
    }

    if (can_have_subtype(node)) {
        for (auto mem : node->members) {
            move_nested(state, mem, node, moved);
        }
    }
}

static void rescope_dds(ptree* node) {
    for (; node; node = node->next) {
        if (node->kind == N_MODULE) {
            // Depth-first traversal to properly capture the qualified name
            rescope_dds(node->members);

            // Capitalization of "X" and "T" leads to "x_types" in snake case,
            // which is a bit unfortunate.
            if (idl_scoped_name(node, nullptr) == "DDS::XTypes") {
                node->name = "xtypes";
            }

            // Generated DDS types are located in intercom::types
            // if (node->super == nullptr && (node->name == "DDS" || node->name == "intercom")) {
            //     node->name = "types";
            // }
        }
    }
}

static void replace_native(parser_state* state) {
    // `InstanceHandle` in Rust is a tuple struct. Since bitmasks are also
    // tuple structs, we can transform `InstanceHandle_t` into a bitmask so
    // that the generated code treats it accordingly. The node is then suppressed
    // since it's already defined in the API (and it's not really a bitmask).
    auto to_bitmask = [&](const char* name, const char* new_name) {
        if (auto node = state->lookup_node(name)) {
            auto handle = create_bitmask(state, new_name, nullptr);
            create_annotation_start(state, "@ext::suppress");
            annotate(state, handle, create_annotation_finish(state, nullptr));

            auto next = node->next;
            *node = *handle;
            node->next = next;
        }
    };

    (void)&to_bitmask;

    // create_module_start(create_identifier("core"));
    // to_bitmask("DDS::InstanceHandle_t", "InstanceHandle");
    // to_bitmask("DDS::SampleStateKind", "SampleState");
    // to_bitmask("DDS::SampleStateMask", "SampleState");
    // to_bitmask("DDS::InstanceStateKind", "InstanceState");
    // to_bitmask("DDS::InstanceStateMask", "InstanceState");
    // to_bitmask("DDS::ViewStateKind", "ViewState");
    // to_bitmask("DDS::ViewStateMask", "ViewState");
    // create_module_finish(nullptr, tree->pos);
}

static std::optional<std::string> conventionalized(ptree* node) {
    switch (node->kind) {
    case N_PRIMITIVE:
    case N_SEQUENCE:
    case N_MAP:
    case N_ARRAY:
        return std::nullopt;
    case N_MODULE:
        return mod_name(node);
    case N_PROTOTYPE:
        return fn_name(node);
    case N_MEMBER:
        return node->super->kind == N_UNION ? type_name(node) : member_name(node);
    case N_CONST:
        if (node->type->kind == N_ENUM && node->value.kind() != PTREE_KIND) {
            return type_name(node);
        }
        return const_name(node);
    default:
        return type_name(node);
    }
}

static std::set<std::string> collect_names(const ptree* node) {
    std::set<std::string> names;
    for (; node; node = node->next) {
        names.insert(node->name);
    }
    return names;
}

static void rename_breadth(ptree* node, const std::set<ptree*>& moved) {
    auto orig_names = collect_names(node);

    std::set<std::string> renamed;
    for (; node; node = node->next) {
        if (auto name = conventionalized(node)) {
            while ((name != node->name && (orig_names.count(*name) || renamed.count(*name))) ||
                   (!renamed.insert(*name).second && moved.count(node))) {
                *name += "_";
            }
            orig_names.erase(node->name);
            node->name = *name;
        }
    }
}

static void rename_tree(ptree* node, const std::set<ptree*>& moved) {
    if (node) {
        rename_breadth(node, moved);
        for (; node; node = node->next) {
            rename_tree(node->members, moved);
        }
    }
}

static bool starts_with_alpha(std::string_view name) {
    for (auto c : name) {
        if (std::isalpha(c)) {
            return true;
        }
        if (c != '_') {
            return false;
        }
    }
    return false;
}

static size_t rfind_delimiter(std::string_view name) {
    bool was_upper = false;
    for (size_t i = name.size() - 1; i > 0; i--) {
        auto c = name[i];
        if (i >= 1) {
            auto peek = name[i - 1];

            if (peek == '_' || (islower(c) && isupper(peek)) ||
                (was_upper && isupper(c) && islower(peek))) {
                return i - 1;
            }
        }
        was_upper = isupper(c) != 0;
    }
    return std::string_view::npos;
}

/// If all enumerators have a prefix that is also found in the name of the enum
/// itself, this function will strip that prefix from the names of the
/// enumerators.
///
/// For example:
/// ```
///     enum Color { COLOR_RED, COLOR_GREEN };
/// ```
/// will be converted to:
/// ```
///     enum Color { RED, GREEN };
/// ```
static void strip_prefix(const ptree* node) {
    size_t pos = 0;
    auto first = std::string_view(node->members->name);
    auto prefix = first.substr(0, rfind_delimiter(first));

    while (pos != std::string_view::npos) {
        // Check if all enumerators have a shared prefix
        bool has_prefix = std::all_of(begin(node->members), end(node->members), [&](ptree* mem) {
            if (mem->name.size() > prefix.size()) {
                auto remainder = std::string_view(mem->name).substr(prefix.size());
                auto view = std::string_view(mem->name).substr(0, prefix.size());
                return starts_with_alpha(remainder) && view == prefix;
            }
            return false;
        });

        if (has_prefix) {
            auto found_prefix = intercom::icgen::snake_case(prefix);
            auto type_name = intercom::icgen::snake_case(node->name);

            // Check if the type name contains the same prefix, though it may
            // be written with a different naming convention.
            if (type_name.size() >= found_prefix.size() &&
                type_name.substr(0, found_prefix.size()) == found_prefix) {
                for (auto mem : node->members) {
                    mem->name = mem->name.substr(prefix.size());
                }
                break;
            }
        }

        // Find the next delimiter and try again
        pos = rfind_delimiter(prefix);
        prefix = prefix.substr(0, pos);
    }
}

static void enum_prefix(const ptree* node) {
    if (CommandLineOption::no_rename()) {
        return;
    }

    for (; node; node = node->next) {
        if (node->kind == N_ENUM || node->kind == N_BITMASK) {
            strip_prefix(node);
        } else if (node->members) {
            enum_prefix(node->members);
        }
    }
}

static void dump_names(const ptree* node) {
    for (; node; node = node->next) {
        std::cout << idl_scoped_name(node, nullptr) << std::endl;
        dump_names(node->members);
    }
}

void intercom::rust::transform_rust(parse_result* result) {
    auto tree = const_cast<ptree*>(result->tree);

    // Flag trivial types and types that can form a total order
    flag_trivial_ord(tree);

    // Annotate all members whose type is any/Object with @non_serialized
    annotate_any(result->state.get(), tree);

    // Move nested types into modules. Keep track of the moved nodes to
    // properly escape their names later on to ensure the correct node gets
    // precedence.
    std::set<ptree*> moved;
    for (auto node = tree; node; node = node->next) {
        move_nested(result->state.get(), node, tree, moved);
    }

    // Squash duplicate modules into one
    std::map<std::string, ptree*> modules;
    result->tree = tree = squash_modules(result->state.get(), tree, modules);

    // Replace some DDS types with their native Rust equivalents
    replace_native(result->state.get());

    // Give select modules more suitable names
    rescope_dds(tree);

    // Strip prefixes from enumerators
    enum_prefix(tree);

    // Rename nodes so they conform with Rust's naming convention
    rename_tree(tree, moved);
}
