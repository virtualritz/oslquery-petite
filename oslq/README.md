# `oslq` – A Petite `oslinfo` Clone

[![Crates.io](https://img.shields.io/crates/v/oslq.svg)](https://crates.io/crates/oslq)
[![Docs.rs](https://docs.rs/oslq/badge.svg)](https://docs.rs/oslq)

A command-line tool for querying OSL (Open Shading Language) shader parameters & metadata.

## Installation

```bash
cargo install oslq
```

## Usage

```bash
# Query a shader.
oslq shader.oso

# Query multiple shaders.
oslq shader1.oso shader2.oso

# Query specific parameter.
oslq --param Kd shader.oso

# Use search path.
oslq -p /path/to/shaders:./local shader

# Verbose output.
oslq -v shader.oso

# JSON output (requires json feature).
oslq --json shader.oso

# Benchmark parsing.
oslq --runstats shader.oso
```

## Features

- Colored output.
- Support for all OSL types including arrays and aggregates.
- Compatible with `oslinfo` output format.
- JSON output support (optional feature).

## License

Licensed under Apache-2.0 OR BSD-3-Clause OR MIT OR Zlib at your option.
