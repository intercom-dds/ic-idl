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
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "utils/string_utils.h"

using namespace intercom::cidl;

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
                create_annotation_start(state, "@non_serialized", annotation_type_non_serialized);
                annotate(state, node, create_annotation_finish(state, nullptr));
            }
        } else {
            annotate_any(state, node->members);
        }
    }
}

static void rescope_dds(parser_state* state, ptree* node) {
    for (; node; node = node->next) {
        if (node->kind == N_MODULE) {
            // Depth-first traversal to properly capture the qualified name
            rescope_dds(state, node->members);

            // Capitalization of "X" and "T" leads to "x_types" in snake case,
            // which is a bit unfortunate.
            if (idl_scoped_name(node, nullptr) == "DDS::XTypes") {
                node->name = "xtypes";
            }

            // Generated DDS types are located in intercom::types
            // if (node->super == nullptr && (node->name == "DDS" || node->name == "intercom")) {
            //     node->name = "types";
            // }
        } else {
            auto qualified = idl_scoped_name(node, nullptr);
            if (string_utils::starts_with(qualified, "intercom::wire")) {
                {
                    auto pos = node->name.find("UDPv4");
                    if (pos != std::string::npos) {
                        node->name.replace(pos, 5, "UDP_V4");
                    }
                }
                {
                    auto pos = node->name.find("UDPv6");
                    if (pos != std::string::npos) {
                        node->name.replace(pos, 5, "UDP_V6");
                    }
                }
            }
        }
    }
}

void transform_rust(parse_result* result) {
    auto tree = const_cast<ptree*>(result->tree);
    auto state = result->state.get();

    // Flag trivial types and types that can form a total order
    flag_trivial_ord(tree);

    // Annotate all members whose type is any/Object with @non_serialized
    annotate_any(state, tree);

    // Give select modules more suitable names
    rescope_dds(state, tree);
}
