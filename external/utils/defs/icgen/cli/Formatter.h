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
#include <cstring>
#include <iostream>
#include <list>
#include <sstream>

#include "InterCOM/PlatformConfig.h"
#include "OptionImpl.h"

#ifdef INTERCOM_PLATFORM_WINDOWS
#  include <winsock2.h>

#  ifdef min
#    undef min
#  endif

#  ifdef max
#    undef max
#  endif

#  ifdef interface
#    undef interface
#  endif
#endif

namespace intercom::cli {
namespace color {

inline bool enable_colors() {
#ifdef INTERCOM_PLATFORM_WINDOWS
    auto enable_virt_term = [](HANDLE handle) {
        if (handle == INVALID_HANDLE_VALUE) {
            return false;
        }

        DWORD dw_mode = 0;
        if (!GetConsoleMode(handle, &dw_mode)) {
            return false;
        }

        dw_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if (!SetConsoleMode(handle, dw_mode)) {
            return false;
        }
        return true;
    };

    auto handle_stdout = GetStdHandle(STD_OUTPUT_HANDLE);
    auto handle_stderr = GetStdHandle(STD_ERROR_HANDLE);
    return enable_virt_term(handle_stdout) && enable_virt_term(handle_stderr);
#else
    return true;
#endif
}

inline bool has_colors() {
    static bool s_enabled = enable_colors();
    return s_enabled;
}

struct IosSgrMetadata {
    IosSgrMetadata() = default;
    std::optional<bool> enabled{std::nullopt};
    const char* last_code{nullptr};
};

inline IosSgrMetadata* metadata(std::ios_base& a_ios) {
    static int s_idx = std::ios_base::xalloc();
    auto*& meta = reinterpret_cast<IosSgrMetadata*&>(a_ios.pword(s_idx));
    if (!meta) {
        meta = new IosSgrMetadata();
        a_ios.register_callback(
            [](std::ios_base::event e, std::ios_base& e_ios, int e_idx) {
                if (e == std::ios_base::erase_event) {
                    auto*& e_meta = reinterpret_cast<IosSgrMetadata*&>(e_ios.pword(e_idx));
                    delete e_meta;
                    e_meta = nullptr;
                }
            },
            s_idx
        );
    }
    return meta;
}

inline void enable_colors(std::ios_base& ios, bool enable) {
    metadata(ios)->enabled = enable;
}

/// CSI {n} m -> ESC[{n}m
inline std::ostream& ansi_csi_sgr(std::ostream& stream, const char* n) {
    auto meta = metadata(stream);
    const bool disabled = meta->enabled.has_value() && !meta->enabled.value();
    const bool enabled = meta->enabled.has_value() && meta->enabled.value();
    if (!disabled && (has_colors() || enabled)) {
        // if (!meta->last_code || strcmp(meta->last_code, n) != 0) {
        if (meta->last_code != n) {
            stream << "\x1b[" << n << 'm';
            meta->last_code = n;
        }
    }
    return stream;
}

inline std::ostream& reset(std::ostream& stream) {
    return ansi_csi_sgr(stream, "0");
}

inline std::ostream& bold(std::ostream& stream) {
    return ansi_csi_sgr(stream, "1");
}

inline std::ostream& red(std::ostream& stream) {
    return ansi_csi_sgr(stream, "31");
}

inline std::ostream& green(std::ostream& stream) {
    return ansi_csi_sgr(stream, "32");
}

/// sometimes slightly green or entirely orange
inline std::ostream& yellow(std::ostream& stream) {
    return ansi_csi_sgr(stream, "33");
}

inline std::ostream& blue(std::ostream& stream) {
    return ansi_csi_sgr(stream, "34");
}

inline std::ostream& magenta(std::ostream& stream) {
    return ansi_csi_sgr(stream, "35");
}

/// often teal
inline std::ostream& cyan(std::ostream& stream) {
    return ansi_csi_sgr(stream, "36");
}

/// usually gray
inline std::ostream& white(std::ostream& stream) {
    return ansi_csi_sgr(stream, "37");
}

inline std::ostream& bright_green(std::ostream& stream) {
    return ansi_csi_sgr(stream, "92");
}

inline std::ostream& orange(std::ostream& stream) {
    return ansi_csi_sgr(stream, "38;2;255;135;0");  // 24-bit sgr has better support than 8-bit
}

}  // namespace color

namespace detail {
struct Section {
    explicit Section(std::string name) : name(std::move(name)) {}

    void insert(OptionRef option) {
        name = option->section;
        width = std::max(width, option->length + option->arg_name.length());

        auto it = std::find(options.begin(), options.end(), option);
        if (it == options.end()) {
            options.emplace_back(std::move(option));
        }
    }

    std::string name;
    size_t width = 0;
    bool has_flags = false;
    bool has_opts = false;
    std::vector<OptionRef> options;
};

/// Pretty printer for command-line options.
class Formatter {
  public:
    explicit Formatter(const Command& info) : m_cmd(info) {
        for (const auto& opt : m_cmd.ordered_options) {
            // skip deprecated options as these should not be visible in the help menu
            if (opt->deprecated || opt->hidden) {
                continue;
            }
            if (opt->section.empty()) {
                opt->section = opt->kind == NONE ? "flags" : "options";
            }

            // order the options by length
            std::sort(opt->tokens.begin(), opt->tokens.end(), len_sort);

            Section* section{};
            for (auto& sec : m_sections) {
                if (sec.name == opt->section) {
                    section = &sec;
                    break;
                }
            }
            if (!section) {
                section = &*m_sections.emplace(m_sections.end(), opt->section);
            }
            section->insert(opt);

            if (opt->kind == NONE) {
                m_has_flags = true;
                section->has_flags = true;
            } else {
                m_has_options = true;
                section->has_opts = true;
            }
        }
    }

    std::string version() const {
        std::ostringstream ss;
        ss << color::green << qualified_name(false, '-') << color::reset;
        ss << ' ' << m_cmd.version;

        if (!m_cmd.copyright.empty()) {
            ss << '\n' << m_cmd.copyright;
        }
        return ss.str();
    }

    std::string usage() const {
        std::ostringstream ss;
        ss << color::yellow << "\nusage:\n" << color::reset;
        ss << space(INDENT) << qualified_name(true) << ' ';

        if (m_has_flags && !m_cmd.hide_flags) {
            ss << "[flags] ";
        }
        if (m_has_options) {
            ss << "[options] ";
        }
        if (!m_cmd.subcommands.empty()) {
            ss << "[command]";
            if (m_cmd.args) {
                ss << " -- ";
            }
        }
        if (m_cmd.args) {
            ss << '<' << m_cmd.arg_name << ">...";
        }
        return ss.str();
    }

    std::string help() const {
        std::stringstream ss;

        if (!m_cmd.description.empty()) {
            ss << '\n' << m_cmd.description << '\n';
        }
        if (!m_cmd.long_desc.empty()) {
            ss << '\n' << m_cmd.long_desc << '\n';
        }
        ss << usage() << '\n';

        if (!m_cmd.hide_flags) {
            ss << sections();
        }

        if (!m_cmd.subcommands.empty()) {
            auto exec = m_cmd.executable.value_or(m_cmd.name);
            ss << color::yellow << "\ncommands:\n" << subcommands();
        }
        if (!m_cmd.after_help.empty()) {
            ss << '\n' << m_cmd.after_help;
        }
        return ss.str();
    }

  private:
    std::string sections() const {
        std::stringstream ss;
        for (const auto& section : m_sections) {
            if (!section.options.empty()) {
                ss << print_section(section);
            }
        }
        return ss.str();
    }

    std::string subcommands() const {
        std::ostringstream ss;
        size_t width = 0;
        for (const auto& cmd : m_cmd.subcommands) {
            width = std::max(width, cmd.first.length());
        }
        width += INDENT;

        for (const auto& subcmd : m_cmd.subcommands) {
            const size_t align = width - subcmd.first.length();
            ss << space(INDENT) << color::green << subcmd.first << space(align) << color::reset
               << subcmd.second->description << std::endl;
        }
        return ss.str();
    }

    template <typename Container>
    static std::string print_all(const Container& container) {
        std::ostringstream ss;
        for (auto it = container.cbegin(); it != container.cend(); ++it) {
            ss << color::green << *it << color::reset;
            if (std::next(it) != container.cend()) {
                ss << ", ";
            }
        }
        return ss.str();
    }

    std::string qualified_name(bool use_exec, char delim = ' ') const {
        std::stringstream name;

        std::function<void(const Command*)> emit_name = [&](const Command* cmd) {
            if (cmd->parent) {
                emit_name(cmd->parent);
                name << delim;
            }
            if (use_exec && cmd->executable) {
                name << cmd->executable->stem().string();
            } else {
                name << cmd->name;
            }
        };

        emit_name(&m_cmd);
        return name.str();
    }

    static std::string print_section(const Section& section) {
        std::stringstream ss;
        ss << color::yellow << "\n" << section.name << ":\n";
        const size_t width = section.width + INDENT;
        const bool combines_flags_and_opts = section.has_flags && section.has_opts;

        for (const auto& option : section.options) {
            const bool is_flag = option->kind == NONE;
            size_t align = width - option->length;
            if (!is_flag) {
                align -= option->arg_name.length();

                if (combines_flags_and_opts) {
                    // three is the number of extra characters an option has
                    // compared to a flag. we subtract that number here to align
                    // the options with the flags.
                    align -= 3;
                }
            }
            ss << space(INDENT) << print_all(option->tokens);

            if (!is_flag) {
                ss << color::green << " <" << option->arg_name << '>';
            }

            auto desc_lines = split(option->description, '\n');
            ss << color::reset << space(align) << desc_lines.front() << std::endl;
            for (auto line = desc_lines.begin() + 1; line != desc_lines.end(); line++) {
                ss << space(width + 2U + (!is_flag && !combines_flags_and_opts ? 3U : 0U)) << *line
                   << std::endl;
            }
        }
        return ss.str();
    }

  private:
    bool m_has_flags = false;
    bool m_has_options = false;

    const Command& m_cmd;
    std::list<Section> m_sections;
    static const size_t INDENT = 3;
};

struct error_t {  // NOLINT
    template <typename T>
    std::ostream& operator<<(const T& src) const {
        std::cerr << color::bold << color::red << "error: " << color::reset << src;
        return std::cerr;
    }
};

struct warn_t {  // NOLINT
    template <typename T>
    std::ostream& operator<<(const T& src) const {
        std::cerr << color::bold << color::yellow << "warning: " << color::reset << src;
        return std::cerr;
    }
};

struct info_t {  // NOLINT
    template <typename T>
    std::ostream& operator<<(const T& src) const {
        std::cerr << color::bold << "info: " << color::reset << src;
        return std::cerr;
    }
};
}  // namespace detail

static constexpr detail::error_t error{};  // NOLINT
static constexpr detail::warn_t warn{};    // NOLINT
static constexpr detail::info_t info{};    // NOLINT

}  // namespace intercom::cli
