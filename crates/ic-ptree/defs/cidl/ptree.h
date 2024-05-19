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

#include <iomanip>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "cidl/constants.h"
#include "cidl/numeric.h"

/**
 * When parsing an IDL file, all information is stored in a recursive structure of
 * ptree nodes. Every node has a kind, a name and a file_name.
 *
 * The ptree nodes form a tree through the fields members (children), next (siblings)
 * and scope (parent). A function that loops over all the next nodes and recurses on
 * members will walk an entire (sub)tree.
 *
 * By including the type filed, the ptree nodes are a directed cyclic graph where e.g. the
 * type of a member may refer to a node earlier in the tree.
 *
 * kind: The node kind, see node_kind enum for legal values
 * name: The node name. Not NULL. Points into a set of names with no duplicates, so if
 *       strcmp(n1->name, n2->name) == 0, then n1->name == n2->name
 * next: Next node on the same level (e.g. next member in a struct or next element in a module).
 * scope: The lexical scope of a node.
 * type: The type for of a member, alias, const, case and prototype.
 *       Not null except for prototypes with void return value.
 * element_type: The element type for a string, sequence, map or array.
 * key_type: The key type of a map.
 * discriminator: The discriminator of a union
 * members: A list of all members in a module, enum, bitset, bitmask, struct, union or valuetype
 * annotations: All annotations applied to node
 * parents: A NULL-terminated array of parents. Not NULL. (For nodes with no parents, *parents ==
 * NULL) getraises: A NULL-terminated array of exceptions. Not NULL. setraises: A NULL-terminated
 * array of exceptions. Not NULL. included_from: Node of include file. NULL for nodes from the main
 * IDL file. bounds: An array of bounds, not NULL. The first value contains the number of bounds.
 *         For nodes with no bounds, bounds == {0}. For e.g. a string<13>, bounds == {1, 13}
 *         and for long x[3][4], bounds == {2, 3, 4}.
 * flags: A bitmask of flags applied to the node
 * file_name: The name of the IDL source file
 * pos, pos_end: The position (line, column) of a ptree node in the IDL file
 * value: The value of a const node (from a const expression or a member of an enum or bitmask)
 */
struct ptree {
    ptree() {
        pos.column = pos.line = 0;
        pos_end.column = pos_end.line = 0;
        value = num_undef;
    }

    struct iterator {
        using iterator_category = std::forward_iterator_tag;
        using difference_type = std::ptrdiff_t;
        using value_type = ptree*;
        using pointer = value_type*;
        using reference = value_type&;

        explicit iterator(value_type node) : node(node) {}
        iterator(const iterator&) = default;
        iterator& operator=(const iterator&) = default;

        iterator& operator++() {
            node = node->next;
            return *this;
        }
        iterator operator++(int) {
            iterator prev = *this;
            ++(*this);
            return prev;
        }
        bool operator==(const iterator& other) const {
            return node == other.node;
        }
        bool operator!=(const iterator& other) const {
            return node != other.node;
        }
        value_type operator*() const {
            return node;
        }

      private:
        value_type node;
    };

    struct const_iterator {
        using iterator_category = std::forward_iterator_tag;
        using difference_type = std::ptrdiff_t;
        using value_type = const ptree*;
        using pointer = value_type*;
        using reference = value_type&;

        explicit const_iterator(value_type node) : node(node) {}
        const_iterator(const const_iterator&) = default;
        const_iterator& operator=(const const_iterator&) = default;

        const_iterator& operator++() {
            node = node->next;
            return *this;
        }
        const_iterator operator++(int) {
            const_iterator prev = *this;
            ++(*this);
            return prev;
        }
        bool operator==(const const_iterator& other) const {
            return node == other.node;
        }
        bool operator!=(const const_iterator& other) const {
            return node != other.node;
        }
        value_type operator*() const {
            return node;
        }

      private:
        value_type node;
    };

    node_kind kind{N_UNDEF};
    std::string name;
    ptree* next{nullptr};
    ptree* super{nullptr};
    ptree* scope{nullptr};
    ptree* type{nullptr};
    ptree* element_type{nullptr};
    ptree* key_type{nullptr};
    ptree* discriminator{nullptr};
    ptree* members{nullptr};
    ptree* annotations{nullptr};
    ptree* generated{nullptr};
    ptree* original_members{nullptr};  //!< before merging \@merge
    const ptree* original{nullptr};
    std::vector<ptree*> parents;
    std::vector<ptree*> getraises;
    std::vector<ptree*> setraises;
    ptree* included_from{nullptr};
    std::vector<numeric> bounds;
    unsigned int flags{0};
    std::string file_name;
    position pos;
    position pos_end;
    numeric value;
    parser* state;
};

struct declarator {
    declarator() {
        ident.name = "";
        ident.pos.line = 0;
        ident.pos.column = 0;
        annotations = nullptr;
        next = nullptr;
    }
    struct identifier ident;
    std::vector<numeric> bounds;
    struct ptree* annotations;
    struct declarator* next;
};

inline bool operator<(struct position p1, struct position p2) {
    return p1.line < p2.line || (p1.line == p2.line && p1.column < p2.column);
}
inline bool operator>(struct position p1, struct position p2) {
    return p2 < p1;
}

inline ptree::iterator begin(ptree* node) {
    return ptree::iterator(node);
}
inline ptree::iterator end(ptree*) {
    return ptree::iterator(nullptr);
}
inline ptree::const_iterator begin(const ptree* node) {
    return ptree::const_iterator(node);
}
inline ptree::const_iterator end(const ptree*) {
    return ptree::const_iterator(nullptr);
}

template <typename T>
T value_impl(const numeric& v) {
    switch (v.kind()) {
    case UNDEF_KIND:
        return 0;
    case BOOLEAN_KIND:
        return static_cast<T>(v.val.b());
    case INT8_KIND:
        return static_cast<T>(v.val.i8());
    case OCTET_KIND:
        return static_cast<T>(v.val.o());
    case SHORT_KIND:
        return static_cast<T>(v.val.s());
    case USHORT_KIND:
        return static_cast<T>(v.val.us());
    case LONG_KIND:
        return static_cast<T>(v.val.l());
    case ULONG_KIND:
        return static_cast<T>(v.val.ul());
    case LONGLONG_KIND:
        return static_cast<T>(v.val.ll());
    case ULONGLONG_KIND:
        return static_cast<T>(v.val.ull());
    case FLOAT_KIND:
        return static_cast<T>(v.val.f());
    case DOUBLE_KIND:
        return static_cast<T>(v.val.d());
    case CHAR_KIND:
        return static_cast<T>(v.val.c());
    case STRING_KIND:
        return static_cast<T>(strtoll(v.val.str().c_str(), nullptr, 0));
    case PTREE_KIND:
        return value_impl<T>(v.val.node()->value);
    }
    return T();
}

template <typename T>
T value(const numeric& v) {
    return value_impl<T>(v);
}

template <>
inline uint8_t value(const numeric& v) {
    switch (v.kind()) {
    case FLOAT_KIND:
        return static_cast<uint8_t>(static_cast<int8_t>(v.val.f()));
    case DOUBLE_KIND:
        return static_cast<uint8_t>(static_cast<int8_t>(v.val.d()));
    default:
        return value_impl<uint8_t>(v);
    }
}

template <>
inline uint16_t value(const numeric& v) {
    switch (v.kind()) {
    case FLOAT_KIND:
        return static_cast<uint16_t>(static_cast<int16_t>(v.val.f()));
    case DOUBLE_KIND:
        return static_cast<uint16_t>(static_cast<int16_t>(v.val.d()));
    default:
        return value_impl<uint16_t>(v);
    }
}

template <>
inline uint32_t value(const numeric& v) {
    switch (v.kind()) {
    case FLOAT_KIND:
        return static_cast<uint32_t>(static_cast<int32_t>(v.val.f()));
    case DOUBLE_KIND:
        return static_cast<uint32_t>(static_cast<int32_t>(v.val.d()));
    default:
        return value_impl<uint32_t>(v);
    }
}

template <>
inline uint64_t value(const numeric& v) {
    switch (v.kind()) {
    case FLOAT_KIND:
        return static_cast<uint64_t>(static_cast<int64_t>(v.val.f()));
    case DOUBLE_KIND:
        return static_cast<uint64_t>(static_cast<int64_t>(v.val.d()));
    default:
        return value_impl<uint64_t>(v);
    }
}

template <>
inline bool value(const numeric& v) {
    switch (v.kind()) {
    case UNDEF_KIND:
        return false;
    case BOOLEAN_KIND:
        return v.val.b() != 0;
    case INT8_KIND:
        return v.val.i8() != 0;
    case OCTET_KIND:
        return v.val.o() != 0;
    case SHORT_KIND:
        return v.val.s() != 0;
    case USHORT_KIND:
        return v.val.us() != 0;
    case LONG_KIND:
        return v.val.l() != 0;
    case ULONG_KIND:
        return v.val.ul() != 0;
    case LONGLONG_KIND:
        return v.val.ll() != 0;
    case ULONGLONG_KIND:
        return v.val.ull() != 0;
    case FLOAT_KIND:
        return v.val.f() != 0;
    case DOUBLE_KIND:
        return v.val.d() != 0;
    case CHAR_KIND:
        return v.val.c() != 0;
    case STRING_KIND:
        return !v.val.str().empty();
    case PTREE_KIND:
        return v.val.node() != nullptr;
    }
    return false;
}

template <>
inline std::string value(const numeric& v) {
    std::stringstream out;
    if (v.base == 16) {
        switch (v.kind()) {
        case OCTET_KIND:
        case USHORT_KIND:
        case ULONG_KIND:
        case ULONGLONG_KIND:
            out << std::nouppercase << std::hex << std::setfill('0') << "0x";
            break;
        case INT8_KIND:
        case SHORT_KIND:
        case LONG_KIND:
        case LONGLONG_KIND:
            if (value<int64_t>(v) >= 0) {
                out << std::nouppercase << std::hex << std::setfill('0') << "0x";
            }
            break;
        default:
            break;
        }
    }
    switch (v.kind()) {
    case UNDEF_KIND:
        break;
    case BOOLEAN_KIND:
        out << (v.val.b() ? "true" : "false");
        break;
    case INT8_KIND:
        out << static_cast<short>(v.val.i8());
        break;
    case OCTET_KIND:
        out << static_cast<short>(v.val.o());
        break;
    case SHORT_KIND:
        out << v.val.s();
        break;
    case USHORT_KIND:
        out << v.val.us();
        break;
    case LONG_KIND:
        out << v.val.l();
        break;
    case ULONG_KIND:
        out << v.val.ul();
        break;
    case LONGLONG_KIND:
        out << v.val.ll();
        break;
    case ULONGLONG_KIND:
        out << v.val.ull();
        break;
    case FLOAT_KIND:
        out << v.val.f();
        break;
    case DOUBLE_KIND:
        out << v.val.d();
        break;
    case CHAR_KIND:
        out.put(static_cast<char>(v.val.c()));
        break;
    case STRING_KIND:
        out << v.val.str();
        break;
    case PTREE_KIND:
        return string_value(v.val.node()->value);
    }
    return out.str();
}
