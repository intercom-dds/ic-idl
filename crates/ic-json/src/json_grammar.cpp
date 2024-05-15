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

#include <algorithm>

#include "InterCOM/JsonParser.h"
#include "cidl/idl_parser.h"
#include "cidl/internal/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

namespace {
void print_node(intercom::JsonWriter& writer, const numeric& value, const ptree* context, bool value_flag = false) {
    switch (value.kind()) {
    case UNDEF_KIND:
        writer.writeNull();
        break;
    case BOOLEAN_KIND:
        writer.write(value.val.b() != 0);
        break;
    case INT8_KIND:
        writer.write(value.val.i8());
        break;
    case OCTET_KIND:
        writer.write(value.val.o());
        break;
    case SHORT_KIND:
        writer.write(value.val.s());
        break;
    case USHORT_KIND:
        writer.write(value.val.us());
        break;
    case LONG_KIND:
        writer.write(value.val.l());
        break;
    case ULONG_KIND:
        writer.write(value.val.ul());
        break;
    case LONGLONG_KIND:
        writer.write(value.val.ll());
        break;
    case ULONGLONG_KIND:
        writer.write(value.val.ull());
        break;
    case FLOAT_KIND:
        writer.write(value.val.f());
        break;
    case DOUBLE_KIND:
        writer.write(value.val.d());
        break;
    case STRING_KIND:
        writer.writeString(value.val.str());
        break;
    case CHAR_KIND: {
        intercom::corba::WString_var str;
        str.reserve(1);
        str[0] = static_cast<intercom::corba::WString_var::value_type>(value.val.c());
        writer.writeString(str);
    } break;
    case PTREE_KIND: {
        if (value.val.node()->members) {
            if (base_type_of(value.val.node())->kind == N_STRUCT) {
                writer.startObject();
                for (auto p : value.val.node()->members) {
                    writer.writeKey(p->name);
                    print_node(writer, p->value, context, value_flag);
                }
                writer.endObject();
            } else {
                bool wasPretty = writer.isPretty();
                writer.setPretty(false);
                writer.startArray();
                for (auto p : value.val.node()->members) {
                    print_node(writer, p->value, context, value_flag);
                }
                writer.endArray();
                writer.setPretty(wasPretty);
            }
        } else {
            if (!value.val.node()->name.empty() && !value_flag) {
                writer.writeString(idl_scoped_name(value.val.node(), context));
            } else {
                print_node(writer, value.val.node()->value, context, value_flag);
            }
        }
    } break;
    }
}

void print_node(intercom::JsonWriter& writer, const ptree* node) {
    if (!node) {
        writer.writeNull();
        return;
    }

    writer.startObject();
    writer.writeKey("kind");
    writer.writeString(node_kind_str(node->kind));
    if (!node->name.empty()) {
        writer.writeKey("name");
        writer.writeString(node->name);
    }
    if (node->type) {
        writer.writeKey("type");
        writer.writeString(idl_name(node->type));
        if (!node->type->bounds.empty()) {
            writer.writeKey("bounds");
            writer.startArray();
            for (const auto& bound : node->type->bounds) {
                writer.write(integer_value(bound));
            }
            writer.endArray();
        }
    }
    if (node->element_type) {
        writer.writeKey("element_type");
        writer.writeString(idl_name(node->element_type));
    }
    if (node->key_type) {
        writer.writeKey("key_type");
        writer.writeString(idl_name(node->key_type));
    }
    if (node->annotations) {
        writer.writeKey("annotations");
        writer.startArray();
        print_node(writer, node->annotations);
        writer.endArray();
    }
    if (node->members) {
        writer.writeKey("members");
        writer.startArray();
        print_node(writer, node->members);
        writer.endArray();
    }
    if (!node->parents.empty()) {
        writer.writeKey("parents");
        writer.startArray();
        for (auto parent : node->parents) {
            writer.writeString(idl_name(parent));
        }
        writer.endArray();
    }
    if (!node->getraises.empty()) {
        writer.writeKey("getraises");
        writer.startArray();
        for (auto getraise : node->getraises) {
            writer.writeString(idl_name(getraise));
        }
        writer.endArray();
    }
    if (!node->setraises.empty()) {
        writer.writeKey("setraises");
        writer.startArray();
        for (auto setraise : node->setraises) {
            writer.writeString(idl_name(setraise));
        }
        writer.endArray();
    }
    if (node->value.kind() != UNDEF_KIND) {
        writer.writeKey("value");
        print_node(writer, node->value, node->super);
    }
    if (node->flags) {
        writer.writeKey("flags");
        std::vector<std::string> flags;
        if (node->flags & OPT_DECLARATION) {
            flags.emplace_back("OPT_DECLARATION");
        }
        if (node->flags & OPT_IN) {
            flags.emplace_back("OPT_IN");
        }
        if (node->flags & OPT_OUT) {
            flags.emplace_back("OPT_OUT");
        }
        if (node->flags & OPT_READONLY) {
            flags.emplace_back("OPT_READONLY");
        }
        if (node->flags & OPT_PRIVATE) {
            flags.emplace_back("OPT_PRIVATE");
        }
        if (node->flags & OPT_DEFAULT) {
            flags.emplace_back("OPT_DEFAULT");
        }
        if (node->flags & OPT_HAS_CHILDREN) {
            flags.emplace_back("OPT_HAS_CHILDREN");
        }
        std::string str;
        for (const auto& flag : flags) {
            if (!str.empty()) {
                str += "|";
            }
            str += flag;
        }
        writer.writeString(str);
    }
    writer.writeKey("line");
    writer.write(node->pos.line);
    writer.endObject();
}

ptree* parse(const std::string& name, intercom::JsonNode& node);

using NodePair = std::pair<intercom::JsonNode, intercom::JsonNode>;

intercom::JsonNode find_member(const std::string& name, const std::vector<NodePair>& values) {
    auto it = std::find_if(values.begin(), values.end(), [&](const NodePair& x) {
        std::string key;
        return x.first.get_string(key) && key == name;
    });
    return it != values.end() ? it->second : intercom::JsonNode();
}

numeric get_const_expr(const intercom::JsonNode& node) {
    numeric val = num_undef;
    switch (node.get_type()) {
    case intercom::JSON_FLOAT: {
        double x;
        if (node.get_number(x)) {
            val.val.d(x);
        }
        break;
    }
    case intercom::JSON_INTEGER: {
        int64_t x;
        if (node.get_integer(x)) {
            val.val.ll(x);
        }
        break;
    }
    case intercom::JSON_STRING: {
        std::string key;
        if (node.get_string(key)) {
            val.val.node(try_lookup_node(key.c_str(), ANY_KIND));
            if (!val.val.node()) {
                val.val.str(key);
            }
        }
        break;
    }
    case intercom::JSON_ARRAY: {
        std::vector<intercom::JsonNode> values;
        if (node.get_array(values)) {
            ptree* members = nullptr;
            for (auto& value : values) {
                auto expr = get_const_expr(value);
                members = append_node(members, create_const_node(nullptr, nullptr, &expr));
            }
            val = *create_value_node(&num_undef, members);
        }
        break;
    }
    case intercom::JSON_OBJECT: {
        std::map<std::string, intercom::JsonNode> values;
        if (node.get_object(values)) {
            ptree* members = nullptr;
            for (auto& value : values) {
                declarator decl;
                decl.ident = create_identifier(value.first.c_str());
                auto expr = get_const_expr(value.second);
                members = append_node(members, create_const_node(&decl, nullptr, &expr));
            }
            val = *create_value_node(&num_undef, members);
        }
        break;
    }
    case intercom::JSON_BOOL: {
        bool x;
        if (node.get_bool(x)) {
            val.val.b(x);
        }
        break;
    }
    case intercom::JSON_NULL:
    case intercom::JSON_PARSE_ERROR:
        break;
    }
    return val;
}

ptree* get_primitive_type(const std::string& kind) {
    if (kind == "int8") {
        return &int8_type;
    }
    if (kind == "int16" || kind == "short") {
        return &short_type;
    }
    if (kind == "int32" || kind == "long") {
        return &long_type;
    }
    if (kind == "int64" || kind == "long long") {
        return &longlong_type;
    }
    if (kind == "uint8" || kind == "byte" || kind == "octet") {
        return &octet_type;
    }
    if (kind == "uint16" || kind == "unsigned short") {
        return &ushort_type;
    }
    if (kind == "uint32" || kind == "unsigned long") {
        return &ulong_type;
    }
    if (kind == "uint64" || kind == "unsigned long long") {
        return &ulonglong_type;
    }
    if (kind == "float32" || kind == "float") {
        return &float_type;
    }
    if (kind == "float64" || kind == "double") {
        return &double_type;
    }
    if (kind == "float128" || kind == "long double") {
        return &ldouble_type;
    }
    if (kind == "boolean") {
        return &boolean_type;
    }
    if (kind == "char8" || kind == "char") {
        return &char_type;
    }
    if (kind == "char16" || kind == "wchar") {
        return &wchar_type;
    }
    if (kind == "string") {
        return &unbounded_string_type;
    }
    if (kind == "wstring") {
        return &unbounded_wstring_type;
    }
    return nullptr;
}

ptree* get_type(const intercom::JsonNode& node) {
    ptree* res = nullptr;
    std::map<std::string, intercom::JsonNode> values;
    if (node.get_object(values)) {
        std::string kind;
        values["kind"].get_string(kind);
        if (kind == "string") {
            numeric max_length = get_const_expr(values["string_max_length"]);
            res = create_string(&max_length);
        } else if (kind == "wstring") {
            numeric max_length = get_const_expr(values["string_max_length"]);
            res = create_wstring(&max_length);
        } else if (kind == "sequence") {
            auto expr = get_const_expr(values["sequence_max_length"]);
            res = create_sequence(get_type(values["type"]), &expr);
        } else if (kind == "array") {
            declarator decl;
            std::vector<intercom::JsonNode> bounds;
            if (values["array_dimensions"].get_array(bounds)) {
                for (auto& bound : bounds) {
                    auto expr = get_const_expr(bound);
                    append_array_size(&decl, &expr);
                }
            }
            res = create_array_type(&decl, get_type(values["type"]));
        } else if (kind == "map") {
            auto expr = get_const_expr(values["map_max_length"]);
            res = create_map(get_type(values["key_type"]), get_type(values["value_type"]), &expr);
        } else {
            res = get_primitive_type(kind);
            if (!res) {
                std::string type;
                values["type"].get_string(type);
                res = lookup_type(create_identifier(type.c_str()));
            }
        }
    } else {
        std::string type;
        node.get_string(type);
        res = get_primitive_type(type);
        if (!res) {
            res = lookup_type(create_identifier(type.c_str()));
        }
    }
    if (res == nullptr) {
        throw std::runtime_error("Type lookup failed");
    }
    return res;
}

ptree* parse_annotations(const intercom::JsonNode& annotations) {
    ptree* res = nullptr;
    std::map<std::string, intercom::JsonNode> annotation_map;
    if (annotations.get_object(annotation_map)) {
        for (auto& annotation : annotation_map) {
            std::string name = "@" + annotation.first;
            create_annotation_start(create_identifier(name.c_str()));
            ptree* args = nullptr;
            if (annotation.second.get_type() == intercom::JSON_OBJECT) {
                std::map<std::string, intercom::JsonNode> annotation_arg_map;
                if (annotation.second.get_object(annotation_arg_map)) {
                    for (auto& arg : annotation_arg_map) {
                        auto expr = get_const_expr(arg.second);
                        args = append_node(args, create_annotation_param(create_identifier(arg.first.c_str()), &expr));
                    }
                }
            } else {
                if (get_const_expr(annotation.second).kind() != UNDEF_KIND) {
                    auto expr = get_const_expr(annotation.second);
                    args = create_annotation_param(create_identifier(nullptr), &expr);
                }
            }
            res = append_node(res, create_annotation_finish(args));
        }
    }
    return res;
}

ptree* parse_enum(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    std::vector<intercom::JsonNode> enumerators;
    find_member("enumerators", members).get_array(enumerators);
    for (auto& value : enumerators) {
        std::map<std::string, intercom::JsonNode> valueMap;
        std::string value_name;
        if (value.get_object(valueMap) && valueMap["name"].get_string(value_name)) {
            auto expr = get_const_expr(valueMap["value"]);
            ptree* elem = create_enum_value(create_identifier(value_name.c_str()), &expr);
            numeric is_default = get_const_expr(valueMap["default"]);
            if (is_default.kind() == BOOLEAN_KIND && is_default.val.b()) {
                create_annotation_start(create_identifier("@default_literal"));
                annotate(elem, create_annotation_finish(nullptr));
            }
            node = append_enum_node(node, elem);
        }
    }
    return create_enum(create_identifier(name.c_str()), node, {0, 0});
}

ptree* parse_struct(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    ptree* parent = nullptr;
    if (members.empty()) {
        return create_struct_dcl(create_identifier(name.c_str()));
    }
    intercom::JsonNode parent_node = find_member("base_type", members);
    if (parent_node.get_type() != intercom::JSON_NULL) {
        parent = get_type(parent_node);
    }
    create_struct_start(create_identifier(name.c_str()), parent);
    std::vector<intercom::JsonNode> struct_members;
    find_member("members", members).get_array(struct_members);
    for (auto& value : struct_members) {
        std::map<std::string, intercom::JsonNode> member;
        std::string value_name;
        if (value.get_object(member) && member["name"].get_string(value_name)) {
            declarator decl;
            decl.ident = create_identifier(value_name.c_str());
            node = append_node(node, create_member(&decl, get_type(value), parse_annotations(member["annotations"])));
        }
    }
    return create_struct_finish(node, {0, 0});
}

ptree* parse_union(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    ptree* discr = nullptr;
    if (members.empty()) {
        return create_union_dcl(create_identifier(name.c_str()));
    }
    create_union_start(create_identifier(name.c_str()));
    {
        std::map<std::string, intercom::JsonNode> discriminator_map;
        intercom::JsonNode discr_value = find_member("discriminator", members);
        discr_value.get_object(discriminator_map);
        declarator decl;
        decl.ident = create_identifier("_d");
        discr = create_member(&decl, get_type(discr_value), parse_annotations(discriminator_map["annotations"]));
    }

    std::vector<intercom::JsonNode> union_members;
    find_member("cases", members).get_array(union_members);
    for (auto& value : union_members) {
        std::map<std::string, intercom::JsonNode> member;
        std::string value_name;
        if (value.get_object(member) && member["name"].get_string(value_name) && member.find("case") != member.end()) {
            std::vector<intercom::JsonNode> union_cases;
            if (!member["case"].get_array(union_cases)) {
                union_cases.push_back(member["case"]);
            }
            for (auto& union_case : union_cases) {
                numeric case_value = get_const_expr(union_case);
                if (case_value.kind() == STRING_KIND && case_value.val.str() == "default") {
                    node = append_node(node, create_default_case());
                } else {
                    node = append_node(node, create_case_label(&case_value));
                }
            }
            declarator decl;
            decl.ident = create_identifier(value_name.c_str());
            node = append_node(node, create_member(&decl, get_type(value), parse_annotations(member["annotations"])));
        }
    }
    return create_union_finish(discr, node, {0, 0});
}

ptree* parse_bitmask(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    std::vector<intercom::JsonNode> bitfields;
    find_member("flags", members).get_array(bitfields);
    for (auto& value : bitfields) {
        std::map<std::string, intercom::JsonNode> valueMap;
        std::string value_name;
        if (value.get_object(valueMap) && valueMap["name"].get_string(value_name)) {
            auto expr = get_const_expr(valueMap["position"]);
            ptree* elem = create_bitmask_value(create_identifier(value_name.c_str()), &expr);
            node = append_enum_node(node, elem);
        }
    }
    return create_bitmask(create_identifier(name.c_str()), node, {0, 0});
}

ptree* parse_bitset(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    ptree* base_type = nullptr;
    std::vector<intercom::JsonNode> enumerators;
    std::string type;
    if (find_member("type", members).get_string(type)) {
        base_type = get_primitive_type(type);
    }
    find_member("bitfields", members).get_array(enumerators);
    for (auto& value : enumerators) {
        std::map<std::string, intercom::JsonNode> valueMap;
        std::string value_name;
        if (value.get_object(valueMap) && valueMap["name"].get_string(value_name)) {
            ptree* field_type = base_type;
            if (valueMap.find("type") != valueMap.end()) {
                field_type = get_type(valueMap["type"]);
            }
            declarator decl;
            decl.ident = create_identifier(value_name.c_str());

            auto bits = get_const_expr(valueMap["bits"]);
            ptree* elem = create_bitfield(&decl, &bits, field_type);
            node = append_node(node, elem);
        }
    }
    return create_bitset(create_identifier(name.c_str()), node, base_type, {0, 0});
}

ptree* parse_module(const std::string& name, std::vector<NodePair>& members) {
    ptree* res = nullptr;
    ptree* node = nullptr;
    if (!name.empty()) {
        create_module_start(create_identifier(name.c_str()));
    }
    for (auto& element : members) {
        std::string key;
        if (element.first.get_string(key)) {
            node = append_node(node, parse(key, element.second));
        }
    }
    if (!name.empty()) {
        res = create_module_finish(node, {0, 0});
    } else {
        res = node;
    }
    return res;
}

ptree* parse_typedef(const std::string& name, std::vector<NodePair>& members) {
    declarator decl;
    decl.ident = create_identifier(name.c_str());
    return annotate(create_type(&decl, get_type(find_member("type", members))),
                    parse_annotations(find_member("annotations", members)));
}

ptree* parse_const(const std::string& name, std::vector<NodePair>& members) {
    declarator decl;
    decl.ident = create_identifier(name.c_str());
    auto expr = get_const_expr(find_member("value", members));
    return create_const_node(&decl, get_type(find_member("type", members)), &expr);
}

ptree* parse_annotation_definition(const std::string& name, std::vector<NodePair>& members) {
    ptree* node = nullptr;
    create_annotation_dcl_start(create_identifier(name.c_str()));
    std::vector<NodePair> struct_members;
    find_member("members", members).get_object(struct_members);
    for (auto& value : struct_members) {
        std::map<std::string, intercom::JsonNode> member;
        std::string value_name;
        if (value.first.get_string(value_name) && value.second.get_object(member)) {
            declarator decl;
            decl.ident = create_identifier(value_name.c_str());
            auto expr = get_const_expr(member["default"]);
            node = append_node(node, create_annotation_member(&decl, get_type(value.second), &expr));
        }
    }
    return create_annotation_dcl_finish(node, {0, 0});
}

ptree* parse(const std::string& name, intercom::JsonNode& node) {
    current_pos.line = static_cast<int>(node.get_data().line());
    ptree* res = nullptr;
    std::vector<NodePair> values;
    if (node.get_object(values)) {
        std::string kind;
        intercom::JsonNode kind_node = find_member("kind", values);
        if (kind_node.get_string(kind)) {
            intercom::JsonNode annotations = find_member("annotations", values);
            values.erase(std::remove_if(values.begin(), values.end(),
                                        [&](const NodePair& x) {
                                            std::string key;
                                            return x.first.get_string(key) && (key == "kind" || key == "annotations");
                                        }),
                         values.end());
            if (kind == "enum") {
                res = parse_enum(name, values);
            } else if (kind == "struct") {
                res = parse_struct(name, values);
            } else if (kind == "union") {
                res = parse_union(name, values);
            } else if (kind == "bitmask") {
                res = parse_bitmask(name, values);
            } else if (kind == "bitset") {
                res = parse_bitset(name, values);
            } else if (kind == "typedef") {
                res = parse_typedef(name, values);
            } else if (kind == "module") {
                res = parse_module(name, values);
            } else if (kind == "const") {
                res = parse_const(name, values);
            } else if (kind == "annotation") {
                res = parse_annotation_definition(name, values);
            } else {
                res = get_type(node);
            }
            annotate(res, parse_annotations(annotations));
        }
    } else if (node.get_type() == intercom::JSON_PARSE_ERROR) {
        ERR << "Failed to parse JSON near line " << node.get_data().line() << ": " << node.get_data().str();
    } else {
        ERR << "Unexpected JSON type " << node.get_type() << ", expected object";
    }
    return res;
}
}  // namespace

namespace intercom {
namespace cidl {
std::string json_value(const numeric& value, const ptree* context, int flags) {
    std::stringstream out;
    JsonWriter writer(out);
    print_node(writer, value, context, (flags & int(JsonValueFlags::FLAG_NUMERICAL_VALUE)) != 0);
    if (flags & int(JsonValueFlags::FLAG_ESCAPED)) {
        std::stringstream escape_out;
        JsonWriter escape_writer(escape_out);
        escape_writer.writeString(out.str());
        return escape_out.str();
    }
    return out.str();
}

std::string json_value(const ptree* obj) {
    std::stringstream out;
    JsonWriter writer(out);
    print_node(writer, obj);
    return out.str();
}

ptree* parse_json_ptree(const std::string& input) {
    JsonData jsonData(input.c_str(), input.size());
    auto node = JsonNode::from_data(jsonData);
    if (node.get_type() == JSON_PARSE_ERROR) {
        ERR << "Failed to parse JSON near line " << node.get_data().line() << ": " << node.get_data().str();
        return nullptr;
    }
    std::vector<std::pair<intercom::JsonNode, intercom::JsonNode>> values;
    if (node.get_object(values)) {
        try {
            return parse_module("", values);
        } catch (std::exception& e) {
            ERR << "Failed to parse JSON module: " << e.what();
        }
    } else {
        ERR << "Unexpected JSON type " << node.get_type() << ", expected object";
    }
    return nullptr;
}
}  // namespace cidl
}  // namespace intercom
