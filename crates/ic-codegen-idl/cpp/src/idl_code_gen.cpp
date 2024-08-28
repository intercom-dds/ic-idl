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

#include <cstdint>
#include <cstring>
#include <filesystem>
#include <iostream>
#include <map>
#include <set>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "utils/md5.h"

// Defined in Windows.h
#ifdef OUT
#  undef OUT
#endif

#define OUT idl_module_stream(obj, out)

using namespace intercom::cidl;

static std::vector<std::string> split_to_lines(const char* text) {
    std::istringstream stream(text);
    std::vector<std::string> lines;
    std::string tmp;
    while (std::getline(stream, tmp)) {
        if (!lines.empty() || !tmp.empty()) {
            lines.emplace_back(tmp);
        }
    }
    while (!lines.empty() && lines.back().empty()) {
        lines.pop_back();
    }
    return lines;
}

using ModuleMap = std::map<std::string, intercom::cidl::PrettyPrinter>;

static intercom::cidl::PrettyPrinter& idl_module_stream(const ptree* obj, ModuleMap& out) {
    return obj->included_from ? out[obj->included_from->name] : out[obj->file_name];
}

static bool ann_argument_is_default(const ptree* arg, const ptree* ann_type) {
    const ptree* ann_type_arg = nullptr;
    for (auto targ : ann_type->members) {
        if (targ->name == arg->name) {
            ann_type_arg = targ;
            break;
        }
    }
    if (ann_type_arg && ann_type_arg->value.kind() != UNDEF_KIND) {
        return string_value(arg->value) == string_value(ann_type_arg->value);
    }
    return false;
}

static bool ann_arguments_are_default(const ptree* ann) {
    if (ann->members != nullptr) {
        for (auto el : ann->members) {
            if (!ann_argument_is_default(el, ann->type)) {
                return false;
            }
        }
    }
    return true;
}

static std::string idl_const_value(const ptree* const_value, const ptree* context = nullptr);
static std::string idl_const_value(const numeric& value, const ptree* context = nullptr);

static bool
emit_annotation(const ptree* ann, const ptree* ctx, bool allow_newlines, PrettyPrinter& stream) {
    stream << "@" << idl_scoped_name(ann, namespace_of(annotation_type_range));
    if (ann->members && !ann_arguments_are_default(ann)) {
        stream << begin_paren << tab_group;
        if (allow_newlines && ann->members->next) {
            stream << endl;
        }
        bool first = true;
        for (auto el : ann->members) {
            if (ann_argument_is_default(el, ann->type)) {
                continue;
            }
            if (!first) {
                stream << ", ";
                if (allow_newlines) {
                    stream << endl;
                }
            }
            if (ann->members->next) {
                stream << idl_name(el) << tab << "=" << tab;
            }
            // Annotation values are looked up in both scopes. Use shortest.
            std::string v1 = idl_const_value(el, ctx->super);
            std::string v2 = idl_const_value(el, ann->type);
            stream << ((v1.length() < v2.length()) ? v1 : v2);
            first = false;
        }
        stream << tab_group << end_paren << endl;
        return true;
    }
    return false;
}

static intercom::cidl::PrettyPrinter idl_annotations(
    const ptree* obj,
    bool allow_newlines = true,
    const std::set<const ptree*>& suppress = std::set<const ptree*>()
) {
    bool newline_between_annotations = false;
    if (allow_newlines) {
        for (auto ann : obj->annotations) {
            if (ann->members && !ann_arguments_are_default(ann)) {
                newline_between_annotations = true;
                break;
            }
        }
    }
    intercom::cidl::PrettyPrinter stream;

    bool has_annotation = false;
    bool first_on_line = true;
    for (auto ann : obj->annotations) {
        if (suppress.find(ann->type) != suppress.end() || ann->type == annotation_type_doc) {
            continue;
        }
        // Annotations are emitted as as Doxygen comments instead since Doxygen
        // doesn't like IDL annotations.
        if (CommandLineOption::doxy_compatible_output()) {
            continue;
        }
        has_annotation = true;
        first_on_line = emit_annotation(ann, obj, allow_newlines, stream);
        if (newline_between_annotations) {
            stream << endl;
        }
    }
    if (has_annotation) {
        if (allow_newlines && obj->kind != N_MEMBER && obj->kind != N_PROTOTYPE) {
            stream << endl;
        } else if (!first_on_line) {
            stream << " ";
        }
    }
    return stream;
}

static intercom::cidl::PrettyPrinter idl_type_name(const ptree* node, const ptree* context) {
    intercom::cidl::PrettyPrinter stream;
    switch (node->kind) {
    case N_PRIMITIVE:
    case N_NATIVE:
    case N_STRING:
    case N_FIXED:
        stream << idl_name(node);
        break;
    case N_SEQUENCE: {
        stream << "sequence<" << idl_type_name(node->element_type, context);
        if (!node->bounds.empty()) {
            stream << "," << idl_const_value(node->bounds[0], context);
        }
        stream << ">";
    } break;
    case N_MAP: {
        stream << "map<" << idl_type_name(node->key_type, context) << ","
               << idl_type_name(node->element_type, context);
        if (!node->bounds.empty()) {
            stream << "," << idl_const_value(node->bounds[0], context);
        }
        stream << ">";
    } break;
    case N_ARRAY:
        return idl_type_name(node->element_type, context);
        break;
    case N_ALIAS:
        if (node->flags & OPT_ANONYMOUS_ALIAS) {
            stream << idl_annotations(node, false);
            stream << idl_type_name(node->type, context);
        } else {
            stream << idl_scoped_name(
                node, context
            );  // same as default TODO [[fallthrough]] (cpp17)
        }
        break;
    default:
        stream << idl_scoped_name(node, context);
        break;
    }
    return stream;
}

static intercom::cidl::PrettyPrinter idl_member_type_name(const ptree* node) {
    if (node->type->kind == N_ARRAY) {
        return idl_type_name(node->type->element_type, node);
    }
    return idl_type_name(node->type, node);
}

static std::string idl_member_name(const ptree* node) {
    if (node->type->kind == N_ARRAY) {
        std::stringstream str;
        str << node->name;
        for (size_t i = 0; i < node->type->bounds.size(); ++i) {
            str << "[" << idl_const_value(node->type->bounds[i], node->super) << "]";
        }
        return str.str();
    }
    return idl_name(node);
}

static std::string idl_const_value(const ptree* const_value, const ptree* context) {
    if (const_value->value.kind() != PTREE_KIND && const_value->type != context) {
        if (const_value->type->kind == N_ENUM) {
            auto uvalue = value<uint64_t>(const_value->value);
            for (auto m : const_value->type->members) {
                if (value<uint64_t>(m->value) == uvalue) {
                    return idl_scoped_name(m, context);
                }
            }
        } else if (const_value->type->kind == N_BITMASK) {
            std::ostringstream out;
            auto uvalue = value<uint64_t>(const_value->value);
            bool empty = true;
            for (auto m : const_value->type->members) {
                if ((value<uint64_t>(m->value) & uvalue) != 0) {
                    if (!empty) {
                        out << "|";
                    }
                    out << idl_scoped_name(m, context);
                    uvalue &= ~value<uint64_t>(m->value);
                    empty = false;
                }
            }
            if (uvalue != 0 && !empty) {
                out << "|" << uvalue;
                uvalue = 0;
            }
            if (uvalue == 0) {
                return out.str();
            }
        }
    }
    return idl_const_value(const_value->value, context);
}

static std::string idl_const_value(const numeric& value, const ptree* context) {
    std::stringstream out;

    switch (value.kind()) {
    case CHAR_KIND:
        out << "'" << (static_cast<char>(value.val.c())) << "'";
        break;
    case STRING_KIND:
        out << "\"" << value.val.str() << "\"";
        break;
    case PTREE_KIND:
        if (value.val.node()->kind == N_CONST && (value.val.node()->flags & OPT_CONST_VALUE) != 0) {
            out << "{";
            for (auto member : value.val.node()->members) {
                if (member != value.val.node()->members) {
                    out << ", ";
                }
                out << idl_const_value(member, context);
            }
            out << "}";
        } else {
            out << idl_scoped_name(value.val.node(), context);
        }
        break;
    default:
        out << string_value(value);
    }

    return out.str();
}

static void include_type(const ptree* obj, const ptree* curr_include, std::set<ptree*>& includes) {
    if (obj) {
        if (obj->included_from && obj->included_from != curr_include) {
            includes.insert(obj->included_from);
        }
        for (auto parent : obj->parents) {
            include_type(parent, curr_include, includes);
        }
        include_type(obj->type, curr_include, includes);
        include_type(obj->element_type, curr_include, includes);
        include_type(obj->key_type, curr_include, includes);
    }
}

static void code_gen_idl_compound(const ptree* obj, ModuleMap& out);

static void
include_dependencies(const ptree* obj, const ptree* curr_include, std::set<ptree*>& includes) {
    for (; obj; obj = obj->next) {
        if (!is_emit(obj, LANG_IDL)) {
            continue;
        }
        if (obj->included_from == curr_include) {
            include_type(obj, curr_include, includes);
            if (obj->value.kind() == PTREE_KIND) {
                include_type(obj->value.val.node(), curr_include, includes);
            }
        }
        include_dependencies(obj->members, curr_include, includes);
    }
}

static void code_gen_idl_comments_post(const ptree* obj, ModuleMap& out) {
    for (auto ann : obj->annotations) {
        if (!is_post_doc(ann)) {
            continue;
        }
        OUT.indent_to_column_begin(" ");
        for (const auto& line : split_to_lines(ann->members->value.val.str().c_str())) {
            OUT << "//!" << line << endl;
        }
        OUT << end("");
    }
}

static void code_gen_idl_comments(const ptree* obj, ModuleMap& out) {
    if (CommandLineOption::doxy_compatible_output()) {
        std::vector<std::string> brief_lines;
        std::vector<std::string> detail_lines;
        ptree* range = get_annotation(obj, annotation_type_range);
        if (range) {
            std::stringstream stream;
            stream << "Range " << string_value(get_annotation_value(range, "min")) << " .. "
                   << string_value(get_annotation_value(range, "max"));
            brief_lines.push_back(stream.str());
        }
        if (obj->kind == N_MEMBER) {
            ptree* default_value = get_annotation(obj, annotation_type_default);
            if (default_value) {
                std::stringstream stream;
                stream << "Default value "
                       << string_value(get_annotation_value(default_value, "value"));
                brief_lines.push_back(stream.str());
            }
        }
        if (obj->kind == N_BITMASK) {
            std::stringstream brief_stream;
            brief_stream << "Bitmask( " << get_bit_size(obj) << " )";
            brief_lines.push_back(brief_stream.str());
            std::stringstream stream;
            stream << "@bitmasktype{ " << get_bit_size(obj) << " }";
            detail_lines.push_back(stream.str());
        }
        if (get_annotation(obj, annotation_type_optional)) {
            brief_lines.emplace_back("Optional member");
        }
        if (obj->kind == N_UNION) {
            std::stringstream brief_stream;
            brief_stream << "Union( " << idl_scoped_name(obj->discriminator->type, obj) << " )";
            brief_lines.push_back(brief_stream.str());
            std::stringstream stream;
            stream << "@union{ " << idl_scoped_name(obj->discriminator->type, obj) << " }";
            detail_lines.push_back(stream.str());
        }
        if (obj->kind == N_MEMBER) {
            if (base_type_of(obj)->kind == N_BITMASK) {
                std::stringstream brief_stream;
                brief_stream << "Bitmask( " << get_bit_size(obj) << " )";
                brief_lines.push_back(brief_stream.str());
            }
            if (obj->super->kind == N_UNION) {
                bool first = true;
                std::stringstream stream;
                stream << "Valid for";
                for (auto case_value : obj->members) {
                    stream << (first ? " " : ", ");
                    first = false;
                    if (case_value->flags & OPT_DEFAULT) {
                        stream << "default";
                    } else if (case_value->value.kind() == PTREE_KIND &&
                               case_value->type == case_value->value.val.node()->type) {
                        stream << "@ref " << idl_scoped_name(case_value->value.val.node(), obj);
                    } else {
                        stream << idl_const_value(case_value, case_value);
                    }
                }
                brief_lines.push_back(stream.str());
            }
        }
        for (auto ann : obj->annotations) {
            if (!is_pre_doc(ann)) {
                continue;
            }
            std::stringstream brief_line;
            bool brief = false;
            for (auto line : split_to_lines(ann->members->value.val.str().c_str())) {
                if (line.empty()) {
                    brief = false;
                } else if (line.substr(0, 8) == "@details" || line.substr(0, 8) == "\\details") {
                    line = line.substr(8);
                    brief = false;
                } else if (line.substr(0, 6) == "@brief" || line.substr(0, 6) == "\\brief") {
                    line = line.substr(6);
                    brief = true;
                }
                if (brief) {
                    if (!brief_line.str().empty()) {
                        brief_line << " ";
                    }
                    brief_line << line;
                } else {
                    detail_lines.emplace_back(line);
                }
            }
            if (!brief_line.str().empty()) {
                brief_lines.push_back(brief_line.str());
            }
        }
        if (!detail_lines.empty() || !brief_lines.empty()) {
            if (obj->kind != N_MEMBER || obj->super->kind != N_UNION) {
                OUT << blank_line;
            }
            for (auto it = brief_lines.begin(); it != brief_lines.end(); ++it) {
                if (!it->empty()) {
                    OUT << (it == brief_lines.begin() ? "//! @brief " : ", ") << *it;
                }
            }
            if (!brief_lines.empty()) {
                OUT << endl;
            }
            if (!detail_lines.empty() && !brief_lines.empty() && !brief_lines.back().empty()) {
                OUT << "//!" << endl;
            }
            for (auto it = detail_lines.begin(); it != detail_lines.end(); ++it) {
                if (it->empty()) {
                    OUT << "//!" << endl;
                } else {
                    OUT << "//! ";
                    if (it == detail_lines.begin()) {
                        if (detail_lines.size() > 1) {
                            OUT << "@details" << endl << "//! ";
                        } else {
                            OUT << "@details ";
                        }
                    }
                    OUT << *it << endl;
                }
            }
        }

        bool has_annotations = false;
        for (auto ann : obj->annotations) {
            if (ann->type != annotation_type_doc) {
                has_annotations = true;
                break;
            }
        }

        if (has_annotations) {
            if (!brief_lines.empty() || !detail_lines.empty()) {
                OUT << "//!" << endl;
            }
            OUT << "//! \\b Annotations:" << endl;
            for (auto ann : obj->annotations) {
                if (ann->type != annotation_type_doc) {
                    OUT << "//! - ";
                    emit_annotation(ann, obj, false, OUT);
                    OUT << endl;
                }
            }
        }
    } else {
        for (auto ann : obj->annotations) {
            if (!is_pre_doc(ann)) {
                continue;
            }
            for (const auto& line : split_to_lines(ann->members->value.val.str().c_str())) {
                OUT << "//!";
                if (!line.empty()) {
                    OUT << " " << line;
                }
                OUT << endl;
            }
        }
    }
}

static std::string code_gen_idl_list(std::vector<ptree*> list, const ptree* context) {
    std::stringstream out;
    for (size_t i = 0; i < list.size(); ++i) {
        out << (i == 0 ? "" : ", ") << idl_scoped_name(list[i], context);
    }
    return out.str();
}

static void code_gen_idl_enum_body(const ptree* obj, ModuleMap& out) {
    OUT << endl << begin_curly << endl << tab_group;
    for (auto el : obj->members) {
        code_gen_idl_comments(el, out);
        OUT << idl_annotations(el);
        if (CommandLineOption::doxy_compatible_output()) {
            OUT << idl_name(el);
            if (obj->flags & OPT_ENUMERATED) {
                OUT << tab << "=" << tab << long_long_value(el->value);
            }
        } else {
            if (obj->flags & OPT_ENUMERATED) {
                if (obj->kind == N_BITMASK) {
                    int position = 0;
                    auto element_value = value<uint64_t>(el->value);
                    while (element_value >>= 1) {
                        ++position;
                    }
                    OUT << "@position(" << position << ")" << tab;
                } else {
                    OUT << "@value(" << idl_const_value(el, obj) << ")" << tab;
                }
            }
            OUT << idl_name(el);
        }
        if (el->next) {
            OUT << ",";
        }
        OUT << tab;
        code_gen_idl_comments_post(el, out);
        OUT << endl;
    }
    OUT << tab_group << end_curly << ";";
}

static bool is_air_dummy_union(const ptree* obj) {
    if (obj && obj->kind == N_UNION) {
        if (idl_scoped_name(base_type_of(obj->discriminator), nullptr) ==
            "CFDFA_vmf_types::vmf_boolean") {
            return true;
        }
        if (idl_scoped_name(base_type_of(obj->discriminator), nullptr) ==
            "vmf_types::vmf_boolean") {
            return true;
        }
        for (auto elem : obj->members) {
            if (elem->kind == N_MEMBER &&
                elem->name.compare(0, 35, "void_void_void_dummy_skipped_in_air") == 0) {
                return true;
            }
        }
    }
    return false;
}

static void code_gen_idl_rec(const ptree* obj, ModuleMap& out) {
    if (!is_emit(obj, LANG_IDL)) {
        return;
    }

    ptree stack_replacement;

    if (is_air_dummy_union(obj)) {
        return;
    }
    switch (obj->kind) {
    case N_UNDEF:
    case N_NATIVE:
    case N_SEQUENCE:
    case N_MAP:
    case N_ARRAY:
    case N_STRING:
    case N_FIXED:
    case N_PRIMITIVE:
    case N_ANNOTATION:
    case N_INCLUDE:
        return;
    default:
        break;
    }
    if (obj->kind == N_MEMBER && is_air_dummy_union(obj->type)) {
        OUT << "@optional ";
        const ptree* first = obj->type->members;
        while (first->kind != N_MEMBER) {
            first = first->next;
        }
        stack_replacement = *first;
        stack_replacement.super = obj->super;
        stack_replacement.next = obj->next;
        stack_replacement.name = obj->name;
        stack_replacement.included_from = obj->included_from;
        obj = &stack_replacement;
    }

    PrettyPrinter::Context context(OUT, obj);

    if (obj->kind == N_MODULE || obj->kind == N_STRUCT || obj->kind == N_UNION ||
        obj->kind == N_VALUETYPE || obj->kind == N_INTERFACE || obj->kind == N_ENUM ||
        obj->kind == N_BITSET || obj->kind == N_BITMASK || obj->kind == N_ANNOTATION_DEF ||
        obj->kind == N_EXCEPTION) {
        OUT << blank_line;
    }

    code_gen_idl_comments(obj, out);
    OUT << idl_annotations(obj);

    switch (obj->kind) {
    case N_MODULE:
        OUT << "module " << idl_name(obj);
        OUT << endl << begin_curly << endl << tab_group;
        for (auto el : obj->members) {
            code_gen_idl_rec(el, out);
            if (el->next && el->kind != el->next->kind) {
                OUT << blank_line;
                OUT << tab_group;
            }
        }
        OUT << tab_group << endl << end_curly << "; ";
        OUT << "// module " << idl_name(obj) << endl;
        break;

    case N_STRUCT:
        OUT << "struct " << idl_name(obj);
        code_gen_idl_compound(obj, out);
        break;

    case N_UNION:
        OUT << "union " << idl_name(obj);
        if (obj->discriminator) {
            std::set<const ptree*> suppress;
            suppress.insert(annotation_type_must_understand);
            OUT << " switch( ";
            OUT << idl_annotations(obj->discriminator, true, suppress);
            OUT << idl_type_name(obj->discriminator->type, obj) << " )";
        }
        code_gen_idl_compound(obj, out);
        break;

    case N_VALUETYPE:
        OUT << "valuetype " << idl_name(obj);
        code_gen_idl_compound(obj, out);
        break;

    case N_INTERFACE:
        if (obj->flags & OPT_LOCAL) {
            OUT << "local ";
        }
        OUT << "interface " << idl_name(obj);
        code_gen_idl_compound(obj, out);
        break;

    case N_ENUM:
        OUT << "enum " << idl_name(obj);
        code_gen_idl_enum_body(obj, out);
        break;

    case N_BITSET:
        OUT << "bitset " << idl_name(obj);
        code_gen_idl_compound(obj, out);
        break;

    case N_BITMASK:
        if (CommandLineOption::doxy_compatible_output()) {
            OUT << "enum " << idl_name(obj);
        } else {
            OUT << "bitmask " << idl_name(obj);
        }
        code_gen_idl_enum_body(obj, out);
        break;

    case N_ANNOTATION_DEF:
        if (!CommandLineOption::doxy_compatible_output()) {
            OUT << "@annotation " << idl_name(obj);
            code_gen_idl_compound(obj, out);
        }
        break;

    case N_EXCEPTION:
        OUT << "exception " << idl_name(obj);
        code_gen_idl_compound(obj, out);
        break;

    case N_ALIAS:
        OUT << "typedef " << idl_member_type_name(obj) << tab << idl_member_name(obj) << ";";
        break;

    case N_CONST:
        OUT << "const " << idl_member_type_name(obj) << tab << idl_member_name(obj) << tab << "="
            << tab << idl_const_value(obj, obj) << ";";
        break;

    case N_CASE:
        if (obj->flags & OPT_DEFAULT) {
            OUT << unindent << "default:";
        } else if (obj->value.kind() == PTREE_KIND && obj->type == obj->value.val.node()->type) {
            OUT << unindent << "case " << idl_scoped_name(obj->value.val.node(), obj) << ":";
        } else {
            OUT << unindent << "case " << idl_const_value(obj, obj) << ":";
        }
        break;

    case N_NULL:
        OUT << "null;";
        break;

    case N_MEMBER:
        if (obj->super->kind == N_INTERFACE) {
            if (obj->flags & OPT_READONLY) {
                OUT << "readonly ";
            }
            OUT << "attribute ";
        }
        if (obj->super->kind == N_VALUETYPE) {
            OUT << ((obj->flags & OPT_PRIVATE) ? "private " : "public ");
        }
        OUT << idl_member_type_name(obj) << tab << idl_member_name(obj);
        if (!obj->getraises.empty()) {
            OUT << " getraises ( " << code_gen_idl_list(obj->getraises, obj->scope) << " )";
        }
        if (!obj->setraises.empty()) {
            OUT << " setraises ( " << code_gen_idl_list(obj->setraises, obj->scope) << " )";
        }
        if (obj->super->kind == N_ANNOTATION_DEF && obj->value.kind() != UNDEF_KIND) {
            OUT << " default " << idl_const_value(obj, obj->super);
        }
        OUT << ";";
        break;

    case N_PROTOTYPE:
        if (obj->super->kind == N_INTERFACE) {
            if (obj->type) {
                OUT << idl_member_type_name(obj);
            } else {
                OUT << "void";
            }
            OUT << " ";
        }
        if (obj->super->kind == N_VALUETYPE) {
            OUT << "factory ";
        }
        OUT << idl_name(obj);
        OUT.begin("( ");
        for (auto el : obj->members) {
            OUT << list_sep;
            if ((el->flags & OPT_INOUT) == OPT_INOUT) {
                OUT << "inout ";
            } else if (el->flags & OPT_IN) {
                OUT << "in ";
            } else if (el->flags & OPT_OUT) {
                OUT << "out ";
            }
            OUT << idl_type_name(el->type, obj) << " " << idl_name(el);
        }
        OUT.end(" )");
        if (!obj->getraises.empty()) {
            OUT << " raises ( " << code_gen_idl_list(obj->getraises, obj->scope) << " )";
        }
        OUT << ";";
        break;
    default:
        break;
    }

    OUT << tab;
    code_gen_idl_comments_post(obj, out);
    OUT << endl;

    // this code will output the generated idl.
    if (CommandLineOption::expand_idl()) {
        for (auto gen : obj->generated) {
            code_gen_idl_rec(gen, out);
        }
    }
}

static void code_gen_idl_compound(const ptree* obj, ModuleMap& out) {
    if (!obj->parents.empty()) {
        OUT.begin(" : ");
        OUT << code_gen_idl_list(obj->parents, obj->scope);
        if (obj->kind == N_VALUETYPE && obj->type) {
            OUT << " supports " << idl_scoped_name(obj->type, obj->super);
        }
        OUT.end("");
    }

    if (obj->flags & OPT_DECLARATION) {
        OUT << ";" << endl;
        return;
    }

    OUT << endl << begin_curly << endl << tab_group;
    for (auto el : obj->members) {
        if (obj->kind == N_UNION) {
            code_gen_idl_rec(el->members, out);
        }
        code_gen_idl_rec(el, out);
    }
    OUT << tab_group << endl << end_curly << ";";
}

static std::string code_gen_idl_content(
    ModuleMap& out,
    const std::string& name,
    const std::string& filename,
    std::set<ptree*>& includes
) {
    PrettyPrinter module = out[name];

    std::string uppername = filename;
    for (char& it : uppername) {
        it = static_cast<std::string::value_type>(toupper(it));
    }
    std::stringstream file;
    // TODO: Do we also want to emit it here? It makes sense for e.g. xmi2idl, but less so for
    // interactive use build info header file << "/*" << generate_info_header(result->file_path,
    // "\t") << " */\n\n";

    std::string uppername_underscore = uppername;
    std::replace(uppername_underscore.begin(), uppername_underscore.end(), '-', '_');
    std::replace(uppername_underscore.begin(), uppername_underscore.end(), '.', '_');

    if (!includes.empty()) {
        for (auto include : includes) {
            file << "#include " << ((include->flags & OPT_SYSTEM_INCLUDE) ? '<' : '"')
                 << include->name << ((include->flags & OPT_SYSTEM_INCLUDE) ? '>' : '"')
                 << std::endl;
        }
        file << std::endl;
    }
    module.print(file);

    auto file_str = file.str();
    intercom::MD5 md5(
        reinterpret_cast<const uint8_t*>(file_str.data()), static_cast<uint32_t>(file_str.size())
    );

    std::stringstream file_out;
    if (CommandLineOption::legacy_idl()) {
        file_out << "#ifndef " << uppername_underscore << "_" << md5.toString() << std::endl;
        file_out << "#define " << uppername_underscore << "_" << md5.toString() << std::endl
                 << std::endl;
        file_out << file_str << std::endl;
        file_out << "#endif" << std::endl;
    } else {
        file_out << "#pragma once" << std::endl << std::endl << file_str << std::endl;
    }
    return file_out.str();
}

static void code_gen_idl_write(
    ModuleMap& out,
    const std::string& name,
    const std::string& filename,
    std::set<ptree*>& includes,
    const parse_result* result
) {
    if (out.find(name) == out.end()) {
        return;
    }
    std::string filepath;
    if (CommandLineOption::idl_target_directory()) {
        filepath = std::string(CommandLineOption::idl_target_directory()) + "/" + filename;
    } else {
        filepath = filename;
    }
    filepath = std::filesystem::path(filepath).replace_extension(".idl").string();

    auto content = code_gen_idl_content(out, name, filename, includes);

    write_if_changed(filepath, content);
}

static void code_gen_idl(const parse_result* result) {
    ModuleMap out;

    auto tree = result->tree;
    if (!CommandLineOption::expand_idl()) {
        tree = original_node(tree);
    }
    for (auto obj : tree) {
        code_gen_idl_rec(obj, out);
    }

    for (auto it = result->includes.begin(); it != result->includes.end(); ++it) {
        const ptree* include = *it;
        std::set<ptree*> includes;
        include_dependencies(result->tree, include, includes);
        if (CommandLineOption::list_only()) {
            std::cout << include->name << std::endl;
        } else {
            code_gen_idl_write(out, include->name, include->name, includes, result);
        }
    }
}

void intercom::cidl::code_gen_idl(const parse_result* result, const char* destination) {
    CommandLineOption::get_instance().idl_target_directory = destination;
    ::code_gen_idl(result);
}
