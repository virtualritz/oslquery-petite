# Type System Refactoring

## Summary

This document describes the refactoring of the oslquery-petite type system from a C++-inspired struct-based approach to a more Rust-idiomatic enum-based design.

## Changes Made

### 1. Method Renaming (Completed)
- `nparams()` → `param_count()`
- `parameter(name)` → `param_name(name)`
- `param(index)` → `param_index(index)`

### 2. New Type System (`types_v2` module)

#### Before (C++ style):
```rust
struct TypeDesc {
    basetype: BaseType,
    arraylen: i32,
    is_closure: bool,
}

struct Parameter {
    name: Ustr,
    type_desc: TypeDesc,
    is_output: bool,
    is_struct: bool,
    valid_default: bool,
    varlen_array: bool,
    idefault: Vec<i32>,
    fdefault: Vec<f32>,
    sdefault: Vec<String>,
    // ... many more fields
}
```

#### After (Rust idiomatic):
```rust
enum TypeDesc {
    Scalar(SemanticType),
    Array { element_type: SemanticType, size: ArraySize },
    Closure { name: Ustr },
}

enum ParameterKind {
    Input(InputParameter),
    Output(OutputParameter),
}

struct Parameter {
    name: Ustr,
    kind: ParameterKind,
    metadata: Vec<Metadata>,
}
```

## Benefits

1. **Type Safety**: Impossible to create invalid states (e.g., output parameter with default value)
2. **Memory Efficiency**: No wasted Vec allocations for unused default types
3. **Better Pattern Matching**: Can directly match on parameter kinds and type descriptors
4. **Clearer Semantics**: Separation between numeric representation and semantic meaning
5. **Rust Idioms**: Uses enums for sum types instead of boolean flags

## Migration Path

The refactoring provides both old and new APIs:

- **Old API**: `oslquery_petite::OslQuery` (in `query` module)
- **New API**: `oslquery_petite::query_v2::OslQuery` (in `query_v2` module)

Conversion is provided via `OslQuery::from_v1()` method.

## API Comparison

### Old API:
```rust
let query = OslQuery::open("shader.oso")?;
for param in query.params() {
    if param.is_output {
        // output parameter
    }
    if param.valid_default {
        // has default - check all three vecs
        if !param.fdefault.is_empty() { /* float */ }
        if !param.idefault.is_empty() { /* int */ }
        if !param.sdefault.is_empty() { /* string */ }
    }
}
```

### New API:
```rust
let query = OslQuery::open("shader.oso")?;
for param in query.params() {
    match &param.kind {
        ParameterKind::Output(_) => {
            // output parameter
        }
        ParameterKind::Input(input) => {
            if let Some(default) = &input.default_value {
                match default {
                    ParameterValue::Float(v) => { /* float values */ }
                    ParameterValue::Int(v) => { /* int values */ }
                    ParameterValue::String(v) => { /* string values */ }
                }
            }
        }
    }
}
```

## Testing

All existing tests pass with both APIs. New tests in `tests/test_v2_api.rs` verify:
- Basic conversion from v1 to v2
- Array handling
- Input/output parameter separation
- Default value conversion

## Next Steps

1. Update parser to directly produce v2 types (currently converts via v1)
2. Add deprecation warnings to v1 API
3. Update documentation and examples
4. Consider making v2 the default in next major version