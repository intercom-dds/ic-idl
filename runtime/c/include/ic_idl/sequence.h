// Copyright 2026 KONGSBERG
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

#pragma once

#include "types.h"

#ifdef __cplusplus
extern "C" {
#endif

/// Creates an empty sequence and stores its handle in `result`.
///
/// The `element_type` descriptor must have static lifetime. `SIZE_MAX` permits
/// any length. `result` is set to `NULL` on failure.
idl_status_t
idl_sequence_create(const idl_type_t* element_type, size_t maximum_length, idl_sequence_t** result);

/// Creates a deep copy of `source` and stores its handle in `result`.
///
/// `result` is set to `NULL` on failure.
idl_status_t idl_sequence_dup(const idl_sequence_t* source, idl_sequence_t** result);

/// Finalizes all elements and releases sequence storage.
///
/// A null `sequence` is accepted.
void idl_sequence_destroy(idl_sequence_t* sequence);

/// Returns the number of elements stored in `sequence`.
///
/// `sequence` must be nonnull.
size_t idl_sequence_len(const idl_sequence_t* sequence);

/// Returns the number of elements `sequence` can hold without allocation.
///
/// `sequence` must be nonnull.
size_t idl_sequence_capacity(const idl_sequence_t* sequence);

/// Returns a borrowed pointer to the first element.
///
/// An empty sequence produces `NULL`. Any mutation invalidates the returned
/// pointer.
const void* idl_sequence_data(const idl_sequence_t* sequence);

/// Returns a mutable borrowed pointer to the first element.
///
/// An empty sequence produces `NULL`. Any mutation invalidates the returned
/// pointer.
void* idl_sequence_data_mut(idl_sequence_t* sequence);

/// Returns a borrowed pointer to the element at `index`.
///
/// An out-of-bounds `index` produces `NULL`. Any mutation invalidates the
/// returned pointer.
const void* idl_sequence_at(const idl_sequence_t* sequence, size_t index);

/// Returns a mutable borrowed pointer to the element at `index`.
///
/// An out-of-bounds `index` produces `NULL`. Any mutation invalidates the
/// returned pointer.
void* idl_sequence_at_mut(idl_sequence_t* sequence, size_t index);

/// Ensures capacity for at least `capacity` elements.
///
/// A `capacity` above the declared maximum returns
/// `IDL_STATUS_BOUND_EXCEEDED`.
idl_status_t idl_sequence_reserve(idl_sequence_t* sequence, size_t capacity);

/// Appends `value` to `sequence`.
///
/// Success transfers ownership to `sequence` and sets `value` to zero state.
idl_status_t idl_sequence_push(idl_sequence_t* sequence, void* value);

/// Inserts `value` at `index` and shifts following elements toward the end.
///
/// An `index` from zero through the current length is accepted. Success transfers
/// ownership to `sequence` and sets `value` to zero state.
idl_status_t idl_sequence_insert(idl_sequence_t* sequence, size_t index, void* value);

/// Replaces the element at `index` and finalizes the previous element.
///
/// Success transfers ownership to `sequence` and sets `value` to zero state.
idl_status_t idl_sequence_set(idl_sequence_t* sequence, size_t index, void* value);

/// Moves the element at `index` into zero-state `result` storage.
///
/// Following elements shift toward the start. Failure leaves `result` unchanged.
idl_status_t idl_sequence_take(idl_sequence_t* sequence, size_t index, void* result);

/// Finalizes and removes the element at `index`.
///
/// Following elements shift toward the start.
idl_status_t idl_sequence_remove(idl_sequence_t* sequence, size_t index);

/// Finalizes all elements and sets `sequence` length to zero.
///
/// Allocated capacity may be retained.
void idl_sequence_clear(idl_sequence_t* sequence);

#ifdef __cplusplus
}
#endif
