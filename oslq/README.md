# oslq

A command-line tool for querying OSL (Open Shading Language) shader parameters & metadata.

## Installation

```bash
cargo install oslq
```

## Usage

```bash
# Query shader parameters
oslq shader.oso

# Verbose output
oslq -v shader.oso

# Query specific parameter
oslq --param paramname shader.oso

# JSON output (if built with json feature)
oslq --json shader.oso
```

## Features

- Colored output.
- Support for all OSL types including arrays and aggregates.
- Compatible with `oslinfo` output format.
- JSON output support (optional feature).

## License

Licensed under Apache-2.0 OR BSD-3-Clause OR MIT OR Zlib at your option.
