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

#include <string>

#include "cidl/constants.h"
#include "cidl/idl_parser.h"
#include "cidl/internal/ptree_builder.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "utils/StringUtils.h"
#include "utils/XMLReader.h"

// NOLINTBEGIN(*-avoid-c-arrays)

const position POS{0, 0};

static ptree* parse_any(const intercom::XMLElement&, const char**);

static std::string_view tag_name(const intercom::XMLElement& elem) {
    std::string_view name(elem.name());
    auto pos = name.rfind(':');
    if (pos != std::string_view::npos) {
        return name.substr(pos + 1);
    }
    return name;
}

static identifier name(const intercom::XMLElement& elem) {
    const auto& attr = elem.xmlAttribute("name");
    if (!attr.isValid()) {
        ERR << "anonymous types are not supported";
        return create_identifier(nullptr);
    }
    return create_identifier(attr.value().c_str());
}

static declarator* name_decl(const intercom::XMLElement& elem) {
    return create_decl(name(elem), nullptr);
}

static const char* attribute(const intercom::XMLElement& elem, const std::string& name) {
    const auto& attr = elem.xmlAttribute(name);
    return attr.isValid() ? attr.value().c_str() : nullptr;
}

static bool is_number(const char* s) {
    while (*s != '\0') {
        if (!isdigit(*s++)) {
            return false;
        }
    }
    return true;
}

static bool is_double(const char* s) {
    while (*s != '\0') {
        if (!isdigit(*s++) && *s != '.') {
            return false;
        }
    }
    return true;
}

static const numeric* const_expr(const char* s) {
    if (!s) {
        ERR << "Invalid \"value\" attribute";
        return &num_undef;
    }
    if (*s) {
        if (is_number(s)) {
            return create_u64(std::stoull(s), 10);
        }
        if (is_double(s)) {
            double out = 0;
            StringUtils::stringToDouble(s, out);
            return create_double(out);
        }
        node_kind kind[] = {N_CONST};
        if (auto node = try_lookup_node(s, kind)) {
            return &node->value;
        }
    }
    return create_str(s);
}

static const numeric* elem_value(const intercom::XMLElement& elem, const ptree* type) {
    auto attr = attribute(elem, "value");
    if (!attr) {
        ERR << "Missing \"value\" attribute on element " << elem.name();
        return nullptr;
    }

    auto base = base_type_of(type);
    if (base->kind == N_STRING) {
        return create_str(attr);
    }
    return const_expr(attr);
}

static ptree* primitive_type(const std::string& name) {
    if (name == "boolean") {
        return &boolean_type;
    }
    if (name == "char8") {
        return &char_type;
    }
    if (name == "char16") {
        return &wchar_type;
    }
    if (name == "int8") {
        return &int8_type;
    }
    if (name == "uint8") {
        return &octet_type;
    }
    if (name == "int16") {
        return &short_type;
    }
    if (name == "uint16") {
        return &ushort_type;
    }
    if (name == "int32") {
        return &long_type;
    }
    if (name == "uint32") {
        return &ulong_type;
    }
    if (name == "int64") {
        return &longlong_type;
    }
    if (name == "uint64") {
        return &ulonglong_type;
    }
    if (name == "float32") {
        return &float_type;
    }
    if (name == "float64") {
        return &double_type;
    }
    if (name == "float128") {
        return &ldouble_type;
    }
    if (name == "string") {
        return &unbounded_string_type;
    }
    if (name == "wstring") {
        return &unbounded_wstring_type;
    }
    return nullptr;
}

static const numeric* create_bound(const char* value) {
    if (value) {
        auto bound = const_expr(value);
        if (long_long_value(*bound) > 0) {
            return bound;
        }
    }
    return &num_undef;
}

static ptree* elem_type(const intercom::XMLElement& elem) {
    ptree* type = nullptr;
    auto type_attr = attribute(elem, "type");
    if (type_attr) {
        if (strcmp(type_attr, "nonBasic") == 0) {
            if (auto non_basic = attribute(elem, "nonBasicTypeName")) {
                type = lookup_type(create_identifier(non_basic));
            }
        } else {
            type = primitive_type(type_attr);
        }
    } else {
        ERR << "Missing \"type\" attribute on element " << elem.name();
        return nullptr;
    }

    // Start with maps as they have the lowest precedence
    if (auto key_attr = attribute(elem, "mapKeyType")) {
        ptree* key_type = nullptr;
        if (strcmp(key_attr, "nonBasic") == 0) {
            if (auto non_basic = attribute(elem, "mapKeyNonBasicTypeName")) {
                key_type = lookup_type(create_identifier(non_basic));
            }
        } else {
            if (auto str_bound = attribute(elem, "mapKeyStringMaxLength")) {
                key_type = create_string(create_bound(str_bound));
            } else {
                key_type = primitive_type(key_attr);
            }
        }
        auto bound = attribute(elem, "mapMaxLength");
        type = create_map(key_type, type, create_bound(bound));
    }

    if (auto arr = attribute(elem, "arrayDimensions")) {
        auto decl = create_decl(create_identifier(type_attr), nullptr);
        decl = append_array_size(decl, create_bound(arr));
        type = create_array_type(decl, type);
    }

    if (auto seq = attribute(elem, "sequenceMaxLength")) {
        type = create_sequence(type, create_bound(seq));
    }
    return type;
}

static ptree* parse_include(const intercom::XMLElement& elem) {
    auto uri = attribute(elem, "file");
    if (!uri) {
        ERR << "No file specified in include tag";
        return nullptr;
    }

    if (!std::filesystem::exists(uri)) {
        for (const auto& inc : CommandLineOption::include_directories()) {
            std::filesystem::path path(inc);
            path /= uri;
            if (std::filesystem::exists(inc)) {
                return intercom::cidl::parse_xml_file(uri);
            }
        }
    }
    return intercom::cidl::parse_xml_file(uri);
}

static ptree* parse_module(const intercom::XMLElement& elem) {
    create_module_start(name(elem));
    ptree* members = nullptr;
    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        members = append_node(members, parse_any(elem.xmlElement(i), nullptr));
    }
    return create_module_finish(members, POS);
}

static ptree* parse_dcl(const intercom::XMLElement& elem) {
    if (auto attr = attribute(elem, "kind")) {
        if (strcmp(attr, "struct") == 0) {
            return create_struct_dcl(name(elem));
        }
        if (strcmp(attr, "union") == 0) {
            return create_union_dcl(name(elem));
        }
        ERR << "Unknown kind " << attr << " specified in forward_dcl";
    } else {
        ERR << "No kind specified in forward_dcl";
    }
    return nullptr;
}

static ptree* parse_struct(const intercom::XMLElement& elem) {
    create_struct_start(name(elem), nullptr);
    ptree* members = nullptr;
    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        const char* filter[] = {"annotate", "struct", "union", "enum", "bitmask", "member", nullptr};
        members = append_node(members, parse_any(mem, filter));
    }
    return create_struct_finish(members, POS);
}

static ptree* parse_union_case(const intercom::XMLElement& elem, const ptree* disc) {
    ptree* cases = nullptr;
    ptree* member = nullptr;

    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        if (tag_name(mem) == "caseDiscriminator") {
            ptree* label = nullptr;
            if (auto value = attribute(mem, "value")) {
                if (strcmp(value, "default") == 0) {
                    label = create_default_case();
                } else {
                    label = create_case_label(elem_value(mem, disc));
                }
            } else {
                label = create_case_label(elem_value(mem, disc));
            }
            cases = append_node(cases, label);
        } else if (tag_name(mem) == "member") {
            auto type = elem_type(mem);
            member = create_member(name_decl(mem), type, nullptr);
        } else {
            WARN << "Unexpected tag \"" << tag_name(mem) << "\" in union case";
        }
    }
    return create_union_member(member, cases, nullptr);
}

static ptree* parse_union(const intercom::XMLElement& elem) {
    auto it = elem.find("discriminator");
    if (!it.isValid()) {
        ERR << "Discriminator not defined for union";
        return nullptr;
    }

    ptree* members = nullptr;
    ptree* disc = create_member(create_decl(create_identifier("_d"), nullptr), elem_type(it), nullptr);
    create_union_start(name(elem));

    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        if (tag_name(mem) == "case") {
            members = append_node(members, parse_union_case(mem, disc));
        } else if (tag_name(mem) != "discriminator") {
            ERR << "Unexpected tag \"" << tag_name(mem) << "\" in union";
        }
    }
    return create_union_finish(disc, members, POS);
}

static ptree* parse_enum_value(const intercom::XMLElement& elem) {
    const numeric* num = &num_undef;
    if (auto value = attribute(elem, "value")) {
        num = const_expr(value);
    }
    return create_enum_value(name(elem), num);
}

static ptree* parse_enum(const intercom::XMLElement& elem) {
    ptree* values = nullptr;
    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        const char* filter[] = {"annotate", "enumerator", nullptr};
        values = append_enum_node(values, parse_any(mem, filter));
    }
    return create_enum(name(elem), values, POS);
}

static ptree* parse_bitmask_value(const intercom::XMLElement& elem) {
    const numeric* num = &num_undef;
    if (auto value = attribute(elem, "position")) {
        num = create_u64(std::stoull(value), 10);
    }
    return create_bitmask_value(name(elem), num);
}

static ptree* parse_bitmask(const intercom::XMLElement& elem) {
    ptree* flags = nullptr;
    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        const char* filter[] = {"annotate", "flag", nullptr};
        flags = append_node(flags, parse_any(mem, filter));
    }
    return create_bitmask(name(elem), flags, POS);
}

static ptree* parse_typedef(const intercom::XMLElement& elem) {
    return create_type(name_decl(elem), elem_type(elem));
}

static ptree* parse_const(const intercom::XMLElement& elem) {
    auto type = elem_type(elem);
    return create_const_node(name_decl(elem), type, elem_value(elem, type));
}

static ptree* parse_annotation(const intercom::XMLElement& elem) {
    ptree* members = nullptr;
    create_annotation_dcl_start(name(elem));

    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        const char* filter[] = {"annotate", "enum", "const", "typedef", "member", nullptr};
        members = append_node(members, parse_any(mem, filter));
    }
    return create_annotation_dcl_finish(members, POS);
}

static ptree* parse_annotate(const intercom::XMLElement& elem) {
    ptree* params = nullptr;
    auto annotation = fmt::format("@{}", name(elem).name);
    create_annotation_start(create_identifier(annotation.c_str()));

    for (size_t i = 0; i < elem.numberOfXMLElements(); i++) {
        const auto& mem = elem.xmlElement(i);
        const numeric* value = &num_undef;
        if (auto v = attribute(mem, "value")) {
            value = const_expr(v);
        }
        params = append_node(params, create_annotation_param(name(mem), value));
    }
    return create_annotation_finish(params);
}

static void add_annotation(ptree* node, const ptree* annotation_type, ptree* params) {
    auto ann_name = fmt::format("@{}", annotation_type->name);
    create_annotation_start(create_identifier(ann_name.c_str()));
    annotate(node, create_annotation_finish(params));
}

static void type_annotations(const intercom::XMLElement& elem, ptree* node) {
    if (elem.xmlAttribute("nested").value() == "true") {
        add_annotation(node, annotation_type_nested, nullptr);
    }
    if (elem.xmlAttribute("autoid").value() == "hash") {
        auto param = create_annotation_param(create_identifier("value"), create_str("HASH"));
        add_annotation(node, annotation_type_nested, param);
    }
    if (auto ext = elem.xmlAttribute("extensibility")) {
        if (ext.value() == "mutable") {
            add_annotation(node, annotation_type_mutable, nullptr);
        } else if (ext.value() == "final") {
            add_annotation(node, annotation_type_final, nullptr);
        }
    }
}

static ptree* parse_member(const intercom::XMLElement& elem) {
    auto member = create_member(name_decl(elem), elem_type(elem), nullptr);
    auto add_if = [&](const std::string& name, const ptree* ann) {
        if (elem.xmlAttribute(name).value() == "true") {
            add_annotation(member, ann, nullptr);
        }
    };

    add_if("key", annotation_type_key);
    add_if("optional", annotation_type_optional);
    add_if("external", annotation_type_external);
    add_if("mustUnderstand", annotation_type_must_understand);
    add_if("nonSerialized", annotation_type_non_serialized);
    return member;
}

static bool apply_filter(std::string_view tag, const char** filter) {
    if (!filter) {
        return true;
    }
    for (const char* f = *filter; f; f = *++filter) {
        if (tag == f) {
            return true;
        }
    }
    return false;
}

static ptree* parse_any(const intercom::XMLElement& elem, const char** filter) {
    current_pos = {static_cast<int>(elem.lineNumber()) + 1, 0};

    auto tag = tag_name(elem);
    if (!apply_filter(tag, filter)) {
        WARN << "Unexpected tag \"" << elem.name() << "\"";
        return nullptr;
    }

    static std::pair<const char*, ptree* (*)(const intercom::XMLElement&)> s_table[] = {
            {"annotation", parse_annotation},
            {"include", parse_include},
            {"forward_cl", parse_dcl},
            {"module", parse_module},
            {"struct", parse_struct},
            {"union", parse_union},
            {"enum", parse_enum},
            {"bitmask", parse_bitmask},
            {"typedef", parse_typedef},
            {"const", parse_const},
            {"member", parse_member},
            {"enumerator", parse_enum_value},
            {"flag", parse_bitmask_value},
    };

    ptree* node = nullptr;
    for (const auto& f : s_table) {
        if (f.first == tag) {
            node = f.second(elem);
            break;
        }
    }

    for (size_t i = 0; i < elem.numberOfXMLAttributes(); i++) {
        const auto& mem = elem.xmlElement(i);
        if (mem.name() == "annotate") {
            node = annotate(node, parse_annotate(mem));
        }
    }
    return node;
}

ptree* intercom::cidl::parse_xml(const std::string& input) {
    std::stringstream errors;
    XMLReader reader;

    std::stringstream stream(input);
    auto status = reader.read(stream);
    if (status != XMLReader::READ_SUCCESS) {
        ERR << "Failed to parse XML: " << errors.str();
        return nullptr;
    }

    auto types = reader.find("types", false);
    if (types == XMLReader::end()) {
        types = reader.find("dds:types", false);
    }
    if (types == XMLReader::end()) {
        ERR << "No type definitions found in the XML document";
        return nullptr;
    }

    ptree* tree = nullptr;
    for (size_t i = 0; i < types->numberOfXMLElements(); i++) {
        const char* filter[] = {
                "include", "module",  "forward_dcl", "struct",     "union", "enum",
                "bitmask", "typedef", "const",       "annotation", nullptr,
        };
        tree = append_node(tree, parse_any(types->xmlElement(i), filter));
    }
    return tree;
}

ptree* intercom::cidl::parse_xml_file(const std::string& uri) {
    try {
        auto contents = std::filesystem::read_to_string(uri);
        create_include_start(create_identifier(uri.c_str()));
        current_input_file = get_symbol(uri.c_str());
        return create_include_finish(parse_xml(contents));
    } catch (const std::exception& e) {
        ERR << e.what();
    }
    return nullptr;
}

// NOLINTEND(*-avoid-c-arrays)
