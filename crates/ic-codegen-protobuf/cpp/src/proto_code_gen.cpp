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

#include <fmt/format.h>

#include <algorithm>
#include <filesystem>
#include <iostream>

#include "cidl/commandline.h"
#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_ffi.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

using PtreeSet = std::set<const ptree*>;
using Path = std::vector<std::string>;
using PackageMap = std::map<const ptree*, PrettyPrinter>;
using DependencyMap = std::map<const ptree*, std::set<const ptree*>>;

static std::vector<std::string> package_path(const ptree*);

namespace {
class Printer {
  public:
    class Scope;

    void package(const ptree* mod) {
        m_curr = m_packages[mod];
    }

    const PackageMap& packages() const {
        return m_packages;
    }

    std::string str() const {
        return m_curr.str();
    }

    template <typename T>
    void operator()(T value) {
        m_curr << value;
    }

    template <typename T, typename... Args>
    void operator()(T value, Args... args) {
        operator()(std::forward<T>(value));
        operator()(std::forward<Args>(args)...);
    }

  private:
    PrettyPrinter m_curr;
    PackageMap m_packages;
};

class Printer::Scope {
  public:
    explicit Scope(Printer& out, const ptree* node) : m_printer(out) {
        m_prev = out.m_curr;
        out.package(node);
    }

    ~Scope() {
        m_printer.m_curr = m_prev;
    }

  private:
    Printer& m_printer;
    PrettyPrinter m_prev;
};
}  // namespace

static void recurse_node(Printer& out, const ptree* node);

std::string intercom::cidl::proto_name(const ptree* node) {
    return safe_name(node, node->name, LANG_PROTO);
}

static Path package_path(const ptree* node) {
    Path path;
    while ((node = node->scope)) {
        path.emplace_back(node->name);
    }
    std::reverse(path.begin(), path.end());
    return path;
}

static std::string package_name(const ptree* node) {
    auto path = package_path(node);
    return fmt::format("{}", fmt::join(path, "."));
}

static std::string file_name(const ptree* node) {
    std::filesystem::path file;
    auto path = package_path(node);
    for (const auto& segment : path) {
        file /= segment;
    }
    file /= proto_name(node);
    file.replace_extension(".proto");

    auto str = file.string();
    std::replace(str.begin(), str.end(), '/', '\\');
    return str;
}

static std::string scoped_name(const ptree* node, const ptree*) {
    return fmt::format("{}.{}", package_name(node), proto_name(node));
}

static std::string proto_value(const ptree* node) {
    if (node->value.kind() == PTREE_KIND) {
        return proto_value(node->value->node());
    }
    return string_value(node->value);
}

static bool is_proto_type(const ptree* node) {
    if (!node) {
        return false;
    }

    switch (node->kind) {
    case N_STRUCT:
    case N_EXCEPTION:
    case N_VALUETYPE:
    case N_UNION:
    case N_ENUM:
        return true;
    default:
        return false;
    }
}

static std::vector<const ptree*> struct_members(const ptree* node) {
    std::vector<const ptree*> members;
    std::function<void(const ptree*)> rec = [&](const ptree* obj) {
        for (auto parent : obj->parents) {
            rec(base_type_of(parent));
        }
        for (auto mem : obj->members) {
            if (mem->kind == N_MEMBER) {
                members.emplace_back(mem);
            }
        }
    };
    rec(base_type_of(node));
    return members;
}

static void collect_deps(const ptree* node, const ptree* root, PtreeSet& dependencies) {
    if (!node || dependencies.find(node) != dependencies.end()) {
        return;
    }

    auto insert = [&](const ptree* obj) {
        auto ty = base_type_of(obj);
        if (is_proto_type(ty) && ty != root) {
            dependencies.insert(ty);
        }
    };

    auto members = struct_members(node);
    for (auto mem : members) {
        auto ty = base_type_of(mem);
        insert(ty);
        insert(ty->key_type);
        insert(ty->element_type);
        insert(ty->discriminator);
    }
}

static PtreeSet find_deps(const ptree* node) {
    auto members = struct_members(node);
    PtreeSet direct_types;
    collect_deps(node, node, direct_types);
    return direct_types;
}

static std::string proto_type(const ptree* node, const ptree* context) {
    if (auto ann = get_annotation(node, annotation_type_ext_protobuf_type)) {
        return string_value(get_annotation_value(ann, "name"));
    }

    auto type = base_type_of(node);
    if (type == &boolean_type) {
        return "bool";
    }
    if (type == &int8_type || type == &short_type || type == &long_type) {
        return "int32";
    }
    if (type == &char_type || type == &wchar_type || type == &octet_type || type == &ushort_type ||
        type == &ulong_type) {
        return "uint32";
    }
    if (type == &longlong_type) {
        return "int64";
    }
    if (type == &ulonglong_type) {
        return "uint64";
    }
    if (type == &float_type) {
        return "float";
    }
    if (type == &double_type || type == &ldouble_type) {
        return "double";
    }
    if (type->kind == N_STRING) {
        return "string";
    }
    if (type->kind == N_BITMASK) {
        return proto_type(type->element_type, context);
    }
    if (type->kind == N_SEQUENCE || type->kind == N_ARRAY) {
        auto elem_ty = base_type_of(type->element_type);
        if (elem_ty == &octet_type || elem_ty == &int8_type) {
            return "bytes";
        }
        return fmt::format("repeated {}", proto_type(type->element_type, context));
    }
    if (type->kind == N_MAP) {
        auto key = proto_type(type->key_type, context);
        auto value = proto_type(type->element_type, context);
        return fmt::format("map<{}, {}>", key, value);
    }
    return scoped_name(type, context);
}

static void emit_message(Printer& out, const ptree* node) {
    out("message ", proto_name(node), " ", begin_curly, endl);

    int last_id = 0;
    auto members = struct_members(node);
    for (auto mem : members) {
        last_id = get_member_id(mem, node, last_id);
        out(proto_type(mem, node), " ", proto_name(mem), " = ", last_id, ";", endl);
    }

    for (auto mem : node->members) {
        if (is_proto_type(mem)) {
            out(blank_line);
            recurse_node(out, mem);
        }
    }
    out(end_curly, endl);
}

static void emit_enum(Printer& out, const ptree* node) {
    auto enum_name = proto_name(node);
    out("enum ", enum_name, " ", begin_curly, endl);

    for (auto mem : node->members) {
        out(proto_name(mem), " = ", proto_value(mem), ";", endl);
    }
    out(end_curly, endl);
}

static void emit_oneof(Printer& out, const ptree* node) {
    out("message ", proto_name(node), " ", begin_curly, endl);
    out("oneof inner ", begin_curly, endl);

    int last_id = 0;
    for (auto mem : node->members) {
        last_id = get_member_id(mem, node, last_id);
        if (mem->kind == N_MEMBER) {
            out(proto_type(mem, node), " ", proto_name(mem), " = ", last_id, ";", endl);
        }
    }

    out(end_curly, endl);
    out(end_curly, endl);
}

static void emit_prelude(Printer& out, const ptree* node) {
    auto path = package_path(node);
    out("syntax = \"proto3\";", blank_line);
    if (!path.empty()) {
        out("package ", package_name(node), ";", blank_line);
    }

    auto imports = find_deps(node);
    for (auto imp : imports) {
        out("import \"", file_name(imp), "\";", endl);
    }
    if (!imports.empty()) {
        out(blank_line);
    }
}

static void emit_package(const ptree* node, const PrettyPrinter& pkg, ic_list_t* list) {
    auto file = file_name(node);
    if (CommandLineOption::list_only()) {
        std::cout << file << std::endl;
    } else {
        Printer content;
        emit_prelude(content, node);
        content(pkg);

        std::filesystem::path out = CommandLineOption::proto_target_directory();
        out /= file;
        ic_push_source(list, out.c_str(), content.str().c_str());
    }
}

static void recurse_node(Printer& out, const ptree* node) {
    for (; node; node = node->next) {
        if (!is_emit(node, LANG_PROTO) || node->flags & OPT_DECLARATION) {
            continue;
        }

        Printer::Scope scope(out, node);
        switch (node->kind) {
        case N_STRUCT:
        case N_EXCEPTION:
        case N_VALUETYPE:
            emit_message(out, node);
            break;
        case N_UNION:
            emit_oneof(out, node);
            break;
        case N_ENUM:
            emit_enum(out, node);
            break;
        case N_MODULE:
            recurse_node(out, node->members);
            break;
        default:
            break;
        }
        out(blank_line);
    }
}

// TODO(idarcar): not used anywhere after refactoring
void validate_proto(parser_state* state, const ptree* node) {
    if (node->kind == N_ENUM) {
        if (long_long_value(node->members->value) != 0) {
            state->error() << "The first enum value must be zero in proto3";
        }
    }
}

void intercom::cidl::code_gen_proto(const parse_result* result, ic_list_t* list) {
    Printer out;
    recurse_node(out, result->tree);

    for (const auto& pkg : out.packages()) {
        if (!pkg.second.empty()) {
            emit_package(pkg.first, pkg.second, list);
        }
    }
}
