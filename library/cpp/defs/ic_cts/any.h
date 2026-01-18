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

#include <any>
#include <typeinfo>
#include <utility>

namespace ic_cts {

class Any {
  public:
    Any() = default;

    template <typename T>
    Any(T&& value) : m_value(std::forward<T>(value)) {}

    Any(const Any&) = default;
    Any(Any&&) noexcept = default;
    Any& operator=(const Any&) = default;
    Any& operator=(Any&&) noexcept = default;

    template <typename T>
    Any& operator=(T&& value) {
        m_value = std::forward<T>(value);
        return *this;
    }

    void reset() noexcept {
        m_value.reset();
    }

    void swap(Any& other) noexcept {
        m_value.swap(other.m_value);
    }

    bool has_value() const noexcept {
        return m_value.has_value();
    }

    const std::type_info& type() const noexcept {
        return m_value.type();
    }

    bool operator==(const Any& other) const {
        return type() == other.type();
    }

    bool operator!=(const Any& other) const {
        return !(*this == other);
    }

    bool operator<(const Any& other) const {
        return type().hash_code() < other.type().hash_code();
    }

    bool operator>(const Any& other) const {
        return other < *this;
    }

    bool operator<=(const Any& other) const {
        return !(other < *this);
    }

    bool operator>=(const Any& other) const {
        return !(*this < other);
    }

    template <typename T>
    friend T* any_cast(Any* operand) noexcept {
        return std::any_cast<T>(&operand->m_value);
    }

    template <typename T>
    friend const T* any_cast(const Any* operand) noexcept {
        return std::any_cast<T>(&operand->m_value);
    }

    template <typename T>
    friend T any_cast(Any& operand) {
        return std::any_cast<T>(operand.m_value);
    }

    template <typename T>
    friend T any_cast(const Any& operand) {
        return std::any_cast<T>(operand.m_value);
    }

    template <typename T>
    friend T any_cast(Any&& operand) {
        return std::any_cast<T>(std::move(operand.m_value));
    }

  private:
    std::any m_value;
};

inline void swap(Any& lhs, Any& rhs) noexcept {
    lhs.swap(rhs);
}

}  // namespace ic_cts

namespace std {
template <>
struct hash<ic_cts::Any> {
    std::size_t operator()(const ic_cts::Any& a) const noexcept {
        return a.type().hash_code();
    }
};
}  // namespace std
