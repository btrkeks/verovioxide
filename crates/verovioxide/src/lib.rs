//! Safe Rust bindings to the Verovio music notation engraving library.
//!
//! Verovioxide provides a safe, idiomatic Rust API for [Verovio](https://www.verovio.org/),
//! a lightweight open-source library for engraving Music Encoding Initiative (MEI)
//! music scores as SVG.
//!
//! # Features
//!
//! - **Safe API**: All FFI calls are wrapped in safe Rust functions with proper error handling
//! - **Builder Pattern**: Fluent API for configuring rendering options
//! - **Multiple Formats**: Load MEI, MusicXML, Humdrum, ABC, and Plaine & Easie
//! - **SVG Output**: Render music notation as scalable vector graphics
//! - **Bundled Resources**: Optional embedded SMuFL fonts (enabled by default)
//!
//! # Quick Start
//!
//! ```no_run
//! use verovioxide::{Toolkit, Options, Result};
//!
//! fn main() -> Result<()> {
//!     // Create a toolkit with bundled resources
//!     let mut toolkit = Toolkit::new()?;
//!
//!     // Load MEI data
//!     let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
//!     <mei xmlns="http://www.music-encoding.org/ns/mei">
//!       <music><body><mdiv><score>
//!         <scoreDef><staffGrp>
//!           <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
//!         </staffGrp></scoreDef>
//!         <section><measure><staff n="1"><layer n="1">
//!           <note pname="c" oct="4" dur="4"/>
//!         </layer></staff></measure></section>
//!       </score></mdiv></body></music>
//!     </mei>"#;
//!
//!     toolkit.load_data(mei)?;
//!
//!     // Configure rendering options
//!     let options = Options::builder()
//!         .scale(100)
//!         .adjust_page_height(true)
//!         .build();
//!     toolkit.set_options(&options)?;
//!
//!     // Render to SVG
//!     let svg = toolkit.render_to_svg(1)?;
//!     println!("Rendered {} bytes of SVG", svg.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `bundled-data` (default): Include bundled SMuFL fonts and resources. Disable this
//!   feature if you want to provide your own resource path.
//!
//! # Loading Music Data
//!
//! Verovio auto-detects the input format. Supported formats include:
//!
//! - **MEI**: Music Encoding Initiative XML
//! - **MusicXML**: Standard interchange format
//! - **Humdrum**: Text-based music representation
//! - **ABC**: Simple text-based notation
//! - **PAE**: Plaine & Easie Code (RISM)
//!
//! ```no_run
//! use verovioxide::Toolkit;
//!
//! let mut toolkit = Toolkit::new().unwrap();
//!
//! // Load from string
//! toolkit.load_data("<mei>...</mei>").unwrap();
//!
//! // Or load from file
//! use std::path::Path;
//! toolkit.load_file(Path::new("score.musicxml")).unwrap();
//! ```
//!
//! # Rendering Options
//!
//! Use the [`Options`] builder to configure rendering:
//!
//! ```
//! use verovioxide::{Options, BreakMode, HeaderMode};
//!
//! let options = Options::builder()
//!     .scale(80)                          // 80% scale
//!     .page_width(2100)                   // A4 width in MEI units
//!     .page_height(2970)                  // A4 height
//!     .adjust_page_height(true)           // Fit content
//!     .font("Bravura")                    // Use Bravura font
//!     .breaks(BreakMode::Auto)            // Automatic page breaks
//!     .header(HeaderMode::None)           // No header
//!     .build();
//! ```
//!
//! # Thread Safety
//!
//! [`Toolkit`] implements `Send` but not `Sync`. You can move a toolkit between
//! threads, but you cannot share references across threads. For concurrent rendering,
//! create separate toolkit instances.
//!
//! # Error Handling
//!
//! All fallible operations return [`Result<T, Error>`](Result). The [`Error`] type
//! provides detailed information about what went wrong.
//!
//! ```no_run
//! use verovioxide::{Toolkit, Error};
//!
//! let toolkit = Toolkit::new().unwrap();
//!
//! match toolkit.render_to_svg(1) {
//!     Ok(svg) => println!("Success!"),
//!     Err(Error::RenderError(msg)) => eprintln!("Render failed: {}", msg),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! # Low-Level Access
//!
//! For advanced use cases, the raw FFI bindings are available in the
//! [`verovioxide-sys`](https://docs.rs/verovioxide-sys) crate.
//!
//! # Architecture
//!
//! Verovioxide is organized as a layered crate structure, with each layer providing
//! a specific level of abstraction:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        verovioxide                              │
//! │      (safe Rust API: Toolkit, Options, Error)                   │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ uses
//! ┌───────────────────────────┴─────────────────────────────────────┐
//! │                      verovioxide-sys                            │
//! │        (unsafe FFI bindings to Verovio C API)                   │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │ links to
//! ┌───────────────────────────┴─────────────────────────────────────┐
//! │                        Verovio (C++)                            │
//! │    (music notation engraving library, statically linked)        │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     verovioxide-data                            │
//! │      (embedded SMuFL fonts and Verovio resources)               │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Data Flow
//!
//! The typical data flow through verovioxide follows this pattern:
//!
//! 1. **Input**: Music notation in various formats (MusicXML, MEI, ABC, Humdrum,
//!    Plaine & Easie) is loaded via [`Toolkit::load_data`] or [`Toolkit::load_file`].
//!
//! 2. **Processing**: Verovio parses the input, builds an internal representation,
//!    and lays out the music notation according to the configured [`Options`].
//!
//! 3. **Output**: The processed music is rendered or exported via methods like:
//!    - [`Toolkit::render_to_svg`] - Render to SVG graphics
//!    - [`Toolkit::get_mei`] - Export as MEI XML
//!    - [`Toolkit::render_to_midi`] - Export as MIDI (base64-encoded)
//!    - [`Toolkit::get_humdrum`] - Export as Humdrum
//!
//! ## Resource Management
//!
//! When created with [`Toolkit::new()`], the toolkit automatically:
//!
//! 1. Extracts bundled SMuFL fonts and Verovio resources from the embedded data
//!    (compiled into the binary via `verovioxide-data`) to a temporary directory.
//!
//! 2. Initializes the underlying Verovio C++ toolkit with the path to these
//!    extracted resources.
//!
//! 3. When the [`Toolkit`] is dropped, the temporary directory and all extracted
//!    files are automatically cleaned up.
//!
//! For applications that need to avoid this extraction overhead (e.g., creating
//! many toolkit instances), use [`Toolkit::with_resource_path`] with a
//! pre-extracted directory, or [`Toolkit::without_resources`] for operations
//! that do not require font rendering.
//!
//! ## Thread Safety Model
//!
//! [`Toolkit`] implements [`Send`] but not [`Sync`]:
//!
//! - **Send**: A toolkit can be moved between threads. This is safe because
//!   each toolkit owns its Verovio instance exclusively.
//!
//! - **Not Sync**: A toolkit cannot be shared between threads via references.
//!   The underlying Verovio C++ code maintains internal mutable state that is
//!   not thread-safe to access concurrently.
//!
//! For parallel rendering scenarios:
//!
//! - Create a separate [`Toolkit`] instance for each thread.
//! - Each toolkit must load its own copy of the document.
//! - Consider using [`Toolkit::with_resource_path`] with a shared pre-extracted
//!   resource directory to avoid redundant extraction overhead.
//!
//! # Performance Considerations
//!
//! Understanding the performance characteristics of verovioxide helps you write
//! efficient applications:
//!
//! ## Toolkit Creation
//!
//! [`Toolkit::new()`] extracts bundled resources (fonts, symbols) to a temporary
//! directory. This involves disk I/O and typically takes a few hundred milliseconds.
//! For better performance:
//!
//! - **Reuse toolkit instances** when rendering multiple documents
//! - **Use [`Toolkit::with_resource_path()`]** with a pre-extracted directory
//!   if you need to create many toolkit instances
//! - **Use [`Toolkit::without_resources()`]** for operations that don't require
//!   font rendering (e.g., format conversion, metadata extraction)
//!
//! ## Document Loading and Parsing
//!
//! Parsing time scales with document complexity. Simple scores parse in milliseconds;
//! complex orchestral works may take longer. Tips:
//!
//! - **Load once, render many**: After loading a document, you can render pages
//!   and change options without reloading
//! - **Use [`redo_layout()`](Toolkit::redo_layout)** instead of reloading when
//!   changing layout-affecting options
//!
//! ## Rendering
//!
//! SVG rendering is CPU-intensive. Each page renders independently, so:
//!
//! - **Cache rendered SVGs** if you display the same page multiple times
//! - **Render on demand** rather than pre-rendering all pages for large documents
//! - **Consider lower scale values** for preview rendering
//!
//! ## Concurrent Rendering
//!
//! Each [`Toolkit`] instance has its own internal state and is `Send` but not `Sync`.
//! For concurrent rendering:
//!
//! - **Create separate toolkit instances** for each thread
//! - **Each instance needs its own loaded document copy**
//! - For rendering different pages of the same document concurrently, you would
//!   need multiple toolkits each with the document loaded
//!
//! ## Memory Usage
//!
//! - Toolkit instances hold parsed document state in memory
//! - Large documents with many pages consume more memory
//! - SVG output strings can be large; consider streaming to files for big documents
//! - Dropping a toolkit releases all associated memory and temporary files

mod error;
mod options;
mod query;
mod render;
mod toolkit;

pub use error::{Error, Result};
pub use options::{
    BreakMode, CondenseMode, FooterMode, HeaderMode, Options, OptionsBuilder, TextFont,
};
pub use query::{
    Attrs, Elements, ExpansionIds, Features, FeaturesOptionsBuilder, MidiValues, NotatedId, Page,
    QueryOutput, Time, Times,
};
pub use render::{
    ExpansionMap, Humdrum, Mei, MeiOptionsBuilder, Midi, Pae, RenderOutput, RenderSpec, Svg,
    SvgAllPages, SvgPage, SvgPages, Timemap, TimemapOptionsBuilder,
};
pub use toolkit::{LoadSource, Toolkit, ZipBase64, ZipBuffer};

// PNG exports (feature-gated)
#[cfg(feature = "png")]
pub use render::{Png, PngAllPages, PngOptions, PngPage, PngPages};

// Re-export data crate types when bundled-data feature is enabled
#[cfg(feature = "bundled-data")]
pub use verovioxide_data::{
    DataError, available_fonts, default_font, extract_resources, extract_resources_cached,
    has_bravura, has_gootville, has_leipzig, has_leland, has_petaluma, resource_dir,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_types_available() {
        // Ensure all public types are accessible
        let _ = std::any::type_name::<Error>();
        let _ = std::any::type_name::<Options>();
        let _ = std::any::type_name::<OptionsBuilder>();
        let _ = std::any::type_name::<Toolkit>();
        let _ = std::any::type_name::<BreakMode>();
        let _ = std::any::type_name::<CondenseMode>();
        let _ = std::any::type_name::<FooterMode>();
        let _ = std::any::type_name::<HeaderMode>();
        let _ = std::any::type_name::<TextFont>();
    }

    #[test]
    fn test_result_type_alias() {
        fn example_function() -> Result<String> {
            Ok("test".to_string())
        }
        assert!(example_function().is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_bundled_data_exports() {
        // Ensure bundled-data re-exports are available
        let _ = default_font();
        let _ = available_fonts();
        let _ = has_leipzig();
        let _ = has_bravura();
    }
}
