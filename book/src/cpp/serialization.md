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
