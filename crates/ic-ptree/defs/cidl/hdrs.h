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

#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <string_view>
#include <vector>

#include "cidl/codegen.h"
#include "cidl/idl_parser.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

enum ExtensibilityKind { FINAL_EXTENSIBILITY, EXTENSIBLE_EXTENSIBILITY, MUTABLE_EXTENSIBILITY };

/// generates an info header to append to the start of all generated files
/// \param src_file name of or path to source file (e.g. .idl file)
/// \param single_line_comment_start start of every line (e.g. "//" or "#")
///     [should start a single line comment]
/// \note returned string doesn't end with newline
/// \warning not thread safe (std::ctime)
std::string generate_info_header(const std::filesystem::path& src_file,
                                 const std::string& single_line_comment_start = "");
/// generates an info header to append to the start of all generated files
/// \param src_file name of or path to source file (e.g. .idl file)
/// \param single_line_comment_start start of every line (e.g. "//" or "#")
///     [should start a single line comment]
/// \note returned string doesn't end with newline
/// \warning not thread safe (std::ctime)
std::string generate_info_header(const std::vector<std::filesystem::path>& src_files,
                                 const std::string& single_line_comment_start);

std::string trim_include_name(std::filesystem::path name, bool trim_absolute);
void code_gen_cs(parse_result* result, std::list<File>* generated = nullptr);
void code_gen_idl(parse_result* result, std::list<File>* generated = nullptr);
void code_gen_json(parse_result* result, bool listonly);
void code_gen_json_schema(parse_result* result, std::list<File>* generated = nullptr);
void code_gen_dds_cplpl(struct parse_result* result, std::list<File>* generated = nullptr);
void code_gen_ada_types(struct parse_result* head);
void code_gen_ada_interfaces(struct parse_result* head);
void code_gen_ada_proxies(parse_result* head);
void code_gen_ada_cdr(struct parse_result* head);
void code_gen_java(struct parse_result* result);
void code_gen_java_proxies(struct parse_result* head);
void code_gen_python(parse_result* result, std::list<File>* generated = nullptr);
void code_gen_properties(parse_result* result);
void code_gen_rust(const parse_result* result);
void code_gen_proto(const parse_result* result);
void code_gen_xml(const parse_result* result);
void transform_rust(parse_result* node);
void generate_json_type(std::ostream& stream, const ptree* tree);
void ast_dump(parse_result* result);
std::string toml_emit_node(std::string_view name, const ptree* tree);

void validate_proto(const ptree* node);

INTERCOM_PUBLIC std::string cpp_type_name(const ptree* node, const ptree* context);

void get_type_library(const ptree* obj, unsigned char** cdr_typedef, size_t* len);
std::string get_type_id(const ptree* obj);

enum lang_kind_t { ADA_FILE = 1, C_JAVA_FILE };

#define MEMF_MAX_STATEMENTS 50

struct memf {
    char* memp;
    char* memfile;
    int indent, do_indent;
    int extern_lock_indent;                     //!< will not alter indent while true
    int statement_indent[MEMF_MAX_STATEMENTS];  // 50 levels of nested statements
    char statement_end[MEMF_MAX_STATEMENTS];    // 50 levels of nested statements
    int current_statement;
    int size;
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

void ada_conv_gen_elem(struct memf* code_file, struct memf* with_file, const ptree* obj, const char* in_tag,
                       unsigned in_flag, const ptree* actual_object);

void savememf(struct memf* memf, struct memf* memf2, const char* basedir, const char* filedir, const char* frmt,
              const char* module);

File memf_to_file(struct memf* memf, struct memf* memf2, const char* basedir, const char* filedir, const char* frmt,
                  const char* module);

void memfcat(struct memf* f1, struct memf* f2);
void memfcat_str(struct memf* f1, const char* f2);

using PtreeSeq = std::vector<const ptree*>;
using PtreeSeqSeq = std::vector<PtreeSeq>;
PtreeSeqSeq ptree_build_order(const ptree* obj);

/// Duplicates the entire tree and all related state.
parse_result clone_tree(const parse_result* result);

// Output content to file_name unless content equals the current file content.
// file_name may contain an absolute or relative path, and any missing directories
// are recursively created.
bool write_if_changed(const std::string& file_name, const std::string& content);

class MemfIndentScopeLock {
  public:
    explicit MemfIndentScopeLock(memf* a_memf) : m_memf(a_memf), m_prev_indent_lock(a_memf->extern_lock_indent) {
        m_memf->extern_lock_indent = 1;
    }
    ~MemfIndentScopeLock() { m_memf->extern_lock_indent = m_prev_indent_lock; }

  private:
    memf* m_memf;
    int m_prev_indent_lock;
};

std::string copyright_header(const std::string& comment_str = "//");

#define INTERCOM_PUBLIC_MACRO_NAME "INTERCOM_PUBLIC"

#define JAVANAME_FLAGS_CLASS 1
#define JAVANAME_FLAGS_PROXY 2
#define JAVANAME_NO_ARRAY_SUFFIX 4

#if _WIN32
#  define snprintf sprintf_s
#endif
