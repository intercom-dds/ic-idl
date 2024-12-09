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

#include <cstdio>
#include <sstream>
#include <string>

namespace intercom {
#define XML_MAX_IO_BUFFER_SIZE 5000

// Read one specific XML file
class XMLReaderFile {
  public:
    // Legal XML structures
    enum XMLState {
        STATE_UNKNOWN = 0,
        STATE_NEW_ELEMENT = 1,
        STATE_NEW_ATTRIBUTE = 2,
        STATE_ATTRIBUTE_DATA = 3,
        STATE_ELEMENT_DATA = 4,
        STATE_EMPTY = 5,
        STATE_COMMENT = 6,
        STATE_CONTROL = 7,
        STATE_END_ELEMENT = 8,
        STATE_END_OF_FILE = 9
    };
    // Legal XML data types
    enum XMLDataType {
        DATA_TYPE_UNKNOWN = 0,
        DATA_TYPE_BOOL = 1,
        DATA_TYPE_STRING = 2,
        DATA_TYPE_DOUBLE = 3,
        DATA_TYPE_LONG = 4,
        DATA_TYPE_DOUBLE_ARRAY = 5,
        DATA_TYPE_LONG_ARRAY = 6,
        DATA_TYPE_STRING_ARRAY = 7
    };

    // Constructor taking a XML file name
    explicit XMLReaderFile(const char* a_file_name) noexcept;
    // Constructor taking a stringstream
    explicit XMLReaderFile(const std::stringstream& ss) noexcept;
    // Destructor
    virtual ~XMLReaderFile() noexcept;

    // Checks if specified XML file exists and is accessible
    bool isValid() const noexcept;

    // Read one section of the XML file.
    const char* readNext(XMLState& state, XMLDataType& dataType) noexcept;

    unsigned long currentLine() const {
        return m_Line;
    }

  private:
    bool read() noexcept;
    void replaceXMLEscapeCharacters() noexcept;
    static XMLDataType decideDataType(
        char* data,
        bool stringFlag,
        bool numberFlag,
        bool floatFlag,
        bool arrayFlag
    ) noexcept;
    static void makeUpper(std::string& oneString) noexcept;
    std::stringstream m_stream;
    FILE* m_XmlFile;
    bool m_EndOfFile;
    unsigned long m_IOBufferSize;
    char* m_IOBuffer;
    unsigned long m_BufferPos;
    unsigned long m_Line = 0;
    char* m_NextWord;
    unsigned long m_NextWordPos;
    bool m_IsValid;
    XMLState m_CurState;
};
}  // namespace intercom
