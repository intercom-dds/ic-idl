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

#include "InterCOM/JsonParser.h"
#include "cidl/idl_parser.h"
#include "cidl/internal/commandline.h"
#include "cidl/internal/hdrs.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

namespace {

void print_json(intercom::JsonWriter& writer, const ptree* obj);

void print_annotations(intercom::JsonWriter& writer, const ptree* obj) {
    if (obj) {
        writer.writeKey("annotations");
        writer.startObject();
        for (const auto& ann : obj) {
            print_json(writer, ann);
        }
        writer.endObject();
    }
}

void print_kind(intercom::JsonWriter& writer, const std::string& kind) {
    writer.writeKey("kind");
    writer.writeString(kind);
}

void print_json_type(intercom::JsonWriter& writer, const ptree* type, const ptree* context) {
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

void print_json_member(intercom::JsonWriter& writer, const ptree* member) {
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

void print_json(intercom::JsonWriter& writer, const ptree* obj) {
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
}  // namespace

void code_gen_json(parse_result* result, bool listonly) {
    for (auto include : result->includes) {
        std::string file_name = trim_include_name(include->name, true);
        file_name += ".json";
        if (listonly) {
            std::cout << file_name << std::endl;
            continue;
        }
        std::string filepath;
        if (CommandLineOption::json_target_directory()) {
            filepath = std::string(CommandLineOption::json_target_directory()) + "/" + file_name;
        } else {
            filepath = file_name;
        }
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

void generate_json_type(std::ostream& stream, const ptree* tree) {
    intercom::JsonWriter writer(stream, true);
    writer.startObject();
    print_json(writer, tree);
    writer.endObject();
}
