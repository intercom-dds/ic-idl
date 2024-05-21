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

#include "utils/XMLReaderFile.h"

#include <cctype>
#include <cstdlib>
#include <cstring>
#include <string>

#if _WIN32
#  pragma warning(disable : 4456)
#  include <share.h>
#  include <stdio.h>
#endif

using namespace intercom;
using std::toupper;

XMLReaderFile::XMLReaderFile(const char* a_file_name) noexcept
        : m_XmlFile(nullptr),
          m_EndOfFile(true),
          m_IOBufferSize(0),
          m_IOBuffer(nullptr),
          m_BufferPos(0),
          m_NextWord(nullptr),
          m_NextWordPos(XML_MAX_IO_BUFFER_SIZE),
          m_IsValid(false),
          m_CurState(STATE_ELEMENT_DATA) {
    m_NextWord = static_cast<char*>(malloc(sizeof(char) * XML_MAX_IO_BUFFER_SIZE));
    m_IOBuffer = static_cast<char*>(malloc(sizeof(char) * XML_MAX_IO_BUFFER_SIZE));
    if (a_file_name) {
#if _WIN32
        m_XmlFile = _fsopen(a_file_name, "rt", _SH_DENYWR);
#else
        m_XmlFile = fopen(a_file_name, "rt");
#endif
        if (m_XmlFile) {
            m_IsValid = true;
            m_EndOfFile = false;
            read();
        }
    }
}

XMLReaderFile::XMLReaderFile(const std::stringstream& ss) noexcept
        : m_stream(ss.str()),
          m_XmlFile(nullptr),
          m_EndOfFile(true),
          m_IOBufferSize(0),
          m_IOBuffer(nullptr),
          m_BufferPos(0),
          m_NextWord(nullptr),
          m_NextWordPos(XML_MAX_IO_BUFFER_SIZE),
          m_IsValid(false),
          m_CurState(STATE_ELEMENT_DATA) {
    m_NextWord = static_cast<char*>(malloc(sizeof(char) * XML_MAX_IO_BUFFER_SIZE));
    m_IOBuffer = static_cast<char*>(malloc(sizeof(char) * XML_MAX_IO_BUFFER_SIZE));
    if (!m_stream.str().empty()) {
        m_IsValid = true;
        m_EndOfFile = false;
        read();
    }
}

// ---------------------------------------------------------------------
XMLReaderFile::~XMLReaderFile() noexcept {
    if (m_NextWord) {
        free(m_NextWord);
    }
    if (m_IOBuffer) {
        free(m_IOBuffer);
    }
    if (m_XmlFile) {
        fclose(m_XmlFile);
    }
}

// ---------------------------------------------------------------------
bool XMLReaderFile::isValid() const noexcept {
    return m_IsValid;
}

// ---------------------------------------------------------------------
const char* XMLReaderFile::readNext(XMLReaderFile::XMLState& state, XMLReaderFile::XMLDataType& data_type) noexcept {
    state = XMLReaderFile::STATE_ELEMENT_DATA;
    data_type = XMLReaderFile::DATA_TYPE_UNKNOWN;

    bool insideFnutFlag(false);
    bool arrayFlag(false);
    bool floatFlag(false);
    bool numberFlag(false);
    bool stringFlag(false);
    bool startString(false);
    bool endComments(false);

    m_NextWord[0] = '\0';
    unsigned long numChar = 0;
    bool b(true);
    while (b) {
        if (m_BufferPos >= m_IOBufferSize) {
            if (!read())

            {
                state = STATE_UNKNOWN;
                if (m_EndOfFile) {
                    state = XMLReaderFile::STATE_END_OF_FILE;
                }
                m_CurState = state;
                return m_NextWord;
            }
            if (numChar + m_IOBufferSize >= m_NextWordPos) {
                m_NextWord = static_cast<char*>(realloc(m_NextWord, sizeof(char) * (m_NextWordPos + m_IOBufferSize)));
                m_NextWordPos = m_NextWordPos + m_IOBufferSize;
            }
        }
        if (m_IOBuffer[m_BufferPos] == '\0') {
            if (!read()) {
                state = STATE_UNKNOWN;
                if (m_EndOfFile) {
                    state = XMLReaderFile::STATE_END_OF_FILE;
                }
                m_CurState = state;
                return m_NextWord;
            }
            if (numChar + m_IOBufferSize >= m_NextWordPos) {
                m_NextWord = static_cast<char*>(realloc(m_NextWord, sizeof(char) * (m_NextWordPos + m_IOBufferSize)));
                m_NextWordPos = m_NextWordPos + m_IOBufferSize;
            }
        }

        // Special handling when comments
        if ((m_CurState == STATE_COMMENT) && (!endComments)) {
            if (m_IOBuffer[m_BufferPos] == '-') {
                unsigned long tmpPos = 1;
                if (m_BufferPos + 1 >= m_IOBufferSize) {
                    if (!read()) {
                        state = STATE_UNKNOWN;
                        if (m_EndOfFile) {
                            state = XMLReaderFile::STATE_END_OF_FILE;
                        }
                        m_CurState = state;
                        return m_NextWord;
                    }
                    if (numChar + m_IOBufferSize >= m_NextWordPos) {
                        m_NextWord = static_cast<char*>(
                                realloc(m_NextWord, sizeof(char) * (m_NextWordPos + m_IOBufferSize)));
                        m_NextWordPos = m_NextWordPos + m_IOBufferSize;
                    }
                    // Need to add the '-' we lost when reading more
                    m_NextWord[numChar++] = '-';
                    tmpPos = 0;
                }
                if (m_IOBuffer[m_BufferPos + tmpPos] == '-') {
                    endComments = true;
                }
            }
            if (!endComments) {
                m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
                continue;
            }
        }
        switch (m_IOBuffer[m_BufferPos]) {
        case '<': {
            if (insideFnutFlag) {
                m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
            } else {
                if (numChar == 0) {
                    unsigned long tmpPos = 1;
                    if (m_BufferPos + 1 >= m_IOBufferSize) {
                        if (!read()) {
                            state = STATE_UNKNOWN;
                            if (m_EndOfFile) {
                                state = XMLReaderFile::STATE_END_OF_FILE;
                            }
                            m_CurState = state;
                            return m_NextWord;
                        }
                        if (numChar + m_IOBufferSize >= m_NextWordPos) {
                            m_NextWord = static_cast<char*>(
                                    realloc(m_NextWord, sizeof(char) * (m_NextWordPos + m_IOBufferSize)));
                            m_NextWordPos = m_NextWordPos + m_IOBufferSize;
                        }
                        tmpPos = 0;
                    }
                    switch (m_IOBuffer[m_BufferPos + tmpPos]) {
                    case '/': {
                        // End block is coming
                        state = STATE_END_ELEMENT;
                        m_CurState = state;
                        m_BufferPos += tmpPos;
                        m_BufferPos++;
                        startString = false;
                    } break;
                    case '!': {
                        if ((m_IOBuffer[m_BufferPos + tmpPos + 1] == '-') &&
                            (m_IOBuffer[m_BufferPos + tmpPos + 2] == '-')) {
                            // Comment block is coming
                            state = STATE_COMMENT;
                            m_CurState = state;
                            m_BufferPos += tmpPos;
                            m_BufferPos += 3;
                            startString = true;
                        } else {
                            // XML Keyword is coming
                            state = STATE_CONTROL;
                            m_CurState = state;
                            m_BufferPos += tmpPos;
                            m_BufferPos++;
                            startString = true;
                        }
                    } break;
                    case '?': {
                        // Control block is coming
                        state = STATE_CONTROL;
                        m_CurState = state;
                        m_BufferPos += tmpPos;
                        m_BufferPos++;
                        startString = true;
                    } break;
                    default:  // Start block is coming
                    {
                        state = STATE_NEW_ELEMENT;
                        m_CurState = state;
                        if (tmpPos == 1) {
                            m_BufferPos++;
                        }
                        startString = false;
                    } break;
                    }
                } else {
                    switch (state) {
                    case STATE_NEW_ELEMENT: {
                        numChar = 0;
                        startString = false;
                        m_BufferPos++;
                    } break;
                    case STATE_ELEMENT_DATA: {
                        m_NextWord[numChar++] = '\0';  // Terminate Data block
                        data_type = decideDataType(m_NextWord, stringFlag, numberFlag, floatFlag, arrayFlag);
                        m_CurState = state;
                        return m_NextWord;
                    } break;
                    case STATE_END_ELEMENT: {
                        // Error in file format!
                        state = STATE_UNKNOWN;
                        m_CurState = state;
                        return m_NextWord;
                    } break;
                    default:
                        m_BufferPos++;
                        break;
                    }
                }
            }
        } break;
        case '>': {
            if (insideFnutFlag) {
                m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
            } else {
                if (numChar == 1) {
                    if (m_NextWord[0] == '/') {
                        state = STATE_END_ELEMENT;
                        m_BufferPos++;
                        m_NextWord[numChar] = '\0';
                        return m_NextWord;
                    }
                }
                if (numChar > 0) {
                    switch (m_CurState) {
                    case STATE_COMMENT: {
                        // End Comments?
                        if ((m_NextWord[numChar - 1] == '-') && (m_NextWord[numChar - 2] == '-')) {
                            m_NextWord[numChar - 2] = '\0';
                            m_BufferPos++;
                            m_CurState = STATE_UNKNOWN;
                            return m_NextWord;
                        }
                    } break;
                    case STATE_NEW_ELEMENT:
                    case STATE_END_ELEMENT: {
                        if (m_NextWord[numChar - 1] == '/') {
                            // Empty field
                            m_NextWord[numChar - 1] = '\0';
                            state = STATE_EMPTY;
                        } else {
                            m_NextWord[numChar++] = '\0';
                        }
                        m_BufferPos++;
                        m_CurState = state;
                        return m_NextWord;
                    } break;
                    case STATE_ATTRIBUTE_DATA: {
                        // We're in the process of parsing attribute data,
                        // but got a '>', which is wrong.
                        state = STATE_UNKNOWN;
                        m_CurState = state;
                        return m_NextWord;
                    } break;
                    case STATE_ELEMENT_DATA: {
                        // Error in file format!
                        state = STATE_UNKNOWN;
                        m_CurState = state;
                        return m_NextWord;
                    } break;
                    case STATE_CONTROL: {
                        if (m_NextWord[numChar - 1] == '?') {
                            // End of control field
                            m_NextWord[0] = '\0';
                            m_BufferPos++;
                            m_CurState = STATE_UNKNOWN;
                            return m_NextWord;
                        }  // End of XML keyword field
                        m_NextWord[0] = '\0';
                        m_BufferPos++;
                        m_CurState = STATE_UNKNOWN;
                        return m_NextWord;

                    } break;
                    default:
                        m_BufferPos++;
                        break;
                    }
                } else if (numChar == 0 && m_CurState == STATE_NEW_ATTRIBUTE) {
                    // Special handling. We just finished parsing attribute data, this
                    // terminates this element
                    m_CurState = STATE_ELEMENT_DATA;
                }
                m_BufferPos++;
            }
        } break;
        case ' ':
        case '\t': {
            if (!startString) {
                m_BufferPos++;  // Skip character
            } else {
                if (state == STATE_NEW_ELEMENT) {
                    // Attributes found
                    m_NextWord[numChar++] = '\0';
                    m_BufferPos++;
                    state = STATE_NEW_ELEMENT;
                    m_CurState = STATE_NEW_ATTRIBUTE;
                    return m_NextWord;
                }
                if (m_CurState == STATE_NEW_ATTRIBUTE) {
                    state = STATE_NEW_ATTRIBUTE;
                    m_NextWord[numChar++] = '\0';
                    m_BufferPos++;
                    m_CurState = STATE_ATTRIBUTE_DATA;
                    return m_NextWord;
                } else if (m_CurState == STATE_ATTRIBUTE_DATA) {
                    // Attributes data found
                    if (insideFnutFlag) {
                        // We are inside '"' and space is allowed for attribute data so we'll just
                        // continue parsing until the ending '"'
                        m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
                        // Space inside attribute data identifies an array
                        // (string, long or double)
                        arrayFlag = true;
                    } else {
                        // We're not inside '"', which is wrong
                        state = STATE_UNKNOWN;
                        m_CurState = state;
                        return m_NextWord;
                    }
                } else if (state == STATE_ELEMENT_DATA) {
                    m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
                    if (!insideFnutFlag) {
                        arrayFlag = true;
                    }
                } else if (state == STATE_COMMENT) {
                    m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
                } else {
                    m_BufferPos++;  // Skip character
                }
            }
        } break;
        case '"':
        case '\'': {
            if (insideFnutFlag) {
                insideFnutFlag = false;
                if (m_CurState == STATE_ATTRIBUTE_DATA) {
                    // Attributes data found
                    m_NextWord[numChar++] = '\0';
                    m_BufferPos++;
                    state = STATE_ATTRIBUTE_DATA;
                    m_CurState = STATE_NEW_ATTRIBUTE;
                    data_type = decideDataType(m_NextWord, stringFlag, numberFlag, floatFlag, arrayFlag);
                    return m_NextWord;
                }
            } else {
                insideFnutFlag = true;
            }
            m_BufferPos++;
        } break;
        case '=': {
            if (m_CurState == STATE_NEW_ATTRIBUTE) {
                state = STATE_NEW_ATTRIBUTE;
                m_NextWord[numChar++] = '\0';
                m_BufferPos++;
                m_CurState = STATE_ATTRIBUTE_DATA;
                return m_NextWord;
            }
            if (state == STATE_ELEMENT_DATA) {
                m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
                if (!insideFnutFlag) {
                    arrayFlag = true;
                }
            } else if (state == STATE_COMMENT) {
                m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
            } else {
                m_BufferPos++;  // Skip character
            }
        } break;
        case '\r':
        case '\n': {
            m_BufferPos++;  // Skip character
            m_Line++;
        } break;
        case '\0': {
            if (!read()) {
                if (m_EndOfFile) {
                    state = XMLReaderFile::STATE_END_OF_FILE;
                    m_CurState = state;
                    return m_NextWord;
                }
                if (numChar + m_IOBufferSize >= m_NextWordPos) {
                    m_NextWord =
                            static_cast<char*>(realloc(m_NextWord, sizeof(char) * (m_NextWordPos + m_IOBufferSize)));
                    m_NextWordPos = m_NextWordPos + m_IOBufferSize;
                }
            }
        } break;
        default:
            if (state == STATE_ELEMENT_DATA) {
                if (m_CurState == STATE_ATTRIBUTE_DATA && !insideFnutFlag) {
                    // We're expecting attribute data, but we're not inside
                    // '"', so this is wrong...
                    state = STATE_UNKNOWN;
                    m_CurState = state;
                    return m_NextWord;
                }
                if (m_CurState != STATE_NEW_ATTRIBUTE) {
                    // Analyze data type in current data block
                    switch (m_IOBuffer[m_BufferPos]) {
                    case ',':
                    case ';': {
                        if (!insideFnutFlag) {
                            arrayFlag = true;
                        }
                    } break;
                    case '.': {
                        floatFlag = true;
                    } break;
                    case '-':
                    case '+':
                        break;
                    default:
                        if ((m_IOBuffer[m_BufferPos] >= '0') && (m_IOBuffer[m_BufferPos] <= '9')) {
                            numberFlag = true;
                        } else {
                            stringFlag = true;
                        }
                    }
                }
            }
            m_NextWord[numChar++] = m_IOBuffer[m_BufferPos++];
            startString = true;
        }
    }
    // Error in file format!
    state = STATE_UNKNOWN;
    m_CurState = state;
    return m_NextWord;
}

// ---------------------------------------------------------------------
XMLReaderFile::XMLDataType XMLReaderFile::decideDataType(char* data, bool string_flag, bool number_flag,
                                                         bool float_flag, bool array_flag) noexcept {
    XMLDataType dataType = DATA_TYPE_UNKNOWN;
    // Decide data type of current data block
    if (string_flag) {
        if (array_flag) {
            dataType = DATA_TYPE_STRING_ARRAY;
        } else {
            auto numChar = static_cast<unsigned long>(strlen(data));
            dataType = DATA_TYPE_STRING;
            // Check if bool
            if ((numChar == 4) || (numChar == 5)) {
                std::string tmpString(data);
                makeUpper(tmpString);
                if (tmpString == std::string("TRUE")) {
                    dataType = DATA_TYPE_BOOL;
                } else if (tmpString == std::string("FALSE")) {
                    dataType = DATA_TYPE_BOOL;
                }
            }
        }
    } else if (number_flag) {
        if (float_flag) {
            if (array_flag) {
                dataType = DATA_TYPE_DOUBLE_ARRAY;
            } else {
                dataType = DATA_TYPE_DOUBLE;
            }
        } else {
            if (array_flag) {
                dataType = DATA_TYPE_LONG_ARRAY;
            } else {
                dataType = DATA_TYPE_LONG;
            }
        }
    }
    return dataType;
}

// ---------------------------------------------------------------------
bool XMLReaderFile::read() noexcept {
    bool dataRead = false;
    if (!m_EndOfFile) {
        m_BufferPos = 0;
        m_IOBufferSize = 0;
        size_t numRead = 0;

        if (m_XmlFile) {
            numRead = fread(m_IOBuffer, 1, XML_MAX_IO_BUFFER_SIZE - 1, m_XmlFile);
        } else {
            m_stream.read(m_IOBuffer, XML_MAX_IO_BUFFER_SIZE - 1);
            numRead = static_cast<size_t>(m_stream.gcount());
        }

        m_IOBufferSize = static_cast<unsigned long>(numRead);
        dataRead = (numRead > 0);
        // Check if we reached end of file
        if (m_XmlFile) {
            if (feof(m_XmlFile)) {
                m_EndOfFile = true;
            }
        } else if (!m_stream) {
            m_EndOfFile = true;
        }
    }
    return dataRead;
}
// ---------------------------------------------------------------------
void XMLReaderFile::replaceXMLEscapeCharacters() noexcept {
    auto len = static_cast<unsigned long>(strlen(m_NextWord));
    char* escapeWord = nullptr;

    unsigned long pos = 0;
    unsigned long newPos = 0;
    bool b(true);
    while (b) {
        char oneChar = m_NextWord[pos++];
        switch (oneChar) {
        case '\0': {
            m_NextWord[newPos++] = '\0';
            if (escapeWord) {
                free(escapeWord);
            }
            return;
        } break;
        case '&': {
            if (!escapeWord) {
                escapeWord = static_cast<char*>(malloc(sizeof(char) * len));
            }
            unsigned long escapePos = 0;
            bool loopCtrl = true;
            while (loopCtrl) {
                oneChar = m_NextWord[pos++];
                switch (oneChar) {
                case ';': {
                    escapeWord[escapePos++] = '\0';
                    loopCtrl = false;
                } break;
                case '&': {
                    escapeWord[escapePos++] = '&';
                    escapeWord[escapePos++] = '&';
                    escapeWord[escapePos++] = '\0';
                    loopCtrl = false;
                } break;
                default: {
                    escapeWord[escapePos++] = oneChar;
                } break;
                }
            }
            // Decide which escape character!
            if (strlen(escapeWord) > 1) {
                std::string escapeString(escapeWord);
                if (escapeString == "amp") {
                    m_NextWord[newPos++] = '&';
                } else if (escapeString == "gt") {
                    m_NextWord[newPos++] = '>';
                } else if (escapeString == "lt") {
                    m_NextWord[newPos++] = '<';
                } else if (escapeString == "&&") {
                    m_NextWord[newPos++] = '&';
                    m_NextWord[newPos++] = '&';
                }
            }
        } break;
        default: {
            m_NextWord[newPos++] = oneChar;
        } break;
        }
        if (pos >= len) {
            m_NextWord[newPos++] = '\0';
            break;
        }
    }
    if (escapeWord) {
        free(escapeWord);
    }
}

//! Convert this string to upper-case.
void XMLReaderFile::makeUpper(std::string& one_string) noexcept {
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
