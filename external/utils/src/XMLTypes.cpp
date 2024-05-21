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

#include "utils/XMLTypes.h"

#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "InterCOM/IntegerTypes.h"
#include "InterCOM/string_view.h"
#include "utils/string_utils.h"

using namespace intercom;

#include <cctype>
#include <utility>

using std::toupper;

// =============================================================================
/*!
 * \brief A class that provides the %XML element information.
 *
 * The %XMLElement object is a placeholder for %XML %Element data.
 */

const XMLElement XMLElement::m_Invalid = XMLElement();
const XMLAttribute XMLAttribute::m_Invalid = XMLAttribute();

// ---------------------------------------------------------------------
XMLAttribute::XMLAttribute(std::string name) noexcept : m_Name(std::move(name)), m_XMLDataType(XML_DATA_TYPE_UNKNOWN) {}
// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the operation
 */
// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the assignment.
 * \return A reference to this object
 */
XMLAttribute& XMLAttribute::operator=(const XMLAttribute& rhs) noexcept {
    if (this != &rhs) {
        m_Name = rhs.m_Name;
        m_Value = rhs.m_Value;
        m_XMLDataType = rhs.m_XMLDataType;
    }
    return *this;
}

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the comparison.
 * \return True if the compared attributes are equal, false otherwise.
 */
bool XMLAttribute::operator==(const XMLAttribute& rhs) const noexcept {
    return (m_XMLDataType == rhs.m_XMLDataType) && (m_Name == rhs.m_Name) && (m_Value == rhs.m_Value);
}

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the comparison.
 * \return True if the compared attributes are inequal, false otherwise.
 */
bool XMLAttribute::operator!=(const XMLAttribute& rhs) const noexcept {
    return !(operator==(rhs));
}

// ---------------------------------------------------------------------
/*!
 * \return True when the %XMLAttribute object is valid.
 */
bool XMLAttribute::isValid() const noexcept {
    return this != &XMLAttribute::m_Invalid;
}
// ---------------------------------------------------------------------
/*!
 * \return An invalid %XMLAttribute object.
 */
const XMLAttribute& XMLAttribute::invalid() noexcept {
    return XMLAttribute::m_Invalid;
}
// ---------------------------------------------------------------------
/*!
 * \return The name of current %XMLAttribute object.
 */
const std::string& XMLAttribute::name() const noexcept {
    return m_Name;
}
// ---------------------------------------------------------------------
/*!
 * \return The value of current %XMLAttribute object.
 */
const std::string& XMLAttribute::value() const noexcept {
    return m_Value;
}
// ---------------------------------------------------------------------
/*!
 * \return The datatype of the value for current %XMLAttribute object.
 */
XMLDataType XMLAttribute::xmlDataType() const noexcept {
    return m_XMLDataType;
}
// ---------------------------------------------------------------------
/*!
 * \param name : The name of current object.
 */
void XMLAttribute::setName(const std::string& name) noexcept {
    m_Name = name;
}

// ---------------------------------------------------------------------
/*!
 * \param value : The value of current object.
 */
void XMLAttribute::setValue(const std::string& value) noexcept {
    m_Value = value;
}

// ---------------------------------------------------------------------
/*!
 * \param type : The datatype of current object.
 */
void XMLAttribute::setXMLDataType(const XMLDataType type) noexcept {
    m_XMLDataType = type;
}

// ---------------------------------------------------------------------
/*!
 * \param name  : The name of current object.
 * \param value : The value of current object.
 * \param type  : The datatype of current object.
 */
void XMLAttribute::set(const std::string& name, const std::string& value, const XMLDataType type) noexcept {
    m_Name = name;
    m_Value = value;
    m_XMLDataType = type;
}

// ---------------------------------------------------------------------
// ---------------------------------------------------------------------
// ---------------------------------------------------------------------
/*!
 * \param name : The name of constructed %XMLElement object.
 */
XMLElement::XMLElement(std::string name) noexcept
        : m_Name(std::move(name)), m_XMLDataType(XML_DATA_TYPE_UNKNOWN), m_ParentElement(nullptr) {}
// ---------------------------------------------------------------------
/*!
 * \param name   : The name of current object.
 * \param value  : The value of current object.
 * \param parent : The parent of current object.
 */
XMLElement::XMLElement(std::string name, std::string value, XMLElement* parent) noexcept
        : m_Name(std::move(name)),
          m_Value(std::move(value)),
          m_XMLDataType(XML_DATA_TYPE_UNKNOWN),
          m_ParentElement(parent) {}
// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the operation.
 */
XMLElement::XMLElement(const XMLElement& rhs) noexcept
        : m_Name(rhs.m_Name), m_Value(rhs.m_Value), m_XMLDataType(rhs.m_XMLDataType), m_ParentElement(nullptr) {
    for (auto element : rhs.m_XMLElements) {
        if (element) {
            auto* copy = new XMLElement(*element);
            m_XMLElements.push_back(copy);
            copy->m_ParentElement = this;
        }
    }
    for (auto attribute : rhs.m_XMLAttributes) {
        if (attribute) {
            m_XMLAttributes.push_back(new XMLAttribute(*attribute));
        }
    }
}

// ---------------------------------------------------------------------
XMLElement::~XMLElement() noexcept {
    for (auto element : m_XMLElements) {
        delete element;
    }
    for (auto attribute : m_XMLAttributes) {
        delete attribute;
    }
}

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the assignment.
 * \return A reference to this object
 */
XMLElement& XMLElement::operator=(const XMLElement& rhs) noexcept {
    if (this != &rhs) {
        m_Name = rhs.m_Name;
        m_Value = rhs.m_Value;
        m_XMLDataType = rhs.m_XMLDataType;
        m_ParentElement = rhs.m_ParentElement;
        // Delete old elements
        for (auto element : m_XMLElements) {
            delete element;
        }
        m_XMLElements.clear();
        // Copy new elements
        for (auto element : rhs.m_XMLElements) {
            if (element) {
                auto* copy = new XMLElement(*element);
                m_XMLElements.push_back(copy);
                copy->m_ParentElement = this;
            }
        }
        // Delete old attributes
        for (auto attribute : m_XMLAttributes) {
            delete attribute;
        }
        m_XMLAttributes.clear();
        // Copy new attributes
        for (auto attribute : rhs.m_XMLAttributes) {
            if (attribute) {
                m_XMLAttributes.push_back(new XMLAttribute(*attribute));
            }
        }
    }
    return *this;
}

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the comparison.
 * \return True if the compared elements are equal, false otherwise.
 *
 * This comparison operator compares this element and all attributes
 * and contained elements. All must be equal in order for this method
 * to return true.
 */
bool XMLElement::operator==(const XMLElement& rhs) const noexcept {
    if ((m_XMLDataType == rhs.m_XMLDataType) && (m_Name == rhs.m_Name) && (m_Value == rhs.m_Value) &&
        (m_XMLElements.size() == rhs.m_XMLElements.size()) && (m_XMLAttributes.size() == rhs.m_XMLAttributes.size())) {
        for (std::size_t i = 0; i < m_XMLElements.size(); ++i) {
            if (*m_XMLElements[i] != *rhs.m_XMLElements[i]) {
                return false;
            }
        }
        for (std::size_t j = 0; j < m_XMLAttributes.size(); ++j) {
            if (*m_XMLAttributes[j] != *rhs.m_XMLAttributes[j]) {
                return false;
            }
        }
        return true;
    }
    return false;
}

// ---------------------------------------------------------------------
/*!
 * \param rhs : Right-hand side of the comparison.
 * \return True if the compared elements are inequal, false otherwise.
 *
 * This comparison operator compares this element and all attributes
 * and contained elements. All must be inequal in order for this method
 * to return false.
 */
bool XMLElement::operator!=(const XMLElement& rhs) const noexcept {
    return !(operator==(rhs));
}

// ---------------------------------------------------------------------
/*!
 * \return True when the %XMLElement object is valid.
 */
bool XMLElement::isValid() const noexcept {
    return *this != XMLElement::m_Invalid;
}
// ---------------------------------------------------------------------
/*!
 * \return An invalid %XMLElement object.
 */
const XMLElement& XMLElement::invalid() noexcept {
    return XMLElement::m_Invalid;
}

// ---------------------------------------------------------------------
/*!
 * \return The name of current %XMLElement object.
 */
const std::string& XMLElement::name() const noexcept {
    return m_Name;
}
// ---------------------------------------------------------------------
/*!
 * \return The value of current %XMLElement object.
 */
const std::string& XMLElement::value() const noexcept {
    return m_Value;
}
// ---------------------------------------------------------------------
/*!
 * \return The datatype of the value for current %XMLElement object.
 */
XMLDataType XMLElement::xmlDataType() const noexcept {
    return m_XMLDataType;
}
// ---------------------------------------------------------------------
/*!
 * \param name : The name of current object.
 */
void XMLElement::setName(const std::string& name) noexcept {
    m_Name = name;
}

// ---------------------------------------------------------------------
/*!
 * \param value : The value of current object.
 */
void XMLElement::setValue(const std::string& value) noexcept {
    m_Value = value;
}

// ---------------------------------------------------------------------
/*!
 * \param type : The datatype of current object.
 */
void XMLElement::setXMLDataType(const XMLDataType type) noexcept {
    m_XMLDataType = type;
}

// ---------------------------------------------------------------------
/*!
 * \param name  : The name of current object.
 * \param value : The value of current object.
 * \param type  : The datatype of current object.
 */
void XMLElement::set(const std::string& name, const std::string& value, const XMLDataType type) noexcept {
    m_Name = name;
    m_Value = value;
    m_XMLDataType = type;
}

// ---------------------------------------------------------------------
/*!
 * \return A long representing the number of %XMLElement member objects.
 */
std::size_t XMLElement::numberOfXMLElements() const noexcept {
    return m_XMLElements.size();
}
// ---------------------------------------------------------------------
/*!
 * \param index : An index representing the identification of a %XMLElement member object.
 * \return A %XMLElement object.
 */
const XMLElement& XMLElement::xmlElement(std::size_t index) const& noexcept {
    if (index < m_XMLElements.size()) {
        return *m_XMLElements[index];
    }
    return XMLElement::m_Invalid;
}

XMLElement& XMLElement::xmlElement(std::size_t index) & noexcept {
    return *m_XMLElements[index];
}

XMLElement&& XMLElement::xmlElement(std::size_t index) && noexcept {
    return std::move(*m_XMLElements[index]);
}
// ---------------------------------------------------------------------
/*!
 * \return A long representing the number of related %XMLAttribute objects.
 */
std::size_t XMLElement::numberOfXMLAttributes() const noexcept {
    return m_XMLAttributes.size();
}
// ---------------------------------------------------------------------
/*!
 * \param index : An index representing the identification of a related %XMLAttribute object.
 * \return A %XMLAttribute object.
 */
const XMLAttribute& XMLElement::xmlAttribute(std::size_t index) const noexcept {
    if (index < m_XMLAttributes.size()) {
        return *m_XMLAttributes[index];
    }
    return XMLAttribute::m_Invalid;
}

// ---------------------------------------------------------------------
/*!
 * \param name : Name of a related %XMLAttribute object.
 * \return A %XMLAttribute object.
 *
 * When a related %XMLAttribute object with specified name does not exists, an
 * invalid %XMLAttribute object will be returned.
 */
const XMLAttribute& XMLElement::xmlAttribute(const std::string& name) const noexcept {
    for (auto attribute : m_XMLAttributes) {
        if (attribute) {
            if (attribute->m_Name == name) {
                return *attribute;
            }
        }
    }
    return XMLAttribute::m_Invalid;
}

// ---------------------------------------------------------------------
/*!
 * \return A new %XMLElement object.
 *
 * To be used for creation of a member %XMLElement.
 */
XMLElement& XMLElement::createMemberXMLElement() noexcept {
    auto* elem = new XMLElement();
    m_XMLElements.push_back(elem);
    elem->m_ParentElement = this;
    return *elem;
}

// ---------------------------------------------------------------------
/*!
 * \return A %XMLAttribute object.
 *
 * To be used for creation of a member %XMLAttribute.
 */
XMLAttribute& XMLElement::createMemberXMLAttribute() noexcept {
    auto* attr = new XMLAttribute();
    m_XMLAttributes.push_back(attr);
    return *attr;
}
// ---------------------------------------------------------------------
/*!
 * \param elementName : Name of a related %XMLElement object.
 * \param caseSensitive : Whether name of element should be case sensitive or not
 * \return A %XMLElement object.
 *
 * When a member %XMLElement object with specified name does not exists, an
 * invalid %XMLElement object will be returned.
 * The elementName parameter can contain names for several %XML elements.
 * Each name must be separated via a '/' and must reflecting the node
 * structure from the %XML element tree. (Element1/Element2/Element3)
 */
const XMLElement& XMLElement::find(const std::string& element_name, bool case_sensitive) const noexcept {
    std::vector<std::string> names;
    if (XML::decodeName(element_name, names)) {
        const XMLElement* element = this;
        for (auto& name : names) {
            element = element->doFind(name, case_sensitive);
            if (!element) {
                break;
            }
        }
        if (element) {
            return *element;
        }
    }
    return XMLElement::m_Invalid;
}

// ---------------------------------------------------------------------
/*!
 * \return The parent %XMLElement object.
 *
 * When no parent %XMLElement object is found, an invalid %XMLElement
 * object will be returned.
 */
const XMLElement& XMLElement::parentXMLElement() const noexcept {
    return (m_ParentElement) ? *m_ParentElement : XMLElement::m_Invalid;
}

// ---------------------------------------------------------------------
const XMLElement* XMLElement::doFind(const std::string& one_element_name, bool case_sensitive) const noexcept {
    const XMLElement* result = nullptr;
    std::string elementName = m_Name;
    std::string searchName = one_element_name;

    if (!case_sensitive) {
        XML::makeUpper(elementName);
        XML::makeUpper(searchName);
    }

    if (elementName == searchName) {
        result = this;
    } else {
        for (auto element : m_XMLElements) {
            if (element) {
                elementName = element->m_Name;

                if (!case_sensitive) {
                    XML::makeUpper(elementName);
                }

                if (elementName == searchName) {
                    result = element;
                    break;
                }
            }
        }
    }
    return result;
}

// ---------------------------------------------------------------------
XML::XML() noexcept = default;
// ---------------------------------------------------------------------

// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to bool type.
 * \return True when the convertion was executed successfully.
 */
bool XML::convertValueToBool(const std::string& source, bool& value) noexcept {
    value = false;
    bool result = false;
    auto numOfChar = static_cast<unsigned long>(source.size());
    if (numOfChar > 0) {
        if (numOfChar == 1) {
            if (source[0] == '1') {
                value = true;
                result = true;
            } else if (source[0] == '0') {
                value = false;
                result = true;
            }
        } else {
            if ((numOfChar == 4) || (numOfChar == 5)) {
                std::string tmpString(source);
                XML::makeUpper(tmpString);
                if (tmpString == std::string("TRUE")) {
                    value = true;
                    result = true;
                } else if (tmpString == std::string("FALSE")) {
                    value = false;
                    result = true;
                }
            }
        }
    }
    return result;
}

// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to double type.
 * \return True when the convertion was executed successfully.
 */
bool XML::convertValueToDouble(const std::string& source, double& value) noexcept {
    value = 0.0;
    if (!source.empty()) {
        if (checkNumber(source.c_str(), true)) {
            return StringUtils::stringToDouble(source.c_str(), value);
        }
    }
    return false;
}
// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to long type.
 * \return True when the convertion was executed successfully.
 */
bool XML::convertValueToLong(const std::string& source, long& value) noexcept {
    value = 0;
    if (!source.empty()) {
        if (checkNumber(source.c_str(), false)) {
            value = static_cast<long>(atol(source.c_str()));
            if (value == 0) {
                if (source == std::string("0")) {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    return false;
}
// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to std::vector<double> type.
 * \return True when the convertion was executed successfully.
 *
 * Legal array element seperators are ' ', ',' or ';'
 */
bool XML::convertValueToDoubleArray(const std::string& source, std::vector<double>& value) noexcept {
    bool result = false;
    if (!source.empty()) {
        unsigned long index = 0;
        char* strFloat = nullptr;
        while (XML::getOneSubString(source.c_str(), index, strFloat)) {
            if (strFloat) {
                if (checkNumber(strFloat, true)) {
                    double curValue = 0.0;
                    result = StringUtils::stringToDouble(strFloat, curValue);
                    value.push_back(curValue);
                    free(strFloat);
                }
            }
        }
    }
    return result;
}
// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to std::vector<long> type.
 * \return True when the convertion was executed successfully.
 *
 * Legal array element seperators are ' ', ',' or ';'
 */
bool XML::convertValueToLongArray(const std::string& source, std::vector<long>& value) noexcept {
    bool result = false;
    if (!source.empty()) {
        unsigned long index = 0;
        char* strInt = nullptr;
        while (XML::getOneSubString(source.c_str(), index, strInt)) {
            if (strInt) {
                if (checkNumber(strInt, false)) {
                    long curValue = atol(strInt);
                    value.push_back(curValue);
                    free(strInt);
                    result = true;
                }
            }
        }
    }
    return result;
}
// ---------------------------------------------------------------------
/*!
 * \param source : A value represented via a string.
 * \param value : The value converted to std::vector<std::string> type.
 * \return True when the convertion was executed successfully.
 *
 * Legal array element seperators are ' ', ',' or ';'
 */
bool XML::convertValueToStringArray(const std::string& source, std::vector<std::string>& value) noexcept {
    bool result = false;
    if (!source.empty()) {
        unsigned long index = 0;
        char* one_string = nullptr;
        while (XML::getOneSubString(source.c_str(), index, one_string)) {
            if (one_string) {
                value.emplace_back(one_string);
                free(one_string);
                result = true;
            }
        }
    }
    return result;
}

// ---------------------------------------------------------------------
/*!
 * \param one_string : String to be converted to upper case.
 */
void XML::makeUpper(std::string& one_string) noexcept {
    auto size = static_cast<unsigned long>(one_string.size());
    char* curString = static_cast<char*>(malloc(size + 1));
#if _WIN32
    strcpy_s(curString, size + 1, one_string.c_str());
#else
    strcpy(curString, one_string.c_str());
#endif
    for (unsigned int i = 0; i < size; i++) {
        curString[i] = char(toupper(curString[i]));
    }
    one_string = std::string(curString);
    free(curString);
}

// ---------------------------------------------------------------------
bool XML::checkNumber(intercom::string_view source, bool float_value) noexcept {
    auto numOfChar = static_cast<unsigned long>(source.length());
    for (unsigned long i = 0; i < numOfChar; i++) {
        switch (source[i]) {
        case '0':
        case '1':
        case '2':
        case '3':
        case '4':
        case '5':
        case '6':
        case '7':
        case '8':
        case '9':
            break;
        case '.':
        case ',':
            if (!float_value) {
                return false;
            }
            break;
        case '-':
            break;
        default:
            return false;
        }
    }
    return true;
}

// ---------------------------------------------------------------------
bool XML::getOneSubString(intercom::string_view source_string, unsigned long& string_index,
                          char*& sub_string) noexcept {
    unsigned long subStringSize = 100;
    sub_string = static_cast<char*>(malloc(sizeof(char) * subStringSize));
    unsigned long numChar = 0;
    auto len = static_cast<unsigned long>(source_string.length());
    if (string_index < len) {
        unsigned long i = 0;
        for (i = string_index; i < len; i++) {
            switch (source_string[i]) {
            case '{':
            case '}':
            case '(':
            case ')':
            case '/':
            case '\r':
                // Skip characters !
                break;
            case ' ':
            case '\0':
            case ',':
            case ';': {
                if (source_string[i] == ' ' && numChar == 0) {
                    break;
                }
                // Separator character !
                string_index = i + 1;
                sub_string[numChar++] = '\0';
                return true;
            } break;
            default:
                sub_string[numChar++] = source_string[i];
                if (numChar >= subStringSize) {
                    subStringSize = subStringSize * 2;
                    sub_string = static_cast<char*>(realloc(sub_string, sizeof(char) * subStringSize));
                }
            }
        }
        if (numChar) {
            string_index = i + 1;
            sub_string[numChar++] = '\0';
            return true;
        }
    }
    free(sub_string);
    return false;
}

// ---------------------------------------------------------------------
bool XML::decodeName(const std::string& element_name, std::vector<std::string>& names) noexcept {
    bool result = false;
    auto numOfChar = static_cast<unsigned long>(element_name.size());
    if (numOfChar > 0) {
        unsigned long prevPos = 0;
        auto slachPos = static_cast<unsigned long>(element_name.find_first_of('/', 0));
        if (slachPos >= numOfChar) {
            names.push_back(element_name);
            result = true;
        } else {
            while (slachPos < numOfChar) {
                std::string newString = element_name.substr(prevPos, slachPos - prevPos);
                if (!newString.empty()) {
                    names.push_back(newString);
                    prevPos = slachPos + 1;
                    result = true;
                }
                slachPos = static_cast<unsigned long>(element_name.find_first_of('/', slachPos + 1));
            }
            if (prevPos < numOfChar - 1) {
                std::string newString = element_name.substr(prevPos, numOfChar - prevPos);
                names.push_back(newString);
                result = true;
            }
        }
    }
    return result;
}
