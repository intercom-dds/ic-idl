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

#include <fmt/compile.h>
#include <fmt/format.h>

#include <string_view>

namespace intercom::cidl {

// NOLINTNEXTLINE
enum lang_kind_t { ADA_FILE = 1, C_JAVA_FILE };

#define MEMF_MAX_STATEMENTS 50

// NOLINTNEXTLINE
struct memf {
    char* memp;
    char* memfile;
    int indent, do_indent;
    int extern_lock_indent;                     //!< will not alter indent while true
    int statement_indent[MEMF_MAX_STATEMENTS];  // 50 levels of nested statements
    char statement_end[MEMF_MAX_STATEMENTS];    // 50 levels of nested statements
    int current_statement;
    size_t size;
    int ticktick;
    int column;
    lang_kind_t lang_kind;
};

void mreset_l(struct memf* memf, lang_kind_t lang_kind);
void mreset(struct memf* memf);
int mempty(struct memf* memf);

void mprintflv(struct memf** memfl, std::string_view format, std::string_view string);

template <typename... Args>
void mprintf(struct memf* memf, const std::string& format, Args&&... args) {
    struct memf* memfl[2];
    memfl[0] = memf;
    memfl[1] = nullptr;
    mprintfl(memfl, format, std::forward<Args>(args)...);
}

template <typename... Args>
void mprintfl(struct memf** memfl, const std::string& format, Args&&... args) {
    auto str = fmt::format(fmt::runtime(format), std::forward<Args>(args)...);
    mprintflv(memfl, format, str);
}

void memfcat(struct memf* f1, struct memf* f2);
void memfcat_str(struct memf* f1, const char* f2);

class MemfIndentScopeLock {
  public:
    explicit MemfIndentScopeLock(memf* a_memf)
        : m_memf(a_memf), m_prev_indent_lock(a_memf->extern_lock_indent) {
        m_memf->extern_lock_indent = 1;
    }
    ~MemfIndentScopeLock() {
        m_memf->extern_lock_indent = m_prev_indent_lock;
    }

  private:
    memf* m_memf;
    int m_prev_indent_lock;
};

}  // namespace intercom::cidl
