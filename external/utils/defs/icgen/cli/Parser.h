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

#include <cstring>
#include <utility>
#include <vector>

#include "Error.h"
#include "OptionImpl.h"
#include "ParseResult.h"

namespace intercom {
namespace cli {
namespace detail {

/// Helper class for iterating through the arguments.
class ArgStream {
  public:
    explicit ArgStream(const char* const* argv) : m_argv(argv) {}

    const char* peek() const { return m_done ? nullptr : *(m_argv + 1); }

    const char* next() {
        const char* next = *(++m_argv);
        m_done |= !next;
        return m_done ? nullptr : next;
    }

    const char* operator*() const { return m_done ? nullptr : *m_argv; }

  private:
    const char* const* m_argv;
    bool m_done{};
};

/// Parser for GNU-style command-line options.
class Parser {
  public:
    explicit Parser(CommandRef command, const char* const* argv)
            : m_stream(argv), m_cmd(std::move(command)), m_context(&*m_cmd) {}

    ParseResult parse(const char* const*) {
        const char* arg;

        while ((arg = m_stream.next()) != nullptr) {
            // end of options
            if (strcmp("--", arg) == 0) {
                collect_remaining();
            }
            // long option
            else if (strncmp("--", arg, 2) == 0) {
                handle_long_opt(arg);
            }
            // short, possibly chained option(s)
            else if (*arg == '-' && m_context->chaining) {
                handle_chained_opt(arg);
            }
            // short, non-chained option
            else if (*arg == '-' || *arg == '+') {
                handle_short_opt(arg);
            }
            // subcommand or positional argument
            else {
                handle_unnamed_arg(arg);
            }
        }
        return ParseResult{std::move(m_cmd)};
    }

  private:
    static bool is_value(const char* arg) { return arg && *arg != '-' && *arg != '+'; }

    void maybe_help(const std::string& token) const {
        auto it = std::find(m_context->help.begin(), m_context->help.end(), token);
        if (it != m_context->help.end()) {
            throw HelpException{m_context};
        }
    }

    /// Tries to find an option with the given name.
    /// Throws UnknownOptionException if it fails.
    /// If an option was found, it's added to a vector that keeps track
    /// of parsed options. This is to provide an iterator of all the parsed
    /// options in the order they were parsed in.
    OptionRef find_opt(const std::string& token) {
        maybe_help(token);

        auto it = m_context->options.find(token);
        if (it == m_context->options.cend()) {
            throw UnknownOptionException(m_context, token, "unrecognized option '" + token + '\'');
        }

        auto opt = it->second;
        m_context->parsed.emplace_back(token, opt);
        return opt;
    }

    /// Handles short options when chaining is disabled.
    /// A space is not required between an option and its value.
    /// E.g. `-xVALUE` is perfectly valid.
    void handle_short_opt(const char* arg) {
        if (strlen(arg) == 2) {
            return handle_opt(arg);
        }

#ifdef INTERCOM_CIDL_OPTS
        // Special-case handling for CIDL's deprecated options.
        // These do not follow the GNU convention, and as such have to be handled
        // separately. Some of the options start with a single dash followed by
        // a multi-character name (e.g. `-to`).
        // To circumvent this, we check if the entire argument matches a known option.
        // If it does, treat it as such; if not, treat it as a short, non-chained option.
        {
            auto opt = m_context->options.find(arg);
            if (opt != m_context->options.cend()) {
                return handle_opt(arg);
            }
        }
#endif

        // option was immediately followed by a value, without a delimiting space
        const char opt[3] = {arg[0], arg[1], '\0'};
        const char* value = arg + 2;

        OptionRef ref = find_opt(opt);
        if (ref->kind == NONE) {
            std::ostringstream ss;
            ss << "invalid syntax '" << arg << "' -- flag '" << opt << "' does not accept arguments";
            throw InvalidValueException(ss.str());
        }

        ref->add_value(value);
    }

    /// Handles short options when chaining is enabled.
    void handle_chained_opt(const char* arg) {
        // iterate through every letter, but skip the first one since it will always be '-'
        for (const char* c = arg + 1; *c != '\0'; c++) {
            // this is necessary as long as we support '+' options
            const char opt[3] = {'-', *c, '\0'};
            const char next = *(c + 1);

            // if the option takes a value, make sure it is the last option in the sequence
            OptionRef ref = find_opt(opt);
            if (ref->kind != NONE && next != '\0') {
                throw InvalidValueException{"invalid syntax '" + std::string(arg) + '\''};
            }

            handle_opt(ref);
        }
    }

    /// Handles long options. Unlike short options, long options can use the following
    /// format in addition to the normal formats:
    ///   `--long-option=value1,value2,value3`
    void handle_long_opt(const char* arg) {
        if (strchr(arg, '=') == nullptr) {
            return handle_opt(arg);
        }

        auto key_val = split_at(arg, '=');
        OptionRef option = find_opt(key_val.first);

        auto segments = split(key_val.second);
        for (const auto& segment : segments) {
            option->add_value(segment);
        }
    }

    void handle_opt(const OptionRef& opt) {
        if (opt->kind == NONE) {
            opt->count++;
        } else {
            consume_positional(opt);
        }
    }

    // Overloaded to prevent the lookup from happening twice, as this
    // messes with the tracking of parsed options that happens in `find_opt`
    void handle_opt(const char* arg) {
        OptionRef option = find_opt(arg);
        handle_opt(option);
    }

    /// Consumes arguments that comes after an option and places them into
    /// said option's list of values.
    void consume_positional(const OptionRef& opt) {
        // verify that the next argument is a value and not an option
        if (!m_stream.peek()) {
            throw InvalidValueException{"argument to '" + std::string(*m_stream) + "' is missing"};
        }

        maybe_help(m_stream.peek());

        if (!is_value(m_stream.peek())) {
            throw InvalidValueException{"expected value, found '" + std::string(m_stream.peek()) + '\''};
        }

        do {
            const char* value = m_stream.next();
            opt->add_value(value);
        } while (opt->kind == MULTIPLE && opt->multiple && is_value(m_stream.peek()));
    }

    bool maybe_subcmd(const char* arg) {
        if (!m_context->subcommands.empty()) {
            auto it = m_context->subcommands.find(arg);
            if (it == m_context->subcommands.cend()) {
                std::string found = *m_stream;
                throw UnknownSubcommand{m_context, found, "unknown subcommand '" + found + '\''};
            }
            m_context = &*it->second;
            m_context->present = true;

            while (m_context->external && m_stream.peek() && strcmp(m_stream.peek(), "--") != 0) {
                it->second->params.emplace_back(m_stream.next());
            }
            return true;
        }
        return false;
    }

    /// Handles unnamed arguments -- arguments that do not require leading options.
    void handle_unnamed_arg(const char* arg) {
        if (maybe_subcmd(arg)) {
            return;
        }
        if (!m_context->args) {
            throw InvalidValueException{"unexpected value '" + std::string(arg) + '\''};
        }
        m_context->params.emplace_back(arg);
    }

    /// Collects everything that comes after the end of the options.
    /// Note: these will always be added to the parent command, regardless of whether the
    /// subcommand accepts positionals or not.
    void collect_remaining() {
        while (m_stream.peek()) {
            m_cmd->params.emplace_back(m_stream.next());
        }
    }

  private:
    ArgStream m_stream;
    CommandRef m_cmd;
    // The context reflects the current command instance. Changes for every subcommand.
    Command* m_context;
};

}  // namespace detail
}  // namespace cli
}  // namespace intercom
