# Verovioxide

[![][build-badge]][build]
[![][crate-badge]][crate]
[![][tag-badge]][tag]
[![][docs-badge]][docs]

[![][logo]][logo-large]

*Safe Rust bindings to the [Verovio](https://www.verovio.org/) music notation engraving library*

## Features

- **Multi-format input**: Load MusicXML, MEI, ABC, Humdrum, and Plaine & Easie notation
- **Multi-format output**: Render to SVG, export to MEI/Humdrum/PAE, generate MIDI
- **Bundled fonts**: Leipzig, Bravura, Gootville, Leland, and Petaluma (SMuFL-compliant)
- **Type-safe API**: Builder pattern for options with serde serialization
- **Zero runtime dependencies**: Verovio statically linked
- **Complete API coverage**: 100% of the Verovio C++ API wrapped in safe Rust
- **Production ready**: Comprehensive error handling and 95%+ test coverage
- **PNG support**: Verovioxide extends beyond the Verovio C++ API with PNG output (in-memory and to-file)

## Installation

```bash
cargo add verovioxide
```

The first build compiles the Verovio C++ library from source, which takes several minutes. Subsequent builds use cached artifacts and are fast.

### Faster Builds with Prebuilt Binaries

For faster initial builds, use the prebuilt feature which downloads a pre-compiled Verovio library:

```bash
cargo add verovioxide-sys --features prebuilt
cargo add verovioxide
```

Or in your `Cargo.toml`:

```toml
[dependencies]
verovioxide = "0.3"
verovioxide-sys = { version = "0.3", features = ["prebuilt"] }
```

Prebuilt binaries are available for:

- macOS (x86_64, aarch64)
- Linux (x86_64, aarch64)
- Windows (x86_64 MSVC)

If prebuilt binaries aren't available for your platform, it automatically falls back to compiling from source.

## Quick Start

```rust
use verovioxide::{Toolkit, Options, Result, Svg};
use std::path::Path;

fn main() -> Result<()> {
    // Create a Verovio toolkit with bundled resources
    let mut voxide = Toolkit::new()?;

    // Load notation (format auto-detected)
    voxide.load(Path::new("score.musicxml"))?;

    // Configure rendering
    let options = Options::builder()
        .scale(100)
        .adjust_page_height(true)
        .build();
    voxide.set_options(&options)?;
    voxide.render_to("score.svg")?;
    Ok(())
}
```

## Rendering

Verovioxide provides a unified render API with builder pattern for type-safe, consistent output.

### In-Memory Rendering

Use `render()` with format builders:

```rust
use verovioxide::{Svg, Midi, Mei, Timemap, Humdrum, Pae, ExpansionMap};

// SVG rendering
let svg: String = voxide.render(Svg::page(1))?;           // Single page
let svg: String = voxide.render(Svg::page(1).with_declaration())?;
let pages: Vec<String> = voxide.render(Svg::all_pages())?; // All pages
let pages: Vec<String> = voxide.render(Svg::pages(2, 5))?; // Page range

// Other formats
let midi: String = voxide.render(Midi)?;          // Base64-encoded MIDI
let mei: String = voxide.render(Mei)?;            // MEI XML
let humdrum: String = voxide.render(Humdrum)?;    // Humdrum/Kern
let pae: String = voxide.render(Pae)?;            // Plaine & Easie
let timemap: String = voxide.render(Timemap)?;    // JSON timing data
let expansion: String = voxide.render(ExpansionMap)?;
```

### File Rendering

Use `render_to()` for simple cases (format inferred from extension):

```rust
voxide.render_to("output.svg")?;    // SVG page 1
voxide.render_to("output.mid")?;    // MIDI
voxide.render_to("output.mei")?;    // MEI
```

Use `render_to_as()` for explicit control:

```rust
voxide.render_to_as("output.svg", Svg::page(3))?;        // Specific page
voxide.render_to_as("output.svg", Svg::all_pages())?;    // Creates output/ directory
voxide.render_to_as("output.json", Timemap)?;            // Disambiguate .json
```

### Format-Specific Options

Timemap and MEI support typed options:

```rust
// Timemap with options
let timemap = voxide.render(
    Timemap::with_options()
        .include_measures(true)
        .include_rests(false)
)?;

// MEI with options
let mei = voxide.render(
    Mei::with_options()
        .remove_ids(true)
        .page_based(false)
)?;
```

### Page Information

```rust
let count = voxide.page_count();
println!("Document has {} pages", count);
```

### Legacy Methods

The original render methods remain available for backwards compatibility:

```rust
let svg = voxide.render_to_svg(1)?;
let pages = voxide.render_all_pages()?;
let mei = voxide.get_mei()?;
let humdrum = voxide.get_humdrum()?;
let midi = voxide.render_to_midi()?;
let timemap = voxide.render_to_timemap()?;
```

## Querying Elements

Verovioxide provides a unified query API with type-safe return values.

### Element Queries

Use `get()` with query types for type-safe element information:

```rust
use verovioxide::{Toolkit, Page, Attrs, Time, Times};

let mut voxide = Toolkit::new()?;
voxide.load("score.mei")?;

// Get page containing an element
let page: u32 = voxide.get(Page::of("note-001"))?;

// Get element attributes as JSON
let attrs: String = voxide.get(Attrs::of("note-001"))?;

// Get timing information
let time: f64 = voxide.get(Time::of("note-001"))?;
let times: String = voxide.get(Times::of("note-001"))?;
```

### Time-Based Queries

```rust
use verovioxide::{Toolkit, Elements};

// Get elements sounding at 5000ms
let elements: String = voxide.get(Elements::at(5000))?;
```

### Descriptive Features

```rust
use verovioxide::{Toolkit, Features};

let features: String = voxide.get(Features)?;
```

### Legacy Methods

The original query methods remain available for backwards compatibility:

```rust
let page = voxide.get_page_with_element("note-001")?;
let attrs = voxide.get_element_attr("note-001")?;
let time = voxide.get_time_for_element("note-001")?;
let elements = voxide.get_elements_at_time(5000)?;
```

## Configuration

### Options Builder

The `Options` builder provides type-safe configuration:

```rust
use verovioxide::{Options, BreakMode, HeaderMode, FooterMode};

let options = Options::builder()
    // Page dimensions (MEI units, 10 units = 1mm)
    .page_width(2100)      // A4 width
    .page_height(2970)     // A4 height
    .adjust_page_height(true)

    // Margins
    .page_margin(50)
    .page_margin_top(100)

    // Scale and spacing
    .scale(100)            // Percentage
    .spacing_staff(12)
    .spacing_system(6)

    // Font
    .font("Leipzig")       // or "Bravura", "Gootville", "Leland", "Petaluma"
    .lyric_size(0.8)

    // Layout
    .breaks(BreakMode::Auto)
    .header(HeaderMode::None)
    .footer(FooterMode::None)

    // SVG output
    .svg_view_box(true)
    .svg_remove_xlink(true)
    .svg_css("svg { background: white; }")

    // MIDI generation
    .midi_tempo(120.0)
    .midi_velocity(80)

    // Transposition
    .transpose("M2")       // Up a major second

    .build();

voxide.set_options(&options)?;
```

### Available Options

| Category | Options |
|----------|---------|
| **Page** | `page_width`, `page_height`, `adjust_page_height`, `page_margin`, `page_margin_top`, `page_margin_bottom`, `page_margin_left`, `page_margin_right` |
| **Scale/Spacing** | `scale`, `spacing_staff`, `spacing_system`, `spacing_linear`, `spacing_non_linear`, `even_note_spacing`, `min_measure_width` |
| **Font** | `font`, `lyric_size` |
| **Layout** | `breaks`, `condense`, `condense_first_page`, `condense_tempo_pages`, `header`, `footer` |
| **SVG** | `svg_xml_declaration`, `svg_bounding_boxes`, `svg_view_box`, `svg_remove_xlink`, `svg_css`, `svg_format_raw`, `svg_font_face_include` |
| **MIDI** | `midi_tempo`, `midi_velocity` |
| **Input** | `input_from`, `mdiv_x_path_query`, `expansion` |
| **Transposition** | `transpose`, `transpose_selected_only`, `transpose_to_sounding_pitch` |

### Option Modes

```rust
// Break modes
BreakMode::Auto     // Automatic page/system breaks
BreakMode::None     // No automatic breaks
BreakMode::Encoded  // Use breaks from input file
BreakMode::Line     // Break at each line
BreakMode::Smart    // Smart break placement

// Condense modes
CondenseMode::None
CondenseMode::Auto
CondenseMode::Encoded

// Header/Footer modes
HeaderMode::None | HeaderMode::Auto | HeaderMode::Encoded
FooterMode::None | FooterMode::Auto | FooterMode::Encoded | FooterMode::Always
```

### JSON Serialization

Options can be serialized/deserialized:

```rust
let json = options.to_json()?;
let options = Options::from_json(&json)?;
```

## Supported Input Formats

| Format | Extensions | Description |
|--------|------------|-------------|
| MusicXML | `.musicxml`, `.xml`, `.mxl` | Standard music interchange format |
| MEI | `.mei` | Music Encoding Initiative XML |
| ABC | `.abc` | Text-based notation format |
| Humdrum | `.krn`, `.hmd` | Kern and other Humdrum formats |
| PAE | — | Plaine & Easie Code (RISM) |

Format detection is automatic based on file content.

## Supported Output Formats

| Format | Unified API | Description |
|--------|-------------|-------------|
| SVG | `render(Svg::page(1))` | Scalable vector graphics for display |
| MEI | `render(Mei)` | Music Encoding Initiative XML |
| Humdrum | `render(Humdrum)` | Humdrum/Kern format |
| PAE | `render(Pae)` | Plaine & Easie Code |
| MIDI | `render(Midi)` | Base64-encoded MIDI for playback |
| Timemap | `render(Timemap)` | JSON timing data for synchronization |
| Expansion Map | `render(ExpansionMap)` | JSON expansion/repeat data |
| PNG | `render(Png::page(1))` | Raster image for display/printing |

## PNG Rendering

PNG support is enabled by default via the `png` feature. It uses [resvg](https://github.com/linebender/resvg) for high-quality SVG-to-PNG conversion.

### In-Memory PNG

```rust
use verovioxide::Png;

// Basic rendering (returns Vec<u8>)
let png_bytes: Vec<u8> = voxide.render(Png::page(1))?;

// With options
let png_bytes = voxide.render(
    Png::page(1)
        .width(800)           // Scale to 800px width
        .height(600)          // Or scale to fit height
        .scale(2.0)           // Or use zoom factor
        .white_background()   // White instead of transparent
)?;

// Custom background color
let png_bytes = voxide.render(
    Png::page(1).background(240, 240, 240, 255)  // Light gray
)?;

// Render all pages
let all_pngs: Vec<Vec<u8>> = voxide.render(Png::all_pages())?;

// Render page range
let pngs: Vec<Vec<u8>> = voxide.render(Png::pages(2, 5))?;
```

### File Rendering

```rust
// Format inferred from extension
voxide.render_to("output.png")?;

// Explicit control
voxide.render_to_as("output.png", Png::page(3))?;
voxide.render_to_as("output.png", Png::all_pages())?;  // Creates output/ directory
```

### Terminal Display with viuer

The PNG bytes are compatible with the `image` crate for terminal display:

```rust
use verovioxide::Png;
use image::load_from_memory;
use viuer::Config;

let png_bytes = voxide.render(Png::page(1).width(800))?;
let img = load_from_memory(&png_bytes)?;
viuer::print(&img, &Config::default())?;
```

### Disabling PNG

To disable PNG support (reduces compile time and binary size):

```toml
[dependencies]
verovioxide = { version = "0.3", default-features = false, features = ["bundled-data"] }
```

## Feature Flags

### verovioxide

| Feature | Default | Description |
|---------|---------|-------------|
| `bundled-data` | Yes | Include bundled SMuFL fonts and resources |
| `png` | Yes | PNG rendering support via resvg |
| `font-leipzig` | Yes | Leipzig SMuFL font (default font) |
| `font-bravura` | No | Bravura SMuFL font |
| `font-gootville` | No | Gootville SMuFL font |
| `font-leland` | No | Leland SMuFL font |
| `font-petaluma` | No | Petaluma SMuFL font |
| `all-fonts` | No | Enable all fonts |

Note: Bravura baseline data is always included as it is required for Verovio's glyph name table.

### verovioxide-sys

| Feature | Default | Description |
|---------|---------|-------------|
| `bundled` | Yes | Compile Verovio C++ library from source |
| `prebuilt` | No | Download pre-built library from GitHub releases (faster) |
| `force-rebuild` | No | Force fresh compilation, bypassing cache |

The `prebuilt` and `bundled` features can be used together - if prebuilt download fails, it falls back to compilation.

### Enable Additional Fonts

```toml
[dependencies]
verovioxide = { version = "0.3", features = ["font-bravura", "font-leland"] }
```

### Custom Resource Path

To use your own Verovio resources instead of bundled data:

```toml
[dependencies]
verovioxide = { version = "0.3", default-features = false }
```

```rust
let voxide = Toolkit::with_resource_path(Path::new("/path/to/verovio/data"))?;
```

## Building from Source

Clone with submodules:

```bash
git clone --recursive https://github.com/oxur/verovioxide.git
cd verovioxide
```

Build and test:

```bash
cargo build
cargo test
```

### Build Caching

The Verovio C++ library is compiled once and cached at `target/verovio-cache/`. Subsequent builds link to the cached library and complete in seconds.

To force a fresh recompilation:

```bash
cargo build --features force-rebuild
```

### Corporate/Restricted Networks

If your network blocks GitHub downloads, you can provide a local Verovio source:

```bash
VEROVIO_SOURCE_DIR=/path/to/verovio cargo build
```

## Examples

### Render MusicXML to SVG

```bash
cargo run --example render_musicxml -- \
  test-fixtures/musicxml/simple.musicxml \
  simple.svg
```

[![][simple-screenshot]][simple-screenshot]

### Render ABC Notation

```bash
cargo run --example render_abc
```

[![][twinkle-screenshot]][twinkle-screenshot]

### Render All Pages

```bash
mkdir output-dir
cargo run --example render_all_pages -- \
  "examples/Goldberg-Variationen-1-and-2.musicxml" \
  output-dir/
```

```
Creating Verovio toolkit with bundled resources...
Verovio version: 6.2.1
Setting page dimensions: width=2100, height=2970 (A4-like)
Loading file: examples/Goldberg-Variationen-1-and-2.musicxml (format auto-detected)
[Warning] MusicXML import: Dangling ending tag skipped
Document loaded successfully. Total pages: 3
Rendering 3 pages...
  Page 1/3: output-dir/-001.svg (380226 bytes)
  Page 2/3: output-dir/-002.svg (424047 bytes)
  Page 3/3: output-dir/-003.svg (121350 bytes)
Done! Rendered 3 pages.
```

## Crate Structure

| Crate | Description |
|-------|-------------|
| **verovioxide** | High-level safe Rust API |
| **verovioxide-sys** | Low-level FFI bindings to Verovio C API |
| **verovioxide-data** | Bundled SMuFL fonts and resources |

## Technical Notes

### Thread Safety

`Toolkit` implements `Send` but not `Sync`. You can move a toolkit between threads, but cannot share references across threads. For concurrent rendering, create separate toolkit instances.

### Verovio Version

This release uses Verovio 6.2.1.

### Logging

```rust
// Enable logging to stderr
Toolkit::enable_log(true);

// Or capture to buffer
Toolkit::enable_log_to_buffer(true);
// ... operations ...
let log = voxide.get_log();
```

## License

This project is licensed under the Apache License 2.0.

**Dependencies:**

- [Verovio](https://www.verovio.org/) is licensed under the LGPL-3.0
- SMuFL fonts have their own licenses (see the respective font directories)

[//]: ---Named-Links---

[logo]: assets/images/logo/v1-x250.png
[logo-large]: assets/images/logo/v1.png
[build]: https://github.com/oxur/verovioxide/actions/workflows/ci.yml
[build-badge]: https://github.com/oxur/verovioxide/actions/workflows/ci.yml/badge.svg
[crate]: https://crates.io/crates/verovioxide
[crate-badge]: https://img.shields.io/crates/v/verovioxide.svg
[docs]: https://docs.rs/verovioxide/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
[tag-badge]: https://img.shields.io/github/tag/oxur/verovioxide.svg
[tag]: https://github.com/oxur/verovioxide/tags

[simple-screenshot]: assets/images/screenshots/simple-example.png
[twinkle-screenshot]: assets/images/screenshots/twinkle-example.png
