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

#include <ostream>
#include <string>

#include "InterCOM/dyn_link.h"
#include "utils/XMLTypes.h"

namespace intercom {
//! Writes an %XML file
class INTERCOM_PUBLIC XMLWriter : public XML {
  public:
    //! Default Constructor
    XMLWriter() noexcept;
    //! Constructor taking XML-data to be written to file
    XMLWriter(const XMLElement& element) noexcept;
    //! Destructor
    virtual ~XMLWriter() noexcept;

    //! Returns the root %XML element
    XMLElement& rootElement() noexcept;

    //! Writes the content of current object to specified %string
    bool writeToString(std::string& str) noexcept;

    //! Writes the content of current object to specified %ostream
    bool writeToStream(std::ostream& os) noexcept;

    //! Writes the content of current object to specified %XML file
    bool writeToFile(const std::string& fileName) noexcept;

    //! Returns the indent for each level of %XML elements in the file.
    unsigned int indent() const noexcept;
    //! Sets the indent for each level of %XML elements in the file.
    void setIndent(unsigned int indent) noexcept;
    //! Returns the %XML version used
    std::string version() const noexcept;
    //! Sets the %XML version to use
    void setVersion(const std::string& newVersion) noexcept;

    //! Returns the %XML encoding used
    std::string encoding() const noexcept;
    //! Sets the %XML encoding to use
    void setEncoding(const std::string& newEncoding) noexcept;

  protected:
    bool writeElement(std::ostream& os, const XMLElement& element, int indent) const noexcept;
    bool writeOneElement(
        std::ostream& os,
        const std::string& name,
        const std::string& value,
        int indent
    ) const noexcept;
    static bool
    writeOneAttribute(std::ostream& os, const std::string& name, const std::string& value) noexcept;

  private:
    XMLElement* m_Root;
    int m_Indent;
    std::string m_Version;
    std::string m_Encoding;
};
}  // namespace intercom
