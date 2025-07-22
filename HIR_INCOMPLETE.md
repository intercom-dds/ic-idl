# Incomplete Features in HIR Lowering Implementation

Based on analysis of the codebase, here are the incomplete features and missing implementations in the HIR lowering:

## 1. Missing AST to HIR Collection
- ~~**Bitset types**~~ - ✅ COMPLETED - Bitsets are now fully collected, resolved, and evaluated in HIR

## 2. Incomplete HIR Lowering Features

### Valuetype Members
- In `ic-ptree-lower/src/ast.rs` line 494:
  ```rust
  // TODO: members
  let ty = sys::create_valuetype_finish(state, ptr::null_mut());
  ```
- Valuetype members are not being lowered from AST to ptree

### Numeric Expression Types
- In `ic-ptree-lower/src/hir.rs`, the `lower_numeric` function has a catch-all that returns null:
  ```rust
  _ => ptr::null(),
  ```
- Missing support for:
  - `Numeric::Null` - Null literals
  - `Numeric::Array` - Array initializers like `{1, 2, 3}`
  - `Numeric::Sequence` - Sequence initializers

### Void Type Handling
- In `ic-ptree-lower/src/ast.rs` line 155:
  ```rust
  // FIXME(idarcar): this is just a hack to make void work
  ```
- Void type handling is implemented as a workaround

## 3. Expression Evaluation Limitations
- In `ic-hir/src/lower/evaluate.rs`:
  - Null literals are not supported in constant expressions

## 4. Missing Type System Features
The following AST types are not fully represented or lowered:
- Bitset type definitions and their fields
- Complete valuetype implementations with members and inheritance
- Complex initializer expressions (arrays, sequences)

## 5. Annotation System
While annotations are implemented, default values in annotations are only preserved in the AST lowering path, not the HIR path (as noted in comments).

## Summary of Major Missing Features:
1. ~~**Bitset types**~~ - ✅ COMPLETED
2. **Valuetype members** - Collected but not lowered
3. **Complex numeric expressions** - Array/Sequence initializers not supported
4. **Expression evaluation** - Null literals not supported in constants
5. **Void type** - Implemented as a hack