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

/// Creates an empty map and stores its handle in `result`.
///
/// The descriptors must have static lifetime and `key_type` must define
/// `compare`. `SIZE_MAX` permits any length. `result` is set to `NULL` on failure.
idl_status_t idl_map_create(
    const idl_type_t* key_type,
    const idl_type_t* value_type,
    size_t maximum_length,
    idl_map_t** result
);

/// Creates a deep copy of `source` and stores its handle in `result`.
///
/// `result` is set to `NULL` on failure.
idl_status_t idl_map_dup(const idl_map_t* source, idl_map_t** result);

/// Finalizes all entries and releases map storage.
///
/// A null `map` is accepted.
void idl_map_destroy(idl_map_t* map);

/// Returns the number of entries stored in `map`.
///
/// `map` must be nonnull.
size_t idl_map_len(const idl_map_t* map);

/// Finds the entry whose key compares equal to `key`.
///
/// Success stores borrowed pointers in nonnull output arguments. Failure stores
/// `NULL` in nonnull output arguments. Any mutation invalidates borrowed
/// pointers.
bool idl_map_find(
    const idl_map_t* map,
    const void* key,
    const void** stored_key,
    const void** stored_value
);

/// Finds the entry whose key compares equal to `key`.
///
/// Success stores a borrowed key and mutable borrowed value in nonnull output
/// arguments. Failure stores `NULL` in nonnull output arguments. Any mutation
/// invalidates borrowed pointers.
bool idl_map_find_mut(
    idl_map_t* map,
    const void* key,
    const void** stored_key,
    void** stored_value
);

/// Inserts `key` and `value` or replaces the matching entry.
///
/// Success transfers ownership to `map` and sets both sources to zero state.
/// Replacement finalizes the previous key and value.
idl_status_t idl_map_put(idl_map_t* map, void* key, void* value);

/// Finalizes and removes the entry matching `key`.
///
/// Returns `true` when an entry is removed.
bool idl_map_remove(idl_map_t* map, const void* key);

/// Moves the matching entry into zero-state output storage.
///
/// `stored_key` and `stored_value` must be in zero state. Returns `true` on
/// success. Failure leaves both output values unchanged.
bool idl_map_take(idl_map_t* map, const void* key, void* stored_key, void* stored_value);

/// Finalizes all entries and leaves `map` empty.
///
/// All borrowed pointers and iterators are invalidated.
void idl_map_clear(idl_map_t* map);

/// Creates an iterator over `map` in comparator order and stores it in `result`.
///
/// `result` is set to `NULL` on failure. `map` must outlive the iterator.
idl_status_t idl_map_iterator_create(const idl_map_t* map, idl_map_iterator_t** result);

/// Returns the next borrowed key and value through the output arguments.
///
/// An entry sets `has_entry` to `true`. Exhaustion sets `key` and `value` to
/// `NULL` and `has_entry` to `false`. Mutation after iterator creation returns
/// `IDL_STATUS_MODIFIED`.
idl_status_t idl_map_iterator_next(
    idl_map_iterator_t* iterator,
    const void** key,
    const void** value,
    bool* has_entry
);

/// Releases iterator storage.
///
/// A null `iterator` is accepted.
void idl_map_iterator_destroy(idl_map_iterator_t* iterator);

#ifdef __cplusplus
}
#endif
