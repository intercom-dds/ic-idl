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

#include <algorithm>
#include <cassert>
#include <cstring>
#include <sstream>
#include <string>
#include <string_view>
#include <vector>

#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/memf.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

std::string get_const_value(const numeric& value, const ptree* scope);

static void add_flag(std::string& flag, const std::string& value) {
    const std::string flag_type = "intercom::ULong";
    auto is_null = [&flag_type](const std::string& f) {
        return f == "0" || f == flag_type + "(0)";
    };
    if (flag.find(value) != std::string::npos || (!flag.empty() && is_null(value))) {
        return;
    }
    // prep flag
    if (is_null(flag)) {
        flag = "";
    }
    if (!flag.empty()) {
        flag += "|";
    }
    // append value
    if (value.find(flag_type) == 0U) {
        flag += value;
    } else {
        flag += fmt::format("{}({})", flag_type, value);
    }
}

static void get_type_flag_name(std::string& flag, const ptree* obj) {
    if (flag.empty()) {
        flag = "0";
    }
    if (obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_EXCEPTION) {
        int kind = get_extensibility(obj);
        switch (kind) {
        case FINAL_EXTENSIBILITY:
            add_flag(flag, "intercom::dcps::xtypes::IS_FINAL");
            break;
        case MUTABLE_EXTENSIBILITY:
            add_flag(flag, "intercom::dcps::xtypes::IS_MUTABLE");
            break;
        case EXTENSIBLE_EXTENSIBILITY:
        default:
            add_flag(flag, "intercom::dcps::xtypes::IS_APPENDABLE");
            break;
        }
        if (is_nested(obj)) {
            add_flag(flag, "intercom::dcps::xtypes::IS_NESTED");
        }
    }
}

static void get_member_flag_name(std::string& flag, const ptree* elem) {
    if (flag.empty()) {
        flag = "0";
    }
    if (get_annotation(elem, annotation_type_try_construct)) {
        auto try_construct =
            integer_value(get_annotation(elem, annotation_type_try_construct)->members->value);
        switch (try_construct) {
        case 0:  // use default
            add_flag(flag, "intercom::dcps::xtypes::IS_USE_DEFAULT_TRY_CONSTRUCT");
            break;
        case 1:  // discard
            add_flag(flag, "intercom::dcps::xtypes::IS_DISCARD_TRY_CONSTRUCT");
            break;
        case 2:  // trim
            add_flag(flag, "intercom::dcps::xtypes::IS_TRIM_TRY_CONSTRUCT");
            break;
        default:
            break;
        }
    }
}

static std::string get_member_flag_name(const ptree* obj) {
    std::string flag;
    if (is_key_member(obj)) {
        add_flag(flag, "intercom::dcps::xtypes::IS_KEY");
    }
    if (is_optional(obj)) {
        add_flag(flag, "intercom::dcps::xtypes::IS_OPTIONAL");
    }
    if (is_shared(obj)) {
        add_flag(flag, "intercom::dcps::xtypes::IS_EXTERNAL");
    }
    if (is_must_understand(obj)) {
        add_flag(flag, "intercom::dcps::xtypes::IS_MUST_UNDERSTAND");
    }
    if (flag.empty()) {
        add_flag(flag, "0");
    }
    return flag;
}

static const char* get_type_kind_name(const ptree* obj) {
    const char* res = "";
    switch (obj->kind) {
    case N_PRIMITIVE:
        if (obj == &boolean_type) {
            res = "TK_BOOLEAN";
        } else if (obj == &short_type) {
            res = "TK_INT_16";
        } else if (obj == &ushort_type) {
            res = "TK_UINT_16";
        } else if (obj == &long_type) {
            res = "TK_INT_32";
        } else if (obj == &ulong_type) {
            res = "TK_UINT_32";
        } else if (obj == &longlong_type) {
            res = "TK_INT_64";
        } else if (obj == &ulonglong_type) {
            res = "TK_UINT_64";
        } else if (obj == &float_type) {
            res = "TK_FLOAT_32";
        } else if (obj == &double_type) {
            res = "TK_FLOAT_64";
        } else if (obj == &char_type) {
            res = "TK_CHAR_8";
        } else if (obj == &wchar_type) {
            res = "TK_CHAR_16";
        } else if (obj == &octet_type) {
            res = "TK_UINT8";
        } else if (obj == &int8_type) {
            res = "TK_INT8";
        }
        break;
    case N_ALIAS:
        if (obj->type->kind == N_MAP || obj->type->kind == N_ARRAY ||
            obj->type->kind == N_SEQUENCE || obj->type->kind == N_STRING) {
            res = get_type_kind_name(obj->type);
        } else {
            res = "TK_ALIAS";
        }
        break;
    case N_STRUCT:
    case N_VALUETYPE:
    case N_EXCEPTION:
        res = "TK_STRUCTURE";
        break;
    case N_UNION:
        res = "TK_UNION";
        break;
    case N_BITMASK:
        res = "TK_BITMASK";
        break;
    case N_ENUM:
        res = "TK_ENUM";
        break;
    case N_STRING:
        res = is_wstring(obj) ? "TK_STRING16" : "TK_STRING8";
        break;
    case N_ANNOTATION:
        res = "TK_ANNOTATION";
        break;
    case N_ARRAY:
        res = "TK_ARRAY";
        break;
    case N_MAP:
        res = "TK_MAP";
        break;
    case N_SEQUENCE:
        res = "TK_SEQUENCE";
        break;
    default:
        res = "TK_NO_TYPE";
        break;
    }
    return res;
}

static std::vector<const ptree*> compound_members(const ptree* obj) {
    std::vector<const ptree*> res;
    if (obj->discriminator) {
        res.push_back(obj->discriminator);
    } else if (!obj->parents.empty()) {
        res = compound_members(obj->parents[0]);
    }
    for (auto member : obj->members) {
        if (is_non_serialized(member)) {
            continue;
        }
        if (member->kind == N_MEMBER || member->kind == N_CONST) {
            res.push_back(member);
        }
    }
    return res;
}

static std::string get_type_name(const ptree* obj);

static std::string get_array_type_name(const ptree* obj, size_t idx) {
    std::string res;
    if (obj->kind == N_ARRAY) {
        res = get_type_name(obj->element_type);
        std::stringstream bound;
        while (idx < obj->bounds.size()) {
            bound << "[" << integer_value(obj->bounds[idx]) << "]";
            idx++;
        }
        res = res + bound.str();
    }
    return res;
}

static std::string get_type_name(const ptree* obj) {
    return idl_scoped_name(obj, nullptr);
}

static std::string gen_element_type_info(
    struct memf* memf,
    const ptree* elem,
    const ptree* base,
    const std::string& suffix,
    std::string_view funcname
);

static std::string gen_array_type_info(
    struct memf* memf,
    const ptree* elem,
    const ptree* base,
    size_t indx,
    const std::string& suffix,
    std::string_view funcname
) {
    std::string flag;
    get_type_flag_name(flag, base);
    if (indx < base->bounds.size()) {
        mprintf(
            memf,
            "static intercom::TypeInfo {}_type_info_{} = {{ \"{}\", intercom::dcps::xtypes::{}, {}, {}, 0, {}, "
            "nullptr, nullptr, nullptr, nullptr, {}, 0, nullptr }};\n",
            funcname,
            suffix,
            get_array_type_name(base, indx),
            get_type_kind_name(base),
            flag,
            get_bit_size(base),
            integer_value(base->bounds[indx]),
            gen_array_type_info(memf, elem, base, indx + 1, suffix + "_dim", funcname)
        );
        return fmt::format("&{}_type_info_{}", funcname, suffix);
    }
    return gen_element_type_info(
        memf, elem, base_type_of(base->element_type), suffix + "_element", funcname
    );
}

static const char* primitive_type_info(const ptree* obj) {
    if (obj == &boolean_type) {
        return "::intercom::Boolean_type_info";
    }
    if (obj == &int8_type) {
        return "::intercom::Int8_type_info";
    }
    if (obj == &octet_type) {
        return "::intercom::Uint8_type_info";
    }
    if (obj == &char_type) {
        return "::intercom::Char_type_info";
    }
    if (obj == &wchar_type) {
        return "::intercom::Char16_type_info";
    }
    if (obj == &short_type) {
        return "::intercom::Short_type_info";
    }
    if (obj == &ushort_type) {
        return "::intercom::UShort_type_info";
    }
    if (obj == &long_type) {
        return "::intercom::Long_type_info";
    }
    if (obj == &ulong_type) {
        return "::intercom::ULong_type_info";
    }
    if (obj == &longlong_type) {
        return "::intercom::LongLong_type_info";
    }
    if (obj == &ulonglong_type) {
        return "::intercom::ULongLong_type_info";
    }
    if (obj == &float_type) {
        return "::intercom::Float_type_info";
    }
    if (obj == &double_type) {
        return "::intercom::Double_type_info";
    }
    if (obj == &ldouble_type) {
        return "::intercom::LongDouble_type_info";
    }
    return "";
}

static std::string gen_element_type_info(
    struct memf* memf,
    const ptree* elem,
    const ptree* base,
    const std::string& suffix,
    std::string_view funcname
) {
    auto objname = cpp_type_name(base, nullptr);
    std::string flags;
    get_type_flag_name(flags, base);
    get_member_flag_name(flags, elem);
    if (base->kind == N_ENUM || base->kind == N_BITMASK) {
        if ((get_bit_size(elem) != 0 && get_bit_size(elem) != get_bit_size(base)) || flags != "0") {
            auto name = fmt::format(
                "intercom::TypeTraits<{}{}>::type_info", objname, is_bitmask(base) ? "Bits" : ""
            );
            mprintf(
                memf,
                "static intercom::TypeInfo {}_type_info_{} = {{ {}.name, {}.kind, flags, {}, {}.value_offset, "
                "{}.max_length, {}.default_value, {}.min_value, {}.max_value, {}.key_type, {}.element_type, "
                "{}.member_count, "
                "{}.members }};\n",
                funcname,
                suffix,
                name,
                name,
                flags,
                get_bit_size(elem),
                name,
                name,
                name,
                name,
                name,
                name,
                name,
                name,
                name
            );
            return fmt::format("&{}_type_info_{}", funcname, suffix);
        }
        return fmt::format(
            "&intercom::TypeTraits<{}{}>::type_info", objname, is_bitmask(base) ? "Bits" : ""
        );
    }

    if (base->kind == N_STRUCT || base->kind == N_UNION || base->kind == N_VALUETYPE ||
        base->kind == N_EXCEPTION) {
        return fmt::format("&intercom::TypeTraits<{}>::type_info", objname);
    }

    if (base->kind == N_STRING) {
        int length_bit_size = 32;
        if (get_annotation(elem, annotation_type_ext_length_bit_bound)) {
            length_bit_size = integer_value(
                get_annotation(elem, annotation_type_ext_length_bit_bound)->members->value
            );
        }
        std::string default_value = "nullptr";
        if (has_default_value(elem)) {
            default_value = fmt::format("{}_{}_{}_default", funcname, elem->name, suffix);
            mprintf(
                memf,
                "static {} {} = {};\n",
                cpp_type_name(base_type_of(elem), nullptr),
                default_value,
                get_const_value(get_default_value(elem), nullptr)
            );
            default_value = std::string("&") + default_value;
        }
        auto element_type_info = gen_element_type_info(
            memf, elem, is_wstring(base) ? &wchar_type : &char_type, suffix + "_element", funcname
        );
        if (get_annotation(elem, annotation_type_ext_vmf_decimal)) {
            auto decimal_ann = get_annotation(elem, annotation_type_ext_vmf_decimal);
            auto char_len = integer_value(get_annotation_value(decimal_ann, "chars"));
            auto decimal_bits = integer_value(get_annotation_value(decimal_ann, "decimal_bits"));
            add_flag(flags, "intercom::dcps::xtypes::IS_XRI_SEQUENCE");
            mprintf(
                memf,
                "static intercom::TypeInfo {}_type_info_{} = {{ \"{}\", intercom::dcps::xtypes::{}, {}, {}, {}, "
                "{}, {}, nullptr, nullptr, nullptr, {}, 0, nullptr }};\n",
                funcname,
                suffix,
                get_type_name(base),
                get_type_kind_name(base),
                flags,
                decimal_bits,
                char_len,
                !base->bounds.empty() ? integer_value(base->bounds[0]) : 0,
                default_value,
                element_type_info
            );
        } else {
            mprintf(
                memf,
                "static intercom::TypeInfo {}_type_info_{} = {{ \"{}\", intercom::dcps::xtypes::{}, {}, {}, 0, {}, "
                "{}, nullptr, nullptr, nullptr, {}, 0, nullptr }};\n",
                funcname,
                suffix,
                get_type_name(base),
                get_type_kind_name(base),
                flags,
                length_bit_size,
                !base->bounds.empty() ? integer_value(base->bounds[0]) : 0,
                default_value,
                element_type_info
            );
        }
        return fmt::format("&{}_type_info_{}", funcname, suffix);
    }

    if (base->kind == N_SEQUENCE) {
        int length_bit_size = 32;
        if (get_annotation(elem, annotation_type_ext_length_bit_bound)) {
            length_bit_size = integer_value(
                get_annotation(elem, annotation_type_ext_length_bit_bound)->members->value
            );
        }
        if (get_annotation(elem, annotation_type_ext_vmf_xri)) {
            add_flag(flags, "intercom::dcps::xtypes::IS_XRI_SEQUENCE");
        }
        if (get_annotation(elem, annotation_type_ext_repeat_count)) {
            add_flag(flags, "intercom::dcps::xtypes::HAS_DYNAMIC_ELEMENT_SIZE");
        }
        mprintf(
            memf,
            "static intercom::TypeInfo {}_type_info_{} = {{ \"{}\", intercom::dcps::xtypes::{}, {}, {}, 0, {}, "
            "nullptr, nullptr, nullptr, nullptr, {}, 0, nullptr }};\n",
            funcname,
            suffix,
            get_type_name(base),
            get_type_kind_name(base),
            flags,
            length_bit_size,
            !base->bounds.empty() ? integer_value(base->bounds[0]) : 0,
            gen_element_type_info(
                memf,
                base->element_type,
                base_type_of(base->element_type),
                suffix + "_element",
                funcname
            )
        );
        return fmt::format("&{}_type_info_{}", funcname, suffix);
    }
    if (base->kind == N_ARRAY) {
        return gen_array_type_info(memf, elem, base, 0, suffix, funcname);
    }
    if (base->kind == N_INTERFACE) {
        return "nullptr";
    }
    if (base->kind == N_MAP) {
        int length_bit_size = 32;
        if (get_annotation(elem, annotation_type_ext_length_bit_bound)) {
            length_bit_size = integer_value(
                get_annotation(elem, annotation_type_ext_length_bit_bound)->members->value
            );
        }
        mprintf(
            memf,
            "static intercom::TypeInfo {}_type_info_{} = {{ \"{}\", intercom::dcps::xtypes::{}, {}, {}, 0, {}, "
            "nullptr, nullptr, nullptr, {}, "
            "{}, 0, nullptr }};\n",
            funcname,
            suffix,
            get_type_name(base),
            get_type_kind_name(base),
            flags,
            length_bit_size,
            !base->bounds.empty() ? integer_value(base->bounds[0]) : 0,
            gen_element_type_info(
                memf, base->key_type, base_type_of(base->key_type), suffix + "_key", funcname
            ),
            gen_element_type_info(
                memf,
                base->element_type,
                base_type_of(base->element_type),
                suffix + "_value",
                funcname
            )
        );
        return fmt::format("&{}_type_info_{}", funcname, suffix);
    }
    if (is_primitive(base)) {
        int offset = 0;
        if (get_annotation(elem, annotation_type_ext_value_offset)) {
            offset =
                integer_value(get_annotation(elem, annotation_type_ext_value_offset)->members->value
                );
        }
        if (elem->flags & OPT_SEQUENCE_LENGTH) {
            add_flag(flags, "intercom::dcps::xtypes::IS_ELEMENT_SIZE");
        }
        if (get_direct_annotation(elem, annotation_type_ext_jaus_integer)) {
            add_flag(flags, "intercom::dcps::xtypes::IS_INTEGER_RANGE_VALUE");
        }
        int bit_size = get_bit_size(elem) != 0 ? get_bit_size(elem) : get_bit_size(base);
        const ptree* primitive_elem = elem;
        if (base_type_of(base)->kind != base_type_of(primitive_elem)->kind) {
            primitive_elem = base;
        }
        if (bit_size != get_bit_size(base) || offset != 0 || flags != "0" ||
            has_min_value(primitive_elem) || has_max_value(primitive_elem) ||
            has_default_value(primitive_elem)) {
            const char* name = primitive_type_info(base);
            std::string min_value = "nullptr";
            std::string max_value = "nullptr";
            std::string default_value = "nullptr";
            if (has_min_value(primitive_elem)) {
                min_value = fmt::format("{}_{}_{}_min", funcname, primitive_elem->name, suffix);
                mprintf(
                    memf,
                    "static {} {} = {};\n",
                    cpp_type_name(base, nullptr),
                    min_value,
                    get_const_value(get_min_value(primitive_elem), nullptr)
                );
                min_value = std::string("&") + min_value;
            }
            if (has_max_value(primitive_elem)) {
                max_value = fmt::format("{}_{}_{}_max", funcname, primitive_elem->name, suffix);
                mprintf(
                    memf,
                    "static {} {} = {};\n",
                    cpp_type_name(base, nullptr),
                    max_value,
                    get_const_value(get_max_value(primitive_elem), nullptr)
                );
                max_value = std::string("&") + max_value;
            }
            if (has_default_value(primitive_elem)) {
                default_value =
                    fmt::format("{}_{}_{}_default", funcname, primitive_elem->name, suffix);
                mprintf(
                    memf,
                    "static {} {} = {};\n",
                    cpp_type_name(base, nullptr),
                    default_value,
                    get_const_value(get_default_value(primitive_elem), nullptr)
                );
                default_value = std::string("&") + default_value;
            }
            mprintf(
                memf,
                "static intercom::TypeInfo {}_type_info_{} = {{ {}.name, {}.kind, {}, {}, {}, {}.max_length, "
                "{}, {}, {}, {}.key_type, {}.element_type, {}.member_count, {}.members }};\n",
                funcname,
                suffix,
                name,
                name,
                flags,
                bit_size,
                offset,
                name,
                default_value,
                min_value,
                max_value,
                name,
                name,
                name,
                name
            );
            return fmt::format("&{}_type_info_{}", funcname, suffix);
        }
        return fmt::format("&{}", primitive_type_info(base));
    }
    return fmt::format("&::intercom::{}_type_info", base->name);
}

static std::string member_name(const ptree* obj) {
    if (auto rename = get_annotation(obj, annotation_type_ext_rename)) {
        return string_value(get_annotation_value(rename, "name"));
    }
    return obj->name;
}

void intercom::cidl::gen_cpp_type_info(
    struct memf* memf,
    const ptree* obj,
    std::string_view funcname
) {
    std::vector<const ptree*> members = compound_members(obj);
    int memberId = 0;
    int index = 0;
    int prevMax = -1;
    auto objname = cpp_type_name(obj, nullptr);
    std::string min_value_name = "nullptr";
    std::string max_value_name = "nullptr";
    std::string default_value_name = "nullptr";
    std::string type_flags;
    std::string element_name;

    get_type_flag_name(type_flags, obj);
    if (!members.empty()) {
        if (obj->kind == N_ENUM || obj->kind == N_BITMASK) {
            element_name = fmt::format("&{}", primitive_type_info(obj->element_type));
            int max_value = integer_value(obj->members->value);
            if (obj->kind == N_ENUM) {
                int min_value = integer_value(obj->members->value);
                for (auto elem : obj->members) {
                    min_value = std::min(min_value, integer_value(elem->value));
                    max_value = std::max(max_value, integer_value(elem->value));
                }
                min_value_name = fmt::format("{}_min", funcname);
                mprintf(
                    memf,
                    "static {} {} = {};\n",
                    cpp_type_name(obj->element_type, nullptr),
                    min_value_name,
                    min_value
                );
                min_value_name = std::string("&") + min_value_name;

            } else if (obj->kind == N_BITMASK) {
                max_value = 0;
                for (auto elem : obj->members) {
                    max_value |= integer_value(elem->value);
                }
            }
            max_value_name = fmt::format("{}_max", funcname);
            mprintf(
                memf,
                "static {} {} = {};\n",
                cpp_type_name(obj->element_type, nullptr),
                max_value_name,
                max_value
            );
            max_value_name = std::string("&") + max_value_name;

            default_value_name = fmt::format("{}_default", funcname);
            mprintf(
                memf,
                "static {} {} = {};\n",
                cpp_type_name(obj->element_type, nullptr),
                default_value_name,
                integer_value(get_default_value(obj))
            );
            default_value_name = std::string("&") + default_value_name;

            mprintf(
                memf, "static intercom::MemberInfo {}_members[{}] = {{\n", funcname, members.size()
            );
            for (auto it = members.begin(); it != members.end(); ++it) {
                auto name = member_name(*it);
                if (it != members.begin()) {
                    mprintf(memf, ",\n");
                }
                auto member_id = integer_value((*it)->value);
                if (obj->kind == N_BITMASK) {
                    member_id = 0;
                    auto element_value = value<uint64_t>((*it)->value);
                    while (element_value >>= 1) {
                        ++member_id;
                    }
                }
                if (member_id >= 0) {
                    mprintf(
                        memf, "{{ {}, \"{}\", 0, nullptr, nullptr, nullptr }}", member_id, name
                    );
                } else {
                    mprintf(
                        memf,
                        "{{ static_cast< ::intercom::ULong >( {} ), \"{}\", 0, nullptr, nullptr, nullptr }}",
                        member_id,
                        name
                    );
                }
            }
        } else if (obj->kind == N_STRUCT || obj->kind == N_UNION || obj->kind == N_VALUETYPE ||
                   obj->kind == N_EXCEPTION) {
            std::vector<std::string> type_info_names;
            std::vector<std::string> case_label_names;
            for (auto& member : members) {
                auto suffix = fmt::format("{}", type_info_names.size());
                type_info_names.push_back(
                    gen_element_type_info(memf, member, base_type_of(member), suffix, funcname)
                );

                size_t count = 0;
                for (auto cas = member->members; cas; cas = cas->next) {
                    count++;
                }
                if (count) {
                    auto label_name = fmt::format("{}_labels_{}", funcname, type_info_names.size());
                    mprintf(memf, "static const intercom::Long {}[] = {{ {}", label_name, count);
                    for (auto cas : member->members) {
                        if (base_type_of(cas)->kind == N_BITMASK) {
                            mprintf(memf, ", {}", get_const_value(cas->value, nullptr));
                        } else {
                            mprintf(memf, ", {}", integer_value(cas->value));
                        }
                    }
                    mprintf(memf, " }};\n");
                    case_label_names.emplace_back(label_name);
                } else {
                    case_label_names.emplace_back("intercom::MemberInfo_empty_case_labels");
                }
            }
            mprintf(
                memf, "static intercom::MemberInfo {}_members[{}] = {{\n", funcname, members.size()
            );
            bool has_key = false;
            for (auto& member : members) {
                if (is_key_member(member)) {
                    has_key = true;
                }
            }
            for (auto it = members.begin(); it != members.end(); ++it, ++index) {
                auto name = member_name(*it);
                std::string flag = get_member_flag_name(*it);
                memberId = get_member_id(*it, obj, prevMax);
                prevMax = prevMax > memberId ? prevMax : memberId;
                if ((*it)->name.compare(0, 35, "void_void_void_dummy_skipped_in_air") == 0) {
                    add_flag(flag, "intercom::dcps::xtypes::IS_AIR_DUMMY");
                }
                if (*it == obj->discriminator) {
                    add_flag(flag, "intercom::dcps::xtypes::IS_DISCRIMINATOR");
                }
                if ((*it)->flags & OPT_DEFAULT) {
                    add_flag(flag, "intercom::dcps::xtypes::IS_DEFAULT");
                }
                if (!has_key) {
                    add_flag(flag, "intercom::dcps::xtypes::IS_IMPLICIT_KEY");
                }
                if (it != members.begin()) {
                    mprintf(memf, ",\n");
                }
                mprintf(
                    memf,
                    "{{ {}, \"{}\", {}, {}, {}, ",
                    memberId,
                    name,
                    flag,
                    case_label_names[index],
                    type_info_names[index]
                );

                // TODO(idarcar):
                // if (has_default_value(*it)) {
                //     std::string value = json_value(
                //         get_default_value(*it),
                //         (*it)->type,
                //         int(intercom::cidl::JsonValueFlags::FLAG_ESCAPED)
                //     );
                //     mprintf(memf, "{}", value);
                // } else {
                mprintf(memf, "nullptr");
                // }
                mprintf(memf, " }}");
            }
        }
        mprintf(memf, " }};\n\n");
    }
    mprintf(
        memf,
        "const intercom::TypeInfo intercom::TypeTraits<{}{}>::type_info = {{\n",
        objname,
        is_bitmask(obj) ? "Bits" : ""
    );
    mprintf(
        memf,
        "\"{}\", intercom::dcps::xtypes::{}, {}, {}, 0, 0, {}, {}, {}, nullptr, {}, {}, "
        "{}{}\n}};\n\n",
        get_type_name(obj),
        get_type_kind_name(obj),
        type_flags,
        get_bit_size(obj),
        default_value_name,
        min_value_name,
        max_value_name,
        obj->kind == N_ENUM || obj->kind == N_BITMASK ? element_name : "nullptr",
        members.size(),
        members.empty() ? "nullptr" : funcname,
        members.empty() ? "" : "_members"
    );
}
