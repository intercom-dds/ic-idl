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

#include <list>
#include <map>
#include <set>

#include "InterCOM/dds_curr_xtypes.h"
#include "InterCOM/dds_json.h"
#include "InterCOM/dds_xtypes_constants.h"
#include "InterCOM/intercom_dcps.h"
#include "cidl/constants.h"
#include "cidl/idl_parser.h"
#include "cidl/internal/commandline.h"
#include "cidl/internal/hdrs.h"
#include "cidl/internal/ptree_builder.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "cidl/type_definition.h"

using SchemaMap = std::map<std::string, std::string>;

void code_gen_json_schema_rec(const ptree* tree, SchemaMap& out) {
    if (!tree) {
        return;
    }
    intercom::dcps::TypeRepository* repo = intercom::dcps::TypeRepository::get_instance();
    for (auto node : tree) {
        if (is_emit(node, LANG_NONE) && (node->flags & OPT_DECLARATION) == 0) {
            if (node->kind == N_STRUCT || node->kind == N_UNION) {
                auto definition = get_type_definition(node);
                for (const auto& type_object : definition.type_objects) {
                    if (type_object.type_identifier._d() == intercom::dcps::xtypes::TK_NONE) {
                        continue;
                    }
                    repo->register_type(type_object.type_identifier, type_object.type_object);
                }
                std::string json_str;
                std::stringstream json_pretty;
                repo->lookup_type_schema(definition.type_info.complete.typeid_with_size.type_id, json_str);

                intercom::JsonValue value;
                intercom::JsonReader reader(json_str, 0);
                intercom::JsonWriter writer(json_pretty, intercom::SERIALIZER_PRETTY);
                intercom::dcps::cts::GenericUnmarshal(reader).io(value);
                intercom::dcps::cts::GenericMarshal(writer).io(value);

                out[definition.type_name] = json_pretty.str();
            }
            code_gen_json_schema_rec(node->members, out);
        }
    }
}

static std::string json_schema_filename(const std::string& name) {
    std::string filename = name;
    for (;;) {
        auto pos = filename.find("::");
        if (pos == std::string::npos) {
            break;
        }
        filename.replace(pos, 2, "/");
    }
    return filename + ".json";
}

void code_gen_json_schema(parse_result* result, std::list<File>* generated) {
    SchemaMap out;
    code_gen_json_schema_rec(result->tree, out);
    for (auto& schema : out) {
        auto filename = json_schema_filename(schema.first);
        auto& content = schema.second;
        if (CommandLineOption::list_only()) {
            std::cout << filename << std::endl;
        } else if (generated) {
            generated->emplace_back(filename, std::move(content));
        } else {
            std::string filepath;
            if (CommandLineOption::json_schema_target_directory()) {
                filepath = std::string(CommandLineOption::json_schema_target_directory()) + "/" + filename;
            } else {
                filepath = filename;
            }
            write_if_changed(filepath, content);
        }
    }
}

std::list<File> intercom::cidl::code_gen_json_schema(const intercom::cidl::Config& config,
                                                     struct parse_result* result) {
    std::lock_guard<std::mutex> guard(g_parse_mutex);
    CommandLineOption::get_instance() = config;
    std::list<File> generated;
    ::code_gen_json_schema(result, &generated);
    return generated;
}
