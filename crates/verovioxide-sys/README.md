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

This branch is a local-only development pin for the bounded clef importer
repair at native commit `a2f19019932d1c1d98c66370573eb0e5f10c2d70`.
`build.rs` pins that exact revision.
It requires an explicit clean Git checkout, including on a library cache hit:

```bash
VEROVIO_SOURCE_DIR=/path/to/the/exact/native/commit RUSTC_WRAPPER= cargo build
```

The source guard rejects a missing path, another revision, a subdirectory,
and tracked, staged, untracked or ignored changes. The legacy ignored
`include/vrv/git_commit.h` is the sole exception. Compilation uses a deterministic
commit header in Cargo's output directory and does not modify the source tree.
Build and test outputs must stay outside that source tree.

There is no archive download, archive checksum, upstream source or prebuilt
fallback on this branch. This native commit is unpublished. A portable release
requires a separate published pin with a verified archive checksum.

## Build caching

The compiled library lives under `target/verovio-cache/`, keyed by the build
script, source-guard implementation, source contents, and target. Subsequent
builds validate the exact source checkout again before library cache lookup.

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
