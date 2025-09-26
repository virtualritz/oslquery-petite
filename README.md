# OSLQuery Workspace

Rust implementation for querying [Open Shading Language](https://github.com/AcademySoftwareFoundation/OpenShadingLanguage) (OSL) compiled shader metadata.

## Crates

This workspace contains two crates:

### [`oslquery-petite`](./oslquery-petite/)

The main library for parsing and querying OSL `.oso` files. Provides:

- Pure Rust parser for OSO format (no C++ dependencies)
- Type-safe API where parameter types and values are unified
- Support for all OSL types (int, float, color, point, vector, normal, matrix, arrays, closures)
- Metadata extraction
- Zero-copy string handling

[Full documentation →](./oslquery-petite/README.md)

### [`oslq`](./oslq/)

Command-line tool for inspecting OSO files, similar to OSL's `oslinfo`:

```bash
# Query shader parameters
oslq shader.oso

# Query specific parameter
oslq --param Kd shader.oso

# JSON output
oslq --json shader.oso
```

[Full documentation →](./oslq/README.md)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
oslquery-petite = "0.1"
```

Basic usage:

```rust
use oslquery_petite::{OslQuery, TypedParameter};

let query = OslQuery::open("shader.oso")?;
println!("Shader: {} ({})", query.shader_name(), query.shader_type());

for param in query.params() {
    match param.typed_param() {
        TypedParameter::Color { default: Some([r, g, b]), .. } => {
            println!("  {} : color = [{}, {}, {}]", param.name, r, g, b);
        }
        TypedParameter::Float { default: Some(val) } => {
            println!("  {} : float = {}", param.name, val);
        }
        _ => {}
    }
}
```

## Installation

For the command-line tool:
```bash
cargo install --path oslq
```

For library usage, see the Quick Start above.

## License

BSD 3-Clause (same as OpenShadingLanguage)