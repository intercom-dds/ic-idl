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

#include <stdexcept>

#include "cidl/constants.h"

namespace intercom {

template <typename T, typename... Args>
constexpr T* construct_at(T* ptr, Args&&... args) {
    return ::new (const_cast<void*>(static_cast<const volatile void*>(ptr)))
        T(std::forward<Args>(args)...);
}

template <typename T>
void destroy_at(T* ptr) {
    ptr->T::~T();
}

namespace cidl {

struct numeric_storage {
    numeric_storage();
    numeric_storage(const numeric_storage& a_other);
    numeric_storage& operator=(const numeric_storage& a_other);
    numeric_storage(numeric_storage&& a_other) noexcept;
    numeric_storage& operator=(numeric_storage&& a_other) noexcept;
    ~numeric_storage() noexcept;

    bool operator<(const numeric_storage& a_other) const;
    bool operator==(const numeric_storage& a_other) const;
    bool operator!=(const numeric_storage& a_other) const {
        return !(*this == a_other);
    }
    bool operator>(const numeric_storage& a_other) const {
        return a_other < *this;
    }
    bool operator<=(const numeric_storage& a_other) const {
        return !(a_other < *this);
    }
    bool operator>=(const numeric_storage& a_other) const {
        return !(*this < a_other);
    }

    friend void swap(numeric_storage& a_first, numeric_storage& a_second) noexcept;

    ::numeric_kind _d() const {
        return m_ic_discriminator_value;
    }
    void _d(::numeric_kind discriminator);

    int& b();
    int b() const;
    void b(int value);

    int8_t& i8();
    int8_t i8() const;
    void i8(int8_t value);

    uint8_t& o();
    uint8_t o() const;
    void o(uint8_t value);

    int16_t& s();
    int16_t s() const;
    void s(int16_t value);

    uint16_t& us();
    uint16_t us() const;
    void us(uint16_t value);

    int32_t& l();
    int32_t l() const;
    void l(int32_t value);

    uint32_t& ul();
    uint32_t ul() const;
    void ul(uint32_t value);

    int64_t& ll();
    int64_t ll() const;
    void ll(int64_t value);

    uint64_t& ull();
    uint64_t ull() const;
    void ull(uint64_t value);

    float& f();
    float f() const;
    void f(float value);

    double& d();
    double d() const;
    void d(double value);

    int& c();
    int c() const;
    void c(int value);

    ::std::string& str();
    const ::std::string& str() const;
    void str(const ::std::string& value);

    const ptree*& node();
    const ptree* node() const;
    void node(const ptree* value);

  private:
    union ICUnionType_ {
        ICUnionType_() {}
        ~ICUnionType_() {}
        int b;
        int8_t i8;
        uint8_t o;
        int16_t s;
        uint16_t us;
        int32_t l;
        uint32_t ul;
        int64_t ll;
        uint64_t ull;
        float f;
        double d;
        int c;
        ::std::string str;
        const ptree* node = nullptr;
    } m_ic_union_value;
    ::numeric_kind m_ic_discriminator_value;
    void free_union_();
};

inline numeric_storage::numeric_storage() {
    m_ic_discriminator_value = ::UNDEF_KIND;
}

inline numeric_storage::numeric_storage(const numeric_storage& a_other) {
    m_ic_discriminator_value = a_other.m_ic_discriminator_value;
    switch (m_ic_discriminator_value) {
    case ::BOOLEAN_KIND:
        m_ic_union_value.b = a_other.m_ic_union_value.b;
        break;
    case ::INT8_KIND:
        m_ic_union_value.i8 = a_other.m_ic_union_value.i8;
        break;
    case ::OCTET_KIND:
        m_ic_union_value.o = a_other.m_ic_union_value.o;
        break;
    case ::SHORT_KIND:
        m_ic_union_value.s = a_other.m_ic_union_value.s;
        break;
    case ::USHORT_KIND:
        m_ic_union_value.us = a_other.m_ic_union_value.us;
        break;
    case ::LONG_KIND:
        m_ic_union_value.l = a_other.m_ic_union_value.l;
        break;
    case ::ULONG_KIND:
        m_ic_union_value.ul = a_other.m_ic_union_value.ul;
        break;
    case ::LONGLONG_KIND:
        m_ic_union_value.ll = a_other.m_ic_union_value.ll;
        break;
    case ::ULONGLONG_KIND:
        m_ic_union_value.ull = a_other.m_ic_union_value.ull;
        break;
    case ::FLOAT_KIND:
        m_ic_union_value.f = a_other.m_ic_union_value.f;
        break;
    case ::DOUBLE_KIND:
        m_ic_union_value.d = a_other.m_ic_union_value.d;
        break;
    case ::CHAR_KIND:
        m_ic_union_value.c = a_other.m_ic_union_value.c;
        break;
    case ::STRING_KIND:
        intercom::construct_at(&m_ic_union_value.str, a_other.m_ic_union_value.str);
        break;
    case ::PTREE_KIND:
        m_ic_union_value.node = a_other.m_ic_union_value.node;
        break;
    case ::UNDEF_KIND:
        break;
    }
}

inline numeric_storage& numeric_storage::operator=(const numeric_storage& a_other) {
    if (this != &a_other) {
        _d(a_other.m_ic_discriminator_value);
        switch (m_ic_discriminator_value) {
        case ::BOOLEAN_KIND:
            m_ic_union_value.b = a_other.m_ic_union_value.b;
            break;
        case ::INT8_KIND:
            m_ic_union_value.i8 = a_other.m_ic_union_value.i8;
            break;
        case ::OCTET_KIND:
            m_ic_union_value.o = a_other.m_ic_union_value.o;
            break;
        case ::SHORT_KIND:
            m_ic_union_value.s = a_other.m_ic_union_value.s;
            break;
        case ::USHORT_KIND:
            m_ic_union_value.us = a_other.m_ic_union_value.us;
            break;
        case ::LONG_KIND:
            m_ic_union_value.l = a_other.m_ic_union_value.l;
            break;
        case ::ULONG_KIND:
            m_ic_union_value.ul = a_other.m_ic_union_value.ul;
            break;
        case ::LONGLONG_KIND:
            m_ic_union_value.ll = a_other.m_ic_union_value.ll;
            break;
        case ::ULONGLONG_KIND:
            m_ic_union_value.ull = a_other.m_ic_union_value.ull;
            break;
        case ::FLOAT_KIND:
            m_ic_union_value.f = a_other.m_ic_union_value.f;
            break;
        case ::DOUBLE_KIND:
            m_ic_union_value.d = a_other.m_ic_union_value.d;
            break;
        case ::CHAR_KIND:
            m_ic_union_value.c = a_other.m_ic_union_value.c;
            break;
        case ::STRING_KIND:
            m_ic_union_value.str = a_other.m_ic_union_value.str;
            break;
        case ::PTREE_KIND:
            m_ic_union_value.node = a_other.m_ic_union_value.node;
            break;
        case ::UNDEF_KIND:
            break;
        }
    }

    return *this;
}

inline numeric_storage::numeric_storage(numeric_storage&& a_other) noexcept : numeric_storage() {
    m_ic_discriminator_value = a_other.m_ic_discriminator_value;
    switch (m_ic_discriminator_value) {
    case ::BOOLEAN_KIND:
        m_ic_union_value.b = a_other.m_ic_union_value.b;
        break;
    case ::INT8_KIND:
        m_ic_union_value.i8 = a_other.m_ic_union_value.i8;
        break;
    case ::OCTET_KIND:
        m_ic_union_value.o = a_other.m_ic_union_value.o;
        break;
    case ::SHORT_KIND:
        m_ic_union_value.s = a_other.m_ic_union_value.s;
        break;
    case ::USHORT_KIND:
        m_ic_union_value.us = a_other.m_ic_union_value.us;
        break;
    case ::LONG_KIND:
        m_ic_union_value.l = a_other.m_ic_union_value.l;
        break;
    case ::ULONG_KIND:
        m_ic_union_value.ul = a_other.m_ic_union_value.ul;
        break;
    case ::LONGLONG_KIND:
        m_ic_union_value.ll = a_other.m_ic_union_value.ll;
        break;
    case ::ULONGLONG_KIND:
        m_ic_union_value.ull = a_other.m_ic_union_value.ull;
        break;
    case ::FLOAT_KIND:
        m_ic_union_value.f = a_other.m_ic_union_value.f;
        break;
    case ::DOUBLE_KIND:
        m_ic_union_value.d = a_other.m_ic_union_value.d;
        break;
    case ::CHAR_KIND:
        m_ic_union_value.c = a_other.m_ic_union_value.c;
        break;
    case ::STRING_KIND:
        intercom::construct_at(&m_ic_union_value.str, std::move(a_other.m_ic_union_value.str));
        break;
    case ::PTREE_KIND:
        m_ic_union_value.node = a_other.m_ic_union_value.node;
        break;
    case ::UNDEF_KIND:
        break;
    }
}

inline numeric_storage& numeric_storage::operator=(numeric_storage&& a_other) noexcept {
    if (this != &a_other) {
        _d(a_other.m_ic_discriminator_value);
        switch (m_ic_discriminator_value) {
        case ::BOOLEAN_KIND:
            m_ic_union_value.b = a_other.m_ic_union_value.b;
            break;
        case ::INT8_KIND:
            m_ic_union_value.i8 = a_other.m_ic_union_value.i8;
            break;
        case ::OCTET_KIND:
            m_ic_union_value.o = a_other.m_ic_union_value.o;
            break;
        case ::SHORT_KIND:
            m_ic_union_value.s = a_other.m_ic_union_value.s;
            break;
        case ::USHORT_KIND:
            m_ic_union_value.us = a_other.m_ic_union_value.us;
            break;
        case ::LONG_KIND:
            m_ic_union_value.l = a_other.m_ic_union_value.l;
            break;
        case ::ULONG_KIND:
            m_ic_union_value.ul = a_other.m_ic_union_value.ul;
            break;
        case ::LONGLONG_KIND:
            m_ic_union_value.ll = a_other.m_ic_union_value.ll;
            break;
        case ::ULONGLONG_KIND:
            m_ic_union_value.ull = a_other.m_ic_union_value.ull;
            break;
        case ::FLOAT_KIND:
            m_ic_union_value.f = a_other.m_ic_union_value.f;
            break;
        case ::DOUBLE_KIND:
            m_ic_union_value.d = a_other.m_ic_union_value.d;
            break;
        case ::CHAR_KIND:
            m_ic_union_value.c = a_other.m_ic_union_value.c;
            break;
        case ::STRING_KIND:
            m_ic_union_value.str = std::move(a_other.m_ic_union_value.str);
            break;
        case ::PTREE_KIND:
            m_ic_union_value.node = a_other.m_ic_union_value.node;
            break;
        case ::UNDEF_KIND:
            break;
        }
    }
    return *this;
}

inline numeric_storage::~numeric_storage() noexcept {
    free_union_();
}

inline bool numeric_storage::operator<(const numeric_storage& a_other) const {
    if (_d() < a_other._d()) {
        return true;
    }
    if (a_other._d() < _d()) {
        return false;
    }
    switch (_d()) {
    case ::BOOLEAN_KIND:
        return this->b() < a_other.b();
    case ::INT8_KIND:
        return this->i8() < a_other.i8();
    case ::OCTET_KIND:
        return this->o() < a_other.o();
    case ::SHORT_KIND:
        return this->s() < a_other.s();
    case ::USHORT_KIND:
        return this->us() < a_other.us();
    case ::LONG_KIND:
        return this->l() < a_other.l();
    case ::ULONG_KIND:
        return this->ul() < a_other.ul();
    case ::LONGLONG_KIND:
        return this->ll() < a_other.ll();
    case ::ULONGLONG_KIND:
        return this->ull() < a_other.ull();
    case ::FLOAT_KIND:
        return this->f() < a_other.f();
    case ::DOUBLE_KIND:
        return this->d() < a_other.d();
    case ::CHAR_KIND:
        return this->c() < a_other.c();
    case ::STRING_KIND:
        return this->str() < a_other.str();
    case ::PTREE_KIND:
        return this->node() < a_other.node();
    case ::UNDEF_KIND:
        return false;
    }
    return false;
}

inline bool numeric_storage::operator==(const numeric_storage& a_other) const {
    if (!(_d() == a_other._d())) {
        return false;
    }
    switch (_d()) {
    case ::BOOLEAN_KIND:
        return this->b() == a_other.b();
    case ::INT8_KIND:
        return this->i8() == a_other.i8();
    case ::OCTET_KIND:
        return this->o() == a_other.o();
    case ::SHORT_KIND:
        return this->s() == a_other.s();
    case ::USHORT_KIND:
        return this->us() == a_other.us();
    case ::LONG_KIND:
        return this->l() == a_other.l();
    case ::ULONG_KIND:
        return this->ul() == a_other.ul();
    case ::LONGLONG_KIND:
        return this->ll() == a_other.ll();
    case ::ULONGLONG_KIND:
        return this->ull() == a_other.ull();
    case ::FLOAT_KIND:
        return this->f() == a_other.f();
    case ::DOUBLE_KIND:
        return this->d() == a_other.d();
    case ::CHAR_KIND:
        return this->c() == a_other.c();
    case ::STRING_KIND:
        return this->str() == a_other.str();
    case ::PTREE_KIND:
        return this->node() == a_other.node();
    case ::UNDEF_KIND:
        return true;
    }
    return true;
}

inline void swap(numeric_storage& a_first, numeric_storage& a_second) noexcept {
    numeric_storage a_first_tmp = std::move(a_first);
    a_first = std::move(a_second);
    a_second = std::move(a_first_tmp);
}

inline void numeric_storage::_d(::numeric_kind discriminator) {
    switch (discriminator) {
    case ::BOOLEAN_KIND:
        if (m_ic_discriminator_value != BOOLEAN_KIND) {
            free_union_();
            m_ic_union_value.b = false;
        }
        break;
    case ::INT8_KIND:
        if (m_ic_discriminator_value != INT8_KIND) {
            free_union_();
            m_ic_union_value.i8 = 0;
        }
        break;
    case ::OCTET_KIND:
        if (m_ic_discriminator_value != OCTET_KIND) {
            free_union_();
            m_ic_union_value.o = 0U;
        }
        break;
    case ::SHORT_KIND:
        if (m_ic_discriminator_value != SHORT_KIND) {
            free_union_();
            m_ic_union_value.s = 0;
        }
        break;
    case ::USHORT_KIND:
        if (m_ic_discriminator_value != USHORT_KIND) {
            free_union_();
            m_ic_union_value.us = 0U;
        }
        break;
    case ::LONG_KIND:
        if (m_ic_discriminator_value != LONG_KIND) {
            free_union_();
            m_ic_union_value.l = 0;
        }
        break;
    case ::ULONG_KIND:
        if (m_ic_discriminator_value != ULONG_KIND) {
            free_union_();
            m_ic_union_value.ul = 0U;
        }
        break;
    case ::LONGLONG_KIND:
        if (m_ic_discriminator_value != LONGLONG_KIND) {
            free_union_();
            m_ic_union_value.ll = 0LL;
        }
        break;
    case ::ULONGLONG_KIND:
        if (m_ic_discriminator_value != ULONGLONG_KIND) {
            free_union_();
            m_ic_union_value.ull = 0ULL;
        }
        break;
    case ::FLOAT_KIND:
        if (m_ic_discriminator_value != FLOAT_KIND) {
            free_union_();
            m_ic_union_value.f = static_cast<float>(0.0000000e+00);
        }
        break;
    case ::DOUBLE_KIND:
        if (m_ic_discriminator_value != DOUBLE_KIND) {
            free_union_();
            m_ic_union_value.d = 0.0000000000000000e+00;
        }
        break;
    case ::CHAR_KIND:
        if (m_ic_discriminator_value != CHAR_KIND) {
            free_union_();
            m_ic_union_value.c = '\000';
        }
        break;
    case ::STRING_KIND:
        if (m_ic_discriminator_value != STRING_KIND) {
            free_union_();
            intercom::construct_at(&m_ic_union_value.str, ::std::string{});
        }
        break;
    case ::PTREE_KIND:
        if (m_ic_discriminator_value != PTREE_KIND) {
            free_union_();
            m_ic_union_value.node = nullptr;
        }
        break;
    case ::UNDEF_KIND:
        free_union_();
        break;
    default:
        throw std::logic_error("Illegal discriminator value for union storage");
    }
    m_ic_discriminator_value = discriminator;
}

inline int& numeric_storage::b() {
    if (m_ic_discriminator_value != BOOLEAN_KIND) {
        throw std::logic_error("Union storage not set to value b");
    }
    return m_ic_union_value.b;
}

inline int numeric_storage::b() const {
    if (m_ic_discriminator_value != BOOLEAN_KIND) {
        throw std::logic_error("Union storage not set to value b");
    }
    return m_ic_union_value.b;
}

inline void numeric_storage::b(int value) {
    if (m_ic_discriminator_value != BOOLEAN_KIND) {
        free_union_();
        m_ic_discriminator_value = BOOLEAN_KIND;
    }
    m_ic_union_value.b = value;
}

inline int8_t& numeric_storage::i8() {
    if (m_ic_discriminator_value != INT8_KIND) {
        throw std::logic_error("Union storage not set to value i8");
    }
    return m_ic_union_value.i8;
}

inline int8_t numeric_storage::i8() const {
    if (m_ic_discriminator_value != INT8_KIND) {
        throw std::logic_error("Union storage not set to value i8");
    }
    return m_ic_union_value.i8;
}

inline void numeric_storage::i8(int8_t value) {
    if (m_ic_discriminator_value != INT8_KIND) {
        free_union_();
        m_ic_discriminator_value = INT8_KIND;
    }
    m_ic_union_value.i8 = value;
}

inline uint8_t& numeric_storage::o() {
    if (m_ic_discriminator_value != OCTET_KIND) {
        throw std::logic_error("Union storage not set to value o");
    }
    return m_ic_union_value.o;
}

inline uint8_t numeric_storage::o() const {
    if (m_ic_discriminator_value != OCTET_KIND) {
        throw std::logic_error("Union storage not set to value o");
    }
    return m_ic_union_value.o;
}

inline void numeric_storage::o(uint8_t value) {
    if (m_ic_discriminator_value != OCTET_KIND) {
        free_union_();
        m_ic_discriminator_value = OCTET_KIND;
    }
    m_ic_union_value.o = value;
}

inline int16_t& numeric_storage::s() {
    if (m_ic_discriminator_value != SHORT_KIND) {
        throw std::logic_error("Union storage not set to value s");
    }
    return m_ic_union_value.s;
}

inline int16_t numeric_storage::s() const {
    if (m_ic_discriminator_value != SHORT_KIND) {
        throw std::logic_error("Union storage not set to value s");
    }
    return m_ic_union_value.s;
}

inline void numeric_storage::s(int16_t value) {
    if (m_ic_discriminator_value != SHORT_KIND) {
        free_union_();
        m_ic_discriminator_value = SHORT_KIND;
    }
    m_ic_union_value.s = value;
}

inline uint16_t& numeric_storage::us() {
    if (m_ic_discriminator_value != USHORT_KIND) {
        throw std::logic_error("Union storage not set to value us");
    }
    return m_ic_union_value.us;
}

inline uint16_t numeric_storage::us() const {
    if (m_ic_discriminator_value != USHORT_KIND) {
        throw std::logic_error("Union storage not set to value us");
    }
    return m_ic_union_value.us;
}

inline void numeric_storage::us(uint16_t value) {
    if (m_ic_discriminator_value != USHORT_KIND) {
        free_union_();
        m_ic_discriminator_value = USHORT_KIND;
    }
    m_ic_union_value.us = value;
}

inline int32_t& numeric_storage::l() {
    if (m_ic_discriminator_value != LONG_KIND) {
        throw std::logic_error("Union storage not set to value l");
    }
    return m_ic_union_value.l;
}

inline int32_t numeric_storage::l() const {
    if (m_ic_discriminator_value != LONG_KIND) {
        throw std::logic_error("Union storage not set to value l");
    }
    return m_ic_union_value.l;
}

inline void numeric_storage::l(int32_t value) {
    if (m_ic_discriminator_value != LONG_KIND) {
        free_union_();
        m_ic_discriminator_value = LONG_KIND;
    }
    m_ic_union_value.l = value;
}

inline uint32_t& numeric_storage::ul() {
    if (m_ic_discriminator_value != ULONG_KIND) {
        throw std::logic_error("Union storage not set to value ul");
    }
    return m_ic_union_value.ul;
}

inline uint32_t numeric_storage::ul() const {
    if (m_ic_discriminator_value != ULONG_KIND) {
        throw std::logic_error("Union storage not set to value ul");
    }
    return m_ic_union_value.ul;
}

inline void numeric_storage::ul(uint32_t value) {
    if (m_ic_discriminator_value != ULONG_KIND) {
        free_union_();
        m_ic_discriminator_value = ULONG_KIND;
    }
    m_ic_union_value.ul = value;
}

inline int64_t& numeric_storage::ll() {
    if (m_ic_discriminator_value != LONGLONG_KIND) {
        throw std::logic_error("Union storage not set to value ll");
    }
    return m_ic_union_value.ll;
}

inline int64_t numeric_storage::ll() const {
    if (m_ic_discriminator_value != LONGLONG_KIND) {
        throw std::logic_error("Union storage not set to value ll");
    }
    return m_ic_union_value.ll;
}

inline void numeric_storage::ll(int64_t value) {
    if (m_ic_discriminator_value != LONGLONG_KIND) {
        free_union_();
        m_ic_discriminator_value = LONGLONG_KIND;
    }
    m_ic_union_value.ll = value;
}

inline uint64_t& numeric_storage::ull() {
    if (m_ic_discriminator_value != ULONGLONG_KIND) {
        throw std::logic_error("Union storage not set to value ull");
    }
    return m_ic_union_value.ull;
}

inline uint64_t numeric_storage::ull() const {
    if (m_ic_discriminator_value != ULONGLONG_KIND) {
        throw std::logic_error("Union storage not set to value ull");
    }
    return m_ic_union_value.ull;
}

inline void numeric_storage::ull(uint64_t value) {
    if (m_ic_discriminator_value != ULONGLONG_KIND) {
        free_union_();
        m_ic_discriminator_value = ULONGLONG_KIND;
    }
    m_ic_union_value.ull = value;
}

inline float& numeric_storage::f() {
    if (m_ic_discriminator_value != FLOAT_KIND) {
        throw std::logic_error("Union storage not set to value f");
    }
    return m_ic_union_value.f;
}

inline float numeric_storage::f() const {
    if (m_ic_discriminator_value != FLOAT_KIND) {
        throw std::logic_error("Union storage not set to value f");
    }
    return m_ic_union_value.f;
}

inline void numeric_storage::f(float value) {
    if (m_ic_discriminator_value != FLOAT_KIND) {
        free_union_();
        m_ic_discriminator_value = FLOAT_KIND;
    }
    m_ic_union_value.f = value;
}

inline double& numeric_storage::d() {
    if (m_ic_discriminator_value != DOUBLE_KIND) {
        throw std::logic_error("Union storage not set to value d");
    }
    return m_ic_union_value.d;
}

inline double numeric_storage::d() const {
    if (m_ic_discriminator_value != DOUBLE_KIND) {
        throw std::logic_error("Union storage not set to value d");
    }
    return m_ic_union_value.d;
}

inline void numeric_storage::d(double value) {
    if (m_ic_discriminator_value != DOUBLE_KIND) {
        free_union_();
        m_ic_discriminator_value = DOUBLE_KIND;
    }
    m_ic_union_value.d = value;
}

inline int& numeric_storage::c() {
    if (m_ic_discriminator_value != CHAR_KIND) {
        throw std::logic_error("Union storage not set to value c");
    }
    return m_ic_union_value.c;
}

inline int numeric_storage::c() const {
    if (m_ic_discriminator_value != CHAR_KIND) {
        throw std::logic_error("Union storage not set to value c");
    }
    return m_ic_union_value.c;
}

inline void numeric_storage::c(int value) {
    if (m_ic_discriminator_value != CHAR_KIND) {
        free_union_();
        m_ic_discriminator_value = CHAR_KIND;
    }
    m_ic_union_value.c = value;
}

inline ::std::string& numeric_storage::str() {
    if (m_ic_discriminator_value != STRING_KIND) {
        throw std::logic_error("Union storage not set to value str");
    }
    return m_ic_union_value.str;
}

inline const ::std::string& numeric_storage::str() const {
    if (m_ic_discriminator_value != STRING_KIND) {
        throw std::logic_error("Union storage not set to value str");
    }
    return m_ic_union_value.str;
}

inline void numeric_storage::str(const ::std::string& value) {
    if (m_ic_discriminator_value != STRING_KIND) {
        free_union_();
        m_ic_discriminator_value = STRING_KIND;
        intercom::construct_at(&m_ic_union_value.str, value);
    } else {
        m_ic_union_value.str = value;
    }
}

inline const ptree*& numeric_storage::node() {
    if (m_ic_discriminator_value != PTREE_KIND) {
        throw std::logic_error("Union storage not set to value node");
    }
    return m_ic_union_value.node;
}

inline const ptree* numeric_storage::node() const {
    if (m_ic_discriminator_value != PTREE_KIND) {
        throw std::logic_error("Union storage not set to value node");
    }
    return m_ic_union_value.node;
}

inline void numeric_storage::node(const ptree* value) {
    if (m_ic_discriminator_value != PTREE_KIND) {
        free_union_();
        m_ic_discriminator_value = PTREE_KIND;
    }
    m_ic_union_value.node = value;
}

inline void numeric_storage::free_union_() {
    if (m_ic_discriminator_value == STRING_KIND) {
        intercom::destroy_at(&m_ic_union_value.str);
    }
}

}  // namespace cidl
}  // namespace intercom

struct numeric {
    int base = 10;
    intercom::cidl::numeric_storage val;
    const intercom::cidl::numeric_storage& operator*() const {
        return val;
    }
    const intercom::cidl::numeric_storage* operator->() const {
        return &val;
    }
    [[nodiscard]] numeric_kind kind() const {
        return val._d();
    };
    [[nodiscard]] bool has_val() const {
        return kind() != UNDEF_KIND;
    }
};
