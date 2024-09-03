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

#include <fmt/color.h>
#include <fmt/format.h>

#include <array>
#include <iostream>

#include "cidl/hdrs.h"
#include "cidl/pretty_printer.h"

using namespace intercom::cidl;

namespace {
class ScopedPrinter {
  public:
    explicit ScopedPrinter(ScopedPrinter* scope) : m_scope(scope) {
        if (scope) {
            m_out = scope->m_out << endl;
            indent();
        }
    }

    ScopedPrinter(ScopedPrinter&) = delete;

    ~ScopedPrinter() {
        m_out << endl;
    }

    template <typename T>
    ScopedPrinter& operator<<(const T& value) {
        m_out << value;
        return *this;
    }

    std::string str() const {
        return m_out.str();
    }

    void indent() {
        auto parent = m_scope;
        while (parent) {
            auto fmt = parent->m_scope ? "| " : "|-";
            m_out << fmt::format(fg(fmt::terminal_color::blue), fmt);
            parent = parent->m_scope;
        }
    }

  private:
    PrettyPrinter m_out;
    ScopedPrinter* m_scope;
};
}  // namespace

static void recurse_node(ScopedPrinter&, const ptree*);

static std::string kind_name(node_kind kind) {
    std::array<const char*, 26> names = {
        "Undef",         "Include",    "Primitive", "Native",     "ModuleDef", "StructDef",
        "UnionDef",      "Valuetype",  "Interface", "Exception",  "EnumDef",   "BitsetDef",
        "BitmaskDef",    "Case",       "Null",      "MemberDecl", "Proto",     "Seq",
        "Map",           "Array",      "String",    "Fixed",      "Alias",     "ConstDef",
        "AnnotationDef", "Annotation",
    };
    static_assert(names.size() == node_kind::N_ANNOTATION + 1);
    return names[kind];
}

static std::string decl(const ptree* node) {
    auto kind = kind_name(node->kind);
    return fmt::format(fmt::emphasis::bold | fg(fmt::terminal_color::green), "{} ", kind);
}

static std::string addr(const ptree* node) {
    return fmt::format(fg(fmt::terminal_color::blue), "{} ", fmt::ptr(node));
}

static std::string attrib(const char* name) {
    return fmt::format(fg(fmt::terminal_color::magenta), "{} ", name);
}

static std::string name(const ptree* node) {
    auto name = node->name.empty() ? "<anon>" : node->name;
    return fmt::format(fmt::emphasis::bold | fg(fmt::terminal_color::cyan), "{} ", name);
}

static std::string type(const ptree* node, const ptree* scope) {
    auto name = idl_scoped_name(node, scope);
    return fmt::format(fg(fmt::terminal_color::green), "'{}' ", name);
}

static bool is_complex_type(const ptree* node) {
    return node->kind == N_ENUM || node->kind == N_STRUCT || node->kind == N_UNION ||
           node->kind == N_BITMASK;
}

static void emit_flags(ScopedPrinter& out, const ptree* node) {
    if (node->flags & OPT_DECLARATION) {
        out << "decl ";
    }
    if (node->flags & OPT_INOUT) {
        out << "inout ";
    } else if (node->flags & OPT_IN) {
        out << "in ";
    } else if (node->flags & OPT_OUT) {
        out << "out ";
    }
    if (node->flags & OPT_READONLY) {
        out << "readonly ";
    }
    if (node->flags & OPT_PRIVATE) {
        out << "private ";
    }
    if (node->flags & OPT_DEFAULT) {
        out << "default ";
    }
    if (node->flags & OPT_HAS_CHILDREN) {
        out << "has_children ";
    }
    if (node->flags & OPT_ENUMERATED) {
        out << "enumerated ";
    }
    if (node->flags & OPT_EMIT_CODE) {
        out << "emit ";
    }
    if (node->flags & OPT_SYSTEM_INCLUDE) {
        out << "system_inc ";
    }
    if (node->flags & OPT_CIRCULAR) {
        out << "circular ";
    }
    if (node->flags & OPT_SEQUENCE_LENGTH) {
        out << "sequence_len ";
    }
    if (node->flags & OPT_CONST_VALUE) {
        out << "const_val ";
    }
    if (node->flags & OPT_ANONYMOUS_ALIAS) {
        out << "anon_alias ";
    }
    if (node->flags & OPT_RUST_TRIVIAL) {
        out << "trivial ";
    }
    if (node->flags & OPT_RUST_TOTAL_ORDER) {
        out << "total_ord ";
    }
    if (node->flags & OPT_LOCAL) {
        out << "local ";
    }
    if (node->flags & OPT_BUILTIN) {
        out << "builtin ";
    }
}

static void emit_value(ScopedPrinter& out, numeric val) {
    if (val.kind() == PTREE_KIND) {
        ScopedPrinter scope(&out);
        recurse_node(scope, val.val.node());
    } else {
        auto str = value<std::string>(val);
        if (val.kind() == STRING_KIND || val.kind() == CHAR_KIND) {
            out << fmt::format(fg(fmt::terminal_color::bright_magenta), "'= \"{}\"' ", str);
        } else {
            out << fmt::format(fg(fmt::terminal_color::bright_magenta), "'= {}' ", str);
        }
    }
}

static void recurse_node(ScopedPrinter& out, const ptree* node) {
    out << decl(node) << addr(node) << name(node);
    if (node->type) {
        out << type(node, node->type);
    }
    emit_flags(out, node);

    if (node->value.kind() != UNDEF_KIND) {
        emit_value(out, node->value);
    }

    if (node->key_type) {
        ScopedPrinter scope(&out);
        scope << attrib("key_type");
        recurse_node(scope, node->key_type);
    }

    if (node->element_type) {
        ScopedPrinter scope(&out);
        scope << attrib("element_type");
        recurse_node(scope, node->element_type);
    }

    if (node->type && !is_complex_type(node->type)) {
        ScopedPrinter scope(&out);
        scope << attrib("type");
        recurse_node(scope, node->type);
    }

    // Applied annotations
    for (auto ann : node->annotations) {
        ScopedPrinter scope(&out);
        recurse_node(scope, ann);
    }

    // Iterate all over members regardless of whether the node is intended to have members or not
    for (auto mem : node->members) {
        ScopedPrinter scope(&out);
        recurse_node(scope, mem);
    }

    for (auto mem : node->generated) {
        ScopedPrinter scope(&out);
        recurse_node(scope, mem);
    }
}

void intercom::cidl::ptree_dump(const parse_result* result) {
    ScopedPrinter out(nullptr);
    for (auto node : result->tree) {
        recurse_node(out, node);
        out << endl;
    }
    std::cout << out.str() << std::flush;
}
