# Copyright 2024 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

add_library(utils STATIC
    src/string_utils.cpp)
   # src/CdrSerializer.cpp
   # src/Checksums.cpp
   # src/Environment.cpp
   # src/Exception.cpp
   # src/EventQueue.cpp
   # src/filesystem.cpp
   # src/Global.cpp
   # src/GlobalHandleMap.cpp
   # src/Guid.cpp
   # src/HumanReadable.cpp
   # src/IcomCondition.cpp
   # src/JsonParser.cpp
   # src/LogBuilder.cpp
   # src/Logger.cpp
   # src/LoggerFile.cpp
   # src/LoggerPrint.cpp
   # src/LogStack.cpp
   # src/MemoryMappedFile.cpp
   # src/Mutex.cpp
   # src/ProcessProperties.cpp
   # src/Properties.cpp
   # src/ReferableEntity.cpp
   # src/Registration.cpp
   # src/ShmRingBuffer.cpp
   # src/SocketTools.cpp
   # src/StateObject.cpp
   # src/String.cpp
   # src/StringUtils.cpp
   # src/Thread.cpp
   # src/Time.cpp
   # src/Toml.cpp
   # src/Util.cpp
   # src/XMLReader.cpp
   # src/XMLReaderFile.cpp
   # src/XMLTypes.cpp
   # src/XMLWriter.cpp
   # src/ipc.cpp
   # ${TARGET_HEADERS} )
