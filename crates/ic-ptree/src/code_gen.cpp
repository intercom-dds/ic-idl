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

#include <fcntl.h>
#include <fmt/chrono.h>

#include <cassert>
#include <chrono>
#include <climits>
#include <cstdarg>
#include <cstring>
#include <fstream>
#include <iostream>
#include <string_view>

#include "cidl/commandline.h"
#include "cidl/hdrs.h"
#include "cidl/ptree_builder.h"

#ifdef _WIN32
#  include <direct.h>
#  include <fcntl.h>
#  include <io.h>
#  include <stdio.h>
#  include <sys/stat.h>
#  include <sys/types.h>
#  define mkdir(x, y) _mkdir(x)
#else
#  include <sys/stat.h>
#  include <sys/types.h>
#  include <unistd.h>
#endif

const int MAX_STATEMENTS = MEMF_MAX_STATEMENTS - 1;

bool is_path_sep(char c) {
#ifdef INTERCOM_PLATFORM_WINDOWS
    return c == '/' || c == '\\';
#else
    return c == '/';
#endif
}

std::string trim_include_name(std::filesystem::path name, bool trim_absolute) {
    auto file = name.replace_extension();
    std::string native = name.string();
    if (trim_absolute && (is_path_sep(native[0]) ||
                          (native[0] != '\0' && native[1] == ':' && is_path_sep(native[2])) ||
                          (native[0] == '.' && native[1] == '.' && is_path_sep(native[2])))) {
        return file.stem();
    }
    return file;
}

#define MAX_LINE_LENGTH (90000)

void madjsize(struct memf* memf, int extra) {
    size_t has_size = memf->memp - memf->memfile;
    if ((has_size + MAX_LINE_LENGTH + extra) > static_cast<size_t>(memf->size)) {
        if (extra == 0) {
            extra = memf->size;
        }
        memf->size += extra + MAX_LINE_LENGTH;
        memf->memfile = static_cast<char*>(realloc(memf->memfile, memf->size));
        memf->memp = memf->memfile + has_size;
    }
}

void set_indent(struct memf* a_memf, int a_indent) {
    if (!a_memf->extern_lock_indent) {
        a_memf->indent = a_indent;
    }
}

void incr_indent(struct memf* a_memf, int a_count) {
    if (!a_memf->extern_lock_indent) {
        a_memf->indent += a_count;
    }
}

#define CMP(str) (strncmp(str, ppp, sizeof(str) - 1) == 0)

/*
  Format modifiers:
     '~U' at start: emit only if not found earlier in file.
*/
void mprintflv(struct memf** memfl, std::string_view format, std::string_view string) {
    int iii;
    int tick = 0;
    int make_unique = 0;
    std::string_view::const_iterator ppp;

    if (format.compare(0, 2, "~U") == 0) {
        string = string.substr(2);
        make_unique = 1;
    }
    for (; *memfl; memfl++) {
        struct memf* memf = *memfl;
        if (make_unique && memf->memfile &&
            std::string_view(memf->memfile).find(string) != std::string_view::npos) {
            continue;
        }

        madjsize(memf, 0);
        for (ppp = string.begin(); ppp != string.end(); ++ppp, memf->memp++) {
            int start_pos = memf->column;
            memf->column++;
            switch (*ppp) {
            case '\n':
                memf->do_indent = 1;
                memf->ticktick = 0;
                memf->column = 0;
                break;
            default:
                switch (memf->lang_kind) {
                case ADA_FILE:
                    // end of statement
                    if ((memf->statement_end[memf->current_statement] == 127 && start_pos == 0 &&
                         (CMP("end") || CMP("when"))) ||
                        (memf->statement_end[memf->current_statement] == *ppp)) {
                        memf->current_statement =
                            memf->current_statement <= 0
                                ? 0
                                : (memf->current_statement - 1);  // sanity check
                    }

                    // retrieve current indent level
                    set_indent(
                        memf,
                        memf->statement_indent
                            [memf->current_statement < MAX_STATEMENTS ? memf->current_statement
                                                                      : MAX_STATEMENTS]
                    );

                    // temporarily adjust indent for certain keywords
                    if ((start_pos == 0 &&
                         (CMP("then") || CMP("exception\n") || CMP("begin") || CMP("loop") ||
                          CMP("record") || CMP("else") || CMP("elsif") || CMP("private\n")))) {
                        incr_indent(memf, -4);
                    } else if (start_pos == 0 && (*ppp == '(')) {
                        incr_indent(memf, 4);
                    }
                    break;
                default:
                    if ((start_pos == 0 && (CMP("case") || CMP("default:")))) {
                        incr_indent(memf, -4);
                    }
                    switch (*ppp) {
                    case '}':
                    case ')':
                        if (memf->indent > 2 && !(tick & 1) && !(memf->ticktick & 1)) {
                            incr_indent(memf, -4);
                        }
                        break;
                    case '\'':
                        tick++;
                        break;
                    case '\"':
                        memf->ticktick++;
                        break;
                    }
                }

                if (memf->do_indent && (*ppp != '#')) {
                    memf->column += memf->indent;
                    for (iii = memf->indent; iii > 0; iii--) {
                        *((memf->memp)++) = ' ';
                    }
                }
                memf->do_indent = 0;
                switch (memf->lang_kind) {
                case ADA_FILE:
                    // handle levels of statements
                    if (start_pos == 0 &&
                        (CMP("package ") || CMP("procedure ") || CMP("function ") || CMP("type ") ||
                         CMP("case ") || CMP("for ") || CMP("pragma ") || CMP("if "))) {
                        set_indent(memf, memf->statement_indent[memf->current_statement]);
                        memf->current_statement++;

                        if (memf->current_statement < MAX_STATEMENTS) {
                            memf->statement_indent[memf->current_statement] = memf->indent;
                            memf->statement_end[memf->current_statement] = ';';
                        }
                    } else if (*ppp == '(') {
                        set_indent(
                            memf,
                            memf->statement_indent[memf->current_statement] +
                                (start_pos == 0 ? 4 : 0)
                        );
                        memf->current_statement++;

                        if (memf->current_statement < MAX_STATEMENTS) {
                            memf->statement_indent[memf->current_statement] = memf->indent;
                            memf->statement_end[memf->current_statement] = ')';
                        }

                    } else if (CMP("record\n") || CMP("loop\n") || CMP("then\n") || CMP("=>\n") ||
                               CMP("declare\n") || CMP("do\n") || CMP("is\n")) {
                        set_indent(memf, memf->statement_indent[memf->current_statement] + 3);
                        memf->current_statement++;

                        if (memf->current_statement < MAX_STATEMENTS) {
                            memf->statement_indent[memf->current_statement] = memf->indent;
                            memf->statement_end[memf->current_statement] = 127;
                        }
                    }

                    break;
                default:
                    if (start_pos == 0) {
                        if (CMP("case") || CMP("default:")) {
                            incr_indent(memf, 4);
                        } else if (CMP("namespace")) {
                            incr_indent(memf, -4);
                        }
                    }
                    switch (*ppp) {
                    case '{':
                    case '(':
                        if (!(tick & 1) && !(memf->ticktick & 1)) {
                            incr_indent(memf, 4);
                        }
                        break;
                    }
                }
            }
            *(memf->memp) = *ppp;
        }
        *(memf->memp) = *ppp;
    }
}

void memfcat(struct memf* f1, struct memf* f2) {
    size_t size = f2->memp - f2->memfile;
    if (size > 0) {
        madjsize(f1, size);
        memcpy(f1->memp, f2->memfile, size);
        f1->memp += size;
    }
}

void memfcat_str(struct memf* f1, const char* f2) {
    size_t size = strlen(f2);
    madjsize(f1, size);
    memcpy(f1->memp, f2, size);
    f1->memp += size;
}

void mreset(struct memf* memf) {
    mreset_l(memf, C_JAVA_FILE);
}

void mreset_l(struct memf* memf, lang_kind_t lang_kind) {
    free(memf->memfile);
    memf->memfile = nullptr;
    memf->indent = memf->do_indent = 0;
    memf->size = 0;
    memf->memp = memf->memfile;
    memf->lang_kind = lang_kind;
    memf->current_statement = 0;
    memf->statement_indent[0] = 0;
    memf->statement_end[0] = 127;
}

int mempty(struct memf* memf) {
    return memf->memp == memf->memfile;
}

bool write_if_changed(const std::string& file_name, const std::string& content) {
    bool file_changed = true;
    std::ifstream current_file_content{file_name};
    if (current_file_content.is_open()) {
        std::stringstream current;
        current << current_file_content.rdbuf();
        file_changed = (current.str() != content);
        current_file_content.close();
    }
    if (file_changed) {
        try {
            std::filesystem::create_directories(std::filesystem::path(file_name).parent_path());
            std::ofstream output{file_name};
            if (output.is_open()) {
                output << content;
            } else {
                fmt::print(stderr, "Could not write to file \"{}\"\n", file_name);
                return false;
            }
        } catch (const std::exception& e) {
            fmt::print(stderr, "Could not write to file \"{}\": {}\n", file_name, e.what());
            return false;
        }
    }
    return true;
}

parse_result clone_tree(const parse_result* result) {
    IdlParser parser;
    parser.run([&] { return duplicate_tree(result->tree); });
    g_state = parser.state();

    auto clone = parser.result();
    clone.error_count = result->error_count;
    clone.warning_count = result->warning_count;
    clone.modules = result->modules;
    clone.msg = result->msg;
    for (auto inc : result->includes) {
        clone.includes.emplace(duplicate_node(inc));
    }
    return clone;
}

void savememf(
    struct memf* memf,
    struct memf* memf2,
    const char* basedir,
    const char* filedir,
    const char* frmt,
    const char* module
) {
    size_t size;
    auto filename =
        fmt::format(fmt::runtime(frmt), module, CommandLineOption::cpp_header_postfix());
    std::string fullfilename;
    size = memf->memp - memf->memfile;

    if (filedir && *filedir) {
        fullfilename = fmt::format("{}/{}/{}", basedir, filedir, filename);
    } else {
        fullfilename = fmt::format("{}/{}", basedir, filename);
    }
    if (!CommandLineOption::list_only()) {
        std::stringstream content;
        if (memf2) {
            content.write(
                memf2->memfile, static_cast<std::streamsize>(memf2->memp - memf2->memfile)
            );
        }
        content.write(memf->memfile, static_cast<std::streamsize>(size));
        write_if_changed(fullfilename, content.str());
    } else {
        if (filedir && *filedir) {
            fmt::print("{}/{}\n", filedir, filename);
        } else {
            fmt::print("{}\n", filename);
        }
    }
}

File memf_to_file(
    struct memf* memf,
    struct memf*,
    const char* basedir,
    const char* filedir,
    const char* frmt,
    const char* module
) {
    size_t size;
    std::string filename;
    std::string fullfilename;

    if (module) {
        filename = fmt::format(fmt::runtime(frmt), module, CommandLineOption::cpp_header_postfix());
    } else {
        filename = fmt::format(fmt::runtime(frmt), filedir);
    }
    size = memf->memp - memf->memfile;

    if (filedir && *filedir && basedir && *basedir) {
        fullfilename = fmt::format("{}/{}/{}", basedir, filedir, filename);
    } else if (basedir && *basedir) {
        fullfilename = fmt::format("{}/{}", basedir, filename);
    } else {
        fullfilename = filename;
    }
    return {fullfilename, std::string(memf->memfile, size)};
}

std::string copyright_header(const std::string& comment_str) {
    constexpr const char* header =
        "{0} KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,\n"
        "{0} contain information which is proprietary and confidential to KONGSBERG or its licensors.\n"
        "{0} Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed\n"
        "{0} with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,\n"
        "{0} or disassemble the software, unless such acts are allowed under applicable mandatory law or\n"
        "{0} explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,\n"
        "{0} must include this legend. (C) {1:%Y} KONGSBERG - All rights reserved\n";
    return fmt::format(header, comment_str, std::chrono::system_clock::now());
}
