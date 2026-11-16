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
| `bundled` | Yes | Compile the pinned Verovio fork |
| `prebuilt` | No | Legacy flag; this local fork still requires `bundled` |
| `force-rebuild` | No | Force fresh compilation, bypassing cache |

## Pinned source

This branch downloads native commit `6213a7313cd73438620adad853608500c06d8394`
from `btrkeks/verovio`. Before extraction or library cache lookup, it verifies
the archive SHA256
`edea3afcd0f6d2d2362114cc580a1ba33e06ee9f53dd083ad05197fa70a643ee`.
The verified archive is cached for offline reuse. Extraction is refreshed on
each build-script run. There is no upstream or prebuilt fallback.

For local development, an explicit clean Git checkout of the same commit may
replace the download, including on a library cache hit:

```bash
VEROVIO_SOURCE_DIR=/path/to/the/exact/native/commit RUSTC_WRAPPER= cargo build
```

The source guard rejects another revision, a subdirectory, and tracked, staged,
untracked or ignored changes. The legacy ignored `include/vrv/git_commit.h` is
the sole exception. Compilation uses a deterministic commit header in Cargo's
output directory and does not modify the source tree. Build and test outputs
must stay outside that source tree.

## Build caching

The compiled library lives under `target/verovio-cache/`, keyed by the build
script, source-guard implementation, source contents, and target. Subsequent
builds verify the archive or explicit source checkout before library cache lookup.

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
