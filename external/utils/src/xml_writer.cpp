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

#include "utils/xml_writer.h"

#include <fstream>
#include <sstream>

using namespace intercom;

// =============================================================================
/*!
 * \class intercom::XMLWriter XMLWriter.h <Spider/Gear/Tools/XMLWriter.h>
 *
 * This class contains basic functionality that enables you to write an XML
 * hierarchy to an %XML file. You must start by retrieving the root
 * node using the rootElement() method and build your %XML hierarchy
 * by adding elements and attributes using the XMLElement::createMemberXMLElement()
 * and XMLElement::createMemberXMLAttribute() methods.
 *
 * Here's an example on how to use this class:
 * \include Ex_XMLWriter.cpp
 *
 * The %XML created in the example above would look like this:
 * \include Ex_XMLWriter.xml
 */

// ---------------------------------------------------------------------
XMLWriter::XMLWriter() noexcept
    : m_Root(new XMLElement()), m_Indent(2), m_Version("1.0"), m_Encoding("utf-8") {}

// ---------------------------------------------------------------------
/*!
 * \param element : The root node of the %XML hierarchy
 *
 * Creates an XMLWriter object using the given element as root.
 */
XMLWriter::XMLWriter(const XMLElement& element) noexcept
    : m_Root(new XMLElement(element)), m_Indent(2) {}

// ---------------------------------------------------------------------
XMLWriter::~XMLWriter() noexcept {
    if (m_Root) {
        delete m_Root;
        m_Root = nullptr;
    }
}

// ---------------------------------------------------------------------
/*!
 * \return The root element of the %XML hierarchy
 */
XMLElement& XMLWriter::rootElement() noexcept {
    return *m_Root;
}

// ---------------------------------------------------------------------
/*!
 * \return The indent used in the %XML file
 */
unsigned int XMLWriter::indent() const noexcept {
    return m_Indent;
}

// ---------------------------------------------------------------------
/*!
 * \param indent : The new indent used in the %XML file
 */
void XMLWriter::setIndent(unsigned int indent) noexcept {
    m_Indent = indent;
}

// ---------------------------------------------------------------------
/*!
 * \return The version used in the %XML file
 */
std::string XMLWriter::version() const noexcept {
    return m_Version;
}

// ---------------------------------------------------------------------
/*!
 * \param newVersion : The new version used in the %XML file
 *
 * If no specific version is set the default is 1.0
 */
void XMLWriter::setVersion(const std::string& new_version) noexcept {
    m_Version = new_version;
}

// ---------------------------------------------------------------------
/*!
 * \return The encoding used in the %XML file
 */
std::string XMLWriter::encoding() const noexcept {
    return m_Encoding;
}

// ---------------------------------------------------------------------
/*!
 * \param newEncoding : The new encoding used in the %XML file
 *
 * If no specific encoding is set the default is utf-8
 */
void XMLWriter::setEncoding(const std::string& new_encoding) noexcept {
    m_Encoding = new_encoding;
}

bool XMLWriter::writeToString(std::string& str) noexcept {
    if (!m_Root) {
        return false;
    }

    std::ostringstream os;
    writeToStream(os);
    str = os.str();
    return true;
}

// ---------------------------------------------------------------------
/*!
 * \param os : The stream to write the %XML hierarchy to.
 * \return True if file was successfully written, false otherwise.
 */
bool XMLWriter::writeToStream(std::ostream& os) noexcept {
    if (!m_Root) {
        return false;
    }

    os << "<?xml";
    if (!m_Version.empty()) {
        os << " version=\"" << m_Version << "\"";
    }
    if (!m_Encoding.empty()) {
        os << " encoding=\"" << m_Encoding << "\"";
    }
    os << "?>";
    return writeElement(os, *m_Root, 0);
}

// ---------------------------------------------------------------------
bool XMLWriter::writeElement(std::ostream& os, const XMLElement& element, int indent)
    const noexcept {
    std::string curIndent;
    for (int ii = 0; ii < indent * m_Indent; ii++) {
        curIndent += " ";
    }

    if (element.isValid()) {
        std::size_t numOfAttr(element.numberOfXMLAttributes());
        std::size_t numOfElem(element.numberOfXMLElements());

        if (numOfAttr > 0) {
            os << std::endl << curIndent << "<" << element.name();
            for (std::size_t i = 0; i < numOfAttr; i++) {
                const XMLAttribute& attr = element.xmlAttribute(i);
                writeOneAttribute(os, attr.name(), attr.value());
            }
            if (numOfElem > 0) {
                os << ">";
                for (std::size_t k = 0; k < numOfElem; k++) {
                    writeElement(os, element.xmlElement(k), indent + 1);
                }
                os << std::endl << curIndent << "</" << element.name() << ">";
            } else if (!element.value().empty()) {
                os << ">" << element.value() << "</" << element.name() << ">";
            } else {
                os << "/>";
            }
        } else {
            if (numOfElem > 0) {
                os << std::endl << curIndent << "<" << element.name() << ">";
                for (std::size_t k = 0; k < numOfElem; k++) {
                    writeElement(os, element.xmlElement(k), indent + 1);
                }
                os << std::endl << curIndent << "</" << element.name() << ">";
            } else {
                writeOneElement(os, element.name(), element.value(), indent);
            }
        }
    }
    return true;
}

// ---------------------------------------------------------------------
bool XMLWriter::writeOneElement(
    std::ostream& os,
    const std::string& name,
    const std::string& value,
    int indent
) const noexcept {
    std::string curIndent;
    for (int ii = 0; ii < indent * m_Indent; ii++) {
        curIndent += " ";
    }

    os << std::endl << curIndent << "<" << name << ">" << value << "</" << name << ">";
    return true;
}

// ---------------------------------------------------------------------
bool XMLWriter::writeOneAttribute(
    std::ostream& os,
    const std::string& name,
    const std::string& value
) noexcept {
    auto escape_str = [&](const std::string& str) {
        for (const auto& c : str) {
            switch (c) {
            case '&':
                os << "&amp;";
                break;
            case '"':
                os << "&quot;";
                break;
            case '\'':
                os << "&apos;";
                break;
            case '<':
                os << "&lt;";
                break;
            case '>':
                os << "&gt;";
                break;
            default:
                os << c;
                break;
            }
        }
    };

    os << " ";
    escape_str(name);
    os << "=\"";
    escape_str(value);
    os << "\"";
    return true;
}

// ---------------------------------------------------------------------
/*!
 * \param fileName : The name of the file to write the %XML hierarchy to.
 * \return True if file was successfully written, false otherwise.
 */
bool XMLWriter::writeToFile(const std::string& file_name) noexcept {
    if (!m_Root) {
        return false;
    }

    std::ofstream filestream(file_name.c_str(), std::ios_base::out | std::ios_base::trunc);
    writeToStream(filestream);
    return true;
}
