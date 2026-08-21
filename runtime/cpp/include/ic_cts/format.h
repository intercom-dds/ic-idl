// Copyright 2026 KONGSBERG
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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
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

#include <format>
#include <sstream>
#include <string>

#include "ic_cts/json_serializer.h"

namespace ic_cts {

/// Base class for `std::formatter` specializations that render a value as JSON.
///
/// A leading `#` in the format specification selects indented JSON. The
/// remainder is parsed as a standard string format specification, which
/// supports fill, alignment, width and precision.
template <typename T>
class JsonFormatter {
  public:
    template <typename ParseContext>
    constexpr typename ParseContext::iterator parse(ParseContext& a_context) {
        auto it = a_context.begin();

        if (it != a_context.end() && *it == '#') {
            m_pretty = true;
            a_context.advance_to(++it);
        }

        return m_text.parse(a_context);
    }

    template <typename FormatContext>
    typename FormatContext::iterator format(const T& a_value, FormatContext& a_context) const {
        std::ostringstream out;
        marshal_json(out, a_value, m_pretty);

        // Indented output is terminated by a newline, which does not belong in
        // the middle of a formatted string.
        std::string text = out.str();
        if (!text.empty() && text.back() == '\n') {
            text.pop_back();
        }

        return m_text.format(text, a_context);
    }

  private:
    std::formatter<std::string> m_text;
    bool m_pretty{false};
};

}  // namespace ic_cts
