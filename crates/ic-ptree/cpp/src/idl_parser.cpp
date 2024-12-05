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

#include "cidl/idl_parser.h"

#include <cpplight/internal/PreProcessor.h>
#include <cpplight/internal/ProcessorUtilities.h>

#include <algorithm>
#include <cstdio>
#include <filesystem>
#include <iostream>
#include <memory>
#include <mutex>
#include <set>
#include <sstream>
#include <string>

#include "cidl/commandline.h"
#include "cidl/constants.h"
// #include "cidl/idl_rpc_gen.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "fmt/core.h"
#include "fmt/std.h"

extern "C" {
int scan_string(const char* str);
int scan_file(FILE* file);
int idlparse();
struct ptree g_top_level;

struct ptree* annotation_type_id;
struct ptree* annotation_type_autoid;
struct ptree* annotation_type_optional;
struct ptree* annotation_type_position;
struct ptree* annotation_type_value;
struct ptree* annotation_type_empty;
struct ptree* annotation_type_extensibility;
struct ptree* annotation_type_final;
struct ptree* annotation_type_mutable;
struct ptree* annotation_type_appendable;
struct ptree* annotation_type_shared;
struct ptree* annotation_type_key;
struct ptree* annotation_type_must_understand;
struct ptree* annotation_type_default;
struct ptree* annotation_type_default_literal;
struct ptree* annotation_type_range;
struct ptree* annotation_type_min;
struct ptree* annotation_type_max;
struct ptree* annotation_type_unit;
struct ptree* annotation_type_bit_bound;
struct ptree* annotation_type_external;
struct ptree* annotation_type_nested;
struct ptree* annotation_type_verbatim;
struct ptree* annotation_type_service;
struct ptree* annotation_type_topic;
struct ptree* annotation_type_dds_service;
struct ptree* annotation_type_dds_request_topic;
struct ptree* annotation_type_dds_reply_topic;
struct ptree* annotation_type_oneway;
struct ptree* annotation_type_ami;
struct ptree* annotation_type_bitset_old;
struct ptree* annotation_type_bit_bound_old;
struct ptree* annotation_type_must_understand_old;
struct ptree* annotation_type_minimum_type_check_old;
struct ptree* annotation_type_hashid;
struct ptree* annotation_type_default_nested;
struct ptree* annotation_type_ignore_literal_names;
struct ptree* annotation_type_try_construct;
struct ptree* annotation_type_non_serialized;
struct ptree* annotation_type_data_representation;
struct ptree* annotation_type_doc;
struct ptree* annotation_type_merge;
struct ptree* annotation_type_const;
struct ptree* annotation_type_static;
struct ptree* annotation_type_derive;
struct ptree* annotation_type_ext_rename;
struct ptree* annotation_type_ext_builder;
struct ptree* annotation_type_ext_doc;
struct ptree* annotation_type_ext_minimum_type_check;
struct ptree* annotation_type_ext_suppress;
struct ptree* annotation_type_ext_no_constructor;
struct ptree* annotation_type_ext_no_serializer;
struct ptree* annotation_type_ext_listener;
struct ptree* annotation_type_ext_length_bit_bound;
struct ptree* annotation_type_ext_value_offset;
struct ptree* annotation_type_ext_length_value_offset;
struct ptree* annotation_type_ext_repeat_count;
struct ptree* annotation_type_ext_vmf_xri;
struct ptree* annotation_type_ext_vmf_decimal;
struct ptree* annotation_type_ext_string_constants;
struct ptree* annotation_type_ext_jaus_presence_vector;
struct ptree* annotation_type_ext_jaus_integer;
struct ptree* annotation_type_ext_jaus_integer_function;
struct ptree* annotation_type_ext_protobuf_type;
struct ptree* annotation_type_jaus;

const char* node_kind_str(node_kind kind) {
    switch (kind) {
    case N_UNDEF:
        return "N_UNDEF";
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
    }
    return "";
}

const char* numeric_kind_str(numeric_kind kind) {
    switch (kind) {
    case UNDEF_KIND:
        return "UNDEF_KIND";
    case BOOLEAN_KIND:
        return "BOOLEAN_KIND";
    case INT8_KIND:
        return "INT8_KIND";
    case OCTET_KIND:
        return "OCTET_KIND";
    case SHORT_KIND:
        return "SHORT_KIND";
    case USHORT_KIND:
        return "USHORT_KIND";
    case LONG_KIND:
        return "LONG_KIND";
    case ULONG_KIND:
        return "ULONG_KIND";
    case LONGLONG_KIND:
        return "LONGLONG_KIND";
    case ULONGLONG_KIND:
        return "ULONGLONG_KIND";
    case FLOAT_KIND:
        return "FLOAT_KIND";
    case DOUBLE_KIND:
        return "DOUBLE_KIND";
    case CHAR_KIND:
        return "CHAR_KIND";
    case STRING_KIND:
        return "STRING_KIND";
    case PTREE_KIND:
        return "PTREE_KIND";
    }
    return "";
}
}

using namespace intercom::cidl;

static std::map<std::string, ptree**> initialize_builtin_annotation_map() {
    std::map<std::string, ptree**> res;

    res["intercom::annotations::id"] = &annotation_type_id;
    res["intercom::annotations::autoid"] = &annotation_type_autoid;
    res["intercom::annotations::optional"] = &annotation_type_optional;
    res["intercom::annotations::position"] = &annotation_type_position;
    res["intercom::annotations::value"] = &annotation_type_value;
    res["intercom::annotations::empty"] = &annotation_type_empty;
    res["intercom::annotations::extensibility"] = &annotation_type_extensibility;
    res["intercom::annotations::final"] = &annotation_type_final;
    res["intercom::annotations::mutable"] = &annotation_type_mutable;
    res["intercom::annotations::appendable"] = &annotation_type_appendable;
    res["intercom::annotations::shared"] = &annotation_type_shared;
    res["intercom::annotations::key"] = &annotation_type_key;
    res["intercom::annotations::must_understand"] = &annotation_type_must_understand;
    res["intercom::annotations::default"] = &annotation_type_default;
    res["intercom::annotations::default_literal"] = &annotation_type_default_literal;
    res["intercom::annotations::range"] = &annotation_type_range;
    res["intercom::annotations::min"] = &annotation_type_min;
    res["intercom::annotations::max"] = &annotation_type_max;
    res["intercom::annotations::unit"] = &annotation_type_unit;
    res["intercom::annotations::bit_bound"] = &annotation_type_bit_bound;
    res["intercom::annotations::external"] = &annotation_type_external;
    res["intercom::annotations::nested"] = &annotation_type_nested;
    res["intercom::annotations::verbatim"] = &annotation_type_verbatim;
    res["intercom::annotations::service"] = &annotation_type_service;
    res["intercom::annotations::topic"] = &annotation_type_topic;
    res["intercom::annotations::DDSService"] = &annotation_type_dds_service;
    res["intercom::annotations::DDSRequestTopic"] = &annotation_type_dds_request_topic;
    res["intercom::annotations::DDSReplyTopic"] = &annotation_type_dds_reply_topic;
    res["intercom::annotations::oneway"] = &annotation_type_oneway;
    res["intercom::annotations::ami"] = &annotation_type_ami;
    res["intercom::annotations::bitset"] = &annotation_type_bitset_old;
    res["intercom::annotations::bitbound"] = &annotation_type_bit_bound_old;
    res["intercom::annotations::mustunderstand"] = &annotation_type_must_understand_old;
    res["intercom::annotations::hashid"] = &annotation_type_hashid;
    res["intercom::annotations::doc"] = &annotation_type_doc;
    res["intercom::annotations::merge"] = &annotation_type_merge;
    res["intercom::annotations::default_nested"] = &annotation_type_default_nested;
    res["intercom::annotations::ignore_literal_names"] = &annotation_type_ignore_literal_names;
    res["intercom::annotations::try_construct"] = &annotation_type_try_construct;
    res["intercom::annotations::non_serialized"] = &annotation_type_non_serialized;
    res["intercom::annotations::data_representation"] = &annotation_type_data_representation;
    res["intercom::annotations::const"] = &annotation_type_const;
    res["intercom::annotations::static"] = &annotation_type_static;
    res["intercom::annotations::derive"] = &annotation_type_derive;
    res["intercom::annotations::ext::rename"] = &annotation_type_ext_rename;
    res["intercom::annotations::ext::builder"] = &annotation_type_ext_builder;
    res["intercom::annotations::ext::minimum_type_check"] = &annotation_type_minimum_type_check_old;
    res["intercom::annotations::ext::doc"] = &annotation_type_ext_doc;
    res["intercom::annotations::ext::minimum_type_check"] = &annotation_type_ext_minimum_type_check;
    res["intercom::annotations::ext::suppress"] = &annotation_type_ext_suppress;
    res["intercom::annotations::ext::no_constructor"] = &annotation_type_ext_no_constructor;
    res["intercom::annotations::ext::no_serializer"] = &annotation_type_ext_no_serializer;
    res["intercom::annotations::ext::listener"] = &annotation_type_ext_listener;
    res["intercom::annotations::ext::length_bit_bound"] = &annotation_type_ext_length_bit_bound;
    res["intercom::annotations::ext::value_offset"] = &annotation_type_ext_value_offset;
    res["intercom::annotations::ext::length_value_offset"] =
        &annotation_type_ext_length_value_offset;
    res["intercom::annotations::ext::repeat_count"] = &annotation_type_ext_repeat_count;
    res["intercom::annotations::ext::vmf_xri"] = &annotation_type_ext_vmf_xri;
    res["intercom::annotations::ext::vmf_decimal"] = &annotation_type_ext_vmf_decimal;
    res["intercom::annotations::ext::string_constants"] = &annotation_type_ext_string_constants;
    res["intercom::annotations::ext::jaus_presence_vector"] =
        &annotation_type_ext_jaus_presence_vector;
    res["intercom::annotations::ext::jaus_integer"] = &annotation_type_ext_jaus_integer;
    res["intercom::annotations::ext::jaus_integer_function"] =
        &annotation_type_ext_jaus_integer_function;
    res["intercom::annotations::ext::protobuf_type"] = &annotation_type_ext_protobuf_type;
    res["intercom::annotations::jaus"] = &annotation_type_jaus;

    return res;
}

std::map<std::string, ptree**> g_builtin_annotation_map = initialize_builtin_annotation_map();

namespace {
const char* g_builtin_annotations =
    "module intercom {\n"
    "module annotations {\n"
    "@annotation id {\n"
    "   unsigned long value;\n"
    "};\n"
    "@annotation autoid {\n"
    "   enum AutoidKind {\n"
    "      SEQUENTIAL,\n"
    "      HASH\n"
    "   };\n"
    "   AutoidKind value default HASH;\n"
    "};\n"
    "@annotation optional {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation position {\n"
    "   unsigned short value;\n"
    "};\n"
    "@annotation value {\n"
    "   any value;\n"
    "};\n"
    "@annotation empty {\n"
    "};\n"
    "@annotation extensibility {\n"
    "   enum ExtensibilityKind {\n"
    "      FINAL,\n"
    "      APPENDABLE,\n"
    "      MUTABLE,\n"
    "      EXTENSIBLE = APPENDABLE,\n"  // non standard alias
    "      FINAL_EXTENSIBILITY = FINAL,\n"
    "      APPENDABLE_EXTENSIBILITY = APPENDABLE,\n"
    "      MUTABLE_EXTENSIBILITY = MUTABLE,\n"
    "      EXTENSIBLE_EXTENSIBILITY = APPENDABLE\n"
    "   };\n"
    "   ExtensibilityKind value;\n"
    "};\n"
    "@annotation final {\n"
    "};\n"
    "@annotation mutable {\n"
    "};\n"
    "@annotation appendable {\n"
    "};\n"
    "@annotation shared {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation key {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation must_understand {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation _default {\n"
    "   any value;\n"
    "};\n"
    "@annotation default_literal {\n"
    "};\n"
    "@annotation range {\n"
    "   any min;\n"
    "   any max;\n"
    "};\n"
    "@annotation min {\n"
    "   any value;\n"
    "};\n"
    "@annotation max {\n"
    "   any value;\n"
    "};\n"
    "@annotation unit {\n"
    "   string value;\n"
    "};\n"
    "@annotation bit_bound {\n"
    "   unsigned short value;\n"
    "};\n"
    "@annotation external {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation nested {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation verbatim {\n"
    "   enum PlacementKind {\n"
    "      BEGIN_FILE,\n"
    "      BEFORE_DECLARATION,\n"
    "      BEGIN_DECLARATION,\n"
    "      END_DECLARATION,\n"
    "      AFTER_DECLARATION,\n"
    "      END_FILE\n"
    "   };\n"
    "   string language default \"*\";\n"
    "   PlacementKind placement default BEFORE_DECLARATION;\n"
    "   string text;\n"
    "};\n"
    "@annotation service {\n"
    "   string platform default \"*\";\n"
    "   string name default \"\";\n"
    "   string request_topic default \"\";\n"
    "   string reply_topic default \"\";\n"
    "};\n"
    "@annotation topic {\n"
    "   string name default \"\";\n"
    "   string class default \"\";\n"
    "   string namespace default \"\";\n"
    "   string qosprofile default \"\";\n"
    "   string partition default \"\";\n"
    "   long long domain default -1;\n"
    "   string udpmessage default \"\";\n"
    "   string platform default \"*\";\n"
    "};\n"
    "@annotation qoslibrary {\n"
    "   string name;\n"
    "};\n"
    "@annotation DDSService {\n"
    "   string name default \"\";\n"
    "};\n"
    "@annotation DDSRequestTopic {\n"
    "   string name;\n"
    "};\n"
    "@annotation DDSReplyTopic {\n"
    "   string name;\n"
    "};\n"
    "@annotation _oneway {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation ami {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation _bitset {\n"  // old annotation for bitmask
    "};\n"
    "@annotation bitbound {\n"  // old name for bit_bound
    "   unsigned short value;\n"
    "};\n"
    "@annotation mustunderstand {\n"  // old name for must_understand
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation minimumtypecheck {\n"  // old name for minimum_type_check
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation hashid {\n"
    "   string value default \"\"\n;"
    "};\n"
    "@annotation default_nested {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation ignore_literal_names {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation try_construct {\n"
    "   enum TryConstructFailAction {\n"
    "      USE_DEFAULT, DISCARD, TRIM\n"
    "   };\n"
    "   TryConstructFailAction value default USE_DEFAULT;\n"
    "};\n"
    "@annotation non_serialized {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation data_representation {\n"
    "   bitmask DataRepresentationMask {\n"
    "      XCDR1, XML, XCDR2\n"
    "   };\n"
    "   DataRepresentationMask allowed_kinds;\n"
    "};\n"
    "@annotation jaus {\n"
    "   string id default \"\";\n"
    "   string name default \"\";\n"
    "   string version default \"\";\n"
    "   string assumptions default \"\";\n"
    "   string inherits_from default \"\";\n"
    "};\n"
    "@annotation doc {\n"
    "   enum PlacementKind {\n"
    "      BEGIN_FILE,\n"
    "      BEFORE_DECLARATION,\n"
    "      BEGIN_DECLARATION,\n"
    "      END_DECLARATION,\n"
    "      AFTER_DECLARATION,\n"
    "      END_FILE\n"
    "   };\n"
    "   string text;\n"
    "   PlacementKind placement default BEFORE_DECLARATION;\n"
    "};\n"
    "@annotation merge {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation static {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation _const {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation derive {\n"
    "   string value;\n"
    "};\n"
    "module ext {\n"
    "@annotation doc {\n"
    "   enum PlacementKind {\n"
    "      BEGIN_FILE,\n"
    "      BEFORE_DECLARATION,\n"
    "      BEGIN_DECLARATION,\n"
    "      END_DECLARATION,\n"
    "      AFTER_DECLARATION,\n"
    "      END_FILE\n"
    "   };\n"
    "   string text;\n"
    "   PlacementKind placement default BEFORE_DECLARATION;\n"
    "};\n"
    "@annotation minimum_type_check {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation suppress {\n"
    "   boolean value default TRUE;\n"
    "   string language default \"*\";\n"
    "};\n"
    "@annotation no_constructor {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation no_serializer {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation listener {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation length_bit_bound {\n"
    "   unsigned short value;\n"
    "};\n"
    "@annotation value_offset {\n"
    "   long value;\n"
    "};\n"
    "@annotation length_value_offset {\n"
    "   long value;\n"
    "};\n"
    "@annotation repeat_count {\n"
    "   any value;\n"
    "};\n"
    "@annotation vmf_xri {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation vmf_decimal {\n"
    "   long chars default 0;\n"
    "   long decimal_bits;\n"
    "};\n"
    "@annotation jaus_presence_vector {\n"
    "   long value;\n"
    "};\n"
    "@annotation jaus_integer {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation jaus_integer_function {\n"
    "   boolean value default TRUE;\n"
    "};\n"
    "@annotation rename {\n"
    "   string name;\n"
    "};\n"
    "@annotation builder {\n"
    "   string name default \"\";\n"
    "};\n"
    "@annotation string_constants {\n"
    "   boolean value default TRUE;\n"
    "   string namespace;\n"
    "};\n"
    "@annotation protobuf_type {\n"
    "   string name;\n"
    "};\n"
    "};\n"
    "};\n"
    "};\n";

const char* g_rpc_types =
    "@ext::suppress module DDS {\n"
    "   typedef octet GuidPrefix_t[12];\n"
    "   @final @nested struct EntityId_t {\n"
    "      octet entity_key[3];\n"
    "      octet entity_kind;\n"
    "   };\n"
    "   @final @nested struct GUID_t {\n"
    "      GuidPrefix_t guid_prefix;\n"
    "      EntityId_t   entity_id;\n"
    "   };\n"
    "   @final @nested struct SequenceNumber_t {\n"
    "      long          high;\n"
    "      unsigned long low;\n"
    "   };\n"
    "   @final @nested struct SampleIdentity_t {\n"
    "      GUID_t      writer_guid;\n"
    "      SequenceNumber_t sequence_number;\n"
    "   };\n"
    "   module RPC {\n"
    "      typedef octet UnknownOperation;\n"
    "      typedef octet UnknownException;\n"
    "      typedef octet UnusedMember;\n"
    "      typedef long RemoteExceptionCode_t;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_OK = 0;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_UNSUPPORTED = 1;"
    "      const RemoteExceptionCode_t REMOTE_EX_INVALID_ARGUMENT = 2;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_OUT_OF_RESOURCES = 3;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_UNKNOWN_OPERATION = 4;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_UNKNOWN_EXCEPTION = 5;\n"
    "      const RemoteExceptionCode_t REMOTE_EX_SERVICE_LOST = 6;\n"
    "      typedef string<255> InstanceName;\n"
    "      @final\n"
    "      struct RequestHeader {\n"
    "         SampleIdentity_t request_id;\n"
    "         InstanceName instance_name;\n"
    "      };\n"
    "      @final\n"
    "      struct ReplyHeader {\n"
    "         SampleIdentity_t related_request_id;\n"
    "         RemoteExceptionCode_t remote_ex;\n"
    "      };\n"
    "   };\n"
    "};\n";

intercom::cidl::parse_result g_parse_result;

std::stringstream& msgout() {
    static auto s_messages = std::make_unique<std::stringstream>();
    return *s_messages;
}
}  // namespace

void do_parse_error(const char* msg, const char* file_name, int line_number) {
    msgout() << "error: " << msg << " near line "
             << line_number - 1;  // -1 is statistically more accurate
    if (file_name) {
        msgout() << " in " << file_name;
    }
    msgout() << std::endl;
    ++g_parse_result.error_count;
}

void do_parse_warning(const char* msg, const char* file_name, int line_number) {
    msgout() << "warning: " << msg << " near line "
             << line_number - 1;  // -1 is statistically more accurate
    if (file_name) {
        msgout() << " in " << file_name;
    }
    msgout() << std::endl;
    ++g_parse_result.warning_count;
}

void parse_alert(
    const char* msg,
    const char* file_name,
    int line_number,
    CommandLineOption::WarningType warning_type
) {
    if (!CommandLineOption::suppress_error(warning_type) &&
        !CommandLineOption::suppress_warning(warning_type)) {
        do_parse_error(msg, file_name, line_number);
    } else if (!CommandLineOption::suppress_warning(warning_type)) {
        do_parse_warning(msg, file_name, line_number);
    }
}

void parse_error(const char* msg, const char* file_name, int line_number) {
    parse_alert(msg, file_name, line_number, CommandLineOption::WARNING_UNCATEGORIZED_ERROR);
}

void parse_pedantic(const ptree* node, const char* message) {
    ALERT(CommandLineOption::WARNING_PEDANTIC).context(node) << message;
    if (node) {
        msg << " on node " << node;
    }
}

void parse_warning(const char* msg, const char* file_name, int line_number) {
    parse_alert(msg, file_name, line_number, CommandLineOption::WARNING_UNCATEGORIZED_WARNING);
}

int parser_has_error() {
    return g_parse_result.error_count > 0;
}

static void reset_top_level() {
    g_top_level = ptree();
}

static std::shared_ptr<parser> g_rpc_initial_state;

static void init_parser_state(const std::shared_ptr<parser>& state) {
    static auto s_initial_state = []() -> std::shared_ptr<parser> {
        const CommandLineOption::ScopeDefaultWarnings scp;
        auto initial = std::make_shared<parser>();
        g_state = initial;
        current_input_file = "";
        g_parse_result = parse_result();
        msgout().str("");
        msgout().clear();
        reset_top_level();
        scan_string(g_builtin_annotations);
        clear_namespace_nodes();
        // Everything created up until this point is builtin types
        for (const auto& node : g_state->allocated_nodes) {
            node->flags |= OPT_BUILTIN;
        }

        g_rpc_initial_state = std::make_shared<parser>(*initial);
        g_state = g_rpc_initial_state;
        reset_top_level();
        scan_string(g_rpc_types);

        if (g_parse_result.error_count > 0) {
            std::cerr
                << "[CRITICAL] FAILED to parse builtin idl files\nbuiltin idl parse errors: {\n"
                << msgout().str() << '}' << std::endl;
        }

        return initial;
    }();
    *state = *s_initial_state;
    g_state = state;
    current_input_file = "";
    g_parse_result = parse_result();
    msgout().str("");
    msgout().clear();
    reset_top_level();
}

static void add_rpc_types_to_global_state() {
    g_state->type_map.insert(
        g_rpc_initial_state->type_map.begin(), g_rpc_initial_state->type_map.end()
    );
}

void update_incomplete_type(struct ptree* node, struct ptree*& type) {
    if (type) {
        if (type->flags & OPT_DECLARATION) {
            if (type->type) {
                type = type->type;
                node->flags |= OPT_CIRCULAR;
            } else {
                std::stringstream stream;
                stream << "type \"" << type->name << "\" declared only (as \"" << node->name
                       << "\" on line " << node->pos.line << ")";
                idlerror(stream.str().c_str());
            }
        }
        update_incomplete_type(node, type->type);
        update_incomplete_type(node, type->element_type);
        update_incomplete_type(node, type->key_type);
    }
}

void resolve_incomplete_types(struct ptree* node) {
    while (node) {
        update_incomplete_type(node, node->type);
        resolve_incomplete_types(node->members);
        node = node->next;
    }
}

ptree* prune_annotations(struct ptree* node, struct ptree* super = nullptr) {
    if (!node) {
        return nullptr;
    }
    if (node->kind == N_ANNOTATION &&
        (node->type != annotation_type_doc || super == nullptr || super->kind != N_MODULE)) {
        return node->next;
    }
    node->next = prune_annotations(node->next, super);
    node->members = prune_annotations(node->members, node);
    return node;
}

void generate_code(struct ptree* node) {
    while (node) {
        current_input_file = get_symbol(node->file_name.c_str());
        g_state->include_context.push_back(node->included_from);
        // node->generated = append_node(node->generated, generate_rpc_structs(node));
        push_context(node);
        generate_code(node->members);
        g_state->include_context.pop_back();
        pop_context();
        node = node->next;
    }
}

static void tree_modules_add(const ptree* tree, std::set<std::string>& modules) {
    while (tree) {
        if (is_emit(tree, LANG_NONE)) {
            modules.insert(module_name(tree));
            tree_modules_add(tree->members, modules);
        }
        tree = tree->next;
    }
}

static void tree_modules_prune(const ptree* tree, std::set<std::string>& modules) {
    while (tree) {
        // Exclude modules that are not emit
        if (!is_emit(tree, LANG_NONE)) {
            modules.erase(module_name(tree));
        }
        // ...but keep modules that directly contain emittable modules
        if (is_emit(tree, LANG_NONE) && tree->kind != N_MODULE && tree->scope &&
            tree->scope->kind == N_MODULE) {
            modules.insert(module_name(tree));
        }
        tree_modules_prune(tree->members, modules);
        tree = tree->next;
    }
}

static void tree_modules(const ptree* tree, std::set<std::string>& modules) {
    // Add all modules that are OPT_EMIT to set
    tree_modules_add(tree, modules);
    // Remove module names that are not OPT_EMIT somewhere in the tree.
    // This is done to avoid emitting parent modules in Ada when the
    // --no-header-follow option is used and the contents of the parent
    // module is in an included file while a sub-module is in the main file
    tree_modules_prune(tree, modules);
}

static void tree_includes(const ptree* tree, std::set<const ptree*>& includes) {
    while (tree) {
        if ((tree->flags & OPT_EMIT_CODE) != 0) {
            if (tree->included_from != nullptr) {
                includes.insert(tree->included_from);
            }
            tree_includes(tree->members, includes);
        }
        tree = tree->next;
    }
}

void merge_structs(ptree* node) {
    while (node) {
        ptree* base = base_type_of(node);
        if (base->kind == N_STRUCT && !base->original_members /* only merge once */) {
            if (std::any_of(begin(base->members), end(base->members), [](const ptree* m) {
                    return is_merged(m) != 0;
                })) {
                base->members = merge_members(base, base->members);
            }
        }
        merge_structs(node->members);
        merge_structs(node->generated);
        node = node->next;
    }
}

void register_node_in_scope(ptree* node, ptree* scp) {
    std::swap(node->super, scp);
    register_node(node);
    std::swap(node->super, scp);
}

/// \brief registers inherited and merged members
/// \details register_node(..) is usually called during ptree construction, but for forward
/// declarations it has to happen after
void register_inherited_nodes(ptree* node) {
    if (node->type || (node->kind != N_STRUCT && node->kind != N_INTERFACE)) {
        return;
    }
    // inheritance
    for (ptree* parent = node; !parent->parents.empty();) {
        parent = base_type_of(parent->parents.front());
        for (ptree* elem : parent->members) {
            register_node_in_scope(elem, node);
        }
    }
    // merge
    for (MergeTrace& trace : get_merge_traces(node)) {
        if (trace.size() > 1U) {
            register_node_in_scope(const_cast<ptree*>(trace.back()), node);
        }
    }
}

static struct parse_result get_parse_result() {
    resolve_incomplete_types(g_top_level.next);
    g_top_level.next = prune_annotations(g_top_level.next);
    format_doxy_comments(g_top_level.next);
    if (!try_lookup_node("DDS::SampleIdentity_t", ANY_KIND)) {
        add_rpc_types_to_global_state();
    }
    generate_code(g_top_level.next);
    merge_structs(g_top_level.next);
    for (std::shared_ptr<ptree>& node : g_state->allocated_nodes) {
        register_inherited_nodes(node.get());
    }
    validate_tree(g_top_level.next);
    g_parse_result.tree = g_top_level.next;
    g_parse_result.msg = msgout().str();

    if (g_top_level.next) {
        g_top_level.next->state->numeric_map.clear();
    }

    tree_modules(g_top_level.next, g_parse_result.modules);
    tree_includes(g_top_level.next, g_parse_result.includes);

    return g_parse_result;
}

static parse_result run_parser(const char* input) {
    scan_string(input);
    return get_parse_result();
}

static parse_result run_parser_on_file(FILE* input) {
    scan_file(input);
    return get_parse_result();
}

namespace intercom::cidl {

namespace {
void suppress_content_from_includes(parse_result& result, const FileList& input_files) {
    std::set<std::string> input_file_set;
    for (auto& file : input_files) {
        input_file_set.insert(std::filesystem::canonical(file.first));
    }
    std::function<void(ptree*)> filter = [&](ptree* tree) {
        if (!tree) {
            return;
        }
        for (auto node : tree) {
            if (input_file_set.find(node->file_name) == input_file_set.end()) {
                node->flags &= ~OPT_EMIT_CODE;
            }
            filter(node->members);
        }
    };
    filter(const_cast<ptree*>(result.tree));
    result.includes.clear();
    tree_includes(result.tree, result.includes);
    tree_modules(result.tree, result.modules);
}

void update_include_paths(parse_result& result, const FileList& input_files) {
    std::map<std::string, std::string> path_map;
    for (auto& file : input_files) {
        path_map.emplace(std::filesystem::canonical(file.first), file.second);
    }
    std::function<void(ptree*)> filter = [&](ptree* tree) {
        if (!tree) {
            return;
        }
        for (auto node : tree) {
            auto it = path_map.find(node->file_name);
            if (it != path_map.end()) {
                node->included_from->name = it->second;
            }
            filter(node->members);
        }
    };
    filter(const_cast<ptree*>(result.tree));
}

void validate_consistent_types(
    const ptree* tree,
    std::map<std::string, const ptree*>& type_map,
    parse_result& result
) {
    auto validate_type = [&](const ptree* type) {
        if (type && (type->flags & OPT_DECLARATION) == 0) {
            auto name = lc_scoped_name(type);
            auto it = type_map.find(name);
            if (it != type_map.end() && it->second != type) {
                std::stringstream err;
                err << "Inconsistent type for node " << name << " of kind " << type->kind;
                if (!result.msg.empty()) {
                    result.msg += "\n";
                }
                result.msg += err.str();
                result.error_count++;
            } else {
                type_map[name] = type;
            }
        }
    };
    auto validate_node = [&](const ptree* node) {
        if (node->kind == N_CONST && (node->flags & OPT_CONST_VALUE) == 0) {
            validate_type(node);
        }
        if (node->kind == N_ALIAS || node->kind == N_STRUCT || node->kind == N_UNION ||
            node->kind == N_VALUETYPE || node->kind == N_INTERFACE) {
            validate_type(node);
        }
        validate_type(node->type);
        validate_type(node->element_type);
        validate_type(node->key_type);
    };
    for (auto node : tree) {
        validate_node(node);
        validate_consistent_types(node->members, type_map, result);
        if (node->value->_d() == PTREE_KIND) {
            validate_node(node->value->node());
            validate_consistent_types(node->value->node()->members, type_map, result);
        }
    }
}

// Update type pointers in tree to point into the main
// tree structure if they are defined there
void update_ptree_types_after_merge(parse_result& result) {
    // Update type map with pointers from merged tree
    std::function<void(ptree*)> update_type_map = [&](ptree* tree) {
        for (auto node : tree) {
            auto name = lc_scoped_name(node);
            auto& map = (node->flags & OPT_DECLARATION) != 0 ? result.state->type_dcl_map
                                                             : result.state->type_map;
            auto it = map.find(name);
            if (it != map.end() && it->second->kind == node->kind) {
                it->second = node;
            }
            update_type_map(node->members);
            update_type_map(node->generated);
        }
    };

    // Return node from type map if it exists, otherwise return input argument
    std::function<ptree*(ptree*)> lookup_type = [&](ptree* node) {
        if (node) {
            auto it = result.state->type_map.find(lc_scoped_name(node));
            if (it != result.state->type_map.end()) {
                return it->second;
            }
        }
        return node;
    };

    // Update tree with type pointers from type map if they exist
    std::set<const ptree*> seen;
    std::function<void(ptree*)> update_types = [&](ptree* tree) {
        if (seen.find(tree) != seen.end()) {
            return;
        }
        for (auto node : tree) {
            seen.insert(node);
            node->type = lookup_type(node->type);
            node->key_type = lookup_type(node->key_type);
            node->element_type = lookup_type(node->element_type);
            for (auto& parent : node->parents) {
                parent = lookup_type(parent);
                update_types(parent);
            }
            for (auto& getraise : node->getraises) {
                getraise = lookup_type(getraise);
                update_types(getraise);
            }
            for (auto& setraise : node->setraises) {
                setraise = lookup_type(setraise);
                update_types(setraise);
            }
            for (auto& bound : node->bounds) {
                if (bound->_d() == PTREE_KIND) {
                    bound.val.node(lookup_type(const_cast<ptree*>(bound.val.node())));
                }
            }
            update_types(node->type);
            update_types(node->key_type);
            update_types(node->element_type);

            update_types(node->members);
            update_types(node->generated);
            update_types(node->original_members);
            update_types(node->discriminator);
            update_types(node->annotations);
            if (node->value->_d() == PTREE_KIND) {
                auto updated = const_cast<ptree*>(node->value->node());
                if (updated->kind != N_CONST || (updated->flags & OPT_CONST_VALUE) == 0) {
                    updated = lookup_type(updated);
                }
                update_types(updated);
                node->value.val.node(updated);
            }
            std::for_each(node->parents.begin(), node->parents.end(), update_types);
            std::for_each(node->getraises.begin(), node->getraises.end(), update_types);
            std::for_each(node->setraises.begin(), node->setraises.end(), update_types);
        }
    };

    auto tree = const_cast<ptree*>(result.tree);
    update_type_map(tree);
    update_types(tree);

    // Check that there is only a single definition for each type in the tree.
    // Errors here should never happen and could be asserted, but add it to user error messages
    // to help debug any follow-on errors in the backend in release builds.
    std::map<std::string, const ptree*> type_map;
    validate_consistent_types(tree, type_map, result);
}
}  // namespace

parse_result merge_results(std::vector<parse_result>& to_merge) {
    std::lock_guard<std::mutex> guard(g_parse_mutex);
    parse_result out;
    out.state = std::make_shared<parser>();
    ptree* new_tree = nullptr;

    g_state = out.state;

    std::map<std::string, const ptree*> seen_includes;

    // Filter nodes so that a file is only defined once
    std::function<ptree*(ptree*)> filter_includes = [&](ptree* node) -> ptree* {
        if (!node) {
            return nullptr;
        }
        ptree* prev = nullptr;
        for (auto n : node) {
            if (seen_includes.find(n->file_name) == seen_includes.end()) {
                seen_includes[n->file_name] = n->included_from;
            }
            if (seen_includes[n->file_name] != n->included_from) {
                if (prev) {
                    prev->next = n->next;
                } else {
                    node = n->next;
                }
            } else {
                prev = n;
            }
        }
        for (auto n : node) {
            n->members = filter_includes(n->members);
        }
        return node;
    };

    // Modify ptree nodes to point to the new parser state struct and populate
    // type map
    std::function<void(ptree*)> update_state_ptr = [&](ptree* tree) {
        for (auto node : tree) {
            node->state = out.state.get();
            update_state_ptr(node->members);
            update_state_ptr(node->generated);
            update_state_ptr(node->original_members);
            update_state_ptr(node->annotations);
            update_state_ptr(node->included_from);
        }
    };

    for (auto& to_merge_result : to_merge) {
        if (to_merge_result.tree) {
            // Take ownership of nodes
            out.state->allocated_nodes.insert(
                out.state->allocated_nodes.end(),
                to_merge_result.state->allocated_nodes.begin(),
                to_merge_result.state->allocated_nodes.end()
            );
            out.state->allocated_decl.insert(
                out.state->allocated_decl.end(),
                to_merge_result.state->allocated_decl.begin(),
                to_merge_result.state->allocated_decl.end()
            );
            out.state->type_map.insert(
                to_merge_result.state->type_map.begin(), to_merge_result.state->type_map.end()
            );
            out.state->type_dcl_map.insert(
                to_merge_result.state->type_dcl_map.begin(),
                to_merge_result.state->type_dcl_map.end()
            );

            // Update ptree and add it to the merged tree
            auto to_merge_tree = const_cast<ptree*>(to_merge_result.tree);
            to_merge_tree = filter_includes(to_merge_tree);
            update_state_ptr(to_merge_tree);
            new_tree = append_node(to_merge_tree, new_tree);
        }

        // Merge errors
        out.error_count += to_merge_result.error_count;
        out.warning_count += to_merge_result.warning_count;
        if (!to_merge_result.msg.empty()) {
            if (!out.msg.empty()) {
                out.msg += "\n";
            }
            out.msg += to_merge_result.msg;
        }
    }
    out.tree = new_tree;
    tree_modules(out.tree, out.modules);
    tree_includes(out.tree, out.includes);

    to_merge.clear();
    g_state.reset();
    current_input_file = "";
    return out;
}

bool run_preprocessor(
    const std::string& a_file_name,
    const std::vector<std::string>& a_parameters,
    std::ostream& a_out,
    std::string& a_error
) {
    // Add global parameters and the input file to local parameters
    std::stringstream err_stream;
    ProcessorUtilities::StringVector pp_localparams = a_parameters;
    pp_localparams.emplace_back("-C");
    pp_localparams.push_back((a_file_name));
    PreProcessor p;
    p.setOutStream(a_out);
    p.setErrorStream(err_stream);
    bool res = p.configure(pp_localparams) && p.run();
    if (!res) {
        a_error = err_stream.str();
    }
    return res;
}

bool run_preprocessor(const std::string& a_file_name, std::ostream& a_out, std::string& a_error) {
    return run_preprocessor(a_file_name, std::vector<std::string>(), a_out, a_error);
}

static void collect_files_from_directory(
    const std::filesystem::path& a_base,
    const std::filesystem::path& a_dir,
    FileList& a_files
) {
    for (const auto& f : std::filesystem::recursive_directory_iterator(a_dir)) {
        const auto& path = f.path();
        if (!f.is_directory() && (path.extension() == ".idl" || path.extension() == ".IDL")) {
            a_files.emplace_back(f, std::filesystem::relative(f, a_base));
        }
    }
}

parse_result run_parser(
    const std::vector<std::string>& input_files,
    const std::vector<std::string>& pp_options,
    uint32_t flags
) {
    std::vector<parse_result> parsed_trees;
    FileList expanded_files;

    for (auto file : input_files) {
        if (!std::filesystem::exists(file)) {
            parse_result err;
            err.error_count++;
            err.msg = fmt::format("failed to open file \"{}\"\n", file);
            return err;
        }

        if (std::filesystem::is_directory(file)) {
            collect_files_from_directory(file, file, expanded_files);
        } else {
            expanded_files.emplace_back(file, std::filesystem::path(file).filename());
        }
    }

    for (auto file : expanded_files) {
        bool json_input = file.first.extension() == ".json";
        bool xml_input = file.first.extension() == ".xml";
        std::stringstream ostream;

        if (!json_input && !xml_input && !CommandLineOption::preprocessor_skip()) {
            std::string pp_err;
            bool pp_success = intercom::cidl::run_preprocessor(
                std::filesystem::absolute(file.first).string(), pp_options, ostream, pp_err
            );
            if (!pp_success) {
                parse_result err;
                err.error_count++;
                err.msg =
                    fmt::format("preprocessing stage on \"{}\" failed: {}\n", file.first, pp_err);
                return err;
            }
        } else {
            std::ifstream fs(file.first);
            if (fs) {
                if (!json_input && !xml_input) {
                    ostream << "#included_as \"" << file.first << "\"" << std::endl;
                    ostream << "# 1 \"" << std::filesystem::canonical(file.first) << "\""
                            << std::endl;
                }
                ostream << fs.rdbuf();
                fs.close();
            } else {
                parse_result err;
                err.error_count++;
                err.msg = fmt::format("failed to open file \"{}\"\n", file.first);
                return err;
            }
        }
        // Stop after preprocessing?
        if ((flags & PREPROCESS_ONLY) == 0) {
            parse_result* result = nullptr;
            // TODO(idarcar):
            // JsonParser json_parser;
            // XmlParser xml_parser;
            IdlParser idl_parser;
            // if (json_input) {
            // json_parser.run(ostream.str(), file.first);
            // result = &json_parser.result();
            // } else if (xml_input) {
            // xml_parser.run(ostream.str(), file.first);
            // result = &xml_parser.result();
            // } else {
            idl_parser.run(ostream.str());
            result = &idl_parser.result();
            // }
            parsed_trees.push_back(*result);
        } else {
            parse_result result;
            result.msg += ostream.str();
            parsed_trees.push_back(result);
        }
    }
    auto result = merge_results(parsed_trees);
    update_include_paths(result, expanded_files);
    if ((flags & SUPPRESS_CONTENTS_FROM_INCLUDES) != 0) {
        suppress_content_from_includes(result, expanded_files);
    }
    update_ptree_types_after_merge(result);
    return result;
}

struct IdlParserImpl {
    parse_result result;
    std::shared_ptr<parser> state;
};

// TODO(idarcar):
// struct JsonParserImpl {
//     parse_result result;
//     std::shared_ptr<parser> state;
// };
//
// struct XmlParserImpl {
//     parse_result result;
//     std::shared_ptr<parser> state;
// };

std::mutex g_parse_mutex;

IdlParser::IdlParser() : m_impl(new IdlParserImpl()) {
    m_impl->state = std::make_shared<parser>();
}

IdlParser::~IdlParser() = default;

const parse_result& IdlParser::result() const {
    return m_impl->result;
}

parse_result& IdlParser::result() {
    return m_impl->result;
}

void IdlParser::run(const std::string& input) {
    std::lock_guard<std::mutex> guard(g_parse_mutex);
    init_parser_state(m_impl->state);
    scan_string(input.c_str());
    m_impl->result = get_parse_result();
    m_impl->result.state = m_impl->state;
    g_state = std::make_shared<parser>();
    reset_top_level();
}

void IdlParser::run(FILE* input) {
    std::lock_guard<std::mutex> guard(g_parse_mutex);
    init_parser_state(m_impl->state);
    scan_file(input);
    m_impl->result = get_parse_result();
    m_impl->result.state = m_impl->state;
    g_state = std::make_shared<parser>();
    reset_top_level();
}

void IdlParser::run(const std::function<ptree*()>& input) {
    std::lock_guard<std::mutex> guard(g_parse_mutex);
    init_parser_state(m_impl->state);
    auto node = input();
    g_top_level.state = node->state;
    g_top_level.next = node;
    m_impl->result = get_parse_result();
    m_impl->result.state = m_impl->state;
    g_state = std::make_shared<parser>();
    reset_top_level();
}

std::shared_ptr<parser> IdlParser::state() {
    return m_impl->state;
}

// TODO(idarcar):
// JsonParser::JsonParser() : m_impl(new JsonParserImpl()) {
//     m_impl->state = std::make_shared<parser>();
// }
//
// JsonParser::~JsonParser() = default;
//
// const parse_result& JsonParser::result() const {
//     return m_impl->result;
// }
//
// parse_result& JsonParser::result() {
//     return m_impl->result;
// }
//
// void JsonParser::run(const std::string& input, const std::string& input_file_name) {
//     std::lock_guard<std::mutex> guard(g_parse_mutex);
//     init_parser_state(m_impl->state);
//     create_include_start(create_identifier(input_file_name.c_str()));
//     std::string canonical_file_name;
//     try {
//         canonical_file_name = std::filesystem::canonical(input_file_name);
//     } catch (std::exception&) {
//         canonical_file_name = input_file_name;
//     }
//     current_input_file = canonical_file_name.c_str();
//     g_top_level.next = create_include_finish(parse_json_ptree(input));
//     m_impl->result = get_parse_result();
//     m_impl->result.state = m_impl->state;
//     g_state = std::make_shared<parser>();
//     current_input_file = "";
//     reset_top_level();
// }
//
// void JsonParser::run(std::istream& input, const std::string& input_file_name) {
//     std::stringstream stream;
//     stream << input.rdbuf();
//     run(stream.str(), input_file_name);
// }
//
// std::shared_ptr<parser> JsonParser::state() {
//     return m_impl->state;
// }
//
// XmlParser::XmlParser() : m_impl(new XmlParserImpl()) {
//     m_impl->state = std::make_shared<parser>();
// }
//
// XmlParser::~XmlParser() = default;
//
// const parse_result& XmlParser::result() const {
//     return m_impl->result;
// }
//
// parse_result& XmlParser::result() {
//     return m_impl->result;
// }
//
// void XmlParser::run(const std::string& input, const std::string& input_file_name) {
//     std::lock_guard<std::mutex> guard(g_parse_mutex);
//     init_parser_state(m_impl->state);
//
//     auto inc_name = fmt::format("\"{}\"", input_file_name);
//     create_include_start(create_identifier(inc_name.c_str()));
//     current_input_file = get_symbol(inc_name.c_str());
//     g_top_level.next = create_include_finish(parse_xml(input));
//     m_impl->result = get_parse_result();
//     m_impl->result.state = m_impl->state;
//     g_state = std::make_shared<parser>();
//     reset_top_level();
// }
//
// std::shared_ptr<parser> XmlParser::state() {
//     return m_impl->state;
// }
}  // namespace intercom::cidl
