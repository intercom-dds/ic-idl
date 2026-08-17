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

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifndef __cplusplus
#  if defined(__has_include)
#    if __has_include(<uchar.h>)
#      include <uchar.h>
#    elif defined(__CHAR16_TYPE__)
typedef __CHAR16_TYPE__ char16_t;
#    else
typedef uint_least16_t char16_t;
#    endif
#  else
#    include <uchar.h>
#  endif
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef int32_t idl_status_t;

#define IDL_STATUS_OK ((idl_status_t)0)
#define IDL_STATUS_INVALID_ARGUMENT ((idl_status_t)1)
#define IDL_STATUS_OUT_OF_MEMORY ((idl_status_t)2)
#define IDL_STATUS_OUT_OF_BOUNDS ((idl_status_t)3)
#define IDL_STATUS_BOUND_EXCEEDED ((idl_status_t)4)
#define IDL_STATUS_MODIFIED ((idl_status_t)5)
#define IDL_STATUS_EXCEPTION ((idl_status_t)6)

/// Type descriptors and their callbacks have static lifetime.
///
/// Values are copyable, bitwise relocatable, and valid for finalization in
/// zero state. `copy_init` creates a deep copy in zero-state storage and
/// leaves that storage in zero state on failure. A null `copy_init` selects
/// bytewise copying. `fini` releases resources and returns a value to zero
/// state. A null `fini` requires no action. `compare` defines a strict total
/// order and returns a negative, zero, or positive value. Map key descriptors
/// require `compare`. Compares two values.
///
/// Returns a negative value, zero, or a positive value to define a strict
/// total order.
typedef int (*idl_compare_fn)(const void* left, const void* right);

typedef struct idl_type_t {
    /// Defines the size of one value in bytes.
    size_t size;

    /// Defines the nonzero power-of-two alignment of one value.
    size_t alignment;

    /// Creates a deep copy in zero-state destination storage.
    ///
    /// Failure leaves `destination` in zero state.
    idl_status_t (*copy_init)(void* destination, const void* source);

    /// Releases resources owned by a value.
    ///
    /// Finalization returns `value` to zero state.
    void (*fini)(void* value);

    /// Defines a strict total order for values.
    idl_compare_fn compare;
} idl_type_t;

typedef struct idl_error_t {
    const idl_type_t* type;
    void* value;
} idl_error_t;

typedef struct idl_sequence_t idl_sequence_t;
typedef struct idl_map_t idl_map_t;
typedef struct idl_map_iterator_t idl_map_iterator_t;
typedef struct idl_any_t idl_any_t;

#ifdef __cplusplus
}
#endif
