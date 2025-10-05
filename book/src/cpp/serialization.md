# Copyright 2025 KONGSBERG
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
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

# Serialization

The generated C++ code integrates with the CTS runtime shipped in
`library/cpp/defs`. Every struct/union/valuetype implements `marshal` and
`unmarshal` member functions that operate on CTS archives (CDR, JSON, …).

## CDR example

Consult the runtime headers under `ic_cts/cdr_serializer.h` and
`ic_cts/json_serializer.h` for the archive classes that pair with the generated
`marshal`/`unmarshal` functions. The CTS layer offers little- and big-endian
CDR archives as well as JSON encoders. A typical pattern is:

1. Construct an archive object.
2. Pass it to `marshal`/`unmarshal`.
3. Consume the resulting buffer or parse incoming data into the archive.

Exact class names may vary as the runtime evolves; always refer to the headers
in your checked-out version for the latest API.

## Optional members and unions

Optional fields carry `dcps::xtypes::IS_OPTIONAL` flags in their metadata. The
runtime consults these flags when encoding so you can leave fields at their
constructed defaults to omit them. Unions emit discriminant-aware helper
functions that ensure only the active case is serialised.

## Interoperability

Because the same metadata drives the Rust and Python runtimes, binaries produced
in C++ interoperate with those languages provided you use the same archive type
(CDR endianness, JSON flavour, …).

Refer to the headers under `library/cpp/defs/ic_cts/detail` for the low-level
APIs exposed by the runtime.
