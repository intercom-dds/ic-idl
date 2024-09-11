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

#include <cstdlib>
#include <cstring>
#include <iostream>
#include <optional>

#ifdef _WIN32
#  include <Windows.h>
#else
#  include <unistd.h>
#endif

namespace ic::color {

inline bool enable_colors() {
    auto term = std::getenv("TERM");  // NOLINT
    if (term && strcmp(term, "dumb") == 0) {
        return false;
    }

#ifdef _WIN32
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
    return isatty(STDOUT_FILENO) && isatty(STDERR_FILENO);
#endif
}

inline bool has_colors() {
    static bool s_enabled = enable_colors();
    return s_enabled;
}

struct IosSgrMetadata {
    IosSgrMetadata() = default;
    std::optional<bool> enabled;
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

inline std::ostream& ansi_csi_sgr(std::ostream& stream, const char* n) {
    auto meta = metadata(stream);
    const bool disabled = meta->enabled.has_value() && !meta->enabled.value();
    const bool enabled = meta->enabled.has_value() && meta->enabled.value();
    if (!disabled && (has_colors() || enabled)) {
        if (meta->last_code != n) {
            stream << "\x1b[" << n << 'm';
            meta->last_code = n;
        }
    }
    return stream;
}

constexpr const char* CSI_SGR_RESET = "0";

inline std::ostream& reset(std::ostream& stream) {
    return ansi_csi_sgr(stream, CSI_SGR_RESET);
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

inline std::ostream& yellow(std::ostream& stream) {
    return ansi_csi_sgr(stream, "33");
}

inline std::ostream& blue(std::ostream& stream) {
    return ansi_csi_sgr(stream, "34");
}

inline std::ostream& magenta(std::ostream& stream) {
    return ansi_csi_sgr(stream, "35");
}

inline std::ostream& cyan(std::ostream& stream) {
    return ansi_csi_sgr(stream, "36");
}

inline std::ostream& white(std::ostream& stream) {
    return ansi_csi_sgr(stream, "37");
}

inline std::ostream& bright_green(std::ostream& stream) {
    return ansi_csi_sgr(stream, "92");
}

inline std::ostream& bright_magenta(std::ostream& stream) {
    return ansi_csi_sgr(stream, "95");
}

inline std::ostream& orange(std::ostream& stream) {
    return ansi_csi_sgr(stream, "38;2;255;135;0");
}

}  // namespace ic::color
