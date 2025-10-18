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

#pragma once

#include <algorithm>
#include <array>
#include <iterator>
#include <type_traits>

namespace ic_cts {

// NOLINTBEGIN(hicpp-explicit-conversions, readability-identifier-naming)

constexpr const std::size_t dynamic_extent = std::size_t(-1);

template <typename T>
class span {
  public:
    using element_type = T;
    using value_type = typename std::remove_cv_t<T>;
    using size_type = std::size_t;
    using difference_type = std::ptrdiff_t;
    using pointer = T*;
    using const_pointer = const T*;
    using reference = T&;
    using const_reference = const T&;
    using iterator = pointer;
    using reverse_iterator = std::reverse_iterator<iterator>;

    constexpr span() noexcept = default;

    constexpr span(const span& other) noexcept = default;

    template <typename It>
    explicit constexpr span(It first, size_type count) : m_data(first), m_size(count) {}

    template <typename It>
    constexpr span(It first, It end) noexcept : span(first, std::distance(first, end)) {}

    template <std::size_t N>
    constexpr span(T (&value)[N]) : span(value, N) {}

    template <typename U, std::size_t N>
    constexpr span(std::array<U, N>& array) noexcept : span(array.data(), N) {}

    template <typename U, std::size_t N>
    constexpr span(const std::array<U, N>& array) noexcept : span(array.data(), N) {}

    template <typename U, typename = typename std::enable_if_t<!std::is_const_v<T>, U>>
    constexpr span(U& value) noexcept : m_data(value.data()), m_size(value.size()) {}

    template <typename U, typename = typename std::enable_if_t<std::is_const_v<T>, U>>
    constexpr span(const U& value) noexcept : m_data(value.data()), m_size(value.size()) {}

    template <typename U = T, typename = typename std::enable_if_t<std::is_const_v<T>, U>>
    constexpr span(std::initializer_list<U> value) : span(value.begin(), value.size()) {}

    span& operator=(const span&) noexcept = default;

    constexpr iterator begin() const noexcept {
        return data();
    }

    constexpr iterator end() const noexcept {
        return data() + size();
    }

    constexpr reverse_iterator rbegin() const noexcept {
        return reverse_iterator(end());
    }

    constexpr reverse_iterator rend() const noexcept {
        return reverse_iterator(begin());
    }

    constexpr reference front() const {
        return m_data[0];
    }

    constexpr reference back() const {
        return m_data[size() - 1];
    }

    constexpr reference operator[](size_type idx) const {
        return m_data[idx];
    }

    constexpr pointer data() const {
        return m_data;
    }

    constexpr size_type size() const {
        return m_size;
    }

    constexpr size_type size_bytes() const {
        return size() * sizeof(element_type);
    }

    constexpr bool empty() const {
        return size() == 0;
    }

    constexpr span first(size_type count) const {
        return span(data(), count);
    }

    constexpr span last(size_type count) const {
        return span(data() + size() - count, count);
    }

    constexpr span subspan(size_type pos = 0, size_t count = ic_cts::dynamic_extent) const {
        return span(data() + pos, (std::min)(size() - pos, count));
    }

  private:
    pointer m_data{};
    size_type m_size{};
};

template <typename T>
inline bool operator==(const span<T>& lhs, const span<T>& rhs) {
    if (lhs.size() != rhs.size()) {
        return false;
    }
    if (lhs.data() == rhs.data()) {
        return true;
    }
    return std::equal(lhs.begin(), lhs.end(), rhs.begin());
}

template <typename T, typename U>
inline bool operator!=(const span<T>& lhs, const span<U>& rhs) {
    return !(lhs == rhs);
}

template <typename T, typename U>
inline bool operator<(const span<T>& lhs, const span<U>& rhs) {
    return std::lexicographical_compare(lhs.begin(), lhs.end(), rhs.begin(), rhs.end());
}

template <typename T, typename U>
inline bool operator<=(const span<T>& lhs, const span<U>& rhs) {
    return !(rhs < lhs);
}

template <typename T, typename U>
inline bool operator>(const span<T>& lhs, const span<U>& rhs) {
    return !(lhs < rhs);
}

template <typename T, typename U>
inline bool operator>=(const span<T>& lhs, const span<U>& rhs) {
    return !(lhs < rhs);
}

// NOLINTEND(hicpp-explicit-conversions, readability-identifier-naming)

}  // namespace ic_cts
