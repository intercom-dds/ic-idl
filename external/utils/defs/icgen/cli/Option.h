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

#include <iostream>
#include <memory>
#include <set>
#include <sstream>
#include <string>
#include <type_traits>
#include <typeinfo>
#include <unordered_map>
#include <utility>

#include "Cast.h"
#include "OptionImpl.h"
#include "Validate.h"

namespace intercom {
namespace cli {
class Option {
  public:
    /// Creates a new option.
    /// Each parameter should be a token for the option, e.g. "-f" and "--file".
    template <typename... Args>
    explicit Option(Args&&... args) : m_impl(new detail::OptionImpl(std::forward<Args&&>(args)...)) {}

    Option& desc(const std::string& desc) {
        m_impl->description = desc;
        return *this;
    }

    Option& required(bool required) {
        m_impl->required = required;
        return *this;
    }

    /// Specifies whether the option takes a value or not.
    /// `arg_name` sets the name for the argument that is displayed in the help menu.
    Option& value(ValueKind kind, const std::string& arg_name = "arg") {
        m_impl->kind = kind;
        m_impl->arg_name = arg_name;
        return *this;
    }

    /// Specifies whether the otion accepts multiple values at once. Only relevant for
    /// options that accept multiple values.
    /// E.g. `--include a b c`: if this is set to false, only `a` will be a value of `include`.
    /// Default is `true`.
    Option& multiple(bool multiple_values_at_once) {
        m_impl->multiple = multiple_values_at_once;
        return *this;
    }

    /// Specifies whether the option is deprecated.
    /// Deprecated options are hidden from the help menu.
    /// If a deprecated option is used, an info message will be displayed.
    Option& deprecated(bool is_deprecated) {
        m_impl->deprecated = is_deprecated;
        return *this;
    }

    template <typename... Args>
    Option& deprecated(Args... tokens) {
        m_impl->deprecated_tokens = {tokens...};
        return *this;
    }

    Option& hidden(bool hidden) {
        m_impl->hidden = hidden;
        return *this;
    }

    /// Callbacks will only be triggered if *all* options are parsed successfully.
    /// The user-specified arguments will be converted to the type inferred from
    /// parameter of the callback. If you want to get the parameters exactly as
    /// written by the user, you can use the `cli::Args` type.
    /// Multiple values are supported through the use of containers.
    template <typename C>
    Option& callback(C&& callback) {
        using param_type = typename detail::Param<C>::type;
        // generalized lambda captures are not available in C++11
        m_impl->callback = std::bind(
                [](C& fun, const Args& args) {
                    param_type value;
                    detail::lexical_cast(value, args);
                    fun(value);
                },
                std::forward<C>(callback), std::placeholders::_1);
        return *this;
    }

    /// Takes a comma-separated list of options that conflicts with this
    /// option. If the user tries to provide two options that conflict with
    /// one another, an error will be thrown.
    template <typename... Args>
    Option& conflicts(Args... conflicts) {
        m_impl->conflicts = {conflicts...};
        return *this;
    }

    /// Specifies the variable that this option will set if the option was given
    /// on the command line. This serves an alternative to `ParseResult`, where one
    /// would have to manually extract the value of each option.
    template <typename T>
    Option& var(T& var) {
        auto cb = [&var](const Args& args) { detail::lexical_cast(var, args); };
        m_impl->callback = std::move(cb);
        return *this;
    }

    template <typename T>
    Option& validate(const Validate<T>& validator) {
        // generalized lambda captures are not available in C++11.
        auto clone = validator.clone();
        m_impl->validator = [clone](const std::string& arg) { return clone.check(arg); };
        return *this;
    }

  private:
    std::shared_ptr<detail::OptionImpl> m_impl;
    friend class CommandLine;
};
}  // namespace cli
}  // namespace intercom
