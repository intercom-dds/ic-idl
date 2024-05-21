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
#include <map>
#include <optional>
#include <set>
#include <string>
#include <utility>

// IWYU pragma: begin_exports
#include "Error.h"
#include "Formatter.h"
#include "Option.h"
#include "ParseResult.h"
#include "Parser.h"
#include "Utils.h"
// IWYU pragma: end_exports

namespace intercom::cli {

/// The following formats are supported:
///   (1) `-a` and `+a`
///   (2) `-a VALUE1 VALUE2` and `+a VALUE1 VALUE2`
///   (3) `--long-opt`
///   (4) `--long-opt VALUE1 VALUE2`
///   (5) `--long-opt=VALUE1,VALUE2`
///
/// If chaining is enabled, the following formats are also accepted:
///   (a) `-abc` (equivalent to `-a -b -c`)
///   (b) `-abc VALUE1 VALUE2` (if `-c` takes arguments)
///
/// If chaining is disabled, the following is accepted:
///   (a) `-aVALUE` and `+aVALUE` (only works for single-value options)
///
class CommandLine {
  public:
    explicit CommandLine(std::string name) : m_cmd(new detail::Command{std::move(name)}) {}

    CommandLine& desc(std::string desc) {
        m_cmd->description = std::move(desc);
        return *this;
    }

    CommandLine& long_desc(std::string desc) {
        m_cmd->long_desc = std::move(desc);
        return *this;
    }

    CommandLine& version(std::string version) {
        m_cmd->version = std::move(version);
        return *this;
    }

    CommandLine& copyright(std::string copyright) {
        m_cmd->copyright = std::move(copyright);
        return *this;
    }

    CommandLine& after_help(std::string after_help) {
        m_cmd->after_help = std::move(after_help);
        return *this;
    }

    CommandLine& hide_flags(bool hide) {
        m_cmd->hide_flags = hide;
        return *this;
    }

    /// Sets the tokens for the auto-generated help menu.
    ///
    /// Default is `-h` and `--help`
    template <typename... Args>
    CommandLine& help_args(Args... args) {
        m_cmd->help = {args...};
        return *this;
    }

    /// Specifies whether the parser should provide a suggestion if an unrecognized
    /// option was given by the user.
    ///
    /// Default is true.
    CommandLine& suggestions(bool enable_suggestions) {
        m_cmd->suggestions = enable_suggestions;
        return *this;
    }

    /// Specifies whether option aggregation should be enabled or not.
    /// If enabled, '-laf' will be equivalent to '-l -a -f'
    /// Note that if chaining is enabled, a space is required between the option
    /// and its value. E.g. '-fVALUE' is not valid, but '-f VALUE' is. With chaining
    /// disabled, both are valid.
    ///
    /// Default is false.
    CommandLine& chaining(bool enable_chaining) {
        m_cmd->chaining = enable_chaining;
        return *this;
    }

    /// Whether positional arguments are accepted or not.
    /// Positionals are defined as arguments that do not need a leading option.
    /// E.g. `cidl some_file.idl`
    /// Default is false. `arg_name` sets the name shown in the help menu.
    CommandLine& positionals(bool accept_positionals, std::string arg_name = "ARG") {
        m_cmd->args = accept_positionals;
        m_cmd->arg_name = std::move(arg_name);
        return *this;
    }

    /// If this is set to true, the parser will not try to resolve options or parameters
    /// passed to this command. Instead, it will collect all values in an array.
    /// Primarily used for subcommands for things like plugins.
    CommandLine& external(bool external) {
        m_cmd->external = external;
        return *this;
    }

    /// Adds a new subcommand.
    /// Note: subcommands are converted to lowercase.
    CommandLine& subcommand(CommandLine subcmd) {
        subcmd.m_cmd->name = detail::to_lower(subcmd.m_cmd->name);
        subcmd.m_cmd->parent = m_cmd.get();

        // Propagate version information for internal subcommands.
        if (!subcmd.m_cmd->external && subcmd.m_cmd->version.empty()) {
            subcmd.m_cmd->version = m_cmd->version;
        }
        subcmd.add_default_opts();
        m_cmd->subcommands.emplace(subcmd.m_cmd->name, subcmd.m_cmd);
        return *this;
    }

    /// Adds new subcommands.
    template <typename... Args>
    CommandLine& subcommands(Args&&... args) {
        for (const auto& cmd : {args...}) {
            subcommand(cmd);
        }
        return *this;
    }

    /// Sets the 'category' of the command instance.
    /// This affects the section in which the command will appear in.
    /// Only relevant for subcommands.
    CommandLine& category(std::string category) {
        m_cmd->category = std::move(category);
        return *this;
    }

    /// Defines a new section in the help menu.
    /// All options added after this will be added to the named section.
    CommandLine& section(std::string section) {
        m_section = std::move(section);
        return *this;
    }

    CommandLine& opt(const Option& option) {
        detail::OptionRef opt(option.m_impl);
        opt->section = m_section;

        auto insert_opt = [&](const std::vector<std::string>& tokens) {
            for (const auto& token : tokens) {
                // sanity check
                if (m_cmd->options.find(token) != m_cmd->options.cend()) {
                    throw std::logic_error("option '" + token + "' already exists");
                }
                m_cmd->ordered_options.emplace_back(opt);
                m_cmd->options.emplace(token, opt);
            }
        };

        insert_opt(opt->tokens);
        insert_opt(opt->deprecated_tokens);
        return *this;
    }

    CommandLine& opts(std::initializer_list<Option> options) {
        for (const auto& option : options) {
            opt(option);
        }
        return *this;
    }

    template <typename... Args>
    CommandLine& opts(Args&&... args) {
        opts({std::forward<Args&&>(args)...});
        return *this;
    }

    /// Returns the name of the command.
    const std::string& name() const {
        return m_cmd->name;
    }

    /// Returns the formatted version string.
    std::string version() const {
        detail::Formatter fmt{*m_cmd};
        return fmt.version();
    }

    /// Returns the formatted help menu.
    std::string help() const {
        return help(*m_cmd);
    }

    ParseResult parse(const char* const* argv) {
        add_default_opts();
        m_cmd->executable = std::filesystem::absolute(*argv);

        detail::Parser parser{m_cmd, argv};
        int ec = 1;

        try {
            auto result = parser.parse(argv);
            validate_result(argv, result);
            return result;
        } catch (const HelpException& e) {
            std::cout << help(e.command()) << std::flush;
            ec = 0;
        } catch (const UnknownSubcommand& e) {
            auto suggestion = suggest_cmd(e.command(), e.name());
            cli::error << e.what();
            ec = 127;

            if (m_cmd->suggestions && suggestion.has_value()) {
                std::cerr << ", did you mean '" << suggestion.value() << "'?";
            }
        } catch (const UnknownOptionException& e) {
            auto suggestion = suggest_opt(e.command(), e.option());
            cli::error << e.what();

            if (m_cmd->suggestions && suggestion.has_value()) {
                std::cerr << ", did you mean '" << suggestion.value() << "'?";
            }
        } catch (const std::exception& e) {
            cli::error << e.what();
        }

        std::cerr << std::endl;
        exit(ec);  // NOLINT
    }

  private:
    void add_default_opts() {
        section("flags");

        if (!m_cmd->help.empty()) {
            auto help = Option(m_cmd->help);
            help.desc("Display help information");
            opt(help);
        }

        if (!m_cmd->version.empty()) {
            auto vers = Option("-v", "--version");
            vers.desc("Display version information");
            vers.callback([this](const Args&) {
                std::cout << this->version() << std::endl;
                exit(0);  // NOLINT
            });
            opt(vers);
        }
    }

    static std::string help(const detail::Command& cmd) {
        detail::Formatter fmt{cmd};
        return fmt.version() + '\n' + fmt.help();
    }

    static std::optional<std::string>
    suggest_opt(const detail::Command& cmd, const std::string& opt) {
        // only provide suggestions for long options
        if (opt.size() <= 2 || strncmp(opt.data(), "--", 2) != 0) {
            return std::nullopt;
        }

        // filter out duplicate values and deprecated options
        std::set<std::string> options;
        for (const auto& cmd_opt : cmd.options) {
            if (cmd_opt.first.length() > 2 && !cmd_opt.second->deprecated) {
                options.insert(cmd_opt.first);
            }
        }
        return detail::did_you_mean(opt, options);
    }

    static std::optional<std::string>
    suggest_cmd(const detail::Command& cmd, const std::string& name) {
        std::set<std::string> commands;
        for (const auto& sub : cmd.subcommands) {
            commands.insert(sub.first);
        }
        return detail::did_you_mean(name, commands);
    }

    static void trigger_callbacks(const ParseResult& result) {
        for (const auto& parsed : result) {
            const auto& opt = parsed.option();
            if (opt->callback) {
                opt->callback(parsed.values());
            }
        }
    }

    static void validate_options(const ParseResult& result) {
        // Verify that all options marked as required are present
        for (const auto& opt : result.impl()->options) {
            if (opt.second->required && opt.second->count == 0) {
                throw std::runtime_error{
                    "option '" + opt.first + "' is required but was not provided"
                };
            }
        }

        for (const auto& opt : result) {
            // Check for deprecated options
            if (opt.option()->is_deprecated(opt.token())) {
                cli::info << "option '" << opt.token() << "' is deprecated" << std::endl;
            }

            // Check for conflicting options
            for (const auto& conflict : opt.option()->conflicts) {
                auto it = result.find(conflict);
                if (it != result.end()) {
                    throw std::runtime_error{
                        "option '" + opt.token() + "' conflicts with '" += conflict + '\''
                    };
                }
            }

            // Validate the input against the option's validator
            if (opt.option()->validator) {
                for (const auto& value : opt.values()) {
                    if (!opt.option()->validator(value)) {
                        throw std::runtime_error{"error"};
                    }
                }
            }
        }
    }

    void validate_result(const char* const* args, const ParseResult& result) {
        if (result.impl()->args && result.impl()->params.empty() && !result.subcommand()) {
            throw HelpException{result.impl().get()};
        }
        validate_options(result);
        trigger_callbacks(result);

        if (auto subcmd = result.subcommand()) {
            validate_result(args, *subcmd);
        }
    }

  private:
    detail::CommandRef m_cmd;
    std::string m_section;
};

}  // namespace intercom::cli
