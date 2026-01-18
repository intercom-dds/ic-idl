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

pub mod ann_template;
pub mod bit_bound;
pub mod conflicting_annotations;
pub mod default_type_mismatch;
pub mod derived_struct_key;
pub mod duplicate_annotations_hir;
pub mod duplicate_bounds;
pub mod duplicate_case_labels;
pub mod duplicate_enum_values;
pub mod duplicate_name;
pub mod exception_member;
pub mod exhaustive_union_default;
pub mod initializer_list_size;
pub mod invalid_annotation_target;
pub mod invalid_enum_literal;
pub mod invalid_enum_value;
pub mod invalid_inheritance;
pub mod keywords;
pub mod multiple_default_cases;
pub mod oneway;
pub mod recursive_type;
pub mod redundant_inheritance;
pub mod union_case_label_range;
pub mod union_case_type_mismatch;
pub mod union_key;
pub mod unreachable_union_cases;
pub mod void_ty;
pub mod zero_bound;
