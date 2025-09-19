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

#include "cidl/symbols.h"

#include <algorithm>
#include <cctype>
#include <cstring>
#include <sstream>
#include <string>

#include "cidl/commandline.h"
#include "cidl/hdrs.h"
#include "cidl/keywords.h"
#include "cidl/ptree_helpers.h"
#include "ic_cts/integer_types.h"
#include "utils/md5.h"

static void idl_scoped_name_rec(
    const ptree* node,
    const ptree* scope,
    const ptree* common_parent,
    std::stringstream& out
) {
    if (node) {
        if (scope != common_parent) {
            idl_scoped_name_rec(scope, scope ? scope->super : nullptr, common_parent, out);
            if (!out.str().empty() && !node->name.empty()) {
                out << "::";
            }
        }
        out << node->name;
    }
}

namespace intercom::cidl {

std::string safe_name(const ptree* node, const std::string& name, Language lang) {
    std::string res = name;
    std::string lookup = res;
    if (lang == LANG_ADA) {
        std::string lcname = name;
        transform(lcname.begin(), lcname.end(), lcname.begin(), ::tolower);
        lookup = lcname;
    }
    const char* const* keywords = nullptr;
    switch (lang) {
    case LANG_CPP:
        keywords = CPP_KEYWORDS;
        break;
    case LANG_CS:
        keywords = CS_KEYWORDS;
        break;
    case LANG_JAVA:
        keywords = JAVA_KEYWORDS;
        break;
    case LANG_ADA:
        keywords = ADA_KEYWORDS;
        break;
    case LANG_PYTHON:
        keywords = PYTHON_KEYWORDS;
        break;
    case LANG_IDL:
        keywords = IDL_KEYWORDS;
        break;
    case LANG_RUST:
        keywords = RUST_KEYWORDS;
        break;
    case LANG_PROTO:
        keywords = PROTO_KEYWORDS;
        break;
    case LANG_NONE:
        break;
    }
    if (keywords) {
        for (auto keyword = keywords; *keyword; ++keyword) {
            if (*keyword == lookup) {
                std::string safe_name = name;
                switch (lang) {
                case LANG_CPP:
                case LANG_CS:
                case LANG_PYTHON:
                case LANG_RUST:
                case LANG_PROTO:
                    safe_name += "_";
                    break;
                case LANG_JAVA:
                case LANG_IDL:
                    safe_name = "_" + safe_name;
                    break;
                case LANG_ADA:
                    safe_name = "IDL_" + safe_name;
                    break;
                case LANG_NONE:
                    break;
                }

                res = safe_name;
                break;
            }
        }
    }

    if (lang == LANG_ADA) {
        std::string_view view = res;

        // Remove any leading underscore
        for (auto c : view) {
            if (c != '_') {
                break;
            }
            view = view.substr(1);
        }

        // Replace any second consecutive underscore with 'U'
        std::string buf(view);
        for (size_t i = 0; i < buf.length(); i++) {
            if (i < buf.length() + 1) {
                if (buf[i] == '_' && buf[i + 1] == '_') {
                    buf[i + 1] = 'U';
                }
            }
        }

        // Add 'U' if ending in underscore
        if (buf[buf.length() - 1] == '_') {
            buf += 'U';
        }
        res = buf;
    } else if (lang == LANG_CPP) {
        // exceptions inherit from std::runtime_error, which defines a virtual `what` function.
        if (node->super && node->super->kind == N_EXCEPTION &&
            CommandLineOption::cpp_access_functions() && node->name == "what") {
            res = "what_";
        }
    }
    return res;
}

std::string cpp_name(const ptree* node) {
    return node ? safe_name(node, node->name, LANG_CPP) : "";
}

std::string java_name(const ptree* node) {
    return node ? safe_name(node, node->name, LANG_JAVA) : "";
}

std::string ada_name(const ptree* node) {
    return node ? safe_name(node, node->name, LANG_ADA) : "";
}

std::string cs_name(const ptree* node) {
    return node ? safe_name(node, node->name, LANG_CS) : "";
}

std::string python_name(const ptree* node) {
    return node ? safe_name(node, node->name, LANG_PYTHON) : "";
}

std::string idl_name(const ptree* node) {
    if (!node) {
        return {};
    }
    if (node->kind == N_PRIMITIVE || node->kind == N_STRING || node->kind == N_ANNOTATION ||
        node->kind == N_ANNOTATION_DEF) {
        if (CommandLineOption::legacy_idl()) {
            if (node->name == "int16") {
                return "short";
            }
            if (node->name == "uint16") {
                return "unsigned short";
            }
            if (node->name == "int32") {
                return "long";
            }
            if (node->name == "uint32") {
                return "unsigned long";
            }
            if (node->name == "int64") {
                return "long long";
            }
            if (node->name == "uint64") {
                return "unsigned long long";
            }
        }
        return node->name;
    }
    return safe_name(node, node->name, LANG_IDL);
}

const ptree* common_scope(const ptree* node, const ptree* context) {
    if (!context) {
        return nullptr;
    }
    if (base_type_of(context)->kind != N_ENUM && base_type_of(context)->kind != N_BITMASK) {
        // If node is a child of the context, use context as common scope
        auto context_name = idl_scoped_name(context, nullptr);
        for (const ptree* p1 = node; p1; p1 = p1->super) {
            if (idl_scoped_name(p1, nullptr) == context_name) {
                return p1;
            }
        }
    }

    // If node is a child of the namespace of context, use namespace of context as common scope
    // Treat interfaces as namespaces here, since an interface may contain a struct
    const ptree* context_namespace = context->super;
    while (context_namespace && context_namespace->kind != N_MODULE &&
           context_namespace->kind != N_INTERFACE) {
        context_namespace = context_namespace->super;
    }
    if (context_namespace) {
        auto context_namespace_name = idl_scoped_name(context_namespace, nullptr);
        for (const ptree* p1 = node; p1; p1 = p1->super) {
            if (idl_scoped_name(p1, nullptr) == context_namespace_name) {
                return p1;
            }
        }
    }
    // Otherwise, use top level scope
    return nullptr;
}

const ptree* namespace_of(const ptree* node) {
    while (node && node->kind != N_MODULE) {
        node = node->super;
    }
    return node;
}

std::string module_name(const ptree* node) {
    while (node && node->kind != N_MODULE) {
        node = node->super;
    }
    return idl_scoped_name(node, nullptr);
}

uint32_t member_name_hash_id(const std::string& name) {
    uint32_t hash_id = 0;
    if (!name.empty()) {
        intercom::MD5 md5(reinterpret_cast<const unsigned char*>(name.c_str()), name.size());
        ic_cts::get_uint<ic_cts::LittleEndian>(md5.digest().data(), hash_id);
        hash_id &= 0x0fffffffU;
    }
    return hash_id;
}

std::string idl_scoped_name_impl(const ptree* node, const ptree* scope, const ptree* context) {
    if (node == context || scope == context) {
        return idl_name(node);
    }
    std::stringstream name;
    idl_scoped_name_rec(node, scope, common_scope(scope, context), name);
    return name.str();
}

std::string idl_scoped_name(const ptree* node, const ptree* context) {
    if (node) {
        if (node->kind == N_ANNOTATION) {
            return idl_scoped_name_impl(node, node->type->super, context);
        }
        const bool in_enum =
            node->super && (node->super->kind == N_ENUM || node->super->kind == N_BITMASK);
        return idl_scoped_name_impl(node, in_enum ? node->super->super : node->super, context);
    }
    return {};
}

static std::string idl_internal_scoped_name(const ptree* node, const ptree* context) {
    return idl_scoped_name_impl(node, node->super, context);
}

std::string lc_scoped_name(const ptree* p) {
    return std::string("::") + intercom::cidl::tolower(idl_internal_scoped_name(p, nullptr));
}

}  // namespace intercom::cidl
