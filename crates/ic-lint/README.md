# ic-lint

A comprehensive linting framework for IDL (Interface Definition Language) files.

## Overview

ic-lint provides static analysis and diagnostics for IDL files, helping developers catch errors, maintain code quality, and adhere to language standards.

## Features

- **Multi-level analysis**: Both AST-based and HIR-based lints
- **Configurable severity**: Control warning levels per category or individual lint
- **Comprehensive coverage**: Syntax errors, semantic validation, style checks, and more
- **Clear diagnostics**: Detailed error messages with source locations

## Lint Categories

### Syntax (Error by default)
- `ann_members`: Validates annotation member syntax
- `ascii`: Ensures identifiers use ASCII characters
- `empty`: Checks for empty type definitions
- `sanity`: General sanity checks

### Semantic (Error by default)
- `keywords`: Prevents keywords used as identifiers
- `oneway`: Validates oneway operations return void

### Pedantic (Warning by default)
- `ambiguous_precedence`: Warns about potentially confusing operator precedence
- `array_param`: Checks for non-standard array parameters
- `assign_expr`: Warns about assignment operators on enums/bitmasks
- `bitmask_ann`: Checks bitmask usage in annotations
- `complex_lit`: Warns about complex literals
- `complex_key`: Warns about complex map key types (HIR)
- `empty_mod`: Detects empty modules
- `lowercase_bool`: Warns about lowercase `true`/`false`
- `null`: Checks for null union variants
- `omitted_in`: Warns about missing parameter directions
- `scoped_lit`: Checks for scoped literal usage

### Unsupported (Warning by default)
- `items`: Checks for unsupported language items
- `proto`: Validates proto3 compatibility (HIR)

### Annotation (Warning by default)
- `annotated_decl`: Checks annotated declarations

## Usage

```rust
use ic_lint::{lint_syntax, LintConfig, Category, Level};
use ic_vfs::SourceMap;

// Configure lints
let mut config = LintConfig::new();
config.set_category_level(Category::Pedantic, Level::Error);
config.set_lint_level("null", Level::Allow);

// Run lints
let report = lint_syntax_with_config(&ast, &vfs, &config);
```

## Adding New Lints

1. Create a module in the appropriate category directory
2. Implement the `Lint` trait
3. Register in `lint_syntax()` or `lint_hir()`
4. Add to `all_lint_names()`
5. Write tests using snapshot testing

See the crate documentation for detailed examples.