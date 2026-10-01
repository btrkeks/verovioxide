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

## Local source requirement

This development fork requires a clean Git checkout of Verovio commit
`5a02114b5abf25dc938f634a0018a20f8513479b`, based on version 6.2.1. Supply it
explicitly for every build:

```bash
VEROVIO_SOURCE_DIR=/path/to/verovio-fingering-layer cargo build
```

The build rejects a missing path, another revision, or any tracked, staged,
untracked, or ignored changes. The old ignored `include/vrv/git_commit.h`
may remain from earlier builds; this build always overrides it with a
commit header in Cargo's output directory. It does not modify the source
checkout.

Validation runs before cached libraries can be used. There is no upstream,
submodule, download, or prebuilt-library fallback. This fork is local-only
until its source pin is published through a separate approved change.

## Build caching

The compiled library lives under `target/verovio-cache/`, keyed by the build
script, source inputs, and target. Subsequent builds use that library only
after the source checkout passes validation again.

To force a fresh recompilation:

```bash
VEROVIO_SOURCE_DIR=/path/to/verovio-fingering-layer cargo build --features force-rebuild
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
