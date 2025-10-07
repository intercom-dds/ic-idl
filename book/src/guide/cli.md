# Command Line Interface Reference

Complete reference for the `ic-idl` command-line tool.

## Synopsis

```bash
ic-idl [OPTIONS] <files>...
```

## Basic Usage

```bash
# Generate Rust code from a single IDL file
ic-idl --rust-out src/generated schema.idl

# Generate Python code
ic-idl --python-out python/ schema.idl

# Process multiple files
ic-idl --rust-out rust/ types.idl services.idl models.idl

# Generate for multiple backends simultaneously
ic-idl --rust-out rust/ --python-out python/ --cpp-out cpp/ schema.idl
```

---

## General Options

### `-E, --preprocessor-only`

Run only the preprocessor, outputting the preprocessed IDL without code generation.

```bash
ic-idl --preprocessor-only schema.idl > preprocessed.idl
```

Useful for:
- Debugging macro expansions
- Seeing the result of `#include` directives
- Understanding conditional compilation

### `-H, --no-header-follow`

Do not generate code for included files, only for the files specified on the command line.

```bash
ic-idl --no-header-follow main.idl
```

By default, IC-IDL generates code for all included files. This option generates code only for `main.idl`, not for any files it includes.

### `-I, --include <dir>`

Add a directory to the include search path for `#include` directives.

```bash
ic-idl -I /usr/include/idl -I ./common schema.idl
```

Multiple include directories can be specified. Files are searched in order:
1. Current directory
2. Directories specified with `-I` (in order)

### `-D, --define <name>[=<value>]`

Define a preprocessor macro.

```bash
# Define a macro without a value
ic-idl -D DEBUG schema.idl

# Define a macro with a value
ic-idl -D VERSION=2 -D MAX_SIZE=1024 schema.idl
```

These macros can be used in `#ifdef`, `#if`, and other preprocessor directives.

### `-l, --list`

List the files that would be generated without actually generating them.

```bash
ic-idl --list schema.idl --rust-out output/
```

Useful for:
- Preview generation before running
- Build system integration
- Understanding what files will be created

### `--purge-dirs`

Remove all files in output directories before generating code.

```bash
ic-idl --purge-dirs schema.idl --rust-out src/generated/
```

**⚠️ Warning**: This deletes all files in the output directory, not just generated files.

### `--ignore-comments`

Do not parse documentation comments.

```bash
ic-idl --ignore-comments schema.idl
```

Slightly faster compilation when documentation comments aren't needed.

### `-W <lint>`

Enable or disable specific warnings. See [Warning Types](#warning-types) below.

```bash
# Enable a specific warning
ic-idl -W unused-types schema.idl

# See all available warnings
ic-idl -W help
```

### `-Z <flag>`

Enable unstable/experimental features.

```bash
# See available unstable features
ic-idl -Z help
```

### `-V, --version`

Display version information.

```bash
ic-idl --version
```

Output includes:
- Version number
- Git commit hash
- Build target and type

### `-h, --help`

Display help information.

```bash
ic-idl --help
```

---

## Backend Options

### Rust Backend

**`--rust-out <dir>`**

Generate Rust code in the specified directory.

```bash
ic-idl schema.idl --rust-out src/generated
```

**`--no-rename`**

Do not automatically rename types to follow Rust conventions.

```bash
ic-idl schema.idl --rust-out out/ --no-rename
```

By default, types are converted to Rust naming conventions (e.g., `snake_case` → `SnakeCase`). This option disables that.

**`--must-use`**

Annotate all types with `#[must_use]`.

```bash
ic-idl schema.idl --rust-out out/ --must-use
```

Generates Rust code where function results and types are marked with `#[must_use]`, causing compiler warnings if values are ignored.

### Python Backend

**`--python-out <dir>`**

Generate Python code in the specified directory.

```bash
ic-idl schema.idl --python-out src/
```

**`--use-pep8`**

Rename types to conform to PEP-8 style guidelines.

```bash
ic-idl schema.idl --python-out out/ --use-pep8
```

Converts type names to PEP-8 conventions (e.g., `MyType` stays `MyType`, `my_type` becomes `MyType`).

**`--global-postfix <suffix>`**

Add a suffix to global module names.

```bash
ic-idl schema.idl --python-out out/ --global-postfix _pb
```

Useful for avoiding naming conflicts when integrating with other generated code.

### C++ Backend

**`--cpp-out <dir>`**

Generate C++ code in the specified directory.

```bash
ic-idl schema.idl --cpp-out include/
```

**`--scoped-enums`**

Generate C++11 scoped enums (`enum class`) instead of unscoped enums.

```bash
ic-idl schema.idl --cpp-out out/ --scoped-enums
```

Generates:
```cpp
enum class Color { Red, Green, Blue };
// vs
enum Color { Red, Green, Blue };
```

**`--no-stream-op`**

Do not generate `operator<<` for ostream serialization.

```bash
ic-idl schema.idl --cpp-out out/ --no-stream-op
```

By default, IC-IDL generates streaming operators for easy debugging.

**`--use-fmt`**

Generate formatting specializations for the {fmt} library.

```bash
ic-idl schema.idl --cpp-out out/ --use-fmt
```

Generates `fmt::formatter` specializations for all types.

**`--dll-export <symbol>`**

Add DLL export macros to generated classes.

```bash
ic-idl schema.idl --cpp-out out/ --dll-export MY_API
```

Generates:
```cpp
class MY_API MyClass { ... };
```

Useful for building Windows DLLs.

**`--header-ext <extension>`**

Use a custom file extension for header files (default: `.h`).

```bash
ic-idl schema.idl --cpp-out out/ --header-ext .hpp
```

**`--header-subdir <dir>`**

Store header files in a subdirectory.

```bash
ic-idl schema.idl --cpp-out out/ --header-subdir include
```

Generates headers in `out/include/` instead of `out/`.

### IDL Backend

**`--idl-out <dir>`**

Generate normalized/reformatted IDL files.

```bash
ic-idl schema.idl --idl-out clean/
```

Useful for:
- Code formatting
- Standardizing style across projects
- Removing comments and preprocessing directives

**`--idl-doxygen`**

Output Doxygen-compatible IDL files.

```bash
ic-idl schema.idl --idl-out docs/ --idl-doxygen
```

Converts comments to Doxygen format.

**`--idl-legacy`**

Emit IDL compatible with older parsers.

```bash
ic-idl schema.idl --idl-out out/ --idl-legacy
```

Avoids using newer IDL 4.x features for compatibility.

### Protocol Buffers Backend

**`--proto-out <dir>`**

Generate Protocol Buffers (`.proto`) files.

```bash
ic-idl schema.idl --proto-out proto/
```

Converts IDL to Protocol Buffers format for use with `protoc`.

### JSON Backend

**`--json-out <dir>`**

Emit a structural JSON description of the schema (useful for tooling and
debugging).

```bash
ic-idl schema.idl --json-out json/
```

**`--json-schema-out <dir>`**

Generate JSON Schema files.

```bash
ic-idl schema.idl --json-schema-out schemas/
```

Creates JSON Schema (draft-07) definitions for validation.

### XML Backend

**`--xml-out <dir>`**

Generate XML serialization code.

```bash
ic-idl schema.idl --xml-out xml/
```

---

## Warning Types

Use `-W <warning>` to enable specific warnings:

### `-W help`

List all available warnings.

```bash
ic-idl -W help
```

### Common Warnings

- `unused-types`: Warn about defined but unused types
- `unused-constants`: Warn about defined but unused constants
- `shadowing`: Warn when names shadow earlier declarations
- `deprecated`: Warn about use of deprecated features
- `keywords`: Warn when identifiers are keywords in target languages

### Enabling/Disabling Warnings

```bash
# Enable a warning
ic-idl -W unused-types schema.idl

# Disable a warning (prefix with 'no-')
ic-idl -W no-keywords schema.idl

# Enable all warnings
ic-idl -W all schema.idl

# Treat warnings as errors
ic-idl -W error schema.idl
```

---

## Examples

### Generate Rust and Python

```bash
ic-idl types.idl \
    --rust-out src/generated \
    --python-out ../python-client/generated
```

### With Preprocessing

```bash
ic-idl -D PRODUCTION -I common/ schema.idl --rust-out out/
```

### Clean Build

```bash
ic-idl schema.idl --rust-out target/generated/ --purge-dirs
```

### Multiple Files

```bash
ic-idl -I idl/ common.idl types.idl services.idl \
    --rust-out rust/ \
    --cpp-out cpp/include/ --scoped-enums \
    --python-out python/ --use-pep8
```

### Preview Generation

```bash
# See what would be generated
ic-idl --list schema.idl --rust-out out/

# Then actually generate
ic-idl schema.idl --rust-out out/
```

---

## Exit Codes

- `0`: Success
- `1`: Compilation error (syntax, type checking, etc.)
- `2`: Invalid command-line arguments
- `3`: File not found or I/O error

---

## Environment Variables

### `IC_IDL_INCLUDE_PATH`

Additional directories to search for include files (colon-separated on Unix, semicolon-separated on Windows).

```bash
export IC_IDL_INCLUDE_PATH=/usr/local/include/idl:/opt/idl
ic-idl schema.idl
```

### `IC_IDL_NO_COLOR`

Disable colored output in error messages.

```bash
IC_IDL_NO_COLOR=1 ic-idl schema.idl
```

---

## Tips and Tricks

### Integration with Build Systems

**Makefile:**
```makefile
.PHONY: generate
generate:
	ic-idl -I idl/ schema.idl --rust-out src/generated/

clean:
	rm -rf src/generated/
```

**Cargo build.rs:**
```rust
fn main() {
    println!("cargo:rerun-if-changed=schema.idl");

    let status = std::process::Command::new("ic-idl")
        .args(&["schema.idl", "--rust-out", "src/generated"])
        .status()
        .expect("Failed to run ic-idl");

    if !status.success() {
        panic!("ic-idl failed");
    }
}
```

### Debugging Generation

1. Use `--preprocessor-only` to see preprocessed output
2. Use `--list` to preview generated files
3. Generate to a temporary directory first
4. Use `-W all` to catch potential issues

### Performance

For large projects:
- Use `--no-header-follow` if you only need specific files
- Use `--ignore-comments` if you don't need documentation
- Consider splitting large IDL files into modules

---

## See Also

- [Language Reference](language-reference.md) - IDL syntax and semantics
- [Quick Start](quickstart.md) - Getting started tutorial
- [Code Generation Guide](code-generation.md) - Backend-specific details
