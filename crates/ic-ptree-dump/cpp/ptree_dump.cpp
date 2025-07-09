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

#include <iostream>

#include "cidl/hdrs.h"
#include "cidl/pretty_printer.h"
#include "color.h"

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
            m_out << ic::color::blue << fmt << ic::color::reset;
            parent = parent->m_scope;
        }
    }

  private:
    PrettyPrinter m_out;
    ScopedPrinter* m_scope;
};
}  // namespace

static void recurse_node(ScopedPrinter&, const ptree*, std::set<const ptree*>&);

static std::string kind_name(node_kind kind) {
    switch (kind) {
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
    case N_UNDEF:
    default:
        return "N_UNDEF";
    }
}

static std::string decl(const ptree* node) {
    auto kind = kind_name(node->kind);
    std::stringstream ss;
    ss << ic::color::bold << ic::color::green << kind << " " << ic::color::reset;
    return ss.str();
}

static std::string addr(const ptree* node) {
    std::stringstream ss;
    ss << ic::color::blue << node << " " << ic::color::reset;
    return ss.str();
}

static std::string attrib(const char* name) {
    std::stringstream ss;
    ss << ic::color::magenta << name << " " << ic::color::reset;
    return ss.str();
}

static std::string name(const ptree* node) {
    std::stringstream ss;
    auto name = node->name.empty() ? "<anon>" : node->name;
    ss << ic::color::bold << ic::color::cyan << name << " " << ic::color::reset;
    return ss.str();
}

static std::string type(const ptree* node, const ptree* scope) {
    std::stringstream ss;
    auto name = idl_scoped_name(node, scope);
    ss << ic::color::green << '\'' << name << "' " << ic::color::reset;
    return ss.str();
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

static void emit_value(ScopedPrinter& out, numeric val, std::set<const ptree*>& seen) {
    if (val.kind() == PTREE_KIND) {
        ScopedPrinter scope(&out);
        recurse_node(scope, val.val.node(), seen);
    } else {
        auto str = value<std::string>(val);
        if (val.kind() == STRING_KIND || val.kind() == CHAR_KIND) {
            size_t pos = 0;
            while ((pos = str.find('\n')) != std::string::npos) {
                str.replace(pos, str.length(), "\\n");
            }
            out << ic::color::bright_magenta << "'= \"" << str << "\"' " << ic::color::reset;
        } else {
            out << ic::color::bright_magenta << "'= '" << str << "' " << ic::color::reset;
        }
    }
}

static void recurse_node(ScopedPrinter& out, const ptree* node, std::set<const ptree*>& seen) {
    out << decl(node) << addr(node) << name(node);
    if (node->type) {
        out << type(node, node->type);
    }
    emit_flags(out, node);

    if (node->value.kind() != UNDEF_KIND) {
        emit_value(out, node->value, seen);
    }

    if (node->key_type) {
        ScopedPrinter scope(&out);
        scope << attrib("key_type");
        recurse_node(scope, node->key_type, seen);
    }

    if (node->element_type) {
        ScopedPrinter scope(&out);
        scope << attrib("element_type");
        recurse_node(scope, node->element_type, seen);
    }

    if (node->type && !is_complex_type(node->type) && seen.count(node->type) == 0) {
        ScopedPrinter scope(&out);
        scope << attrib("type");
        recurse_node(scope, node->type, seen);
    }

    // Applied annotations
    for (auto ann : node->annotations) {
        ScopedPrinter scope(&out);
        recurse_node(scope, ann, seen);
    }

    for (auto exception : node->setraises) {
        ScopedPrinter scope(&out);
        scope << attrib("setraises");
        recurse_node(scope, exception, seen);
    }

    for (auto exception : node->getraises) {
        ScopedPrinter scope(&out);
        scope << attrib("getraises");
        recurse_node(scope, exception, seen);
    }

    // Iterate all over members regardless of whether the node is intended to have members or not
    for (auto mem : node->members) {
        ScopedPrinter scope(&out);
        recurse_node(scope, mem, seen);
    }

    for (auto mem : node->generated) {
        ScopedPrinter scope(&out);
        recurse_node(scope, mem, seen);
    }
}

void intercom::cidl::ptree_dump(const parse_result* result) {
    ScopedPrinter out(nullptr);
    std::set<const ptree*> seen;
    for (auto node : result->tree) {
        recurse_node(out, node, seen);
        out << endl;
    }
    std::cout << out.str() << std::flush;
}

extern "C" {
void ic_ptree_dump(const parse_result* result) {
    intercom::cidl::ptree_dump(result);
}
}
