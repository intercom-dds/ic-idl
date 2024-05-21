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

#include <map>
#include <optional>
#include <set>
#include <string>
#include <utility>

#include "Cast.h"
#include "OptionImpl.h"

namespace intercom::cli {

class ParseResult {
  public:
    using iterator = std::vector<detail::ParsedOption>::const_iterator;

    explicit ParseResult(detail::CommandRef&& command) : m_cmd(std::move(command)) {}

    const std::string& name() const {
        return m_cmd->name;
    }

    /// Returns the user-provided value for the given option.
    /// If the option takes multiple values, this will only return the first value.
    template <typename T>
    std::optional<T> get(const std::string& option) const {
        auto it = m_cmd->options.find(option);
        if (it == m_cmd->options.cend()) {
            throw std::logic_error{"undefined option '" + option + '\''};
        }
        if (it->second->values.empty()) {
            return std::nullopt;
        }
        return detail::convert_to<T>(it->second->values.at(0));
    }

    /// Returns a vector of user-provided values for the given option.
    template <typename T>
    std::vector<T> get_vec(const std::string& option) const {
        std::vector<T> vec;

        auto it = m_cmd->options.find(option);
        if (it == m_cmd->options.cend()) {
            throw std::logic_error{"undefined option '" + option + '\''};
        }

        const auto& values = it->second->values;
        for (const auto& value : values) {
            vec.push_back(detail::convert_to<T>(value));
        }
        return vec;
    }

    /// Returns a set of user-provided values for the given option.
    template <typename T>
    std::set<T> get_set(const std::string& option) const {
        auto it = m_cmd->options.find(option);
        if (it == m_cmd->options.cend()) {
            throw std::logic_error{"undefined option '" + option + '\''};
        }

        std::set<T> set;
        for (const auto& val : it->second->values) {
            set.insert(detail::convert_to<T>(val));
        }
        return set;
    }

    /// Returns true if the value was provided by the user, otherwise false.
    template <typename T>
    bool get(const std::string& option, T& value) const {
        auto it = m_cmd->options.find(option);
        if (it == m_cmd->options.cend() || it->second->count == 0) {
            return false;
        }
        if (it->second->kind == NONE) {
            return true;
        }
        try {
            value = detail::convert_to<T>(it->second->values.at(0));
            return true;
        } catch (...) {
            return false;
        }
    }

    /// Returns the number of occurrences of this option.
    /// For options that take values, this will return the number of values provided.
    size_t count(const std::string& option) const {
        auto it = m_cmd->options.find(option);
        return it == m_cmd->options.cend() ? 0 : it->second->count;
    }

    bool is_present(const std::string& option) const {
        auto it = m_cmd->options.find(option);
        if (it == m_cmd->options.cend()) {
            return false;
        }
        return it->second->count > 0;
    }

    /// Returns a const iterator of the parsed values.
    /// Note that the iterator may contain duplicate entries for each option.
    /// If option '-a' is a flag that was repeated multiple times on the command line,
    /// it will have multiple entries in the iterator.
    iterator begin() const {
        return m_cmd->parsed.begin();
    }

    iterator end() const {
        return m_cmd->parsed.end();
    }

    iterator find(const std::string& option) const {
        for (auto it = begin(); it != end(); ++it) {
            for (const auto& token : it->option()->tokens) {
                if (token == option) {
                    return it;
                }
            }
        }
        return end();
    }

    bool empty() const {
        return size() == 0;
    }

    size_t size() const {
        return m_cmd->parsed.size();
    }

    const std::vector<std::string>& positionals() const {
        return m_cmd->params;
    }

    std::optional<ParseResult> subcommand() const {
        for (const auto& cmd : m_cmd->subcommands) {
            if (cmd.second->present) {
                auto copy = cmd.second;
                return ParseResult{std::move(copy)};
            }
        }
        return std::nullopt;
    }

    detail::CommandRef impl() const {
        return m_cmd;
    }

  private:
    detail::CommandRef m_cmd;
};

}  // namespace intercom::cli
