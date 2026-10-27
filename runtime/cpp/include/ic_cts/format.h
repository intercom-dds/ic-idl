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

#include "ic_cts/debug_serializer.h"

namespace ic_cts {

/// Base class for `std::formatter` specializations that render a value as debug output.
///
/// A leading `#` in the format specification selects indented debug output. The
/// remainder is parsed as a standard string format specification, which
/// supports fill, alignment, width and precision.
template <typename T>
class DebugFormatter {
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
        marshal_debug(out, a_value, m_pretty);
        return m_text.format(out.str(), a_context);
    }

  private:
    std::formatter<std::string> m_text;
    bool m_pretty{false};
};

}  // namespace ic_cts
