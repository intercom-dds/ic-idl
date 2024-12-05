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

#include <cstdint>
#include <filesystem>
#include <functional>
#include <memory>
#include <ostream>
#include <set>
#include <string>
#include <vector>

#include "InterCOM/dyn_link.h"
#include "cidl/ptree.h"

namespace intercom::cidl {

using FileList = std::vector<std::pair<std::filesystem::path, std::filesystem::path>>;

struct parse_result {
    parse_result() = default;
    const ptree* tree{nullptr};
    std::set<const ptree*> includes;
    std::set<std::string> modules;
    int error_count{0};
    int warning_count{0};
    std::string msg;
    std::shared_ptr<parser> state;
};

struct IdlParserImpl;
struct JsonParserImpl;
struct XmlParserImpl;

enum class JsonValueFlags { FLAG_ESCAPED = 1, FLAG_NUMERICAL_VALUE = 2 };

INTERCOM_PUBLIC std::string
json_value(const numeric& value, const ptree* context = nullptr, int flags = 0);
INTERCOM_PUBLIC std::string json_value(const ptree* value);
INTERCOM_PUBLIC ptree* parse_json_ptree(const std::string& input);

INTERCOM_PUBLIC ptree* parse_xml(const std::string& input);
INTERCOM_PUBLIC ptree* parse_xml_file(const std::string& uri);

INTERCOM_PUBLIC parse_result merge_results(std::vector<parse_result>& to_merge);

INTERCOM_PUBLIC bool run_preprocessor(
    const std::string& a_file_name,
    const std::vector<std::string>& a_parameters,
    std::ostream& a_out,
    std::string& a_error
);

INTERCOM_PUBLIC bool
run_preprocessor(const std::string& a_file_name, std::ostream& a_out, std::string& a_error);

enum ParserFlagBits : uint32_t { SUPPRESS_CONTENTS_FROM_INCLUDES = 1, PREPROCESS_ONLY = 2 };

INTERCOM_PUBLIC parse_result run_parser(
    const std::vector<std::string>& input_files,
    const std::vector<std::string>& pp_options,
    uint32_t flags = 0
);

class IdlParser {
  public:
    INTERCOM_PUBLIC IdlParser();
    INTERCOM_PUBLIC ~IdlParser();

    INTERCOM_PUBLIC const parse_result& result() const;
    INTERCOM_PUBLIC parse_result& result();

    INTERCOM_PUBLIC void run(const std::string& input);
    INTERCOM_PUBLIC void run(FILE* input);
    INTERCOM_PUBLIC void run(const std::function<ptree*()>& input);

    INTERCOM_PUBLIC std::shared_ptr<parser> state();

  private:
    std::unique_ptr<IdlParserImpl> m_impl;
};

// TODO(idarcar);
// class JsonParser {
//   public:
//     INTERCOM_PUBLIC JsonParser();
//     INTERCOM_PUBLIC ~JsonParser();
//
//     INTERCOM_PUBLIC const parse_result& result() const;
//     INTERCOM_PUBLIC parse_result& result();
//
//     INTERCOM_PUBLIC void
//     run(const std::string& input, const std::string& input_file_name = "<stdin>");
//     INTERCOM_PUBLIC void run(std::istream& input, const std::string& input_file_name =
//     "<stdin>");
//
//     INTERCOM_PUBLIC std::shared_ptr<parser> state();
//
//   private:
//     std::unique_ptr<JsonParserImpl> m_impl;
// };
//
// class XmlParser {
//   public:
//     INTERCOM_PUBLIC XmlParser();
//     INTERCOM_PUBLIC ~XmlParser();
//
//     INTERCOM_PUBLIC const parse_result& result() const;
//     INTERCOM_PUBLIC parse_result& result();
//
//     INTERCOM_PUBLIC void
//     run(const std::string& input, const std::string& input_file_name = "<stdin>");
//     INTERCOM_PUBLIC std::shared_ptr<parser> state();
//
//   private:
//     std::unique_ptr<XmlParserImpl> m_impl;
// };
}  // namespace intercom::cidl
