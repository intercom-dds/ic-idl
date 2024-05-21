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

#include <exception>
#include <string>
#include <utility>

#include "OptionImpl.h"

namespace intercom {
namespace cli {
class ExceptionBase : public std::exception {
  public:
    explicit ExceptionBase(detail::Command* context, std::string err) : m_cmd(context), m_err(std::move(err)) {}

    const detail::Command& command() const { return *m_cmd; }

    const char* what() const noexcept override { return m_err.c_str(); }

  private:
    detail::Command* m_cmd;
    const std::string m_err;
};

/// The provided value could not properly be converted to the requested type.
class InvalidValueException : public ExceptionBase {
  public:
    explicit InvalidValueException(const std::string& err) : ExceptionBase(nullptr, err) {}
};

/// User provided an unknown option.
class UnknownOptionException : public ExceptionBase {
  public:
    explicit UnknownOptionException(detail::Command* cmd, std::string opt, const std::string& err)
            : ExceptionBase(cmd, err), m_option(std::move(opt)) {}

    const std::string& option() const { return m_option; }

  private:
    std::string m_option;
};

/// User provided an unknown subcommand.
class UnknownSubcommand : public ExceptionBase {
  public:
    explicit UnknownSubcommand(detail::Command* cmd, std::string name, const std::string& err)
            : ExceptionBase(cmd, err), m_cmd(std::move(name)) {}

    const std::string& name() const { return m_cmd; }

  private:
    std::string m_cmd;
};

/// If any of the help tokens ever occur on the command line, the parser will return early
/// by throwing this exception. Happens regladless of the state of the parser.
class HelpException : public ExceptionBase {
  public:
    explicit HelpException(detail::Command* cmd) : ExceptionBase(cmd, {}) {}
};
}  // namespace cli
}  // namespace intercom
