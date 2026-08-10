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
| `bundled` | Yes | Compile Verovio C++ library from source |
| `prebuilt` | No | Download pre-built library from GitHub releases (faster) |
| `force-rebuild` | No | Force fresh compilation, bypassing cache |

### Faster Builds with Prebuilt Binaries

For faster initial builds, enable the `prebuilt` feature:

```toml
[dependencies]
verovioxide-sys = { version = "0.3", features = ["prebuilt"] }
```

Prebuilt binaries are available for:

- macOS (x86_64, aarch64)
- Linux (x86_64, aarch64)
- Windows (x86_64 MSVC)

If prebuilt binaries aren't available for your platform, it automatically falls back to compiling from source.

## Build Caching

The Verovio C++ library is compiled once and cached at `target/verovio-cache/`. Subsequent builds link to the cached library and complete in seconds.

To force a fresh recompilation:

```bash
cargo build --features force-rebuild
```

## Corporate/Restricted Networks

If your network blocks GitHub downloads, provide a local Verovio source:

```bash
VEROVIO_SOURCE_DIR=/path/to/verovio cargo build
```

## Verovio Version

This crate bundles Verovio 6.2.1.

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
