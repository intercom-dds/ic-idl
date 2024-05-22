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

#include <cstring>
#include <iostream>
#include <map>
#include <set>
#include <sstream>
#include <unordered_set>
#include <vector>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/memf.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "utils/stdprintf.h"

using namespace intercom::cidl;

using ModuleMap = std::map<std::string, std::stringstream*>;

static void code_gen_cs_rec(const ptree* obj, ModuleMap& out);

static std::vector<const ptree*> struct_members(const ptree* obj, bool inherited = false);

static std::string cs_type(const ptree* obj, const ptree* module);

static std::string cs_value(const numeric& value, const ptree* module, int level);

static std::string
cs_value(const ptree* obj, const ptree* type, const ptree* module, int level = 0);

static std::string cs_init_value(const ptree* obj, const ptree* module);

static std::string cs_qualified_type(const ptree* obj, const ptree* module);

static void cs_emit_member_init(const ptree* member, const ptree* obj, ModuleMap& out);

static bool cs_type_is_nullable(const ptree* member);

static std::stringstream& module_stream(const ptree* obj, ModuleMap& out) {
    auto name = obj->included_from ? obj->included_from->name : "/Top level/";
    if (out.find(name) == out.end()) {
        auto* stream = new std::stringstream();
        *stream << "using System;" << std::endl;
        *stream << "using System.Collections.Generic;" << std::endl;
        out[name] = stream;
    }
    return *out[name];
}

// Defined by VS2010 header
#ifdef OUT
#  undef OUT
#endif
#define OUT module_stream(obj, out)

static std::map<std::string, std::string> g_name_mapper;

static std::string safe_name(std::string res_str) {
    for (auto& c : res_str) {
        if (c == '<' || c == '>' || c == ',' || c == ' ' || c == '[' || c == ']' || c == '.') {
            c = '_';
        }
    }
    return res_str;
}

static std::string public_member_name(const ptree* obj, const ptree* member) {
    auto mapIt = g_name_mapper.find(idl_scoped_name(member, nullptr));
    if (mapIt != g_name_mapper.end()) {
        return mapIt->second;
    }
    // Enum and bitmask labels can also be specified scoped inside their type
    if (member->kind == N_CONST && member->type &&
        (member->type->kind == N_ENUM || member->type->kind == N_BITMASK)) {
        std::string name =
            std::string(idl_scoped_name(member->type, nullptr)) + "::" + idl_name(member);
        mapIt = g_name_mapper.find(name);
        if (mapIt != g_name_mapper.end()) {
            return mapIt->second;
        }
    }

    std::stringstream res;

    if (CommandLineOption::no_rename()) {
        res << cs_name(member);
    } else {
        std::string name;
        switch (member->kind) {
        case N_SEQUENCE:
            name = "Sequence";
            break;
        case N_ARRAY:
            name = "Array";
            break;
        case N_STRING:
            name = "String";
            break;
        case N_MAP:
            name = "Map";
            break;
        default:
            name = cs_name(member);
        }
        int underscores = 0;
        bool dolower = true;
        bool first = true;
        for (std::string::iterator it = name.begin(); it != name.end(); ++it) {
            if (*it == '_') {
                ++underscores;
            } else {
                bool upcase = underscores > 0 || first;
                if (upcase) {
                    dolower = true;
                    std::string::iterator part = it;
                    while (part != name.end() && *part != '_' && dolower) {
                        dolower = toupper(*part) == *part;
                        ++part;
                    }
                }
                if (first) {
                    for (int i = 0; i < underscores; ++i) {
                        res << "_";
                    }
                }
                res << static_cast<char>((upcase ? toupper(*it) : (dolower ? tolower(*it) : *it)));
                underscores = 0;
                first = false;
            }
        }
        for (int i = 0; i < underscores; ++i) {
            res << "_";
        }
    }
    for (const auto& bound : member->bounds) {
        res << "_" << integer_value(bound);
    }
    if (obj && res.str() == obj->name) {
        res << "_";
    }
    return safe_name(res.str());
}

static std::string public_name(const ptree* obj) {
    return public_member_name(nullptr, obj);
}

static std::string private_member_name(const ptree* obj, const ptree* member) {
    std::string name = public_member_name(obj, member);
    if (name[0] != static_cast<char>(tolower(name[0])) && !CommandLineOption::no_rename()) {
        name[0] = static_cast<char>(tolower(name[0]));
    } else {
        name += "_";
    }
    return name;
}

static bool is_numeric(const ptree* obj) {
    const ptree* type_obj = base_type_of(obj);

    return type_obj == &boolean_type || type_obj == &float_type || type_obj == &double_type ||
           type_obj == &short_type || type_obj == &ushort_type || type_obj == &char_type ||
           type_obj == &octet_type || type_obj == &int8_type || type_obj == &long_type ||
           type_obj == &ulong_type || type_obj == &longlong_type || type_obj == &ulonglong_type;
}

static bool has_case_default(const ptree* obj) {
    for (auto elem : obj->members) {
        if (elem->kind == N_MEMBER) {
            for (auto cas : elem->members) {
                if (cas->flags & OPT_DEFAULT) {
                    return true;
                }
            }
        }
    }
    return false;
}

static bool is_case_default(const ptree* member, const ptree* obj) {
    for (auto elem : obj->members) {
        for (auto cas : elem->members) {
            if ((cas->flags & OPT_DEFAULT) != 0 && member == elem) {
                return true;
            }
        }
    }
    return false;
}

static std::vector<const ptree*> case_non_default_values(const ptree* obj) {
    std::vector<const ptree*> res;
    for (auto elem : obj->members) {
        std::vector<const ptree*> cases;
        for (auto cas : elem->members) {
            if (cas->flags & OPT_DEFAULT) {
                cases.clear();
            } else {
                cases.emplace_back(cas);
            }
        }
        res.insert(res.end(), cases.begin(), cases.end());
    }
    return res;
}

static bool is_union(const ptree* obj) {
    return obj->kind == N_UNION;
}

static bool has_parent(const ptree* obj) {
    return !obj->parents.empty();
}

static bool has_children(const ptree* obj) {
    return (obj->flags & OPT_HAS_CHILDREN) != 0;
}

static bool is_simple_struct(const ptree* obj) {
    if (is_union(obj) || has_parent(obj) || has_children(obj)) {
        return false;
    }
    for (auto elem : obj->members) {
        const ptree* type = base_type_of(elem);
        if (type->kind == N_ARRAY || type->kind == N_MAP || type->kind == N_SEQUENCE ||
            type->kind == N_STRUCT || type->kind == N_UNION) {
            return false;
        }
    }
    return true;
}

static std::string
cs_null_array_init_list(const ptree* type, const ptree* module, unsigned int depth = 0) {
    std::stringstream res;
    auto bounds = type->bounds;
    res << " {";
    for (int bound = integer_value(bounds[depth]); bound > 0; bound--) {
        if (depth + 1 < bounds.size()) {
            res << cs_null_array_init_list(type, module, depth + 1);
        } else {
            res << ' ' << cs_init_value(base_type_of(type->element_type), module);
        }
        if (bound != 1) {
            res << ',';
        }
    }
    res << " }";
    return res.str();
}

static std::string cs_array_value(const ptree* array, const ptree* module, int level) {
    std::stringstream res;
    res << " {";
    for (const ptree* elem : array->members) {
        if (elem != array->members) {
            res << ',';
        }
        if (elem->type->kind == N_ARRAY) {
            const ptree* sub_array = elem->value.val.node();
            if (!sub_array->name.empty() && sub_array->kind == N_CONST) {
                res << cs_qualified_type(sub_array, module);  // reference existing constant
            } else {
                res << cs_array_value(base_value_of(sub_array), module, level + 1);
            }
        } else {
            res << ' ' << cs_value(elem->value, module, level + 1);
        }
    }
    res << " }";
    return res.str();
}

/// \brief \verbatim new instance with given "numeric" value
static std::string cs_value(const numeric& value, const ptree* module, int level) {
    std::stringstream res;
    switch (value.kind()) {
    case UNDEF_KIND:
        break;
    case BOOLEAN_KIND:
        res << (integer_value(value) ? "true" : "false");
        break;
    case INT8_KIND:
    case OCTET_KIND:
        res << integer_value(value);
        break;
    case SHORT_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << static_cast<short>(integer_value(value));
        break;
    case USHORT_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << static_cast<unsigned short>(integer_value(value));
        break;
    case LONG_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << integer_value(value);
        break;
    case ULONG_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << static_cast<unsigned int>(integer_value(value)) << "u";
        break;
    case LONGLONG_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << long_long_value(value) << "L";
        break;
    case ULONGLONG_KIND:
        if (value.base == 16) {
            res << std::hex << "0x";
        }
        res << static_cast<unsigned long long>(long_long_value(value)) << "UL";
        break;
    case FLOAT_KIND:
        res << to_string(float_value(value)) << "f";
        break;
    case DOUBLE_KIND:
        res << to_string(double_value(value)) << "d";
        break;
    case CHAR_KIND:
        if (integer_value(value) < 'A' || integer_value(value) > 'z') {
            res << "'\\x" << std::hex << integer_value(value) << "'";
        } else {
            res << fmt::format("'{:c}'", static_cast<char>(integer_value(value)));
        }
        break;
    case STRING_KIND:
        res << "\"" << string_value(value) << "\"";
        break;
    case PTREE_KIND:
        res << cs_value(value.val.node(), value.val.node()->type, module, level + 1);
        break;
    }
    return res.str();
}

/// \brief new instance of obj with values defined by obj->value and obj->member(->next)->value
static std::string cs_value(const ptree* obj, const ptree* type, const ptree* module, int level) {
    std::stringstream res;
    const ptree* type_obj = base_type_of(type);
    const bool derives_from_annotation = obj->super && obj->super->kind == N_ANNOTATION;
    const ptree* obj_members = obj->members;
    const ptree* type_members = type->members;
    // get members from value, if obj does not have any members directly
    if (!obj->members && obj->value.kind() == PTREE_KIND && obj->value.val.node()) {
        obj_members = obj->value.val.node()->members;
        if (obj->value.val.node()->type) {
            type_members = obj->value.val.node()->type->members;
        }
    }

    if (type_obj->kind == N_ENUM || type_obj->kind == N_BITMASK) {
        const ptree* elem;
        for (elem = type_obj->members;
             elem && integer_value(elem->value) != integer_value(obj->value);
             elem = elem->next) {
        }
        if (elem) {
            res << cs_qualified_type(type_obj, module) << "." << public_member_name(type_obj, elem);
        } else {
            res << cs_value(obj, type_obj->element_type, module);
        }
    } else if (obj->kind == N_CONST && !obj->name.empty() && level > 0 &&
               !derives_from_annotation) {
        // this (else if) is skipped if obj derives from an annotation, so cs_init_value() can reuse
        // cs_value()'s logic for complex types
        res << cs_qualified_type(obj, module);
    } else if (type_obj->kind == N_STRING) {
        res << "\"" << string_value(obj->value) << "\"";
    } else if (type_obj->kind == N_STRUCT) {
        res << "new " << cs_type(type_obj, module) << " { ";
        for (const ptree *elem = obj_members, *tpelem = type_members; elem && tpelem;
             tpelem = tpelem->next, elem = elem->next) {
            res << public_member_name(module, tpelem) << " = ";
            res << cs_value(elem, tpelem, module, level + 1);
            if (elem->next && tpelem->next) {
                res << ", ";
            }
        }
        res << " }";
    } else if (type_obj->kind == N_SEQUENCE) {
        res << "new List<" << cs_type(type_obj->element_type, module) << "> { ";
        for (auto elem : obj_members) {
            res << cs_value(elem, type_obj->element_type, module, level + 1);
            if (elem->next) {
                res << ", ";
            }
        }
        res << " }";
    } else if (type_obj->kind == N_ARRAY) {
        res << "new " << cs_type(type_obj->element_type, module) << "[";
        for (size_t i = 0; i < type_obj->bounds.size(); ++i) {
            if (i > 0) {
                res << ",";
            }
            res << integer_value(type_obj->bounds[i]);
        }
        res << "]";
        res << cs_array_value(base_value_of(obj), module, level);
    } else {
        res << cs_value(obj->value, module, level + 1);
    }
    return res.str();
}

/// \brief \verbatim new instance of obj with values defined by obj's default annotation (@default)
static std::string cs_init_value(const ptree* obj, const ptree* module) {
    if (is_optional(obj)) {
        return "null";
    }
    std::stringstream res;
    const ptree* type_obj = base_type_of(obj);
    numeric default_value = get_default_value(obj);
    switch (type_obj->kind) {
    case N_ENUM:
    case N_BITMASK:
        if (default_value.kind() == PTREE_KIND) {
            res << cs_qualified_type(type_obj, module) << "."
                << public_member_name(type_obj, default_value.val.node());
        } else {
            res << cs_init_value(type_obj->element_type, module);
        }
        break;
    case N_STRING:
        if (default_value.kind() == STRING_KIND) {
            res << cs_value(default_value, module, 0);
        } else {
            res << "string.Empty";
        }
        break;
    case N_STRUCT:
    case N_UNION:
        if (default_value.kind() == UNDEF_KIND) {
            res << "new " << cs_type(obj, module) << "()";
        } else {
            res << cs_value(default_value, module, 0);
        }
        break;
    case N_SEQUENCE:
        if (default_value.kind() == UNDEF_KIND) {
            res << "new List<" << cs_type(type_obj->element_type, module) << ">()";
        } else {
            res << cs_value(default_value, module, 0);
        }
        break;
    case N_MAP:
        res << "new Dictionary<" << cs_type(type_obj->key_type, module) << ", "
            << cs_type(type_obj->element_type, module) << ">()";
        break;
    case N_ARRAY:
        if (default_value.kind() == UNDEF_KIND) {
            int subarray_count = 0;
            const ptree* element_type = base_type_of(type_obj->element_type);
            while (element_type->kind == N_ARRAY) {
                element_type = base_type_of(element_type->element_type);
                ++subarray_count;
            }
            res << "new " << cs_type(element_type, module) << "[";
            for (size_t i = 0; i < type_obj->bounds.size(); ++i) {
                if (i > 0) {
                    res << ",";
                }
                res << integer_value(type_obj->bounds[i]);
            }
            res << "]";
            for (int i = 0; i < subarray_count; ++i) {
                res << "[]";
            }
            res << cs_null_array_init_list(type_obj, module);
        } else {
            res << cs_value(default_value, module, 0);
        }
        break;
    default:
        res << cs_value(default_value, module, 0);
    }
    return res.str();
}

static std::string cs_type(const ptree* obj, const ptree* module) {
    std::stringstream res;
    const ptree* type_obj = base_type_of(obj);
    if (type_obj->kind == N_STRING) {
        res << "string";
    } else if (type_obj == &long_type) {
        res << "int";
    } else if (type_obj == &char_type) {
        res << "char";
    } else if (type_obj == &wchar_type) {
        res << "char";
    } else if (type_obj == &double_type) {
        res << "double";
    } else if (type_obj == &ldouble_type) {
        res << "double";
    } else if (type_obj == &float_type) {
        res << "float";
    } else if (type_obj == &short_type) {
        res << "short";
    } else if (type_obj == &ushort_type) {
        res << "ushort";
    } else if (type_obj == &int8_type) {
        res << "sbyte";
    } else if (type_obj == &octet_type) {
        res << "byte";
    } else if (type_obj == &ulong_type) {
        res << "uint";
    } else if (type_obj == &ulonglong_type) {
        res << "ulong";
    } else if (type_obj == &longlong_type) {
        res << "long";
    } else if (type_obj == &boolean_type) {
        res << "bool";
    } else if (type_obj->kind == N_SEQUENCE) {
        res << "IList<" << cs_type(type_obj->element_type, module) << ">";
    } else if (type_obj->kind == N_MAP) {
        res << "IDictionary<" << cs_type(type_obj->key_type, module) << ", "
            << cs_type(type_obj->element_type, module) << ">";
    } else if (type_obj->kind == N_ARRAY) {
        res << cs_type(type_obj->element_type, module) << "[";
        for (size_t i = 0; i < type_obj->bounds.size() - 1; ++i) {
            res << ",";
        }
        res << "]";
    } else {
        res << cs_qualified_type(type_obj, module);
    }
    return res.str();
}

static std::string cs_qualified_type(const ptree* obj, const ptree* module) {
    std::stringstream res;
    const ptree* common_parent = nullptr;
    for (const ptree* p1 = obj; p1 && !common_parent; p1 = p1->super) {
        for (const ptree* p2 = module; p2 && !common_parent; p2 = p2->super) {
            if (p1->kind == N_MODULE && p1 == p2) {
                common_parent = p1;
            }
        }
    }
    std::vector<std::string> mod_names;
    mod_names.push_back(public_name(obj));
    if (obj->kind == N_CONST) {
        mod_names.emplace_back("Constants");
    }
    for (const ptree* p2 = obj; p2 && p2 != common_parent; p2 = p2->super) {
        if (p2->kind == N_MODULE) {
            if (idl_scoped_name(p2, nullptr) == "DDS") {
                mod_names.emplace_back("DotNet");
                mod_names.emplace_back("Intercom");
            } else {
                mod_names.push_back(public_name(p2));
            }
        }
    }
    for (auto it = mod_names.rbegin(); it != mod_names.rend(); ++it) {
        if (it != mod_names.rbegin()) {
            res << ".";
        }
        res << *it;
    }
    return res.str();
}

static std::vector<const ptree*> struct_members(const ptree* obj, bool inherited) {
    std::vector<const ptree*> res;
    if (!obj->parents.empty() && inherited) {
        res = struct_members(obj->parents[0], inherited);
    }
    for (auto member : obj->members) {
        if (member->kind == N_MEMBER) {
            res.push_back(member);
        }
    }
    return res;
}

static void cs_emit_typesupport_methods(const ptree* obj, ModuleMap& out) {
    if (is_nested(obj) || CommandLineOption::intercom_build()) {
        return;
    }
    OUT << "// CDR serialized type DDS::XTypes::TypeDefinition" << std::endl;
    OUT << "private static readonly byte[] m_TypeDefinition = new byte[] {" << std::endl;
    // TODO: idarcar
    size_t cdrSize = 0;
    unsigned char* cdr = nullptr;
    // get_type_library(obj, &cdr, &cdrSize);
    for (size_t i = 0; i < cdrSize; i++) {
        if (i != 0) {
            OUT << ", ";
            if ((i % 24) == 0) {
                OUT << std::endl;
            }
        }
        char buf[16];
        snprintf(buf, sizeof(buf), "0x%02x", cdr[i]);
        OUT << buf;
    }
    OUT << std::endl << "};" << std::endl;

    free(cdr);

    OUT << "static " << public_name(obj) << "()" << std::endl;
    OUT << "{" << std::endl;
    OUT << "Intercom.DotNet.TypeRegistry.Register<" << public_name(obj)
        << ">(m_TypeDefinition, Read, Write);" << std::endl;
    OUT << "}" << std::endl;
}

static void cs_emit_enum(const ptree* obj, ModuleMap& out) {
    if (is_bitmask(obj)) {
        OUT << "[Flags]" << std::endl;
        OUT << "public enum " << public_name(obj);
        if (base_type_of(obj)->element_type != &long_type) {
            OUT << ": " << cs_type(base_type_of(obj)->element_type, nullptr);
        }
        OUT << std::endl;
        OUT << "{" << std::endl;
        for (auto elem : obj->members) {
            OUT << public_member_name(obj, elem) << " = " << cs_value(elem->value, obj, 0);
            if (elem->next) {
                OUT << ",";
            }
            OUT << std::endl;
        }
        OUT << "}" << std::endl;
    } else {
        OUT << "public enum " << public_name(obj);
        if (base_type_of(obj)->element_type != &long_type) {
            OUT << ": " << cs_type(base_type_of(obj)->element_type, nullptr);
        }
        OUT << std::endl;
        OUT << "{" << std::endl;
        for (auto elem : obj->members) {
            OUT << public_member_name(obj, elem);
            if (obj->flags & OPT_ENUMERATED) {
                OUT << " = " << integer_value(elem->value);
            }
            if (elem->next) {
                OUT << ",";
            }
            OUT << std::endl;
        }
        OUT << "}" << std::endl;
    }
}

static std::string cs_member_flags(const ptree* member) {
    std::vector<std::string> flags;
    if (is_key_member(member)) {
        flags.emplace_back("Intercom.DotNet.MemberFlag.IsKey");
    }
    if (is_optional(member)) {
        flags.emplace_back("Intercom.DotNet.MemberFlag.IsOptional");
    }
    if (is_shared(member)) {
        flags.emplace_back("Intercom.DotNet.MemberFlag.IsShareable");
    }
    if (is_must_understand(member)) {
        flags.emplace_back("Intercom.DotNet.MemberFlag.IsMustUnderstand");
    }
    if (flags.empty()) {
        flags.emplace_back("0");
    }
    std::stringstream res;
    for (size_t i = 0; i < flags.size(); ++i) {
        if (i != 0) {
            res << "|";
        }
        res << flags[i];
    }
    return res.str();
}

static std::string cs_type_flags(const ptree* member) {
    int kind = get_extensibility(member);
    switch (kind) {
    case FINAL_EXTENSIBILITY:
        return "Intercom.DotNet.TypeFlag.IsFinal";
    case MUTABLE_EXTENSIBILITY:
        return "Intercom.DotNet.TypeFlag.IsMutable";
    default:
        return "0";
    }
}

static void cs_emit_bounds_check(
    const ptree* member,
    const ptree* obj,
    const std::string& name,
    bool append_count,
    ModuleMap& out
) {
    int limit = 0;
    std::string cmpName = name;
    const ptree* type = base_type_of(member);
    if (!type->bounds.empty()) {
        limit = integer_value(type->bounds[0]);
        if (type->kind == N_STRING) {
            if (append_count) {
                cmpName += ".Length";
            }
        } else if (type->kind == N_SEQUENCE) {
            if (append_count) {
                cmpName += ".Count";
            }
        }
        if (type->kind == N_MAP) {
            if (append_count) {
                cmpName += ".Count";
            }
        }
    }

    if (limit > 0) {
        OUT << "if ( " << cmpName << " > " << limit << " )" << std::endl;
        OUT << "{" << std::endl;
        OUT << "throw new ArgumentOutOfRangeException(\"Length of " << public_name(member)
            << " out of range\");" << std::endl;
        OUT << "}" << std::endl;
    }
}

static void cs_emit_ctx_write(
    const ptree* member,
    const ptree* obj,
    const std::string& name,
    int& max_member_id,
    ModuleMap& out
) {
    static char iter_char = 'i';
    std::stringstream array_str;

    std::string seqname;
    if (name == public_member_name(obj, member)) {
        int memberId = get_member_id(member, obj, max_member_id);
        if (memberId > max_member_id) {
            max_member_id = memberId;
        }
        std::string memberName = (name == "discriminator") ? name : cs_name(member);
        OUT << "ctx.BeginProperty(" << memberId << ", \"" << memberName << "\", "
            << cs_member_flags(member) << ");" << std::endl;
        seqname = "element";
    } else {
        seqname = safe_name(name) + "_";
    }
    if (is_optional(member)) {
        if (cs_init_value(member, obj) == "null") {
            OUT << "ctx.Write(" << name << " != null);" << std::endl;
            OUT << "if ( " << name << " != null )" << std::endl;
            OUT << "{" << std::endl;
        } else {
            OUT << "ctx.Write( true );" << std::endl;
        }
    }
    const ptree* type = base_type_of(member);
    switch (type->kind) {
    case N_SEQUENCE:
        OUT << "if ( " << name << " != null )" << std::endl;
        OUT << "{" << std::endl;
        cs_emit_bounds_check(member, obj, name, true, out);
        if (base_type_of(type->element_type)->kind == N_PRIMITIVE) {
            std::string elementTypeName;
            if (base_type_of(type->element_type) == &wchar_type) {
                elementTypeName = "Utf16";
            } else if (base_type_of(type->element_type) == &char_type) {
                elementTypeName = "Utf8";
            }
            OUT << "ctx.Write" << elementTypeName << "(" << name << ");" << std::endl;
        } else if (base_type_of(type->element_type)->kind == N_STRING) {
            if (base_type_of(type->element_type)->element_type == &wchar_type) {
                OUT << "ctx.WriteUtf16(" << name << ");" << std::endl;
            } else if (base_type_of(type->element_type)->element_type == &char_type) {
                OUT << "ctx.WriteUtf8(" << name << ");" << std::endl;
            }
        } else {
            OUT << "ctx.WriteLength(" << name << ".Count);" << std::endl;
            OUT << "foreach (var " << seqname << " in " << name << ")" << std::endl;
            OUT << "{" << std::endl;
            cs_emit_ctx_write(type->element_type, obj, seqname, max_member_id, out);
            OUT << "}" << std::endl;
        }
        OUT << "}" << std::endl;
        OUT << "else" << std::endl;
        OUT << "{" << std::endl;
        OUT << "ctx.WriteLength(0);" << std::endl;
        OUT << "}" << std::endl;
        break;
    case N_MAP:
        OUT << "if ( " << name << " != null )" << std::endl;
        OUT << "{" << std::endl;
        cs_emit_bounds_check(member, obj, name, true, out);
        OUT << "ctx.WriteLength(" << name << ".Count);" << std::endl;
        OUT << "foreach (var " << seqname << " in " << name << ")" << std::endl;
        OUT << "{" << std::endl;
        cs_emit_ctx_write(type->key_type, obj, seqname + ".Key", max_member_id, out);
        cs_emit_ctx_write(type->element_type, obj, seqname + ".Value", max_member_id, out);
        OUT << "}" << std::endl;
        OUT << "}" << std::endl;
        OUT << "else" << std::endl;
        OUT << "{" << std::endl;
        OUT << "ctx.WriteLength(0);" << std::endl;
        OUT << "}" << std::endl;
        break;
    case N_ARRAY:
        array_str << "[";
        for (size_t i = 0; i < type->bounds.size(); ++i) {
            if (i > 0) {
                array_str << ",";
            }
            OUT << "for ( var " << iter_char << " = 0; " << iter_char << " < "
                << integer_value(type->bounds[i]) << "; " << iter_char << "++ )" << std::endl;
            OUT << "{" << std::endl;
            array_str << iter_char++;
        }
        array_str << "]";
        cs_emit_ctx_write(type->element_type, obj, name + array_str.str(), max_member_id, out);
        for (size_t i = 0; i < type->bounds.size(); ++i) {
            OUT << "}" << std::endl;
        }
        iter_char -= type->bounds.size();
        break;
    case N_ENUM:
    case N_BITMASK:
        OUT << "ctx.Write(";
        OUT << "(" << cs_type(type->element_type, nullptr) << ")" << name << ");" << std::endl;
        break;
    case N_STRUCT:
        OUT << name;
        if (cs_type_is_nullable(member)) {
            OUT << ".Value";
        }
        OUT << ".Write(ctx);" << std::endl;
        break;
    case N_UNION:
        OUT << name;
        if (cs_type_is_nullable(member)) {
            OUT << ".Value";
        }
        OUT << ".Write(ctx);" << std::endl;
        break;
    case N_STRING:
        cs_emit_bounds_check(member, obj, name, true, out);
        if (is_wstring(type)) {
            OUT << "ctx.WriteUtf16(" << name << ");" << std::endl;
        } else {
            OUT << "ctx.WriteUtf8(" << name << ");" << std::endl;
        }
        break;
    default:
        if (type == &wchar_type) {
            OUT << "ctx.WriteUtf16(" << name << ");" << std::endl;
        } else {
            if (cs_type_is_nullable(member)) {
                OUT << "ctx.Write(" << name << ".Value);" << std::endl;
            } else {
                OUT << "ctx.Write(" << name << ");" << std::endl;
            }
        }
    }
    if (is_optional(member) && cs_init_value(member, obj) == "null") {
        OUT << "}" << std::endl;
    }
    if (name == public_member_name(obj, member)) {
        OUT << "ctx.EndProperty();" << std::endl;
    }
}

static std::string cs_ctx_read_value(const ptree* member, const ptree* obj) {
    std::stringstream res;
    switch (base_type_of(member)->kind) {
    case N_ARRAY:
    case N_SEQUENCE:
    case N_MAP:
        res << "read" << public_member_name(obj, member) << "(ctx)";
        break;
    case N_ENUM:
    case N_BITMASK: {
        std::string type = cs_type(base_type_of(member)->element_type, obj->super);
        type[0] = static_cast<char>(toupper(type[0]));
        res << "(" << cs_type(member, obj->super) << ")ctx.Read" << type << "()";
    } break;
    case N_STRUCT:
        res << "new " << cs_type(member, obj->super) << "(ctx)";
        break;
    case N_UNION:
        res << "new " << cs_type(member, obj->super) << "(ctx)";
        break;
    case N_STRING:
        if (!base_type_of(member)->bounds.empty()) {
            res << "read" << public_member_name(obj, member) << "(ctx)";
        } else if (is_wstring(base_type_of(member))) {
            res << "ctx.ReadUtf16String()";
        } else {
            res << "ctx.ReadUtf8String()";
        }
        break;
    default: {
        std::string type = cs_type(member, obj->super);
        type[0] = static_cast<char>(toupper(type[0]));
        if (base_type_of(member) == &wchar_type) {
            type = std::string("Utf16") + type;
        } else if (base_type_of(member) == &char_type) {
            type = std::string("Utf8") + type;
        }
        res << "ctx.Read" << type << "()";
        break;
    }
    }
    return res.str();
}

static void cs_emit_ctx_read(
    const ptree* member,
    const ptree* obj,
    const std::string& name,
    int& max_member_id,
    ModuleMap& out
) {
    int memberId = get_member_id(member, obj, max_member_id);
    if (memberId > max_member_id) {
        max_member_id = memberId;
    }
    std::string memberName = (name == "discriminator") ? name : cs_name(member);
    OUT << "ctx.BeginProperty(" << memberId << ", \"" << memberName << "\", "
        << cs_member_flags(member) << ");" << std::endl;
    if (is_optional(member)) {
        OUT << "if ( ctx.ReadBool() )" << std::endl;
        OUT << "{" << std::endl;
    }
    OUT << name << " = " << cs_ctx_read_value(member, obj) << ";" << std::endl;
    if (is_optional(member)) {
        OUT << "}" << std::endl;
        OUT << "else" << std::endl;
        OUT << "{" << std::endl;
        OUT << name << " = " << cs_init_value(member, obj) << ";" << std::endl;
        OUT << "}" << std::endl;
    }
    OUT << "ctx.EndProperty();" << std::endl;
}

static void cs_emit_ctx_read_sequence(
    const ptree* member,
    const ptree* obj,
    ModuleMap& out,
    std::set<std::string>& emitted
) {
    std::string memberName = public_member_name(obj, member);
    if (emitted.find(memberName) != emitted.end()) {
        return;
    }
    emitted.insert(memberName);
    const ptree* type = base_type_of(member);
    // Emit reader helpers for map, sequence, array and bounded string
    if (type->kind == N_MAP || type->kind == N_SEQUENCE || type->kind == N_ARRAY ||
        (type->kind == N_STRING && !type->bounds.empty())) {
        cs_emit_ctx_read_sequence(type->element_type, obj, out, emitted);
        OUT << "private static " << cs_type(member, obj->super) << " read" << memberName;
        OUT << "(Intercom.DotNet.IReadContext ctx)" << std::endl;
        OUT << "{" << std::endl;
        if (type->kind == N_MAP) {
            cs_emit_ctx_read_sequence(type->element_type, obj, out, emitted);
            OUT << "var value = " << cs_init_value(base_type_of(member), obj->super) << ";"
                << std::endl;
            OUT << "var count = ctx.ReadLength();" << std::endl;
            cs_emit_bounds_check(member, obj, "count", false, out);
            OUT << "for ( var i = 0; i < count; i++ )" << std::endl;
            OUT << "{" << std::endl;
            OUT << "value.Add(" << cs_ctx_read_value(type->key_type, obj) << ", "
                << cs_ctx_read_value(type->element_type, obj) << ");" << std::endl;
            OUT << "}" << std::endl;
        } else if (type->kind == N_SEQUENCE) {
            if (base_type_of(type->element_type)->kind == N_PRIMITIVE) {
                std::string elementTypeName = cs_type(base_type_of(type->element_type), obj->super);
                elementTypeName[0] = static_cast<char>(toupper(elementTypeName[0]));
                if (base_type_of(type->element_type) == &wchar_type) {
                    elementTypeName = std::string("Utf16") + elementTypeName;
                } else if (base_type_of(type->element_type) == &char_type) {
                    elementTypeName = std::string("Utf8") + elementTypeName;
                }
                OUT << "var value = ctx.Read" << elementTypeName << "List();" << std::endl;
                cs_emit_bounds_check(member, obj, "value", true, out);
            } else if (base_type_of(type->element_type)->kind == N_STRING) {
                if (base_type_of(type->element_type)->element_type == &wchar_type) {
                    OUT << "var value = ctx.ReadUtf16StringList();" << std::endl;
                } else if (base_type_of(type->element_type)->element_type == &char_type) {
                    OUT << "var value = ctx.ReadUtf8StringList();" << std::endl;
                }
                cs_emit_bounds_check(member, obj, "value", true, out);
            } else {
                OUT << "var count = ctx.ReadLength();" << std::endl;
                OUT << "var value = new List<"
                    << cs_type(base_type_of(member)->element_type, obj->super) << ">(count);"
                    << std::endl;
                cs_emit_bounds_check(member, obj, "count", false, out);
                OUT << "for ( var i = 0; i < count; i++ )" << std::endl;
                OUT << "{" << std::endl;
                OUT << "value.Add(" << cs_ctx_read_value(type->element_type, obj) << ");"
                    << std::endl;
                OUT << "}" << std::endl;
            }
        } else if (type->kind == N_STRING) {
            if (is_wstring(type)) {
                OUT << "var value = ctx.ReadUtf16String();" << std::endl;
            } else {
                OUT << "var value = ctx.ReadUtf8String();" << std::endl;
            }
            cs_emit_bounds_check(member, obj, "value", true, out);
        } else {
            OUT << "var value = " << cs_init_value(base_type_of(member), obj->super) << ";"
                << std::endl;
            char iter_char = 'i';
            std::stringstream array_str;
            array_str << "value[";
            for (size_t i = 0; i < type->bounds.size(); ++i) {
                if (i > 0) {
                    array_str << ",";
                }
                OUT << "for ( var " << iter_char << " = 0; " << iter_char << " < "
                    << integer_value(type->bounds[i]) << "; " << iter_char << "++ )" << std::endl;
                OUT << "{" << std::endl;
                array_str << iter_char++;
            }
            array_str << "]";
            OUT << array_str.str() << " = " << cs_ctx_read_value(type->element_type, obj) << ";"
                << std::endl;
            for (size_t i = 0; i < type->bounds.size(); ++i) {
                OUT << "}" << std::endl;
            }
        }
        OUT << "return value;" << std::endl;
        OUT << "}" << std::endl;
    }
}

static void cs_emit_write_method(const ptree* obj, ModuleMap& out) {
    int maxMemberId = -1;
    OUT << "public";
    if (!obj->parents.empty()) {
        OUT << " new";
    }
    OUT << " void Write(Intercom.DotNet.IWriteContext ctx)" << std::endl;
    OUT << "{" << std::endl;
    OUT << "ctx.BeginType(typeof(" << public_name(obj) << "), " << cs_type_flags(obj) << ");"
        << std::endl;
    std::vector<const ptree*> members = struct_members(obj, true);
    if (obj->discriminator) {
        cs_emit_ctx_write(obj->discriminator, obj, "discriminator", maxMemberId, out);
        OUT << "switch ( discriminator )" << std::endl;
        OUT << "{" << std::endl;
        for (auto& it : members) {
            const ptree* member = it;
            if (!is_case_default(member, obj)) {
                for (auto cas : member->members) {
                    OUT << "case " << cs_value(cas, obj->discriminator, obj) << ":" << std::endl;
                }
                cs_emit_ctx_write(it, obj, public_member_name(obj, it), maxMemberId, out);
                OUT << "break;" << std::endl;
            }
        }
        for (auto& it : members) {
            const ptree* member = it;
            if (is_case_default(member, obj)) {
                OUT << "default:" << std::endl;
                cs_emit_ctx_write(it, obj, public_member_name(obj, it), maxMemberId, out);
                OUT << "break;" << std::endl;
            }
        }
        OUT << "}" << std::endl;
    } else {
        for (auto& member : members) {
            if (is_non_serialized(member)) {
                continue;
            }
            cs_emit_ctx_write(member, obj, public_member_name(obj, member), maxMemberId, out);
        }
    }
    OUT << "ctx.EndType();" << std::endl;
    OUT << "}" << std::endl;
    OUT << "public static void Write( " << public_name(obj)
        << " value, Intercom.DotNet.IWriteContext ctx)" << std::endl;
    OUT << "{" << std::endl;
    OUT << "value.Write(ctx);" << std::endl;
    OUT << "}" << std::endl;
};

static void cs_emit_read_method(const ptree* obj, ModuleMap& out) {
    int maxMemberId = -1;
    OUT << "public " << public_name(obj) << "(Intercom.DotNet.IReadContext ctx)";
    OUT << std::endl;
    OUT << "{" << std::endl;
    OUT << "ctx.BeginType(typeof(" << public_name(obj) << "), " << cs_type_flags(obj) << ");"
        << std::endl;
    std::vector<const ptree*> members = struct_members(obj, true);
    if (obj->discriminator) {
        cs_emit_ctx_read(obj->discriminator, obj, "discriminator", maxMemberId, out);
        OUT << "switch ( discriminator )" << std::endl;
        OUT << "{" << std::endl;
        for (auto& it : members) {
            const ptree* member = it;
            if (!is_case_default(member, obj)) {
                for (auto cas : member->members) {
                    OUT << "case " << cs_value(cas, obj->discriminator, obj) << ":" << std::endl;
                }
                cs_emit_ctx_read(it, obj, public_member_name(obj, it), maxMemberId, out);
                OUT << "break;" << std::endl;
            }
        }
        for (auto& member : members) {
            if (is_non_serialized(member)) {
                continue;
            }
            if (is_case_default(member, obj)) {
                OUT << "default:" << std::endl;
                cs_emit_ctx_read(member, obj, public_member_name(obj, member), maxMemberId, out);
                OUT << "break;" << std::endl;
            }
        }
        OUT << "}" << std::endl;
    } else {
        for (auto& member : members) {
            cs_emit_ctx_read(member, obj, public_member_name(obj, member), maxMemberId, out);
        }
    }
    OUT << "ctx.EndType();" << std::endl;
    OUT << "}" << std::endl;
    std::set<std::string> emitted;
    for (auto& member : members) {
        cs_emit_ctx_read_sequence(member, obj, out, emitted);
    }
    OUT << "public";
    if (!obj->parents.empty()) {
        OUT << " new";
    }
    OUT << " static " << public_name(obj) << " Read(Intercom.DotNet.IReadContext ctx)" << std::endl;
    OUT << "{" << std::endl;
    OUT << "return new " << public_name(obj) << "(ctx);" << std::endl;
    OUT << "}" << std::endl;
};

static void cs_emit_module(const ptree* obj, ModuleMap& out) {
    OUT << "namespace " << public_name(obj);
    while (obj->members->kind == N_MODULE && !obj->members->next) {
        obj = obj->members;
        OUT << "." << public_name(obj);
    }
    OUT << " {" << std::endl;
    code_gen_cs_rec(obj->members, out);
    OUT << "}" << std::endl;
}

static bool cs_emit_const(const ptree* obj, ModuleMap& out) {
    if (obj->flags & OPT_DECLARATION) {
        return false;
    }
    std::map<std::string, std::vector<const ptree*>> constantMap;
    for (auto elem : obj) {
        if (is_emit(elem, LANG_CS) && elem->kind == N_CONST) {
            constantMap[elem->file_name].push_back(elem);
        }
    }
    for (auto& it : constantMap) {
        obj = it.second[0];
        OUT << "public static partial class Constants" << std::endl;
        OUT << "{" << std::endl;
        for (auto& pit : it.second) {
            obj = pit;
            OUT << "public";
            OUT << (is_numeric(obj->type) ? " const " : " static readonly ");
            OUT << cs_type(obj->type, obj->super) << " " << public_name(obj) << " = "
                << cs_value(obj->value, obj->super, 0) << ";" << std::endl;
        }
        OUT << "}" << std::endl;
    }
    return !constantMap.empty();
}

static ModuleMap& cs_emit_range_check(const ptree* obj, const ptree* member, ModuleMap& out) {
    if (has_min_value(member) || has_max_value(member)) {
        if (cs_type_is_nullable(member)) {
            OUT << "if ( value.HasValue )" << std::endl;
            OUT << "{" << std::endl;
        }
        const char* prefix = "if (";
        if (has_min_value(member)) {
            OUT << prefix << " value < " << cs_value(get_min_value(member), obj, 0);
            prefix = " || ";
        }
        if (has_max_value(member)) {
            OUT << prefix << " value > " << cs_value(get_max_value(member), obj, 0);
        }
        OUT << " )" << std::endl;
        OUT << "{" << std::endl;
        OUT << "throw new InvalidOperationException(\"Attempt to set value out of range for class member "
            << public_name(obj) << "::" << public_name(member) << "\");" << std::endl;
        OUT << "}" << std::endl;
        if (cs_type_is_nullable(member)) {
            OUT << "}" << std::endl;
        }
    }
    return out;
}

static void cs_emit_struct(const ptree* obj, ModuleMap& out) {
    OUT << "public struct " << public_name(obj) << " : Intercom.DotNet.IWriteable" << std::endl;
    OUT << "{" << std::endl;

    std::vector<const ptree*> members = struct_members(obj);

    for (auto member : members) {
        OUT << "public " << cs_type(member, obj->super) << " " << public_member_name(obj, member)
            << " { get; private set; }" << std::endl;
    }
    OUT << std::endl;

    OUT << "public " << public_name(obj) << "( ";
    for (auto it = members.begin(); it != members.end(); ++it) {
        const ptree* member = *it;
        if (it != members.begin()) {
            OUT << ", ";
        }
        OUT << cs_type(member, obj->super) << " " << private_member_name(obj, member);
    }
    OUT << " ) : this()" << std::endl;
    OUT << "{" << std::endl;
    for (auto member : members) {
        OUT << public_member_name(obj, member) << " = " << private_member_name(obj, member) << ";"
            << std::endl;
    }
    OUT << "}" << std::endl;

    cs_emit_read_method(obj, out);
    cs_emit_write_method(obj, out);
    cs_emit_typesupport_methods(obj, out);

    OUT << "}" << std::endl;
}

static void cs_emit_class(const ptree* obj, ModuleMap& out) {
    std::vector<const ptree*> members = struct_members(obj);
    std::vector<const ptree*> allMembers = struct_members(obj, true);

    const ptree* parent = nullptr;
    if (!obj->parents.empty()) {
        parent = base_type_of(obj->parents[0]);
    }
    std::vector<const ptree*> parentMembers;

    OUT << "public partial class " << public_name(obj) << " : ";
    if (parent) {
        parentMembers = struct_members(parent, true);
        OUT << public_name(parent) << " , ";
    }
    OUT << "Intercom.DotNet.IWriteable" << std::endl;
    OUT << "{" << std::endl;

    if (CommandLineOption::generate_default_literals()) {
        std::unordered_set<std::string> local_member_names;
        for (const auto& member : members) {
            local_member_names.insert(public_member_name(obj, member));
            if (has_min_value(member) || has_max_value(member)) {
                local_member_names.insert(private_member_name(obj, member));
            }
        }
        for (const auto& member : members) {
            std::string var_name = std::string("Default") + public_member_name(obj, member);
            // prepend '_' until no name conflicts
            while (local_member_names.find(var_name) != local_member_names.end()) {
                var_name.insert(var_name.begin(), '_');
            }
            const char* type_suffix = cs_type_is_nullable(member) ? "? " : " ";
            OUT << "public static readonly " << cs_type(member, obj->super) << type_suffix
                << var_name << " = " << cs_init_value(member, obj->super) << ';' << std::endl;
        }
        OUT << std::endl;
    }

    // Properties
    for (auto member : members) {
        const char* type_suffix = cs_type_is_nullable(member) ? "? " : " ";
        if (has_min_value(member) || has_max_value(member)) {
            OUT << "private " << cs_type(member, obj->super) << type_suffix
                << private_member_name(obj, member) << ";" << std::endl;
            OUT << "public " << cs_type(member, obj->super) << type_suffix
                << public_member_name(obj, member) << std::endl;
            OUT << "{" << std::endl;
            OUT << "get" << std::endl;
            OUT << "{" << std::endl;
            OUT << "return " << private_member_name(obj, member) << ";" << std::endl;
            OUT << "}" << std::endl;
            OUT << "set" << std::endl;
            OUT << "{" << std::endl;
            cs_emit_range_check(obj, member, out);
            OUT << private_member_name(obj, member) << " = value;" << std::endl;
            OUT << "}" << std::endl;
            OUT << "}" << std::endl;
        } else {
            OUT << "public " << cs_type(member, obj->super) << type_suffix
                << public_member_name(obj, member) << " { get; set; }" << std::endl;
        }
    }
    OUT << std::endl;

    // Default constructor
    OUT << "public " << public_name(obj) << "()";
    if (parent) {
        OUT << " : base()";
    }
    OUT << std::endl;
    OUT << "{" << std::endl;
    for (auto& member : members) {
        cs_emit_member_init(member, obj, out);
    }
    OUT << "}" << std::endl;

    cs_emit_read_method(obj, out);
    cs_emit_write_method(obj, out);
    cs_emit_typesupport_methods(obj, out);

    OUT << "}" << std::endl;
}

static std::string
cs_case_if_expr(const ptree* obj, const ptree* cases, const std::string& name, bool is_or_test) {
    std::string sep = is_or_test ? " || " : " && ";
    std::string eq = is_or_test ? " == " : " != ";
    std::stringstream res;
    for (auto cas : cases) {
        if (cas != cases) {
            res << sep;
        }
        res << name << eq << cs_value(cas, obj->discriminator, obj->super);
    }
    return res.str();
}

static std::string cs_case_if_expr(
    const ptree* obj,
    const std::vector<const ptree*>& cases,
    const std::string& name,
    bool is_or_test
) {
    std::string sep = is_or_test ? " || " : " && ";
    std::string eq = is_or_test ? " == " : " != ";
    std::stringstream res;
    for (auto case_it = cases.begin(); case_it != cases.end(); ++case_it) {
        if (case_it != cases.begin()) {
            res << sep;
        }
        res << name << eq << cs_value(*case_it, obj->discriminator, obj->super);
    }
    return res.str();
}

static void cs_emit_union(const ptree* obj, ModuleMap& out) {
    OUT << "public class " << public_name(obj) << " : Intercom.DotNet.IWriteable" << std::endl;
    OUT << "{" << std::endl;

    std::vector<const ptree*> members = struct_members(obj);

    OUT << "private " << cs_type(obj->discriminator, obj->super) << " discriminator;" << std::endl;
    for (auto member : members) {
        const char* type_suffix = cs_type_is_nullable(member) ? "? " : " ";
        OUT << "private " << cs_type(member, obj->super) << type_suffix
            << private_member_name(obj, member) << ";" << std::endl;
    }
    OUT << std::endl;
    OUT << "public " << cs_type(obj->discriminator, obj->super) << " Discriminator" << std::endl;
    OUT << "{" << std::endl;
    OUT << "get { return discriminator; }" << std::endl;
    OUT << "set" << std::endl;
    OUT << "{" << std::endl;
    OUT << "if ( discriminator != value )" << std::endl;
    OUT << "{" << std::endl;
    for (auto it = members.begin(); it != members.end(); ++it) {
        const ptree* member = *it;
        if (it != members.begin()) {
            OUT << "else ";
        }
        if (is_case_default(member, obj)) {
            std::vector<const ptree*> case_non_default = case_non_default_values(obj);
            OUT << "if ( " << cs_case_if_expr(obj, case_non_default, "discriminator", false) << " )"
                << std::endl;
            OUT << "{" << std::endl;
            OUT << "if ( " << cs_case_if_expr(obj, case_non_default, "value", true) << " )"
                << std::endl;
            OUT << "{" << std::endl;
            OUT << "throw new InvalidOperationException(\"Attempt to set illegal default discriminator value to union "
                << public_name(obj) << "\");" << std::endl;
            OUT << "}" << std::endl;
            OUT << "}" << std::endl;
        } else {
            OUT << "if ( " << cs_case_if_expr(obj, member->members, "discriminator", true) << " )"
                << std::endl;
            OUT << "{" << std::endl;
            if (member->members->next) {
                OUT << "if ( " << cs_case_if_expr(obj, member->members, "value", false) << " )"
                    << std::endl;
                OUT << "{" << std::endl;
            }
            OUT << "throw new InvalidOperationException(\"Attempt to set illegal discriminator value to union "
                << public_name(obj) << "\");" << std::endl;
            OUT << "}" << std::endl;
            if (member->members->next) {
                OUT << "}" << std::endl;
            }
        }
    }
    OUT << "discriminator = value;" << std::endl;
    OUT << "}" << std::endl;
    OUT << "}" << std::endl;
    OUT << "}" << std::endl;
    for (auto member : members) {
        const char* type_suffix = cs_type_is_nullable(member) ? "? " : " ";
        OUT << "public " << cs_type(member, obj->super) << type_suffix
            << public_member_name(obj, member) << std::endl;
        OUT << "{" << std::endl;
        OUT << "get" << std::endl;
        OUT << "{" << std::endl;
        if (is_case_default(member, obj)) {
            std::vector<const ptree*> case_non_default = case_non_default_values(obj);
            OUT << "if ( " << cs_case_if_expr(obj, member->members, "discriminator", true) << " )"
                << std::endl;
        } else {
            OUT << "if ( " << cs_case_if_expr(obj, member->members, "discriminator", false) << " )"
                << std::endl;
        }
        OUT << "{" << std::endl;
        OUT << "throw new InvalidOperationException(\"Attempt to read wrong value type from union "
            << public_name(obj) << "\");" << std::endl;
        OUT << "}" << std::endl;
        OUT << "return " << private_member_name(obj, member) << ";" << std::endl;
        OUT << "}" << std::endl;
        OUT << "set" << std::endl;
        OUT << "{" << std::endl;

        cs_emit_range_check(obj, member, out);

        if (is_case_default(member, obj)) {
            // std::vector<const ptree*> case_non_default = case_non_default_values(obj);
            OUT << "if ( " << cs_case_if_expr(obj, member->members, "discriminator", true) << " )"
                << std::endl;
            OUT << "{" << std::endl;
            OUT << "discriminator = " << cs_value(member->members, obj->discriminator, obj) << ";"
                << std::endl;
            OUT << "}" << std::endl;
        } else if (member->members->next) {
            OUT << "if ( " << cs_case_if_expr(obj, member->members, "discriminator", false) << " )"
                << std::endl;
            OUT << "{" << std::endl;
            OUT << "discriminator = " << cs_value(member->members, obj->discriminator, obj) << ";"
                << std::endl;
            OUT << "}" << std::endl;
        } else {
            OUT << "discriminator = " << cs_value(member->members, obj->discriminator, obj) << ";"
                << std::endl;
        }
        OUT << private_member_name(obj, member) << " = value;" << std::endl;
        OUT << "}" << std::endl;
        OUT << "}" << std::endl;
    }

    OUT << "public object GetValue()" << std::endl;
    OUT << "{" << std::endl;
    OUT << "switch ( discriminator )" << std::endl;
    OUT << "{" << std::endl;
    for (auto member : members) {
        if (!is_case_default(member, obj)) {
            for (auto cas : member->members) {
                OUT << "case " << cs_value(cas, obj->discriminator, obj) << ":" << std::endl;
            }
            OUT << "return " << public_member_name(obj, member) << ";" << std::endl;
        }
    }
    if (has_case_default(obj)) {
        for (auto member : members) {
            if (is_case_default(member, obj)) {
                OUT << "default:" << std::endl;
                OUT << "return " << public_member_name(obj, member) << ";" << std::endl;
            }
        }
    } else {
        OUT << "default:" << std::endl;
        OUT << "return null;" << std::endl;
    }
    OUT << "}" << std::endl;
    OUT << "}" << std::endl;

    OUT << "public void SetValue( object value )" << std::endl;
    OUT << "{" << std::endl;
    for (auto it = members.begin(); it != members.end(); ++it) {
        const ptree* member = *it;
        if (it != members.begin()) {
            OUT << "else ";
        }
        OUT << "if ( value is " << cs_type(member, obj->super) << " )" << std::endl;
        OUT << "{" << std::endl;
        OUT << public_member_name(obj, member) << " = (" << cs_type(member, obj->super) << ")value;"
            << std::endl;
        OUT << "}" << std::endl;
    }
    OUT << "else" << std::endl;
    OUT << "{" << std::endl;
    OUT << "throw new InvalidOperationException(\"Unknown type for union " << public_name(obj)
        << "\");" << std::endl;
    OUT << "}" << std::endl;
    OUT << "}" << std::endl;

    // Default constructor
    OUT << "public " << public_name(obj) << "()";
    OUT << std::endl;
    OUT << "{" << std::endl;
    if (has_case_default(obj)) {
        for (auto member : members) {
            if (is_case_default(member, obj)) {
                cs_emit_member_init(member, obj, out);
                break;
            }
        }
    } else {
        cs_emit_member_init(members[0], obj, out);
    }
    OUT << "}" << std::endl;

    cs_emit_read_method(obj, out);
    cs_emit_write_method(obj, out);
    cs_emit_typesupport_methods(obj, out);

    OUT << "}" << std::endl;
}

static void cs_emit_member_init(const ptree* member, const ptree* obj, ModuleMap& out) {
    OUT << public_member_name(obj, member) << " = " << cs_init_value(member, obj->super) << ";"
        << std::endl;
}

static void code_gen_cs_rec(const ptree* obj, ModuleMap& out) {
    bool did_emit = cs_emit_const(obj, out);

    for (; obj; obj = obj->next) {
        if (is_emit(obj, LANG_CS)) {
            if (obj->flags & OPT_DECLARATION) {
                continue;
            }
            switch (obj->kind) {
            case N_MODULE:
                if (obj->members) {
                    if (did_emit) {
                        OUT << std::endl;
                    }
                    cs_emit_module(obj, out);
                    did_emit = true;
                }
                break;
            case N_UNION:
                if (did_emit) {
                    OUT << std::endl;
                }
                cs_emit_union(obj, out);
                did_emit = true;
                break;
            case N_STRUCT:
                if (did_emit) {
                    OUT << std::endl;
                }
                if (is_simple_struct(obj) && CommandLineOption::intercom_build()) {
                    cs_emit_struct(obj, out);
                } else {
                    cs_emit_class(obj, out);
                }
                did_emit = true;
                break;
            case N_ENUM:
            case N_BITMASK:
                if (did_emit) {
                    OUT << std::endl;
                }
                cs_emit_enum(obj, out);
                did_emit = true;
                break;
            default:
                break;
            }
        }
    }
}

static bool cs_type_is_nullable(const ptree* member) {
    if (is_optional(member)) {
        const ptree* type_obj = base_type_of(member);
        if (is_numeric(member) || type_obj->kind == N_ENUM || type_obj->kind == N_BITMASK) {
            return true;
        }
        if (is_simple_struct(member->type) &&
            idl_scoped_name(member->type->super, nullptr) == "DDS") {
            return true;
        }
    }
    return false;
}

void intercom::cidl::code_gen_cs(const parse_result* result) {
    ModuleMap out;
    const ptree* mapping = lookup_node(create_identifier("INTERCOM_CS_NAME_MAPPING"));
    if (mapping && mapping->value.kind() == PTREE_KIND) {
        const_cast<ptree*>(mapping)->flags &= ~OPT_EMIT_CODE;
        for (const ptree* value = mapping->value.val.node()->members; value && value->next;
             value = value->next->next) {
            g_name_mapper[string_value(value->value)] = string_value(value->next->value);
        }
    }
    g_name_mapper["DDS::BuiltinTopicKey_t"] = "BuiltinTopicKey";
    g_name_mapper["DDS::Time_t"] = "Time";
    g_name_mapper["DDS::Duration_t"] = "Duration";
    if (result->tree) {
        code_gen_cs_rec(result->tree, out);
    }
    for (auto& it : out) {
        std::string name = it.first;
        // TODO(idarcar):
        // intercom::corba::String_var new_name = name.c_str();
        // name = trim_include_name(new_name.inout(), true);
        memf cs_file;
        memset(&cs_file, 0, sizeof(memf));
        cs_file.do_indent = 1;
        cs_file.lang_kind = C_JAVA_FILE;

        std::istringstream istream(it.second->str());
        std::string line;
        while (std::getline(istream, line)) {
            mprintf(&cs_file, "{}\n", line.c_str());
        }

        savememf(
            &cs_file, nullptr, CommandLineOption::cs_target_directory(), "", "{}.cs", name.c_str()
        );
        mreset(&cs_file);
        delete it.second;
    }
}

void intercom::cidl::code_gen_cs(const parse_result* result, const char* destination) {
    intercom::cidl::CommandLineOption::get_instance().cs_target_directory = destination;
    code_gen_cs(result);
}
