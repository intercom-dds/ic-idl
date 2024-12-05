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

#include <string_view>

#include "InterCOM/dyn_link.h"
#include "cidl/ptree.h"
#include "cidl/symbols.h"

namespace intercom::cidl {

using AnnotationGetter = ptree* (*)(const ptree*, const ptree*);

INTERCOM_PUBLIC ptree* get_annotation(const ptree* node, const ptree* annot_type);
INTERCOM_PUBLIC ptree* get_direct_annotation(const ptree* node, const ptree* annot_type);
INTERCOM_PUBLIC int is_local(const ptree* node);
INTERCOM_PUBLIC int is_autoid_hash(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int get_member_id(const ptree* member, const ptree* context, int prevMax);
INTERCOM_PUBLIC int is_key_member(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_shared(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_nested(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_optional(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_merged(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_must_understand(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_bitmask(const ptree* node);
INTERCOM_PUBLIC int is_minimumtypecheck(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_emit(const ptree* node, Language lang);
INTERCOM_PUBLIC int is_listener(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_primitive(const ptree* node);
INTERCOM_PUBLIC int is_rpc_service(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_anonymous(const ptree* node);
INTERCOM_PUBLIC bool is_non_serialized(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC int is_ignored(const ptree* ann);
INTERCOM_PUBLIC int get_extensibility(const ptree* node);
INTERCOM_PUBLIC const char* get_extensibility_name(const ptree* node);
INTERCOM_PUBLIC bool is_wstring(const ptree* node);
INTERCOM_PUBLIC bool is_decl(const ptree* node);
INTERCOM_PUBLIC int get_bit_bound(const ptree* node);
INTERCOM_PUBLIC bool has_default_value(const ptree* node);
INTERCOM_PUBLIC numeric get_default_value(const ptree* node);
INTERCOM_PUBLIC bool has_default_case(const ptree* node);
INTERCOM_PUBLIC const ptree* get_default_case(const ptree* node);
INTERCOM_PUBLIC const ptree* default_union_member(const ptree* node);
INTERCOM_PUBLIC bool has_min_value(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC numeric get_min_value(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC bool has_max_value(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC numeric get_max_value(const ptree* node, AnnotationGetter get = get_annotation);
INTERCOM_PUBLIC std::string default_topic_name(const ptree* node);
INTERCOM_PUBLIC int somehow_contains_interfaces(const ptree* obj);
INTERCOM_PUBLIC const ptree* base_type_of(const ptree*);
INTERCOM_PUBLIC ptree* base_type_of(ptree*);
INTERCOM_PUBLIC numeric get_annotation_value(const ptree* ann, std::string_view name = "value");
INTERCOM_PUBLIC std::string get_root_filename(const ptree* node);
INTERCOM_PUBLIC const ptree* original_node(const ptree* node);
INTERCOM_PUBLIC bool is_signed(const ptree* node);
INTERCOM_PUBLIC bool is_unsigned(const ptree* node);
/// \returns #nodes in linked list
INTERCOM_PUBLIC size_t list_len(const ptree* list);
INTERCOM_PUBLIC size_t exception_count(const ptree* node);

INTERCOM_PUBLIC int get_bit_size(const ptree* elem);
/// returns number of bits reserved for node's type
/// \n returns 0 if unknown
INTERCOM_PUBLIC int get_bit_size_of_type(const ptree* node);

/// skips past all nested references, to the base value
/// \param node should derive from a numeric::val.node()
/// \note will return the last reference node if the base value is not a node
INTERCOM_PUBLIC const ptree* base_value_of(const ptree* node);
/// skips past all nested references, to the base value
INTERCOM_PUBLIC numeric base_value_of(numeric value);
/// returns #dimensions of node's type (e.g. sequence<sequence<int>> returns 2)
INTERCOM_PUBLIC size_t type_dimensions(const ptree* node);
/// returns #elements in node->value (e.g. #elements from \@default annotation)
INTERCOM_PUBLIC size_t value_len(const ptree* node);
/// returns #dimensions in node->value (e.g. #dimensions from \@default annotation)
INTERCOM_PUBLIC size_t value_dimensions(const ptree* node);
/// returns #dimensions in value \verbatim (e.g. #dimensions from \@default annotation)
INTERCOM_PUBLIC size_t value_dimensions(const numeric& value);

/// entire path from first \@merge to final non \@merge member
using MergeTrace = std::vector<const ptree*>;
/// \notabene every element in every trace derives from ptree->original_members, except for the last
/// element; it derives from ptree->members. This ensures that the last member will have the correct
/// 'inherited' annotations, as opposed to the original annotations of the type.
std::vector<MergeTrace> get_merge_traces(const ptree* node);

/// Returns true if node is a doc annotation with specified placement
INTERCOM_PUBLIC bool is_doc_with_placement(const ptree* annotation, int placement);
/// pre-declaration documentation
INTERCOM_PUBLIC bool is_pre_doc(const ptree* annotation);
/// post-declaration documentation
INTERCOM_PUBLIC bool is_post_doc(const ptree* annotation);

}  // namespace intercom::cidl
