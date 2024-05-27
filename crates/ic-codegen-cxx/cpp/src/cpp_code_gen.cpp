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

#include <array>
#include <cassert>
#include <cctype>
#include <cstdint>
#include <cstring>
#include <deque>
#include <memory>

#include "InterCOM/version.h"
#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/memf.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "utils/stdprintf.h"
#include "utils/string_utils.h"

#define INTERCOM_PUBLIC_MACRO_NAME "INTERCOM_PUBLIC"

using namespace intercom::cidl;

static void cgcpl_recurs(const ptree* obj);

void emit_docs(struct memf* f, const ptree* obj);
void emit_post_docs(struct memf* f, const ptree* obj);

void cpl_rpc_service_gen(
    const ptree* a_node,
    struct memf* a_memf_head,
    struct memf* a_memf_body,
    const ptree* current_include
);

static memf g_hd_file;
static memf g_tbd_file;
static memf g_hd_ts_file;
static memf g_hd_json_file;
static memf g_hd_hash_file;
static memf g_tbd_hash_file;
static memf g_hd_impl_file;
static memf g_hd_rpc_file;
static memf g_hd_fmt_file;
static memf g_prebd_file;
static memf* g_all_headers[] = {&g_hd_file, &g_hd_impl_file, &g_hd_json_file, nullptr};
static memf* g_hd_tbd_files[] = {&g_hd_file, &g_tbd_file, nullptr};

static void cpl_prototype_c_def(const ptree* obj);

static std::string name(const ptree* obj) {
    return cpp_name(obj);
}

static std::string const_name(const ptree* obj) {
    return "DEFAULT_" + obj->name;
}

static void
cpp_scoped_name_rec(const ptree* node, const ptree* common_parent, std::stringstream& out) {
    if (node) {
        ptree* scope = node->super;
        const bool skip_enum_scp = !CommandLineOption::cpp_gen_cpp11();
        if (scope && (scope->kind == N_BITMASK || (skip_enum_scp && scope->kind == N_ENUM))) {
            scope = scope->super;  // skip cpp enum scope
        }
        if (scope && scope == common_parent) {
            out << safe_name(node, node->name, LANG_CPP);
        } else if (idl_scoped_name(node, nullptr) == "DDS") {
            out << "::intercom::dcps";
        } else if (idl_scoped_name(node, nullptr) == "DDS::Security") {
            out << "::intercom::dcps::security";
        } else if (idl_scoped_name(node, nullptr) == "DDS::RPC") {
            out << "::intercom::dcps::rpc";
        } else if (idl_scoped_name(node, nullptr) == "DDS::XTypes") {
            out << "::intercom::dcps::xtypes";
        } else {
            cpp_scoped_name_rec(scope, common_parent, out);
            out << "::" << safe_name(node, node->name, LANG_CPP);
        }
    }
}

static void
cpp_array_name_rec(const ptree* obj, const ptree* context, int pos, std::stringstream& out) {
    if (pos < static_cast<int>(obj->bounds.size())) {
        if (CommandLineOption::corba_types()) {
            out << "::intercom::corba::Array<";
        } else {
            out << "::std::array<";
        }
        cpp_array_name_rec(obj, context, pos + 1, out);
        out << ", " << integer_value(obj->bounds[pos]) << ">";
    } else {
        out << cpp_type_name(obj->element_type, context);
    }
}

static std::string
cplpl_member_type(const ptree* elem, const ptree* context, bool suppress_indirection = false);

std::string intercom::cidl::cpp_type_name(const ptree* node, const ptree* context) {
    if (node == nullptr) {
        return "";
    }
    if (node->kind == N_ALIAS && node->flags & OPT_ANONYMOUS_ALIAS) {
        return cplpl_member_type(node, context);
    }

    std::stringstream out;
    if (node == &boolean_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Boolean" : "bool");
    } else if (node == &int8_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Int8" : "int8_t");
    } else if (node == &octet_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::UInt8" : "uint8_t");
    } else if (node == &char_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Char" : "char");
    } else if (node == &wchar_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Char16" : "char16_t");
    } else if (node == &short_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Int16" : "int16_t");
    } else if (node == &ushort_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::UInt16" : "uint16_t");
    } else if (node == &long_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Int32" : "int32_t");
    } else if (node == &ulong_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::UInt32" : "uint32_t");
    } else if (node == &longlong_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Int64" : "int64_t");
    } else if (node == &ulonglong_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::UInt64" : "uint64_t");
    } else if (node == &float_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Float" : "float");
    } else if (node == &double_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::Double" : "double");
    } else if (node == &ldouble_type) {
        out << (CommandLineOption::corba_types() ? "::intercom::corba::LongDouble" : "long double");
    } else if (node == &any_type) {
        out << "::intercom::corba::Any";
    } else if (node == &object_type) {
        out << "::intercom::corba::Object";
    } else if (node->kind == N_ARRAY) {
        cpp_array_name_rec(node, context, 0, out);
    } else if (node->kind == N_SEQUENCE) {
        if (CommandLineOption::corba_types()) {
            if (node->bounds.empty()) {
                out << "::intercom::corba::Sequence<" << cpp_type_name(node->element_type, context)
                    << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << ">";
            } else {
                out << "::intercom::corba::BoundedSequence<"
                    << cpp_type_name(node->element_type, context)
                    << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << ", "
                    << integer_value(node->bounds[0]) << ">";
            }
        } else {
            if (node->bounds.empty()) {
                out << "::std::vector< " << cpp_type_name(node->element_type, context)
                    << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << " >";
            } else {
                out << "::intercom::bounded_vector< " << cpp_type_name(node->element_type, context)
                    << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << ", "
                    << unsigned_value(node->bounds[0]) << " >";
            }
        }
    } else if (node->kind == N_STRING) {
        if (CommandLineOption::corba_types()) {
            out << "::intercom::corba::";
            if (!node->bounds.empty()) {
                out << "Bounded";
            }
            if (is_wstring(node)) {
                out << "W";
            } else if (CommandLineOption::string_encoding_utf8()) {
                out << "Utf8";
            }
            out << "String_var";
            if (!node->bounds.empty()) {
                out << "< " << unsigned_value(node->bounds[0]) << " >";
            }
        } else {
            if (node->bounds.empty()) {
                out << "::std::";
            } else {
                out << "::intercom::bounded_";
            }
            if (is_wstring(node)) {
                out << (CommandLineOption::use_wstring() ? "w" : "u16");
            }
            out << "string";
            if (!node->bounds.empty()) {
                out << "< " << unsigned_value(node->bounds[0]) << " >";
            }
        }
    } else if (node->kind == N_MAP) {
        if (node->bounds.empty()) {
            out << "::std::map< " << cpp_type_name(node->key_type, context)
                << (base_type_of(node->key_type)->kind == N_INTERFACE ? "*" : "") << ", "
                << cpp_type_name(node->element_type, context)
                << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << " >";
        } else {
            out << "::intercom::bounded_map< " << cpp_type_name(node->key_type, context)
                << (base_type_of(node->key_type)->kind == N_INTERFACE ? "*" : "") << ", "
                << cpp_type_name(node->element_type, context)
                << (base_type_of(node->element_type)->kind == N_INTERFACE ? "*" : "") << ", "
                << unsigned_value(node->bounds[0]) << " >";
        }
    } else if (node == context) {
        out << safe_name(node, node->name, LANG_CPP);
    } else {
        cpp_scoped_name_rec(node, common_scope(node, context), out);
    }
    if (context == nullptr && out.str().substr(0, 2) == "::") {
        return out.str().substr(2);
    }
    return out.str();
}

static std::string cpp_string_view_type_name(const ptree* node) {
    if (is_wstring(base_type_of(node))) {
        return CommandLineOption::use_wstring() ? "::intercom::wstring_view"
                                                : "::intercom::u16string_view";
    }
    return "::std::string_view";
}

static std::string public_member_name(const ptree* elem) {
    if (CommandLineOption::cpp_access_functions() || is_merged(elem) ||
        (elem->kind == N_MEMBER && elem->super->kind == N_UNION)) {
        return fmt::format("{}()", cpp_name(elem));
    }
    return cpp_name(elem);
}

static std::string private_member_name(const ptree* elem) {
    if (CommandLineOption::cpp_access_functions() ||
        (elem->kind == N_MEMBER && elem->super->kind == N_UNION)) {
        return fmt::format("m_{}_", cpp_name(elem));
    }
    return cpp_name(elem);
}

static std::string scoped_name(const ptree* obj, const ptree* context) {
    return cpp_type_name(obj, context);
}

static std::string type_name(const ptree* obj) {
    return cpp_type_name(obj->type, namespace_of(obj));
}

static void dll_export(struct memf* memf, const ptree* obj) {
    (void)obj;
    if (CommandLineOption::dll_exp_sym()) {
        mprintf(memf, "{} ", CommandLineOption::dll_exp_sym());
    }
}

static void includeit(struct memf* memf, const ptree* source) {
    const char* include_prefix = CommandLineOption::header_subfolder();
    std::string name = trim_include_name(source->name, !CommandLineOption::disable_header_follow());
    bool system_inc = (source->flags & OPT_SYSTEM_INCLUDE) != 0;

    if (include_prefix && !system_inc) {
        mprintf(
            memf,
            "~U#include <{}/{}.{}>\n",
            include_prefix,
            name,
            CommandLineOption::cpp_header_postfix()
        );
    } else {
        mprintf(
            memf,
            "~U#include {}{}.{}{}\n",
            system_inc ? '<' : '"',
            name,
            CommandLineOption::cpp_header_postfix(),
            system_inc ? '>' : '"'
        );
    }
}

static int is_pointer_type(const ptree* elem) {
    return base_type_of(elem)->kind == N_INTERFACE;
}

static bool is_pass_by_value(const ptree* elem) {
    return
        // Pass pointers and primitive types by value
        (is_primitive(base_type_of(elem)) || (base_type_of(elem)->kind == N_ENUM) ||
         (base_type_of(elem)->kind == N_BITMASK)) &&
        // ...but not if they are optional templates
        !is_optional(elem) && !is_shared(elem);
}

static bool has_multiple_cases(const ptree* elem) {
    bool is_default = false;
    int count = 0;
    for (auto cv : elem->members) {
        count++;
        if (cv->flags & OPT_DEFAULT) {
            is_default = true;
        }
    }
    return is_default || count > 1;
}

static std::string
cplpl_member_type(const ptree* elem, const ptree* context, bool suppress_indirection) {
    if (is_optional(elem, get_direct_annotation)) {
        return fmt::format("::intercom::optional<{}>", scoped_name(elem->type, context));
    }
    if (!suppress_indirection && is_shared(elem, get_direct_annotation)) {
        return fmt::format("::std::unique_ptr<{}>", scoped_name(elem->type, context));
    }
    return fmt::format("{}{}", scoped_name(elem->type, context), is_pointer_type(elem) ? "*" : "");
}

static int member_count(const ptree* obj) {
    int count = 0;
    if (obj) {
        for (const ptree* elem : obj->members) {
            if (elem->kind == N_MEMBER) {
                ++count;
            }
        }
        for (auto parent : obj->parents) {
            count += member_count(parent);
        }
    }
    return count;
}

static int original_member_count(const ptree* obj) {
    if (!obj->original_members) {
        return member_count(obj);
    }
    int count = 0;
    if (obj) {
        for (const ptree* elem : obj->original_members) {
            if (elem->kind == N_MEMBER) {
                ++count;
            }
        }
        for (auto parent : obj->parents) {
            count += member_count(parent);
        }
    }
    return count;
}

static int has_name_collision(const ptree* obj, std::string_view elem_name) {
    const ptree* elem;
    if (!obj) {
        return 0;
    }
    for (elem = obj->members; elem; elem = elem->next) {
        if (elem->kind == N_MEMBER && elem_name == name(elem)) {
            return 1;
        }
    }
    for (auto parent : obj->parents) {
        if (has_name_collision(parent, elem_name)) {
            return 1;
        }
    }
    return 0;
}

static std::string cplpl_param_name_force(const ptree* obj, std::string_view base_name) {
    auto buf = fmt::format("a_{}", base_name);
    if (has_name_collision(obj, buf)) {
        auto new_base = fmt::format("{}_", base_name);
        buf = cplpl_param_name_force(obj, new_base);
    }
    return buf;
}

std::string cplpl_param_name(const ptree* obj, std::string_view base_name) {
    if (member_count(obj) == 0) {
        return {};
    }
    return cplpl_param_name_force(obj, base_name);
}

static void emit_const_value(
    struct memf* mfil,
    const numeric& value,
    const ptree* scope,
    const ptree* context
) {
    switch (value.kind()) {
    case UNDEF_KIND:
        mprintf(mfil, " nullptr");
        break;
    case BOOLEAN_KIND:
        mprintf(mfil, " {}", value.val.b() ? "true" : "false");
        break;
    case INT8_KIND:
        mprintf(mfil, " {}", static_cast<int>(value.val.i8()));
        break;
    case OCTET_KIND:
        if (value.base == 16) {
            mprintf(mfil, " 0x{:02x}U", value.val.o());
        } else if (value.base == 8) {
            mprintf(mfil, " 0{:02o}U", value.val.o());
        } else {
            mprintf(mfil, " {}U", value.val.o());
        }
        break;
    case SHORT_KIND:
        mprintf(mfil, " {}", static_cast<int>(value.val.s()));
        break;
    case USHORT_KIND:
        if (value.base == 16) {
            mprintf(mfil, " 0x{:x}U", static_cast<unsigned int>(value.val.us()));
        } else if (value.base == 8) {
            mprintf(mfil, " 0{:o}U", static_cast<unsigned int>(value.val.us()));
        } else {
            mprintf(mfil, " {}U", static_cast<unsigned int>(value.val.us()));
        }
        break;
    case LONG_KIND:
        mprintf(mfil, " {}", value.val.l());
        break;
    case ULONG_KIND:
        if (value.base == 16) {
            mprintf(mfil, " 0x{:x}U", value.val.ul());
        } else if (value.base == 8) {
            mprintf(mfil, " 0{:o}U", value.val.ul());
        } else {
            mprintf(mfil, " {}U", value.val.ul());
        }
        break;
    case LONGLONG_KIND:
        mprintf(mfil, " {}LL", static_cast<long long int>(value.val.ll()));
        break;
    case ULONGLONG_KIND:
        if (value.base == 16) {
            mprintf(mfil, " 0x{:x}ULL", static_cast<unsigned long long int>(value.val.ull()));
        } else if (value.base == 8) {
            mprintf(mfil, " 0{:o}ULL", static_cast<unsigned long long int>(value.val.ull()));
        } else {
            mprintf(mfil, " {}ULL", static_cast<unsigned long long int>(value.val.ull()));
        }
        break;
    case FLOAT_KIND:
        mprintf(mfil, " static_cast<float>({})", to_string(value.val.f()));
        break;
    case DOUBLE_KIND:
        mprintf(mfil, " {}", to_string(value.val.d()));
        break;
    case STRING_KIND:
        mprintf(mfil, " \"{}\"", value.val.str());
        break;
    case CHAR_KIND:
        mprintf(mfil, " '\\{:03o}'", static_cast<char>(value.val.c()));
        break;
    case PTREE_KIND: {
        const ptree* node = value.val.node();
        const bool double_brace =
            !CommandLineOption::corba_types() && node->type && node->type->kind == N_ARRAY;
        if (node->kind == N_CONST) {
            if ((node->flags & OPT_CONST_VALUE) != 0) {
                if (node->type) {
                    mprintf(
                        mfil, " {}{}", scoped_name(node->type, scope), double_brace ? "{{" : "{"
                    );
                } else {
                    mprintf(mfil, " {{");
                }
                const ptree* p;
                for (p = node->members; p; p = p->next) {
                    if (p != node->members) {
                        mprintf(mfil, ", ");
                    }
                    emit_const_value(mfil, p->value, scope, context);
                }
                mprintf(mfil, " {}", double_brace ? "}}" : "}");
            }
            // MSVC v140 isn't capable of handling nested initializer lists in constants.
            // As a workaround, if the constant is defined in the same file, we refer to it
            // directly.
            else if (!is_pass_by_value(base_type_of(node->type)) &&
                     !(context && node->file_name == context->file_name)) {
                emit_const_value(mfil, node->value, scope, context);
            } else {
                mprintf(mfil, " {}", scoped_name(node, scope));
            }
        } else if (node == &boolean_type) {
            mprintf(mfil, " false");
        } else if (node->value.kind() != UNDEF_KIND) {
            emit_const_value(mfil, node->value, scope, context);
        } else {
            mprintf(mfil, " {}()", scoped_name(node, scope));
        }
    } break;
    }
}

std::string get_const_value(const numeric& value, const ptree* scope) {
    struct memf tmp;
    memset(&tmp, 0, sizeof(struct memf));
    mreset(&tmp);
    emit_const_value(&tmp, value, scope, scope);
    std::string res = tmp.memfile;
    mreset(&tmp);
    return res;
}

static void
emit_default_value(struct memf* mfil, const struct ptree* type, const struct ptree* context) {
    numeric default_value = get_default_value(type);
    if (is_shared(type)) {
        mprintf(mfil, " {}(new", cplpl_member_type(type, context));
        if (default_value.kind() != PTREE_KIND) {
            mprintf(mfil, " {}{{", cpp_type_name(base_type_of(type), context));
        }
    }
    emit_const_value(mfil, default_value, context, type);
    if (is_shared(type)) {
        mprintf(mfil, " {}", default_value.kind() != PTREE_KIND ? "})" : ")");
    }
}

static void emit_default_array_initializer_list(
    struct memf* mfil,
    const ptree* array_type,
    size_t level,
    bool emit_value
);

static void
emit_void_value(struct memf* mfil, const struct ptree* type, const struct ptree* context) {
    mprintf(mfil, " {}", cplpl_member_type(type, context));
    if (is_shared(type)) {
        mprintf(mfil, "(new {}", cpp_type_name(type->type, context));
    }
    if (base_type_of(type)->kind == N_ARRAY && !is_optional(type) &&
        (is_pass_by_value(base_type_of(type)->element_type) || !CommandLineOption::corba_types())) {
        emit_default_array_initializer_list(mfil, base_type_of(type), 0, false);
    } else {
        mprintf(mfil, "{{}}");
    }
    if (is_shared(type)) {
        mprintf(mfil, ")");
    }
}

static void emit_default_array_initializer_list(
    struct memf* mfil,
    const ptree* array_type,
    size_t level,
    bool emit_value
) {
    mprintf(mfil, "{{");
    // std::array only has implicit constructors, so we need an additional pair of
    // braces to properly initialize the inner array.
    if (!CommandLineOption::corba_types()) {
        mprintf(mfil, "{{");
    }
    if (level < array_type->bounds.size() - 1) {
        for (size_t i = 0; i < static_cast<size_t>(integer_value(array_type->bounds[level])); ++i) {
            if (i > 0) {
                mprintf(mfil, ", ");
            }
            emit_default_array_initializer_list(mfil, array_type, level + 1, emit_value);
        }
    } else if (emit_value) {
        for (size_t i = 0; i < static_cast<size_t>(integer_value(array_type->bounds[level])); ++i) {
            if (i > 0) {
                mprintf(mfil, ", ");
            }
            emit_default_value(mfil, array_type->element_type, nullptr);
        }
    }
    if (!CommandLineOption::corba_types()) {
        mprintf(mfil, "}}");
    }
    mprintf(mfil, "}}");
}

static void emit_initializer_list(
    struct memf* mfil,
    const numeric& value,
    const ptree* scope,
    const ptree* context
) {
    mprintf(mfil, " {{");
    if (value.kind() == PTREE_KIND && value.val.node()->kind == N_CONST) {
        const ptree* node = value.val.node();
        if ((node->flags & OPT_CONST_VALUE) != 0) {
            for (auto p : node->members) {
                if (p != node->members) {
                    mprintf(mfil, ", ");
                }
                emit_const_value(mfil, p->value, scope, context);
            }
        } else {
            emit_const_value(mfil, value, scope, context);
        }
    } else {
        emit_const_value(mfil, value, scope, context);
    }
    mprintf(mfil, " }}");
}

static void gen_case_test(
    struct memf* mfil,
    const ptree* obj,
    const ptree* elem,
    std::string_view discr_name = "ic_discriminator_value_"
) {
    bool first = true;

    if (elem->flags & OPT_DEFAULT) {
        for (auto mem : obj->members) {
            if (mem == elem) {
                continue;
            }
            for (auto cas : mem->members) {
                if (!first) {
                    mprintf(mfil, " || ");
                }
                mprintf(mfil, "{} ==", discr_name);
                emit_const_value(mfil, cas->value, obj->super, obj);
                first = false;
            }
        }
    } else {
        for (auto cas : elem->members) {
            if (!first) {
                mprintf(mfil, " && ");
            }
            mprintf(mfil, "{} !=", discr_name);
            emit_const_value(mfil, cas->value, obj->super, obj);
            first = false;
        }
    }
    if (first) {  // only case is default
        mprintf(mfil, "{}", "false");
    }
}

static bool cpl_gen_cases(struct memf* mfil, const ptree* elem, const ptree* scope) {
    if (has_default_case(elem)) {
        mprintf(mfil, "default:\n");
        return true;
    }
    for (auto c : elem->members) {
        mprintf(mfil, "case");
        emit_const_value(mfil, c->value, scope, elem);
        mprintf(mfil, ":\n");
    }
    return false;
}

static void
cpl_gen_hash_member(struct memf* memf, std::string_view name, const ptree* type, int level) {
    switch (type->kind) {
    case N_ARRAY: {
        std::string new_name(name);
        for (int i = 0; i < static_cast<int>(type->bounds.size()); ++i) {
            mprintf(memf, "for (auto& value_{} : {}) {{\n", level + i, new_name);
            new_name = fmt::format("value_{}", level + i);
        }
        cpl_gen_hash_member(
            memf, new_name, type->element_type, level + static_cast<int>(type->bounds.size())
        );
        for (size_t i = 0; i < type->bounds.size(); ++i) {
            mprintf(memf, "}}\n");
        }
        break;
    }
    case N_MAP: {
        auto new_name = fmt::format("value_{}", level);
        mprintf(
            memf,
            "for (auto{} {} : {}) {{\n",
            is_primitive(base_type_of(type->element_type)) ? "" : "&",
            new_name,
            name
        );
        new_name = fmt::format("value_{}.first", level);
        cpl_gen_hash_member(memf, new_name, type->key_type, level + 1);
        new_name = fmt::format("value_{}.second", level);
        cpl_gen_hash_member(memf, new_name, type->element_type, level + 1);
        mprintf(memf, "}}\n");
        break;
    }
    case N_SEQUENCE: {
        auto new_name = fmt::format("value_{}", level);
        mprintf(
            memf,
            "for (auto{} {} : {}) {{\n",
            is_primitive(base_type_of(type->element_type)) ? "" : "&",
            new_name,
            name
        );
        cpl_gen_hash_member(memf, new_name, type->element_type, level + 1);
        mprintf(memf, "}}\n");
        break;
    }
    case N_ENUM:
        mprintf(
            memf,
            "h ^= std::hash< {} >()(static_cast< {} >({}));\n",
            scoped_name(type->element_type, nullptr),
            scoped_name(type->element_type, nullptr),
            name
        );
        break;
    case N_ALIAS:
        cpl_gen_hash_member(memf, name, base_type_of(type), level);
        break;
    default:
        mprintf(memf, "h ^= std::hash< {} >()({});\n", scoped_name(type, nullptr), name);
        break;
    }
}

static void cpl_gen_hash(const ptree* obj) {
    if (!obj || (obj->kind != N_STRUCT && obj->kind != N_UNION && obj->kind != N_EXCEPTION)) {
        return;
    }

    if (mempty(&g_hd_hash_file)) {
        mprintf(&g_hd_hash_file, "namespace std {{\n");
    }

    mprintf(&g_hd_hash_file, "template<> struct hash<{}> {{\n", scoped_name(obj, nullptr));
    mprintf(&g_hd_hash_file, "using argument_type = {};\n", scoped_name(obj, nullptr));
    mprintf(&g_hd_hash_file, "using result_type = std::size_t;\n");
    dll_export(&g_hd_hash_file, obj);
    mprintf(&g_hd_hash_file, "result_type operator()(const argument_type&) const noexcept;\n");
    mprintf(&g_hd_hash_file, "}};\n");

    bool has_members = !obj->parents.empty() || member_count(obj) > 0;
    mprintf(
        &g_tbd_hash_file,
        "std::size_t std::hash<{}>::operator()(const {}& {}) const noexcept {{\n",
        scoped_name(obj, nullptr),
        scoped_name(obj, nullptr),
        has_members ? "s" : ""
    );
    mprintf(&g_tbd_hash_file, "result_type h = 0;\n");
    if (obj->kind == N_UNION) {
        cpl_gen_hash_member(&g_tbd_hash_file, "s._d()", obj->discriminator->type, 0);
        mprintf(&g_tbd_hash_file, "switch (s._d()) {{\n");
    }
    if (!obj->parents.empty()) {
        cpl_gen_hash_member(&g_tbd_hash_file, "s", obj->parents[0], 0);
    }
    for (const ptree* elem : obj->members) {
        if (elem->kind == N_MEMBER) {
            if (obj->kind == N_UNION) {
                cpl_gen_cases(&g_tbd_hash_file, elem, nullptr);
            }
            if (is_shared(elem)) {
                auto member_name = fmt::format("*s.{}", public_member_name(elem));
                mprintf(&g_tbd_hash_file, "if (s.{} != nullptr) {{\n", public_member_name(elem));
                cpl_gen_hash_member(&g_tbd_hash_file, member_name, elem->type, 0);
                mprintf(&g_tbd_hash_file, "}}\n");
            } else if (is_optional(elem)) {
                auto member_name = fmt::format("*s.{}", public_member_name(elem));
                mprintf(&g_tbd_hash_file, "if (s.{}.has_value()) {{\n", public_member_name(elem));
                cpl_gen_hash_member(&g_tbd_hash_file, member_name, elem->type, 0);
                mprintf(&g_tbd_hash_file, "}}\n");
            } else {
                auto member_name = fmt::format("s.{}", public_member_name(elem));
                cpl_gen_hash_member(&g_tbd_hash_file, member_name, elem->type, 0);
            }
            if (obj->kind == N_UNION) {
                mprintf(&g_tbd_hash_file, "break;\n");
            }
        }
        if (elem->kind == N_NULL) {
            cpl_gen_cases(&g_tbd_hash_file, elem, nullptr);
            mprintf(&g_tbd_hash_file, "break;\n");
        }
    }
    if (obj->kind == N_UNION) {
        mprintf(&g_tbd_hash_file, "}}\n");
    }
    mprintf(&g_tbd_hash_file, "return h;\n");
    mprintf(&g_tbd_hash_file, "}}\n\n");
}

static void cpl_gen_marshal_member(
    int member_index,
    std::string_view name,
    std::string_view expr,
    std::string_view info_name,
    int shared
) {
    if (name.compare(0, 35, "void_void_void_dummy_skipped_in_air") == 0) {
        return;
    }
    if (shared) {
        mprintf(
            &g_hd_ts_file,
            "if (!{0}) {{\nthrow std::runtime_error(\"{0} can not be null\");\n}}\n",
            expr
        );
    }
    mprintf(
        &g_hd_ts_file,
        "serializer.io({}->members[{}], {}{});\n",
        info_name,
        member_index,
        (shared ? "*" : ""),
        expr
    );
}

static bool emit_range_check_body(
    const ptree* obj,
    const ptree* elem,
    std::string_view element_name,
    std::stringstream& out,
    int level = 0
) {
    bool has_range_check = false;
    const ptree* type = base_type_of(elem);
    if (type->kind == N_SEQUENCE || type->kind == N_ARRAY || type->kind == N_MAP) {
        std::stringstream iter_name_stream;
        iter_name_stream << "it";
        if (level > 0) {
            iter_name_stream << level;
        }
        std::string iter_name = iter_name_stream.str();
        out << "for (" << scoped_name(type, nullptr) << "::const_iterator " << iter_name << " = ("
            << element_name << ").begin(); " << iter_name << " != (" << element_name << ").end(); "
            << "++" << iter_name << ") {\n";
        if (type->kind == N_MAP) {
            bool has_key_check =
                emit_range_check_body(obj, type->key_type, iter_name + "->first", out, level + 1);
            bool has_element_check = emit_range_check_body(
                obj, type->element_type, iter_name + "->second", out, level + 1
            );
            has_range_check = has_key_check || has_element_check;
        } else {
            has_range_check =
                emit_range_check_body(obj, type->element_type, "*" + iter_name, out, level + 1);
        }
        out << "}\n";
    } else if (type->kind == N_PRIMITIVE) {
        std::string_view prefix = "if (Archive::IS_WRITER && (";
        if (has_min_value(elem)) {
            out << prefix << " " << element_name << " < "
                << get_const_value(get_min_value(elem), nullptr);
            prefix = " || ";
            has_range_check = true;
        }
        if (has_max_value(elem)) {
            out << prefix << " " << element_name << " > "
                << get_const_value(get_max_value(elem), nullptr);
            has_range_check = true;
        }
        out << ")) {\n"
            << "throw std::range_error(\"Illegal value for " << cpp_type_name(obj, nullptr)
            << "::" << name(elem) << "\");\n}\n";
    }
    return has_range_check;
}

static void emit_range_check(const ptree* obj, const ptree* elem, std::string_view element_name) {
    bool has_range_check = false;
    std::stringstream out;

    if (is_optional(elem)) {
        out << "if (" << element_name << ".has_value()) {\n";
        auto b2 = fmt::format("{}.value()", element_name);
        has_range_check = emit_range_check_body(obj, elem, b2, out);
        out << "}\n";
    } else if (is_shared(elem)) {
        auto b2 = fmt::format("*{}", element_name);
        has_range_check = emit_range_check_body(obj, elem, b2, out);
    } else {
        has_range_check = emit_range_check_body(obj, elem, element_name, out);
    }
    if (has_range_check) {
        mprintf(&g_hd_ts_file, "{}", out.str());
    }
}

static int cpl_gen_marshal_members(const ptree* obj, std::string_view info_name, int member_index) {
    auto param = cplpl_param_name(obj, "value");

    if (!obj) {
        return member_index;
    }
    for (auto parent : obj->parents) {
        member_index = cpl_gen_marshal_members(parent, info_name, member_index);
    }
    for (const ptree* elem = obj->members; elem != nullptr; elem = elem->next) {
        if (elem->kind == N_MEMBER) {
            if (is_non_serialized(elem)) {
                continue;
            }
            auto b1 = fmt::format("{}.{}", param, public_member_name(elem));
            if (obj->kind == N_UNION) {
                cpl_gen_cases(&g_hd_ts_file, elem, nullptr);
                mprintf(&g_hd_ts_file, "if (Archive::IS_READER) {{\n{}._d(discr);\n}}\n", param);
            }
            cpl_gen_marshal_member(member_index++, name(elem), b1, info_name, is_shared(elem));
            emit_range_check(obj, elem, b1);
            if (obj->kind == N_UNION) {
                mprintf(&g_hd_ts_file, "break;\n");
            }
        } else if (elem->kind == N_NULL) {
            cpl_gen_cases(&g_hd_ts_file, elem, nullptr);
            mprintf(&g_hd_ts_file, "if (Archive::IS_READER) {{\n{}._d(discr);\n}}\n", param);
            mprintf(&g_hd_ts_file, "break;\n");
        }
    }
    if (obj->kind == N_UNION && !has_default_case(obj)) {
        mprintf(&g_hd_ts_file, "default:\n");
        mprintf(
            &g_hd_ts_file,
            "if (Archive::IS_READER) {{\n{} = {}();\n}}\n",
            param,
            scoped_name(obj, nullptr)
        );
        mprintf(&g_hd_ts_file, "break;\n");
    }
    return member_index;
}

using node_filter = bool (*)(const ptree*);
void gen_cpp_type_definition(struct memf* memf, const ptree* obj);
void gen_cpp_type_registration(struct memf* memf, const ptree* obj, node_filter filter);
bool no_struct_or_enum_filter(const ptree* obj);

static void cpl_conv_gen(const ptree* obj) {
    bool has_key = false;
    bool has_seq_type = false;
    const ptree* descr;
    auto objname = scoped_name(obj, nullptr);

    if (is_non_serialized(obj)) {
        return;
    }

    std::string funcname = objname;
    for (auto& c : funcname) {
        if (c == ':') {
            c = '_';
        }
    }
    for (descr = obj->members; descr && !has_key; descr = descr->next) {
        has_key = has_key || is_key_member(descr);
    }

    if ((obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_VALUETYPE) &&
        !get_annotation(obj, annotation_type_ext_suppress) && !is_nested(obj)) {
#ifndef CIDL_BOOTSTRAP
        mprintf(
            &g_hd_file,
            "\nusing {}TypeSupport = ::intercom::dcps::TTypeSupport<{}>;\n",
            name(obj),
            name(obj)
        );

        mprintf(
            &g_hd_file,
            "using {}DataWriter = ::intercom::dcps::DataWriterTemplate<{}>;\n",
            name(obj),
            name(obj)
        );

        mprintf(
            &g_hd_file,
            "using {}DataReader = ::intercom::dcps::DataReaderTemplate<{}>;\n",
            name(obj),
            name(obj)
        );

        has_seq_type = true;

        if (CommandLineOption::corba_types()) {
            mprintf(
                &g_hd_file, "using {}Seq = ::intercom::corba::Sequence<{}>;\n", name(obj), name(obj)
            );
        } else {
            mprintf(&g_hd_file, "using {}Seq = ::std::vector<{}>;\n", name(obj), name(obj));
        }
#endif
    }

    if (mempty(&g_hd_ts_file)) {
        mprintf(&g_hd_ts_file, "namespace intercom {{\n");
    }

    mprintf(
        &g_hd_ts_file,
        "template <>\nstruct TypeTraits<{}{}> {{ //< \\private\n",
        objname,
        is_bitmask(obj) ? "Bits" : ""
    );
    mprintf(&g_hd_ts_file, "using value_type = {};\n", objname);
    mprintf(&g_hd_ts_file, "using in_type = const {}&;\n", objname);
    mprintf(&g_hd_ts_file, "using out_type = {}&;\n", objname);
    mprintf(&g_hd_ts_file, "using inout_type = {}&;\n", objname);
    mprintf(&g_hd_ts_file, "using ref_type = std::shared_ptr<{}>;\n", objname);
    mprintf(&g_hd_ts_file, "using weak_ref_type = std::weak_ptr<{}>;\n", objname);
    if (has_seq_type) {
        mprintf(&g_hd_ts_file, "using sequence_type = {}Seq;\n", objname);
    }
    dll_export(&g_hd_ts_file, obj);
    mprintf(&g_hd_ts_file, "static const TypeInfo type_info;\n");
#ifndef CIDL_BOOTSTRAP
    dll_export(&g_hd_ts_file, obj);
    mprintf(&g_hd_ts_file, "static intercom::dcps::xtypes::TypeIdentifier type_identifier();\n");
    dll_export(&g_hd_ts_file, obj);
    mprintf(
        &g_hd_ts_file, "static void register_type(intercom::dcps::TypeRepository* a_repository);\n"
    );

    if ((obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_VALUETYPE) &&
        !is_nested(obj)) {
        mprintf(&g_hd_ts_file, "using reader_type = {}DataReader;\n", objname);
        mprintf(&g_hd_ts_file, "using writer_type = {}DataWriter;\n", objname);
        mprintf(&g_hd_ts_file, "using type_support_type = {}TypeSupport;\n", objname);
        mprintf(&g_hd_ts_file, "static const char* default_topic_name;\n");
        mprintf(&g_hd_ts_file, "static const char* intercom_type_identifier;\n");
        mprintf(
            &g_hd_ts_file,
            "static const bool has_member_accessor_functions = {};\n",
            ((CommandLineOption::cpp_access_functions() || obj->kind == N_UNION) ? "true" : "false")
        );
    }
    if (is_nested(obj)) {
        mprintf(&g_hd_ts_file, "static const bool is_nested = true;\n");
    }
    if (obj->kind == N_STRUCT) {
        mprintf(&g_hd_ts_file, "static const bool is_struct = true;\n");
    } else if (obj->kind == N_UNION) {
        mprintf(&g_hd_ts_file, "static const bool is_union = true;\n");
    } else if (obj->kind == N_ENUM) {
        mprintf(&g_hd_ts_file, "static const bool is_enum = true;\n");
    } else if (obj->kind == N_VALUETYPE) {
        mprintf(&g_hd_ts_file, "static const bool is_valuetype = true;\n");
    } else if (obj->kind == N_BITMASK) {
        mprintf(&g_hd_ts_file, "static const bool is_bitmask = true;\n");
    } else if (obj->kind == N_EXCEPTION) {
        mprintf(&g_hd_ts_file, "static const bool is_exception = true;\n");
    }
#endif
    mprintf(&g_hd_ts_file, "}};\n");

#ifndef CIDL_BOOTSTRAP
    {
        mprintf(
            &g_tbd_file,
            "intercom::dcps::xtypes::TypeIdentifier intercom::TypeTraits< {}{} >::type_identifier() {{\n",
            objname,
            is_bitmask(obj) ? "Bits" : ""
        );

        gen_cpp_type_definition(&g_tbd_file, obj);

        mprintf(
            &g_tbd_file,
            "void intercom::TypeTraits<{}{}>::register_type(intercom::dcps::TypeRepository* a_repository) {{\n",
            objname,
            is_bitmask(obj) ? "Bits" : ""
        );

        gen_cpp_type_registration(&g_tbd_file, obj, no_struct_or_enum_filter);

        mprintf(&g_tbd_file, "}}\n\n");
    }
#endif

    // load_type_object implementation
    if ((obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_VALUETYPE) &&
        !is_nested(obj)) {
#ifndef CIDL_BOOTSTRAP
        mprintf(
            &g_tbd_file,
            "const char* intercom::TypeTraits< {} >::default_topic_name = \"{}\";\n\n",
            objname,
            default_topic_name(obj)
        );
        mprintf(
            &g_tbd_file,
            "const char* intercom::TypeTraits< {} >::intercom_type_identifier = \"{}\";\n\n",
            objname,
            get_type_id(obj)
        );
#endif
    }

    // TypeInfo implementation
    gen_cpp_type_info(&g_tbd_file, obj, funcname);

    if ((obj->kind == N_ENUM || obj->kind == N_BITMASK) && !is_non_serialized(obj)) {
        mprintf(
            &g_hd_ts_file,
            "template <class Archive>\nstruct Serializer<Archive, {}{}> {{\n",
            objname,
            is_bitmask(obj) ? "Bits" : ""
        );
        mprintf(
            &g_hd_ts_file,
            "void operator()(Archive& a_archive, {}& a_value, const TypeInfo* a_info) {{\n",
            objname
        );
        mprintf(
            &g_hd_ts_file,
            "auto integer_value = static_cast<{}>(a_value);\n",
            scoped_name(obj->element_type, nullptr)
        );
        mprintf(
            &g_hd_ts_file,
            "a_archive.primitive_io(integer_value, a_info ? a_info : &intercom::TypeTraits< {}{} >::type_info);\n",
            objname,
            is_bitmask(obj) ? "Bits" : ""
        );
        mprintf(&g_hd_ts_file, "a_value = static_cast<{}>(integer_value);\n", objname);
        if (obj->kind == N_ENUM && get_extensibility(obj) == FINAL_EXTENSIBILITY) {
            ptree* first = obj->members;
            ptree* last = first;
            while (last->next) {
                last = last->next;
            }
            mprintf(
                &g_hd_ts_file,
                "if(a_value < {} || a_value > {}) {{\n",
                cpp_type_name(first, nullptr),
                cpp_type_name(last, nullptr)
            );
            mprintf(
                &g_hd_ts_file, "throw std::range_error(\"Illegal value for enum {}\");\n", name(obj)
            );
            mprintf(&g_hd_ts_file, "}}\n");
        }
        mprintf(&g_hd_ts_file, "}}\n}};\n\n");
    }
    if ((obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_VALUETYPE ||
         obj->kind == N_EXCEPTION) &&
        !is_non_serialized(obj)) {
        int memberIndex = 0;

        auto p_value = cplpl_param_name(obj, "value");
        auto p_archive = cplpl_param_name_force(obj, "archive");
        auto p_info = cplpl_param_name_force(obj, "info");

        mprintf(
            &g_hd_ts_file, "template <class Archive>\nstruct Serializer<Archive, {}> {{\n", objname
        );
        mprintf(
            &g_hd_ts_file,
            "void operator()(Archive& {}, {}& {}, const TypeInfo*) {{\n",
            p_archive,
            objname,
            p_value
        );
        if (member_count(obj) > 0) {
            mprintf(
                &g_hd_ts_file,
                "const TypeInfo* {} = &intercom::TypeTraits<{}>::type_info;\n",
                p_info,
                objname
            );
            mprintf(
                &g_hd_ts_file,
                "typename Archive::StructValue serializer({}, {});\n",
                p_archive,
                p_info
            );
            if (obj->kind == N_UNION) {
                mprintf(
                    &g_hd_ts_file,
                    "{} discr = {}._d();\n",
                    scoped_name(obj->discriminator->type, nullptr),
                    p_value
                );
                cpl_gen_marshal_member(memberIndex++, "discriminator", "discr", p_info, 0);
                mprintf(&g_hd_ts_file, "switch (discr) {{\n");
            }

            cpl_gen_marshal_members(obj, p_info, memberIndex);

            if (obj->kind == N_UNION) {
                mprintf(&g_hd_ts_file, "}}\n");
            }
        } else {
            mprintf(
                &g_hd_ts_file,
                "const TypeInfo* {} = &intercom::TypeTraits<{}>::type_info;\n",
                p_info,
                objname
            );
            mprintf(&g_hd_ts_file, "typename Archive::StructValue({}, {});\n", p_archive, p_info);
        }
        mprintf(&g_hd_ts_file, "}}\n}};\n");
    }
}

static void rec_cpl_gen_member_arguments(
    struct memf* mfil,
    const ptree* obj,
    const ptree* context,
    bool is_declaration,
    bool suppress_indirection,
    unsigned int& n_arguments
) {
    if (!obj) {
        return;
    }
    obj = base_type_of(obj);
    for (auto parent : obj->parents) {
        rec_cpl_gen_member_arguments(
            mfil, parent, context, is_declaration, suppress_indirection, n_arguments
        );
    }
    if (obj->kind == N_STRUCT || obj->kind == N_VALUETYPE || obj->kind == N_EXCEPTION) {
        const ptree* members = obj->original_members ? obj->original_members : obj->members;
        for (const ptree* elem : members) {
            if (elem->kind != N_MEMBER) {
                continue;
            }
            if (is_declaration) {
                mprintf(mfil, "{}\n", n_arguments ? "," : "");
                mprintf(
                    mfil,
                    "{} {}",
                    cplpl_member_type(elem, context, suppress_indirection),
                    cplpl_param_name(obj, name(elem))
                );
            } else {
                mprintf(mfil, "{}", n_arguments ? ", " : "");
                mprintf(mfil, "std::move({})", cplpl_param_name(obj, name(elem)));
            }
            n_arguments++;
        }
    }
}

static unsigned int cpl_gen_member_arguments(
    struct memf* mfil,
    const ptree* obj,
    const ptree* context,
    bool is_declaration,
    bool suppress_indirection
) {
    unsigned int n_arguments = 0;
    rec_cpl_gen_member_arguments(
        mfil, obj, context, is_declaration, suppress_indirection, n_arguments
    );
    return n_arguments;
};

static void cpl_gen_member_swap(struct memf* mfil, const ptree* obj) {
    if (obj) {
        for (auto parent : obj->parents) {
            cpl_gen_member_swap(mfil, parent);
        }
        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                mprintf(
                    mfil,
                    "swap({}.{}, {}.{});\n",
                    cplpl_param_name(obj, "first"),
                    private_member_name(elem),
                    cplpl_param_name(obj, "second"),
                    private_member_name(elem)
                );
            }
        }
    }
}

std::string combine_conds(
    const std::vector<std::string>& conditions,
    const std::string& sep,
    const bool& invert = false
) {
    std::stringstream exp;
    for (const std::string& cond : conditions) {
        if (invert) {
            exp << (cond.front() == '!' ? cond.substr(1) : "!" + cond);
        } else {
            exp << cond;
        }
        if (&cond != &conditions.back()) {
            exp << sep;
        }
    }
    return exp.str();
}

std::string opt_ternary(const std::string& expression, const std::string& on_true) {
    return expression.empty() ? expression : fmt::format("{} ? {} : ", expression, on_true);
}

/// \param suppress_indirection parameter only
/// \returns expression to read merged member
std::string safe_read(
    const std::string& parameter_name,
    bool suppress_indirection,
    const MergeTrace& trace,
    std::vector<std::string>& fail_conditions
) {
    const ptree* parameter = trace.front();
    const ptree* elem = trace.back();
    if (elem == parameter) {  // not merged -> safe
        return parameter_name;
    }
    std::string accessor = ".";
    if (!suppress_indirection && is_shared(parameter)) {
        fail_conditions.push_back(fmt::format("!{}", parameter_name));
        accessor = "->";
    } else if (is_optional(parameter)) {
        fail_conditions.push_back(fmt::format("!{}.has_value()", parameter_name));
        accessor = ".value().";
    }
    return fmt::format("{}{}{}", parameter_name, accessor, public_member_name(elem));
}

/// \param suppress_indirection parameter only
/// \returns expression to copy merged member
std::string safe_copy(
    const std::string& parameter_name,
    bool suppress_indirection,
    const MergeTrace& trace,
    std::vector<std::string>& fail_conditions
) {
    const ptree* elem = trace.back();
    const ptree* parameter = trace.front();
    const ptree* parameter_members = base_type_of(parameter)->members;
    const ptree* elem_in_parameter_type = *std::find_if(
        begin(parameter_members),
        end(parameter_members),
        [&elem](const ptree* needle) { return elem->name == needle->name; }
    );

    std::string exp = safe_read(parameter_name, suppress_indirection, trace, fail_conditions);
    if (is_shared(elem)) {
        std::string deref;
        if ((!suppress_indirection && elem == parameter) || is_shared(elem_in_parameter_type)) {
            fail_conditions.push_back(fmt::format("!{}", exp));
            deref = "*";
        }
        exp = fmt::format(
            "std::unique_ptr<{}>(new {}({}{}))", type_name(elem), type_name(elem), deref, exp
        );
    }
    return exp;
}

/// \param suppress_indirection parameter only
/// \returns expression to copy merged member
std::string
safe_copy(const std::string& parameter_name, bool suppress_indirection, const MergeTrace& trace) {
    const ptree* elem = trace.back();
    std::vector<std::string> fail_conditions{};
    const std::string exp = safe_copy(parameter_name, suppress_indirection, trace, fail_conditions);
    const std::string fallback =
        is_optional(elem) ? fmt::format("intercom::optional<{}>{{}}", type_name(elem)) : "nullptr";
    return fmt::format("{}{}", opt_ternary(combine_conds(fail_conditions, " || "), fallback), exp);
}

/// \param suppress_indirection parameter only
/// \returns expression to move merged member
std::string
safe_move(const std::string& parameter_name, bool suppress_indirection, const MergeTrace& trace) {
    const ptree* elem = trace.back();
    std::vector<std::string> fail_conditions{};
    std::string exp = safe_read(parameter_name, suppress_indirection, trace, fail_conditions);
    const std::string fallback =
        is_optional(elem) ? fmt::format("intercom::optional<{}>{{}}", type_name(elem)) : "nullptr";
    return fmt::format(
        "{}std::move({})", opt_ternary(combine_conds(fail_conditions, " || "), fallback), exp
    );
}

/// \param suppress_indirection parameter only
static void cpl_gen_member_copy_ctor(
    struct memf* mfil,
    const ptree* obj,
    std::string_view prefix,
    std::string_view delim,
    bool suppress_indirection
) {
    std::string param;
    if (!obj) {
        return;
    }
    for (MergeTrace trace : get_merge_traces(obj)) {
        const ptree* parameter = trace.front();
        const ptree* elem = trace.back();
        if (elem->kind != N_MEMBER) {
            continue;
        }
        if (!prefix.empty()) {
            param = fmt::format("{}.{}", prefix, private_member_name(elem));
            trace = {elem};
        } else {
            param = cplpl_param_name(obj, name(parameter));
        }
        mprintf(mfil, "{}", delim);
        if (is_shared(elem) && elem->type->kind != N_INTERFACE) {
            mprintf(
                mfil,
                "{}({})",
                private_member_name(elem),
                safe_copy(param, suppress_indirection, trace)
            );
        } else {
            if (!is_pass_by_value(elem)) {
                mprintf(
                    mfil,
                    "{}({})",
                    private_member_name(elem),
                    safe_move(param, suppress_indirection, trace)
                );
            } else {
                mprintf(
                    mfil,
                    "{}({})",
                    private_member_name(elem),
                    safe_copy(param, suppress_indirection, trace)
                );
            }
        }
        delim = ",\n";
    }
}

/// \breif one external parameter to many internal members
/// the traces in \param members must all derive from the same member, i.e. .front() is identical
/// for all traces.
void emit_merged_getters_and_setters(const ptree* obj, const std::vector<MergeTrace>& members) {
    const ptree* parameter = members.front().front();
    emit_docs(&g_hd_file, parameter);

    std::string parameter_name = cplpl_param_name(obj, name(parameter));
    std::string parameter_type = cplpl_member_type(parameter, parameter->super);

    std::stringstream safe_get{};
    std::stringstream safe_set{};
    safe_get << fmt::format("{} res = {{}};\n", type_name(parameter));
    for (const MergeTrace& trace : members) {
        const ptree* elem = trace.back();
        const ptree* parameter_members = base_type_of(parameter)->members;
        const ptree* elem_in_parameter_type = *std::find_if(
            begin(parameter_members),
            end(parameter_members),
            [&elem](const ptree* needle) { return elem->name == needle->name; }
        );
        // get [\note this has to copy elem into parameter (inverted), and therefore can not rely on
        // safe_copy() & co.]
        std::vector<std::string> preconditions{};
        std::string copy_exp;
        if (is_shared(elem) && !is_shared(elem_in_parameter_type)) {
            preconditions.push_back(fmt::format("{}", private_member_name(elem)));
            copy_exp = fmt::format("*{}", private_member_name(elem));
        } else if (is_optional(elem) && !is_optional(elem_in_parameter_type)) {
            preconditions.push_back(fmt::format("{}.has_value()", private_member_name(elem)));
            copy_exp = fmt::format("{}.value()", private_member_name(elem));
        }
        if (!preconditions.empty()) {
            safe_get << fmt::format(
                "if ({}) {{\nres.{} = {};\n}}\n",
                combine_conds(preconditions, " && "),
                public_member_name(elem),
                copy_exp
            );
        } else {  // elem & parameter are similar enough that safe_copy() is probably fine
            safe_get << fmt::format(
                "res.{} = {};\n",
                public_member_name(elem),
                safe_copy(private_member_name(elem), false, {elem})
            );
        }
        // set
        safe_set << fmt::format(
            "{} = {};\n", private_member_name(elem), safe_copy(parameter_name, false, trace)
        );
    }
    safe_get << "return "
             << (is_shared(parameter) ? "std::unique_ptr<decltype(res)>{new decltype(res) {res}}"
                                      : "res")
             << ";\n";

    mprintf(
        &g_hd_file,
        "const {} {}() const {{\n{}}}\n",
        parameter_type,
        name(parameter),
        safe_get.str()
    );
    mprintf(
        &g_hd_file,
        "void {} (const {}& {}) {{\n{}}}\n\n",
        name(parameter),
        parameter_type,
        parameter_name,
        safe_set.str()
    );
}

void emit_getters_and_setters(const ptree* obj, const ptree* parameter) {
    emit_docs(&g_hd_file, parameter);

    std::string parameter_name = cplpl_param_name(obj, name(parameter));
    std::string parameter_type = cplpl_member_type(parameter, parameter->super);

    const std::string simple_get = fmt::format("return {};", private_member_name(parameter));
    const std::string simple_set = fmt::format(
        "{} = {};", private_member_name(parameter), safe_copy(parameter_name, false, {parameter})
    );

    if (is_pass_by_value(parameter)) {
        // get
        mprintf(&g_hd_file, "{}& {}() {{ {} }}\n", parameter_type, name(parameter), simple_get);
        mprintf(
            &g_hd_file, "{} {}() const {{ {} }}\n", parameter_type, name(parameter), simple_get
        );
        // set
        mprintf(
            &g_hd_file,
            "{}& {} ({} {}) {{ ",
            name(obj),
            name(parameter),
            parameter_type,
            parameter_name
        );
        // TODO use emit_range_check() instead [without serializer code]
        if (!is_optional(parameter) && (has_min_value(parameter) || has_max_value(parameter))) {
            const char* prefix = "\nif (";
            if (has_min_value(parameter)) {
                mprintf(&g_hd_file, "{} {} <", prefix, parameter_name);
                emit_const_value(&g_hd_file, get_min_value(parameter), nullptr, parameter);
                prefix = " || ";
            }
            if (has_max_value(parameter)) {
                mprintf(&g_hd_file, "{} {}>", prefix, parameter_name);
                emit_const_value(&g_hd_file, get_max_value(parameter), nullptr, parameter);
            }
            mprintf(&g_hd_file, ") {{\n");
            mprintf(
                &g_hd_file,
                "throw std::range_error(\"Illegal value for {}::{}\");\n",
                cpp_type_name(obj, nullptr),
                name(parameter)
            );
            mprintf(&g_hd_file, "}}\n");
        }
        mprintf(&g_hd_file, "{}return *this; }}\n\n", simple_set);
    } else {
        // get
        mprintf(&g_hd_file, "{}& {}() {{ {} }}\n", parameter_type, name(parameter), simple_get);
        mprintf(
            &g_hd_file,
            "const {}& {}() const {{ {} }}\n",
            parameter_type,
            name(parameter),
            simple_get
        );
        // set
        mprintf(
            &g_hd_file,
            "{}& {} (const {}& {}) {{ ",
            name(obj),
            name(parameter),
            parameter_type,
            parameter_name
        );
        mprintf(&g_hd_file, "{}", simple_set);
        mprintf(&g_hd_file, "return *this; }}\n");

        mprintf(
            &g_hd_file,
            "{}& {} ({}&& {}) {{ ",
            name(obj),
            name(parameter),
            parameter_type,
            parameter_name
        );
        mprintf(
            &g_hd_file,
            "using std::swap; swap({}, {});",
            private_member_name(parameter),
            parameter_name
        );
        mprintf(&g_hd_file, "return *this; }}\n\n");
    }
}

void cpl_gen_access_functions(const ptree* obj) {
    struct AccessSignature {
        const ptree* parameter;                //!< external facade
        std::vector<MergeTrace> merge_traces;  //!< actual internal data members
    };
    std::vector<AccessSignature> signatures{};
    for (std::vector<const ptree*>& merge_trace : get_merge_traces(obj)) {
        const ptree* parameter = merge_trace.front();
        if (!signatures.empty() && signatures.back().parameter == parameter) {
            signatures.back().merge_traces.push_back(merge_trace);  // merge
        } else {
            signatures.push_back({parameter, {merge_trace}});
        }
    }
    for (const AccessSignature& signature : signatures) {
        const bool is_merged =
            signature.merge_traces != decltype(signature.merge_traces){{signature.parameter}};
        const ptree* parameter = signature.parameter;
        if (parameter->kind != N_MEMBER) {
            continue;
        }
        // indirect access functions (one to many)
        if (is_merged) {
            emit_merged_getters_and_setters(obj, signature.merge_traces);
        }
        if (!CommandLineOption::cpp_access_functions()) {
            continue;
        }
        // direct access functions (one to one)
        for (const MergeTrace& trace : signature.merge_traces) {
            const ptree* elem = trace.back();
            emit_getters_and_setters(obj, elem);
        }
    }
}

static void cpl_constr_gen_lessthan(struct memf* mfil, const ptree* obj, int is_parent) {
    for (auto parent : obj->parents) {
        cpl_constr_gen_lessthan(mfil, parent, 1);
    }
    auto param = cplpl_param_name(obj, "other");
    if (obj->kind == N_UNION) {
        mprintf(mfil, "if (_d() < {}._d()) {{ return true; }}\n", param);
        mprintf(mfil, "if ({}._d() < _d()) {{ return false; }}\n", param);
        mprintf(mfil, "switch (_d()) {{\n");

        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                cpl_gen_cases(mfil, elem, obj);
                if (is_shared(elem)) {
                    mprintf(
                        mfil,
                        "if (!this->{} || !{}.{}) {{\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                    mprintf(
                        mfil,
                        "return this->{} < {}.{};\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                    mprintf(mfil, "}}\n");
                    mprintf(
                        mfil,
                        "return *(this->{}) < *{}.{};\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                } else {
                    mprintf(
                        mfil,
                        "return this->{} < {}.{};\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                }
            } else if (elem->kind == N_NULL) {
                cpl_gen_cases(mfil, elem, obj);
                mprintf(mfil, "return false;\n");
            }
        }
        mprintf(mfil, "}}\n");
        if (!has_default_case(obj)) {
            mprintf(mfil, "return false;\n");
        }
    } else if (obj->kind == N_STRUCT || obj->kind == N_VALUETYPE || obj->kind == N_EXCEPTION) {
        int has_member = 0;
        const ptree* elem = obj->members;
        while (elem && elem->kind != N_MEMBER) {
            elem = elem->next;
        }
        while (elem) {
            auto member_name = public_member_name(elem);
            has_member = 1;
            const ptree* next = elem->next;
            while (next && next->kind != N_MEMBER) {
                next = next->next;
            }

            if (is_shared(elem)) {
                mprintf(mfil, "if (!this->{} || !{}.{}) {{\n", member_name, param, member_name);
                if (next || is_parent) {
                    mprintf(
                        mfil,
                        "if (this->{} != {}.{}) {{ return this->{} < {}.{}; }}\n",
                        member_name,
                        param,
                        member_name,
                        member_name,
                        param,
                        member_name
                    );
                    mprintf(mfil, "}} else {{\n");
                    mprintf(
                        mfil,
                        "if (*(this->{}) < *{}.{}) {{ return true; }}\n",
                        member_name,
                        param,
                        member_name
                    );
                    mprintf(
                        mfil,
                        "if (*{}.{} < *(this->{})) {{ return false; }}\n",
                        param,
                        member_name,
                        member_name
                    );
                } else {
                    mprintf(mfil, "return this->{} < {}.{};\n", member_name, param, member_name);
                    mprintf(mfil, "}} else {{\n");
                    mprintf(
                        mfil, "return *(this->{}) < *{}.{};\n", member_name, param, member_name
                    );
                }
                mprintf(mfil, "}}\n");
            } else {
                if (next || is_parent) {
                    mprintf(
                        mfil,
                        "if (this->{} < {}.{}) {{ return true; }}\n",
                        member_name,
                        param,
                        member_name
                    );
                    mprintf(
                        mfil,
                        "if ({}.{} < this->{}) {{ return false; }}\n",
                        param,
                        member_name,
                        member_name
                    );
                } else {
                    mprintf(mfil, "return this->{} < {}.{};\n", member_name, param, member_name);
                }
            }
            elem = next;
        }
        if (!has_member && !is_parent) {
            mprintf(mfil, "return false;\n");
        }
    }
}

static void cpl_constr_gen_equal(struct memf* mfil, const ptree* obj, int is_parent) {
    auto param = cplpl_param_name(obj, "other");

    if (obj->kind == N_UNION) {
        mprintf(mfil, "if (!(_d() == {}._d())) return false;\n", param);
        mprintf(mfil, "switch (_d()) {{\n");
        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                cpl_gen_cases(mfil, elem, obj);
                if (is_shared(elem)) {
                    mprintf(
                        mfil,
                        "if (this->{} == {}.{}) {{ return true; }}\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                    mprintf(
                        mfil,
                        "if (!this->{} || !{}.{}) {{ return false; }}\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                    mprintf(
                        mfil,
                        "return *(this->{}) == *{}.{};\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                } else {
                    mprintf(
                        mfil,
                        "return this->{} == {}.{};\n",
                        public_member_name(elem),
                        param,
                        public_member_name(elem)
                    );
                }
            } else if (elem->kind == N_NULL) {
                cpl_gen_cases(mfil, elem, obj);
                mprintf(mfil, "return true;\n");
            }
        }
        mprintf(mfil, "}}\n");
        if (!has_default_case(obj)) {
            mprintf(mfil, "return true;\n");
        }
    } else if (obj->kind == N_STRUCT || obj->kind == N_VALUETYPE || obj->kind == N_EXCEPTION) {
        for (auto parent : obj->parents) {
            cpl_constr_gen_equal(mfil, parent, 1);
        }
        const ptree* elem = obj->members;
        while (elem && elem->kind != N_MEMBER) {
            elem = elem->next;
        }
        while (elem) {
            auto member_name = public_member_name(elem);
            const ptree* next = elem->next;
            while (next && next->kind != N_MEMBER) {
                next = next->next;
            }

            if (is_shared(elem)) {
                mprintf(mfil, "if (!(this->{} == {}.{})) {{\n", member_name, param, member_name);
                mprintf(
                    mfil,
                    "if (!this->{} || !{}.{}) {{ return false; }}\n",
                    member_name,
                    param,
                    member_name
                );
                mprintf(
                    mfil,
                    "if (!(*this->{} == *{}.{})) {{ return false; }}\n",
                    member_name,
                    param,
                    member_name
                );
                mprintf(mfil, "}}\n");
            } else {
                mprintf(
                    mfil,
                    "if (!(this->{} == {}.{})) {{ return false; }}\n",
                    member_name,
                    param,
                    member_name
                );
            }
            elem = next;
        }
        if (!is_parent) {
            mprintf(mfil, "return true;\n");
        }
    }
}

static void cpl_comparator_def(const ptree* obj) {
    auto body_name = scoped_name(obj, namespace_of(obj));
    auto param = cplpl_param_name(obj, "other");

    if (member_count(obj) > 0) {
        mprintf(&g_hd_file, "bool operator<(const {} & {}) const;\n", name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline bool {}::operator<(const {} & {}) const {{\n",
            body_name,
            body_name,
            param
        );
        cpl_constr_gen_lessthan(&g_hd_impl_file, obj, 0);
        mprintf(&g_hd_impl_file, "}}\n\n");

        mprintf(&g_hd_file, "bool operator==(const {} & {}) const;\n", name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline bool {}::operator==(const {} & {}) const {{\n",
            body_name,
            body_name,
            param
        );
        cpl_constr_gen_equal(&g_hd_impl_file, obj, 0);
        mprintf(&g_hd_impl_file, "}}\n\n");

        mprintf(&g_hd_file, "bool operator!=(const {} & {}) const {{ ", name(obj), param);
        mprintf(&g_hd_file, "return !(*this == {});", param);
        mprintf(&g_hd_file, " }}\n");

        mprintf(&g_hd_file, "bool operator>(const {} & {}) const {{ ", name(obj), param);
        mprintf(&g_hd_file, "return {} < *this;", param);
        mprintf(&g_hd_file, " }}\n");

        mprintf(&g_hd_file, "bool operator<=(const {} & {}) const {{ ", name(obj), param);
        mprintf(&g_hd_file, "return !({} < *this);", param);
        mprintf(&g_hd_file, " }}\n");

        mprintf(&g_hd_file, "bool operator>=(const {} & {}) const {{ ", name(obj), param);
        mprintf(&g_hd_file, "return !(*this < {});", param);
        mprintf(&g_hd_file, " }}\n\n");
    } else {
        mprintf(&g_hd_file, "bool operator<(const {} &) const {{ return false; }}\n", name(obj));
        mprintf(&g_hd_file, "bool operator!=(const {} &) const {{ return false; }}\n", name(obj));
        mprintf(&g_hd_file, "bool operator==(const {} &) const {{ return true; }}\n", name(obj));
        mprintf(&g_hd_file, "bool operator>(const {} &) const {{ return false; }}\n", name(obj));
        mprintf(&g_hd_file, "bool operator<=(const {} &) const {{ return true; }}\n", name(obj));
        mprintf(&g_hd_file, "bool operator>=(const {} &) const {{ return true; }}\n", name(obj));
    }
}

static void cpl_iostream_def(const ptree* obj) {
    if (!is_non_serialized(obj)) {
        if (!CommandLineOption::cpp_no_stream_op()) {
            mprintf(
                &g_hd_json_file,
                "inline std::ostream& operator<<(std::ostream& stream, const {}& value) {{\n",
                cpp_type_name(obj, namespace_of(obj))
            );
            mprintf(&g_hd_json_file, "return intercom::marshal_json(stream, value);\n");
            mprintf(&g_hd_json_file, "}}\n\n");

            mprintf(
                &g_hd_json_file,
                "inline std::istream& operator>>(std::istream& stream, {}& value) {{\n",
                cpp_type_name(obj, namespace_of(obj))
            );
            mprintf(&g_hd_json_file, "return intercom::unmarshal_json(stream, value);\n");
            mprintf(&g_hd_json_file, "}}\n\n");
        }
        if (CommandLineOption::use_fmtlib()) {
            mprintf(
                &g_hd_fmt_file,
                "template <> struct formatter<{}> : ostream_formatter {{}};\n",
                cpp_type_name(obj, nullptr)
            );
        }
    }
}

static void cpl_union_construct_body(const ptree* obj, std::string_view param, bool move) {
    mprintf(&g_hd_impl_file, "ic_discriminator_value_ = {}.ic_discriminator_value_;\n", param);
    if (member_count(obj) == 0) {
        return;
    }
    mprintf(&g_hd_impl_file, "switch (ic_discriminator_value_) {{\n");
    for (auto lhs : obj->members) {
        if (lhs->kind == N_MEMBER) {
            cpl_gen_cases(&g_hd_impl_file, lhs, obj);
            if (is_pass_by_value(lhs)) {
                mprintf(
                    &g_hd_impl_file,
                    "ic_union_value_.{} = {}.ic_union_value_.{};\n",
                    name(lhs),
                    param,
                    name(lhs)
                );
            } else if (move) {
                mprintf(
                    &g_hd_impl_file,
                    "intercom::construct_at(&ic_union_value_.{}, std::move({}.ic_union_value_.{}));\n",
                    name(lhs),
                    param,
                    name(lhs)
                );
            } else if (is_shared(lhs)) {
                mprintf(
                    &g_hd_impl_file,
                    "intercom::construct_at(&ic_union_value_.{}, new {}( *{}.ic_union_value_.{}));\n",
                    name(lhs),
                    cpp_type_name(lhs->type, obj),
                    param,
                    name(lhs)
                );
            } else {
                mprintf(
                    &g_hd_impl_file,
                    "intercom::construct_at(&ic_union_value_.{}, {}.ic_union_value_.{});\n",
                    name(lhs),
                    param,
                    name(lhs)
                );
            }
            mprintf(&g_hd_impl_file, "break;\n");
        }
        if (lhs->kind == N_NULL) {
            cpl_gen_cases(&g_hd_impl_file, lhs, obj);
            mprintf(&g_hd_impl_file, "break;\n");
        }
    }
    mprintf(&g_hd_impl_file, "}}\n");
}

static void
cpl_union_assignment_body(const ptree* obj, std::string_view param, bool has_ptr, bool move) {
    if (has_ptr) {
        mprintf(&g_hd_impl_file, "free_union_();\n");
    }
    cpl_union_construct_body(obj, param, move);
}

static const ptree* get_default_member(const ptree* obj) {
    const ptree* res = nullptr;
    for (const ptree* mem : obj->members) {
        for (auto lhs : mem->members) {
            if (lhs->flags & OPT_DEFAULT) {
                if (!res) {
                    res = mem;
                }
            } else if (integer_value(lhs->value) ==
                       integer_value(get_default_value(obj->discriminator))) {
                return mem;
            }
        }
    }
    return res;
}

static void cpl_union_gen_getters(
    const ptree* obj,
    const ptree* member,
    bool by_value,
    const std::string& body_name
) {
    const auto member_name = name(member);
    const auto member_type = cplpl_member_type(member, member->super);

    mprintf(&g_hd_file, "{}& {}();\n", member_type, member_name);
    mprintf(&g_hd_impl_file, "inline {}& {}::{}() {{\n", member_type, body_name, member_name);
    mprintf(&g_hd_impl_file, "if ( ");
    gen_case_test(&g_hd_impl_file, obj, member);
    mprintf(
        &g_hd_impl_file,
        " ) {{\nthrow std::logic_error(\"Union {} not set to value {}\");\n}}\n",
        name(obj),
        member_name
    );
    mprintf(&g_hd_impl_file, "return ic_union_value_.{};\n}}\n\n", member_name);

    if (by_value) {
        mprintf(&g_hd_file, "{} {}() const;\n", member_type, member_name);
        mprintf(
            &g_hd_impl_file, "inline {} {}::{}() const {{\n", member_type, body_name, member_name
        );
    } else {
        mprintf(&g_hd_file, "const {}& {}() const;\n", member_type, member_name);
        mprintf(
            &g_hd_impl_file,
            "inline const {}& {}::{}() const {{\n",
            member_type,
            body_name,
            member_name
        );
    }
    mprintf(&g_hd_impl_file, "if ( ");
    gen_case_test(&g_hd_impl_file, obj, member);
    mprintf(
        &g_hd_impl_file,
        " ) {{\nthrow std::logic_error(\"Union {} not set to value {}\");\n}}\n",
        name(obj),
        member_name
    );
    mprintf(&g_hd_impl_file, "return ic_union_value_.{};\n}}\n\n", member_name);
}

namespace {
enum UnionSetterKind {
    set_by_value,
    set_by_rvalue_ref,
    set_by_ref,
};
}

static void cpl_union_gen_setter(
    const ptree* obj,
    const ptree* member,
    bool has_ptr,
    UnionSetterKind kind,
    const std::string& body_name,
    const std::string& discr_name,
    const std::string& param_name
) {
    const auto member_name = name(member);
    const auto member_type = cplpl_member_type(member, member->super);

    switch (kind) {
    case set_by_value:
        mprintf(&g_hd_file, "void {} ({} {});\n", member_name, member_type, param_name);
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}({} {}) {{\n",
            body_name,
            member_name,
            member_type,
            param_name
        );
        break;
    case set_by_rvalue_ref:
        mprintf(&g_hd_file, "void {}({}&& {});\n", member_name, member_type, param_name);
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}({}&& {}) {{\n",
            body_name,
            member_name,
            member_type,
            param_name
        );
        break;
    case set_by_ref:
        mprintf(&g_hd_file, "void {}(const {}& {});\n", member_name, member_type, param_name);
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}(const {}& {}) {{\n",
            body_name,
            member_name,
            member_type,
            param_name
        );
        break;
    }

    mprintf(&g_hd_impl_file, "if ( ");
    gen_case_test(&g_hd_impl_file, obj, member);
    mprintf(&g_hd_impl_file, " ) {{\n");
    if (has_ptr) {
        mprintf(&g_hd_impl_file, "free_union_();\n");
    }
    mprintf(&g_hd_impl_file, "ic_discriminator_value_ =");
    emit_const_value(&g_hd_impl_file, member->members->value, obj->super, obj);
    mprintf(&g_hd_impl_file, ";\n");
    switch (kind) {
    case set_by_value:
        mprintf(&g_hd_impl_file, "}}\n");
        mprintf(&g_hd_impl_file, "ic_union_value_.{} = {};\n", member_name, param_name);
        break;
    case set_by_rvalue_ref:
        mprintf(
            &g_hd_impl_file,
            "intercom::construct_at(&ic_union_value_.{}, std::move({}));\n",
            member_name,
            param_name
        );
        mprintf(&g_hd_impl_file, "}} else {{\n");
        mprintf(&g_hd_impl_file, "ic_union_value_.{} = std::move({});\n", member_name, param_name);
        mprintf(&g_hd_impl_file, "}}\n");
        break;
    case set_by_ref:
        if (is_shared(member)) {
            mprintf(
                &g_hd_impl_file,
                "intercom::construct_at(&ic_union_value_.{}, new {}(*{}));\n",
                member_name,
                cpp_type_name(member->type, obj),
                param_name
            );
            mprintf(&g_hd_impl_file, "}} else {{\n");
            mprintf(
                &g_hd_impl_file,
                "ic_union_value_.{}.reset(new {}(*{}));\n",
                member_name,
                cpp_type_name(member->type, obj),
                param_name
            );
            mprintf(&g_hd_impl_file, "}}\n");
        } else {
            mprintf(
                &g_hd_impl_file,
                "intercom::construct_at(&ic_union_value_.{}, {});\n",
                member_name,
                param_name
            );
            mprintf(&g_hd_impl_file, "}} else {{\n");
            mprintf(&g_hd_impl_file, "ic_union_value_.{} = {};\n", member_name, param_name);
            mprintf(&g_hd_impl_file, "}}\n");
        }
        break;
    }
    mprintf(&g_hd_impl_file, "}}\n\n");

    if (!has_multiple_cases(member)) {
        return;
    }
    switch (kind) {
    case set_by_value:
        mprintf(
            &g_hd_file,
            "void {}({} {}, {} discriminator);\n",
            member_name,
            member_type,
            param_name,
            discr_name
        );
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}({} {}, {} discriminator) {{\n",
            body_name,
            member_name,
            member_type,
            param_name,
            discr_name
        );
        break;
    case set_by_rvalue_ref:
        mprintf(
            &g_hd_file,
            "void {}({}&& {}, {} discriminator);\n",
            member_name,
            member_type,
            param_name,
            discr_name
        );
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}({}&& {}, {} discriminator) {{\n",
            body_name,
            member_name,
            member_type,
            param_name,
            discr_name
        );
        break;
    case set_by_ref:
        mprintf(
            &g_hd_file,
            "void {}(const {}& {}, {} discriminator);\n",
            member_name,
            member_type,
            param_name,
            discr_name
        );
        mprintf(
            &g_hd_impl_file,
            "inline void {}::{}(const {}& {}, {} discriminator) {{\n",
            body_name,
            member_name,
            member_type,
            param_name,
            discr_name
        );
        break;
    }

    mprintf(&g_hd_impl_file, "if (");
    gen_case_test(&g_hd_impl_file, obj, member, "discriminator");
    mprintf(
        &g_hd_impl_file,
        ") {{\nthrow std::logic_error(\"Illegal discriminator for member {} of union {}\");\n}}\n",
        member_name,
        name(obj)
    );
    mprintf(&g_hd_impl_file, "if (");
    gen_case_test(&g_hd_impl_file, obj, member);
    mprintf(&g_hd_impl_file, ") {{\n");
    if (has_ptr) {
        mprintf(&g_hd_impl_file, "free_union_();\n");
    }
    switch (kind) {
    case set_by_value:
        mprintf(&g_hd_impl_file, "}}\n");
        mprintf(&g_hd_impl_file, "ic_union_value_.{} = {};\n", member_name, param_name);
        break;
    case set_by_rvalue_ref:
        mprintf(
            &g_hd_impl_file,
            "intercom::construct_at(&ic_union_value_.{}, std::move({}));\n",
            member_name,
            param_name
        );
        mprintf(&g_hd_impl_file, "}} else {{\n");
        mprintf(&g_hd_impl_file, "ic_union_value_.{} = std::move({});\n", member_name, param_name);
        mprintf(&g_hd_impl_file, "}}\n");
        break;
    case set_by_ref:
        if (is_shared(member)) {
            mprintf(
                &g_hd_impl_file,
                "intercom::construct_at(&ic_union_value_.{}, new {}(*{}));\n",
                member_name,
                cpp_type_name(member->type, obj),
                param_name
            );
            mprintf(&g_hd_impl_file, "}} else {{\n");
            mprintf(
                &g_hd_impl_file,
                "ic_union_value_.{}.reset(new {}(*{}));\n",
                member_name,
                cpp_type_name(member->type, obj),
                param_name
            );
            mprintf(&g_hd_impl_file, "}}\n");
        } else {
            mprintf(
                &g_hd_impl_file,
                "intercom::construct_at(&ic_union_value_.{}, {});\n",
                member_name,
                param_name
            );
            mprintf(&g_hd_impl_file, "}} else {{\n");
            mprintf(&g_hd_impl_file, "ic_union_value_.{} = {};\n", member_name, param_name);
            mprintf(&g_hd_impl_file, "}}\n");
        }
        break;
    }
    mprintf(&g_hd_impl_file, "ic_discriminator_value_ = discriminator;\n");
    mprintf(&g_hd_impl_file, "}}\n\n");
}

static void cpl_union_c_def(const ptree* obj) {
    bool has_ptr = false;
    auto body_name = scoped_name(obj, namespace_of(obj));
    auto discr_name = scoped_name(obj->discriminator->type, obj);
    auto param = cplpl_param_name(obj, "other");
    auto p_first = cplpl_param_name(obj, "first");
    auto p_second = cplpl_param_name(obj, "second");

    for (const ptree* lhs = obj->members; lhs; lhs = lhs->next) {
        if (lhs->kind == N_MEMBER && !is_pass_by_value(lhs)) {
            has_ptr = true;
            break;
        }
    }

    mprintf(&g_hd_file, "struct {} {{", name(obj));
    emit_post_docs(&g_hd_file, obj);

    if (!get_annotation(obj, annotation_type_ext_no_constructor)) {
        // Generate default constructor
        mprintf(&g_hd_file, "{}();\n", name(obj));
        mprintf(&g_hd_impl_file, "inline {}::{}() {{\n", body_name, name(obj));
        mprintf(&g_hd_impl_file, "ic_discriminator_value_ =");
        emit_default_value(&g_hd_impl_file, obj->discriminator, obj);
        mprintf(&g_hd_impl_file, ";\n");

        const ptree* default_mem = get_default_member(obj);
        if (default_mem && default_mem->kind != N_NULL) {
            if (is_pass_by_value(default_mem)) {
                mprintf(&g_hd_impl_file, "ic_union_value_.{} =", name(default_mem));
                emit_default_value(&g_hd_impl_file, default_mem, namespace_of(obj));
            } else {
                mprintf(
                    &g_hd_impl_file,
                    "intercom::construct_at(&ic_union_value_.{},",
                    name(default_mem)
                );
                if (has_default_value(default_mem)) {
                    emit_default_value(&g_hd_impl_file, default_mem, namespace_of(obj));
                } else {
                    emit_void_value(&g_hd_impl_file, default_mem, namespace_of(obj));
                }
                mprintf(&g_hd_impl_file, ")");
            }
            mprintf(&g_hd_impl_file, ";\n");
        }
        mprintf(&g_hd_impl_file, "}}\n\n");

        // Generate copy constructor
        mprintf(&g_hd_file, "{}(const {} & {});\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline {}::{}(const {} & {}) {{\n",
            body_name,
            name(obj),
            body_name,
            param
        );
        cpl_union_construct_body(obj, param, false);
        mprintf(&g_hd_impl_file, "}}\n\n");

        // Generate assignment operator
        mprintf(&g_hd_file, "{}& operator=(const {}& {});\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline {}& {}::operator=(const {}& {}) {{\n",
            body_name,
            body_name,
            name(obj),
            param
        );
        if (member_count(obj) > 0) {
            mprintf(&g_hd_impl_file, "if (this != &{}) {{\n", param);
            cpl_union_assignment_body(obj, param, has_ptr, false);
            mprintf(&g_hd_impl_file, "}}\n");
        }
        mprintf(&g_hd_impl_file, "\nreturn *this;\n}}\n\n");

        // Generate move constructor
        mprintf(&g_hd_file, "{}({} && {}) noexcept;\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline {}::{}({} && {}) noexcept : {}() {{\n",
            body_name,
            name(obj),
            body_name,
            param,
            name(obj)
        );
        cpl_union_construct_body(obj, param, true);
        mprintf(&g_hd_impl_file, "}}\n\n");

        // Generate move assignment operator
        mprintf(&g_hd_file, "{}& operator=({}&& {}) noexcept;\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline {}& {}::operator=({}&& {}) noexcept {{\n",
            body_name,
            body_name,
            name(obj),
            param
        );
        mprintf(&g_hd_impl_file, "if (this != &{}) {{\n", param);
        cpl_union_assignment_body(obj, param, has_ptr, true);
        mprintf(&g_hd_impl_file, "}}\n");
        mprintf(&g_hd_impl_file, "return *this;\n");
        mprintf(&g_hd_impl_file, "}}\n\n");

        // Generate destructor
        mprintf(&g_hd_file, "~{}() noexcept;\n\n", name(obj));
        mprintf(&g_hd_impl_file, "inline {}::~{}() noexcept {{\n", body_name, name(obj));
        if (has_ptr) {
            mprintf(&g_hd_impl_file, "free_union_();\n");
        }
        mprintf(&g_hd_impl_file, "}}\n\n");
    }

    // Comparison operators
    cpl_comparator_def(obj);

    // Swap function
    mprintf(
        &g_hd_file,
        "friend void swap({}& {}, {}& {}) noexcept;\n\n",
        name(obj),
        p_first,
        name(obj),
        p_second
    );
    mprintf(
        &g_hd_impl_file,
        "inline void swap({}& {}, {}& {}) noexcept {{\n",
        body_name,
        p_first,
        body_name,
        p_second
    );
    mprintf(&g_hd_impl_file, "{} {}_tmp = std::move({});\n", body_name, p_first, p_first);
    mprintf(&g_hd_impl_file, "{} = std::move({});\n", p_first, p_second);
    mprintf(&g_hd_impl_file, "{} = std::move({}_tmp);\n", p_second, p_first);
    mprintf(&g_hd_impl_file, "}}\n\n");

    // Generate _d() functions
    mprintf(&g_hd_file, "{} _d() const {{ return ic_discriminator_value_; }}\n", discr_name);
    mprintf(&g_hd_file, "void _d({} discriminator);\n\n", discr_name);
    mprintf(&g_hd_impl_file, "inline void {}::_d({} discriminator) {{\n", body_name, discr_name);

    if (has_ptr) {
        mprintf(&g_hd_impl_file, "switch (discriminator) {{\n");
        bool has_default = false;
        for (const ptree* lhs = obj->members; lhs; lhs = lhs->next) {
            if (lhs->kind == N_MEMBER) {
                if (cpl_gen_cases(&g_hd_impl_file, lhs, obj)) {
                    has_default = true;
                }
                mprintf(&g_hd_impl_file, "if (");
                gen_case_test(&g_hd_impl_file, obj, lhs);
                mprintf(&g_hd_impl_file, ") {{\n");
                mprintf(&g_hd_impl_file, "free_union_();\n");
                if (!is_pass_by_value(lhs)) {
                    mprintf(
                        &g_hd_impl_file, "intercom::construct_at(&ic_union_value_.{},", name(lhs)
                    );
                    if (has_default_value(lhs)) {
                        emit_default_value(&g_hd_impl_file, lhs, obj);
                    } else {
                        emit_void_value(&g_hd_impl_file, lhs, namespace_of(obj));
                    }
                    mprintf(&g_hd_impl_file, ");\n");
                } else {
                    mprintf(&g_hd_impl_file, "ic_union_value_.{} =", name(lhs));
                    emit_default_value(&g_hd_impl_file, lhs, obj);
                    mprintf(&g_hd_impl_file, ";\n");
                }
                mprintf(&g_hd_impl_file, "}}\n");
                mprintf(&g_hd_impl_file, "break;\n");
            }
            if (lhs->kind == N_NULL) {
                if (cpl_gen_cases(&g_hd_impl_file, lhs, obj)) {
                    has_default = true;
                }
                mprintf(&g_hd_impl_file, "free_union_();\n");
                mprintf(&g_hd_impl_file, "break;\n");
            }
        }
        if (!has_default) {
            mprintf(&g_hd_impl_file, "default:\n");
            mprintf(
                &g_hd_impl_file,
                "throw std::logic_error(\"Illegal discriminator value for union {}\");",
                name(obj)
            );
        }
        mprintf(&g_hd_impl_file, "}}\n");
    }
    mprintf(&g_hd_impl_file, "ic_discriminator_value_ = discriminator;\n");
    mprintf(&g_hd_impl_file, "}}\n\n");

    // Generate setters and getters
    const auto param_name = cplpl_param_name(obj, "value");
    for (const ptree* lhs = obj->members; lhs; lhs = lhs->next) {
        if (lhs->kind == N_MEMBER) {
            bool by_value = is_pass_by_value(lhs);
            cpl_union_gen_getters(obj, lhs, by_value, body_name);
            if (by_value) {
                cpl_union_gen_setter(
                    obj, lhs, has_ptr, set_by_value, body_name, discr_name, param_name
                );
            } else {
                cpl_union_gen_setter(
                    obj, lhs, has_ptr, set_by_ref, body_name, discr_name, param_name
                );
                cpl_union_gen_setter(
                    obj, lhs, has_ptr, set_by_rvalue_ref, body_name, discr_name, param_name
                );
            }
            mprintf(&g_hd_file, "\n");
        }
    }

    // Generate private members
    mprintf(&g_hd_file, "private:\nunion ICUnionType_ {{\n");
    mprintf(&g_hd_file, "ICUnionType_() {{}}\n");
    mprintf(&g_hd_file, "~ICUnionType_() {{}}\n");
    for (const ptree* lhs = obj->members; lhs; lhs = lhs->next) {
        switch (lhs->kind) {
        case N_MEMBER: {
            emit_docs(&g_hd_file, lhs);
            mprintf(&g_hd_file, "{} {};", cplpl_member_type(lhs, lhs->super), name(lhs));
            emit_post_docs(&g_hd_file, lhs);
        } break;
        default:
            break;
        }
    }
    mprintf(&g_hd_file, "}} ic_union_value_;\n");
    mprintf(&g_hd_file, "{} ic_discriminator_value_;\n", discr_name);

    // Generate private free_union_ function
    if (has_ptr) {
        mprintf(&g_hd_file, "void free_union_();\n");

        mprintf(
            &g_hd_impl_file,
            "inline void {}::free_union_() {{\nswitch (ic_discriminator_value_) {{\n",
            body_name
        );

        for (const ptree* lhs = obj->members; lhs; lhs = lhs->next) {
            if (lhs->kind == N_MEMBER) {
                cpl_gen_cases(&g_hd_impl_file, lhs, obj);
                if (!is_pass_by_value(lhs)) {
                    std::string type_name = cpp_type_name(base_type_of(lhs), nullptr);
                    auto pos = type_name.rfind("::", type_name.find('<'));
                    if (pos != std::string::npos) {
                        type_name = type_name.substr(pos + 2);
                    }
                    mprintf(
                        &g_hd_impl_file, "intercom::destroy_at(&ic_union_value_.{});\n", name(lhs)
                    );
                }
                mprintf(&g_hd_impl_file, "break;\n");
            }
            if (lhs->kind == N_NULL) {
                cpl_gen_cases(&g_hd_impl_file, lhs, obj);
                mprintf(&g_hd_impl_file, "break;\n");
            }
        }
        mprintf(&g_hd_impl_file, "}}\n");
        mprintf(&g_hd_impl_file, "}}\n\n");
    }

    mprintf(&g_hd_file, "}};\n");
}

static std::string renamed_name(const ptree* obj) {
    if (auto rename = get_annotation(obj, annotation_type_ext_rename)) {
        return string_value(get_annotation_value(rename, "name"));
    }
    return name(obj);
}

static std::string str_to_upper(std::string val) {
    std::transform(val.begin(), val.end(), val.begin(), toupper);
    return val;
};

/// \n will print namespaces with names defined using annotation \@ext::rename
/// \nb hardcodes a lot of values
static void cpl_property_value_constants_rec(
    const ptree* obj,
    std::deque<const ptree*>& member_trace,
    const std::string& initial_namespace
) {
    // backwards compatability
    auto hardcode_namespace = [](const std::string& node_name) -> std::string {
        static const std::map<std::string, std::string> s_replacements{
            {"sockets", "Socket"}, {"dds", ""}, {"persistence", ""}
        };
        auto rep = s_replacements.find(node_name);
        if (rep != s_replacements.end()) {
            return rep->second;
        }
        std::string name = node_name;
        if (!name.empty()) {  // capitalize first letter
            name.front() = static_cast<char>(std::toupper(static_cast<int>(name.front())));
        }
        return name;
    };
    // backwards compatability
    auto hardcode_variable_name = [&member_trace](const std::string& node_name) -> std::string {
        using KeyT = std::pair<std::string, std::string>;  // {parent_name, variable_name}
        static const std::map<KeyT, std::string> s_replacements{
            {{"persistence", "location"}, "persistence_location"},
            {{"DomainParticipant", "viewer_access"}, "viewersupport"}
        };
        const std::string parent =
            member_trace.size() > 1 ? renamed_name(member_trace[member_trace.size() - 2]) : "";
        auto rep = s_replacements.find(KeyT{parent, node_name});
        if (rep != s_replacements.end()) {
            return rep->second;
        }
        return node_name;
    };
    auto emit_value = [&member_trace, &hardcode_namespace, &initial_namespace](
                          const std::string& variable_name, const std::string& value
                      ) -> void {
        const std::string type =
            CommandLineOption::cpp_gen_cpp11() ? "const std::string" : "const char*";
        mprintf(&g_hd_file, INTERCOM_PUBLIC_MACRO_NAME " extern {} {};\n", type, variable_name);
        mprintf(&g_tbd_file, "{} {}::", type, initial_namespace);
        for (const auto& trace : member_trace) {
            if (base_type_of(trace)->kind != N_STRUCT && base_type_of(trace)->kind != N_UNION) {
                continue;
            }
            std::string name = hardcode_namespace(renamed_name(trace));
            if (!name.empty()) {
                mprintf(&g_tbd_file, "{}::", name);
            }
        }
        mprintf(&g_tbd_file, "{} = \"{}\";\n", variable_name, value);
    };
    // backwards compatability
    auto hardcode_extra_values = [&emit_value](const std::string& node_name) -> void {
        using ValueT = std::pair<std::string, std::string>;  // {variable_name, value}
        static const std::map<std::string, std::vector<ValueT>> s_extra_values{
            // note: the dds namespace is squashed in hardcode_namespace()
            {"dds", {{"GROUP", "PROPERTIES.GROUP"}}},
            {"checksum",
             {{"VALUE_METHOD_NONE", "none"},
              {"VALUE_METHOD_SIMPLE", "simple"},
              {"VALUE_METHOD_MD5", "md5"},
              {"VALUE_METHOD_CRC32", "crc32"},
              {"VALUE_METHOD_CRC32C", "crc32c"},
              {"VALUE_METHOD_CRC64", "crc64"},
              {"VALUE_REQUIRED", "true"},
              {"VALUE_OPTIONAL", "false"}}},
            {"security",
             {{"AUTH_IDENTITY_CA", "dds.sec.auth.identity_ca"},
              {"AUTH_IDENTITY_CERTIFICATE", "dds.sec.auth.identity_certificate"},
              {"AUTH_PRIVATE_KEY", "dds.sec.auth.private_key"},
              {"AUTH_PASSWORD", "dds.sec.auth.password"},
              {"ACCESS_PERMISSIONS_CA", "dds.sec.access.permissions_ca"},
              {"ACCESS_PERMISSIONS", "dds.sec.access.permissions"},
              {"ACCESS_GOVERNANCE", "dds.sec.access.governance"}}}
        };
        auto values = s_extra_values.find(node_name);
        if (values == s_extra_values.end()) {
            return;
        }
        for (const ValueT& val : values->second) {
            emit_value(val.first, val.second);
        }
    };
    const ptree* base_type = base_type_of(obj);
    switch (obj->kind) {
    case N_STRUCT:
    case N_UNION:
        for (const ptree* member : obj->members) {
            cpl_property_value_constants_rec(member, member_trace, initial_namespace);
        }
        break;
    case N_MEMBER:
        member_trace.push_back(obj);
        if (base_type->kind == N_STRUCT || base_type->kind == N_UNION) {
            const std::string node_name = renamed_name(obj);
            std::string name = hardcode_namespace(node_name);
            if (!name.empty()) {
                mprintf(&g_hd_file, "namespace {} {{\n", name);
            }
            cpl_property_value_constants_rec(base_type, member_trace, initial_namespace);
            hardcode_extra_values(node_name);
            if (!name.empty()) {
                mprintf(&g_hd_file, "}} // namespace {}\n", name);
            }
        } else {
            const std::string node_name = renamed_name(obj);
            const std::string variable_name = str_to_upper(hardcode_variable_name(node_name));
            std::stringstream value{};
            for (auto step = member_trace.begin(); step != member_trace.end() - 1; step++) {
                value << str_to_upper(renamed_name(*step)) << '.';
            }
            value << str_to_upper(node_name);
            emit_value(variable_name, value.str());
        }
        member_trace.pop_back();
        break;
    default:
        break;
    }
}

inline void cpl_property_value_constants_def(const ptree* obj) {
    // catch up tbd_file's namespace with hd_file's
    std::string tbd_namespace = idl_scoped_name(obj, nullptr);
    tbd_namespace = tbd_namespace.substr(0, tbd_namespace.rfind("::"));

    const ptree* ann = get_annotation(obj, annotation_type_ext_string_constants);
    // extract namespace(s)
    numeric ann_module = get_annotation_value(ann, "namespace");
    std::vector<std::string> prop_namespaces{};
    if (ann_module.kind() == STRING_KIND && !ann_module.val.str().empty()) {
        string_utils::split_string(prop_namespaces, ann_module.val.str(), "::", true);
    }

    for (const auto& prop_namespace : prop_namespaces) {
        mprintf(&g_hd_file, "namespace {} {{\n", prop_namespace);
        tbd_namespace += std::string("::") + prop_namespace;
    }
    std::deque<const ptree*> member_trace;
    // actual string constants
    cpl_property_value_constants_rec(obj, member_trace, tbd_namespace);
    for (const auto& prop_namespace : prop_namespaces) {
        mprintf(&g_hd_file, "}} // namespace {}\n", prop_namespace);
    }
    mprintfl(g_hd_tbd_files, "\n");
}

static void cpl_struct_c_def(const ptree* obj) {
    int number_of_elems = original_member_count(obj);
    auto body_name = scoped_name(obj, namespace_of(obj));
    auto param = cplpl_param_name(obj, "other");
    auto p_first = cplpl_param_name(obj, "first");
    auto p_second = cplpl_param_name(obj, "second");

    if (get_annotation(obj, annotation_type_ext_string_constants)) {
        cpl_property_value_constants_def(obj);
    }

    mprintf(&g_hd_file, "struct {}", name(obj));
    if (!obj->parents.empty()) {
        for (auto parent = obj->parents.begin(); parent != obj->parents.end(); ++parent) {
            const char* sep = parent == obj->parents.begin() ? " : " : ", ";
            mprintf(&g_hd_file, "{}public {}", sep, scoped_name(*parent, obj->super));
        }
    } else if (obj->kind == N_EXCEPTION) {
        mprintf(&g_hd_file, " : std::runtime_error\n");
    }

    mprintf(&g_hd_file, " {{");
    emit_post_docs(&g_hd_file, obj);

    cgcpl_recurs(obj->members);

    bool generate_constructors = false;
    for (const ptree* elem : obj->members) {
        if (is_shared(elem)) {
            generate_constructors = true;
        }
    }

    if (obj->kind == N_EXCEPTION) {
        mprintf(&g_hd_file, "{}();\n", name(obj));
        mprintf(&g_hd_impl_file, "inline {}::{} () ", body_name, name(obj));
        mprintf(&g_hd_impl_file, " :\nruntime_error(\"{}\")", name(obj));
        mprintf(&g_hd_impl_file, " {{}}\n\n");
        mprintf(&g_hd_file, "{}(const {}&) = default;\n", name(obj), name(obj));
        mprintf(&g_hd_file, "{}& operator=(const {}&) = default;\n", name(obj), name(obj));
    } else if (!generate_constructors) {
        mprintf(&g_hd_file, "{}() = default;\n", name(obj));
        mprintf(&g_hd_file, "{}(const {}&) = default;\n", name(obj), name(obj));
        mprintf(&g_hd_file, "{}& operator=(const {}&) = default;\n", name(obj), name(obj));
    } else {
        // Generate default constructor
        mprintf(&g_hd_file, "{}();\n", name(obj));
        mprintf(&g_hd_impl_file, "inline {}::{}() ", body_name, name(obj));
        char cchar = ':';
        if (obj->kind == N_EXCEPTION) {
            mprintf(&g_hd_impl_file, " :\nruntime_error(\"{}\")", name(obj));
            cchar = ',';
        }
        for (const ptree* elem : obj->members) {
            if (elem->kind == N_MEMBER) {
                if (is_shared(elem)) {
                    mprintf(
                        &g_hd_impl_file,
                        "{}\n{}{{ new {} {{}}",
                        cchar,
                        private_member_name(elem),
                        scoped_name(elem->type, obj)
                    );
                    if (has_default_value(elem)) {
                        emit_initializer_list(
                            &g_hd_impl_file, get_default_value(elem), namespace_of(obj), elem
                        );
                    }
                    mprintf(&g_hd_impl_file, " }}");
                    cchar = ',';
                } else if (is_pass_by_value(elem) || has_default_value(elem)) {
                    mprintf(&g_hd_impl_file, "{}\n{}", cchar, private_member_name(elem));
                    emit_initializer_list(
                        &g_hd_impl_file, get_default_value(elem), namespace_of(obj), elem
                    );
                    cchar = ',';
                } else if (base_type_of(elem)->kind == N_ARRAY &&
                           is_pass_by_value(base_type_of(elem)->element_type) &&
                           !is_optional(elem)) {
                    mprintf(&g_hd_impl_file, "{}\n{}", cchar, private_member_name(elem));
                    emit_default_array_initializer_list(
                        &g_hd_impl_file, base_type_of(elem), 0, true
                    );
                    cchar = ',';
                }
            }
        }
        mprintf(&g_hd_impl_file, "{{}}\n\n");

        // Generate copy constructor
        mprintf(&g_hd_file, "{}(const {} & {});\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file, "inline {}::{}(const {} & {})", body_name, name(obj), body_name, param
        );
        if (!obj->parents.empty()) {
            mprintf(&g_hd_impl_file, ":\n{}({})", scoped_name(obj->parents[0], obj->super), param);
            cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, param, ",\n", false);
        } else if (obj->kind == N_EXCEPTION) {
            mprintf(&g_hd_impl_file, " :\nruntime_error(\"{}\")", name(obj));
            cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, param, ",\n", false);
        } else {
            cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, param, " :\n", false);
        }
        mprintf(&g_hd_impl_file, "{{}}\n\n");

        // Generate assignment operator
        mprintf(&g_hd_file, "{}& operator=(const {}& {});\n\n", name(obj), name(obj), param);
        mprintf(
            &g_hd_impl_file,
            "inline {}& {}::operator=(const {}& {}) {{\n",
            body_name,
            body_name,
            name(obj),
            param
        );
        if (member_count(obj) > 0) {
            auto param_name = cplpl_param_name(obj, "copy");
            mprintf(&g_hd_impl_file, "{} {}({});\n", body_name, param_name, param);
            mprintf(&g_hd_impl_file, "swap(*this, {});\n", param_name);
        }
        mprintf(&g_hd_impl_file, "return *this;\n}}\n\n");
    }

    mprintf(&g_hd_file, "{}({} &&) = default;\n", name(obj), name(obj));
    mprintf(&g_hd_file, "{}& operator=({} &&) = default;\n", name(obj), name(obj));
    if (obj->parents.empty() && (obj->flags & OPT_HAS_CHILDREN) != 0) {
        mprintf(&g_hd_file, "virtual ~{}() noexcept {{}}\n", name(obj));
    }

    if (number_of_elems > 0 && !get_annotation(obj, annotation_type_ext_no_constructor)) {
        // Generate argument constructors (one with \@shared parameters and one without)
        const auto is_shared_rec = [](const ptree* node) {
            return is_shared(node);
        };  // fn ptr lacks default params
        const int has_shared_member =
            std::any_of(begin(obj->members), end(obj->members), is_shared_rec);
        for (int i = 0; i <= has_shared_member; i++) {
            const bool suppress_shared = i != 0;
            if (number_of_elems == 1) {
                mprintf(&g_hd_file, "explicit ");
            }
            mprintf(&g_hd_file, "{}(", name(obj));
            cpl_gen_member_arguments(&g_hd_file, obj, obj, true, suppress_shared);
            mprintf(&g_hd_file, ");\n");

            mprintf(&g_hd_impl_file, "inline {}::{} (", body_name, name(obj));
            cpl_gen_member_arguments(&g_hd_impl_file, obj, obj, true, suppress_shared);
            mprintf(&g_hd_impl_file, ")");

            if (!obj->parents.empty()) {
                mprintf(&g_hd_impl_file, " :\n{}( ", scoped_name(obj->parents[0], obj->super));
                cpl_gen_member_arguments(
                    &g_hd_impl_file,
                    obj->parents[0],
                    namespace_of(obj->parents[0]),
                    false,
                    suppress_shared
                );
                mprintf(&g_hd_impl_file, " )");
                cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, "", ",\n", suppress_shared);
            } else if (obj->kind == N_EXCEPTION) {
                mprintf(&g_hd_impl_file, " :\nruntime_error(\"{}\")", name(obj));
                cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, "", ",\n", suppress_shared);
            } else {
                cpl_gen_member_copy_ctor(&g_hd_impl_file, obj, "", " :\n", suppress_shared);
            }
            mprintf(&g_hd_impl_file, " {{}}\n\n");
        }
    }
    // Generate setters and getters
    cpl_gen_access_functions(obj);

    // Comparison operators
    cpl_comparator_def(obj);

    if (generate_constructors) {
        // Swap function
        // Only generate if we have created our own constructors (which use swap)
        // otherwise, std::swap (which uses generated constructors) is fine
        mprintf(
            &g_hd_file,
            "friend void swap({}& {}, {}& {}) noexcept;\n\n",
            name(obj),
            p_first,
            name(obj),
            p_second
        );
        mprintf(
            &g_hd_impl_file,
            "inline void swap({}& {}, {}& {}) noexcept {{\n",
            body_name,
            p_first,
            body_name,
            p_second
        );
        mprintf(&g_hd_impl_file, "using std::swap;\n");
        cpl_gen_member_swap(&g_hd_impl_file, obj);
        mprintf(&g_hd_impl_file, "}}\n\n");
    }

    for (const ptree* elem : obj->members) {
        if (elem->kind == N_PROTOTYPE) {
            cpl_prototype_c_def(elem);
        }
    }

    if (CommandLineOption::cpp_access_functions()) {
        mprintf(&g_hd_file, "protected:\n");
    }
    // Generate public member variables
    for (const ptree* elem : obj->members) {
        if (elem->kind == N_MEMBER) {
            if (CommandLineOption::cpp_access_functions()) {
                mprintf(&g_hd_file, "{} m_{}_", cplpl_member_type(elem, elem->super), name(elem));
            } else {
                emit_docs(&g_hd_file, elem);
                mprintf(&g_hd_file, "{} {}", cplpl_member_type(elem, elem->super), name(elem));
            }
            if (!generate_constructors) {
                if (is_pointer_type(elem)) {
                    mprintf(&g_hd_file, " {{ nullptr }}");
                }
                if (is_pass_by_value(elem) || has_default_value(elem)) {
                    emit_initializer_list(
                        &g_hd_file, get_default_value(elem), namespace_of(obj), elem
                    );
                } else if (base_type_of(elem)->kind == N_ARRAY &&
                           is_pass_by_value(base_type_of(elem)->element_type) &&
                           !is_optional(elem)) {
                    emit_default_array_initializer_list(&g_hd_file, base_type_of(elem), 0, true);
                }
            }
            mprintf(&g_hd_file, ";");
            emit_post_docs(&g_hd_file, elem);
        }
    }
    // Generate constants for default values
    if (CommandLineOption::generate_default_literals()) {
        bool added = false;
        for (const ptree* elem : obj->members) {
            if (elem->kind == N_MEMBER) {
                if (has_default_value(elem)) {
                    added = true;
                    mprintf(
                        &g_hd_file,
                        "static const {} {};\n",
                        cplpl_member_type(elem, elem->super),
                        const_name(elem)
                    );
                    mprintf(
                        &g_tbd_file,
                        "const {} {}::{}",
                        cplpl_member_type(elem, nullptr),
                        scoped_name(obj, nullptr),
                        const_name(elem)
                    );
                    emit_initializer_list(
                        &g_tbd_file, get_default_value(elem), namespace_of(obj), elem
                    );
                    mprintf(&g_tbd_file, ";\n");
                }
            }
        }
        if (added) {
            mprintf(&g_tbd_file, "\n");
        }
    }
    mprintf(&g_hd_file, "}};\n\n");
}

static void cpl_struct_enum_def(const ptree* obj) {
    const ptree* elem;
    if (is_bitmask(obj)) {
        mprintf(
            &g_hd_file,
            "enum {}Bits : {} {{",
            name(obj),
            scoped_name(obj->element_type, namespace_of(obj))
        );
        emit_post_docs(&g_hd_file, obj);

        elem = obj->members;
        while (elem) {
            emit_docs(&g_hd_file, elem);
            mprintf(&g_hd_file, "{} ={}", name(elem), get_const_value(elem->value, obj));
            if (elem->next) {
                mprintf(&g_hd_file, ",");
            }
            emit_post_docs(&g_hd_file, elem);
            elem = elem->next;
        }
        mprintf(&g_hd_file, "}};\n");

        mprintf(
            &g_hd_file,
            "\nusing {} = {};\n\n",
            name(obj),
            scoped_name(obj->element_type, namespace_of(obj))
        );
    } else {
        if (CommandLineOption::cpp_gen_cpp11()) {
            mprintf(
                &g_hd_file,
                "enum class {} : {} {{",
                name(obj),
                scoped_name(obj->element_type, namespace_of(obj))
            );
        } else {
            mprintf(
                &g_hd_file,
                "enum {} : {} {{",
                name(obj),
                scoped_name(obj->element_type, namespace_of(obj))
            );
        }
        emit_post_docs(&g_hd_file, obj);

        elem = obj->members;
        while (elem) {
            emit_docs(&g_hd_file, elem);

            if (elem->flags & OPT_ENUMERATED || obj->flags & OPT_ENUMERATED) {
                mprintf(&g_hd_file, "{} = {}", name(elem), integer_value(elem->value));
            } else {
                mprintf(&g_hd_file, "{}", name(elem));
            }
            if (elem->next) {
                mprintf(&g_hd_file, ",");
            }
            emit_post_docs(&g_hd_file, elem);
            elem = elem->next;
        }
        mprintf(&g_hd_file, "}};\n\n");
    }
}

static void cpl_prototype_c_def(const ptree* obj) {
    const ptree* par;
    int is_virtual = 0;

    if (get_annotation(obj, annotation_type_static)) {
        mprintf(&g_hd_file, "static ");
    } else if (obj->super->kind == N_INTERFACE) {
        mprintf(&g_hd_file, "virtual ");
        is_virtual = 1;
    }
    if (obj->type) {
        if (!is_pass_by_value(obj->type) && !is_pointer_type(obj->type) &&
            get_annotation(obj, annotation_type_const)) {
            mprintf(&g_hd_file, "const {}&", scoped_name(obj->type, obj));
        } else {
            mprintf(&g_hd_file, "{}", scoped_name(obj->type, obj));
            if (is_shared(obj->type)) {
                mprintf(&g_hd_file, "Ptr");
            } else if (is_pointer_type(obj)) {
                mprintf(&g_hd_file, "*");
            }
        }
    } else {
        mprintf(&g_hd_file, "void");
    }
    mprintf(&g_hd_file, " {}(\n", name(obj));
    for (par = obj->members; par; par = par->next) {
        const ptree* part = par->type;

        bool part_is_string = base_type_of(part)->kind == N_STRING;
        bool part_is_input_only = !(par->flags & OPT_OUT);
        bool part_is_span = part_is_input_only && (base_type_of(part)->kind == N_SEQUENCE ||
                                                   base_type_of(part)->kind == N_ARRAY);

        if (!part_is_string && part_is_input_only && !part_is_span) {
            if ((!is_pass_by_value(par) && !is_pointer_type(par)) || is_shared(par->type) ||
                (get_annotation(obj, annotation_type_const) && is_pointer_type(par))) {
                mprintf(&g_hd_file, "const ");
            }
        }
        if (part_is_string && part_is_input_only) {
            mprintf(&g_hd_file, cpp_string_view_type_name(par));
        } else {
            if (is_pointer_type(par) && is_shared(par->type)) {
                mprintf(&g_hd_file, "{}Ptr&", scoped_name(part, obj));
            } else if (part_is_span) {
                mprintf(
                    &g_hd_file,
                    "::intercom::span<const {}>",
                    scoped_name(base_type_of(part)->element_type, obj)
                );
            } else {
                mprintf(&g_hd_file, "{}", scoped_name(part, obj));
                if (is_pointer_type(par)) {
                    mprintf(&g_hd_file, "*");
                    if (par->flags & OPT_OUT) {
                        mprintf(&g_hd_file, "&");
                    }
                } else if (!is_pass_by_value(par) || (par->flags & OPT_OUT)) {
                    mprintf(&g_hd_file, "&");
                }
            }
        }
        mprintf(&g_hd_file, " a_{}", scoped_name(par, par));
        if (par->next) {
            mprintf(&g_hd_file, ",\n");
        }
    }
    mprintf(
        &g_hd_file,
        "\n){}{}\n",
        get_annotation(obj, annotation_type_const) ? " const" : "",
        is_virtual ? " = 0;\n" : ";\n"
    );
}

static void cpl_interface_c_def(const ptree* obj) {
    const ptree* elem;

    /* if (!(obj->options&OPT_NO_INTERFACE) ) */
    {
        mprintf(&g_hd_file, "class ");
        dll_export(&g_hd_file, obj);
        mprintf(&g_hd_file, "{}", name(obj));
        for (auto parent = obj->parents.begin(); parent != obj->parents.end(); ++parent) {
            mprintf(&g_hd_file, "{}", parent == obj->parents.begin() ? " : " : ", ");
            mprintf(&g_hd_file, "public {}", scoped_name(*parent, obj->super));
        }
        mprintf(&g_hd_file, " {{\npublic:\n");

        mprintf(&g_hd_file, "virtual ~{}();\n", name(obj));
        mprintf(&g_hd_impl_file, "inline {}::~{}() {{\n}}\n", name(obj), name(obj));

        cgcpl_recurs(obj->members);

        for (elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_PROTOTYPE) {
                cpl_prototype_c_def(elem);
            }
        }

        for (elem = obj->members; elem; elem = elem->next) {
            if (elem->kind != N_MEMBER) {
                continue;
            }
            auto elem_type = scoped_name(elem->type, obj);
            auto elem_name = name(elem);
            if (is_pointer_type(elem)) {
                if (!(elem->flags & OPT_READONLY)) {
                    mprintf(
                        &g_hd_file,
                        "virtual void {}({}*) = 0;\nvirtual {}* {}() = 0;\n",
                        elem_name,
                        elem_type,
                        elem_type,
                        elem_name
                    );
                }
                mprintf(&g_hd_file, "virtual const {}* {}() const = 0;\n", elem_type, elem_name);
            } else if (base_type_of(elem)->kind == N_STRUCT ||
                       base_type_of(elem)->kind == N_UNION ||
                       base_type_of(elem)->kind == N_SEQUENCE ||
                       base_type_of(elem)->kind == N_STRING ||
                       base_type_of(elem)->kind == N_ARRAY) {
                if (!(elem->flags & OPT_READONLY)) {
                    mprintf(&g_hd_file, "virtual void {}(const {}&) = 0;\n", elem_name, elem_type);
                }
                mprintf(&g_hd_file, "virtual const {}& {}() const = 0;\n", elem_type, elem_name);
            } else {
                if (!(elem->flags & OPT_READONLY)) {
                    mprintf(&g_hd_file, "virtual void {}({}) = 0;\n", elem_name, elem_type);
                }
                mprintf(&g_hd_file, "virtual {} {}() const = 0;\n", elem_type, elem_name);
            }
        }
        mprintf(&g_hd_file, "}};\n\n");
        if (is_shared(obj)) {
            mprintf(&g_hd_file, "using {}Ptr = std::shared_ptr<{}>;\n\n", name(obj), name(obj));
        }
    }
}

static void cpl_const_c_def(const ptree* obj) {
    int is_global =
        (obj->super == nullptr || (obj->super->kind != N_INTERFACE &&
                                   obj->super->kind != N_STRUCT && obj->super->kind != N_UNION));

    auto qualified_name = scoped_name(obj, nullptr);
    auto qualified_type_name = scoped_name(obj->type, nullptr);

    if (!is_global) {
        mprintf(&g_hd_file, "static const {} {};\n", type_name(obj), name(obj));

        if (obj->value.kind() != UNDEF_KIND) {
            mprintf(
                &g_tbd_file,
                "const {} {}",
                scoped_name(obj->type, nullptr),
                scoped_name(obj, nullptr)
            );
            emit_initializer_list(&g_tbd_file, obj->value, nullptr, obj);
            mprintf(&g_tbd_file, ";");
            emit_post_docs(&g_tbd_file, obj);
        }
    } else if (base_type_of(obj)->kind == N_STRING) {
        dll_export(&g_hd_file, base_type_of(obj));
        mprintf(&g_hd_file, "{} const char* {};\n", (is_global ? "extern" : "static"), name(obj));
        mprintf(&g_tbd_file, "const char* {} = ", qualified_name);
        emit_const_value(&g_tbd_file, obj->value, nullptr, obj);
        mprintf(&g_tbd_file, ";");
        emit_post_docs(&g_tbd_file, obj);
    } else if (!is_pass_by_value(base_type_of(obj))) {
        dll_export(&g_hd_file, base_type_of(obj));
        mprintf(&g_hd_file, "extern const {} {};\n", qualified_type_name, name(obj));

        if (obj->value.kind() != UNDEF_KIND) {
            mprintf(&g_tbd_file, "const {} {}", qualified_type_name, qualified_name);
            emit_initializer_list(&g_tbd_file, obj->value, nullptr, obj);
            mprintf(&g_tbd_file, ";");
            emit_post_docs(&g_tbd_file, obj);
        }
    } else {
        mprintf(&g_hd_file, "const {} {}", type_name(obj), name(obj));
        emit_initializer_list(&g_hd_file, obj->value, namespace_of(obj), obj);
        mprintf(&g_hd_file, ";");
        emit_post_docs(&g_hd_file, obj);
    }
}

static const ptree* g_current_include;

static void include_type(struct memf* memf, const ptree* obj, const ptree* curr_include) {
    if (obj) {
        if (obj->included_from && obj->included_from != curr_include) {
            includeit(memf, obj->included_from);
        }
        for (auto parent : obj->parents) {
            include_type(memf, parent, curr_include);
        }
        include_type(memf, obj->type, curr_include);
        include_type(memf, obj->element_type, curr_include);
        include_type(memf, obj->key_type, curr_include);
    }
}

static void include_dependencies(struct memf* memf, const ptree* obj, const ptree* curr_include) {
    for (; obj; obj = obj->next) {
        if (!is_emit(obj, LANG_CPP)) {
            continue;
        }
        if (obj->included_from == curr_include) {
            include_type(memf, obj, curr_include);
            if (obj->value.kind() == PTREE_KIND) {
                include_type(memf, obj->value.val.node(), curr_include);
            }
        }
        include_dependencies(memf, obj->members, curr_include);
    }
}

static int has_rpc_service(const ptree* obj) {
    for (; obj; obj = obj->next) {
        if (is_rpc_service(obj) || has_rpc_service(obj->members)) {
            return 1;
        }
    }
    return 0;
}

static int has_exceptions(const ptree* obj) {
    for (; obj; obj = obj->next) {
        if (obj->kind == N_EXCEPTION || has_exceptions(obj->members)) {
            return 1;
        }
    }
    return 0;
}

static void cgcpl_recurs(const ptree* obj) {
    for (; obj; obj = obj->next) {
        if (obj->included_from != g_current_include) {
            cgcpl_recurs(obj->members);
            continue;
        }
        if (!is_emit(obj, LANG_CPP)) {
            continue;
        }
        cgcpl_recurs(obj->generated);

        switch (obj->kind) {
        case N_EXCEPTION:
        case N_VALUETYPE:
        case N_STRUCT:
            emit_docs(&g_hd_file, obj);
            if (obj->flags & OPT_DECLARATION) {
                mprintf(&g_hd_file, "struct {};\n", name(obj));
            } else {
                cpl_struct_c_def(obj);
                cpl_conv_gen(obj);
                cpl_iostream_def(obj);
                cpl_gen_hash(obj);
            }
            break;
        case N_UNION:
            emit_docs(&g_hd_file, obj);
            if (obj->flags & OPT_DECLARATION) {
                mprintf(&g_hd_file, "struct {};\n", name(obj));
            } else {
                cpl_union_c_def(obj);
                cpl_conv_gen(obj);
                cpl_gen_hash(obj);
                cpl_iostream_def(obj);
            }
            break;
        case N_ENUM:
            emit_docs(&g_hd_file, obj);
            cpl_struct_enum_def(obj);
            cpl_conv_gen(obj);
            cpl_iostream_def(obj);
            break;
        case N_BITMASK:
            emit_docs(&g_hd_file, obj);
            cpl_struct_enum_def(obj);
            cpl_conv_gen(obj);
            break;
        case N_INTERFACE:
            emit_docs(&g_hd_file, obj);
            if (obj->flags & OPT_DECLARATION) {
                mprintf(&g_hd_file, "class ");
                dll_export(&g_hd_file, obj);
                mprintf(&g_hd_file, "{};\n", name(obj));
            } else if (!is_rpc_service(obj)) {
                cpl_interface_c_def(obj);
            }
            break;
        case N_ALIAS:
            emit_docs(&g_hd_file, obj);
            mprintf(
                &g_hd_file, "using {} = {};\n", name(obj), cplpl_member_type(obj, namespace_of(obj))
            );
            break;
        case N_MODULE:
            emit_docs(&g_hd_file, obj);
            if (idl_scoped_name(obj, nullptr) == "DDS") {
                mprintfl(g_all_headers, "namespace intercom {{\n");
                mprintfl(g_all_headers, "namespace dcps {{\n");
                cgcpl_recurs(obj->members);
                mprintfl(g_all_headers, "}} // namespace dcps\n");
                mprintfl(g_all_headers, "}} // namespace intercom\n");
            } else if (idl_scoped_name(obj, nullptr) == "DDS::Security") {
                mprintfl(g_all_headers, "namespace security {{\n");
                cgcpl_recurs(obj->members);
                mprintfl(g_all_headers, "}} // namespace security\n");
            } else if (idl_scoped_name(obj, nullptr) == "DDS::RPC") {
                mprintfl(g_all_headers, "namespace rpc {{\n");
                cgcpl_recurs(obj->members);
                mprintfl(g_all_headers, "}} // namespace rpc\n");
            } else if (idl_scoped_name(obj, nullptr) == "DDS::XTypes") {
                mprintfl(g_all_headers, "namespace xtypes {{\n");
                cgcpl_recurs(obj->members);
                mprintfl(g_all_headers, "}} // namespace xtypes\n");
            } else {
                mprintfl(g_all_headers, "namespace {} {{", name(obj));
                for (auto h : g_all_headers) {
                    emit_post_docs(h, obj);
                }
                cgcpl_recurs(obj->members);
                mprintfl(g_all_headers, "}} // namespace {}\n\n", name(obj));
            }
            break;
        case N_CONST:
            emit_docs(&g_hd_file, obj);
            cpl_const_c_def(obj);
            break;
        default:
            break;
        }
    }
}

static void
cpl_saveit(const ptree* tree, const std::string& module, const std::string& source_name) {
    auto include_prefix = CommandLineOption::header_subfolder();
    if (!module.empty()) {
        static struct memf pk_file;
        mreset(&pk_file);

        if (CommandLineOption::copyright_notice()) {
            mprintf(&pk_file, "{}", CommandLineOption::copyright_notice());
        }

        mprintf(&pk_file, "#pragma once\n");

        mprintf(
            &pk_file,
            "#include <InterCOM/version.h>\n"
            "#ifndef INTERCOM_VERSION_" INTERCOM_VERSION_S
            "\n"
            "#error \"CIDL-generated code does not match InterCOM product version: " INTERCOM_VERSION_S
            "\"\n"
            "#endif // INTERCOM_VERSION_" INTERCOM_VERSION_S "\n\n"
        );
        mprintf(
            &pk_file,
            "#ifdef _WIN32\n"
            "#pragma warning(push)\n"
            "#pragma warning(disable:4065)\n"
            "#pragma warning(disable:4127)\n"
            "#endif\n\n"
        );

        // This must come before the other includes
        if (CommandLineOption::corba_types()) {
            mprintf(&pk_file, "#ifndef INTERCOM_CORBA_TYPES\n");
            mprintf(&pk_file, "#define INTERCOM_CORBA_TYPES\n");
            mprintf(&pk_file, "#endif\n\n");
        }

        if (CommandLineOption::dll_exp_sym() &&
            strcmp(CommandLineOption::dll_exp_sym(), INTERCOM_PUBLIC_MACRO_NAME) != 0) {
            mprintf(
                &pk_file,
                "#ifndef {}\n#ifdef _WIN32\n#define {} __declspec(dllimport)\n#else\n#define "
                "{}\n#endif\n#endif\n\n",
                CommandLineOption::dll_exp_sym(),
                CommandLineOption::dll_exp_sym(),
                CommandLineOption::dll_exp_sym()
            );

            mprintf(
                &g_prebd_file,
                "#ifndef {}\n"
                "#ifdef _WIN32\n"
                "#define {} __declspec(dllexport)\n"
                "#else\n"
                "#define {} __attribute((visibility(\"default\")))\n"
                "#endif\n"
                "#endif\n\n",
                CommandLineOption::dll_exp_sym(),
                CommandLineOption::dll_exp_sym(),
                CommandLineOption::dll_exp_sym()
            );
        }
        if (include_prefix) {
            mprintf(
                &g_prebd_file,
                "#include \"{}/{}.{}\"\n",
                include_prefix,
                module,
                CommandLineOption::cpp_header_postfix()
            );
        } else {
            mprintf(
                &g_prebd_file,
                "#include \"{}.{}\"\n",
                module,
                CommandLineOption::cpp_header_postfix()
            );
        }

        if (!mempty(&g_hd_rpc_file) && !CommandLineOption::cpp_gen_cpp11()) {
            mprintf(&pk_file, "#include <InterCOM/rpc.h>\n");
        }

        if (CommandLineOption::cpp_gen_cpp11()) {
            if (!mempty(&g_hd_hash_file)) {
                if (!mempty(&g_hd_rpc_file)) {
                    mprintf(&pk_file, "#include <dds/dds.hpp>\n");
                    mprintf(&pk_file, "#include <dds/rpc/rpc.hpp>\n");
                    mprintf(&pk_file, "#include <dds/rpc/rpc_types.hpp>\n");
                }
            }
        }
        mprintf(&pk_file, "#include <InterCOM/optional.h>\n");
        mprintf(&pk_file, "#include <InterCOM/span.h>\n");

        if (CommandLineOption::intercom_build()) {
#ifndef CIDL_BOOTSTRAP
            mprintf(&pk_file, "#define INTERCOM_TYPESUPPORT_INTERFACE_ONLY\n");
            mprintf(&pk_file, "#include <InterCOM/TypeSupport.h>\n");
            mprintf(&pk_file, "#undef INTERCOM_TYPESUPPORT_INTERFACE_ONLY\n");
#else
            mprintf(&pk_file, "#include <InterCOM/intercom_dcps.h>\n");
#endif
            mprintf(&pk_file, "#include <InterCOM/MemberInfo.h>\n");
        } else {
            mprintf(&pk_file, "#include <InterCOM/CdrSerializer.h>\n");
            mprintf(&pk_file, "#include <InterCOM/TypeSupport.h>\n");
        }

        if (has_rpc_service(tree)) {
            mprintf(&pk_file, "#include <InterCOM/dds_curr_rpc.h>\n");
        }

        if (module != "dds_xtypes_constants") {
            mprintf(&pk_file, "#include <InterCOM/JsonSerializer.h>\n");
        }

        if (has_exceptions(tree)) {
            mprintf(&pk_file, "#include <stdexcept>\n");
        }
        mprintf(&pk_file, "#include <functional>\n");

        include_dependencies(&pk_file, tree, g_current_include);

        if (CommandLineOption::use_fmtlib()) {
            if (CommandLineOption::intercom_build()) {
                mprintf(&pk_file, "#ifdef INTERCOM_FMTLIB\n");
            }
            mprintf(&pk_file, "#include <fmt/ostream.h>\n");
            if (CommandLineOption::intercom_build()) {
                mprintf(&pk_file, "#endif\n");
            }
        }
        mprintf(&pk_file, "\n");

        memfcat(&pk_file, &g_hd_file);
        if (!mempty(&g_hd_hash_file)) {
            mprintf(&g_hd_hash_file, "}}\n");
            memfcat(&pk_file, &g_hd_hash_file);
        }
        if (!mempty(&g_hd_ts_file)) {
            mprintf(&g_hd_ts_file, "}}\n");
            memfcat(&pk_file, &g_hd_ts_file);
        }
        memfcat(&pk_file, &g_hd_impl_file);
        memfcat(&pk_file, &g_hd_rpc_file);
        memfcat(&pk_file, &g_hd_json_file);
        if (!mempty(&g_hd_fmt_file)) {
            if (CommandLineOption::intercom_build()) {
                mprintf(&pk_file, "#ifdef INTERCOM_FMTLIB\n");
            }
            mprintf(&pk_file, "namespace fmt {{\n");
            memfcat(&pk_file, &g_hd_fmt_file);
            mprintf(&pk_file, "}} // namespace fmt\n");
            if (CommandLineOption::intercom_build()) {
                mprintf(&pk_file, "#endif // INTERCOM_FMTLIB\n");
            }
        }
        mprintf(
            &pk_file,
            "#ifdef _WIN32\n"
            "#pragma warning(pop)\n"
            "#endif\n\n"
        );

        std::string cname(module);
        if (CommandLineOption::c_file_prefix()) {
            cname = fmt::format("{}{}", CommandLineOption::c_file_prefix(), module);
        }

        mprintf(&g_prebd_file, "\n");
#ifdef CIDL_BOOTSTRAP
        mprintf(&g_prebd_file, "#include <InterCOM/dds_xtypes_constants.h>\n");
#else
        mprintf(&g_prebd_file, "#include <InterCOM/dds_curr_xtypes.h>\n");
        mprintf(&g_prebd_file, "#include <InterCOM/TypeSupport.h>\n\n");
#endif

        mprintf(
            &g_prebd_file,
            "#ifdef _WIN32\n"
            "#pragma warning(push)\n"
            "#pragma warning(disable:4065)\n"
            "#endif\n\n"
        );

        memfcat(&g_prebd_file, &g_tbd_file);
        if (!mempty(&g_tbd_hash_file)) {
            memfcat(&g_prebd_file, &g_tbd_hash_file);
        }
        mprintf(
            &g_prebd_file,
            "#ifdef _WIN32\n"
            "#pragma warning(pop)\n"
            "#endif\n\n"
        );

        savememf(
            &g_prebd_file,
            nullptr,
            CommandLineOption::c_target_directory(),
            "",
            "{}.cpp",
            cname.c_str()
        );
        savememf(
            &pk_file,
            nullptr,
            CommandLineOption::c_target_directory(),
            include_prefix,
            "{}.{}",
            module.c_str()
        );

        if (CommandLineOption::compatibility()) {
            mreset(&pk_file);
            mprintf(&pk_file, "#include \"{}.h\"\n", module);
            savememf(
                &pk_file,
                nullptr,
                CommandLineOption::c_target_directory(),
                "",
                "{}Support.h",
                module.c_str()
            );
            savememf(
                &pk_file,
                nullptr,
                CommandLineOption::c_target_directory(),
                "",
                "ccpp_{}.h",
                module.c_str()
            );
            savememf(
                &pk_file,
                nullptr,
                CommandLineOption::c_target_directory(),
                "",
                "{}Dcps_impl.h",
                module.c_str()
            );
        }
        mreset(&pk_file);
    }

    mreset(&g_hd_file);
    mreset(&g_hd_ts_file);
    mreset(&g_hd_impl_file);
    mreset(&g_hd_json_file);
    mreset(&g_hd_hash_file);
    mreset(&g_tbd_hash_file);
    mreset(&g_tbd_file);
    mreset(&g_prebd_file);
    mreset(&g_hd_rpc_file);
    mreset(&g_hd_fmt_file);
}

void intercom::cidl::code_gen_dds_cplpl(const parse_result* result) {
    for (auto include : result->includes) {
        g_current_include = include;
        cgcpl_recurs(result->tree);
        std::string file_name = trim_include_name(include->name, true);
        // TODO(idarcar);
        // cpl_rpc_service_gen(result->tree, &g_hd_rpc_file, &g_tbd_file, g_current_include);
        cpl_saveit(result->tree, file_name, include->name);
    }
}

void intercom::cidl::code_gen_dds_cplpl(const parse_result* result, const char* destination) {
    CommandLineOption::get_instance().c_target_directory = destination;
    code_gen_dds_cplpl(result);
}
