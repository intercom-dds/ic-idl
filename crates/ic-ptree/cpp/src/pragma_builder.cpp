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

#include <cstdlib>
#include <cstring>
#include <iostream>
#include <sstream>
#include <vector>

#include "cidl/commandline.h"
#include "cidl/ptree_builder.h"

namespace {
std::vector<std::string> g_pragma_command;

bool string_ends_with(const std::string& pragma, const std::string& end) {
    return pragma.size() >= end.size() && pragma.substr(pragma.size() - end.size()) == end;
}

ptree* lookup_member(ptree* node, const char* name) {
    std::vector<std::string> parts;
    std::string to_split = name;
    size_t pos = 0;
    size_t prev_pos = 0;
    while ((pos = to_split.find('.', prev_pos)) != std::string::npos) {
        parts.push_back(to_split.substr(prev_pos, pos - prev_pos));
        prev_pos = pos + 1;
    }
    parts.push_back(to_split.substr(prev_pos, to_split.size() - prev_pos));

    pos = 0;
    ptree* element = nullptr;
    while (node && pos < parts.size()) {
        ptree* prev_node = node;
        prev_pos = pos;
        if (parts[pos] == "_d" && node->kind == N_UNION) {
            element = node->discriminator;
            node = element->type;
            ++pos;
        } else if (node->kind == N_ARRAY || node->kind == N_SEQUENCE) {
            if (parts[pos] == "[]") {
                node = node->element_type;
                ++pos;
            }
        } else {
            for (auto m : node->members) {
                if (m->kind == N_MEMBER && parts[pos] == m->name) {
                    element = m;
                    node = element->type;
                    ++pos;
                    break;
                }
            }
        }
        while (node && node->kind == N_ALIAS) {
            node = node->type;
        }
        if (prev_node == node && prev_pos == pos) {
            break;
        }
    }
    if (pos != parts.size() && !(pos == parts.size() - 1 && parts[pos] == "_length")) {
        element = nullptr;
    }
    return element;
}

bool pragma_finish_impl() {
    if (g_pragma_command.size() < 2) {
        if (g_pragma_command[0] == "INTERCOM_DOC") {
            intercom::cidl::g_state->current_under_documentation.reset();
            return true;
        }
        return false;
    }
    if (g_pragma_command[0] == "INTERCOM_KONGSBERG_COPYRIGHT") {
        return true;
    }

    ptree* node = try_lookup_node(g_pragma_command[1].c_str(), ANY_KIND);
    if (!node) {
        // Try lookup again with "." changed to "::"
        std::stringstream replace_dots;
        for (const char* c = g_pragma_command[1].c_str(); *c; ++c) {
            if (*c == '.') {
                replace_dots << "::";
            } else {
                replace_dots << *c;
            }
        }
        node = try_lookup_node(replace_dots.str().c_str(), ANY_KIND);
        if (!node) {
            if (g_pragma_command[0] != "INTERCOM_DOC") {
                return false;
            }
        }
    }
    ptree* member = nullptr;
    if (g_pragma_command.size() >= 3) {
        member = lookup_member(node, g_pragma_command[2].c_str());
        if (!member) {
            return false;
        }
    }
    if (g_pragma_command[0] == "DCPS_DATA_TYPE" || g_pragma_command[0] == "INTERCOM_DLL_EXPORT" ||
        g_pragma_command[0] == "INTERCOM_PACKED_BIT_LITTLE" ||
        g_pragma_command[0] == "INTERCOM_STRING_TERM") {
    } else if (g_pragma_command[0] == "DCPS_DATA_KEY") {
        if (!member) {
            return false;
        }
        create_annotation_start(create_identifier("@key"));
        annotate(member, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_SUPPRESS") {
        create_annotation_start(create_identifier("@ext::suppress"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "DCPS_NOT_DATA_TYPE") {
        create_annotation_start(create_identifier("@nested"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_LISTENER") {
        create_annotation_start(create_identifier("@ext::listener"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_CLASS_WIDE") {
        create_annotation_start(create_identifier("@static"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_NO_CONSTRUCTOR") {
        create_annotation_start(create_identifier("@ext::no_constructor"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_NO_SERIALIZER") {
        create_annotation_start(create_identifier("@ext::no_serializer"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_CONST") {
        create_annotation_start(create_identifier("@const"));
        annotate(node, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_DOC") {
        // This can also be a nullptr if documentation node does not exist.
        // If so, any doxy comments will be suppressed since there is no node
        // to attach them to.
        intercom::cidl::g_state->current_under_documentation = node;
    } else if (g_pragma_command[0] == "INTERCOM_VMF_XRI") {
        if (!member) {
            return false;
        }
        create_annotation_start(create_identifier("@ext::vmf_xri"));
        annotate(member, create_annotation_finish(nullptr));
    } else if (g_pragma_command[0] == "INTERCOM_FIELD_WIDTH") {
        std::string annotation = "@bit_bound";
        if (string_ends_with(g_pragma_command[2], "._length")) {
            annotation = "@ext::length_bit_bound";
        }
        if (!member || g_pragma_command.size() < 4) {
            return false;
        }
        ptree* param = create_node(N_CONST, create_identifier("value"));
        param->value.val.us(
            static_cast<unsigned short>(strtol(g_pragma_command[3].c_str(), nullptr, 0))
        );

        create_annotation_start(create_identifier(annotation.c_str()));
        if (member->type->kind == N_ALIAS || member->type->kind == N_ENUM) {
            member = member->type;
        }
        if (annotation == "@bit_bound" && member->kind == N_ALIAS &&
            member->type->kind == N_SEQUENCE) {
            member = member->type->element_type;
        }
        annotate(member, create_annotation_finish(param));
    } else if (g_pragma_command[0] == "INTERCOM_FIELD_VALUE_OFFSET") {
        std::string annotation = "@ext::value_offset";
        if (!member || g_pragma_command.size() < 4) {
            return false;
        }
        ptree* param = create_node(N_CONST, create_identifier("value"));
        param->value.val.l(static_cast<int>(strtol(g_pragma_command[3].c_str(), nullptr, 0)));

        create_annotation_start(create_identifier(annotation.c_str()));
        annotate(member, create_annotation_finish(param));
    } else if (g_pragma_command[0] == "INTERCOM_REPEAT_COUNT") {
        ptree* repeater;
        if (!member || g_pragma_command.size() < 4 ||
            !(repeater = lookup_member(node, g_pragma_command[3].c_str()))) {
            return false;
        }
        ptree* param = create_node(N_CONST, create_identifier("value"));
        param->value.val.node(repeater);

        create_annotation_start(create_identifier("@ext::repeat_count"));
        annotate(member, create_annotation_finish(param));
    } else {
        return false;
    }
    return true;
}

}  // namespace

extern "C" {

void pragma_finish() {
    if (g_pragma_command.empty()) {
        return;
    }
    pragma_finish_impl();
    g_pragma_command.clear();
}

void pragma_arg(const char* pragma) {
    g_pragma_command.emplace_back(pragma);
}
}
