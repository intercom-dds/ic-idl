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

#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/ptree.h"
#include "cidl/ptree_ffi.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "ic_cts/json_parser.h"

using namespace intercom::cidl;

static void print_json(ic_cts::JsonWriter& writer, const ptree* obj);

static void print_annotations(ic_cts::JsonWriter& writer, const ptree* obj) {
    if (obj) {
        writer.write_key("annotations");
        writer.start_object();
        for (const auto& ann : obj) {
            print_json(writer, ann);
        }
        writer.end_object();
    }
}

static void print_kind(ic_cts::JsonWriter& writer, const std::string& kind) {
    writer.write_key("kind");
    writer.write_string(kind);
}

static void print_json_type(ic_cts::JsonWriter& writer, const ptree* type, const ptree* context) {
    bool print_type = false;
    switch (type->kind) {
    case N_PRIMITIVE:
        print_kind(writer, type->name);
        break;
    case N_STRING:
        print_kind(writer, type->element_type == &char_type ? "string" : "wstring");
        if (!type->bounds.empty()) {
            writer.write_key("string_max_length");
            writer.write_json(json_value(type->bounds[0], context));
        }
        break;
    case N_SEQUENCE:
        print_kind(writer, "sequence");
        writer.write_key("type");
        writer.write_string(idl_scoped_name(type->element_type, context));
        if (!type->bounds.empty()) {
            writer.write_key("sequence_max_length");
            writer.write_json(json_value(type->bounds[0], context));
        }
        break;
    case N_MAP:
        print_kind(writer, "map");
        writer.write_key("key_type");
        writer.write_string(idl_scoped_name(type->key_type, context));
        writer.write_key("value_type");
        writer.write_string(idl_scoped_name(type->element_type, context));
        if (!type->bounds.empty()) {
            writer.write_key("map_max_length");
            writer.write_json(json_value(type->bounds[0], context));
        }
        break;
    case N_ARRAY:
        print_kind(writer, "array");
        writer.write_key("type");
        writer.write_string(idl_scoped_name(type->element_type, context));
        writer.write_key("array_dimensions");
        writer.start_array();
        for (const auto& b : type->bounds) {
            writer.write_json(json_value(b, context));
        }
        writer.end_array();
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
        writer.write_key("type");
        writer.write_string(idl_scoped_name(type, context));
    }
}

static void print_json_member(ic_cts::JsonWriter& writer, const ptree* member) {
    if (member->kind == N_MEMBER) {
        writer.start_object();
        for (auto cas : member->members) {
            writer.write_key("case");
            if (cas->next && cas->next != member) {
                writer.start_array();
                for (auto c = cas; c && c != member; c = c->next) {
                    writer.write_json(json_value(c->value, member->super));
                }
                writer.end_array();
            } else if (cas->flags & OPT_DEFAULT) {
                writer.write_string("default");
            } else {
                writer.write_json(json_value(cas->value, member->super));
            }
        }
        writer.write_key("name");
        writer.write_string(member->name);
        print_annotations(writer, member->annotations);
        print_json_type(writer, member->type, member->super);
        writer.end_object();
    } else if (member->kind == N_PROTOTYPE) {
    }
}

static void print_json(ic_cts::JsonWriter& writer, const ptree* obj) {
    if (obj->flags & OPT_DECLARATION) {
        switch (obj->kind) {
        case N_STRUCT:
            writer.write_key(obj->name);
            writer.start_object();
            print_kind(writer, "struct");
            writer.end_object();
            break;
        case N_UNION:
            writer.write_key(obj->name);
            writer.start_object();
            print_kind(writer, "union");
            writer.end_object();
            break;
        case N_VALUETYPE:
            writer.write_key(obj->name);
            writer.start_object();
            print_kind(writer, "valuetype");
            writer.end_object();
            break;
        case N_INTERFACE:
            writer.write_key(obj->name);
            writer.start_object();
            print_kind(writer, "interface");
            writer.end_object();
            break;
        default:
            break;
        }
        return;
    }
    switch (obj->kind) {
    case N_MODULE:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "module");
        print_annotations(writer, obj->annotations);
        for (const auto& elem : obj->members) {
            print_json(writer, elem);
        }
        writer.end_object();
        break;
    case N_STRUCT:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "struct");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.write_key("base_type");
            writer.start_object();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.end_object();
        }
        writer.write_key("members");
        writer.start_array();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_UNION:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "union");
        print_annotations(writer, obj->annotations);
        writer.write_key("discriminator");
        writer.start_object();
        print_annotations(writer, obj->discriminator->annotations);
        print_json_type(writer, obj->discriminator->type, obj->super);
        writer.end_object();
        writer.write_key("cases");
        writer.start_array();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_VALUETYPE:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "valuetype");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.write_key("base_type");
            writer.start_object();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.end_object();
        }
        if (obj->type) {
            writer.write_key("interface");
            writer.start_object();
            print_json_type(writer, obj->type, obj->super);
            writer.end_object();
        }
        writer.write_key("attributes");
        writer.start_array();
        for (const auto& elem : obj->members) {
            if (obj->kind == N_MEMBER) {
                print_json_member(writer, elem);
            }
        }
        writer.end_array();
        writer.write_key("methods");
        writer.start_array();
        for (const auto& elem : obj->members) {
            if (obj->kind == N_PROTOTYPE) {
                print_json_member(writer, elem);
            }
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_INTERFACE:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "interface");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.write_key("base_type");
            if (obj->parents.size() > 1) {
                writer.start_array();
                for (auto parent : obj->parents) {
                    writer.start_object();
                    print_json_type(writer, parent, obj->super);
                    writer.end_object();
                }
                writer.end_array();
            } else {
                writer.start_object();
                print_json_type(writer, obj->parents[0], obj->super);
                writer.end_object();
            }
        }
        writer.write_key("members");
        writer.start_array();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_EXCEPTION:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "exception");
        print_annotations(writer, obj->annotations);
        if (!obj->parents.empty()) {
            writer.write_key("base_type");
            writer.start_object();
            print_json_type(writer, obj->parents[0], obj->super);
            writer.end_object();
        }
        writer.write_key("members");
        writer.start_array();
        for (const auto& elem : obj->members) {
            print_json_member(writer, elem);
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_ENUM:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "enum");
        print_annotations(writer, obj->annotations);
        writer.write_key("enumerators");
        writer.start_array();
        for (const auto& elem : obj->members) {
            writer.start_object();
            writer.write_key("name");
            writer.write_string(elem->name);
            if (elem->flags & OPT_ENUMERATED) {
                writer.write_key("value");
                writer.write_json(json_value(elem->value, obj));
            }
            if (get_annotation(elem, annotation_type_default_literal) != nullptr) {
                writer.write_key("default");
                writer.write(true);
            }
            writer.end_object();
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_BITSET:
        break;
    case N_BITMASK:
        writer.write_key(obj->name);
        writer.start_object();
        print_kind(writer, "bitmask");
        print_annotations(writer, obj->annotations);
        writer.write_key("flags");
        writer.start_array();
        for (const auto& elem : obj->members) {
            writer.start_object();
            writer.write_key("name");
            writer.write_string(elem->name);
            if (elem->flags & OPT_ENUMERATED) {
                writer.write_key("position");
                writer.write_json(json_value(elem->value, obj));
            }
            writer.end_object();
        }
        writer.end_array();
        writer.end_object();
        break;
    case N_ALIAS:
        writer.write_key(obj->name);
        writer.start_object();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "typedef");
        writer.write_key("type");
        writer.start_object();
        print_json_type(writer, obj->type, obj->super);
        writer.end_object();
        writer.end_object();
        break;
    case N_CONST:
        writer.write_key(obj->name);
        writer.start_object();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "const");
        writer.write_key("type");
        writer.start_object();
        print_json_type(writer, obj->type, obj->super);
        writer.end_object();
        writer.write_key("value");
        writer.write_json(json_value(obj->value, obj));
        writer.end_object();
        break;
    case N_ANNOTATION_DEF:
        writer.write_key(obj->name);
        writer.start_object();
        print_annotations(writer, obj->annotations);
        print_kind(writer, "annotation");
        if (obj->members) {
            writer.write_key("members");
            writer.start_object();
            for (const auto* elem : obj->members) {
                writer.write_key(elem->name);
                writer.start_object();
                print_json_type(writer, elem->type, obj);
                if (elem->value.kind() != UNDEF_KIND) {
                    writer.write_key("default");
                    writer.write_json(json_value(elem->value, obj));
                }
                writer.end_object();
            }
            writer.end_object();
        }
        writer.end_object();
        break;
    case N_ANNOTATION:
        writer.write_key(idl_scoped_name(obj, namespace_of(annotation_type_key)));
        if (obj->members && obj->members->next) {
            writer.start_object();
            for (const auto& elem : obj->members) {
                writer.write_key(elem->name);
                writer.write_json(json_value(elem->value, obj->type));
            }
            writer.end_object();
        } else if (obj->members) {
            writer.write_json(json_value(obj->members->value, obj->type));
        } else {
            writer.start_object();
            writer.end_object();
        }
        break;
    default:
        break;
    }
}

static void print_node(
    ic_cts::JsonWriter& writer,
    const numeric& value,
    const ptree* context,
    bool value_flag = false
) {
    switch (value.kind()) {
    case UNDEF_KIND:
        writer.write_null();
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
        writer.write_string(value.val.str());
        break;
    case CHAR_KIND:
        writer.write(static_cast<char>(value.val.c()));
        break;
    case PTREE_KIND: {
        if (value.val.node()->members) {
            if (base_type_of(value.val.node())->kind == N_STRUCT) {
                writer.start_object();
                for (auto p : value.val.node()->members) {
                    writer.write_key(p->name);
                    print_node(writer, p->value, context, value_flag);
                }
                writer.end_object();
            } else {
                bool was_pretty = writer.is_pretty();
                writer.set_pretty(false);
                writer.start_array();
                for (auto p : value.val.node()->members) {
                    print_node(writer, p->value, context, value_flag);
                }
                writer.end_array();
                writer.set_pretty(was_pretty);
            }
        } else {
            if (!value.val.node()->name.empty() && !value_flag) {
                writer.write_string(idl_scoped_name(value.val.node(), context));
            } else {
                print_node(writer, value.val.node()->value, context, value_flag);
            }
        }
    } break;
    }
}

static void print_node(ic_cts::JsonWriter& writer, const ptree* node) {
    if (!node) {
        writer.write_null();
        return;
    }

    writer.start_object();
    writer.write_key("kind");
    writer.write_string(node_kind_str(node->kind));
    if (!node->name.empty()) {
        writer.write_key("name");
        writer.write_string(node->name);
    }
    if (node->type) {
        writer.write_key("type");
        writer.write_string(idl_name(node->type));
        if (!node->type->bounds.empty()) {
            writer.write_key("bounds");
            writer.start_array();
            for (const auto& bound : node->type->bounds) {
                writer.write(integer_value(bound));
            }
            writer.end_array();
        }
    }
    if (node->element_type) {
        writer.write_key("element_type");
        writer.write_string(idl_name(node->element_type));
    }
    if (node->key_type) {
        writer.write_key("key_type");
        writer.write_string(idl_name(node->key_type));
    }
    if (node->annotations) {
        writer.write_key("annotations");
        writer.start_array();
        print_node(writer, node->annotations);
        writer.end_array();
    }
    if (node->members) {
        writer.write_key("members");
        writer.start_array();
        print_node(writer, node->members);
        writer.end_array();
    }
    if (!node->parents.empty()) {
        writer.write_key("parents");
        writer.start_array();
        for (auto parent : node->parents) {
            writer.write_string(idl_name(parent));
        }
        writer.end_array();
    }
    if (!node->getraises.empty()) {
        writer.write_key("getraises");
        writer.start_array();
        for (auto getraise : node->getraises) {
            writer.write_string(idl_name(getraise));
        }
        writer.end_array();
    }
    if (!node->setraises.empty()) {
        writer.write_key("setraises");
        writer.start_array();
        for (auto setraise : node->setraises) {
            writer.write_string(idl_name(setraise));
        }
        writer.end_array();
    }
    if (node->value.kind() != UNDEF_KIND) {
        writer.write_key("value");
        print_node(writer, node->value, node->super);
    }
    if (node->flags) {
        writer.write_key("flags");
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
        writer.write_string(str);
    }
    writer.end_object();
}

std::string intercom::cidl::json_value(const numeric& value, const ptree* context, int flags) {
    std::stringstream out;
    ic_cts::JsonWriter writer(out);
    print_node(writer, value, context, (flags & int(JsonValueFlags::FLAG_NUMERICAL_VALUE)) != 0);
    if (flags & int(JsonValueFlags::FLAG_ESCAPED)) {
        std::stringstream escape_out;
        ic_cts::JsonWriter escape_writer(escape_out);
        escape_writer.write_string(out.str());
        return escape_out.str();
    }
    return out.str();
}

std::string intercom::cidl::json_value(const ptree* obj) {
    std::stringstream out;
    ic_cts::JsonWriter writer(out);
    print_node(writer, obj);
    return out.str();
}

void intercom::cidl::code_gen_json(const parse_result* result, ic_list_t* list) {
    for (auto include : result->includes) {
        std::string file_name = trim_include_name(include->name, true);
        file_name += ".json";
        std::stringstream file;
        {
            ic_cts::JsonWriter writer(file, true);
            writer.start_object();
            for (const auto& obj : result->tree) {
                if (is_emit(obj, LANG_NONE) && obj->included_from == include) {
                    print_json(writer, obj);
                }
            }
            writer.end_object();
        }
        if (!file.str().empty()) {
            ic_push_source(list, file_name.c_str(), file.str().c_str());
        }
    }
}

void intercom::cidl::generate_json_type(std::ostream& stream, const ptree* tree) {
    ic_cts::JsonWriter writer(stream, true);
    writer.start_object();
    print_json(writer, tree);
    writer.end_object();
}

extern "C" {
void ic_codegen_json(const parse_result* result, ic_list_t* list) {
    intercom::cidl::code_gen_json(result, list);
}

void ic_codegen_json_schema(const parse_result* result, const char*) {
    intercom::cidl::code_gen_json_schema(result);
}
}
