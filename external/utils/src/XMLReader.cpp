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

#include "utils/XMLReader.h"

#include <algorithm>
#include <vector>

#include "InterCOM/string_view.h"
#include "utils/XMLReaderFile.h"
#include "utils/XMLTypes.h"

using namespace intercom;

// =============================================================================
/*!
 * \brief A type that provides a iterator that can read elements from an %XML file.
 *
 * The %Iterator object is utilized to navigate in the %XML element tree and
 * for accessing %XML element data.
 */

// ---------------------------------------------------------------------
XMLReader::Iterator::Iterator() noexcept : m_XMLElement(nullptr), m_XMLElementIndex(0) {}
// ---------------------------------------------------------------------
/*!
 * \param element : A %XML element object.
 *
 * Construct an iterator object pointing to specified %XML element object.
 */
XMLReader::Iterator::Iterator(const XMLElement& element) noexcept : m_XMLElement(nullptr), m_XMLElementIndex(0) {
    if (element.isValid()) {
        m_XMLElement = const_cast<XMLElement*>(&element);
    }
}
// ---------------------------------------------------------------------
/*!
 * \param rhs : An iterator object.
 */
XMLReader::Iterator::Iterator(const Iterator&) noexcept = default;
// ---------------------------------------------------------------------

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right hand side %Iterator object.
 */
bool XMLReader::Iterator::operator!=(const Iterator& rhs) const noexcept {
    return element() != rhs.element();
}
// ---------------------------------------------------------------------
/*!
 * \param rhs : Right hand side %Iterator object.
 */
bool XMLReader::Iterator::operator==(const Iterator& rhs) const noexcept {
    return element() == rhs.element();
}
// ---------------------------------------------------------------------
/*!
 * \param rhs : Right hand side %Iterator object.
 */
XMLReader::Iterator& XMLReader::Iterator::operator=(const Iterator&) noexcept = default;
// ---------------------------------------------------------------------
/*!
 * \return A reference to a %XMLElement object.
 */
const XMLElement& XMLReader::Iterator::operator*() const noexcept {
    const XMLElement* curElem = element();
    return curElem ? *curElem : XMLElement::invalid();
}
// ---------------------------------------------------------------------
/*!
 * \return A pointer to a %XMLElement object.
 */
const XMLElement* XMLReader::Iterator::operator->() const noexcept {
    return element();
}
// ---------------------------------------------------------------------
XMLReader::Iterator& XMLReader::Iterator::operator++() noexcept {
    if (m_XMLElement) {
        const XMLElement& parentElem = m_XMLElement->parentXMLElement();

        if (!parentElem.isValid())  // Root node
        {
            if (m_XMLElementIndex < m_XMLElement->numberOfXMLElements() - 1) {
                m_XMLElementIndex++;
                m_XMLElement = &m_XMLElement->xmlElement(m_XMLElementIndex);
            }
        } else if (m_XMLElementIndex < parentElem.numberOfXMLElements() - 1) {
            m_XMLElementIndex++;
            m_XMLElement = const_cast<XMLElement*>(&parentElem.xmlElement(m_XMLElementIndex));
        } else {
            m_XMLElement = nullptr;
            m_XMLElementIndex = 0;
        }
    }

    return (*this);
}
// ---------------------------------------------------------------------
XMLReader::Iterator XMLReader::Iterator::operator++(int) noexcept {
    XMLReader::Iterator tmp(*this);
    ++*this;
    return tmp;
}
// ---------------------------------------------------------------------
XMLReader::Iterator& XMLReader::Iterator::operator--() noexcept {
    if (m_XMLElement) {
        const XMLElement& parentElem = m_XMLElement->parentXMLElement();
        if (parentElem.isValid()) {
            if (m_XMLElementIndex > 0) {
                m_XMLElementIndex--;
                m_XMLElement = const_cast<XMLElement*>(&parentElem.xmlElement(m_XMLElementIndex));
            }
        }
    }
    return (*this);
}
// ---------------------------------------------------------------------
XMLReader::Iterator XMLReader::Iterator::operator--(int) noexcept {
    XMLReader::Iterator tmp(*this);
    --*this;
    return tmp;
}
// ---------------------------------------------------------------------
const XMLElement* XMLReader::Iterator::element() const noexcept {
    if (m_XMLElement) {
        const XMLElement& parentElem = m_XMLElement->parentXMLElement();
        if (parentElem.isValid()) {
            if (m_XMLElementIndex < parentElem.numberOfXMLElements()) {
                return m_XMLElement;
            }
        } else {
            return m_XMLElement;
        }
    }
    return &XMLElement::invalid();
}

// ---------------------------------------------------------------------
/*!
 * The default constructor creates an invalid object.
 */
XMLReader::XMLReader() : m_Root(nullptr), m_Valid(false), m_Verbose(false), m_VerboseStream(nullptr) {}
/*!
 * Copy constructor.
 */
XMLReader::XMLReader(const XMLReader& rhs)
        : m_Root(new XMLElement(*rhs.m_Root)),
          m_Valid(rhs.m_Valid),
          m_Verbose(rhs.m_Verbose),
          m_VerboseStream(rhs.m_VerboseStream),
          m_FileName(rhs.m_FileName) {}
// ---------------------------------------------------------------------
/*!
 * \param fileName : An %XML file name.
 * \param verbose  : Whether to print verbose information when parsing or not,
 *                   defaults to false.
 * \param targetStream : The stream you want verbose information to be written to,
 *                       defaults to std::cout.
 *
 * Reads specified %XML file into objects of %XML elements and %XML attributes.
 * If any syntax errors is found, the file reading is terminated and the
 * %XMLReader object is set to invalid.
 *
 * An %XMLReader in verbose mode will print information while parsing an
 * %XML file to the given ostream. The %XMLReader will print the %XML as it is
 * being parsed, but it will skip blank lines and comments. The printed indentation
 * is used to indicate the hierarchy in the XML, it doesn't necessarily follow
 * the indentation of the parsed %XML file.
 */
XMLReader::XMLReader(const std::string& file_name, bool verbose, std::ostream& target_stream)
        : m_Root(nullptr), m_Valid(false), m_Verbose(verbose), m_VerboseStream(&target_stream), m_FileName(file_name) {
    // Read updates the valid flag
    read(file_name);
}
/*!
 * \param fileName : An %XML file name.
 * \param verbose  : Whether to print verbose information when parsing or not,
 *                   defaults to false.
 * \param targetStream : The stream you want verbose information to be written to,
 *                       defaults to std::cout.
 *
 * Reads specified %stringstream into objects of %XML elements and %XML attributes.
 * If any syntax errors is found, the file reading is terminated and the
 * %XMLReader object is set to invalid.
 *
 * An %XMLReader in verbose mode will print information while parsing an
 * %XML file to the given ostream. The %XMLReader will print the %XML as it is
 * being parsed, but it will skip blank lines and comments. The printed indentation
 * is used to indicate the hierarchy in the XML, it doesn't necessarily follow
 * the indentation of the parsed %XML file.
 */
XMLReader::XMLReader(const std::stringstream& input_stream, bool verbose, std::ostream& target_stream)
        : m_Root(nullptr), m_Valid(false), m_Verbose(verbose), m_VerboseStream(&target_stream) {
    // Read updates the valid flag
    read(input_stream);
}

// ---------------------------------------------------------------------
/*!
 * Copy assignment operator.
 */
XMLReader& XMLReader::operator=(const XMLReader& rhs) {
    *this = XMLReader(rhs);
    return *this;
}

// ---------------------------------------------------------------------
/*!
 * \return True when the %XMLReader object is valid.
 *
 * When the %XMLReader object is invalid, it means either the specified
 * file was not found, or a syntax error was found during reading of the
 * %XML file. Use an %XML editor tool to check and correct the syntax of the %XML file.
 */
bool XMLReader::isValid() const {
    return m_Valid;
}

// ---------------------------------------------------------------------
/*!
 * \return The filename passed to this XMLReader)
 *
 * If the returned filename is empty, this XMLReader or the last read-operation was initialized with a stringstream.
 */
const std::string& XMLReader::getFileName() const {
    return m_FileName;
}

// ---------------------------------------------------------------------
/*!
 * \param verbose : The new verbose state for this %XMLReader, defaults to true.
 * \param targetStream : The stream you want verbose information to be written to,
 *                       defaults to std::cout.
 *
 * An %XMLReader in verbose mode will print information while parsing an
 * %XML file to the given ostream. The %XMLReader will print the %XML as it is
 * being parsed, but it will skip blank lines and comments. The printed indentation
 * is used to indicate the hierarchy in the XML, it doesn't necessarily follow
 * the indentation of the parsed %XML file.
 */
void XMLReader::setVerbose(bool verbose, std::ostream& target_stream) noexcept {
    m_Verbose = verbose;
    m_VerboseStream = &target_stream;
}

// ---------------------------------------------------------------------
/*!
 * \return True if this %XMLReader is in verbose mode.
 *
 */
bool XMLReader::isVerbose() const {
    return m_Verbose;
}

// ---------------------------------------------------------------------
/*!
 * \return An %Iterator object
 */
XMLReader::Iterator XMLReader::begin() const noexcept {
    Iterator result;
    if (m_Valid) {
        if ((m_Root) && (m_Root->numberOfXMLElements() > 0)) {
            result.m_XMLElement = const_cast<XMLElement*>(m_Root.get());
            ;
        }
    }
    return result;
}
// ---------------------------------------------------------------------
/*!
 * \return An %Iterator object
 */
XMLReader::Iterator XMLReader::end() noexcept {
    return Iterator(XMLElement::invalid());
}
// ---------------------------------------------------------------------
/*!
 * \param iter : An %Iterator object.
 * \return An %Iterator object.
 *
 * The returned %Iterator object is the first child element of the element
 * object kept by this %Iterator.
 */
XMLReader::Iterator XMLReader::begin(const Iterator& iter) const noexcept {
    Iterator result;
    if ((m_Valid) && (iter.m_XMLElement)) {
        if (!iter->m_XMLElements.empty()) {
            result.m_XMLElement = iter->m_XMLElements[0];
            result.m_XMLElementIndex = 0;
        }
    }
    return result;
}
// ---------------------------------------------------------------------
/*!
 * \param iter : An %Iterator object.
 * \return An %Iterator object.
 *
 * The returned %Iterator object is invalid and pointing behind the last
 * child element of the element object kept by this %Iterator.
 */
XMLReader::Iterator XMLReader::end(const Iterator& iter) const noexcept {
    Iterator result;
    if ((m_Valid) && (iter.m_XMLElement)) {
        if (!iter->m_XMLElements.empty()) {
            result.m_XMLElementIndex = static_cast<unsigned long>(iter->m_XMLElements.size());
            result.m_XMLElement = iter->m_XMLElements[result.m_XMLElementIndex - 1];
        }
    }
    return result;
}
// ---------------------------------------------------------------------
/*!
 * \param elementName : A name of %XML element.
 * \param caseSensitive : Whether name of element should be case sensitive or not
 * \return An %Iterator object.
 *
 * Find and return the first %XML element matching the specified name.
 * When no element object was found, the return %Iterator object is invalid.
 * The elementName parameter can contain names for several %XML elements.
 * Each name must be separated via a '/' and must reflecting the node
 * structure from the %XML element tree. (Element1/Element2/Element3)
 * The find method starts the search from the root element node.
 */
XMLReader::Iterator XMLReader::find(const std::string& element_name, bool case_sensitive) const noexcept {
    return find(Iterator(*m_Root), element_name, case_sensitive);
}
// ---------------------------------------------------------------------
/*!
 * \param iter : %Iterator pointing to the start element for the search.
 * \param elementName : A name of %XML element.
 * \param caseSensitive : Whether name of element should be case sensitive or not
 * \return An %Iterator object.
 *
 * See the method find(elementName) for description.
 */
XMLReader::Iterator XMLReader::find(const Iterator& iter, const std::string& element_name,
                                    bool case_sensitive) const noexcept {
    Iterator result;
    if (m_Valid) {
        std::vector<std::string> names;
        if (decodeName(element_name, names)) {
            const XMLElement* element = iter->doFind(names[0], case_sensitive);
            if (element) {
                for (unsigned long i = 1; i < names.size(); i++) {
                    element = element->doFind(names[i], case_sensitive);
                    if (!element) {
                        break;
                    }
                }
            }
            if (element) {
                result.m_XMLElementIndex = 0;
                result.m_XMLElement = const_cast<XMLElement*>(element);
            }
        }
    }
    return result;
}

XMLReader::ReadStatus XMLReader::read(XMLReaderFile& xml_file) noexcept {
    // Cleanup old configuration
    m_Valid = false;
    if (m_Root) {
        m_Root.reset();
    }
    ReadStatus result = READ_FILE_NOT_FOUND;

    if (xml_file.isValid()) {
        int indent = 0;
        if (m_Verbose && m_VerboseStream) {
            printVerbose("XMLReader start parsing file.", "", "\n");
        }

        std::vector<XMLElement*> parentElements;
        std::vector<XMLElement*> elements;
        bool error = false;
        bool reading = true;
        while (reading) {
            XMLReaderFile::XMLDataType dataType;
            XMLReaderFile::XMLState state;
            const char* nextWord = xml_file.readNext(state, dataType);
            switch (state) {
            case XMLReaderFile::STATE_UNKNOWN: {
                reading = false;
                error = true;
                if (m_Verbose && m_VerboseStream) {
                    printVerbose("\n", "ERROR: Unknown reading state", "\n");
                }
            } break;
            // New element detected
            case XMLReaderFile::STATE_NEW_ELEMENT: {
                if (m_Verbose && m_VerboseStream) {
                    std::string info = "<";
                    info += nextWord;
                    if (m_Root) {
                        printVerbose(">\n", info, "", indent++);
                    } else {
                        printVerbose("", info, "", indent++);
                    }
                }
                XMLElement* parent = nullptr;
                if (!parentElements.empty()) {
                    parent = parentElements[parentElements.size() - 1];
                } else if ((parentElements.empty()) && m_Root) {
                    // This element would have no parent, something is wrong.
                    // We have to terminate and report the read error or we'll
                    // end up with potential memory leaks.
                    error = true;
                    reading = false;
                    if (m_Verbose && m_VerboseStream) {
                        printVerbose("\n", "ERROR: Orphan element", "\n");
                    }
                    break;
                }
                auto* element = new XMLElement(std::string(nextWord));
                element->m_Line = xml_file.currentLine();
                element->m_ParentElement = parent;
                elements.push_back(element);
                parentElements.push_back(element);
                if (!m_Root) {
                    m_Root.reset(element);
                }
                if (parent) {
                    parent->m_XMLElements.push_back(element);
                }
            } break;
            // New attribute detected
            case XMLReaderFile::STATE_NEW_ATTRIBUTE: {
                if (m_Verbose && m_VerboseStream) {
                    printVerbose(" ", nextWord, "");
                }
                if (!elements.empty()) {
                    XMLElement* curElement = elements[elements.size() - 1];
                    if (curElement) {
                        auto* attr = new XMLAttribute(std::string(nextWord));
                        curElement->m_XMLAttributes.push_back(attr);
                    } else {
                        error = true;
                        reading = false;
                        if (m_Verbose && m_VerboseStream) {
                            printVerbose("\n", "ERROR: 0-element to put attribute in", "\n");
                        }
                        break;
                    }
                } else {
                    error = true;
                    reading = false;
                    if (m_Verbose && m_VerboseStream) {
                        printVerbose("\n", "ERROR: No element to put attribute in", "\n");
                    }
                    break;
                }
            } break;
            case XMLReaderFile::STATE_ATTRIBUTE_DATA: {
                if (m_Verbose && m_VerboseStream) {
                    printVerbose("=\"", nextWord, "\"");
                }
                if (!elements.empty()) {
                    XMLElement* curElement = elements[elements.size() - 1];
                    if (curElement) {
                        if (!curElement->m_XMLAttributes.empty()) {
                            XMLAttribute* attr = curElement->m_XMLAttributes[curElement->m_XMLAttributes.size() - 1];
                            if (attr) {
                                attr->m_XMLDataType = static_cast<XMLDataType>(dataType);
                                attr->m_Value = std::string(nextWord);
                            } else {
                                error = true;
                                reading = false;
                                if (m_Verbose && m_VerboseStream) {
                                    printVerbose("\n", "ERROR: 0-attribute to put attribute data in", "\n");
                                }
                                break;
                            }
                        } else {
                            error = true;
                            reading = false;
                            if (m_Verbose && m_VerboseStream) {
                                printVerbose("\n", "ERROR: No attribute to put attribute data in", "\n");
                            }
                            break;
                        }
                    } else {
                        error = true;
                        reading = false;
                        if (m_Verbose && m_VerboseStream) {
                            printVerbose("\n", "ERROR: 0-element to put attribute data in", "\n");
                        }
                        break;
                    }
                } else {
                    error = true;
                    reading = false;
                    if (m_Verbose && m_VerboseStream) {
                        printVerbose("\n", "ERROR: No element to put attribute data in", "\n");
                    }
                    break;
                }
            } break;
            case XMLReaderFile::STATE_ELEMENT_DATA: {
                if (m_Verbose && m_VerboseStream) {
                    printVerbose(" \"", nextWord, "\"");
                }
                if (!elements.empty()) {
                    XMLElement* curElement = elements[elements.size() - 1];
                    if (curElement) {
                        curElement->m_XMLDataType = static_cast<XMLDataType>(dataType);
                        curElement->m_Value = std::string(nextWord);
                    } else {
                        error = true;
                        reading = false;
                        if (m_Verbose && m_VerboseStream) {
                            printVerbose("\n", "ERROR: 0-element to put element data in", "\n");
                        }
                        break;
                    }
                } else {
                    error = true;
                    reading = false;
                    if (m_Verbose && m_VerboseStream) {
                        printVerbose("\n", "ERROR: No element to put element data in", "\n");
                    }
                    break;
                }
            } break;
            case XMLReaderFile::STATE_EMPTY: {
            } break;
            case XMLReaderFile::STATE_COMMENT: {
            } break;
            case XMLReaderFile::STATE_CONTROL: {
            } break;
            case XMLReaderFile::STATE_END_ELEMENT: {
                if (m_Verbose && m_VerboseStream) {
                    if (std::string(nextWord) == "/") {
                        printVerbose(" ", nextWord, "");
                        --indent;
                    } else {
                        std::string endTag = "</";
                        endTag += nextWord;
                        printVerbose(">\n", endTag, "", --indent);
                    }
                }
                if (!elements.empty()) {
                    elements.pop_back();
                    parentElements.pop_back();
                } else {
                    error = true;
                    reading = false;
                    if (m_Verbose && m_VerboseStream) {
                        printVerbose("\n", "ERROR: No element corresponding to end-element", "\n");
                    }
                    break;
                }
            } break;
            case XMLReaderFile::STATE_END_OF_FILE: {
                reading = false;
            } break;
            default: {
                reading = false;
            } break;
            }
        }
        if (!error) {
            result = XMLReader::READ_SUCCESS;
            m_Valid = true;
        } else {
            result = XMLReader::READ_PARSE_ERROR;
        }
        if (m_Verbose && m_VerboseStream) {
            switch (result) {
            case XMLReader::READ_FILE_NOT_FOUND:
                printVerbose(">\nXMLReader finished parsing, result is: READ_FILE_NOT_FOUND\n", "", "");
                break;
            case XMLReader::READ_SUCCESS:
                printVerbose(">\nXMLReader finished parsing, result is: READ_SUCCESS\n", "", "");
                break;
            case XMLReader::READ_PARSE_ERROR:
                printVerbose(">\nXMLReader finished parsing, result is: READ_PARSE_ERROR\n", "", "");
                break;
            default:
                printVerbose(">\nXMLReader finished parsing, result is: Unknown\n", "", "");
                break;
            }
        }
    }
    return result;
}

// ---------------------------------------------------------------------
/*!
 * \param fileName : A %stringstream containing the %XML
 *
 * Reads specified %XML from stream into objects of %XML elements and %XML attributes.
 * If any syntax errors is found, the file reading is terminated and the
 * %XMLReader object is set to invalid.
 */
XMLReader::ReadStatus XMLReader::read(const std::stringstream& ss) noexcept {
    m_FileName = "";
    XMLReaderFile xmlFile(ss);
    return read(xmlFile);
}

// ---------------------------------------------------------------------
/*!
 * \param fileName : A %XML file name.
 *
 * Reads specified %XML file into objects of %XML elements and %XML attributes.
 * If any syntax errors is found, the file reading is terminated and the
 * %XMLReader object is set to invalid.
 */
XMLReader::ReadStatus XMLReader::read(const std::string& file_name) noexcept {
    m_FileName = file_name;
    XMLReaderFile xmlFile(m_FileName.c_str());
    return read(xmlFile);
}

// ---------------------------------------------------------------------
// Private
void XMLReader::printVerbose(const std::string& before, const std::string& info, const std::string& after,
                             int indent) noexcept {
    *m_VerboseStream << before;
    for (int i = 0; i < indent; ++i) {
        *m_VerboseStream << " ";
    }
    *m_VerboseStream << info << after;
}
