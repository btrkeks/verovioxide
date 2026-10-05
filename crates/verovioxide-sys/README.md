# verovioxide-sys

[![][crate-badge]][crate]
[![][docs-badge]][docs]

Raw FFI bindings to the [Verovio](https://www.verovio.org/) music notation engraving library.

## Overview

This crate provides low-level C bindings to Verovio. Most users should use the high-level [`verovioxide`](https://crates.io/crates/verovioxide) crate instead.

## Installation

```bash
cargo add verovioxide-sys
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `bundled` | Yes | Compile the pinned local Verovio fork |
| `prebuilt` | No | Legacy flag; this local fork still requires `bundled` |
| `force-rebuild` | No | Force fresh compilation, bypassing cache |

## Pinned source

This fork builds the `clef` branch of `btrkeks/verovio` at commit `35b35a2eacee5748616349387309d314a4448d48`,
based on version 6.3.0. With no override, the build downloads that exact
commit archive from `btrkeks/verovio` and requires SHA256
`a52e446bf523352e196bea427a65276220c6ba980e380b158e451ebc04024007`.
It caches the verified archive under `target/verovio-cache/` and extracts
fresh source into Cargo's output directory before library cache lookup.
A cached archive allows subsequent builds without network access. A failed
download or checksum mismatch stops the build; there is no upstream,
submodule, or prebuilt-library fallback.

An optional local override must point to a clean Git checkout of the same
commit:

```bash
VEROVIO_SOURCE_DIR=/path/to/verovio-clef cargo build
```

The override rejects a missing path, another revision, or any tracked,
staged, untracked, or ignored changes. The old ignored
`include/vrv/git_commit.h` may remain from earlier builds; the compiler
uses a deterministic commit header in Cargo's output directory. The build
does not modify the override checkout. An invalid override stops the build
without using the downloaded archive or a cached library.

## Build caching

The compiled library lives under `target/verovio-cache/`, keyed by the build
script, source inputs, and target. Subsequent builds use that library only
after verifying the archive or validating the override checkout again.

To force a fresh recompilation:

```bash
cargo build --features force-rebuild
```

## Verify the source guard

These tests use temporary Git repositories and do not compile Verovio:

```bash
rustc --edition 2024 --test crates/verovioxide-sys/build_source.rs -o /tmp/verovio-source-guard-tests
/tmp/verovio-source-guard-tests
```

## Related Crates

- [`verovioxide`](https://crates.io/crates/verovioxide) - High-level safe Rust API
- [`verovioxide-data`](https://crates.io/crates/verovioxide-data) - Bundled SMuFL fonts and resources

## License

This project is licensed under the Apache License 2.0.

Verovio is licensed under the LGPL-3.0.

[crate]: https://crates.io/crates/verovioxide-sys
[crate-badge]: https://img.shields.io/crates/v/verovioxide-sys.svg
[docs]: https://docs.rs/verovioxide-sys/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
