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

#pragma once

#include "cidl/constants.h"

#ifdef __cplusplus
extern "C" {
#endif

const char* get_symbol(const char* name);

void parse_error(const char* msg, const char* file_name, int line_number);

void parse_warning(const char* msg, const char* file_name, int line_number);

void parse_pedantic(const struct ptree* node, const char* message);

int parser_has_error(void);

struct identifier create_identifier(const char* name);

struct identifier build_scoped_name(struct identifier base, struct identifier next);

struct ptree* append_node(struct ptree* list, struct ptree* node);

struct ptree* append_enum_node(struct ptree* list, struct ptree* node);

struct declarator* append_decl(struct declarator* list, struct declarator* decl);

struct declarator* create_decl(struct identifier ident, struct ptree* annotations);

void push_context(struct ptree* p);

int register_node(struct ptree* p);

int register_node_dcl(struct ptree* p);

struct ptree* lookup_node(struct identifier ident);

struct ptree* lookup_type(struct identifier ident);

struct ptree* pop_context(void);

struct ptree* duplicate_node(const struct ptree* node);

struct ptree* duplicate_tree(const struct ptree* node);

void create_include_start(struct identifier ident);

struct ptree* create_array_type(struct declarator* declarator, struct ptree* type);

struct ptree* create_include_finish(struct ptree* members);

void create_module_start(struct identifier ident);

struct ptree* create_module_finish(struct ptree* members, struct position pos_end);

const struct numeric* lookup_value(struct identifier ident);

const struct numeric* create_value_node(const struct numeric* value, struct ptree* members);

struct ptree* create_const_node(struct declarator* decl, struct ptree* type, const struct numeric* value);

struct ptree* create_sequence(struct ptree* element_type, const struct numeric* bound);

struct ptree* create_string(const struct numeric* bound);

struct ptree* create_wstring(const struct numeric* bound);

struct ptree* create_fixed(const struct numeric* bound1, const struct numeric* bound2);

struct numeric* new_numeric(enum numeric_kind kind);

const struct numeric* create_bool(int value);

const struct numeric* create_char(char value);

const struct numeric* create_i64(int64_t value, int base);

const struct numeric* create_u64(uint64_t value, int base);

const struct numeric* create_str(const char* value);

const struct numeric* create_float(float value);

const struct numeric* create_double(double value);

struct ptree* create_struct_start(struct identifier ident, struct ptree* parent);

struct ptree* create_struct_finish(struct ptree* members, struct position pos_end);

struct ptree* create_struct_dcl(struct identifier ident);

struct ptree* create_union_start(struct identifier ident);

struct ptree* create_union_finish(struct ptree* discriminator, struct ptree* members, struct position pos_end);

struct ptree* create_union_dcl(struct identifier ident);

struct ptree* create_member(struct declarator* declarators, struct ptree* type, struct ptree* annotations);

struct ptree* create_union_member(struct ptree* value, struct ptree* cases, struct ptree* annotations);

struct ptree* create_case_label(const struct numeric* value);

struct ptree* create_default_case(void);

struct ptree* create_null_node(void);

struct ptree* create_enum(struct identifier ident, struct ptree* values, struct position pos_end);

struct ptree* create_enum_value(struct identifier ident, const struct numeric* value);

struct ptree* create_type(struct declarator* declarators, struct ptree* type);

struct ptree* create_native_type(struct identifier ident);

void create_exception_start(struct identifier ident);

struct ptree* create_exception_finish(struct ptree* members, struct position pos_end);

struct ptree* create_interface_dcl(struct identifier ident, int is_local);

void create_interface_start(struct identifier ident, struct declarator* parents, int is_local);

struct ptree* create_interface_finish(struct ptree* members, struct position pos_end);

struct ptree* annotate(struct ptree* node, struct ptree* annotations);

struct ptree* annotate_alias(struct ptree* node, struct ptree* annotations);

struct ptree* annotate_list(struct ptree* node, struct ptree* annotations);

struct ptree* annotate_last(struct ptree* node, struct ptree* annotations);

struct ptree* create_interface_op(struct identifier ident, struct ptree* params, struct ptree* retval,
                                  struct declarator* raises);

struct ptree* create_param_dcl(struct declarator* decl, struct ptree* type, int kind);

struct ptree* create_attribute(struct declarator* decl, struct ptree* type, struct declarator* getraises,
                               struct declarator* setraises, int readonly);

struct ptree* create_map(struct ptree* key_type, struct ptree* element_type, const struct numeric* bound);

struct ptree* create_bitset(struct identifier ident, struct ptree* fields, struct ptree* type, struct position pos_end);

struct ptree* create_bitfield(struct declarator* declarators, const struct numeric* bits, struct ptree* type);

struct ptree* create_bitmask(struct identifier ident, struct ptree* values, struct position pos_end);

struct ptree* create_bitmask_value(struct identifier ident, const struct numeric* value);

void create_annotation_dcl_start(struct identifier ident);

struct ptree* create_annotation_dcl_finish(struct ptree* members, struct position pos_end);

struct ptree* create_annotation_member(struct declarator* decl, struct ptree* type,
                                       const struct numeric* default_value);

void create_annotation_start(struct identifier ident);

struct ptree* create_annotation_finish(struct ptree* params);

struct ptree* create_annotation_param(struct identifier ident, const struct numeric* value);

struct ptree* create_valuetype_dcl(struct identifier ident);

struct ptree* create_valuetype_start(struct identifier ident, struct ptree* parent, struct ptree* interface);

struct ptree* create_valuetype_finish(struct ptree* members, struct position pos_end);

struct ptree* create_valuetype_factory(struct identifier ident, struct ptree* params, struct declarator* raises);

struct ptree* create_valuetype_factory_param(struct declarator* decl, struct ptree* type);

struct ptree* create_valuetype_member(struct declarator* declarators, struct ptree* type, int is_public);

struct declarator* append_array_size(struct declarator* decl, const struct numeric* value);

struct declarator* set_array_bounds(struct declarator* decl, struct declarator* bounds);

struct identifier create_anon_name(void);

void validate_tree(struct ptree* node);

void format_doxy_comments(struct ptree* tree);

struct ptree* try_lookup_node(const char* name, const enum node_kind kind[]);

struct ptree* create_node(enum node_kind kind, struct identifier ident);

struct ptree* create_doc(struct identifier ident, int post_comment);

INTERCOM_PUBLIC struct ptree* merge_members(struct ptree* node, struct ptree* members);

#ifdef __cplusplus
}

#  include <map>
#  include <mutex>
#  include <set>
#  include <sstream>
#  include <string>
#  include <vector>

#  include "InterCOM/RefPointer.h"
#  include "cidl/internal/commandline.h"
#  include "cidl/ptree.h"

extern "C" struct parser {
    ptree* lookup_node(const char* name) const;

    long long enum_counter{0};
    int anonymous_name_count{0};
    intercom::optional<ptree*> current_under_documentation;
    std::vector<std::vector<ptree*>> context;
    std::vector<ptree*> include_context;
    std::map<std::string, ptree*> type_map;
    std::map<std::string, ptree*> type_dcl_map;
    std::string comment_string;

    std::vector<intercom::RefPointer<ptree>> allocated_nodes;
    std::vector<intercom::RefPointer<declarator>> allocated_decl;
    std::set<std::string> symbol_map;
    std::list<numeric> numeric_map;
};

namespace intercom {
namespace cidl {
inline std::string tolower(std::string res) {
    transform(res.begin(), res.end(), res.begin(), [](std::string::value_type c) {
        return static_cast<std::string::value_type>(std::tolower(static_cast<int>(c)));
    });
    return res;
}

/// parses messages, but does not print them
class ParserMessage {
  public:
    using WriterType = void (*)(const char*, const char*, int, CommandLineOption::WarningType);

    ParserMessage(WriterType writer, CommandLineOption::WarningType warning)
            : line_number(current_pos.line), writer(writer), warning_type(warning) {}

    ~ParserMessage() {
        if (context_node) {
            writer(stream.str().c_str(), context_node->file_name.c_str(), context_node->pos.line, warning_type);
        } else {
            writer(stream.str().c_str(), current_input_file, line_number, warning_type);
        }
    }

    ParserMessage& context(const ptree* n) {
        context_node = n;
        return *this;
    }

    ParserMessage& context(struct identifier ident) {
        line_number = ident.pos.line;
        return *this;
    }

    template <typename T>
    ParserMessage& operator<<(const T& val) {
        stream << val;
        return *this;
    }

    ParserMessage& operator<<(numeric_kind val) {
        stream << numeric_kind_str(val);
        return *this;
    }

    ParserMessage& operator<<(node_kind val) {
        stream << node_kind_str(val);
        return *this;
    }

    ParserMessage& operator<<(struct ptree* val) { return operator<<(static_cast<const struct ptree*>(val)); }

    ParserMessage& operator<<(const struct ptree* val) {
        if (val) {
            stream << "\"" << val->name << "\"";
        } else if (val) {
            stream << "\"(no name)\"";
        } else {
            stream << "\"(null)\"";
        }
        return *this;
    }

    template <typename Iter_t>
    ParserMessage& append(Iter_t begin, Iter_t end, const char* separator = ", ") {
        while (begin != end) {
            *this << *begin;
            if (begin + 1 != end) {
                *this << separator;
            }
            begin++;
        }
        return *this;
    }

  protected:
    std::stringstream stream{};
    const ptree* context_node{nullptr};
    int line_number;

    const WriterType writer;
    const CommandLineOption::WarningType warning_type;
};

INTERCOM_PUBLIC extern intercom::RefPointer<::parser> g_state;
INTERCOM_PUBLIC extern std::mutex g_parse_mutex;
}  // namespace cidl
}  // namespace intercom

/// creates error, warning, or nothing depending on commandline user inputs
void parse_alert(const char* msg, const char* file_name, int line_number, CommandLineOption::WarningType warning_type);

/// creates error, warning, or nothing depending on commandline user inputs
#  define ALERT(warning_type)                                       \
      intercom::cidl::ParserMessage msg(parse_alert, warning_type); \
      msg

#  define ERR                                                                                         \
      intercom::cidl::ParserMessage msg(parse_alert, CommandLineOption::WARNING_UNCATEGORIZED_ERROR); \
      msg

#  define WARN                                                                                          \
      intercom::cidl::ParserMessage msg(parse_alert, CommandLineOption::WARNING_UNCATEGORIZED_WARNING); \
      msg

#endif
