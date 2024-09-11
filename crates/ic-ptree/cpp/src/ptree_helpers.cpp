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

#include "cidl/ptree_helpers.h"

#include <array>
#include <cassert>
#include <cstring>

#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/symbols.h"
#include "utils/string_utils.h"

namespace intercom::cidl {

const ptree* base_type_of(const ptree* obj) {
    if (obj && obj->type) {
        obj = obj->type;
        while (obj->type && obj->kind == N_ALIAS) {
            obj = obj->type;
        }
    }
    return obj;
}

struct ptree* base_type_of(ptree* obj) {
    return const_cast<ptree*>(base_type_of(static_cast<const struct ptree*>(obj)));
}

struct numeric get_min_value(const ptree* node, AnnotationGetter get) {
    const ptree* min = get(node, annotation_type_min);
    const ptree* range = get(node, annotation_type_range);
    // node has @range and @max -> use the annotation deriving from node's closest type, with a
    // preference for @min
    while (node && min && range) {
        node = node->type;
        if (min != get(node, annotation_type_min)) {
            range = nullptr;
        } else if (range != get(node, annotation_type_range)) {
            min = nullptr;
        }
    }
    struct numeric num =
        min ? get_annotation_value(min, "value") : get_annotation_value(range, "min");
    // Suppress zero or negative min values for unsigned types.
    if (num.kind() != UNDEF_KIND && double_value(num) <= 0.0 && is_unsigned(node)) {
        num = num_undef;
    }
    return num;
}

bool has_min_value(const ptree* node, AnnotationGetter get) {
    return get_min_value(node, get).kind() != UNDEF_KIND;
}

struct numeric get_max_value(const ptree* node, AnnotationGetter get) {
    const ptree* max = get(node, annotation_type_max);
    const ptree* range = get(node, annotation_type_range);
    // node has @range and @max -> use the annotation deriving from node's closest type, with a
    // preference for @max
    while (node && max && range) {
        node = node->type;
        if (max != get(node, annotation_type_max)) {
            range = nullptr;
        } else if (range != get(node, annotation_type_range)) {
            max = nullptr;
        }
    }
    struct numeric num =
        max ? get_annotation_value(max, "value") : get_annotation_value(range, "max");
    return num;
}

bool has_max_value(const ptree* node, AnnotationGetter get) {
    return get_max_value(node, get).kind() != UNDEF_KIND;
}

bool has_default_value(const ptree* node) {
    const ptree* ann = get_direct_annotation(node, annotation_type_default);
    if (!ann && is_optional(node))  // Do not apply type defaults to optionals
    {
        return false;
    }
    ann = ann ? ann : get_annotation(node, annotation_type_default);
    if (ann) {
        return true;
    }
    switch (base_type_of(node)->kind) {
    case N_ENUM:
    case N_BITMASK:
        return true;
    default:
        return false;
    }
}

struct numeric get_default_value(const ptree* node) {
    struct numeric num;
    const ptree* ann = get_annotation(node, annotation_type_default);
    ann = ann ? ann : get_annotation(base_type_of(node), annotation_type_default);
    if (ann) {
        num = get_annotation_value(ann, "value");
    } else if (base_type_of(node)->kind == N_ENUM) {
        ptree* default_value = nullptr;
        ptree* zero_value = nullptr;
        ptree* min_value = nullptr;
        ptree* member = base_type_of(node)->members;
        min_value = member;
        while (member) {
            if (integer_value(member->value) == 0) {
                zero_value = member;
            }
            if (integer_value(min_value->value) > integer_value(member->value)) {
                min_value = member;
            }
            if (get_annotation(member, annotation_type_default_literal)) {
                default_value = member;
            }
            member = member->next;
        }
        if (!default_value) {
            default_value = zero_value;
        }
        if (!default_value) {
            default_value = min_value;
        }
        num.val.node(default_value);
    } else if (base_type_of(node)->kind == N_BITMASK) {
        ptree* default_value = nullptr;
        for (auto member : base_type_of(node)->members) {
            if (get_annotation(member, annotation_type_default_literal)) {
                default_value = member;
            }
        }
        if (default_value) {
            num.val.node(default_value);
        } else {
            num = base_type_of(node)->element_type->value;
        }
    }
    if (num.kind() == UNDEF_KIND && base_type_of(node)->kind == N_PRIMITIVE) {
        num = base_type_of(node)->value;
    }
    return num;
}

bool has_default_case(const ptree* node) {
    return get_default_case(node) != nullptr;
}

const ptree* get_default_case(const ptree* node) {
    for (auto cas : node->members) {
        if (cas->flags & OPT_DEFAULT) {
            return cas;
        }
    }
    return nullptr;
}

const ptree* default_union_member(const ptree* node) {
    for (auto mem : node->members) {
        for (auto cas : mem->members) {
            if (integer_value(cas->value) ==
                integer_value(get_default_value(node->discriminator))) {
                return mem;
            }
        }
    }
    return get_default_case(node);
}

bool is_local(const ptree* node) {
    if (!node) {
        return false;
    }
    if (node->flags & OPT_LOCAL) {
        return true;
    }
    return is_local(node->super);
}

bool is_autoid_hash(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_autoid);
    if (ann) {
        return integer_value(ann->members->value) == 1;
    }
    if (!node->parents.empty()) {
        return is_autoid_hash(node->parents[0]);
    }
    if (node->super) {
        return is_autoid_hash(node->super);
    }
    return false;
}

int get_member_id(const ptree* member, const ptree* context, int prev_max) {
    if (context->kind == N_UNION && member == context->discriminator) {
        return 0;
    }
    const ptree* ann = get_annotation(member, annotation_type_id);
    if (ann) {
        return integer_value(ann->members->value);
    }
    ann = get_annotation(member, annotation_type_hashid);
    if (is_autoid_hash(context) || ann) {
        if (ann) {
            struct numeric str = get_annotation_value(ann, "value");
            if (!string_value(str).empty()) {
                return static_cast<int>(member_name_hash_id(string_value(str)));
            }
        }
        return static_cast<int>(member_name_hash_id(member->name));
    }
    return prev_max + 1;
}

bool is_key_member(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_key);
    return ann ? integer_value(ann->members->value) : false;
}

bool is_shared(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_shared);
    if (!ann) {
        ann = get(node, annotation_type_external);
    }
    return ann ? integer_value(ann->members->value) : false;
}

bool is_nested(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_nested);
    const ptree* scope = node->super;
    while (scope && !ann) {
        ann = get(scope, annotation_type_default_nested);
        scope = scope->super;
    }
    return ann ? integer_value(ann->members->value) : false;
}

bool is_optional(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_optional);
    return ann ? integer_value(ann->members->value) : false;
}

bool is_must_understand(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_must_understand);
    return ann ? integer_value(ann->members->value) : false;
}

bool is_bitmask(const ptree* node) {
    return node->kind == N_BITMASK;
}

bool is_minimumtypecheck(const ptree* node, AnnotationGetter get) {
    const ptree* ann = get(node, annotation_type_ext_minimum_type_check);
    return ann;
}

static bool is_language(const std::string& value, Language lang) {
    auto lower = string_utils::to_lower_case(string_utils::trim_string(value));
    if (lower == "*") {
        return true;
    }
    switch (lang) {
    case LANG_ADA:
        return lower == "ada";
    case LANG_CS:
        return lower == "c#" || lower == "csharp";
    case LANG_CPP:
        return lower == "c++" || lower == "cpp";
    case LANG_JAVA:
        return lower == "java";
    case LANG_PYTHON:
        return lower == "python";
    case LANG_IDL:
        return lower == "idl";
    case LANG_RUST:
        return lower == "rust";
    case LANG_NONE:
    default:
        return true;
    }
}

bool is_emit(const ptree* node, Language lang) {
    if (!node || (node->flags & OPT_EMIT_CODE) == 0) {
        return false;
    }
    if (auto ann = get_direct_annotation(node, annotation_type_ext_suppress)) {
        auto value = string_value(get_annotation_value(ann, "language"));
        if (is_language(value, lang)) {
            return integer_value(get_annotation_value(ann)) == 0;
        }
    }
    return true;
}

bool is_listener(const ptree* node, AnnotationGetter get) {
    return get(node, annotation_type_ext_listener);
}

bool is_primitive(const ptree* node) {
    return node && (node->kind == N_PRIMITIVE || node == &ldouble_type);
}

bool is_rpc_service(const ptree* node, AnnotationGetter get) {
    return node && node->kind == N_INTERFACE &&
           (get(node, annotation_type_service) != nullptr ||
            get(node, annotation_type_dds_service) != nullptr);
}

bool is_anonymous(const ptree* node) {
    return node && node->name[0] == '<';
}

bool is_ignored(const ptree* ann) {
    const bool only_one_member =
        ann->members && !ann->members->next && ann->type->members && !ann->type->members->next;
    const numeric val = base_value_of(get_annotation_value(ann, "value"));
    const numeric type_val = base_value_of(get_annotation_value(ann->type, "value"));
    // heuristic: annotation can only be deactivated if node only has one, default true, boolean
    // value named "value"
    return only_one_member && val.kind() == BOOLEAN_KIND && type_val.kind() == BOOLEAN_KIND &&
           type_val.val.b() && !val.val.b();
}

std::string default_topic_name(const ptree* node) {
    const ptree* ann = get_direct_annotation(node, annotation_type_topic);
    if (ann) {
        struct numeric val = get_annotation_value(ann, "name");
        if (val.kind() != UNDEF_KIND && !string_value(val).empty()) {
            return string_value(val);
        }
    }
    return idl_scoped_name(node, nullptr);
}

int get_extensibility(const ptree* node) {
    const ptree* ann = get_annotation(node, annotation_type_extensibility);
    if (ann) {
        switch (integer_value(ann->members->value)) {
        case EXTENSIBLE_EXTENSIBILITY:
            return EXTENSIBLE_EXTENSIBILITY;
        case MUTABLE_EXTENSIBILITY:
            return MUTABLE_EXTENSIBILITY;
        case FINAL_EXTENSIBILITY:
            return FINAL_EXTENSIBILITY;
        default:
            break;
        }
    }
    if (!node->parents.empty()) {
        return get_extensibility(node->parents[0]);
    }
    return EXTENSIBLE_EXTENSIBILITY;
}

const char* get_extensibility_name(const ptree* node) {
    const std::array<const char*, 3> names = {
        "FINAL_EXTENSIBILITY", "EXTENSIBLE_EXTENSIBILITY", "MUTABLE_EXTENSIBILITY"
    };
    return names[get_extensibility(node)];
}

int get_bit_bound(const ptree* node) {
    const ptree* ann = get_annotation(node, annotation_type_bit_bound);
    return ann ? integer_value(ann->members->value) : 32;
}

bool is_wstring(const ptree* node) {
    return base_type_of(node)->kind == N_STRING && base_type_of(node)->element_type == &wchar_type;
}

bool is_decl(const ptree* node) {
    return node->flags & OPT_DECLARATION;
}

bool somehow_contains_interfaces(const ptree* obj) {
    const ptree* base = base_type_of(obj);
    switch (base->kind) {
    case N_INTERFACE:
        return true;
    case N_UNION:
        if (somehow_contains_interfaces(base->discriminator)) {
            return true;
        }
        for (base = base->members; base; base = base->next) {
            if (somehow_contains_interfaces(base)) {
                return true;
            }
        }
        break;
    case N_STRUCT:
        for (base = base->members; base; base = base->next) {
            if (somehow_contains_interfaces(base)) {
                return true;
            }
        }
        break;
    default:
        break;
    }
    return false;
}

ptree* get_direct_annotation(const ptree* node, const ptree* annot_type) {
    // will only search for directly applied annotations
    // for example @range on a member of a struct
    ptree* p;
    if (node) {
        for (p = node->annotations; p; p = p->next) {
            struct numeric platform_value = get_annotation_value(p, "platform");
            if (platform_value.kind() == STRING_KIND) {
                const auto& platform = platform_value.val.str();
                if (platform != "*" && platform != "DDS") {
                    continue;
                }
            }
            if (p->type == annot_type) {
                return p;
            }
        }
    }
    return nullptr;
}

bool is_non_serialized(const ptree* node, AnnotationGetter get) {
    auto ann = get(node, annotation_type_non_serialized);
    if (ann) {
        auto v = get_annotation_value(ann, "value");
        if (v.kind() != UNDEF_KIND) {
            return value<bool>(v);
        }
        return true;
    }
    return false;
}

ptree* get_annotation(const ptree* node, const ptree* annot_type) {
    ptree* ann = nullptr;
    for (; node && !ann; node = node->type) {
        ann = get_direct_annotation(node, annot_type);
    }
    return ann;
}

int get_bit_size(const ptree* elem) {
    int bit_size = 0;
    if (elem) {
        const ptree* base = base_type_of(elem);
        ptree* ann = get_annotation(elem, annotation_type_bit_bound);
        if (base->kind == N_SEQUENCE) {
            ann = ann ? ann : get_annotation(elem, annotation_type_ext_length_bit_bound);
        }
        if (base->kind == N_STRUCT || base->kind == N_UNION) {
            ann = ann ? ann : get_annotation(elem, annotation_type_ext_jaus_presence_vector);
        }
        if (ann) {
            bit_size = integer_value(ann->members->value);
        } else if (base->kind == N_ENUM || base->kind == N_BITMASK) {
            bit_size = get_bit_size(base->element_type);
        } else {
            bit_size = get_bit_size_of_type(base);
        }
    }
    return bit_size;
}

numeric get_annotation_value(const ptree* ann, std::string_view name) {
    for (const ptree* p : ann ? ann->members : nullptr) {
        if (name.compare(p->name) == 0) {
            return p->value;
        }
    }
    return num_undef;
}

std::string get_root_filename(const ptree* node) {
    while (node->included_from) {
        node = node->included_from;
    }
    return node->file_name;
}

const ptree* original_node(const ptree* node) {
    if (node->original) {
        return original_node(node->original);
    }
    return node;
}

bool is_signed(const ptree* node) {
    node = base_type_of(node);
    return node == &int8_type || node == &char_type || node == &wchar_type || node == &short_type ||
           node == &long_type || node == &ulonglong_type || node == &float_type ||
           node == &double_type || node == &ldouble_type;
}

bool is_unsigned(const ptree* node) {
    node = base_type_of(node);
    return node == &octet_type || node == &ushort_type || node == &ulong_type ||
           node == &ulonglong_type;
}

size_t list_len(const ptree* list) {
    size_t len = 0U;
    while (list) {
        len++;
        list = list->next;
    }
    return len;
}

size_t exception_count(const ptree* node) {
    return node->getraises.size() + node->setraises.size();
}

size_t type_dimensions(const ptree* node) {
    const ptree* base_type = base_type_of(node);
    size_t size = 0U;
    switch (base_type->kind) {
    case N_ARRAY:
        size = node->bounds.size();
        break;
    case N_SEQUENCE:
        for (const ptree* element_type = base_type;
             element_type && element_type->kind == base_type->kind;
             element_type = base_type_of(element_type->element_type)) {
            size++;
        }
        break;
    case N_MAP:     // value_type is always a pair, never a map
    case N_STRING:  // value_type is always a char, never a string
        size = 1U;
        break;
    default:
        break;
    }
    return size;
}

size_t value_len(const ptree* node) {
    const ptree* base_type = base_type_of(node);
    size_t size = 0U;
    switch (base_type->kind) {
    case N_STRING:
        size = node->value.val.str().size();
        break;
    case N_SEQUENCE:
    case N_ARRAY:
    case N_MAP:
        if (node->value.kind() == PTREE_KIND) {
            size = list_len(node->value->node()->members);
        }
        break;
    default:
        break;
    }
    return size;
}

const ptree* base_value_of(const ptree* node) {
    while (node && !node->name.empty() && node->kind == N_CONST &&
           node->value.kind() == PTREE_KIND && node->value.val.node()) {
        node = node->value.val.node();
    }
    return node;
}

numeric base_value_of(numeric value) {
    while (value.kind() == PTREE_KIND && value.val.node() && !value.val.node()->name.empty() &&
           value.val.node()->kind == N_CONST) {
        value = value.val.node()->value;
    }
    return value;
}

size_t value_dimensions(const ptree* node) {
    if (!node) {
        return 0U;
    }
    const ptree* base_type = base_type_of(node);
    node = base_value_of(node);
    if (node->value.kind() == STRING_KIND || !node->members /* emtpy {} */) {
        return 1U;
    }
    size_t depth = 0U;
    node = node->members;
    while (node) {
        depth++;
        if (base_type_of(node)->kind != base_type->kind || node->value.kind() != PTREE_KIND ||
            !node->value.val.node()) {
            break;
        }
        node = base_value_of(node->value.val.node());
        node = node->members;
    }
    return depth;
}

size_t value_dimensions(const numeric& value) {
    if (value.kind() == STRING_KIND) {
        return 1U;
    }
    if (value.kind() != PTREE_KIND) {
        return 0U;
    }
    return value_dimensions(value.val.node());
}

int get_bit_size_of_type(const ptree* node) {
    if (node == nullptr) {
        return 0;
    }
    const ptree* base = base_type_of(node);
    if (base->kind == N_ENUM || base->kind == N_BITMASK) {
        return get_bit_size_of_type(base->element_type);
    }
    if (base->kind == N_PRIMITIVE) {
        if (base == &boolean_type || base == &char_type || base == &int8_type ||
            base == &octet_type) {
            return 8;
        }
        if (base == &short_type || base == &ushort_type) {
            return 16;
        }
        if (base == &long_type || base == &ulong_type || base == &float_type ||
            base == &wchar_type) {
            return 32;
        }
        if (base == &longlong_type || base == &ulonglong_type || base == &double_type) {
            return 64;
        }
        if (base == &ldouble_type) {
            return 128;
        }
    }
    return 0;
}

std::vector<MergeTrace> rec_get_merge_traces(const ptree* node, MergeTrace trace) {
    std::vector<MergeTrace> traces = {};
    auto base = base_type_of(node);
    auto members = base->members;
    for (const ptree* elem : members) {
        trace.push_back(elem);
        traces.push_back(trace);
        trace.pop_back();
    }
    return traces;
}

std::vector<MergeTrace> get_merge_traces(const ptree* node) {
    const ptree* base = base_type_of(node);
    std::vector<MergeTrace> flat_traces{};
    for (const ptree* member : base->members) {
        flat_traces.push_back({member});
    }
    return flat_traces;
}

bool is_doc_with_placement(const ptree* annotation, int placement) {
    return annotation_type_doc != nullptr && annotation && annotation->kind == N_ANNOTATION &&
           annotation->type == annotation_type_doc &&
           value<int32_t>(get_annotation_value(annotation, "placement")) == placement;
};

bool is_pre_doc(const ptree* annotation) {
    return is_doc_with_placement(annotation, PlacementKind::BEFORE_DECLARATION) ||
           is_doc_with_placement(annotation, PlacementKind::BEGIN_FILE);
}

bool is_post_doc(const ptree* annotation) {
    return is_doc_with_placement(annotation, PlacementKind::AFTER_DECLARATION);
}

}  // namespace intercom::cidl
