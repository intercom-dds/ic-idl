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

#include <cstring>
#include <filesystem>
#include <iostream>
#include <map>
#include <string_view>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree.h"
#include "cidl/ptree_ffi.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "rust_common.h"
#include "utils/string_utils.h"

// TODO(idarcar): fix before release
#define INTERCOM_VERSION_MAJOR 0
#define INTERCOM_VERSION_MINOR 1
#define INTERCOM_VERSION_PATCH 0

using namespace intercom::rust;
using namespace intercom::cidl;

namespace {
class NumSep : public std::numpunct<char> {
  public:
    explicit NumSep(std::string grouping) : m_group(std::move(grouping)) {}

    [[nodiscard]] char do_thousands_sep() const override {
        return '_';
    }

    [[nodiscard]] char do_decimal_point() const override {
        return '.';
    }

    [[nodiscard]] std::string do_grouping() const override {
        return m_group;
    }

  private:
    std::string m_group;
};

struct Module {
    PrettyPrinter printer;
    std::map<std::string, Module> entries;

    PrettyPrinter& find(const ptree* node) {
        std::vector<std::string> scope;
        for (auto obj = namespace_of(node); obj; obj = namespace_of(obj->scope)) {
            scope.emplace_back(obj->name);
        }

        Module* curr = this;
        for (auto it = scope.rbegin(); it != scope.rend(); ++it) {
            curr = &curr->entries[*it];
        }
        return curr->printer;
    }
};

class Twine {
  public:
    Twine() {
        m_curr.set_indent_size(4);
    }

    Twine(const Twine&) = delete;

    void module(const ptree* mod) {
        m_curr = m_root.find(mod);
        m_curr.set_indent_size(4U);
    }

    [[nodiscard]] const Module& modules() const {
        return m_root;
    }

    template <typename T, typename... Args>
    void operator()(T&& value, Args&&... args) {
        print(std::forward<T>(value), std::forward<Args>(args)...);
    }

    template <typename T>
    Twine& operator<<(T value) {
        m_curr << value;
        return *this;
    }

    [[nodiscard]] bool empty() const {
        return m_curr.empty();
    }

    [[nodiscard]] std::string str() const {
        return m_curr.str();
    }

  private:
    template <typename T>
    void print(const T& arg) {
        m_curr << arg;
    }

    void print(const std::string& arg) {
        print(arg.c_str());
    }

    template <typename T>
    void putc(char c, const T& value, int inc) {
        if (m_count == 0 || (m_count + inc) == 0) {
            m_curr << value;
        } else {
            m_curr << c;
        }
        m_count += inc;
    }

    void print(const char* arg) {
        size_t len = strlen(arg);
        for (size_t i = 0; i < len; i++) {
            switch (arg[i]) {
            case '{':
                putc(arg[i], begin_curly, 1);
                break;
            case '}':
                putc(arg[i], end_curly, -1);
                break;
            case '[':
                putc(arg[i], begin("["), 1);
                break;
            case ']':
                putc(arg[i], end("]"), -1);
                break;
            case '(':
                putc(arg[i], begin("("), 1);
                break;
            case ')':
                putc(arg[i], end(")"), -1);
                break;
            case '\t':
                m_curr << "    ";
                break;
            case '\n': {
                if (i < strlen(arg) && arg[i + 1] == '\n') {
                    m_curr << blank_line;
                    i++;
                } else {
                    m_curr << endl;
                }
                m_count = 0;
                break;
            }
            default: {
                if (arg[i] == ' ') {
                    if (i + 1 < len && arg[i + 1] == ' ') {
                        i++;
                        continue;
                    }
                }
                m_curr << arg[i];
                break;
            }
            }
        }
    }

    void print(ptree* node) {
        m_curr << rust_name(node);
    }

    void print(const ptree* node) {
        m_curr << rust_name(node);
    }

    template <typename T, typename... Args>
    void print(const T& arg, Args&&... args) {
        print(arg);
        print(std::forward<Args>(args)...);
    }

  private:
    int m_count = 0;
    Module m_root;
    PrettyPrinter m_curr;
};
}  // namespace

template <typename... Args>
static std::string str(Args&&... args) {
    Twine out;
    out(std::forward<Args>(args)...);
    return out.str();
}

template <typename... Args>
static std::string quote(Args&&... value) {
    return str('"', value..., '"');
}

template <typename C>
static std::string join(const C& container, const char* sep) {
    return fmt::format("{}", fmt::join(container, sep));
}

static void recurse_node(Twine&, const ptree*);

static void emit_complex(Twine&, const numeric&, const ptree*, const ptree*);

static void emit_builder(Twine&, const ptree*);

static std::string rust_type(const ptree*, const ptree*);

static std::vector<const ptree*> struct_members(const ptree*);

static void emit_const_value(Twine&, const numeric&, const ptree*, const ptree*);

static bool is_trivial(const ptree*);

static bool is_copy(const ptree*);

bool no_struct_or_enum_filter(const ptree*);

static std::ostream& prefix(std::ostream& out) {
    if (CommandLineOption::intercom_build()) {
        return out << "crate";
    }
    return out << "::intercom_cts";
}

static std::ostream& cts_prefix(std::ostream& out) {
    return out << prefix;
}

std::string intercom::cidl::rust_name(const ptree* node) {
    switch (node->kind) {
    case N_PRIMITIVE:
    case N_SEQUENCE:
    case N_MAP:
    case N_ARRAY:
        return rust_type(node, node);
    case N_STRUCT:
        if (get_annotation(node, annotation_type_ext_builder)) {
            return str(type_name(node), "Inner");
        }
        break;
    default:
        break;
    }
    return node->name;
}

static std::string value(const ptree* node, const ptree* ctx) {
    Twine out;
    emit_complex(out, node->value, node, ctx);
    return out.str();
}

static size_t log2(const ptree* node) {
    size_t res = 0;
    auto val = value<uint64_t>(node->value);
    while (val >>= 1) {
        ++res;
    }
    return res;
}

static bool is_pass_by_value(const ptree* node) {
    auto type = base_type_of(node);
    return !is_optional(node) && !is_shared(node) &&
           (is_primitive(type) || type->kind == N_ENUM || type->kind == N_BITMASK);
}

static void array_type(Twine& out, const ptree* node, size_t pos) {
    if (pos < node->bounds.size()) {
        out('[');
        array_type(out, node, pos + 1);
        out("; ", string_value(node->bounds[pos]), ']');
    } else {
        out(rust_type(base_type_of(node)->element_type, node));
    }
}

static void array_value(Twine& out, const ptree* node) {
    out('[');
    for (auto mem : node->members) {
        out(value(mem, node));
        if (mem->next) {
            out(", ");
        }
    }
    out(']');
}

static void array_default(Twine& out, const ptree* node, size_t pos) {
    if (pos < node->bounds.size()) {
        if (is_copy(node->element_type)) {
            out('[');
            array_default(out, node, pos + 1);
            out("; ", string_value(node->bounds[pos]), ']');
        } else {
            out("std::array::from_fn(|_| ");
            array_default(out, node, pos + 1);
            out(")");
        }
    } else {
        emit_const_value(out, node->value, node->element_type, node);
    }
}

static void map_value(Twine& out, const ptree* node) {
    out("::std::collections::BTreeMap::from([\n");
    for (auto v : node->members) {
        out("(");
        for (auto mem : v->value.val.node()->members) {
            out(list_sep, value(mem, node));
        }
        out("),\n");
    }
    out("])");
}

static void struct_value(Twine& out, const ptree* node, const ptree* obj, const ptree* ctx) {
    auto members = struct_members(node->type);
    auto mit = members.begin();
    auto vit = obj->members;

    out(rust_type(base_type_of(obj), ctx), " {\n");
    for (; mit != members.end(); ++mit, vit = vit->next) {
        out(*mit, ": ");
        emit_complex(out, vit->value, *mit, ctx);
        out(",\n");
    }
    out("}");
}

static std::string scoped_name(const ptree* node, const ptree*) {
    Twine out;
    auto common = common_scope(node, nullptr);
    std::function<void(const ptree*)> rec = [&](const ptree* obj) {
        if (obj) {
            auto scope = obj->super;
            if (!scope || scope != common) {
                rec(scope);
                out("::");
            }
            out(obj);
        }
    };

    auto full_name = idl_scoped_name(node, nullptr);
    std::string_view view(full_name);
    if (!CommandLineOption::intercom_build() &&
        (string_utils::starts_with(view, "types") || string_utils::starts_with(view, "core"))) {
        out("intercom");
    } else {
        out("crate");
    }
    rec(node);
    return out.str();
}

static std::string param_type(const ptree* node, const ptree* ctx) {
    Twine out;
    if (!is_pass_by_value(node) || (ctx->flags & OPT_OUT) != 0) {
        out("&");

        if (ctx->flags & OPT_OUT) {
            out("mut ");
        }
        if ((ctx->flags & OPT_OUT) == 0) {
            auto type = base_type_of(node);
            if (type->kind == N_STRING) {
                out("str");
                return out.str();
            }
            if (type->kind == N_SEQUENCE) {
                out("[", rust_type(type->element_type, ctx), "]");
                return out.str();
            }
        }
    }
    out(rust_type(node, ctx));
    return out.str();
}

static std::string rust_type(const ptree* node, const ptree* ctx) {
    if (node == &any_type || node == &object_type) {
        return "()";
    }
    if (node == &boolean_type) {
        return "bool";
    }
    if (node == &int8_type) {
        return "i8";
    }
    if (node == &octet_type) {
        return "u8";
    }
    if (node == &char_type || node == &wchar_type) {
        return "char";
    }
    if (node == &short_type) {
        return "i16";
    }
    if (node == &ushort_type) {
        return "u16";
    }
    if (node == &long_type) {
        return "i32";
    }
    if (node == &ulong_type) {
        return "u32";
    }
    if (node == &longlong_type) {
        return "i64";
    }
    if (node == &ulonglong_type) {
        return "u64";
    }
    if (node == &float_type) {
        return "f32";
    }
    if (node == &double_type || node == &ldouble_type) {
        return "f64";
    }
    if (node->kind == N_STRING) {
        return "String";
    }

    Twine out;
    if (node->kind == N_ARRAY) {
        array_type(out, node, 0);
    } else if (node->kind == N_SEQUENCE) {
        out("Vec<");
        if (base_type_of(node->element_type)->kind == N_INTERFACE) {
            out("Box<");
        }
        out(rust_type(node->element_type, ctx), ">");
        if (base_type_of(node->element_type)->kind == N_INTERFACE) {
            out(">");
        }
    } else if (node->kind == N_MAP) {
        out("::std::collections::BTreeMap<", rust_type(node->key_type, ctx), ", ");
        if (base_type_of(node->element_type)->kind == N_INTERFACE) {
            out("Box<");
        }
        out(rust_type(node->element_type, ctx), ">");
        if (base_type_of(node->element_type)->kind == N_INTERFACE) {
            out(">");
        }
    } else if (node->kind == N_INTERFACE) {
        out("dyn ", scoped_name(node, ctx));
    } else {
        return scoped_name(node, ctx);
    }
    return out.str();
}

static std::string apply_bounds(const ptree* node, const std::string& body, bool is_mutable) {
    Twine out;
    auto ref = is_mutable ? "&mut " : "&";

    if (has_min_value(node) && has_max_value(node)) {
        out(ref, cts_prefix, "::range(", body, ", ");
        emit_const_value(out, get_min_value(node), node, node);
        out(", ");
        emit_const_value(out, get_max_value(node), node, node);
        out(")");
    } else if (has_min_value(node)) {
        out(ref, cts_prefix, "::min(", body, ", ");
        emit_const_value(out, get_min_value(node), node, node);
        out(")");
    } else if (has_max_value(node)) {
        out(ref, cts_prefix, "::max(", body, ", ");
        emit_const_value(out, get_max_value(node), node, node);
        out(")");
    } else if (base_type_of(node)->kind != N_ARRAY && !base_type_of(node)->bounds.empty()) {
        auto bound = unsigned_value(base_type_of(node)->bounds[0]);
        out(ref, cts_prefix, "::bound::<_, ", bound, ">(", body, ")");
    } else {
        out(body);
    }
    return out.str();
}

// Helper function for wrapping a member to modify its serialization
static std::string seri_accesor(const ptree* node, std::string body, bool is_mutable) {
    if (is_mutable) {
        body = str("&mut ", body);
    }
    if (is_wstring(node) || base_type_of(node) == &wchar_type) {
        if (is_wstring(node)) {
            body = str(cts_prefix, "::WString(", body, ")");
        } else {
            body = str(cts_prefix, "::WChar(", body, ")");
        }

        if (node->bounds.empty()) {
            std::string new_body = "&";
            if (is_mutable) {
                new_body += "mut ";
            }
            body = new_body + body;
        }
    }
    return apply_bounds(node, body, is_mutable);
}

static std::string builder_name(const ptree* node) {
    if (auto ann = get_annotation(node, annotation_type_ext_builder)) {
        auto value = string_value(get_annotation_value(ann, "name"));
        if (!value.empty()) {
            return value;
        }
    }
    return type_name(node);
}

static std::string member_type(const ptree* node, const ptree* ctx) {
    auto name = rust_type(node->type, ctx);
    if (is_shared(node)) {
        return str("Box<", name, ">");
    }
    if (is_optional(node)) {
        return str("Option<", name, ">");
    }
    return name;
}

static std::vector<const ptree*> struct_members(const ptree* node) {
    std::vector<const ptree*> members;
    std::function<void(const ptree*)> rec = [&](const ptree* obj) {
        for (auto parent : obj->parents) {
            rec(base_type_of(parent));
        }
        for (auto mem : obj->members) {
            members.emplace_back(mem);
        }
    };
    rec(base_type_of(node));
    return members;
}

static std::vector<std::pair<uint32_t, const ptree*>> ordered_members(const ptree* node) {
    std::vector<std::pair<uint32_t, const ptree*>> members;

    int last_id = node->kind == N_UNION ? 0 : -1;
    std::function<void(const ptree*)> rec = [&](const ptree* obj) {
        for (auto parent : obj->parents) {
            rec(base_type_of(parent));
        }
        for (auto mem : obj->members) {
            if (!is_non_serialized(mem)) {
                last_id = get_member_id(mem, obj, last_id);
                members.emplace_back(last_id, mem);
            }
        }
    };

    rec(base_type_of(node));
    return members;
}

static size_t member_count(const ptree* node) {
    size_t i = 0;
    for (auto parent : node->parents) {
        i += member_count(parent);
    }
    for (auto mem = node->members; mem; mem = mem->next) {
        i++;
    }
    return i;
}

static std::string union_member_name(const ptree* node, const ptree* cas) {
    Twine out;
    if (member_count(node) <= 1 || cas->value.val._d() != PTREE_KIND) {
        out(node);
    }
    if (member_count(node) > 1) {
        if (cas->value.val._d() == PTREE_KIND) {
            out(type_name(cas->value.val.node()));
        } else {
            out(cas);
        }
    }
    return out.str();
}

template <typename T>
static void emit_literal(Twine& out, const T& value, int base, std::streamsize precision = 0) {
    std::stringstream stream;
    stream.precision(precision);

    if (base == 16) {
        stream.imbue(std::locale(stream.getloc(), new NumSep("\04")));
        stream << "0x" << std::uppercase << std::hex;
    } else {
        if (base == 8) {
            stream << "0o" << std::oct;
        }
        stream.imbue(std::locale(stream.getloc(), new NumSep("\03")));
    }
    stream << value;
    out(stream.str());
}

static std::string member_id(const ptree* mem) {
    int last_id = -1;
    for (auto elem : mem->super->members) {
        last_id = get_member_id(elem, mem->super, last_id);
        if (elem == mem) {
            break;
        }
    }

    Twine out;
    emit_literal(out, last_id, 10);
    return out.str();
}

static void emit_const_value(Twine& out, const numeric& val, const ptree* node, const ptree* ctx) {
    switch (val.kind()) {
    case UNDEF_KIND: {
        auto type = node->kind == N_MEMBER ? node->type : node;
        if (type->kind == N_ARRAY) {
            array_default(out, type, 0);
        } else {
            out("<", rust_type(type, ctx), ">", "::default()");
        }
        break;
    }
    case BOOLEAN_KIND:
        out((val.val.b() ? "true" : "false"));
        break;
    case INT8_KIND:
        emit_literal(out, static_cast<int>(val.val.i8()), val.base);
        break;
    case OCTET_KIND:
        emit_literal(out, static_cast<int>(val.val.o()), val.base);
        break;
    case SHORT_KIND:
        emit_literal(out, val.val.s(), val.base);
        break;
    case USHORT_KIND:
        emit_literal(out, val.val.us(), val.base);
        break;
    case LONG_KIND:
        emit_literal(out, val.val.l(), val.base);
        break;
    case ULONG_KIND:
        emit_literal(out, val.val.ul(), val.base);
        break;
    case LONGLONG_KIND:
        emit_literal(out, val.val.ll(), val.base);
        break;
    case ULONGLONG_KIND:
        emit_literal(out, val.val.ull(), val.base);
        break;
    case FLOAT_KIND:
        emit_literal(out, val.val.f(), val.base, 7);
        out("_", rust_type(base_type_of(node), nullptr));
        break;
    case DOUBLE_KIND:
        emit_literal(out, val.val.d(), val.base, 16);
        out("_", rust_type(base_type_of(node), nullptr));
        break;
    case STRING_KIND:
        out('"');
        emit_literal(out, val.val.str(), val.base);
        out('"');
        if (base_type_of(ctx)->kind != N_STRING) {
            out(".into()");
        }
        break;
    case CHAR_KIND: {
        auto c = val.val.c();
        if (c >= 32 && c <= 126) {
            out(fmt::format("'{}'", static_cast<char>(c)));
        } else {
            out(fmt::format("'\\x{:02X}'", c));
        }
        break;
    }
    case PTREE_KIND: {
        auto obj = val.val.node();
        if (obj->kind == N_CONST) {
            if (obj->flags & OPT_CONST_VALUE) {
                if (base_type_of(node)->kind == N_ARRAY) {
                    array_value(out, obj);
                } else if (base_type_of(node)->kind == N_SEQUENCE) {
                    out("vec!");
                    array_value(out, obj);
                } else if (base_type_of(node)->kind == N_MAP) {
                    map_value(out, obj);
                } else {
                    struct_value(out, node, obj, ctx);
                }
            } else {
                out(rust_type(obj, ctx));
                if (base_type_of(node)->kind == N_STRING && base_type_of(ctx)->kind != N_STRING) {
                    out(".into()");
                } else if (!is_trivial(obj)) {
                    out(".clone()");
                }
            }
        } else {
            out(rust_type(node->type, ctx), "::new()");
        }
    } break;
    }
}

static void emit_complex(Twine& out, const numeric& val, const ptree* node, const ptree* ctx) {
    bool is_some = is_optional(node) != 0;
    bool wrap_self = base_type_of(node)->kind == N_BITMASK && val.kind() != PTREE_KIND;

    if (is_some && !has_default_value(node)) {
        out("None");
        return;
    }

    if (is_some) {
        out("Some(");
    }
    if (is_shared(node)) {
        out("Box::new(");
    }
    if (wrap_self) {
        out(rust_type(base_type_of(node), ctx), "(");
    }
    emit_const_value(out, val, node, ctx);

    if (wrap_self) {
        out(")");
    }
    if (is_some) {
        out(")");
    }
    if (is_shared(node)) {
        out(")");
    }
}

static void emit_default_value(Twine& out, const ptree* node, const ptree* ctx) {
    emit_complex(out, get_default_value(node), node, ctx);
}

static bool is_trivial(const ptree* node) {
    return (base_type_of(node)->kind == N_STRING && node->kind == N_CONST) ||
           (node->flags & OPT_RUST_TRIVIAL);
}

static bool is_copy(const ptree* node) {
    return is_trivial(node);
}

static bool is_debug(const ptree* node) {
    // Bitmasks do not derive debug. We provide a nicer implementation of
    // the `Debug` trait which emits the name of the constants instead of
    // just the underlying value.
    return node->kind != N_BITMASK;
}

static bool is_ord(const ptree* node) {
    return (node->flags & OPT_RUST_TOTAL_ORDER) != 0;
}

static bool is_eq(const ptree* node) {
    return is_ord(node);
}

static bool is_hash(const ptree* node) {
    return is_ord(node);
}

static void emit_prelude(std::ostream& out) {
    // Putting this in one of the first five lines of the file will allow
    // rustfmt to recognize that the file is generated, and will not try to
    // format the file even if it belongs to a crate.
    out << "// @generated\n";

    if (CommandLineOption::copyright_notice()) {
        out << copyright_header() << "\n";
    }
    out << "\n";

    // Omitted for now
    // if (!CommandLineOption::no_typesupport()) {
    //     out << "const _: () = " << prefix << "::version_check(";
    //     out << INTERCOM_VERSION_MAJOR << ", " << INTERCOM_VERSION_MINOR << ", "
    //         << INTERCOM_VERSION_PATCH;
    //     out << ");\n\n";
    // }
}

static void emit_docs(Twine& out, const ptree* node) {
    for (auto ann : node->annotations) {
        if (ann->type != annotation_type_doc) {
            continue;
        }
        std::string_view input = ann->members->value.val.str();

        size_t pos = 0;
        while ((pos = input.find('\n')) != std::string_view::npos) {
            auto line = input.substr(0, pos);
            input.remove_prefix(pos + 1);

            if (node->kind != N_MODULE) {
                out << "/// " << line << endl;
            }
        }
    }
}

static void emit_derives(Twine& out, const ptree* node) {
    std::vector<std::string> derives;
    if (is_copy(node)) {
        derives.emplace_back("Copy");
    }
    derives.emplace_back("Clone");

    if (is_debug(node)) {
        derives.emplace_back("Debug");
    }
    if (is_eq(node)) {
        derives.emplace_back("Eq");
    }
    derives.emplace_back("PartialEq");

    if (is_ord(node)) {
        derives.emplace_back("Ord");
    }
    derives.emplace_back("PartialOrd");

    if (is_hash(node)) {
        derives.emplace_back("Hash");
    }

    if (!derives.empty()) {
        out("#[derive(", join(derives, ", "), ")]\n");
    }

    if (auto ann = get_annotation(node, annotation_type_derive)) {
        auto name = get_annotation_value(ann, "value");
        if (name.kind() == STRING_KIND) {
            out("#[derive(", name.val.str(), ")]\n");
        }
    }
}

static void emit_module_def(Twine& out, const ptree* node) {
    for (auto mem : node->members) {
        recurse_node(out, mem);
    }
}

static void emit_struct_def(Twine& out, const ptree* node) {
    emit_derives(out, node);
    out("pub struct ", node, " {");

    auto members = struct_members(node);
    for (auto mem : members) {
        out("\n");
        emit_docs(out, mem);
        out("pub ", mem, ": ");
        out(member_type(mem, node), ",\n");
    }
    out("}\n\n");

    if (node->kind == N_EXCEPTION) {
        out("pub type ", node);
        out("Result<T> = ::std::result::Result<T, ", node, ">;\n\n");
    }
}

static void emit_struct_impl(Twine& out, const ptree* node) {
    out("impl ", node, " {\n");

    // constructor
    out("#[must_use]\n");
    out("pub fn new() -> Self {\n");
    out("Self {");
    for (auto mem : struct_members(node)) {
        out("\n", mem, ": ");
        emit_default_value(out, mem, node);
        out(",\n");
    }
    out("}\n");
    out("}\n");
    out("}\n\n");

    if (node->kind == N_EXCEPTION) {
        out("impl ::std::fmt::Display for ", node, " {\n");
        out("fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {\n");
        out("write!(f, \"", node, "\")\n");
        out("}\n");
        out("}\n\n");

        out("impl ::std::error::Error for ", node, " {}\n\n");
    }

    if (get_annotation(node, annotation_type_ext_builder)) {
        emit_builder(out, node);
    }
}

static void emit_union_def(Twine& out, const ptree* node) {
    emit_derives(out, node);
    out("pub enum ", node, " {\n");
    for (auto mem : node->members) {
        emit_docs(out, mem);

        for (auto cas : mem->members) {
            out(union_member_name(mem, cas));
            if (mem->kind == N_MEMBER) {
                out("(", member_type(mem, node), ")");
            }
            out(",\n");
        }
    }
    out("}\n\n");
}

static void emit_union_impl(Twine& out, const ptree* node) {
    out("impl ", node, " {\n");
    out("#[must_use]\n");
    out("pub fn new() -> Self {\n");

    auto def_mem = default_union_member(node);
    out("Self::", union_member_name(def_mem, def_mem->members));
    if (def_mem->kind == N_MEMBER) {
        out("(");
        emit_default_value(out, def_mem, node);
        out(")");
    }
    out("\n}\n\n");

    // Function for deducing the discriminant from the populated enum variant
    auto disc = rust_type(node->discriminator->type, node);
    out("#[must_use]\n");
    out("pub const fn disc(&self) -> ", disc, " {\n");
    out("match self {\n");

    for (auto mem : node->members) {
        for (auto cas : mem->members) {
            out("Self::", union_member_name(mem, cas));
            if (mem->kind == N_MEMBER) {
                out("(_)");
            }
            out(" => ", value(cas, node), ",\n");
        }
    }
    out("}\n");
    out("}\n");
    out("}\n\n");

    // Constructor that initializes the variant that corresponds to the given discriminant.
    // Useful for retrieving the default value specified in IDL.
    out("impl From<", disc, "> for ", node, " {\n");
    out("fn from(disc: ", disc, ") -> Self {\n");
    out("match disc {\n");
    for (auto mem : node->members) {
        if ((mem->flags & OPT_DEFAULT) == 0) {
            for (auto cas : mem->members) {
                out(value(cas, node), " => Self::", union_member_name(mem, cas));
                if (mem->kind == N_MEMBER) {
                    out("(");
                    emit_default_value(out, mem, node);
                    out(")");
                }
                out(",\n");
            }
        }
    }
    if (get_default_case(node)) {
        out("_ => Self::default(),\n");
    }
    out("}\n");
    out("}\n");
    out("}\n\n");
}

static void emit_enum_def(Twine& out, const ptree* node) {
    auto type = rust_type(base_type_of(node)->element_type, nullptr);

    emit_derives(out, node);
    out("#[repr(", type, ")]\n");
    out("pub enum ", node, " {\n");

    for (auto elem : node->members) {
        emit_docs(out, elem);
        out(elem);

        if (elem->flags & OPT_ENUMERATED) {
            out(" = ", integer_value(elem->value));
        }
        out(",\n");
    }
    out("}\n\n");
}

static void emit_enum_impl(Twine& out, const ptree* node) {
    out("impl ", node, " {\n");
    out("#[must_use]\n");
    out("pub const fn new() -> Self {\n");
    emit_default_value(out, node, node);
    out("\n");
    out("}\n");
    out("}\n\n");

    out("impl ::std::str::FromStr for ", node, " {\n");
    out("type Err = ", cts_prefix, "::error::UnknownVariant;\n\n");
    out("fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {\n");
    out("match s {\n");
    for (auto mem : node->members) {
        out('"', seri_name(mem), "\" => Ok(Self::", mem, "),\n");
    }
    out("_ => Err(", cts_prefix, "::error::UnknownVariant),\n");
    out("}\n");
    out("}\n");
    out("}\n\n");

    out("impl ::std::fmt::Display for ", node, " {\n");
    out("fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {\n");
    out("match self {\n");
    for (auto mem : node->members) {
        out("Self::", mem, " => f.write_str(", quote(seri_name(mem)), "),\n");
    }
    out("}\n");
    out("}\n");
    out("}\n\n");
}

static void emit_bitmask_def(Twine& out, const ptree* node) {
    out(cts_prefix, "::bitmask! {\n");
    // Bitmask comments must be inside the macro invocation
    emit_docs(out, node);
    emit_derives(out, node);

    out("pub ", node, ": ", node->element_type, " {\n");
    for (auto bit : node->members) {
        emit_docs(out, bit);
        out(bit, " = 1 << ", log2(bit), ",\n");
    }

    out("}\n");
    out("}\n\n");
}

static void emit_bitmask_impl(Twine& out, const ptree* node) {
    out("impl ", node, " {\n");
    out("#[must_use]\n");
    out("pub fn new() -> Self {\n");
    emit_default_value(out, node, node);
    out("\n");
    out("}\n");
    out("}\n\n");
}

static void emit_default_impl(Twine& out, const ptree* node) {
    out("impl ::std::default::Default for ", node, " {\n");
    out("fn default() -> Self {\n");
    out("Self::new()\n");
    out("}\n");
    out("}\n\n");
}

static void emit_alias_def(Twine& out, const ptree* node) {
    auto type = rust_type(node->type, node);
    if (node->type->kind == N_INTERFACE) {
        out("pub use ", type, " as ", node, ";\n");
    } else {
        out("pub type ", node, " = ", type, ";\n");
    }
}

static void prototype_return_type(Twine& out, const ptree* node, const ptree* ctx) {
    if (!node) {
        out("()");
        return;
    }

    auto exceptions = exception_count(node);
    if (exceptions > 1) {
        out("\n");
        out("\t::std::result::Result<");
        prototype_return_type(out, node->type, ctx);
        out(", Box<dyn ::std::error::Error>>");
    } else if (exceptions == 1) {
        out(node->getraises.empty() ? node->setraises[0] : node->getraises[0]);
        out("Result<");
        prototype_return_type(out, node->type, ctx);
        out(">");
    } else if (node->kind == N_PROTOTYPE) {
        prototype_return_type(out, node->type, ctx);
    } else if (node->kind == N_INTERFACE) {
        out("Box<", rust_type(node, ctx), ">");
    } else {
        out(rust_type(node, ctx));
    }
}

static void emit_prototype_def(Twine& out, const ptree* node) {
    out("fn ", node, "(");
    bool is_static = get_annotation(node, annotation_type_static) != nullptr;
    bool multi = (member_count(node) + !is_static) > 2;

    if (!is_static) {
        if (multi) {
            out("\n");
        }
        out("&");
        if (!get_annotation(node, annotation_type_const)) {
            out("mut ");
        }
        out("self");

        if (node->members) {
            out(", ");
        }
    }

    for (auto mem : node->members) {
        if (multi) {
            out("\n");
        }
        emit_docs(out, mem);
        out(mem, ": ", param_type(mem->type, mem));
        if (mem->next || multi) {
            out(",");
        }
    }
    if (multi) {
        out("\n");
    }
    out(")");

    auto exceptions = exception_count(node);
    if (node->type || exceptions > 0) {
        out(" -> ");
        prototype_return_type(out, node, node);
    }
    if (is_static) {
        out("\n", "where\n");
        out("\tSelf: Sized");
    }
    out(";\n");
}

static void emit_interface_def(Twine& out, const ptree* node) {
    out("pub trait ", node);

    if (!node->parents.empty()) {
        out(": ");
        for (auto parent : node->parents) {
            out(parent);
            if (parent != node->parents.back()) {
                out(" + ");
            }
        }
    }
    out(" {");
    for (auto mem : node->members) {
        out("\n");
        recurse_node(out, mem);
    }
    out("}\n\n");
}

static void emit_const_def(Twine& out, const ptree* node) {
    bool trivial = is_trivial(node);
    out("pub ", (trivial ? "const " : "static "));
    out(node, ": ");

    if (trivial) {
        if (base_type_of(node)->kind == N_STRING) {
            out("&str");
        } else {
            out(rust_type(node->type, node));
        }
        out(" = ", value(node, node));
    } else {
        out("::std::sync::LazyLock<", rust_type(node->type, node), "> =");
        out(begin(""), "\n");
        out("::std::sync::LazyLock::new(|| ", value(node, node), ")");
        out(end(""));
    }
    out(";\n");
}

static void emit_builder(Twine& out, const ptree* node) {
    emit_derives(out, node);
    out("#[derive(Default)]\n");
    out("pub struct ", builder_name(node), "(pub ", node, ");\n\n");

    out("impl ", builder_name(node), " {\n");
    out("#[must_use]\n");
    out("pub fn new() -> Self {\n");
    out("Self::default()\n");
    out("}\n\n");

    auto members = struct_members(node);
    for (auto mem : members) {
        emit_docs(out, mem);
        out("#[must_use]\n");
        out("pub fn ", mem, "(mut self, ", mem, ": ", member_type(mem, node), ") -> Self {\n");
        out("self.0.", mem, " = ", mem, ";\n");
        out("self\n");
        out("}\n\n");

        auto ref = is_trivial(mem) ? "" : "&";
        out("#[must_use]\n");
        out("pub fn get_", mem, "(&self) -> ", ref, member_type(mem, node), " {\n");
        out(ref, "self.0.", mem, "\n");
        out("}\n\n");
    }
    out("}\n\n");
}

static std::string type_flags(const ptree* node) {
    Twine out;

    auto add_flag = [&](auto flag) {
        if (!out.empty()) {
            out(".union(");
            out(cts_prefix, "::TypeFlag::", flag);
            out(")");
        } else {
            out(cts_prefix, "::TypeFlag::", flag);
        }
    };

    int kind = get_extensibility(node);
    switch (kind) {
    case FINAL_EXTENSIBILITY:
        add_flag("IS_FINAL");
        break;
    case MUTABLE_EXTENSIBILITY:
        add_flag("IS_MUTABLE");
        break;
    case EXTENSIBLE_EXTENSIBILITY:
    default:
        add_flag("IS_APPENDABLE");
        break;
    }

    if (is_nested(node)) {
        add_flag("IS_NESTED");
    }

    if (is_autoid_hash(node)) {
        add_flag("IS_AUTOID_HASH");
    }

    bool has_key = std::any_of(begin(node->members), end(node->members), [](auto p) {
        return is_key_member(p);
    });
    if (has_key) {
        add_flag("IS_KEYED");
    }

    return out.str();
}

static std::string member_flags(const ptree* node) {
    Twine out;

    auto add_flag = [&](auto flag) {
        if (!out.empty()) {
            out(".union(");
            out(cts_prefix, "::MemberFlag::", flag);
            out(")");
        } else {
            out(cts_prefix, "::MemberFlag::", flag);
        }
    };

    if (is_key_member(node)) {
        add_flag("IS_KEY");
    }

    if (is_optional(node)) {
        add_flag("IS_OPTIONAL");
    }

    if (is_shared(node)) {
        add_flag("IS_EXTERNAL");
    }

    if (is_must_understand(node)) {
        add_flag("IS_MUST_UNDERSTAND");
    }

    if (out.empty()) {
        add_flag("nil()");
    }
    return out.str();
}

const char* type_kind(const ptree* obj) {
    if (!obj) {
        return "None";
    }

    switch (obj->kind) {
    case N_PRIMITIVE:
        if (obj == &boolean_type) {
            return "Bool";
        }
        if (obj == &octet_type) {
            return "U8";
        }
        if (obj == &int8_type) {
            return "I8";
        }
        if (obj == &short_type) {
            return "I16";
        }
        if (obj == &ushort_type) {
            return "U16";
        }
        if (obj == &long_type) {
            return "I32";
        }
        if (obj == &ulong_type) {
            return "U32";
        }
        if (obj == &longlong_type) {
            return "I64";
        }
        if (obj == &ulonglong_type) {
            return "U64";
        }
        if (obj == &float_type) {
            return "F32";
        }
        if (obj == &double_type) {
            return "F64";
        }
        if (obj == &char_type) {
            return "Char8";
        }
        if (obj == &wchar_type) {
            return "Char16";
        }
        return "None";
    case N_ALIAS:
        if (obj->type->kind == N_MAP || obj->type->kind == N_ARRAY ||
            obj->type->kind == N_SEQUENCE || obj->type->kind == N_STRING) {
            return type_kind(obj->type);
        }
        return "Alias";
    case N_STRUCT:
    case N_VALUETYPE:
    case N_EXCEPTION:
        return "Struct";
    case N_UNION:
        return "Union";
    case N_BITMASK:
        return "Bitmask";
    case N_ENUM:
        return "Enum";
    case N_STRING:
        return is_wstring(obj) ? "String16" : "String8";
    case N_ANNOTATION:
        return "Annotation";
    case N_ARRAY:
        return "Array";
    case N_MAP:
        return "Map";
    case N_SEQUENCE:
        return "Sequence";
    default:
        return "None";
    }
}

static void emit_type_info(Twine& out, const ptree* node) {
    auto qualified_name = idl_scoped_name(original_node(node), nullptr);
    out("const TYPE_INFO: ", cts_prefix, "::TypeInfo<'static> = ", cts_prefix, "::TypeInfo {\n");
    out("name: ", quote(qualified_name), ",\n");
    out("flags: ", type_flags(node), ",\n");
    out("kind: ", cts_prefix, "::TypeKind::", type_kind(node), ",\n");
    out("key_kind: ", cts_prefix, "::TypeKind::", type_kind(node->key_type), ",\n");
    out("element_kind: ", cts_prefix, "::TypeKind::", type_kind(node->element_type), ",\n");
    out("};\n\n");
}

static void
emit_member_info(Twine& out, const std::vector<std::pair<uint32_t, const ptree*>>& members) {
    out("const MEMBER_INFO: &[", cts_prefix, "::MemberInfo<'static>] = &[\n");
    for (auto [id, mem] : members) {
        out(cts_prefix, "::MemberInfo {\n");
        out("name: ", quote(seri_name(mem)), ",\n");
        out("member_id: ");
        emit_literal(out, id, 10);
        out(",\n");
        out("flags: ", member_flags(mem), ",\n");
        out("},\n");
    }
    out("];\n\n");
}

static void emit_marshal(Twine& out, const ptree* node) {
    out("impl ", cts_prefix, "::Marshal for ", node, " {\n");
    out("fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
    out("where\n");
    out("\tS: ", cts_prefix, "::encode::Serializer,\n");
    out("{\n");

    auto members = ordered_members(node);
    auto mut = members.empty() ? "" : "mut ";

    if (node->kind == N_STRUCT || node->kind == N_EXCEPTION || node->kind == N_VALUETYPE) {
        out("use ", cts_prefix, "::encode::StructSerializer as _;\n\n");
        out("let ", mut, "state = ar.encode_struct(&TYPE_INFO)?;\n");

        size_t index = 0;
        for (auto [_, mem] : members) {
            auto mem_var = str("&self.", mem);
            auto accessor = seri_accesor(mem, mem_var, false);

            if (is_optional(mem)) {
                out("state.encode_optional(&MEMBER_INFO[", index++, "], ", mem_var, ")?;\n");
            } else {
                out("state.encode_field(&MEMBER_INFO[", index++, "], ", accessor, ")?;\n");
            }
        }
        out("state.end()\n");
    } else if (node->kind == N_UNION) {
        out("use ", cts_prefix, "::encode::UnionSerializer as _;\n\n");
        out("let mut state = ar.encode_union(&TYPE_INFO)?;\n");
        out("state.encode_discriminant(&self.disc())?;\n");

        bool emit_wildcard = false;

        out("match self {\n");
        size_t index = 0;
        for (auto mem : node->members) {
            if (mem->kind == N_MEMBER) {
                for (auto cas : mem->members) {
                    out("Self::", union_member_name(mem, cas), "(v)");

                    if (cas->next) {
                        out("\n\t| ");
                    } else {
                        auto accessor = seri_accesor(mem, "v", false);
                        out(" => state.encode_variant(&MEMBER_INFO[", index, "], ", accessor, "),\n"
                        );
                    }
                }
            } else {
                emit_wildcard = true;
            }
            index++;
        }
        if (emit_wildcard) {
            out("_ => state.encode_null(),\n");
        }
        out("}\n");
    } else if (node->kind == N_ENUM) {
        auto type = rust_type(base_type_of(node)->element_type, nullptr);
        out("use ", cts_prefix, "::encode::EnumSerializer as _;\n\n");
        out("let state = ar.encode_enum(TYPE_INFO.name)?;\n");
        out("match self {\n");
        for (auto mem : node->members) {
            out("Self::", mem, " => state.encode_variant::<", type, ">(");
            out(quote(seri_name(mem)), ", ", value(mem, node), "),\n");
        }
        out("}\n");
    }
    out("}\n");
    out("}\n\n");

    if (get_annotation(node, annotation_type_ext_builder)) {
        out("impl ", cts_prefix, "::Marshal for ", type_name(node), " {\n");
        out("fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        out("where\n");
        out("\tS: ", cts_prefix, "::encode::Serializer,\n");
        out("{\n");
        out("self.0.marshal(ar)\n");
        out("}\n");
        out("}\n\n");
    }
}

static void emit_unmarshal(Twine& out, const ptree* node) {
    out("impl ", cts_prefix, "::Unmarshal for ", node, " {\n");
    out("fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
    out("where\n");
    out("\tD: ", cts_prefix, "::decode::Deserializer,\n");
    out("{\n");

    auto members = ordered_members(node);
    if (node->kind == N_STRUCT || node->kind == N_EXCEPTION || node->kind == N_VALUETYPE) {
        out("use ", cts_prefix, "::decode::StructDeserializer as _;\n\n");
        out("let ");
        if (!members.empty()) {
            out("mut ");
        }
        out("state = ar.decode_struct(&TYPE_INFO)?;\n");
        size_t index = 0;
        for (auto [_, mem] : members) {
            auto accessor = seri_accesor(mem, str("self.", mem), true);
            out("state.decode_field(&MEMBER_INFO[", index++, "], ", accessor, ")?;\n");
        }
        out("state.end()?;\n");
        out("Ok(())\n");
    } else if (node->kind == N_ENUM) {
        out("use ", cts_prefix, "::decode::EnumDeserializer as _;\n\n");
        out("let state = ar.decode_enum(TYPE_INFO.name)?;\n");
        out("*self = state.decode_enumerator(*self)?;\n");
        out("Ok(())\n");
    } else if (node->kind == N_UNION) {
        auto disc = rust_type(node->discriminator->type, node);
        out("use ", cts_prefix, "::decode::UnionDeserializer as _;\n\n");
        out("let mut state = ar.decode_union(&TYPE_INFO)?;\n");
        out("let mut disc = ", disc, "::default();\n");
        out("state.decode_discriminant(&mut disc)?;\n");
        out("*self = match disc {\n");

        size_t index = 0;
        size_t def_index = 0;
        auto def_mem = get_default_case(node);
        for (auto [id, mem] : members) {
            if (mem == def_mem) {
                def_index = index;
            } else {
                for (auto cas : mem->members) {
                    out(value(cas, node), " => ");
                    if (mem->kind == N_MEMBER) {
                        out("{\n");
                        out("let mut value = ");
                        emit_default_value(out, mem, node);
                        out(";\n");

                        auto accessor = seri_accesor(mem, "value", true);
                        out("state.decode_variant(&MEMBER_INFO[", index, "], ", accessor, ")?;\n");
                        out("Self::", union_member_name(mem, cas), "(value)\n");
                        out("}");
                    } else {
                        out("Self::", union_member_name(mem, cas));
                    }
                    out(",\n");
                }
            }
            ++index;
        }
        if (def_mem) {
            out("_ => ");
            if (def_mem->kind == N_MEMBER) {
                out("{\n");
                out("let mut value = ");
                emit_default_value(out, def_mem, node);
                out(";\n");
                auto accessor = seri_accesor(def_mem, "value", true);
                out("state.decode_variant(&MEMBER_INFO[", def_index, "], ", accessor, ")?;\n");
                out("Self::", union_member_name(def_mem, def_mem->members), "(value)\n");
                out("}");
            } else {
                out("Self::", union_member_name(def_mem, def_mem->members));
            }
            out(",\n");
        }

        out("};\n");
        out("Ok(())\n");
    }
    out("}\n");
    out("}\n");

    if (get_annotation(node, annotation_type_ext_builder)) {
        out("impl ", cts_prefix, "::Unmarshal for ", type_name(node), " {\n");
        out("fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        out("where\n");
        out("\tD: ", cts_prefix, "::decode::Deserializer,\n");
        out("{\n");
        out("self.0 = ", node, "::unmarshal(ar)?;\n");
        out("Ok(())\n");
        out("}\n");
        out("}\n");
    }
}

static void emit_visitor(Twine& out, const ptree* node) {
    out("impl ", cts_prefix, "::decode::EnumVisitor for ", node, " {\n");

    // id => enum variant
    auto type = rust_type(base_type_of(node)->element_type, nullptr);
    out("fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>\n");
    out("where\n");
    out("\tD: ", cts_prefix, "::decode::Deserializer,\n");
    out("{\n");
    out("use ", cts_prefix, "::error::Error as _;\n\n");
    out("let value = match de.decode_", type, "()? {\n");
    for (auto mem : node->members) {
        out(value(mem, node), " => Self::", mem, ",\n");
    }

    auto qualified_err =
        quote("invalid enum value for type ", idl_scoped_name(node->original, nullptr));
    out("_ => return Err(D::Error::custom(", qualified_err, ")),\n");
    out("};\n");
    out("Ok(value)\n");
    out("}\n\n");

    // string => enum variant
    out("fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>\n");
    out("where\n");
    out("\tD: ", cts_prefix, "::decode::Deserializer,\n");
    out("{\n");
    out("use ", cts_prefix, "::error::Error as _;\n\n");
    out("let value = match name {\n");
    for (auto mem : node->members) {
        out(quote(seri_name(mem)), " => Self::", mem, ",\n");
    }
    out("_ => return Err(D::Error::custom(", qualified_err, ")),\n");
    out("};\n");
    out("Ok(value)\n");
    out("}\n");

    out("}\n");
}

static void emit_marshal_all(Twine& out, const ptree* node) {
    if (is_non_serialized(node)) {
        return;
    }

    auto members = ordered_members(node);
    out("const _: () = {\n");

    emit_type_info(out, node);
    emit_member_info(out, members);
    emit_marshal(out, node);
    emit_unmarshal(out, node);

    if (node->kind == N_ENUM) {
        emit_visitor(out, node);
    }

    out("};\n\n");
}

static void recurse_node(Twine& out, const ptree* node) {
    // Rust does not care about the order of definitions.
    // For that reason, forward declarations do not exist in the language.
    if (is_decl(node) || !is_emit(node, LANG_RUST)) {
        return;
    }

    out.module(node);

    switch (node->kind) {
    case N_MODULE:
        emit_docs(out, node);
        emit_module_def(out, node);
        break;

    case N_EXCEPTION:
    case N_VALUETYPE:
    case N_STRUCT:
        emit_docs(out, node);
        emit_struct_def(out, node);
        emit_struct_impl(out, node);
        emit_default_impl(out, node);
        emit_marshal_all(out, node);
        break;

    case N_UNION:
        emit_docs(out, node);
        emit_union_def(out, node);
        emit_union_impl(out, node);
        emit_default_impl(out, node);
        emit_marshal_all(out, node);
        break;

    case N_ENUM:
        emit_docs(out, node);
        emit_enum_def(out, node);
        emit_enum_impl(out, node);
        emit_default_impl(out, node);
        emit_marshal_all(out, node);
        break;

    case N_BITMASK:
        emit_bitmask_def(out, node);
        emit_bitmask_impl(out, node);
        emit_default_impl(out, node);
        break;

    case N_ALIAS:
        emit_docs(out, node);
        emit_alias_def(out, node);
        break;

    case N_PROTOTYPE:
        emit_docs(out, node);
        emit_prototype_def(out, node);
        break;

    case N_INTERFACE:
        emit_docs(out, node);
        emit_interface_def(out, node);
        break;

    case N_CONST:
        emit_docs(out, node);
        emit_const_def(out, node);
        break;

    default:
        break;
    }
    out(blank_line);
}

static void save_file(std::stringstream& stream, std::filesystem::path file, ic_list_t* list) {
    if (file.empty()) {
        file = CommandLineOption::intercom_build() ? "mod" : "lib";
    }
    file.replace_extension(".rs");
    ic_push_source(list, file.string().c_str(), stream.str().c_str());
}

static void
emit_module(const Module& module, ic_list_t* list, const std::filesystem::path& name = "") {
    std::stringstream stream;
    emit_prelude(stream);

    for (const auto& mod : module.entries) {
        stream << "pub mod " << mod.first << ";\n";
        emit_module(mod.second, list, name / mod.first);
    }
    if (!module.entries.empty()) {
        stream << '\n';
    }
    stream << module.printer.str();
    save_file(stream, name, list);
}

template <typename P>
static void emit_crate(ic_list_t* list, const ptree* node, P predicate) {
    Twine out;
    for (auto obj : node) {
        if (predicate(obj)) {
            recurse_node(out, obj);
        }
    }
    emit_module(out.modules(), list);
}

void intercom::cidl::code_gen_rust(const parse_result* result, ic_list_t* list) {
    auto cloned = clone_tree(result);
    transform_rust(&cloned);
    emit_crate(list, cloned.tree, [&](const ptree* node) { return is_emit(node, LANG_RUST); });
}
