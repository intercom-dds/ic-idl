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

#include <fstream>

#include "InterCOM/json_parser.h"
#include "cidl/commandline.h"
#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

static void print_json(intercom::JsonWriter& writer, const ptree* obj);

static void print_annotations(intercom::JsonWriter& writer, const ptree* obj) {
    if (obj) {
        writer.writeKey("annotations");
        writer.startObject();
        for (const auto& ann : obj) {
            print_json(writer, ann);
        }
        writer.endObject();
    }
}

static void print_kind(intercom::JsonWriter& writer, const std::string& kind) {
    writer.writeKey("kind");
    writer.writeString(kind);
}

static void print_json_type(intercom::JsonWriter& writer, const ptree* type, const ptree* context) {
    bool print_type = false;
    switch (type->kind) {
    case N_PRIMITIVE:
        print_kind(writer, type->name);
        break;
    case N_STRING:
        print_kind(writer, type->element_type == &char_type ? "string" : "wstring");
        if (!type->bounds.empty()) {
            writer.writeKey("string_max_length");
            writer.writeJson(json_value(type->bounds[0], context));
        }
        break;
    case N_SEQUENCE:
        print_kind(writer, "sequence");
        writer.writeKey("type");
        writer.writeString(idl_scoped_name(type->element_type, context));
        if (!type->bounds.empty()) {
            writer.writeKey("sequence_max_length");
            writer.writeJson(json_value(type->bounds[0], context));
        }
        break;
    case N_MAP:
        print_kind(writer, "map");
        writer.writeKey("key_type");
        writer.writeString(idl_scoped_name(type->key_type, context));
        writer.writeKey("value_type");
        writer.writeString(idl_scoped_name(type->element_type, context));
        if (!type->bounds.empty()) {
            writer.writeKey("map_max_length");
            writer.writeJson(json_value(type->bounds[0], context));
        }
        break;
    case N_ARRAY:
        print_kind(writer, "array");
        writer.writeKey("type");
        writer.writeString(idl_scoped_name(type->element_type, context));
        writer.writeKey("array_dimensions");
        writer.startArray();
        for (const auto& b : type->bounds) {
            writer.writeJson(json_value(b, context));
        }
        writer.endArray();
        break;
    case N_EXCEPTION:
        print_kind(writer, "exception");
        print_type = true;
        break;
    case N_VALUETYPE:
        print_kind(writer, "valuetype");
        print_type = true;
        break;
    case N_STRUCT:
        print_kind(writer, "struct");
        print_type = true;
        break;
    case N_UNION:
        print_kind(writer, "union");
        print_type = true;
        break;
    case N_ALIAS:
        print_kind(writer, "typedef");
        print_type = true;
        break;
    case N_ENUM:
        print_kind(writer, "enum");
        print_type = true;
        break;
    case N_BITMASK:
        print_kind(writer, "bitmask");
        print_type = true;
        break;
    case N_BITSET:
        print_kind(writer, "bitset");
        print_type = true;
        break;
    default:
        break;
    }
    if (print_type) {
        writer.writeKey("type");
        writer.writeString(idl_scoped_name(type, context));
    }
}

static void print_json_member(intercom::JsonWriter& writer, const ptree* member) {
    if (member->kind == N_MEMBER) {
        writer.startObject();
        for (auto cas : member->members) {
            writer.writeKey("case");
            if (cas->next && cas->next != member) {
                writer.startArray();
                for (auto c = cas; c && c != member; c = c->next) {
                    writer.writeJson(json_value(c->value, member->super));
                }
                writer.endArray();
            } else if (cas->flags & OPT_DEFAULT) {
                writer.writeString("default");
            } else {
                writer.writeJson(json_value(cas->value, member->super));
            }
        }
        writer.writeKey("name");
        writer.writeString(member->name);
        print_annotations(writer, member->annotations);
        print_json_type(writer, member->type, member->super);
        writer.endObject();
    } else if (member->kind == N_PROTOTYPE) {
    }
}

static void print_json(intercom::JsonWriter& writer, const ptree* obj) {
    if (obj->flags & OPT_DECLARATION) {
        switch (obj->kind) {
        case N_STRUCT:
            writer.writeKey(obj->name);
            writer.startObject();
            print_kind(writer, "struct");
            writer.endObject();
            break;
        case N_UNION:
            writer.writeKey(obj->name);
            writer.startObject();
            print_kind(writer, "union");
            writer.endObject();
            break;
        case N_VALUETYPE:
            writer.writeKey(obj->name);
            writer.startObject();
            print_kind(writer, "valuetype");
            writer.endObject();
            break;
        case N_INTERFACE:
            writer.writeKey(obj->name);
            writer.startObject();
            print_kind(writer, "interface");
            writer.endObject();
            break;
        default:
            break;
        }
        return;
    }
    switch (obj->kind) {
    case N_MODULE:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "module");
        print_annotations(writer, obj->annotations);
        for (const auto& elem : obj->members) {
            print_json(writer, elem);
        }
        writer.endObject();
        break;
    case N_STRUCT:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "struct");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.writeKey("base_type");
            writer.startObject();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.endObject();
        }
        writer.writeKey("members");
        writer.startArray();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_UNION:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "union");
        print_annotations(writer, obj->annotations);
        writer.writeKey("discriminator");
        writer.startObject();
        print_annotations(writer, obj->discriminator->annotations);
        print_json_type(writer, obj->discriminator->type, obj->super);
        writer.endObject();
        writer.writeKey("cases");
        writer.startArray();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_VALUETYPE:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "valuetype");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.writeKey("base_type");
            writer.startObject();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.endObject();
        }
        if (obj->type) {
            writer.writeKey("interface");
            writer.startObject();
            print_json_type(writer, obj->type, obj->super);
            writer.endObject();
        }
        writer.writeKey("attributes");
        writer.startArray();
        for (const auto& elem : obj->members) {
            if (obj->kind == N_MEMBER) {
                print_json_member(writer, elem);
            }
        }
        writer.endArray();
        writer.writeKey("methods");
        writer.startArray();
        for (const auto& elem : obj->members) {
            if (obj->kind == N_PROTOTYPE) {
                print_json_member(writer, elem);
            }
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_INTERFACE:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "interface");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.writeKey("base_type");
            if (obj->parents.size() > 1) {
                writer.startArray();
                for (auto parent : obj->parents) {
                    writer.startObject();
                    print_json_type(writer, parent, obj->super);
                    writer.endObject();
                }
                writer.endArray();
            } else {
                writer.startObject();
                print_json_type(writer, obj->parents[0], obj->super);
                writer.endObject();
            }
        }
        writer.writeKey("members");
        writer.startArray();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_EXCEPTION:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "exception");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.writeKey("base_type");
            writer.startObject();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.endObject();
        }
        writer.writeKey("members");
        writer.startArray();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_ENUM:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "enum");
        print_annotations(writer, obj->annotations);
        writer.writeKey("enumerators");
        writer.startArray();
        for (const auto& elem : obj->members) {
            writer.startObject();
            writer.writeKey("name");
            writer.writeString(elem->name);
            if (elem->flags & OPT_ENUMERATED) {
                writer.writeKey("value");
                writer.writeJson(json_value(elem->value, obj));
            }
            if (get_annotation(elem, annotation_type_default_literal) != nullptr) {
                writer.writeKey("default");
                writer.write(true);
            }
            writer.endObject();
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_BITSET:
        break;
    case N_BITMASK:
        writer.writeKey(obj->name);
        writer.startObject();
        print_kind(writer, "bitmask");
        print_annotations(writer, obj->annotations);
        writer.writeKey("flags");
        writer.startArray();
        for (const auto& elem : obj->members) {
            writer.startObject();
            writer.writeKey("name");
            writer.writeString(elem->name);
            if (elem->flags & OPT_ENUMERATED) {
                writer.writeKey("position");
                writer.writeJson(json_value(elem->value, obj));
            }
            writer.endObject();
        }
        writer.endArray();
        writer.endObject();
        break;
    case N_ALIAS:
        writer.writeKey(obj->name);
        writer.startObject();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "typedef");
        writer.writeKey("type");
        writer.startObject();
        print_json_type(writer, obj->type, obj->super);
        writer.endObject();
        writer.endObject();
        break;
    case N_CONST:
        writer.writeKey(obj->name);
        writer.startObject();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "const");
        writer.writeKey("type");
        writer.startObject();
        print_json_type(writer, obj->type, obj->super);
        writer.endObject();
        writer.writeKey("value");
        writer.writeJson(json_value(obj->value, obj));
        writer.endObject();
        break;
    case N_ANNOTATION_DEF:
        writer.writeKey(obj->name);
        writer.startObject();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "annotation");
        if (obj->members) {
            writer.writeKey("members");
            writer.startObject();
            for (const auto* elem : obj->members) {
                writer.writeKey(elem->name);
                writer.startObject();
                print_json_type(writer, elem->type, obj);
                if (elem->value.kind() != UNDEF_KIND) {
                    writer.writeKey("default");
                    writer.writeJson(json_value(elem->value, obj));
                }
                writer.endObject();
            }
            writer.endObject();
        }
        writer.endObject();
        break;
    case N_ANNOTATION:
        writer.writeKey(idl_scoped_name(obj, namespace_of(annotation_type_key)));
        if (obj->members && obj->members->next) {
            writer.startObject();
            for (const auto& elem : obj->members) {
                writer.writeKey(elem->name);
                writer.writeJson(json_value(elem->value, obj->type));
            }
            writer.endObject();
        } else if (obj->members) {
            writer.writeJson(json_value(obj->members->value, obj->type));
        } else {
            writer.startObject();
            writer.endObject();
        }
        break;
    default:
        break;
    }
}

static void print_node(
    intercom::JsonWriter& writer,
    const numeric& value,
    const ptree* context,
    bool value_flag = false
) {
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
        // TODO(idarcar);
        // intercom::corba::WString_var str;
        // str.reserve(1);
        // str[0] = static_cast<intercom::corba::WString_var::value_type>(value.val.c());
        // writer.writeString(str);
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
                bool was_pretty = writer.isPretty();
                writer.setPretty(false);
                writer.startArray();
                for (auto p : value.val.node()->members) {
                    print_node(writer, p->value, context, value_flag);
                }
                writer.endArray();
                writer.setPretty(was_pretty);
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

static void print_node(intercom::JsonWriter& writer, const ptree* node) {
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
    writer.endObject();
}

std::string intercom::cidl::json_value(const numeric& value, const ptree* context, int flags) {
    std::stringstream out;
    intercom::JsonWriter writer(out);
    print_node(writer, value, context, (flags & int(JsonValueFlags::FLAG_NUMERICAL_VALUE)) != 0);
    if (flags & int(JsonValueFlags::FLAG_ESCAPED)) {
        std::stringstream escape_out;
        intercom::JsonWriter escape_writer(escape_out);
        escape_writer.writeString(out.str());
        return escape_out.str();
    }
    return out.str();
}

std::string intercom::cidl::json_value(const ptree* obj) {
    std::stringstream out;
    intercom::JsonWriter writer(out);
    print_node(writer, obj);
    return out.str();
}

void intercom::cidl::code_gen_json(const parse_result* result, const char* destination) {
    for (auto include : result->includes) {
        std::string file_name = trim_include_name(include->name, true);
        file_name += ".json";
        std::string filepath = std::string(destination) + "/" + file_name;
        std::stringstream file;
        {
            intercom::JsonWriter writer(file, true);
            writer.startObject();
            for (const auto& obj : result->tree) {
                if (is_emit(obj, LANG_NONE) && obj->included_from == include) {
                    print_json(writer, obj);
                }
            }
            writer.endObject();
        }
        if (!file.str().empty()) {
            write_if_changed(filepath, file.str());
        }
    }
}

void intercom::cidl::generate_json_type(std::ostream& stream, const ptree* tree) {
    intercom::JsonWriter writer(stream, true);
    writer.startObject();
    print_json(writer, tree);
    writer.endObject();
}
