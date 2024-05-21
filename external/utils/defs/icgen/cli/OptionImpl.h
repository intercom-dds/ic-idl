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

#include <algorithm>
#include <filesystem>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "Utils.h"

namespace intercom::cli {

using Args = std::vector<std::string>;
using Callback = std::function<void(const Args&)>;

enum ValueKind { NONE, SINGLE, MULTIPLE };

namespace detail {

struct Command;
using CommandRef = std::shared_ptr<Command>;

struct OptionImpl;
using OptionRef = std::shared_ptr<OptionImpl>;

struct Section;
class ParsedOption;

struct Command {
    std::string name;
    std::string version;
    std::string copyright;
    std::string description;
    std::string long_desc;
    std::string after_help;
    std::string arg_name = "arg";
    std::string category;

    /// Tokens for the help menu. Any of these will short circuit the parsing.
    std::vector<std::string> help{"-h", "--help"};

    /// Whether the command accepts positional arguments.
    bool args = false;

    /// Whether the command should allow chained options, e.g. `-la` instead of `-l -a`
    bool chaining = false;

    /// Provide suggestions for similar options if an unrecognized option was given.
    bool suggestions = true;

    /// Hides all flags.
    bool hide_flags = false;

    /// Whether this command is external or not, i.e. whether its options should
    /// be resovled or not. If this is true, all options will be collected into `params`.
    bool external = false;

    /// Path of the executable.
    /// Only applicable to the parent command; this will be empty for subcommands.
    std::optional<std::filesystem::path> executable;

    /// Positional arguments
    Args params;

    /// List of subcommands reachable through this command
    std::map<std::string, CommandRef> subcommands;

    // If this is a subcommand, this points to the parent command.
    Command* parent = nullptr;

    /// Options particular to this command.
    /// Map + vector to maintain insert order.
    std::map<std::string, const OptionRef> options;
    std::vector<OptionRef> ordered_options;

    /// List of all parsed options.
    std::vector<ParsedOption> parsed;

    ///
    bool present = false;

    explicit Command(std::string&& name);
};

struct OptionImpl {
    /// Tokens for this option
    std::vector<std::string> tokens;

    /// Short description of what the option does
    std::string description;

    /// Name of the argument, if any.
    /// Will be shown in the help menu.
    std::string arg_name;

    /// The section to which this option belongs.
    std::string section;

    /// Deprecated tokens for this option
    std::vector<std::string> deprecated_tokens;

    /// Parsed values
    std::vector<std::string> values;

    /// Number of occurrences
    size_t count = 0;

    /// Combined length of all of the tokens.
    /// Used by the by formatter.
    size_t length = 0;

    /// List of tokens that this option conflicts with
    std::vector<std::string> conflicts;

    /// Specifies whether the option takes zero, one, or multiple arguments
    ValueKind kind = NONE;

    /// Whether the option is required or not
    bool required = false;

    /// Whether the option can occur multiple times on the command line without
    /// ovewriting the previous value.
    bool multiple = false;

    /// Whether the option is deprecated.
    /// Deprecated values are hidden by default and will print a warning if used
    bool deprecated = false;

    /// Hides the option in the generated help menu.
    bool hidden = false;

    /// Callback that is triggered if the option is encountered
    Callback callback = nullptr;

    std::function<bool(const std::string&)> validator;

    template <typename... Args>
    explicit OptionImpl(Args... args) {
        std::vector<std::string> local{args...};

        // We used to accept comma-separated strings before we switched to C++11.
        // It's no longer necessary thanks to variadic templates, but we still
        // support it for compatibility reasons.
        for (const auto& arg : local) {
            auto opt = split(arg);
            length += length_of(opt);
            tokens.insert(tokens.end(), opt.begin(), opt.end());
        }
    }

    void add_value(const std::string& value) {
        if (kind == SINGLE && !values.empty()) {
            values[0] = value;
        } else {
            values.push_back(value);
            count++;
        }
    }

    bool is_deprecated(const std::string& token) {
        if (deprecated) {
            return true;
        }
        auto it = std::find(deprecated_tokens.begin(), deprecated_tokens.end(), token);
        return it != deprecated_tokens.end();
    }
};

class ParsedOption {
  public:
    ParsedOption(std::string token, detail::OptionRef option)
        : m_token(std::move(token)), m_opt(std::move(option)) {}

    const std::string& token() const {
        return m_token;
    }

    const std::vector<std::string>& values() const {
        return m_opt->values;
    }

    detail::OptionRef option() const {
        return m_opt;
    }

  private:
    const std::string m_token;
    const detail::OptionRef m_opt;
};

inline Command::Command(std::string&& name) : name(std::forward<std::string>(name)) {}

}  // namespace detail
}  // namespace intercom::cli
