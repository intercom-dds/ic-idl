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
#include <unordered_map>

#include "InterCOM/detail/filesystem.h"
#include "cidl/constants.h"
#include "cidl/idl_parser.h"
#include "cidl/internal/commandline.h"
#include "cidl/internal/hdrs.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "utils/XMLTypes.h"
#include "utils/XMLWriter.h"

namespace {
class Context {
  public:
    class Scope;

    intercom::XMLElement* operator->() const { return m_current; }
    intercom::XMLElement& operator*() const { return *m_current; }
    std::unordered_map<std::string, intercom::XMLElement>& files() { return m_files; }

  private:
    void context(const ptree* node) {
        auto it = m_files.find(node->file_name);
        if (it == m_files.cend()) {
            m_files[node->file_name] = {};
            m_current = &m_files[node->file_name].createMemberXMLElement();
        } else if (m_current) {
            m_current = &m_current->createMemberXMLElement();
        } else {
            m_current = &it->second.createMemberXMLElement();
        }
    }

  private:
    intercom::XMLElement* m_current = nullptr;
    std::unordered_map<std::string, intercom::XMLElement> m_files;
};

class Context::Scope {
  public:
    Scope(const ptree* node, Context& context) : m_parent(context.m_current), m_context(context) {
        context.context(node);
    }

    ~Scope() { m_context.m_current = m_parent; }

  private:
    intercom::XMLElement* m_parent = nullptr;
    Context& m_context;
};
}  // namespace

static void recurse_node(const ptree*, Context&);

void include_dependencies(const ptree*, const ptree*, std::set<ptree*>&);

template <typename C>
static std::string join(const C& container, const char* sep) {
    return fmt::format("{}", fmt::join(container, sep));
}

static bool is_basic(const ptree* node) {
    return !node || is_primitive(node) || node->kind == N_STRING;
}

static bool is_complex(const ptree* node) {
    auto base = base_type_of(node);
    return !is_basic(base) && base->kind != N_ENUM && base->kind != N_BITMASK;
}

static size_t log2(const ptree* node) {
    size_t res = 0;
    auto val = value<uint64_t>(node->value);
    while (val >>= 1) {
        ++res;
    }
    return res;
}

static const char* extensibility(const ptree* node) {
    switch (get_extensibility(node)) {
    case MUTABLE_EXTENSIBILITY:
        return "mutable";
    case FINAL_EXTENSIBILITY:
        return "final";
    default:
        return "appendable";
    }
}

static void type_attrs(const ptree* node, intercom::XMLElement& out) {
    if (get_extensibility(node) != EXTENSIBLE_EXTENSIBILITY) {
        out.createMemberXMLAttribute().set("extensibility", extensibility(node));
    }
    if (is_nested(node)) {
        out.createMemberXMLAttribute().set("nested", "true");
    }
    if (is_autoid_hash(node)) {
        out.createMemberXMLAttribute().set("autoid", "hash");
    }
    if (is_must_understand(node)) {
        out.createMemberXMLAttribute().set("mustUnderstand", "true");
    }
}

static void member_attrs(const ptree* node, intercom::XMLElement& out) {
    if (is_key_member(node)) {
        out.createMemberXMLAttribute().set("key", "true");
    }
    if (is_optional(node)) {
        out.createMemberXMLAttribute().set("optional", "true");
    }
    if (is_shared(node)) {
        out.createMemberXMLAttribute().set("external", "true");
    }
    if (is_must_understand(node)) {
        out.createMemberXMLAttribute().set("mustUnderstand", "true");
    }
    if (is_non_serialized(node)) {
        out.createMemberXMLAttribute().set("nonSerialized", "true");
    }
}

static std::string xml_type(const ptree* node, const ptree* scope) {
    if (!node) {
        return "null";
    }
    if (node == &boolean_type) {
        return "boolean";
    }
    if (node == &char_type) {
        return "char8";
    }
    if (node == &wchar_type) {
        return "char16";
    }
    if (node == &int8_type) {
        return "int8";
    }
    if (node == &octet_type) {
        return "uint8";
    }
    if (node == &short_type) {
        return "int16";
    }
    if (node == &ushort_type) {
        return "uint16";
    }
    if (node == &long_type) {
        return "int32";
    }
    if (node == &ulong_type) {
        return "uint32";
    }
    if (node == &longlong_type) {
        return "int64";
    }
    if (node == &ulonglong_type) {
        return "uint64";
    }
    if (node == &float_type) {
        return "float32";
    }
    if (node == &double_type) {
        return "float64";
    }
    if (node == &ldouble_type) {
        return "float128";
    }
    if (node->kind == N_STRING) {
        if (is_wstring(node)) {
            return "wstring";
        }
        return "string";
    }
    return idl_scoped_name(node, scope);
}

static std::string bit_bound(const ptree* node) {
    auto type = base_type_of(node)->element_type;
    if (type == &int8_type || type == &octet_type) {
        return "8";
    }
    if (type == &short_type || type == &ushort_type) {
        return "16";
    }
    if (type == &long_type || type == &ulong_type) {
        return "32";
    }
    return "64";
}

static void type_bounds(const ptree* node, intercom::XMLElement& out) {
    auto bound = node->bounds.empty() ? "-1" : string_value(node->bounds[0]);
    if (node->kind == N_SEQUENCE) {
        out.createMemberXMLAttribute().set("sequenceMaxLength", bound);
    } else if (node->kind == N_ARRAY) {
        auto dimensions = join(node->bounds, ", ");
        out.createMemberXMLAttribute().set("arrayDimensions", dimensions);
    } else if (node->kind == N_STRING) {
        if (!node->bounds.empty()) {
            out.createMemberXMLAttribute().set("stringMaxLength", bound);
        }
    } else if (node->kind == N_MAP) {
        if (!node->bounds.empty()) {
            out.createMemberXMLAttribute().set("mapMaxLength", bound);
        }
        if (node->key_type->kind == N_STRING && !node->key_type->bounds.empty()) {
            out.createMemberXMLAttribute().set("mapKeyStringMaxLength", string_value(node->key_type->bounds[0]));
        }
    }
}

static void define_type(const ptree* node, const ptree* scope, intercom::XMLElement& out) {
    if (is_basic(node)) {
        out.createMemberXMLAttribute().set("type", xml_type(node, scope));
    } else if (node->kind == N_ARRAY || node->kind == N_SEQUENCE) {
        define_type(node->element_type, scope, out);
    } else if (node->kind == N_MAP) {
        if (is_basic(node->key_type)) {
            out.createMemberXMLAttribute().set("mapKeyType", xml_type(node->key_type, scope));
        } else {
            out.createMemberXMLAttribute().set("mapKeyType", "nonBasic");
            out.createMemberXMLAttribute().set("mapKeyNonBasicTypeName", xml_type(node->key_type, scope));
        }
        define_type(node->element_type, scope, out);
    } else {
        out.createMemberXMLAttribute().set("type", "nonBasic");
        out.createMemberXMLAttribute().set("nonBasicTypeName", xml_type(node, scope));
    }
    if (node) {
        type_bounds(node, out);
    }
}

static void emit_annotations(const ptree* node, intercom::XMLElement& out) {
    for (const auto& ann : node->annotations) {
        if (ann->type == annotation_type_must_understand || ann->type == annotation_type_doc ||
            ann->type == annotation_type_extensibility || ann->type == annotation_type_mutable ||
            ann->type == annotation_type_final || ann->type == annotation_type_appendable ||
            ann->type == annotation_type_key || ann->type == annotation_type_nested ||
            ann->type == annotation_type_bit_bound || ann->type == annotation_type_autoid ||
            (ann->type == annotation_type_default && is_complex(node))) {
            continue;
        }

        auto& elem = out.createMemberXMLElement();
        elem.setName("annotate");
        elem.createMemberXMLAttribute().set("name", ann->name);

        for (auto mem : ann->members) {
            auto& value = elem.createMemberXMLElement();
            value.setName("member");
            value.createMemberXMLAttribute().set("name", mem->name);
            value.createMemberXMLAttribute().set("value", string_value(mem->value));
        }
    }
}

static void emit_module(const ptree* node, Context& out) {
    out->setName("module");
    out->createMemberXMLAttribute().set("name", node->name);
    emit_annotations(node, *out);

    for (auto mem : node->members) {
        recurse_node(mem, out);
    }
}

static void emit_annotation_def(const ptree* node, Context& out) {
    out->setName("annotation");
    out->createMemberXMLAttribute().set("name", node->name);
    type_attrs(node, *out);

    for (auto mem : node->members) {
        if (mem->kind == N_MEMBER) {
            auto& elem = out->createMemberXMLElement();
            elem.setName("member");
            elem.createMemberXMLAttribute().set("name", mem->name);
            define_type(mem->type, node, elem);
        } else {
            recurse_node(mem, out);
        }
    }
}

static void emit_member(const ptree* mem, const ptree* context, intercom::XMLElement& out, int& last_id) {
    for (auto cas : mem->members) {
        auto& case_disc = out.createMemberXMLElement();
        case_disc.setName("caseDiscriminator");

        auto& value = case_disc.createMemberXMLAttribute();
        if (cas->flags & OPT_DEFAULT) {
            value.set("value", "default");
        } else if (cas->value.kind() == PTREE_KIND) {
            value.set("value", idl_scoped_name(cas->value.val.node(), context));
        } else {
            value.set("value", string_value(cas->value));
        }
    }

    auto& elem = out.createMemberXMLElement();
    elem.setName("member");
    elem.createMemberXMLAttribute().set("name", mem->name);

    if (!is_autoid_hash(context)) {
        last_id = get_member_id(mem, context, last_id);
        if (get_annotation(mem, annotation_type_id)) {
            elem.createMemberXMLAttribute().set("id", std::to_string(last_id));
        }
    }
    define_type(mem->type, mem, elem);
    member_attrs(mem, elem);
    emit_annotations(mem, elem);
}

static void emit_forward_dcl(const ptree* node, Context& out) {
    out->setName("forward_dcl");
    auto kind = node->kind == N_STRUCT ? "struct" : "union";
    out->createMemberXMLAttribute().set("kind", kind);
    out->createMemberXMLAttribute().set("name", node->name);
    emit_annotations(node, *out);
}

static void emit_struct(const ptree* node, Context& out) {
    out->setName("struct");
    out->createMemberXMLAttribute().set("name", node->name);
    if (!node->parents.empty()) {
        out->createMemberXMLAttribute().set("baseType", xml_type(node->parents[0], node));
    }
    type_attrs(node, *out);
    emit_annotations(node, *out);

    int last_id = -1;
    for (auto mem : node->members) {
        if (mem->kind == N_MEMBER) {
            emit_member(mem, node, *out, last_id);
        } else {
            recurse_node(mem, out);
        }
    }
}

static void emit_union(const ptree* node, Context& out) {
    out->setName("union");
    out->createMemberXMLAttribute().set("name", node->name);
    type_attrs(node, *out);
    emit_annotations(node, *out);

    auto& disc = out->createMemberXMLElement();
    disc.setName("discriminator");
    define_type(node->discriminator->type, node, disc);
    member_attrs(node->discriminator, disc);

    int last_id = -1;
    for (auto mem : node->members) {
        auto& elem = out->createMemberXMLElement();
        elem.setName("case");
        emit_member(mem, node, elem, last_id);
    }
}

static void emit_enum(const ptree* node, Context& out) {
    out->setName("enum");
    out->createMemberXMLAttribute().set("name", node->name);
    out->createMemberXMLAttribute().set("bitBound", bit_bound(node));
    emit_annotations(node, *out);

    for (auto mem : node->members) {
        auto& elem = out->createMemberXMLElement();
        elem.setName("enumerator");
        elem.createMemberXMLAttribute().set("name", mem->name);
        if (mem->flags & OPT_ENUMERATED) {
            elem.createMemberXMLAttribute().set("value", string_value(mem->value));
        }
        member_attrs(node, elem);
    }
}

static void emit_bitmask(const ptree* node, Context& out) {
    out->setName("bitmask");
    out->createMemberXMLAttribute().set("name", node->name);
    out->createMemberXMLAttribute().set("bitBound", bit_bound(node));
    emit_annotations(node, *out);

    for (auto mem : node->members) {
        auto& elem = out->createMemberXMLElement();
        elem.setName("flag");
        elem.createMemberXMLAttribute().set("name", mem->name);
        elem.createMemberXMLAttribute().set("position", std::to_string(log2(mem)));
        member_attrs(node, elem);
    }
}

static void emit_alias(const ptree* node, Context& out) {
    out->setName("typedef");
    out->createMemberXMLAttribute().set("name", node->name);
    define_type(node->type, node, *out);
    emit_annotations(node, *out);
}

static void emit_const(const ptree* node, Context& out) {
    if (is_basic(node)) {
        out->setName("const");
        out->createMemberXMLAttribute().set("name", node->name);
        define_type(node->type, node, *out);
        out->createMemberXMLAttribute().set("value", string_value(node->value));
        emit_annotations(node, *out);
    }
}

static void recurse_node(const ptree* node, Context& out) {
    if (!is_emit(node, LANG_NONE)) {
        return;
    }
    Context::Scope scope(node, out);

    if (node->flags & OPT_DECLARATION) {
        emit_forward_dcl(node, out);
        return;
    }

    switch (node->kind) {
    case N_ANNOTATION_DEF:
        emit_annotation_def(node, out);
        break;
    case N_MODULE:
        emit_module(node, out);
        break;
    case N_STRUCT:
        emit_struct(node, out);
        break;
    case N_UNION:
        emit_union(node, out);
        break;
    case N_ENUM:
        emit_enum(node, out);
        break;
    case N_BITMASK:
        emit_bitmask(node, out);
        break;
    case N_ALIAS:
        emit_alias(node, out);
        break;
    case N_CONST:
        emit_const(node, out);
        break;
    default:
        break;
    }
}

void generate_xml_type(std::ostream& stream, intercom::XMLElement&& elem, const std::set<ptree*>& includes) {
    intercom::XMLElement root;
    root.setName("dds:types");
    root.createMemberXMLAttribute().set("xmlns:dds", "http://www.omg.org/dds");

    for (auto inc : includes) {
        auto& include = root.createMemberXMLElement();
        include.setName("include");

        intercom::fs::path name(inc->name);
        name.replace_extension(".xml");
        include.createMemberXMLAttribute().set("file", name);
    }
    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        root.createMemberXMLElement() = std::move(elem.xmlElement(i));
    }

    intercom::XMLWriter writer(root);
    writer.setEncoding("UTF-8");
    writer.writeToStream(stream);
}

void code_gen_xml(const parse_result* result) {
    intercom::fs::path dest = CommandLineOption::xml_target_directory();

    Context modules;
    for (auto node : result->tree) {
        recurse_node(node, modules);
    }

    auto& files = modules.files();
    for (auto inc : result->includes) {
        auto p = intercom::fs::path(inc->file_name).replace_extension(".xml").filename();
        if (CommandLineOption::list_only()) {
            std::cout << p.native() << std::endl;
            continue;
        }

        std::set<ptree*> includes;
        include_dependencies(result->tree, inc, includes);

        auto it = files.find(inc->file_name);
        if (it != files.end()) {
            std::stringstream stream;
            generate_xml_type(stream, std::move(it->second), includes);
            write_if_changed(dest / p, stream.str());
        }
    }
}
