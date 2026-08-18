# Project guide

`ic-idl` is a Rust workspace for an OMG IDL 4.2 compiler and reusable compiler components.

## Compilation architecture

Follow the existing pipeline when deciding where a change belongs:

1. `ic-vfs` owns files, source text, IDs, and spans.
2. `ic-lexer` tokenizes IDL. `ic-preproc` handles C-style preprocessing and include expansion.
3. `ic-parse` is a relaxed recursive-descent parser. It produces the source-shaped AST in `ic-syntax`; semantic validity does not belong in the parser unless syntax cannot be represented.
4. `ic-lint` checks syntax and resolved HIR, including accepted extensions that pedantic IDL mode may reject.
5. `ic-hir-lower` resolves names, evaluates values, and type-checks. `ic-hir` defines the canonical typed, resolved graph consumed by analysis and code generation.
6. `ic-hir-xform` contains reusable graph transformations and fixups.
7. `ic-codegen-*` crates consume resolved HIR and return `ic_emit::File` values. `crates/ic-idl/src/main.rs` selects backends and owns filesystem writes.

Supporting crates have narrow roles: `ic-diagnostic` renders diagnostics, `ic-expr` handles constant expressions, `ic-alloc` provides compiler data structures, `ic-cli` and `ic-cli-derive` implement command parsing, and `ic-hir-tree` renders HIR dumps. `crates/xtask` owns repository automation.

## Tests

- Diagnostics and lints use `insta` snapshot tests.
- `codegen-tests/corpus` checks whether each backend can generate compilable or valid output. Every corpus case is run against all backends.
- `integration-tests/corpus` checks generated API shape and runtime semantics.

Integration and codegen tests can be run through `cargo xtask codegen` and `cargo xtask integration`, respectively.

## Conventions

- All AI-assisted commits must include a `Co-Authored-By: <agent name> <agent email>` trailer identifying agent used.
- Commit messages use `IC-XXX - <area>: <lowercase summary>` for Linear issue changes and `<area>: <lowercase summary>` otherwise.
- Do **not** use Conventional Commits prefixes (`feat:`, `fix:`, `chore:`, `fix(foo):`, etc.).

## Comments

- Do not add comments by default.
- No parentheticals. ASCII only.
- Add a comment only when code cannot clearly express a non-obvious invariant, safety requirement, or external contract, or the logic is genuinely difficult to understand.
- Comments must state precise facts about current behavior.
- Never restate what code already says.
- Never include speculation, uncertainty, implementation history, rejected approaches, debugging narrative, agent actions, or explanations of how code reached its current state. No epistemology.
- Never describe future work unless explicitly asked to add a TODO.
- Never use divider comments like "// ======= something here =======".

## Code Review Rules

### Respect PR scope
- Read the PR description before reviewing the diff.
- Treat explicitly deferred, omitted, unsupported, or out-of-scope work as intentional.
- Do not raise a finding because deliberately deferred functionality is absent.
- Only challenge an explicit scope decision if it makes the implemented change incorrect, unsafe, or incompatible with an existing contract.

### Avoid duplicate findings
- Inspect existing review comments and unresolved threads before posting when available. Missing access to review comments must not block the review.
- Do not restate an issue another reviewer has already identified unless you have materially new information.

### Findings vs suggestions
- Findings must describe a concrete, realistic defect and its observable consequence.
- Do not present implementation preferences as correctness findings.
- Provide non-blocking suggestions when the implementation could be meaningfully simpler, clearer, more maintainable, less duplicated, or more idiomatic.
- Explain the concrete benefit of a suggestion rather than merely proposing a different implementation.

### Review quality
- Prefer a small number of high-signal comments over exhaustive commentary.
- Do not manufacture findings because the review would otherwise be empty.
