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

#include <filesystem>
#include <set>
#include <string>

namespace intercom::cli {

static const size_t COUNT_LEAST_ONE = 0;

/// Interface for validating user-provided values.
/// For example, if you have a `--color` option, you may wish to
/// verify that the passed value is either `red`, `green` or `blue`.
/// That can be done using one of the built-in validators:
/// ```
///     cli::Option("-c", "--color")
///         .validate(cli::one_of("red", "green", "blue"))
/// ```
///
/// For more complex uses, you can define your own validator:
/// ```
///     struct ColorValidator : public cli::Validate<ColorValidator> {
///         bool operator()(const std::string& arg) override {
///             return arg[0] == '#' && arg.size() == 7;
///         }
///     };
///     static const ColorValidator color_validator;
///     // ...
///     cli::Option("-c", "--color")
///         .validate(color_validator);
/// ```
template <typename T>
class Validate {
  public:
    virtual ~Validate() = default;

    /// Number of arguments expected.
    /// Zero implies at least one, but more are accepted.
    virtual size_t count() const {
        return COUNT_LEAST_ONE;
    }

    virtual bool check(const std::string& arg) const = 0;

    T clone() const {
        return static_cast<const T&>(*this);
    }

  protected:
    template <typename... Args>
    T operator()(Args... args) const {
        return T(std::forward<Args>(args)...);
    }
};

namespace detail {
class OneOf : public Validate<OneOf> {
  public:
    using Validate::operator();

    template <typename... Args>
    explicit OneOf(Args... args) : m_values({args...}) {}

    bool check(const std::string& arg) const override {
        return m_values.count(arg) > 0;
    }

  private:
    std::set<std::string> m_values;
};

class FileExists : public Validate<FileExists> {
  public:
    bool check(const std::string& arg) const override {
        return std::filesystem::exists(arg) && !std::filesystem::is_directory(arg);
    }
};

class Count : public Validate<Count> {
  public:
    using Validate::operator();

    Count() = default;

    explicit Count(size_t count) : m_count(count) {}

    size_t count() const override {
        return m_count;
    }

    bool check(const std::string&) const override {
        return true;
    }

  private:
    size_t m_count = COUNT_LEAST_ONE;
};
}  // namespace detail

/// Makes sure all given values exist in the specified set of strings.
static const detail::OneOf one_of;  // NOLINT

/// Makes sure all values point to existing files.
static const detail::FileExists validate_file_exists;  // NOLINT

/// Validates the number of arguments passed to an option.
static const detail::Count validate_count;  // NOLINT

}  // namespace intercom::cli
