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

#include <cctype>
#include <filesystem>
#include <sstream>
#include <string>
#include <type_traits>

#include "Error.h"
#include "OptionImpl.h"
#include "Utils.h"

namespace intercom::cli {
namespace detail {

template <typename T>
const char* type_name() {
    if (std::is_same<T, bool>::value) {
        return "boolean";
    }
    if (std::is_integral<T>::value) {
        return "integer";
    }
    if (std::is_floating_point<T>::value) {
        return "float";
    }
    return "string";
}

template <typename T>
inline T convert_to(const std::string& value) {
    T ret_val;
    std::stringstream ss(value);

    if ((ss >> ret_val).fail()) {
        std::stringstream err;
        err << "invalid value: expected " << type_name<T>();
        err << ", found '" << value << '\'';
        throw intercom::cli::InvalidValueException(err.str());
    }
    return ret_val;
}

template <>
inline std::string convert_to(const std::string& value) {
    return value;
}

template <>
inline std::filesystem::path convert_to(const std::string& value) {
    return value;
}

template <>
inline const char* convert_to(const std::string& value) {
    return value.c_str();
}

template <>
inline char16_t convert_to(const std::string& value) {
    return convert_to<int16_t>(value);
}

template <>
inline char32_t convert_to(const std::string& value) {
    return convert_to<int32_t>(value);
}

template <>
inline bool convert_to(const std::string& value) {
    std::string val = to_lower(value);
    if (val == "yes" || val == "y" || val == "1" || val == "true") {
        return true;
    }
    if (val == "no" || val == "n" || val == "0" || val == "false") {
        return false;
    }
    throw intercom::cli::InvalidValueException("invalid value '" + value + '\'');
}

template <>
inline char convert_to(const std::string& value) {
    if (value.length() > 1) {
        throw intercom::cli::InvalidValueException("invalid value '" + value + '\'');
    }
    return value[0];
}

template <bool T, typename U, typename S>
using conditional_t = typename std::conditional<T, U, S>::type;

template <typename... Args>
struct make_void {
    using type = void;
};

template <typename... Args>
using void_t = typename make_void<Args...>::type;

template <typename T, typename = void>
struct is_container : std::false_type {};

// SFINAE: check if the templated type is a container
template <typename T>
struct is_container<
    T,
    conditional_t<
        false,
        void_t<
            typename T::value_type,
            typename T::iterator,
            decltype(std::declval<T>().end()),
            decltype(std::declval<T>().clear()),
            decltype(std::declval<T>().insert(
                std::declval<typename T::iterator>(),
                std::declval<const typename T::value_type&>()
            ))>,
        void>> : public std::true_type {};

template <
    typename Container,
    typename std::enable_if<is_container<Container>::value, bool>::type = true>
inline void lexical_cast(Container& source, const intercom::cli::Args& values) {
    source.clear();
    for (const auto& val : values) {
        source.insert(source.end(), convert_to<typename Container::value_type>(val));
    }
}

template <typename T, typename std::enable_if<!is_container<T>::value, bool>::type = true>
inline void lexical_cast(T& source, const intercom::cli::Args& values) {
    source = convert_to<T>(values.back());
}

// while strings are containers, they shouldn't be treated as such
template <typename T>
inline void lexical_cast(std::basic_string<T>& source, const intercom::cli::Args& values) {
    source = values.back();
}

template <typename T>
inline void lexical_cast(std::optional<T>& source, const intercom::cli::Args& values) {
    source = convert_to<T>(values.back());
}

template <>
inline void lexical_cast(bool& source, const intercom::cli::Args& values) {
    source = values.empty() ? true : convert_to<bool>(values.back());
}

template <typename T>
struct Param : public Param<decltype(&T::operator())> {};

template <typename C, typename T>
struct Param<void (C::*)(T) const> {
    using type = typename std::remove_const_t<std::remove_reference_t<T>>;
};
}  // namespace detail

/// Negates the given value. Useful for flags that disable things.
template <typename T>
Callback neg(T& source) {
    return [&](const cli::Args& args) {
        detail::lexical_cast(source, args);
        source = !source;
    };
}
}  // namespace intercom::cli
