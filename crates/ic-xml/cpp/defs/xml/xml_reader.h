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

#include <iostream>
#include <memory>
#include <string>
#include <vector>

#include "InterCOM/dyn_link.h"
#include "xml_types.h"

namespace intercom {
class XMLReaderFile;
//! Reads an %XML file
class INTERCOM_PUBLIC XMLReader : public XML {
  public:
    // A type that provides a iterator that can read any element from the xml structure.
    class INTERCOM_PUBLIC Iterator {
        friend class XMLReader;

      public:
        //! Default constructor
        Iterator() noexcept;
        //! Constructor taking an %XML element object
        Iterator(const XMLElement& element) noexcept;
        //! Copy constructor
        Iterator(const Iterator& rhs) noexcept;
        //! Destructor
        ~Iterator() noexcept = default;

        //! Tests if the iterator objects on the left side of the operator is not equal to the
        //! iterator object on the right side
        bool operator!=(const Iterator& rhs) const noexcept;
        //! Tests if the iterator object on the left side of the operator is equal to the iterator
        //! object on the right side.
        bool operator==(const Iterator& rhs) const noexcept;
        //! Assign the content of the iterator object on the right side of the operator to iterator
        //! object on the left side.
        Iterator& operator=(const Iterator& rhs) noexcept;
        //! Returns the designated value
        const XMLElement& operator*() const noexcept;
        //! Returns the pointer to class object
        const XMLElement* operator->() const noexcept;
        //! Preincrement the iterator object
        Iterator& operator++() noexcept;
        //! Postincrement the iterator object
        Iterator operator++(int) noexcept;
        //! Predecrement the iterator object
        Iterator& operator--() noexcept;
        //! Postdecrement the iterator object
        Iterator operator--(int) noexcept;

      protected:
        const XMLElement* element() const noexcept;

      private:
        XMLElement* m_XMLElement;
        std::size_t m_XMLElementIndex;
    };

    //! Defines the read status for the %XML configuration file
    enum ReadStatus {
        READ_FILE_NOT_FOUND = 0,  //!< The %XML Configuration file is not found
        READ_PARSE_ERROR = 1,     //!< An error occoured while parsing the XML-file
        READ_SUCCESS = 2          //!< The XML-file was successfully parsed
    };

    //! Default constructor
    XMLReader();

    //! Constructor taking an %XML file name
    XMLReader(
        const std::string& fileName,
        bool verbose = false,
        std::ostream& targetStream = std::cout
    );
    //! Constructor taking a %stringstream
    XMLReader(
        const std::stringstream& inputStream,
        bool verbose = false,
        std::ostream& targetStream = std::cout
    );
    //! Copy constructor
    XMLReader(const XMLReader& rhs);
    //! Move constructor
    XMLReader(XMLReader&& rhs) noexcept = default;
    //! Copy assignment operator
    XMLReader& operator=(const XMLReader& rhs);
    //! Move assignment operator
    XMLReader& operator=(XMLReader&&) = default;

    //! Checks if the specified XML-file is valid
    bool isValid() const;

    //! Returns the filename last used to read a file (or an empty string for stringstreams)
    const std::string& getFileName() const;

    //! Sets the verbose mode for this %XMLReader
    void setVerbose(bool verbose = true, std::ostream& targetStream = std::cout) noexcept;

    //! Retrieves the verbose mode for this %XMLReader
    bool isVerbose() const;

    //! Returns an iterator addressing the root %XMLElement in the XML-file.
    Iterator begin() const noexcept;
    //! Returns an iterator addressing an invalid %XMLElement object representing an end element.
    static Iterator end() noexcept;
    //! Returns an iterator addressing the first %XMLElement for the specified element.
    Iterator begin(const Iterator& iter) const noexcept;
    //! Returns an iterator that addresses the location succeeding the last %XMLElement for the
    //! specified element.
    Iterator end(const Iterator& iter) const noexcept;
    //! Returns an iterator addressing the location of an %XMLElement in the XML-file that has a
    //! name equivalent to a specified name.
    Iterator find(const std::string& elementName, bool caseSensitive) const noexcept;
    //! Returns an iterator addressing the location of an %XMLElement that has a name equivalent to
    //! a specified name.
    Iterator find(const Iterator& iter, const std::string& elementName, bool caseSensitive)
        const noexcept;

    //! Reads the %XML tags and attributes based on the provided file name
    ReadStatus read(const std::string& fileName) noexcept;

    //! Reads the %XML tags and attributes based on the provided stringstream
    ReadStatus read(const std::stringstream& input) noexcept;

  private:
    void printVerbose(
        const std::string& before,
        const std::string& info,
        const std::string& after,
        int indent = 0
    ) noexcept;

    XMLReader::ReadStatus read(XMLReaderFile& xmlFile) noexcept;

    std::unique_ptr<XMLElement> m_Root;
    bool m_Valid;
    bool m_Verbose;
    std::ostream* m_VerboseStream;
    std::string m_FileName;
};
}  // namespace intercom
