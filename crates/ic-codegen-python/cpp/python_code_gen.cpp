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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <set>
#include <string_view>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/idl_parser.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree.h"
#include "cidl/ptree_ffi.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

namespace {
struct ModuleContext {
    std::string name;
    std::string file_name;
    std::string file_base_name;
    PrettyPrinter pp;
    std::set<std::string> imports;
    ModuleContext* parent;

    template <typename T>
    PrettyPrinter& operator<<(T arg) {
        return pp << arg;
    }

    ModuleContext(
        std::string name,
        std::string file_name,
        std::string file_base_name,
        ModuleContext* parent = nullptr
    )
        : name(std::move(name)),
          file_name(std::move(file_name)),
          file_base_name(std::move(file_base_name)),
          parent(parent) {
        pp.set_indent_size(4);
    };
};
}  // namespace

using ModuleMap = std::map<std::string, ModuleContext*>;
using FileMap = std::map<std::string, ModuleMap>;

static std::string
python_const_value(const numeric& value, const ptree* context, ModuleContext* module);
static std::string python_base_type(const ptree* obj, const ptree* context, ModuleContext* module);
static std::string python_type_name(const ptree* node, const ptree* context, ModuleContext* module);
static std::string
python_class_type(const ptree* node, const ptree* context, ModuleContext* module);

// TOOD(idarcar):
static std::string extract_file_name(const std::string& file_name) {
    return std::filesystem::path(file_name).stem().string();
    // std::replace(file_name.begin(), file_name.end(), '/', '_');
    // intercom::corba::String_var to_trim = file_name.c_str();
    // return trim_include_name(to_trim.inout(), false);
}

static std::string module_file_name(const std::string& file_name) {
    return "_cidl_" + file_name;
}

static std::string
python_variable_name(const ptree* node, const ptree* context, ModuleContext* module) {
    std::stringstream stream;
    std::string var_name = python_name(node);
    if (node->value.kind() == PTREE_KIND) {
        var_name = python_name(node->value.val.node());
    }
    if (base_type_of(node)->kind == N_ENUM || base_type_of(node)->kind == N_STRUCT ||
        base_type_of(node)->kind == N_UNION) {
        {
            if (base_type_of(node) != node) {
                stream << python_class_type(base_type_of(node), context, module) << "." << var_name;
            } else {
                stream << python_class_type(base_type_of(node), context, module);
            }
        }
    } else if (node->value.kind() != UNDEF_KIND) {
        stream << python_const_value(node->value, context, module);
    } else {
        stream << python_name(node);
    }
    return stream.str();
}

// Finds the object path
static std::string python_type_path(const ptree* node, std::string& path, ModuleContext* module) {
    std::string ret;
    if (node->super != nullptr) {
        ret += python_type_path(node->super, path, module);
    }
    ret += python_variable_name(node, node, module);
    if (node->kind == N_MODULE) {
        path = ret;
    }
    ret += ".";
    return ret;
}

static const ptree* python_find_module(const ptree* obj) {  // Finds the object module
    if (obj == nullptr || obj->kind == N_MODULE) {
        return obj;
    }
    return python_find_module(obj->super);
}

static std::string python_scoped_name(const ptree* obj) {
    if (obj == nullptr) {
        return "";
    }
    return python_scoped_name(obj->super) + "_" + obj->name;
}

static bool python_compare_modules(const ptree* obj_1, const ptree* obj_2) {
    return python_scoped_name(obj_1) == python_scoped_name(obj_2);
}

static void python_emit_docs(const ptree* node, ModuleContext* module) {
    for (auto ann : node->annotations) {
        if (ann->type != annotation_type_doc) {
            continue;
        }
        std::string_view input = ann->members->value.val.str();
        if (is_post_doc(ann)) {
            input.remove_prefix(2);
        }
        if (input[0] == '\\' || input[0] == '@') {
            input.remove_prefix(input.find(' '));
        }

        size_t pos = 0;
        *module << R"(""")" << endl;
        while ((pos = input.find('\n')) != std::string_view::npos) {
            auto line = input.substr(0, pos);
            input.remove_prefix(pos + 1);
            *module << line << endl;
        }
        *module << R"(""")" << endl;
    }
}

static std::string
python_class_type(const ptree* node, const ptree* context, ModuleContext* module) {
    std::stringstream str;
    if (node->super != nullptr &&
        (node->included_from != context->included_from ||
         !python_compare_modules(python_find_module(node), python_find_module(context))
        )) {  // Import module
        std::string path;
        str << python_type_path(node->super, path, module);
        if (!path.empty()) {
            std::string temp_file_name;
            if (node->included_from) {  // External Module
                temp_file_name = module_file_name(extract_file_name(node->included_from->name));
            } else {  // Internal Module
                temp_file_name = extract_file_name(module->file_name);
            }
            str << temp_file_name << ".";
            temp_file_name = "." + temp_file_name;
            module->imports.insert(path + temp_file_name);
        }
    }
    str << python_name(node);
    return str.str();
}

// Python type
static std::string
python_type_name(const ptree* node, const ptree* context, ModuleContext* module) {
    std::stringstream str;
    switch (node->kind) {
    case N_SEQUENCE: {
        module->imports.insert("__typing__");
        str << "_typing_.List[" << python_base_type(node->element_type, context, module) << "]";
        return str.str();
    }
    case N_MAP: {
        module->imports.insert("__typing__");
        str << "_typing_.Dict[" << python_base_type(node->key_type, context, module) << ", "
            << python_base_type(node->element_type, context, module) << "]";
        if (!node->bounds.empty()) {
        }
        return str.str();
    }
    case N_ARRAY:
        str << python_name(node->element_type);
        return str.str();
    case N_PRIMITIVE:
    case N_NATIVE:
    case N_FIXED:
    case N_STRING: {
        str << python_base_type(node, context, module);
        return str.str();
    }
    default:
        if (context->kind == N_ALIAS) {
            return python_class_type(node, context, module);
        }
        str << "'" << python_class_type(node, context, module) << "'";
        return str.str();
    }
}

static std::string
python_member_type_name(const ptree* node, ModuleContext* module, bool list_protection = false) {
    std::stringstream ret;
    if (list_protection) {
        const ptree* base = base_type_of(node);
        if (base->kind == N_ARRAY || base->kind == N_SEQUENCE) {
            return "list";
        }
        return python_base_type(base, node, module);
    }
    if (is_optional(node)) {  // Optional type
        module->imports.insert("__typing__");
        ret << "_typing_.Optional[";
    }
    if (base_type_of(node)->kind == N_ARRAY) {  // Array type
        std::string list_string;
        auto list_depth = unsigned(base_type_of(node)->bounds.size());
        if (list_depth) {
            module->imports.insert("__typing__");
        }
        for (unsigned i = 0; i < list_depth; ++i) {
            list_string += "_typing_.List[";
        }
        list_string += python_type_name(base_type_of(node)->element_type, node, module);
        for (unsigned i = 0; i < list_depth; ++i) {
            list_string += "]";
        }
        ret << list_string;
    } else {
        ret << python_type_name(base_type_of(node), node, module);
    }
    if (is_optional(node)) {  // Close optional bracket
        ret << "]";
    }
    return ret.str();
}

static std::string python_base_type(const ptree* obj, const ptree* context, ModuleContext* module) {
    const ptree* type_obj = base_type_of(obj);
    if (type_obj->kind == N_STRING) {
        return "str";
    }
    if (type_obj == &long_type) {
        return "int";
    }
    if (type_obj == &char_type) {
        return "bytes";
    }
    if (type_obj == &wchar_type) {
        return "str";
    }
    if (type_obj == &double_type) {
        return "float";
    }
    if (type_obj == &ldouble_type) {
        return "float";
    }
    if (type_obj == &float_type) {
        return "float";
    }
    if (type_obj == &short_type) {
        return "int";
    }
    if (type_obj == &ushort_type) {
        return "int";
    }
    if (type_obj == &int8_type) {
        return "int";
    }
    if (type_obj == &octet_type) {
        return "int";
    }
    if (type_obj == &ulong_type) {
        return "int";
    }
    if (type_obj == &ulonglong_type) {
        return "int";
    }
    if (type_obj == &longlong_type) {
        return "int";
    }
    if (type_obj == &boolean_type) {
        return "bool";
    }
    std::stringstream res;
    res << python_type_name(type_obj, context, module);
    return res.str();
}

// Default values
static std::string
python_default_type_value(const ptree* obj, const ptree* context, ModuleContext* module) {
    if (obj->kind == N_STRING) {
        return "\"\"";
    }
    if (obj == &long_type) {
        return "0";
    }
    if (obj == &char_type) {
        return R"('\0')";
    }
    if (obj == &wchar_type) {
        return "\'\'";
    }
    if (obj == &double_type) {
        return "0.0";
    }
    if (obj == &ldouble_type) {
        return "0.0";
    }
    if (obj == &float_type) {
        return "0.0";
    }
    if (obj == &short_type) {
        return "0";
    }
    if (obj == &ushort_type) {
        return "0";
    }
    if (obj == &int8_type) {
        return "0";
    }
    if (obj == &octet_type) {
        return "0";
    }
    if (obj == &ulong_type) {
        return "0";
    }
    if (obj == &ulonglong_type) {
        return "0";
    }
    if (obj == &longlong_type) {
        return "0";
    }
    if (obj == &boolean_type) {
        return "False";
    }
    if (obj->kind == N_STRUCT) {
        return python_class_type(obj, context, module) + "()";
    }
    if (obj->kind == N_UNION) {
        return python_class_type(obj, context, module) + "()";
    }
    if (obj->kind == N_ALIAS) {
        return python_default_type_value(base_type_of(obj), context, module);
    }
    if (obj->kind == N_ENUM) {
        if (obj->members) {
            std::stringstream stream;
            stream << python_class_type(obj, context, module) << "." << python_name(obj->members);
            return stream.str();
        }
        return "None";
    }
    if (obj->kind == N_BITMASK) {
        if (obj->members != nullptr) {
            return python_const_value(obj->members->value, context, module);
        }
        return "None";
    }
    if (obj->kind == N_ARRAY) {
        std::string data;
        if (obj->element_type != nullptr) {
            data = python_default_type_value(obj->element_type, context, module);
        } else {
            data = "[]";
        }
        for (auto i = obj->bounds.rbegin(); i != obj->bounds.rend(); ++i) {
            std::string ret;
            ret += "[";
            for (unsigned count = 0; count < unsigned_value(*i); ++count) {
                if (count != 0) {
                    ret += ", ";
                }
                ret += data;
            }
            ret += "]";
            data = ret;
        }
        return data;
    }
    if (obj->kind == N_ARRAY || obj->kind == N_SEQUENCE) {
        return "[]";
    }
    if (obj->kind == N_MAP) {
        return "{}";
    }
    return "None";
}

/// Looks at annotations and returns the default value for an object.
static std::string
python_default_value(const ptree* obj, const ptree* context, ModuleContext* module) {
    if (has_default_value(obj)) {
        const ptree* base_type = base_type_of(obj);
        const ptree* default_value = get_annotation(obj, annotation_type_default);
        if (base_type->kind == N_ENUM && default_value &&
            default_value->value.kind() == PTREE_KIND &&
            default_value->value.val.node()->kind == N_CONST) {
            return python_const_value(default_value->value, obj, module);
        }
        if (base_type->kind == N_ENUM) {
            return python_class_type(base_type, context, module) + "." +
                   python_name(get_default_value(obj).val.node());
        }
        if (base_type->kind == N_STRUCT || base_type->kind == N_UNION) {
            std::string value = python_class_type(base_type_of(obj), context, module) + "(";
            if (default_value) {
                const auto ann = default_value->members;
                for (const auto& el : ann->value.val.node()->members) {
                    if (el != ann->value.val.node()->members) {
                        value += ", ";
                    }
                    if (el->value.kind() == PTREE_KIND) {
                        const ptree* value_node = el->value.val.node();
                        if (base_type_of(value_node)->kind == N_PRIMITIVE ||
                            base_type_of(value_node)->kind == N_ENUM) {
                            value += python_name(el) + "=" +
                                     python_const_value(value_node->value, obj, module);
                        } else {
                            value += python_name(el) + "=" +
                                     python_class_type(base_type_of(value_node), obj, module) + "(";
                            for (const auto& internal_el : value_node->members) {
                                if (internal_el != value_node->members) {
                                    value += ", ";
                                }
                                value += python_name(internal_el) + "=" +
                                         python_const_value(internal_el->value, obj, module);
                            }
                            value += ")";
                        }
                    } else {
                        value += python_name(el) + "=" + python_const_value(el->value, obj, module);
                    }
                }
                value += ")";
                return value;
            }
        }
        auto def = get_default_value(obj);
        std::string const_value = python_const_value(def, context, module);
        return const_value;
    }
    if (base_type_of(obj)->kind == N_ENUM) {
        for (const auto& el : base_type_of(obj)->members) {
            if (get_annotation(el, annotation_type_default_literal)) {
                return python_class_type(base_type_of(obj), context, module) + "." +
                       python_name(el);
            }
        }
    }
    return python_default_type_value(base_type_of(obj), context, module);
}

static std::string
python_primitive_cast(const ptree* obj, const ptree* ctx, ModuleContext* module) {
    if (base_type_of(obj)->kind == N_ENUM) {
        return python_class_type(base_type_of(obj), ctx, module);
    }
    return python_base_type(obj, ctx, module);
}

static std::string python_discriminator_list(
    const ptree* ctx,
    ModuleContext* module,
    const std::vector<const ptree*>& segment
) {
    std::stringstream out_stream;
    for (const auto& seg : segment) {
        if (seg != segment.front()) {
            out_stream << ", ";
        }
        out_stream << python_primitive_cast(ctx->discriminator, ctx, module) << "("
                   << python_variable_name(seg, ctx, module) << ")";
    }
    return out_stream.str();
}

static std::string python_base_type_name(const ptree* obj) {
    const ptree* type_obj = base_type_of(obj);
    if (type_obj->kind == N_STRING) {
        return "string";
    }
    if (type_obj == &long_type) {
        return "long";
    }
    if (type_obj == &char_type) {
        return "char";
    }
    if (type_obj == &wchar_type) {
        return "wchar";
    }
    if (type_obj == &double_type) {
        return "double";
    }
    if (type_obj == &ldouble_type) {
        return "longdouble";
    }
    if (type_obj == &float_type) {
        return "float";
    }
    if (type_obj == &short_type) {
        return "short";
    }
    if (type_obj == &ushort_type) {
        return "ushort";
    }
    if (type_obj == &int8_type) {
        return "byte";
    }
    if (type_obj == &octet_type) {
        return "octet";
    }
    if (type_obj == &ulong_type) {
        return "ulong";
    }
    if (type_obj == &ulonglong_type) {
        return "ulonglong";
    }
    if (type_obj == &longlong_type) {
        return "longlong";
    }
    if (type_obj == &boolean_type) {
        return "boolean";
    }
    if (type_obj->kind == N_ARRAY || type_obj->kind == N_SEQUENCE) {
        return python_base_type_name(type_obj->element_type);
    }
    throw std::logic_error("Invalid state when trying to find the base name");
}

static std::string
python_const_value(const numeric& value, const ptree* context, ModuleContext* module) {
    std::stringstream out;

    switch (value.kind()) {
    case UNDEF_KIND:
        break;
    case BOOLEAN_KIND:
        out << (value.val.b() ? "True" : "False");
        break;
    case INT8_KIND:
        out << static_cast<short>(value.val.i8());
        break;
    case OCTET_KIND:
        out << static_cast<short>(value.val.o());
        break;
    case SHORT_KIND:
        out << value.val.s();
        break;
    case USHORT_KIND:
        out << value.val.us();
        break;
    case LONG_KIND:
        out << value.val.l();
        break;
    case ULONG_KIND:
        out << value.val.ul();
        break;
    case LONGLONG_KIND:
        out << value.val.ll();
        break;
    case ULONGLONG_KIND:
        out << value.val.ull();
        break;
    case FLOAT_KIND:
        out << value.val.f();
        break;
    case DOUBLE_KIND:
        out << value.val.d();
        break;
    case CHAR_KIND:
        out << "'" << (static_cast<char>(value.val.c())) << "'";
        break;
    case STRING_KIND:
        out << "\"" << value.val.str() << "\"";
        break;
    case PTREE_KIND:
        const ptree* node = value.val.node();
        if (node->kind == N_CONST && (node->flags & OPT_CONST_VALUE) != 0) {
            if (base_type_of(context)->kind == N_SEQUENCE ||
                base_type_of(context)->kind == N_ARRAY) {
                out << "[";
            }
            for (const auto& member : value.val.node()->members) {
                if (member != value.val.node()->members) {
                    out << ", ";
                }
                std::string const_value = python_const_value(member->value, context, module);
                out << const_value;
            }
            if (base_type_of(context)->kind == N_SEQUENCE ||
                base_type_of(context)->kind == N_ARRAY) {
                out << "]";
            }
        } else if (node->kind == N_CONST && base_type_of(node)->kind == N_ENUM) {
            out << python_variable_name(node, context, module);
        } else if (node->kind == N_CONST) {
            out << python_const_value(node->value, context, module);
        } else {
            out << idl_scoped_name(node, context);
        }
        break;
    }

    return out.str();
}

static std::vector<const ptree*> get_cases(const ptree* obj) {
    std::vector<const ptree*> cases;
    for (auto cas : obj->members) {
        cases.emplace_back(cas);
    }
    return cases;
}

static void code_gen_python_compound(const ptree* obj, ModuleContext* module, FileMap& module_map);
static void code_gen_python_full_type_check(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    const std::string& var_name
);

static void code_gen_python_enum_body(const ptree* obj, ModuleContext* module) {
    *module << begin("") << endl << tab_group;
    python_emit_docs(obj, module);
    if (obj->members == nullptr) {
        *module << "pass" << endl;
    }
    for (const auto& el : obj->members) {
        *module << python_name(el);

        if (obj->flags & OPT_ENUMERATED) {
            *module << " = " << python_const_value(el->value, obj, module);
        } else {
            if (el == obj->members) {
                *module << " = 0";
            } else {
                module->imports.insert("__auto__");
                *module << " = _auto_()";
            }
        }
        *module << endl;
        python_emit_docs(el, module);
    }
    *module << end("") << blank_line;
}

static std::string code_gen_python_deserialize_types(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    unsigned depth = 0
) {
    std::stringstream stream;
    const ptree* base_type = base_type_of(obj);
    std::string name =
        depth == 0 ? "_data[\"" + python_name(obj) + "\"]" : "__x" + std::to_string(depth);
    if (base_type->kind == N_SEQUENCE) {
        stream << "["
               << code_gen_python_deserialize_types(
                      base_type->element_type, context, module, depth + 1
                  )
               << " for __x" << depth + 1 << " in " << name << "]";
    } else if (base_type->kind == N_ARRAY) {
        for (unsigned i = 0; i < base_type->bounds.size(); i++) {
            stream << "[";
        }

        stream << code_gen_python_deserialize_types(
            base_type->element_type, context, module, depth + unsigned(base_type->bounds.size())
        );

        for (int i = int(base_type->bounds.size()) - 1; i >= 0; i--) {
            std::string scoped_name = depth + i == 0 ? "_data[\"" + python_name(obj) + "\"]"
                                                     : "__x" + std::to_string(depth + i);
            stream << " for __x" << depth + i + 1 << " " << "in " << scoped_name << "]";
        }
    } else if (base_type->kind == N_STRUCT || base_type->kind == N_ENUM ||
               base_type->kind == N_UNION) {
        stream << python_class_type(base_type, context, module) << ".deserialize_json(" << name
               << ")";
    } else {
        stream << name;
    }
    return stream.str();
}

static void code_gen_python_cdr(const ptree* obj, ModuleContext* module) {
    if (is_nested(obj)) {
        return;
    }
    *module << "@staticmethod" << endl;
    *module << "def cdr_type_definition():" << tab_group << begin("") << endl << tab_group;
    *module << "return bytearray([" << tab_group << begin("") << endl << tab_group;
    size_t cdr_size = 0;
    unsigned char* cdr = nullptr;
    // TODO(idarcar);
    // get_type_library(obj, &cdr, &cdr_size);

    for (size_t i = 0; i < cdr_size; i++) {
        if (i != 0) {
            *module << ", ";
            if ((i % 18) == 0) {
                *module << endl;
            }
        }
        *module << fmt::format("'0x{:02x}'", cdr[i]);
    }
    *module << endl << end("") << "])" << endl << end("");

    // free(cdr);
}

static std::string get_bit_bound_type(const ptree* obj) {
    auto bit_bound = get_annotation(obj, annotation_type_bit_bound);
    if (!bit_bound) {
        return "long";
    }
    unsigned long long size = bit_bound->value.val.ull();
    if (size == 0) {
        throw std::invalid_argument("Bit_bound cannot be 0");
    }
    if (size <= 8) {
        return "octet";
    }
    if (size <= 16) {
        return "ushort";
    }
    if (size <= 32) {
        return "ulong";
    }
    if (size <= 64) {
        return "ulonglong";
    }
    throw std::invalid_argument("Bit_bound cannot be greater than 64");
}

static void code_gen_python_read_cdr(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    const std::string& return_name = "",
    unsigned depth_value = 0
) {
    if (is_non_serialized(context)) {
        *module << endl;
        return;
    }
    if (is_optional(context) && depth_value == 0) {
        *module << python_name(context) << " = None" << endl;
        *module << "if ctx.read_boolean():" << tab_group << begin("") << endl << tab_group;
    }
    auto base_type = base_type_of(obj);
    if (base_type->kind == N_PRIMITIVE || base_type->kind == N_STRING) {
        *module << return_name << " = ctx.read_" << python_base_type_name(obj) << "()" << endl;
    } else if (base_type->kind == N_STRUCT || base_type->kind == N_UNION) {
        *module << return_name << " = " << python_class_type(obj, context, module)
                << ".deserialize_cdr(ctx)" << endl;
    } else if (base_type->kind == N_ENUM) {
        *module << return_name << " = " << python_class_type(obj, context, module) << "(ctx.read_"
                << get_bit_bound_type(base_type) << "())" << endl;
    } else if (base_type->kind == N_SEQUENCE) {
        *module << return_name << " = []" << endl;
        *module << "for _ in range(ctx.read_length()):" << tab_group << begin("") << endl
                << tab_group;
        std::string temp_name = "__x" + std::to_string(depth_value);
        code_gen_python_read_cdr(
            base_type_of(obj->element_type), context, module, temp_name, depth_value + 1
        );
        *module << return_name << ".append(" << temp_name << ")" << endl;
        *module << end("");
    } else if (base_type->kind == N_ARRAY) {
        for (unsigned i = 0; i < obj->bounds.size(); ++i) {
            *module << "__x" << depth_value + i + 1 << " = []" << endl;
            *module << "for _ in range(" << unsigned_value(obj->bounds[i]) << "): " << tab_group
                    << begin("") << endl
                    << tab_group;
        }

        std::string ret_name = "__x" + std::to_string(depth_value + obj->bounds.size() + 1);
        code_gen_python_read_cdr(
            base_type_of(obj->element_type),
            context,
            module,
            ret_name,
            depth_value + unsigned(obj->bounds.size()) + 2
        );
        *module << "__x" + std::to_string(depth_value + obj->bounds.size()) << ".append("
                << ret_name << ")" << endl;

        for (int i = int(obj->bounds.size()) - 1; i != 0; --i) {
            *module << end("");
            *module << "__x" << depth_value + i << ".append(__x" << depth_value + i + 1 << ")"
                    << endl;
        }
        *module << end("") << return_name << " = __x" << depth_value + 1 << endl;
    } else if (base_type->kind == N_MAP) {
        std::string key_name = "__key" + std::to_string(depth_value);
        std::string element_name = "__element" + std::to_string(depth_value);

        *module << return_name << " = {}" << endl;
        *module << "for _ in range(ctx.read_length()):" << tab_group << begin("") << endl
                << tab_group;
        code_gen_python_read_cdr(
            base_type_of(obj->key_type), context, module, key_name, depth_value + 1
        );
        code_gen_python_read_cdr(
            base_type_of(obj->element_type), context, module, element_name, depth_value + 1
        );
        *module << return_name << "[" << key_name << "] = " << element_name << endl << end("");
    } else if (base_type->kind == N_BITMASK) {
        *module << return_name << " = ctx.read_" << python_base_type_name(obj->element_type) << "()"
                << endl;
    }
    if (is_optional(context) && depth_value == 0) {
        *module << end("");
    }
}
static void code_gen_python_write_cdr(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    unsigned depth_value = 0
) {
    if (is_non_serialized(context)) {
        *module << endl;
        return;
    }
    std::string variable =
        (depth_value == 0 ? "self._" + python_name(context) : "__x" + std::to_string(depth_value));
    if (is_optional(context) && depth_value == 0)  // Start optional
    {
        *module << "ctx.write_boolean(self._" << python_name(context) << " is not None)" << endl;
        *module << "if self._" << python_name(context) << " is not None:" << tab_group << begin("")
                << endl
                << tab_group;
    }
    if (obj->kind == N_PRIMITIVE || obj->kind == N_STRING) {  // Primitive type
        *module << "ctx.write_" << python_base_type_name(obj) << "(" << variable << ")" << endl;
    } else if (obj->kind == N_STRUCT || obj->kind == N_UNION) {
        *module << variable << ".serialize_cdr(ctx)" << endl;
    } else if (obj->kind == N_ENUM) {
        *module << "ctx.write_" << get_bit_bound_type(base_type_of(obj)) << "(int(" << variable
                << "))" << endl;
    } else if (obj->kind == N_ARRAY) {
        auto list_depth = unsigned(obj->bounds.size());
        for (unsigned i = 0; i < list_depth; ++i) {
            *module << "for __x" << depth_value + i + 1 << " in "
                    << (i == 0 ? variable : "__x" + std::to_string(depth_value + i)) << ":"
                    << tab_group << begin("") << endl
                    << tab_group;
        }
        code_gen_python_write_cdr(base_type_of(obj->element_type), context, module, list_depth);
        for (unsigned i = 0; i < list_depth; ++i) {
            *module << end("");
        }
    } else if (obj->kind == N_SEQUENCE) {
        *module << "ctx.write_length(len(" << variable << "))" << endl;
        *module << "for __x" << depth_value + 1 << " in " << variable << ":" << begin("") << endl
                << tab_group;
        code_gen_python_write_cdr(
            base_type_of(obj->element_type), context, module, depth_value + 1
        );
        *module << end("");
    } else if (obj->kind == N_MAP) {
        std::string key_name = "__key" + std::to_string(depth_value);
        std::string element_name = "__element" + std::to_string(depth_value);
        *module << "ctx.write_length(len(" << variable << "))" << endl;
        *module << "for " << key_name << ", " << element_name << " in " << variable
                << ".items():" << begin("") << endl
                << tab_group;
        *module << "__x" << depth_value + 1 << " = " << key_name << endl;
        code_gen_python_write_cdr(base_type_of(obj->key_type), context, module, depth_value + 1);
        *module << "__x" << depth_value + 1 << " = " << element_name << endl;
        code_gen_python_write_cdr(
            base_type_of(obj->element_type), context, module, depth_value + 1
        );
        *module << end("");
    } else if (obj->kind == N_BITMASK) {
        *module << "ctx.write_" << python_base_type_name(obj->element_type) << "(" << variable
                << ")" << endl;
    }
    if (is_optional(context) && depth_value == 0) {  // Close optional
        *module << end("");
    }
}

static void code_gen_python_struct_operators(const ptree* obj, ModuleContext* module) {
    /// __eq__
    *module << "def __eq__(self, other: '" << python_name(obj) << "'):" << begin("") << endl
            << tab_group;
    *module << "if not isinstance(other, " << python_name(obj) << "):" << begin("") << endl
            << tab_group;
    *module << "return False" << end("") << endl;
    if (!obj->members) {
        *module << "return True" << endl;
    } else {
        *module << "return all([" << begin("") << endl << tab_group;
        for (const auto& el : obj->members) {
            *module << "self." << python_name(el) << " == other." << python_name(el) << "," << endl;
        }
        *module << end("") << "])";
    }
    *module << end("") << blank_line;

    /// __lt__
    *module << "def __lt__(self, other: '" << python_name(obj) << "'):" << begin("") << endl
            << tab_group;
    if (!obj->members) {
        *module << "return False" << endl;
    } else {
        for (const auto& el : obj->members) {
            *module << "if self." << python_name(el) << " < other." << python_name(el) << ":"
                    << begin("") << endl
                    << tab_group;
            *module << "return True" << end("") << endl;
            *module << "if self." << python_name(el) << " > other." << python_name(el) << ":"
                    << begin("") << endl
                    << tab_group;
            *module << "return False" << end("") << endl;
        }
        *module << "return False";
    }
    *module << end("") << blank_line;
}

static void code_gen_python_inherited_arguments(const ptree* obj, std::vector<const ptree*>& vec) {
    if (!obj->parents.empty()) {
        code_gen_python_inherited_arguments(obj->parents.front(), vec);
    }

    for (const auto& el : obj->members) {
        vec.push_back(el);
    }
}

static void code_gen_python_rec(const ptree* obj, ModuleContext* module, FileMap& module_map) {
    if (!is_emit(obj, LANG_PYTHON)) {
        return;
    }

    if (!module || module->file_base_name != extract_file_name(obj->included_from->name)) {
        std::string file_name = extract_file_name(obj->included_from->name);
        std::string top_module_name(obj->included_from->name);
        module = module_map[file_name][top_module_name];
        if (!module) {
            module = new ModuleContext(top_module_name, module_file_name(file_name), file_name);
            module_map[file_name][top_module_name] = module;
        }
        if (!module->pp.str().empty()) {
            *module << blank_line;
        }
    } else if (obj->kind == N_MODULE || obj->kind == N_STRUCT || obj->kind == N_UNION ||
               obj->kind == N_VALUETYPE || obj->kind == N_INTERFACE || obj->kind == N_ENUM ||
               obj->kind == N_BITSET || obj->kind == N_BITMASK || obj->kind == N_ANNOTATION_DEF ||
               obj->kind == N_EXCEPTION) {
        *module << blank_line;
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
        break;

    case N_MODULE: {
        std::string new_module_name =
            (module->name == obj->included_from->name ? "" : module->name) + python_name(obj) + "/";
        std::string file_name = extract_file_name(obj->included_from->name);

        ModuleContext* new_module;
        if (module_map.count(file_name) && module_map.at(file_name).count(new_module_name) > 0) {
            new_module = module_map.at(file_name).at(new_module_name);
            *new_module << blank_line;
        } else {
            PrettyPrinter pp;
            new_module =
                new ModuleContext(new_module_name, module_file_name(file_name), file_name, module);
            if (module_map.count(file_name) == 0) {
                module_map.insert({file_name, ModuleMap()});
            }
            module_map.at(file_name).insert({new_module_name, new_module});
        }
        for (const auto& el : obj->members) {
            code_gen_python_rec(el, new_module, module_map);
            if (el->next &&
                el->kind != el->next->kind)  // Add an extra blank line between different kinds
            {
                *module << blank_line << tab_group;
            }
        }
        break;
    }
    case N_STRUCT: {
        if (obj != nullptr && obj->flags & OPT_DECLARATION)  // Ignore declarations
        {
            break;
        }
        module->imports.insert("__intercom_types__");
        *module << endl << blank_line << "class " << python_name(obj) << "(";

        if (obj->parents.empty()) {
            *module << "intercom_dds.intercom_types.BaseStruct";
        } else {
            *module << python_class_type(obj->parents.front(), obj, module);
        }
        *module << "):" << tab_group << begin("") << endl << tab_group;
        python_emit_docs(obj, module);
        code_gen_python_compound(obj, module, module_map);
        break;
    }
    case N_UNION: {
        if (obj != nullptr && obj->flags & OPT_DECLARATION)  // Ignore declarations
        {
            return;
        }
        std::vector<const ptree*> default_segment;
        module->imports.insert("__intercom_types__");
        module->imports.insert("__typing__");
        *module << "class " << python_name(obj) << "(intercom_dds.intercom_types.BaseUnion):";
        *module << tab_group << begin("") << endl << tab_group;
        python_emit_docs(obj, module);

        /// Slots
        *module << "__slots__ = ";
        bool found_first = false;
        for (const auto& el : obj->members) {
            if (el->kind == N_MEMBER) {
                if (found_first) {
                    *module << ", ";
                }
                found_first = true;
                *module << "\"_" << python_name(el) << "\"";
            }
        }
        *module << endl << blank_line;

        /// Constructor
        *module << "def __init__(self, value=None, discriminator: "
                << python_member_type_name(obj->discriminator, module) << " = None):" << tab_group
                << begin("") << endl
                << tab_group;
        std::string default_discriminator;
        if (has_default_value(obj->discriminator)) {
            default_discriminator = python_default_value(obj->discriminator, obj, module);
        } else {
            default_discriminator = python_variable_name(obj->members->members, obj, module);
        }
        *module << "super().__init__(" << python_primitive_cast(obj->discriminator, obj, module)
                << "(discriminator) if discriminator is not None else " << default_discriminator
                << ")" << endl;
        for (const auto& el : obj->members) {
            if (el->kind == N_MEMBER) {
                *module << "self._" << python_name(el) << ": "
                        << python_member_type_name(el, module);
                *module << " = value if (_temp := self._discriminator";
                auto case_segment = get_cases(el);

                if (el->flags & OPT_DEFAULT) {
                    *module << " not in [None, ";
                    for (auto mem : obj->members) {
                        if ((mem->flags & OPT_DEFAULT) == 0) {
                            for (auto cas : mem->members) {
                                default_segment.push_back(cas);
                            }
                        }
                    }
                    *module << python_discriminator_list(obj, module, default_segment);
                } else {
                    *module << " in [" << python_discriminator_list(obj, module, case_segment);
                }
                *module << "]) and value is not None else ";
                if (base_type_of(el)->kind == N_PRIMITIVE) {
                    *module << python_base_type(base_type_of(el), obj, module) << "("
                            << python_default_value(el, el, module);
                    if (base_type_of(el) == &char_type) {
                        *module << ", 'utf-8'";
                    }
                    *module << ")";
                } else {
                    *module << python_default_value(el, el, module);
                }
                if ((base_type_of(el)->kind == N_STRUCT || base_type_of(el)->kind == N_UNION ||
                     is_optional(el)) &&
                    !has_default_value(el)) {
                    *module << " if _temp else None";
                }
                *module << endl;
                case_segment.clear();
            }
        }
        *module << end("") << blank_line;

        // Setter and getter
        for (const auto& el : obj->members) {
            if (el->kind == N_MEMBER) {
                bool default_member = has_default_case(el);
                auto case_segment = get_cases(el);

                /// PROPERTY
                *module << "@property" << endl;
                *module << "def " << python_name(el) << "(self):" << tab_group << begin("") << endl
                        << tab_group;
                python_emit_docs(el, module);
                if (default_member) {
                    *module << "if self._discriminator in ["
                            << python_discriminator_list(obj, module, default_segment);
                } else {
                    *module << "if self._discriminator not in ["
                            << python_discriminator_list(obj, module, case_segment);
                }
                *module << "]:" << tab_group << begin("") << endl << tab_group;
                module->imports.insert("__intercom_exceptions__");
                *module << "raise _except_.UnionInvalidLookupException" << end("") << endl;
                *module << "return self._" << python_name(el) << end("") << endl << blank_line;

                /// SETTER
                *module << "@" << python_name(el) << ".setter" << endl;
                *module << "def " << python_name(el) << "(self, value: _typing_.Union["
                        << python_member_type_name(el, module) << ", " << "_typing_.Tuple["
                        << python_member_type_name(el, module) << ", "
                        << python_member_type_name(obj->discriminator, module)
                        << "]]) -> None:" << tab_group << begin("") << endl
                        << tab_group;
                python_emit_docs(el, module);
                *module << "real_value = value" << endl << tab_group;
                *module << "if isinstance(value, tuple): real_value = value[0]" << endl
                        << tab_group;
                code_gen_python_full_type_check(el, obj, module, "real_value");
                *module << blank_line << tab_group;
                *module << "if isinstance(value, tuple):" << tab_group << begin("") << endl
                        << tab_group;
                *module << "if len(value) != 2:" << tab_group << begin("") << endl << tab_group;
                *module
                    << "raise ValueError(f\"Expecting 'value, discriminator', but got {len(value)} arguments.\")"
                    << end("") << endl;

                *module << "variable_value, disc = value" << endl;
                *module << "disc = " << python_primitive_cast(obj->discriminator, obj, module)
                        << "(disc)" << endl;

                if (default_member) {
                    *module << "if disc in ["
                            << python_discriminator_list(obj, module, default_segment);
                } else {
                    *module << "if disc not in ["
                            << python_discriminator_list(obj, module, case_segment);
                }
                *module << "]:" << tab_group << begin("") << endl << tab_group;
                module->imports.insert("__intercom_exceptions__");
                *module << "raise _except_.UnionInvalidDiscriminatorException" << end("") << endl;
                *module << "self._" << python_name(el) << " = variable_value" << endl;
                *module << "self._discriminator = disc" << end("") << endl;
                *module << "else:" << tab_group << begin("") << endl << tab_group;
                if (default_member && case_segment.size() == 1) {
                    if (case_segment.back()->value.kind() == UNDEF_KIND) {
                        *module
                            << "raise self.UnionException(\"No appropriate default discriminator was found\")"
                            << endl;
                    } else if (case_segment.back()->value.kind() == PTREE_KIND) {
                        *module << "self._" << python_name(el) << " = value" << endl;
                        *module << "self._discriminator = "
                                << python_variable_name(
                                       case_segment.back()->value.val.node(), obj, module
                                   );
                    } else {
                        *module << "self._" << python_name(el) << " = value" << endl;
                        *module << "self._discriminator = "
                                << python_variable_name(case_segment.back(), obj, module);
                    }
                } else {
                    *module << "self._" << python_name(el) << " = value" << endl;

                    if (default_member) {
                        *module << "if self._discriminator in ["
                                << python_discriminator_list(obj, module, default_segment)
                                << "]:" << tab_group << begin("") << endl
                                << tab_group << "self._discriminator = ";
                        for (auto it = case_segment.rbegin(); it != case_segment.rend(); ++it) {
                            if ((*it)->name == "default") {
                                *module << python_variable_name(*it, obj, module);
                                break;
                            }
                        }
                    } else {
                        *module << "if self._discriminator not in [";
                        bool first_element = true;
                        const ptree* default_seg_value = nullptr;

                        for (const ptree* seg : case_segment) {
                            if (seg->flags & OPT_DEFAULT) {
                                continue;
                            }
                            if (!first_element) {
                                *module << ", ";
                            }
                            first_element = false;
                            default_seg_value = seg;
                            *module << python_primitive_cast(obj->discriminator, obj, module) << "("
                                    << python_variable_name(seg, obj, module) << ")";
                        }
                        *module << "]:" << tab_group << begin("") << endl << tab_group;
                        *module << "self._discriminator = "
                                << python_primitive_cast(obj->discriminator, obj, module);
                        *module << "(" << python_variable_name(default_seg_value, obj, module)
                                << ")";
                    }
                    *module << end("");
                }

                *module << end("") << end("") << endl << blank_line;
            }
        }

        // Discriminator to type
        std::vector<const ptree*> members;
        std::string default_string;
        *module << "@staticmethod" << endl;
        *module << "def fetch_type(discriminator):" << tab_group << begin("") << endl << tab_group;
        for (const auto& el : obj->members) {
            std::string ret_name = el->kind == N_NULL ? "null" : python_name(el);
            auto case_segment = get_cases(el);

            if (el->flags & OPT_DEFAULT) {
                default_string = ret_name;
            } else {
                *module << "if discriminator in ["
                        << python_discriminator_list(obj, module, case_segment);
                *module << "]:" << tab_group << begin("") << endl << tab_group;
                *module << "return \"" << ret_name << "\"" << endl << end("");
            }
            if (el->kind != N_NULL) {
                members.push_back(el);
            }
            case_segment.clear();
        }
        if (!default_string.empty()) {
            *module << "return \"" << default_string << "\"" << endl;
        } else {
            *module << "pass" << endl;
        }
        *module << end("") << blank_line;

        // Serialiser
        *module << "def serialize_json(self):" << tab_group << begin("") << endl << tab_group;
        *module << "_ret = {\"_d\": self._discriminator}" << endl;
        *module << "_key = self.fetch_type(self._discriminator)" << endl;
        *module << "if _key != \"null\":" << tab_group << begin("") << endl << tab_group;
        *module << "_ret[_key] = getattr(self, _key)" << end("") << endl;
        *module << "return _ret" << endl << end("") << blank_line;

        // Deserialiser
        *module << "@classmethod" << endl;
        *module << "def deserialize_json(cls, _data):" << tab_group << begin("") << endl
                << tab_group;
        *module << "if _data is None:" << tab_group << begin("") << endl << tab_group;
        *module << "return None" << end("") << endl;
        if (base_type_of(obj->discriminator)->kind == N_ENUM) {
            *module << "_disc = "
                    << python_class_type(base_type_of(obj->discriminator), obj, module)
                    << "(_data[\"_d\"])" << endl;
        } else {
            *module << "_disc = _data[\"_d\"]" << endl;
        }
        *module << "_key = cls.fetch_type(_disc)" << endl;
        *module << "_value = None" << endl;
        for (const auto& el : members) {
            *module << "if _key == \"" << python_name(el) << "\":";
            *module << tab_group << begin("") << endl << tab_group;
            *module << "_value = " << code_gen_python_deserialize_types(el, obj, module) << endl
                    << end("");
        }
        *module << "return " << python_class_type(obj, obj, module)
                << "(value=_value, discriminator=_disc)" << endl;
        *module << end("") << blank_line;

        /// CDR
        {
            *module << "@staticmethod" << endl;
            *module << "def deserialize_cdr(ctx):" << tab_group << begin("") << endl << tab_group;
            *module << "value = None" << endl;
            code_gen_python_read_cdr(
                base_type_of(obj->discriminator), obj, module, "discriminator"
            );
            bool first_element = true;
            const ptree* default_element = nullptr;
            for (const auto& el : obj->members) {
                if (el->kind == N_MEMBER) {
                    auto case_segment = get_cases(el);
                    if (el->flags & OPT_DEFAULT) {
                        default_element = el;
                    } else {
                        if (!first_element) {
                            *module << "el";  // else if
                        } else {
                            first_element = false;
                        }
                        *module << "if discriminator in ["
                                << python_discriminator_list(obj, module, case_segment);
                        *module << "]:" << tab_group << begin("") << endl << tab_group;
                        code_gen_python_read_cdr(base_type_of(el), el, module, "value");
                        *module << end("");
                    }
                    members.push_back(el);
                    case_segment.clear();
                }
            }
            if (default_element) {
                if (!first_element) {
                    *module << "else:" << tab_group << begin("") << endl << tab_group;
                }
                code_gen_python_read_cdr(
                    base_type_of(default_element), default_element, module, "value"
                );
                if (!first_element) {
                    *module << end("");
                }
            }
            *module << "return " << python_name(obj) << "(value, discriminator)" << endl
                    << end("") << blank_line;
        }
        {
            *module << "def serialize_cdr(self, ctx):" << tab_group << begin("") << endl
                    << tab_group;
            auto base_discriminator = base_type_of(obj->discriminator);
            if (base_discriminator->kind == N_PRIMITIVE ||
                base_discriminator->kind == N_STRING) {  // Primitive type
                *module << "ctx.write_" << python_base_type_name(base_discriminator)
                        << "(self._discriminator)" << endl;
            } else if (base_discriminator->kind == N_ENUM) {
                *module << "ctx.write_long(int(self._discriminator))" << endl;
            } else {
                std::cerr << "Invalid discriminator state" << std::endl;
            }
            bool first_element = true;
            const ptree* default_element = nullptr;
            for (const auto& el : obj->members) {
                if (el->kind == N_MEMBER) {
                    if (el->flags & OPT_DEFAULT) {
                        default_element = el;
                    } else {
                        if (!first_element) {
                            *module << "el";  // else if
                        } else {
                            first_element = false;
                        }
                        auto case_segment = get_cases(el);
                        *module << "if self._discriminator in ["
                                << python_discriminator_list(obj, module, case_segment);
                        *module << "]:" << tab_group << begin("") << endl << tab_group;
                        code_gen_python_write_cdr(base_type_of(el), el, module);
                        *module << end("");
                    }
                    members.push_back(el);
                }
            }
            if (default_element) {
                if (!first_element) {
                    *module << "else:" << tab_group << begin("") << endl << tab_group;
                }
                code_gen_python_write_cdr(base_type_of(default_element), default_element, module);
                if (!first_element) {
                    *module << end("");
                }
            }
        }
        *module << end("") << blank_line;

        code_gen_python_cdr(obj, module);
        *module << blank_line << end("");
        break;
    }

    case N_VALUETYPE:
        *module << "class " << python_name(obj) << ": " << tab
                << "# Dummy ValueType";  //"(metaclass=abc.ABCMeta)";
        // code_gen_python_compound( obj, module, module_map );
        *module << tab_group << begin("") << endl << tab_group << "pass" << endl << end("");
        break;

    case N_INTERFACE:
        *module << "class " << python_name(obj) << ":" << tab << "# Dummy Interface";
        // code_gen_python_compound( obj, module, module_map );
        *module << tab_group << begin("") << endl << tab_group << "pass" << endl << end("");
        break;

    case N_BITMASK:
        module->imports.insert("__typing__");
        module->imports.insert("__intercom_types__");
        *module << "class " << python_name(obj) << "(intercom_dds.intercom_types.BaseBitMask):";
        *module << tab_group << begin("") << endl << tab_group;
        python_emit_docs(obj, module);
        if (obj->members == nullptr) {
            *module << "pass" << end("") << endl << blank_line;
            break;
        }
        for (const auto& el : obj->members) {
            *module << python_name(el)
                    << ": _typing_.Final = " << python_const_value(el->value, obj, module) << endl;
        }
        *module << end("") << endl << blank_line;
        break;

    case N_BITSET:
        *module << "class " << python_name(obj) << ":" << tab << "# Dummy BitSet";
        *module << tab_group << begin("") << endl << tab_group << "pass" << endl << end("");
        break;

    case N_ENUM:
        module->imports.insert("__intercom_types__");
        *module << "class " << python_name(obj) << "(intercom_dds.intercom_types.BaseEnum):";
        code_gen_python_enum_body(obj, module);
        break;

    case N_ANNOTATION_DEF:
        *module << "annotation " << python_name(obj);
        // code_gen_python_compound( obj, module, module_map );
        *module << tab_group << begin("") << endl << tab_group << "pass" << endl << end("");
        break;

    case N_EXCEPTION:
        *module << "exception " << python_name(obj);
        // code_gen_python_compound( obj, module, module_map );
        *module << tab_group << begin("") << endl << tab_group << "pass" << endl << end("");
        break;

    case N_ALIAS:
        *module << python_name(obj) << " = " << python_member_type_name(obj, module, true) << " "
                << tab << "# ALIAS" << endl;
        python_emit_docs(obj, module);
        break;

    case N_CONST: {
        module->imports.insert("__typing__");
        node_kind base = base_type_of(obj)->kind;
        *module << python_name(obj) << ": _typing_.Final[" << python_member_type_name(obj, module)
                << "] = ";
        bool encapsulated = false;
        if (base == N_ENUM || base == N_STRUCT || base == N_UNION) {
            *module << python_class_type(base_type_of(obj), obj, module) << "(";
            encapsulated = true;
        } else if (base != N_SEQUENCE && base != N_ARRAY) {
            *module << python_base_type(base_type_of(obj), obj, module) << "(";
            encapsulated = true;
        }
        *module << python_const_value(obj->value, obj, module);
        if (encapsulated) {
            if (base_type_of(obj) == &char_type) {
                *module << ", 'utf-8'";
            }
            *module << ") ";
        }
        *module << tab << "# CONST";
        break;
    }
    case N_NULL:
        *module << "None";
        break;

    case N_MEMBER: {
        /*if ( obj->super->kind == N_INTERFACE )*/
        /*if ( obj->super->kind == N_VALUETYPE )*/

        std::stringstream member_type;

        member_type << python_member_type_name(obj, module);

        if (obj->super->kind == N_STRUCT) {
            if (base_type_of(obj)->kind == N_STRUCT || base_type_of(obj)->kind == N_UNION) {
                *module << "if " << python_name(obj) << " is not None and not isinstance("
                        << python_name(obj) << ", "
                        << python_class_type(base_type_of(obj), obj, module) << "):" << tab_group
                        << begin("") << endl;
                *module << tab_group << "raise TypeError(f\"Expected None or an instance of {"
                        << python_class_type(base_type_of(obj), obj, module) << "}, but got {type("
                        << python_name(obj) << ")}\")" << end("") << endl;
            }
            if (is_optional(obj)) {
                *module << "self." << python_name(obj) << ": " << member_type.str() << " = "
                        << python_name(obj);
            } else {
                std::string default_value = python_default_value(obj, obj, module);
                if (default_value == "None") {
                    *module << "self." << python_name(obj) << ": " << member_type.str() << " = "
                            << python_name(obj);
                } else {
                    if (base_type_of(obj)->kind == N_PRIMITIVE) {
                        default_value = python_base_type(base_type_of(obj), obj, module) + "(" +
                                        default_value +
                                        (base_type_of(obj) == &char_type ? ", 'utf-8'" : "") + ")";
                    }
                    *module << "self." << python_name(obj) << ": " << member_type.str() << " = "
                            << default_value << " if " << python_name(obj) << " is None else "
                            << python_name(obj);
                }
            }
        } else {
            if (obj->annotations != nullptr && is_optional(obj)) {
                *module << python_name(obj) << ": Optional[" << member_type.str() << "]";
            } else {
                *module << python_name(obj) << ": " << member_type.str();
            }
        }

        break;
    }
        /*case N_PROTOTYPE:*/
    default:;
    }

    *module << endl;
}

static void code_gen_python_primitive_range_check(
    const ptree* obj,
    ModuleContext* module,
    const std::string& var_name
) {
    auto base_obj = base_type_of(obj);
    std::string min;
    std::string max;
    if (base_obj && base_obj->kind == N_PRIMITIVE) {
        switch (base_obj->value.kind()) {
        case INT8_KIND:
            min = std::to_string(INT8_MIN);
            max = std::to_string(INT8_MAX);
            break;
        case OCTET_KIND:
            min = std::to_string(0U);
            max = std::to_string(UCHAR_MAX);
            break;
        case SHORT_KIND:
            min = std::to_string(INT16_MIN);
            max = std::to_string(INT16_MAX);
            break;
        case USHORT_KIND:
            min = std::to_string(0U);
            max = std::to_string(UINT16_MAX);
            break;
        case LONG_KIND:
            min = std::to_string(INT32_MIN);
            max = std::to_string(INT32_MAX);
            break;
        case ULONG_KIND:
            min = std::to_string(0U);
            max = std::to_string(UINT32_MAX);
            break;
        case LONGLONG_KIND:
            min = std::to_string(INT64_MIN);
            max = std::to_string(INT64_MAX);
            break;
        case ULONGLONG_KIND:
            min = std::to_string(0U);
            max = std::to_string(UINT64_MAX);
            break;
        case FLOAT_KIND:
            min = "-3.402823466e+38";
            max = "3.402823466e+38";
            break;
        case DOUBLE_KIND:
            min = "-1.7976931348623158e+308";
            max = "1.7976931348623158e+308";
            break;
        case CHAR_KIND:
        case UNDEF_KIND:
        case BOOLEAN_KIND:
        case STRING_KIND:
        case PTREE_KIND:
            break;
        }
    }
    if (has_max_value(obj)) {
        max = python_const_value(get_max_value(obj), obj, module);
    }
    if (has_min_value(obj)) {
        min = python_const_value(get_min_value(obj), obj, module);
    }
    if (min.empty() && max.empty()) {
        return;
    }

    if (is_optional(obj)) {
        *module << "if " << var_name << " is not None:" << begin("") << endl << tab_group;
    }
    if (!min.empty()) {
        module->imports.insert("__intercom_exceptions__");
        *module << "if " << var_name << " < " << min << ": raise _except_.OutOfRangeException("
                << var_name << ", " << min << ", False)" << endl
                << tab_group;
    }
    if (!max.empty()) {
        module->imports.insert("__intercom_exceptions__");
        *module << "if " << var_name << " > " << max << ": raise _except_.OutOfRangeException("
                << var_name << ", " << max << ")" << endl
                << tab_group;
    }
    if (is_optional(obj)) {
        *module << end("") << endl << tab_group;
    }
}

/**
 * appends appendix to end of base, and compresses suffix if equal to appendix
 * e.g. squash_duplicate("value", "_e") => "value_e"
 * -||- squash_duplicate("value_e", "_e") => "value_e1"
 * -||- squash_duplicate("value_e1", "_e") => "value_e2"
 * etc.
 */
static std::string squash_duplicate_suffix(const std::string& base, const std::string& appendix) {
    size_t suffix_begin = base.rfind(appendix);
    if (suffix_begin == std::string::npos) {
        return base + appendix;
    }
    size_t i = base.length();
    while (i > 1 && ('0' <= base[i - 1] && base[i - 1] <= '9')) {
        i--;
    }
    int num = i == base.length() ? 0 : std::stoi(base.substr(i));
    std::string str_num = std::to_string(num + 1);
    return base.substr(0, suffix_begin) + appendix + str_num;
}

static void code_gen_python_check_castable_to(
    ModuleContext* module,
    const std::string& var_name,
    const std::string& type
) {
    *module << "try: " << begin("") << endl << tab_group;
    *module << var_name << " = " << type << '(' << var_name << ')' << end("") << endl << tab_group;
    *module << "except ValueError as e: raise TypeError(str(e))" << endl << tab_group;
};

static void code_gen_python_check_isinstance(
    ModuleContext* module,
    const std::string& var_name,
    const std::initializer_list<std::string>& types
) {
    if (types.size() == 0) {
        return;
    }
    *module << "if not isinstance(" << var_name << ", " << *types.begin() << ')';
    for (unsigned i = 1; i < types.size(); i++) {
        *module << " and not isinstance(" << var_name << ", " << types.begin()[i] << ')';
    }
    *module << ": raise TypeError(f\"Expected type " << *types.begin();
    if (types.size() > 1) {
        *module << " or similar";
    }
    *module << ", but got {type(" << var_name << ")}\")" << endl << tab_group;
};

static void code_gen_python_check_isinstance(
    ModuleContext* module,
    const std::string& var_name,
    const std::string& type
) {
    code_gen_python_check_isinstance(module, var_name, {type});
}

/**
 * checks that object is roughly of expected type
 * e.g. checks if sequence<int> is of type sequence,
 * but does not check if elements are of type int.
 *
 * @param obj
 * @param context
 * @param module
 * @param var_name
 */
static void code_gen_python_simple_type_check(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    const std::string& var_name
) {
    const ptree* base_type = base_type_of(obj);

    // isinstance
    if (base_type->kind == N_STRUCT || base_type->kind == N_UNION) {
        code_gen_python_check_isinstance(
            module, var_name, python_class_type(base_type, context, module)
        );
        return;
    }
    if (base_type->kind == N_SEQUENCE || base_type->kind == N_ARRAY) {
        code_gen_python_check_isinstance(module, var_name, "list");
        return;
    }
    if (base_type->kind == N_MAP) {
        code_gen_python_check_isinstance(module, var_name, "dict");
        return;
    }
    if (base_type->kind == N_STRING) {
        code_gen_python_check_isinstance(module, var_name, "str");
        return;
    }
    if (base_type->kind == N_ENUM) {
        // TODO
        *module << "pass  # TODO check enum type" << endl << tab_group;
        return;
    }
    if (base_type->kind == N_BITMASK) {
        // TODO
        *module << "pass  # TODO check bitmask type" << endl << tab_group;
        return;
    }

    if (base_type->kind == N_PRIMITIVE) {
        switch (base_type->value.kind()) {
        case BOOLEAN_KIND:
            code_gen_python_check_castable_to(module, var_name, "bool");
            break;
        case INT8_KIND:
        case OCTET_KIND:
        case SHORT_KIND:
        case USHORT_KIND:
        case LONG_KIND:
        case ULONG_KIND:
        case LONGLONG_KIND:
        case ULONGLONG_KIND:
            code_gen_python_check_castable_to(module, var_name, "int");
            break;
        case FLOAT_KIND:
        case DOUBLE_KIND:
            code_gen_python_check_castable_to(module, var_name, "float");
            break;
        case CHAR_KIND:
            code_gen_python_check_isinstance(module, var_name, {"str", "bytes"});
            *module << "if len(" << var_name << ") != 1: raise TypeError(\"Expected type char\")"
                    << endl
                    << tab_group;
            break;
        case STRING_KIND:
        case PTREE_KIND:
        case UNDEF_KIND:
            break;
        }
        return;
    }
    // should not get here
    *module << "pass  # TODO check unknown type" << endl << tab_group;
}

/**
 * checks type, bounds, and range (implicit and annotated)
 *
 * @param obj
 * @param context context of object
 * @param module
 * @param var_name python variable name related to obj
 */
static void code_gen_python_full_type_check(
    const ptree* obj,
    const ptree* context,
    ModuleContext* module,
    const std::string& var_name
) {
    const ptree* base_type = base_type_of(obj);

    if (is_optional(obj)) {
        *module << "if " << var_name << " is not None:" << begin("") << endl << tab_group;
    }

    if (base_type->kind != N_ARRAY) {  // avoid double check
        code_gen_python_simple_type_check(obj, context, module, var_name);

        if (!base_type->bounds.empty()) {
            auto bound = unsigned_value(base_type->bounds.back());
            module->imports.insert("__intercom_exceptions__");
            *module << "if len(" << var_name << ") > " << bound
                    << ": raise _except_.OutOfRangeException(" << var_name << ", " << bound << ")"
                    << endl
                    << tab_group;
        }
    }
    code_gen_python_primitive_range_check(obj, module, var_name);

    switch (base_type->kind) {
    case N_MAP: {
        std::string key_name = squash_duplicate_suffix(var_name, "_k");
        *module << "for " << key_name << " in " << var_name << ".keys():" << begin("") << endl
                << tab_group;
        code_gen_python_full_type_check(base_type->key_type, context, module, key_name);
        *module << end("") << endl << tab_group;
        std::string element_name = squash_duplicate_suffix(var_name, "_e");
        *module << "for " << element_name << " in " << var_name << ".values()";
        *module << ":" << begin("") << endl << tab_group;
        code_gen_python_full_type_check(base_type->element_type, context, module, element_name);
        *module << end("") << endl << tab_group;
        break;
    }
    case N_SEQUENCE: {
        std::string element_name = squash_duplicate_suffix(var_name, "_e");
        *module << "for " << element_name << " in " << var_name;
        *module << ":" << begin("") << endl << tab_group;
        code_gen_python_full_type_check(base_type->element_type, context, module, element_name);
        *module << end("") << endl << tab_group;
        break;
    }
    case N_ARRAY: {
        size_t i = 0;
        size_t size = base_type->bounds.size();
        std::string scope_name = var_name;
        module->imports.insert("__intercom_exceptions__");
        for (; i < size; ++i) {
            auto bound = unsigned_value(base_type->bounds[i]);
            code_gen_python_simple_type_check(obj, context, module, scope_name);
            *module << "if len(" << scope_name << ") != " << bound
                    << ": raise _except_.OutOfRangeException(len(" << scope_name << "), " << bound
                    << ')' << endl;
            std::string next_scope_name =
                squash_duplicate_suffix(scope_name, (i == size - 1) ? "_e" : "_a");
            *module << "for " << next_scope_name << " in " << scope_name << ":" << tab_group
                    << begin("") << endl
                    << tab_group;
            scope_name = next_scope_name;
        }
        code_gen_python_full_type_check(base_type->element_type, context, module, scope_name);
        for (; i; --i) {
            *module << end("");
        }
    }
    default:
        break;
    }
    if (is_optional(obj)) {
        *module << end("") << endl << tab_group;
    }
}

static void code_gen_python_getter_and_setter(const ptree* obj, ModuleContext* module) {
    if (obj == nullptr) {
        return;
    }
    // Getter
    *module << "@property" << endl;
    *module << "def " << python_name(obj) << "(self):" << tab_group << begin("") << endl
            << tab_group;
    python_emit_docs(obj, module);
    *module << "return self._" << python_name(obj) << endl << end("") << blank_line;

    // Setter
    *module << "@" << python_name(obj) << ".setter" << endl;
    *module << "def " << python_name(obj)
            << "(self, value: " << python_member_type_name(obj, module) << "):";
    *module << tab_group << begin("") << endl << tab_group;
    python_emit_docs(obj, module);
    code_gen_python_full_type_check(obj, obj, module, "value");
    *module << "self._" << python_name(obj) << " = value" << endl << end("") << blank_line;

    code_gen_python_getter_and_setter(obj->next, module);
}

static void code_gen_python_compound(const ptree* obj, ModuleContext* module, FileMap& module_map) {
    if (obj->flags & OPT_DECLARATION) {
        return;
    }
    if (obj->kind == N_STRUCT && obj->members != nullptr) {
        *module << "def __init__(" << tab_group << begin("") << endl
                << tab_group << "self," << endl;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                *module << python_name(el) << ": " << python_member_type_name(el, module)
                        << " = None," << endl;
            }
        }

        for (const auto& el : obj->members) {
            *module << python_name(el) << ": " << python_member_type_name(el, module) << " = None,"
                    << endl;
        }
        *module << end("") << endl;
        *module << "):" << tab_group << begin("") << endl << tab_group;

        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            *module << "super().__init__(";
            for (const auto& el : inherited_arguments) {
                if (el != inherited_arguments.front()) {
                    *module << ", ";
                }
                *module << python_name(el);
            }
            *module << ")" << endl;
        }
    }
    for (const auto& el : obj->members) {
        code_gen_python_rec(el, module, module_map);
    }

    if (obj->kind == N_STRUCT) {
        if (obj->members != nullptr) {
            *module << end("") << blank_line;
        }

        // Generate serialise and so on.
        code_gen_python_getter_and_setter(obj->members, module);

        // Generate serialiser
        *module << "def serialize_json(self):" << tab_group << begin("") << endl << tab_group;
        *module << "ret = {" << tab_group << begin("") << endl << tab_group;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                if (!is_optional(el)) {
                    *module << "\"" << python_name(el) << "\": self._" << python_name(el) << ","
                            << endl;
                }
            }
        }

        for (const auto& el : obj->members) {
            if (!is_optional(el)) {
                *module << "\"" << python_name(el) << "\": self._" << python_name(el) << ","
                        << endl;
            }
        }
        *module << end("") << "}" << endl;

        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                if (is_optional(el)) {
                    *module << "if self._" << python_name(el) << " is not None:" << tab_group
                            << begin("") << endl
                            << tab_group;
                    *module << "ret[\"" << python_name(el) << "\"] = self._" << python_name(el)
                            << end("") << endl;
                }
            }
        }
        for (const auto& el : obj->members) {
            if (is_optional(el)) {
                *module << "if self._" << python_name(el) << " is not None:" << tab_group
                        << begin("") << endl
                        << tab_group;
                *module << "ret[\"" << python_name(el) << "\"] = self._" << python_name(el)
                        << end("") << endl;
            }
        }
        *module << "return ret" << end("") << blank_line;

        // Generate deserialiser
        *module << "@classmethod" << endl;
        *module << "def deserialize_json(cls, _data):" << tab_group << begin("") << endl
                << tab_group;
        *module << "if _data is None:" << tab_group << begin("") << endl << tab_group;
        *module << "return None" << end("") << endl;
        *module << "return " << python_class_type(obj, obj, module) << "(" << tab_group << begin("")
                << endl
                << tab_group;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                *module << python_name(el) << "="
                        << code_gen_python_deserialize_types(el, obj, module);
                if (is_optional(el)) {
                    *module << " if \"" << python_name(el) << "\" in _data else None";
                }
                *module << "," << endl;
            }
        }
        for (const auto& el : obj->members) {
            *module << python_name(el) << "=" << code_gen_python_deserialize_types(el, obj, module);
            if (is_optional(el)) {
                *module << " if \"" << python_name(el) << "\" in _data else None";
            }
            *module << "," << endl;
        }
        *module << end("") << ")" << end("") << blank_line;

        /// CDR CONVERTER
        *module << "@staticmethod" << endl;
        *module << "def deserialize_cdr(ctx):" << tab_group << begin("") << endl << tab_group;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                code_gen_python_read_cdr(base_type_of(el), el, module, python_name(el));
            }
        }
        for (const auto& el : obj->members) {
            code_gen_python_read_cdr(base_type_of(el), el, module, python_name(el));
        }
        *module << "return " << python_name(obj) << "(" << tab_group << begin("") << endl
                << tab_group;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                *module << python_name(el) << "=" << python_name(el) << "," << endl;
            }
        }
        for (const auto& el : obj->members) {
            *module << python_name(el) << "=" << python_name(el) << "," << endl;
        }
        *module << end("") << ")" << endl;

        *module << end("") << blank_line << endl;

        *module << "def serialize_cdr(self, ctx):" << tab_group << begin("") << endl << tab_group;
        if (!obj->parents.empty()) {
            std::vector<const ptree*> inherited_arguments;
            code_gen_python_inherited_arguments(obj->parents.front(), inherited_arguments);
            for (const auto& el : inherited_arguments) {
                *module << "value = self._" << python_name(el) << endl;
                code_gen_python_full_type_check(el, obj, module, "value");
                code_gen_python_write_cdr(base_type_of(el), el, module);
            }
        }
        if (obj->members == nullptr) {
            *module << "pass" << endl;
        }
        for (const auto& el : obj->members) {
            *module << "value = self._" << python_name(el) << endl;
            code_gen_python_full_type_check(el, obj, module, "value");
            code_gen_python_write_cdr(base_type_of(el), el, module);
        }

        *module << end("") << blank_line << endl;

        code_gen_python_struct_operators(obj, module);

        code_gen_python_cdr(obj, module);
        *module << end("") << blank_line;
    }
}

static void code_gen_python_write(
    FileMap& file_map,
    const std::string& name,
    const std::string& filename,
    ic_list_t* list
) {
    if (file_map.find(filename) == file_map.end()) {
        std::cerr << "COULD NOT FIND FILE: " << name << std::endl;
        return;
    }

    ModuleMap module_map = file_map.at(filename);

    for (auto const& module : module_map) {
        if (!module.second->pp.has_text_content()) {
            continue;
        }
        std::string folderpath;

        if (module.first == name) {
            folderpath += extract_file_name(filename);

            if (!CommandLineOption::python_global_postfix()) {
                folderpath += "_global";
            } else {
                std::string postfix(CommandLineOption::python_global_postfix());
                if (postfix != " ") {
                    folderpath += postfix;
                }
            }
            folderpath += ".py";
        } else {
            std::string init_filepath = folderpath + module.first + "__init__.py";
            std::string output_name = extract_file_name(module.second->file_name);
            folderpath += module.first + output_name + ".py";

            std::string import_string = "from ." + output_name + " import *";
            std::ifstream read_file(init_filepath.c_str());
            bool needs_update = true;
            if (read_file.is_open()) {
                std::string line;
                while (std::getline(read_file, line)) {
                    if (line == import_string) {
                        needs_update = false;
                        break;
                    }
                }
                read_file.close();
            }

            if (needs_update) {
                std::stringstream data_stream;
                data_stream << "from ." << output_name << " import *" << std::endl;
                std::ofstream init_file;
                init_file.open(init_filepath.c_str(), std::ios::app);
                if (init_file.good()) {
                    init_file << data_stream.str();
                    init_file.close();
                } else {
                    ic_push_source(list, init_filepath.c_str(), data_stream.str().c_str());
                }
            }
        }

        std::stringstream file;
        ModuleContext* module_ctxt = module.second;

        // import packages before modules
        bool found = false;
        for (auto const& include : module_ctxt->imports) {
            if (include == "__typing__") {
                file << "import typing as _typing_";
            } else if (include == "__auto__") {
                file << "from enum import auto as _auto_";
            } else if (include == "__intercom_types__") {
                file << "import intercom_dds.intercom_types" << std::endl;
            } else if (include == "__intercom_exceptions__") {
                file << "import intercom_dds.core.exceptions as _except_";
            } else {
                continue;
            }
            file << std::endl;
            found = true;
        }
        if (found) {
            file << std::endl;
        }
        for (auto const& include : module_ctxt->imports) {
            if (include != "__typing__" && include != "__auto__" &&
                include != "__intercom_types__" && include != "__intercom_exceptions__") {
                file << "import " << include << std::endl;
            }
        }

        if (!module_ctxt->imports.empty()) {
            file << std::endl;
        }

        module_ctxt->pp.print(file);
        file << std::endl;
        ic_push_source(list, folderpath.c_str(), file.str().c_str());
    }
}

void intercom::cidl::code_gen_python(const parse_result* result, ic_list_t* list) {
    FileMap module_map;
    for (const auto& obj : result->tree) {
        code_gen_python_rec(obj, nullptr, module_map);
    }

    for (const auto& include : result->includes) {
        code_gen_python_write(module_map, include->name, extract_file_name(include->name), list);
    }

    // Clean up
    for (const auto& file : module_map) {
        for (const auto& el : file.second) {
            delete el.second;
        }
    }
}

extern "C" {

struct python_options_t {
    uint8_t use_pep8;
    const char* global_postfix;
};

void ic_codegen_python(const parse_result* result, python_options_t options, ic_list_t* list) {
    auto& config = CommandLineOption::get_instance();
    config.python_use_pep8 = options.use_pep8 != 0;
    config.python_global_postfix = options.global_postfix ? options.global_postfix : "";

    intercom::cidl::code_gen_python(result, list);
}
}
