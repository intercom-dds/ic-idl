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

#include <fcntl.h>
#include <fmt/format.h>

#include <cstring>
#include <list>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/memf.h"
#include "cidl/ptree.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

#define JAVANAME_FLAGS_CLASS 1
#define JAVANAME_FLAGS_PROXY 2
#define JAVANAME_NO_ARRAY_SUFFIX 4

static intercom::cidl::memf g_hd_file;

using namespace intercom::cidl;

static void java_savememf(struct memf* memf, const std::string& filedir, const char* frmt) {
    size_t size;
    auto filename = fmt::format(fmt::runtime(frmt), filedir);
    auto fullfilename = fmt::format("{}/{}", CommandLineOption::java_target_directory(), filename);
    size = memf->memp - memf->memfile;

    if (CommandLineOption::list_only()) {
        fmt::print("{}\n", filename);
    } else {
        std::stringstream content;
        content.write(memf->memfile, static_cast<std::streamsize>(size));
        write_if_changed(fullfilename, content.str());
    }
}

static const char* stream_integer_type(const ptree* obj) {
    if (obj->kind == N_ENUM || obj->kind == N_BITMASK) {
        return stream_integer_type(obj->element_type);
    }
    if (obj == &boolean_type) {
        return "boolean";
    }
    if (obj == &int8_type) {
        return "octet";
    }
    if (obj == &octet_type) {
        return "octet";
    }
    if (obj == &char_type) {
        return "char";
    }
    if (obj == &wchar_type) {
        return "wchar";
    }
    if (obj == &short_type) {
        return "short";
    }
    if (obj == &ushort_type) {
        return "short";
    }
    if (obj == &long_type) {
        return "long";
    }
    if (obj == &ulong_type) {
        return "long";
    }
    if (obj == &longlong_type) {
        return "longlong";
    }
    if (obj == &ulonglong_type) {
        return "longlong";
    }
    if (obj == &float_type) {
        return "float";
    }
    if (obj == &double_type) {
        return "double";
    }
    if (obj == &ldouble_type) {
        return "longdouble";
    }
    return "";
}

static const char* java_integer_type(const ptree* obj) {
    if (obj->kind == N_ENUM || obj->kind == N_BITMASK) {
        return java_integer_type(obj->element_type);
    }
    if (obj == &boolean_type) {
        return "boolean";
    }
    if (obj == &int8_type) {
        return "byte";
    }
    if (obj == &octet_type) {
        return "byte";
    }
    if (obj == &char_type) {
        return "char";
    }
    if (obj == &wchar_type) {
        return "char";
    }
    if (obj == &short_type) {
        return "short";
    }
    if (obj == &ushort_type) {
        return "short";
    }
    if (obj == &long_type) {
        return "int";
    }
    if (obj == &ulong_type) {
        return "int";
    }
    if (obj == &longlong_type) {
        return "long";
    }
    if (obj == &ulonglong_type) {
        return "long";
    }
    if (obj == &float_type) {
        return "float";
    }
    if (obj == &double_type) {
        return "double";
    }
    if (obj == &ldouble_type) {
        return "double";
    }
    return "";
}

static std::string java_integer_cast(const ptree* obj) {
    if (obj->kind == N_ENUM || obj->kind == N_BITMASK) {
        return java_integer_cast(obj->element_type);
    }
    if (obj == &long_type || obj == &ulong_type) {
        return {};
    }
    return fmt::format("({})", java_integer_type(obj));
}

static const char* java_integer_zero(const ptree* obj) {
    if (obj->kind == N_ENUM || obj->kind == N_BITMASK) {
        return java_integer_zero(obj->element_type);
    }
    if (obj == &boolean_type) {
        return "false";
    }
    if (obj == &int8_type) {
        return "'\\u0000'";
    }
    if (obj == &octet_type) {
        return "'\\u0000'";
    }
    if (obj == &char_type) {
        return "'\\u0000'";
    }
    if (obj == &wchar_type) {
        return "'\\u0000'";
    }
    if (obj == &short_type) {
        return "(short)0";
    }
    if (obj == &ushort_type) {
        return "(short)0";
    }
    if (obj == &long_type) {
        return "0";
    }
    if (obj == &ulong_type) {
        return "0";
    }
    if (obj == &longlong_type) {
        return "(long)0";
    }
    if (obj == &ulonglong_type) {
        return "(long)0";
    }
    if (obj == &float_type) {
        return "0.0f";
    }
    if (obj == &double_type) {
        return "0.0d";
    }
    if (obj == &ldouble_type) {
        return "0.0d";
    }
    return "";
}

std::string javaname(const ptree* obj, const char* delim, int flags) {
    const ptree* bobj = base_type_of(obj);
    if (obj == nullptr) {
        return {};
    }
    if (obj->kind == N_CONST) {
        bobj = obj;
    }
    if (bobj->kind == N_ENUM || bobj->kind == N_INTERFACE) {
        obj = bobj;
    }
    if (bobj->kind == N_STRING) {
        return "java.lang.String";
    }
    if (obj->kind == N_INTERFACE) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Long" : "long";
    }
    if (bobj == &long_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Integer" : "int";
    }
    if (bobj == &char_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Character" : "char";
    }
    if (bobj == &wchar_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Character" : "char";
    }
    if (bobj == &double_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Double" : "double";
    }
    if (bobj == &ldouble_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.LongDouble" : "double";
    }
    if (bobj == &float_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Float" : "float";
    }
    if (bobj == &short_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Short" : "short";
    }
    if (bobj == &ushort_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Short" : "short";
    }
    if (bobj == &int8_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Byte" : "byte";
    }
    if (bobj == &octet_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Byte" : "byte";
    }
    if (bobj == &ulong_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Integer" : "int";
    }
    if (bobj == &ulonglong_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Long" : "long";
    }
    if (bobj == &longlong_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Long" : "long";
    }
    if (bobj == &boolean_type) {
        return (flags & JAVANAME_FLAGS_CLASS) ? "java.lang.Boolean" : "boolean";
    }
    if (bobj->kind == N_ARRAY || bobj->kind == N_SEQUENCE) {
        std::string ss = javaname(bobj->element_type, ".", flags);
        if (!(flags & JAVANAME_NO_ARRAY_SUFFIX)) {
            if (bobj->kind == N_SEQUENCE) {
                ss += "[]";
            } else {
                for (size_t i = 0; i < bobj->bounds.size(); ++i) {
                    ss += "[]";
                }
            }
        }
        return ss;
    }
    if (bobj->kind == N_MAP) {
        auto key = javaname(bobj->key_type, ".", flags | JAVANAME_FLAGS_CLASS);
        auto elem = javaname(bobj->element_type, ".", flags | JAVANAME_FLAGS_CLASS);
        return fmt::format("java.util.Map<{}, {}>", key, elem);
    }

    std::string ss;
    for (; bobj && !bobj->name.empty(); bobj = bobj->super) {
        if (idl_scoped_name(bobj, nullptr) == "DDS") {
            ss = fmt::format(
                "com.kongsberg.intercom{}{}{}{}",
                delim,
                (flags & JAVANAME_FLAGS_PROXY) ? "dcps" : "jni",
                delim,
                ss
            );
        } else if (bobj->kind != N_INCLUDE) {
            ss = fmt::format("{}{}{}", java_name(bobj), delim, ss);
            if (!delim[0]) {
                break;
            }
        }
    }
    if (delim[0] && !ss.empty() && CommandLineOption::java_package_prefix()) {
        ss = fmt::format("{}{}{}", CommandLineOption::java_package_prefix(), delim, ss);
    }
    if (delim[0] && !ss.empty()) {
        ss.pop_back();
    }
    return ss;
}

static std::string filter_package(const std::string& name, const ptree* obj) {
    auto obj_name = javaname(obj, ".", 0);
    auto dot1 = name.find_last_of('.');
    auto dot2 = obj_name.find_last_of('.');
    if (dot1 != 0 && dot1 == dot2 && name.compare(0, dot1, obj_name, 0, dot1) == 0) {
        return name.substr(dot1 + 1);
    }
    return name;
}

static std::string javavalue(const numeric& value, int qualified);

/// \param node should derive from \@default annotation
static std::string javavalue(const ptree* node) {
    node = base_value_of(node);
    const ptree* base_type = base_type_of(node);
    std::stringstream out;

    auto new_container = [&](const char bracket_open, const char bracket_close) {
        out << "new " << javaname(node->type, ".", 0) << bracket_open;
        const ptree* members =
            node->value.kind() == PTREE_KIND ? node->value.val.node()->members : node->members;
        for (const ptree* member : members) {
            out << javavalue(member);
            if (member->next) {
                out << ", ";
            }
        }
        out << bracket_close;
    };
    switch (base_type->kind) {
    case N_BITMASK:
    case N_BITSET:
    case N_ENUM:
        out << javaname(base_type, ".", 0) << "." << node->name;
        break;
    case N_PRIMITIVE:
    case N_STRING:
        out << javavalue(node->value, 0);
        break;
    case N_STRUCT:
        new_container('(', ')');
        break;
    case N_ARRAY:
    case N_SEQUENCE:
        new_container('{', '}');
        break;
    default:
        break;
    }
    return out.str();
}

static std::string javavalue(const numeric& value, int qualified) {
    std::stringstream out;

    switch (value.kind()) {
    case UNDEF_KIND:
        break;
    case BOOLEAN_KIND:
        return integer_value(value) ? "true" : "false";
    case INT8_KIND:
        return fmt::format("(byte){}", value.val.i8());
    case OCTET_KIND:
        return fmt::format("(byte){}", value.val.o());
    case SHORT_KIND:
        return fmt::format("(short){}", value.val.s());
    case USHORT_KIND:
        return fmt::format("(short){}", value.val.us());
    case LONG_KIND:
        return fmt::format("{}", value.val.l());
    case ULONG_KIND:
        return fmt::format("{}", static_cast<int>(value.val.ul()));
    case LONGLONG_KIND:
        return fmt::format("{}L", value.val.ll());
    case ULONGLONG_KIND:
        return fmt::format("{}L", static_cast<long long int>(value.val.ull()));
    case FLOAT_KIND:
        return fmt::format("(float){}", fmt::format("{:.7f}", value.val.f()));
    case DOUBLE_KIND:
        return fmt::format("{}", fmt::format("{:.16f}", value.val.d()));
    case CHAR_KIND:
        return fmt::format("'\\u{:04x}'", value.val.c());
    case STRING_KIND:
        return fmt::format("\"{}\"", value.val.str());
    case PTREE_KIND: {
        const ptree* obj = value.val.node();
        if (obj->name.empty()) {
            return javavalue(obj);
        }
        // reference
        if (qualified) {
            if (obj->kind == N_CONST &&
                (obj->type->kind == N_ENUM || obj->type->kind == N_BITMASK)) {
                out << javaname(obj->type, ".", 0) << "." << base_value_of(obj)->name;
            } else {
                out << javaname(obj, ".", 0);
            }
        } else {
            out << obj->name;
        }
        if (obj->kind == N_CONST && /*whitelist, since i.a. enums use the ref's value directly (it
                                       should not)*/
            !std::set<node_kind>({N_PRIMITIVE, N_STRING, N_STRUCT})
                 .insert(base_type_of(obj)->kind)
                 .second) {
            out << ".value";
        }
    } break;
    }
    return out.str();
}

static void
java_par_elem(const ptree* owner, const ptree* tobj, const std::string& name, int flags) {
    const ptree* btobj = base_type_of(tobj);
    mprintf(&g_hd_file, "{} {}", filter_package(javaname(btobj, ".", flags), owner), name);
}

#define DO_CONSTRUCTOR 0x0200
#define DO_CLONER 0x0400
#define DO_DISCRIMINATOR 0x0800
/// avoids use of "this."
/// \n must be paired with either DO_CONSTRUCTOR or DO_CLONER
#define DO_EXTERNAL 0x1000

#define FR_AIR_FLAG 0x1

static size_t sequence_index_count(const ptree* obj, size_t count) {
    if (obj) {
        switch (obj->kind) {
        case N_ALIAS:
            return sequence_index_count(obj->type, count);
        case N_ARRAY:
            count += obj->bounds.size();
            return sequence_index_count(obj->element_type, count);
        case N_SEQUENCE:
            return sequence_index_count(obj->element_type, count + 1);
        default:
            break;
        }
    }
    return count;
}

static void sequence_brackets(const ptree* obj, const char* expr, std::ostream& out) {
    if (obj->kind == N_ALIAS) {
        return sequence_brackets(obj->type, expr, out);
    }
    if (obj->kind == N_ARRAY) {
        for (const auto& bound : obj->bounds) {
            out << "[" << integer_value(bound) << "]";
        }
        return sequence_brackets(obj->element_type, expr, out);
    }
    if (obj->kind == N_SEQUENCE) {
        out << "[" << expr << "]";
        return sequence_brackets(obj->element_type, expr, out);
    }
}

static std::string sequence_brackets(const ptree* obj, const char* expr) {
    std::stringstream out;
    sequence_brackets(obj, expr, out);
    return out.str();
}

#define ITER_CHAR ('i' + rec_level)

/// \param alias_origin the parent alias of obj (to extract default values)
static void java_conv_gen_elem(
    const ptree* owner,
    const ptree* obj,
    const std::string& tag,
    size_t rec_level,
    unsigned in_flag,
    const ptree* alias_origin = nullptr
) {
    std::string cast_name = filter_package(javaname(obj, ".", 0), owner);
    std::string cast_name_no_suffix =
        filter_package(javaname(obj, ".", JAVANAME_NO_ARRAY_SUFFIX), owner);
    std::string holder;
    if (in_flag & DO_EXTERNAL) {
        holder = fmt::format("{} = ", tag);
    } else if (in_flag & (DO_CONSTRUCTOR | DO_CLONER)) {
        holder = fmt::format("this.{} = ", tag);
    }

    if (alias_origin == nullptr) {
        alias_origin = obj;
    }
    numeric numeric_default =
        get_annotation_value(get_annotation(alias_origin, annotation_type_default), "value");
    auto default_value = javavalue(numeric_default, 1);

    switch (obj->kind) {
    case N_STRUCT:
    case N_UNION:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (default_value.empty()) {
                mprintf(&g_hd_file, "new {}();\n", cast_name);
            } else {
                mprintf(&g_hd_file, "{};\n", default_value);
            }
        } else if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(&g_hd_file, "new {}(other.{});\n", cast_name, tag);
        } else if (in_flag & FR_AIR_FLAG) {
            mprintf(&g_hd_file, "{} = {}Helper.read(stream, {});\n", tag, cast_name, tag);
        } else {
            mprintf(&g_hd_file, "{}Helper.write(stream, {});\n", cast_name, tag);
        }
        break;

    case N_ENUM:
    case N_BITMASK:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (!default_value.empty() && numeric_default.kind() == PTREE_KIND) {
                mprintf(&g_hd_file, "{};\n", default_value);
            } else {
                mprintf(&g_hd_file, "{}.{};\n", cast_name, obj->members->name);
            }
        } else if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(&g_hd_file, "other.{};\n", tag);
        } else if (in_flag & FR_AIR_FLAG) {
            mprintf(
                &g_hd_file,
                "{} = {}.fr_ordinal(stream.read_{}());\n",
                tag,
                cast_name,
                stream_integer_type(obj)
            );
        } else {
            mprintf(
                &g_hd_file, "stream.write_{}({}.my_ordinal());\n", stream_integer_type(obj), tag
            );
        }
        break;

    case N_PRIMITIVE:
        if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(&g_hd_file, "other.{};\n", tag);
        } else if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (default_value.empty()) {
                mprintf(&g_hd_file, "{};\n", java_integer_zero(obj));
            } else {
                mprintf(&g_hd_file, "{};\n", default_value);
            }
        } else if (in_flag & FR_AIR_FLAG) {
            mprintf(&g_hd_file, "{} = stream.read_{}();\n", tag, stream_integer_type(obj));
        } else {
            mprintf(&g_hd_file, "stream.write_{}({});\n", stream_integer_type(obj), tag);
        }
        break;

    case N_SEQUENCE:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (default_value.empty()) {
                mprintf(&g_hd_file, "new {}", cast_name_no_suffix);
                mprintf(&g_hd_file, "{}", sequence_brackets(obj, "0"));
                mprintf(&g_hd_file, ";\n");
            } else {
                mprintf(&g_hd_file, "{};\n", default_value);
            }
        } else if (!(in_flag & DO_EXTERNAL)) {
            mprintf(&g_hd_file, "{{\nint _{:c}_len = ", ITER_CHAR);
            if (in_flag & DO_CLONER) {
                mprintf(&g_hd_file, "other.{}.length;\n", tag);
            } else if (in_flag & FR_AIR_FLAG) {
                mprintf(&g_hd_file, "stream.read_long();\n");
            } else {
                mprintf(&g_hd_file, "({} != null) ? {}.length : 0;\n", tag, tag);
                mprintf(&g_hd_file, "stream.write_long(_{:c}_len);\n", ITER_CHAR);
            }
            if (in_flag & DO_CLONER) {
                mprintf(
                    &g_hd_file,
                    "if (this.{} == null || this.{}.length != _{:c}_len) {{\n",
                    tag,
                    tag,
                    ITER_CHAR
                );
                mprintf(
                    &g_hd_file, "this.{} = new {}[_{:c}_len]", tag, cast_name_no_suffix, ITER_CHAR
                );
                mprintf(&g_hd_file, "{}", sequence_brackets(obj->element_type, ""));
                mprintf(&g_hd_file, ";\n}}\n");
            }
            if (in_flag & FR_AIR_FLAG) {
                mprintf(
                    &g_hd_file,
                    "if ({} == null || {}.length != _{:c}_len) {{\n",
                    tag,
                    tag,
                    ITER_CHAR
                );
                mprintf(&g_hd_file, "{} = new {}[_{:c}_len]", tag, cast_name_no_suffix, ITER_CHAR);
                mprintf(&g_hd_file, "{}", sequence_brackets(obj->element_type, ""));
                mprintf(&g_hd_file, ";\n}}\n");
            }
            mprintf(
                &g_hd_file,
                "for (int _{:c}_ind = 0; _{:c}_ind < _{:c}_len; _{:c}_ind++) {{\n",
                ITER_CHAR,
                ITER_CHAR,
                ITER_CHAR,
                ITER_CHAR
            );
            auto new_tag = fmt::format("{}[_{:c}_ind]", tag, ITER_CHAR);
            java_conv_gen_elem(owner, obj->element_type, new_tag, rec_level + 1, in_flag);
            mprintf(&g_hd_file, "}}\n}}\n");
        }
        break;

    case N_ARRAY: {
        char jchar = static_cast<char>(ITER_CHAR);
        int i;
        int subarray_count = 0;
        const ptree* element_type = base_type_of(obj->element_type);
        while (element_type->kind == N_ARRAY) {
            element_type = base_type_of(element_type->element_type);
            ++subarray_count;
        }
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (!default_value.empty()) {
                mprintf(&g_hd_file, "{};\n", default_value);
                break;
            }
            mprintf(
                &g_hd_file, "new {}", javaname(obj->element_type, ".", JAVANAME_NO_ARRAY_SUFFIX)
            );
            for (i = 0; i < static_cast<int>(obj->bounds.size()); ++i) {
                mprintf(&g_hd_file, "[{}]", integer_value(obj->bounds[i]));
            }
            for (i = 0; i < subarray_count; ++i) {
                mprintf(&g_hd_file, "[]");
            }
            if (obj->element_type->kind == N_SEQUENCE) {
                mprintf(&g_hd_file, "{}", sequence_brackets(obj->element_type, "0"));
            }
            mprintf(&g_hd_file, ";\n");
        }
        if (in_flag & DO_EXTERNAL) {
            break;
        }
        auto new_tag = tag;
        for (i = 0; i < static_cast<int>(obj->bounds.size()); ++i) {
            mprintf(
                &g_hd_file,
                "for (int _{}_ind = 0; _{}_ind < {}; _{}_ind++) {{\n",
                jchar,
                jchar,
                integer_value(obj->bounds[i]),
                jchar
            );
            new_tag += fmt::format("[_{:c}_ind]", jchar);
            jchar++;
        }
        java_conv_gen_elem(
            owner, obj->element_type, new_tag, rec_level + sequence_index_count(obj, 0), in_flag
        );
        for (i = 0; i < static_cast<int>(obj->bounds.size()); ++i) {
            mprintf(&g_hd_file, "}}\n");
            jchar--;
        }
    } break;

    case N_STRING:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (!default_value.empty()) {
                mprintf(&g_hd_file, "{};\n", default_value);
            } else {
                mprintf(&g_hd_file, "new String();\n");
            }
        } else if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(&g_hd_file, "other.{};\n", tag);
        } else if (in_flag & FR_AIR_FLAG) {
            mprintf(&g_hd_file, "{} = stream.read_{}string();\n", tag, is_wstring(obj) ? "w" : "");
        } else {
            mprintf(&g_hd_file, "stream.write_{}string({});\n", is_wstring(obj) ? "w" : "", tag);
        }
        break;
    case N_ALIAS:
    case N_MEMBER:
        java_conv_gen_elem(owner, obj->type, tag, rec_level, in_flag, alias_origin);
        break;
    case N_INTERFACE:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (default_value.empty()) {
                mprintf(&g_hd_file, "null;\n", tag);
            } else {
                mprintf(&g_hd_file, "{};\n", default_value);
            }
            break;
        }
        if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(&g_hd_file, "other.{};\n", tag, tag);
            break;
        }

        if (in_flag & FR_AIR_FLAG) {
            mprintf(&g_hd_file, "{} = stream.read_longlong();\n", tag);
        } else {
            mprintf(&g_hd_file, "stream.write_longlong({});\n", tag);
        }
        break;
    case N_MAP:
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(&g_hd_file, "{}", holder);
            if (default_value.empty()) {
                mprintf(&g_hd_file, "null;\n");
            } else {
                mprintf(&g_hd_file, "{};\n", default_value);
            }
        } else if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "{}", holder);
            mprintf(
                &g_hd_file,
                "new java.util.HashMap< {}, {} >( other.{} );\n",
                javaname(obj->key_type, ".", JAVANAME_FLAGS_CLASS),
                javaname(obj->element_type, ".", JAVANAME_FLAGS_CLASS),
                tag
            );
        } else if (in_flag & FR_AIR_FLAG) {
            auto key = fmt::format("key{}", rec_level);
            auto value = fmt::format("value{}", rec_level);
            mprintf(&g_hd_file, "{{\nint _{:c}_len = stream.read_long();\n", ITER_CHAR);
            mprintf(
                &g_hd_file,
                "if ( {} == null ) {{\n{} = new java.util.HashMap< {}, {} >();\n}} else {{\n{}.clear();\n}}\n",
                tag,
                tag,
                javaname(obj->key_type, ".", JAVANAME_FLAGS_CLASS),
                javaname(obj->element_type, ".", JAVANAME_FLAGS_CLASS),
                tag
            );
            mprintf(
                &g_hd_file,
                "for ( int {:c} = 0; {:c} < _{:c}_len; {:c}++ ) {{\n",
                ITER_CHAR,
                ITER_CHAR,
                ITER_CHAR,
                ITER_CHAR
            );
            mprintf(&g_hd_file, "{} ", javaname(obj->key_type, ".", JAVANAME_FLAGS_CLASS));
            java_conv_gen_elem(owner, obj->key_type, key, rec_level + 1, in_flag);
            mprintf(&g_hd_file, "{} ", javaname(obj->element_type, ".", JAVANAME_FLAGS_CLASS));
            switch (base_type_of(obj->element_type)->kind) {
            case N_STRUCT:
            case N_UNION:
                mprintf(
                    &g_hd_file,
                    "{} = new {}();\n",
                    value,
                    javaname(obj->element_type, ".", JAVANAME_FLAGS_CLASS)
                );
                break;
            case N_MAP:
                mprintf(
                    &g_hd_file,
                    "{} = new java.util.HashMap< {}, {} >();\n",
                    value,
                    javaname(base_type_of(obj->element_type)->key_type, ".", JAVANAME_FLAGS_CLASS),
                    javaname(
                        base_type_of(obj->element_type)->element_type, ".", JAVANAME_FLAGS_CLASS
                    )
                );
                break;
            case N_SEQUENCE:
            case N_ARRAY:
                mprintf(&g_hd_file, "{} = null;\n", value);
            default:
                break;
            }
            java_conv_gen_elem(owner, obj->element_type, value, rec_level + 1, in_flag);
            mprintf(&g_hd_file, "{}.put( {}, {} );\n", tag, key, value);
            mprintf(&g_hd_file, "}}\n}}\n");
        } else {
            auto entry = fmt::format("entry{}", rec_level);
            mprintf(&g_hd_file, "if ( {} != null ) {{\n", tag);
            mprintf(&g_hd_file, "stream.write_long( {}.size() );\n", tag);
            mprintf(
                &g_hd_file,
                "for ( java.util.Map.Entry< {}, {} > {} : {}.entrySet() ) {{\n",
                javaname(obj->key_type, ".", JAVANAME_FLAGS_CLASS),
                javaname(obj->element_type, ".", JAVANAME_FLAGS_CLASS),
                entry,
                tag
            );
            entry = fmt::format("entry{}.getKey()", rec_level);
            java_conv_gen_elem(owner, obj->key_type, entry, rec_level + 1, in_flag);
            entry = fmt::format("entry{}.getValue()", rec_level);
            java_conv_gen_elem(owner, obj->element_type, entry, rec_level + 1, in_flag);
            mprintf(&g_hd_file, "}}\n");
            mprintf(&g_hd_file, "}} else {{\n");
            mprintf(&g_hd_file, "stream.write_long( 0 );\n");
            mprintf(&g_hd_file, "}}\n");
        }
        break;
    default:
        mprintf(&g_hd_file, "//This cannot happen java_conv_gen_elem {} \n", tag);
        break;
    }
}

static void populate_java_type_library(const ptree*) {
    // TODO: idarcar
    // unsigned int i;
    // size_t cdr_size;
    // unsigned char* cdr;
    //
    // get_type_library(obj, &cdr, &cdr_size);
    // mprintf(&g_hd_file, "private static final String[] typeDefinition = new String[] {{\n\"");
    // for (i = 0; i < cdr_size; i++) {
    //     if (i != 0 && (i % 48) == 0) {
    //         mprintf(&g_hd_file, "\"{}\n\"", (i % (64 * 48)) ? " +" : ",");
    //     }
    //     mprintf(&g_hd_file, "{:x}{:x}", cdr[i] >> 4, cdr[i] & 0xf);
    // }
    // mprintf(&g_hd_file, "\"\n}};\n\n");
    // mprintf(
    //     &g_hd_file,
    //     "/**\n"
    //     " * Get a serialized representation of the type. It is described using a DDS TypeObject
    //     as\n" " * defined by the OMG standard for Extensible and Dynamic Topic Types for DDS,
    //     version 1.1.\n" " *\n" " * @return a byte array containing a big endian CDR serialization
    //     of the DDS TypeObject\n" " */\n"
    // );
    // mprintf(
    //     &g_hd_file,
    //     "public static byte[] getTypeDefinition() {{\n"
    //     "int len = 0;\n"
    //     "for (String s : typeDefinition) {{\n"
    //     "len += s.length();\n"
    //     "}}\n"
    //     "int pos = 0;\n"
    //     "byte[] data = new byte[len/2];\n"
    //     "for (String s : typeDefinition) {{\n"
    //     "for (int i = 0; i < s.length(); i += 2, pos++) {{\n"
    //     "data[pos] = (byte) ((Character.digit(s.charAt(i), 16) << 4) +
    //     Character.digit(s.charAt(i+1), 16));\n"
    //     "}}\n"
    //     "}}\n"
    //     "return data;\n"
    //     "}}\n\n"
    // );
    //
    // std::string name = idl_scoped_name(obj, nullptr);
    // if (name == "com::kongsberg::intercom::jni::KeyedBytes") {
    //     name = "DDS::KeyedBytes";
    // }
    // if (name == "com::kongsberg::intercom::jni::KeyedString") {
    //     name = "DDS::KeyedString";
    // }
    // mprintf(&g_hd_file, "public static String getTypeName() {{\nreturn \"{}\";\n}}\n", name);
    // free(cdr);
}

static void
print_doc_annotation(struct memf* f, const ptree* doc_annotation, const bool& print_as_post_doc) {
    const ptree* elem;
    const char* docs = nullptr;
    for (elem = doc_annotation->members; elem; elem = elem->next) {
        if (elem->name == "text") {
            docs = elem->value.val.str().c_str();
            break;
        }
    }
    MemfIndentScopeLock indent_lk(f);  // comments should not affect indentation
    if (docs) {
        const int post_padding = 2;
        std::string line_start = print_as_post_doc ? "//!" : " * ";
        std::string newline_padding;
        if (print_as_post_doc) {
            int spaces = std::max(f->column + post_padding - f->indent, 0);
            newline_padding += std::string(spaces, ' ');
        }
        int start_of_line = 1;
        const char* pp;
        if (print_as_post_doc) {
            mprintf(f, std::string(post_padding, ' '));
        } else {
            mprintf(f, "/**\n");
        }
        for (pp = docs; *pp; pp++) {
            if (start_of_line) {
                if (pp != docs) {
                    mprintf(f, "{}", newline_padding);
                }
                mprintf(f, "{}", line_start);
            }
            if (*pp) {
                start_of_line = *pp == '\n';
                mprintf(f, "{}", start_of_line ? '\n' : *pp);
            }
        }
        if (!start_of_line) {
            mprintf(f, "\n");
        }
        if (!print_as_post_doc) {
            mprintf(f, " */\n");
        }
    }
}

void emit_docs(struct memf* f, const ptree* obj) {
    if (!f || !obj) {
        return;
    }
    for (const ptree* ann : obj->annotations) {
        if (is_pre_doc(ann)) {
            print_doc_annotation(f, ann, false);
        }
    }
}

/// unlike emit_docs(), emit_post_docs() is expected to be used without a preceding '\n'
/// @n will always end with a newline
void emit_post_docs(struct memf* f, const ptree* obj) {
    if (!f || !obj) {
        return;
    }
    bool no_comments = true;
    for (const ptree* ann : obj->annotations) {
        if (is_post_doc(ann)) {
            print_doc_annotation(f, ann, true);
            no_comments = false;
        }
    }
    if (no_comments) {
        mprintf(f, "\n");
    }
}

static const ptree* parent_base_type(const ptree* obj) {
    return !obj->parents.empty() ? base_type_of(obj->parents[0]) : nullptr;
}

static void java_gen_method_body(const ptree* obj, const std::string& tag, unsigned int in_flag) {
    size_t taglen = tag.size();
    const ptree* base = parent_base_type(obj);

    if (base) {
        if (in_flag & FR_AIR_FLAG) {
            mprintf(
                &g_hd_file,
                "{}Helper.read(stream, titem);\n",
                filter_package(javaname(base, ".", 0), obj)
            );
        } else if (in_flag == 0) {
            mprintf(
                &g_hd_file,
                "{}Helper.write(stream, titem);\n",
                filter_package(javaname(base, ".", 0), obj)
            );
        }
    }

    if (obj->kind == N_UNION) {
        const ptree* discr = base_type_of(obj->discriminator);
        if (in_flag & DO_CONSTRUCTOR) {
            mprintf(
                &g_hd_file,
                "discriminator({});\n",
                javavalue(get_default_value(obj->discriminator), 1)
            );
            return;
        }
        if (in_flag & DO_DISCRIMINATOR) {
            in_flag |= DO_CONSTRUCTOR;
        } else if (in_flag & DO_CLONER) {
            mprintf(&g_hd_file, "discriminator(other.discriminator());\n");
        } else if (in_flag & FR_AIR_FLAG) {
            if (discr->kind == N_ENUM) {
                mprintf(
                    &g_hd_file,
                    "{}.discriminator({}.fr_ordinal(stream.read_{}()));\n",
                    tag,
                    javaname(discr, ".", 0),
                    stream_integer_type(discr)
                );
            } else {
                mprintf(
                    &g_hd_file,
                    "{}.discriminator(stream.read_{}());\n",
                    tag,
                    stream_integer_type(discr)
                );
            }
        } else {
            if (discr->kind == N_ENUM) {
                mprintf(
                    &g_hd_file,
                    "stream.write_{}({}.discriminator().my_ordinal());\n",
                    stream_integer_type(discr),
                    tag
                );
            } else {
                mprintf(
                    &g_hd_file,
                    "stream.write_{}({}.discriminator());\n",
                    stream_integer_type(discr),
                    tag
                );
            }
        }
        if (base_type_of(obj->discriminator->type) != &boolean_type) {
            mprintf(&g_hd_file, "switch ({}{}discriminator()) {{\n", tag, (taglen > 0) ? "." : "");
        }
    }
    for (auto elem : obj->members) {
        // TODO: why was this necessary, again?
        if (obj->kind == N_UNION) {
            for (auto cas : elem->members) {
                if (base_type_of(obj->discriminator->type) == &boolean_type) {
                    if (elem->kind != N_NULL) {
                        if (cas != elem->members) {
                            mprintf(&g_hd_file, "}}\nelse ");
                        }
                        if ((cas->flags & OPT_DEFAULT) == 0) {
                            const char* not_value = integer_value(cas->value) == 0 ? "!" : "";
                            mprintf(
                                &g_hd_file,
                                "if ( {}{}{}discriminator() )",
                                not_value,
                                tag,
                                (taglen > 0) ? "." : ""
                            );
                        }
                        mprintf(&g_hd_file, "\n{{\n");
                    }
                } else {
                    if (cas->flags & OPT_DEFAULT) {
                        mprintf(&g_hd_file, "default:\n");
                    } else {
                        mprintf(&g_hd_file, "case {}:\n", javavalue(cas->value, 0));
                    }
                }
            }
        }
        if (elem->kind == N_MEMBER) {
            if (is_non_serialized(elem)) {
                continue;
            }
            auto value = fmt::format(
                "{}{}{}{}",
                tag,
                (taglen > 0) ? "." : "",
                (obj->kind == N_UNION) ? "_" : "",
                elem->name
            );
            if (is_optional(elem) && (in_flag & (DO_CONSTRUCTOR | DO_CLONER)) == 0) {
                if (in_flag & FR_AIR_FLAG) {
                    mprintf(&g_hd_file, "if ( stream.read_boolean() ) {{\n");
                } else {
                    mprintf(&g_hd_file, "stream.write_boolean( {} != null );\n", value);
                    mprintf(&g_hd_file, "if ( {} != null ) {{\n", value);
                }
            }
            java_conv_gen_elem(obj, elem->type, value, 0, in_flag, elem);
            if (is_optional(elem) && (in_flag & (DO_CONSTRUCTOR | DO_CLONER)) == 0) {
                mprintf(&g_hd_file, "}}\n");
            }
            if (obj->kind == N_UNION && base_type_of(obj->discriminator->type) != &boolean_type) {
                mprintf(&g_hd_file, "break;\n");
            }
        }
    }
    if (obj->kind == N_UNION) {
        mprintf(&g_hd_file, "}}\n");
    }
}

static int java_argument_constructor(const ptree* owner, const ptree* obj, int pos) {
    if (obj) {
        pos = java_argument_constructor(owner, parent_base_type(obj), pos);
        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                if (pos++ > 0) {
                    mprintf(&g_hd_file, ", ");
                }
                mprintf(
                    &g_hd_file,
                    "{} {}",
                    filter_package(javaname(base_type_of(elem->type), ".", 0), owner),
                    elem->name
                );
            }
        }
    }
    return pos;
}

static int java_argument_constructor_super(const ptree* obj, int pos) {
    const ptree* elem;
    if (obj) {
        pos = java_argument_constructor_super(parent_base_type(obj), pos);
        for (elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                if (pos++ > 0) {
                    mprintf(&g_hd_file, ", ");
                }
                mprintf(&g_hd_file, "{}", elem->name);
            }
        }
    }
    return pos;
}

static void java_struct_c_def(const ptree* obj) {
    auto fixed_java_name = javaname(obj, "", 0);

    mprintf(&g_hd_file, "\npublic class {}", fixed_java_name);
    if (!obj->parents.empty()) {
        mprintf(
            &g_hd_file, " extends {}", filter_package(javaname(parent_base_type(obj), ".", 0), obj)
        );
    }
    mprintf(&g_hd_file, " {{\n\n");

    // Default constructor
    mprintf(&g_hd_file, "public {} () {{\n", fixed_java_name);
    if (!obj->parents.empty()) {
        mprintf(&g_hd_file, "super();\n");
    }
    java_gen_method_body(obj, "", DO_CONSTRUCTOR);
    mprintf(&g_hd_file, "}}\n\n");

    // Copy constructor
    mprintf(&g_hd_file, "public {} ({} other) {{\n", fixed_java_name, fixed_java_name);
    if (!obj->parents.empty()) {
        mprintf(&g_hd_file, "super(other);\n");
    }
    java_gen_method_body(obj, "", DO_CLONER);
    mprintf(&g_hd_file, "}}\n\n");

    if (obj->kind == N_STRUCT && obj->members != nullptr) {
        // Argument constructor
        mprintf(&g_hd_file, "public {} (", obj->name);
        java_argument_constructor(obj, obj, 0);
        mprintf(&g_hd_file, ")\n{{\n");
        if (!obj->parents.empty()) {
            mprintf(&g_hd_file, "super(");
            java_argument_constructor_super(parent_base_type(obj), 0);
            mprintf(&g_hd_file, ");\n");
        }
        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                mprintf(&g_hd_file, "this.{} = {};\n", elem->name, elem->name);
            }
        }
        mprintf(&g_hd_file, "}}\n\n");
    }

    // Clone
    mprintf(
        &g_hd_file,
        "@Override\npublic {} clone()\n{{\nreturn new {}( this );\n}}\n\n",
        fixed_java_name,
        fixed_java_name
    );

    for (const ptree* elem = obj->members; elem; elem = elem->next) {
        auto name = javaname(
            base_type_of(elem->type),
            ".",
            is_optional(elem) && is_primitive(base_type_of(elem)) ? JAVANAME_FLAGS_CLASS : 0
        );

        emit_docs(&g_hd_file, elem);

        if (obj->kind == N_UNION) {
            if (elem->kind != N_NULL) {
                mprintf(
                    &g_hd_file, "public {} {}() {{return _{};}}\n", name, elem->name, elem->name
                );
                mprintf(&g_hd_file, "public void {}({} {}){{\n", elem->name, name, elem->name);
                mprintf(&g_hd_file, "_discriminator = {};\n", javavalue(elem->members->value, 1));
                mprintf(&g_hd_file, "this._{} = {};\n}}\n", elem->name, elem->name);
            }
        } else {
            mprintf(&g_hd_file, "public ");
            java_par_elem(
                obj,
                elem,
                elem->name,
                is_optional(elem) && is_primitive(base_type_of(elem)) ? JAVANAME_FLAGS_CLASS : 0
            );
            mprintf(&g_hd_file, ";\n");
        }
    }

    if (CommandLineOption::generate_default_literals() && obj->kind == N_STRUCT && obj->members) {
        std::map<std::string, const ptree*> local_member_names;
        for (const auto& member : obj->members) {
            local_member_names[member->name] = member;
        }
        mprintf(&g_hd_file, "\n");
        for (const auto& member : obj->members) {
            std::string member_name(member->name);
            std::transform(
                member_name.begin(),
                member_name.end(),
                member_name.begin(),
                [](const char& c) { return static_cast<char>(std::toupper(static_cast<int>(c))); }
            );
            std::string var_name = std::string("DEFAULT_") + member_name;
            // prepend '_' until no name conflicts
            while (local_member_names.find(var_name) != local_member_names.end()) {
                var_name.insert(var_name.begin(), '_');
            }
            mprintf(&g_hd_file, "public static final ");
            java_par_elem(obj, member->type, "", 0);
            const ptree* owner = obj;  // avoid clang-tidy warning
            java_conv_gen_elem(owner, member, var_name, 0, DO_CONSTRUCTOR | DO_EXTERNAL);
        }
    }

    if (obj->kind == N_UNION) {
        auto name = javaname(obj->discriminator, ".", 0);
        mprintf(&g_hd_file, "public {} discriminator() {{return _discriminator;}}\n\n", name);
        mprintf(&g_hd_file, "public void discriminator({} discriminator) {{\n", name);
        mprintf(&g_hd_file, "if (_discriminator != discriminator) {{\n");
        mprintf(&g_hd_file, "_discriminator = discriminator;\n");
        java_gen_method_body(obj, "", DO_DISCRIMINATOR);
        mprintf(&g_hd_file, "}}\n}}\n\n");
        mprintf(&g_hd_file, "private {} _discriminator;\n", name);

        for (const ptree* elem = obj->members; elem; elem = elem->next) {
            if (elem->kind == N_MEMBER) {
                mprintf(&g_hd_file, "protected ");
                auto elem_name = "_" + elem->name;
                java_par_elem(
                    obj,
                    elem,
                    elem_name,
                    is_optional(elem) && is_primitive(base_type_of(elem)) ? JAVANAME_FLAGS_CLASS : 0
                );
                mprintf(&g_hd_file, ";\n");
            }
        }
    }
    mprintf(&g_hd_file, "\n}};\n");
}

static void prep_file(const ptree* obj, struct memf* file) {
    mreset(file);
    auto package = javaname(obj->super, ".", 0);

    // build info header
    if (!package.empty()) {
        mprintf(file, "package {};\n", package);
    }
}

static std::string java_filename(const ptree* obj) {
    auto name = javaname(obj, "/", 0);
    for (auto& c : name) {
        if (c == '.') {
            c = '/';
        }
    }
    return name;
}

static void java_emit_typedefinition(const ptree* obj) {
    auto fixed_java_class_name = javaname(obj, ".", 0);
    auto file = java_filename(obj);

    // Create type definition class
    prep_file(obj, &g_hd_file);
    mprintf(
        &g_hd_file,
        "\n/**\n"
        " * InterCOM type definition for {}.\n"
        " * @see {}\n"
        " */\n",
        fixed_java_class_name,
        fixed_java_class_name
    );

    mprintf(&g_hd_file, "final public class {}TypeDefinition\n{{\n", java_name(obj));

    populate_java_type_library(obj);

    mprintf(&g_hd_file, "}}\n");

    java_savememf(&g_hd_file, file, "{}TypeDefinition.java");
}

static void java_emit_struct(const ptree* obj) {
    std::string fixed_java_name = javaname(obj, "", 0);
    std::string fixed_java_class_name = javaname(obj, ".", 0);
    std::string filename = java_filename(obj);
    std::string javname = java_name(obj);

    // Create type class
    prep_file(obj, &g_hd_file);

    emit_docs(&g_hd_file, obj);
    java_struct_c_def(obj);

    java_savememf(&g_hd_file, filename, "{}.java");

    // Create sequence holder class
    prep_file(obj, &g_hd_file);

    mprintf(
        &g_hd_file,
        "/**\n * Holder for sequences of {} objects.\n"
        " * This is used for parameter passing where pass-by-reference\n"
        " * semantics are needed, eg. when a operation creates an array\n"
        " * and needs to return a reference to that array.\n"
        " * @see {}\n"
        "*/\n",
        fixed_java_class_name,
        fixed_java_class_name
    );

    mprintf(
        &g_hd_file,
        "final public class {}SeqHolder\n"
        "{{\n"
        "public {}[] value;\n"
        "public {}SeqHolder() {{}}\n"
        "public {}SeqHolder({}[] initial) {{value=initial;}}\n"
        "}}\n\n",
        javname,
        fixed_java_name,
        javname,
        javname,
        fixed_java_name
    );

    java_savememf(&g_hd_file, filename, "{}SeqHolder.java");

    // Create holder class
    prep_file(obj, &g_hd_file);

    mprintf(
        &g_hd_file,
        "/**\n"
        " * Holder for references to {} objects.\n"
        " * This is used for parameter passing where pass-by-reference\n"
        " * semantics are needed, eg. when a operation creates an object\n"
        " * and needs to return a reference tothat object.\n"
        " * @see {}\n"
        " */\n",
        fixed_java_class_name,
        fixed_java_class_name
    );

    mprintf(
        &g_hd_file,
        "final public class {}Holder\n"
        "{{\n"
        "public {} value;\n"
        "public {}Holder() {{}}\n"
        "public {}Holder({} initial) {{value=initial;}}\n"
        "}}\n\n",
        javname,
        fixed_java_name,
        javname,
        javname,
        fixed_java_name
    );

    java_savememf(&g_hd_file, filename, "{}Holder.java");

    // Create helper class
    prep_file(obj, &g_hd_file);

    mprintf(&g_hd_file, "\nimport org.omg.CORBA.portable.InputStream;\n");
    mprintf(&g_hd_file, "import org.omg.CORBA.portable.OutputStream;\n");
    if (namespace_of(obj) != nullptr) {
        mprintf(&g_hd_file, "import {}TypeDefinition;\n\n", fixed_java_class_name);
    }

    mprintf(
        &g_hd_file,
        "/**\n"
        " * Helper for {} objects.\n"
        " * @see {}\n"
        " */\n",
        fixed_java_class_name,
        fixed_java_class_name
    );

    mprintf(&g_hd_file, "final public class {}Helper\n{{\n", javname);

    mprintf(
        &g_hd_file, "public static void write(OutputStream stream, {} titem)\n{{\n", fixed_java_name
    );
    java_gen_method_body(obj, "titem", 0);
    mprintf(&g_hd_file, "}}\n\n");

    mprintf(
        &g_hd_file,
        "public static {} read(InputStream stream, {} titem)\n{{\n",
        fixed_java_name,
        fixed_java_name
    );
    mprintf(
        &g_hd_file,
        "if (titem == null){{\n"
        "titem = new {}();\n"
        "}}\n",
        fixed_java_name
    );
    java_gen_method_body(obj, "titem", FR_AIR_FLAG);
    mprintf(&g_hd_file, "return titem;\n}}\n\n");

    mprintf(
        &g_hd_file,
        "public static {} read(InputStream stream)\n"
        "{{\n"
        "return read(stream, new {}());\n"
        "}}\n",
        fixed_java_name,
        fixed_java_name
    );

    mprintf(&g_hd_file, "}}\n");

    java_savememf(&g_hd_file, filename, "{}Helper.java");
}

static void java_emit_enum(const ptree* obj) {
    std::string fixed_java_name = javaname(obj, "", 0);
    prep_file(obj, &g_hd_file);
    emit_docs(&g_hd_file, obj);

    mprintf(&g_hd_file, "public enum {} {{\n", fixed_java_name);
    const ptree* elem = obj->members;
    while (elem) {
        if (obj->flags & OPT_ENUMERATED) {
            mprintf(
                &g_hd_file,
                "{}({}{})",
                elem->name,
                java_integer_cast(obj),
                integer_value(elem->value)
            );
        } else {
            mprintf(&g_hd_file, "{}", elem->name);
        }
        elem = elem->next;
        if (elem) {
            mprintf(&g_hd_file, ",\n");
        } else {
            mprintf(&g_hd_file, ";\n");
        }
    }
    if (obj->flags & OPT_ENUMERATED) {
        mprintf(
            &g_hd_file,
            "{} the_ordinal;\n"
            "{}({} a_ordinal){{the_ordinal = a_ordinal;}}\n"
            "public final {} my_ordinal(){{return the_ordinal;}}\n",
            java_integer_type(obj),
            fixed_java_name,
            java_integer_type(obj),
            java_integer_type(obj)
        );
    } else {
        mprintf(
            &g_hd_file,
            "public final {} my_ordinal(){{return {}ordinal();}}\n",
            java_integer_type(obj),
            java_integer_cast(obj)
        );
    }
    mprintf(
        &g_hd_file,
        "public static final {} fr_ordinal({} val)\n{{\n",
        fixed_java_name,
        java_integer_type(obj)
    );
    mprintf(&g_hd_file, "switch (val) {{\n");
    for (elem = obj->members; elem; elem = elem->next) {
        mprintf(
            &g_hd_file,
            "case {}{} : return {} ;\n",
            java_integer_cast(obj),
            integer_value(elem->value),
            elem->name
        );
    }
    mprintf(&g_hd_file, "}}\n");
    mprintf(&g_hd_file, "return {} ;\n", obj->members->name);
    mprintf(&g_hd_file, "}};\n");
    mprintf(&g_hd_file, "}};\n");

    auto filename = java_filename(obj);
    java_savememf(&g_hd_file, filename, "{}.java");
}

static void java_emit_const(const ptree* obj) {
    auto fixed_java_name = javaname(obj, "", 0);
    prep_file(obj, &g_hd_file);
    emit_docs(&g_hd_file, obj);
    const ptree* derived = base_type_of(obj);
    auto derived_name = javaname(derived, ".", 0);

    mprintf(
        &g_hd_file,
        "public interface {}\n"
        "{{\n\n"
        "public static final {} value = ",
        fixed_java_name,
        derived_name
    );
    if (derived->kind == N_INTERFACE || derived->kind == N_ARRAY) {
        mprintf(&g_hd_file, "null;\n");
    } else if (derived->kind == N_STRUCT || derived->kind == N_UNION) {
        mprintf(&g_hd_file, "new {} ();\n", derived_name);
    } else if (derived->kind == N_STRING) {
        mprintf(&g_hd_file, "\"{}\";\n", string_value(obj->value));
    } else if (obj->value.kind() == PTREE_KIND &&
               base_type_of(obj->value.val.node()) != base_type_of(obj)) {
        mprintf(&g_hd_file, "{};\n", javavalue(obj->value.val.node()->value, 1));
    } else {
        mprintf(&g_hd_file, "{};\n", javavalue(obj->value, 1));
    }
    mprintf(&g_hd_file, "}}\n\n");

    auto filename = java_filename(obj);
    java_savememf(&g_hd_file, filename, "{}.java");
}

static std::set<const ptree*> g_seen_obj;

static void cgjava_recurs(const ptree* obj) {
    if (g_seen_obj.find(obj) != g_seen_obj.end()) {
        return;
    }
    g_seen_obj.insert(obj);
    for (; obj; obj = obj->next) {
        if (is_emit(obj, LANG_JAVA)) {
            switch (obj->kind) {
            case N_MODULE:
                cgjava_recurs(obj->members);
                break;
            case N_STRUCT:
            case N_UNION:
                if ((obj->flags & OPT_DECLARATION) == 0) {
                    java_emit_typedefinition(obj);
                    if (!CommandLineOption::generate_typesupport_only()) {
                        java_emit_struct(obj);
                    }
                }
                break;
            case N_ENUM:
            case N_BITMASK:
                if (!CommandLineOption::generate_typesupport_only()) {
                    java_emit_enum(obj);
                }
                break;
            case N_CONST:
                if (!(obj->flags & OPT_DECLARATION) &&
                    !CommandLineOption::generate_typesupport_only()) {
                    java_emit_const(obj);
                }
                break;
            default:
                break;
            }
        }
    }
}

void intercom::cidl::code_gen_java(const parse_result* result) {
    g_seen_obj.clear();
    cgjava_recurs(result->tree);
}

void intercom::cidl::code_gen_java(const parse_result* result, const char* destination) {
    intercom::cidl::CommandLineOption::get_instance().java_target_directory = destination;
    code_gen_java(result);
}
