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

#include <string>
#include <string_view>
#include <vector>

#include "InterCOM/dyn_link.h"

namespace intercom {
class XMLReader;
class XMLElement;

/*!
 * \brief Legal %XML data types
 */
enum XMLDataType {
    XML_DATA_TYPE_UNKNOWN = 0,       //!< Unknown data type
    XML_DATA_TYPE_BOOL = 1,          //!< Boolean type
    XML_DATA_TYPE_STRING = 2,        //!< String type
    XML_DATA_TYPE_DOUBLE = 3,        //!< Double type
    XML_DATA_TYPE_LONG = 4,          //!< Long type
    XML_DATA_TYPE_DOUBLE_ARRAY = 5,  //!< Double-array type
    XML_DATA_TYPE_LONG_ARRAY = 6,    //!< Long-array type
    XML_DATA_TYPE_STRING_ARRAY = 7   //!< String-array type
};

// XML attribute class
class INTERCOM_PUBLIC XMLAttribute {
    friend class XMLReader;
    friend class XMLElement;

  public:
    //! Default constructor
    XMLAttribute(std::string name = std::string("NoName")) noexcept;

    //! Copy constructor
    XMLAttribute(const XMLAttribute&) = default;

    //! Destructor
    ~XMLAttribute() noexcept = default;

    //! Assignment operator
    XMLAttribute& operator=(const XMLAttribute& rhs) noexcept;

    //! "Equal" comparison operator
    bool operator==(const XMLAttribute& rhs) const noexcept;
    //! "Not equal" comparison operator
    bool operator!=(const XMLAttribute& rhs) const noexcept;
    //! Checks if current attribute is valid
    bool isValid() const noexcept;
    //! Returns the attribute defined as invalid attribute
    static const XMLAttribute& invalid() noexcept;
    //! Returns the attribute name
    const std::string& name() const noexcept;
    //! Returns the attribute value
    const std::string& value() const noexcept;
    //! Returns the attribute value data type
    XMLDataType xmlDataType() const noexcept;
    //! Sets the name of this attribute
    void setName(const std::string& name) noexcept;
    //! Sets the value of this attribute
    void setValue(const std::string& value) noexcept;
    //! Sets the datatype of this attribute
    void setXMLDataType(const XMLDataType type) noexcept;
    //! Sets the name, value and datatype of this attribute
    void set(
        const std::string& name,
        const std::string& value,
        const XMLDataType type = XML_DATA_TYPE_STRING
    ) noexcept;

    explicit operator bool() const {
        return isValid();
    }

  protected:
    std::string m_Name;
    std::string m_Value;
    XMLDataType m_XMLDataType;
    static const XMLAttribute m_Invalid;
};

// XML element class
class INTERCOM_PUBLIC XMLElement {
    friend class XMLReader;

  public:
    //! Default constructor
    XMLElement(std::string name = std::string("NoName")) noexcept;
    //! Constructor taking name, value and parent
    XMLElement(std::string name, std::string value, XMLElement* parent = nullptr) noexcept;
    //! Copy constructor
    XMLElement(const XMLElement& rhs) noexcept;
    //! Move constructor
    XMLElement(XMLElement&&) noexcept = default;
    //! Destructor
    ~XMLElement() noexcept;

    //! Assignment operator
    XMLElement& operator=(const XMLElement& rhs) noexcept;
    XMLElement& operator=(XMLElement&&) = default;

    //! "Equal" comparison operator
    bool operator==(const XMLElement& rhs) const noexcept;
    //! "Not equal" comparison operator
    bool operator!=(const XMLElement& rhs) const noexcept;
    //! Checks if current element is valid
    bool isValid() const noexcept;
    //! Returns the element defined as invalid element
    static const XMLElement& invalid() noexcept;
    //! Returns the element name
    const std::string& name() const noexcept;
    //! Returns the element value
    const std::string& value() const noexcept;
    //! Returns the element value data type
    XMLDataType xmlDataType() const noexcept;

    //! Sets the name of this element
    void setName(const std::string& name) noexcept;
    //! Sets the value of this element
    void setValue(const std::string& value) noexcept;
    //! Sets the datatype of this element
    void setXMLDataType(const XMLDataType type) noexcept;
    //! Sets the name, value and datatype of this element
    void set(
        const std::string& name,
        const std::string& value,
        const XMLDataType type = XML_DATA_TYPE_UNKNOWN
    ) noexcept;
    //! Returns number of member elements
    std::size_t numberOfXMLElements() const noexcept;
    //! Returns a member element identified by specified index
    XMLElement& xmlElement(std::size_t index) & noexcept;
    const XMLElement& xmlElement(std::size_t index) const& noexcept;
    XMLElement&& xmlElement(std::size_t index) && noexcept;

    //! Returns number of member attributes
    std::size_t numberOfXMLAttributes() const noexcept;
    //! Returns a member attribute identified by specified index
    const XMLAttribute& xmlAttribute(std::size_t index) const noexcept;
    //! Returns the member attribute identified by specified name
    const XMLAttribute& xmlAttribute(const std::string& name) const noexcept;

    //! Creates a member %XMLElement related to current %XMLElement
    XMLElement& createMemberXMLElement() noexcept;
    //! Creates a member %XMLAttribute related to current %XMLAttribute
    XMLAttribute& createMemberXMLAttribute() noexcept;

    //! Finds the element matching specified element name
    const XMLElement& find(const std::string& elementName, bool caseSensitive = true)
        const noexcept;
    //! Returns the parent member element
    const XMLElement& parentXMLElement() const noexcept;

    unsigned long lineNumber() const {
        return m_Line;
    }

  protected:
    const XMLElement* doFind(const std::string& oneElementName, bool caseSensitive = true)
        const noexcept;
    std::string m_Name;
    std::string m_Value;
    unsigned long m_Line = 0;
    XMLDataType m_XMLDataType;
    std::vector<XMLElement*> m_XMLElements;
    std::vector<XMLAttribute*> m_XMLAttributes;
    XMLElement* m_ParentElement;
    static const XMLElement m_Invalid;
};

// XML Generic class
class INTERCOM_PUBLIC XML {
    friend class XMLElement;

  public:
    //! Default constructor
    XML() noexcept;
    //! Destructor
    ~XML() noexcept = default;

    //! Converts an %XML element value to bool data type
    static bool convertValueToBool(const std::string& source, bool& value) noexcept;
    //! Converts an %XML element value to double data type
    static bool convertValueToDouble(const std::string& source, double& value) noexcept;
    //! Converts an %XML element value to long data type
    static bool convertValueToLong(const std::string& source, long& value) noexcept;
    //! Converts an %XML element value to array of double data type
    static bool
    convertValueToDoubleArray(const std::string& source, std::vector<double>& value) noexcept;
    //! Converts an %XML element value to array of long data type
    static bool
    convertValueToLongArray(const std::string& source, std::vector<long>& value) noexcept;
    //! Converts an %XML element value to array of string data type
    static bool
    convertValueToStringArray(const std::string& source, std::vector<std::string>& value) noexcept;
    //! Converts the provided string to upper case
    static void makeUpper(std::string& oneString) noexcept;

  protected:
    static bool checkNumber(std::string_view source, bool floatValue) noexcept;
    static bool getOneSubString(
        std::string_view sourceString,
        unsigned long& stringIndex,
        char*& subString
    ) noexcept;
    static bool
    decodeName(const std::string& elementName, std::vector<std::string>& names) noexcept;
};
}  // namespace intercom
