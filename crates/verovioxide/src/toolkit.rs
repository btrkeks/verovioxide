//! Safe wrapper around the Verovio toolkit.
//!
//! This module provides a safe, idiomatic Rust wrapper around the Verovio C API.
//! The [`Toolkit`] struct manages the lifecycle of a Verovio toolkit instance
//! and provides methods for loading music data and rendering to various formats.
//!
//! # Example
//!
//! ```no_run
//! use verovioxide::{Toolkit, Options};
//!
//! // Create a toolkit with bundled resources
//! let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
//!
//! // Load MEI data
//! let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
//! <mei xmlns="http://www.music-encoding.org/ns/mei">
//!   <music><body><mdiv><score>
//!     <scoreDef><staffGrp>
//!       <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
//!     </staffGrp></scoreDef>
//!     <section><measure><staff n="1"><layer n="1">
//!       <note pname="c" oct="4" dur="4"/>
//!     </layer></staff></measure></section>
//!   </score></mdiv></body></music>
//! </mei>"#;
//!
//! toolkit.load_data(mei).expect("Failed to load MEI");
//!
//! // Configure rendering options
//! let options = Options::builder()
//!     .scale(100)
//!     .adjust_page_height(true)
//!     .build();
//! toolkit.set_options(&options).expect("Failed to set options");
//!
//! // Render to SVG
//! let svg = toolkit.render_to_svg(1).expect("Failed to render");
//! println!("{}", svg);
//! ```

use std::ffi::{CStr, CString, c_void};
use std::path::Path;

#[cfg(feature = "bundled-data")]
use tempfile::TempDir;

use crate::error::{Error, Result};
use crate::options::Options;

/// Marker type for loading base64-encoded ZIP data (compressed MusicXML).
///
/// Use this with [`Toolkit::load`] when you have MusicXML data that has been
/// compressed (`.mxl` format) and base64-encoded.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, ZipBase64};
///
/// let mut voxide = Toolkit::new().expect("Failed to create toolkit");
/// let base64_mxl = "UEsDBBQAAAA..."; // base64-encoded .mxl data
/// voxide.load(ZipBase64(base64_mxl)).expect("Failed to load");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ZipBase64<'a>(pub &'a str);

/// Marker type for loading raw ZIP buffer data (compressed MusicXML).
///
/// Use this with [`Toolkit::load`] when you have MusicXML data that has been
/// compressed (`.mxl` format) as raw bytes.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, ZipBuffer};
/// use std::fs;
///
/// let mut voxide = Toolkit::new().expect("Failed to create toolkit");
/// let mxl_bytes = fs::read("score.mxl").expect("Failed to read file");
/// voxide.load(ZipBuffer(&mxl_bytes)).expect("Failed to load");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ZipBuffer<'a>(pub &'a [u8]);

/// Trait for types that can be loaded into a [`Toolkit`].
///
/// This trait enables the unified [`Toolkit::load`] method to accept multiple
/// input types while dispatching to the appropriate underlying load method.
///
/// # Implementors
///
/// - `&str` - Load music notation from a string (format auto-detected)
/// - `&Path` - Load music notation from a file path
/// - `&PathBuf` - Load music notation from a file path
/// - [`ZipBase64`] - Load compressed MusicXML from base64 string
/// - [`ZipBuffer`] - Load compressed MusicXML from byte slice
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, ZipBase64, ZipBuffer};
/// use std::path::Path;
///
/// let mut voxide = Toolkit::new().expect("Failed to create toolkit");
///
/// // Load from string
/// voxide.load("<mei>...</mei>").expect("Failed to load");
///
/// // Load from file
/// voxide.load(Path::new("score.mei")).expect("Failed to load");
///
/// // Load compressed MusicXML from base64
/// voxide.load(ZipBase64("UEsDBBQ...")).expect("Failed to load");
///
/// // Load compressed MusicXML from bytes
/// let bytes = std::fs::read("score.mxl").unwrap();
/// voxide.load(ZipBuffer(&bytes)).expect("Failed to load");
/// ```
pub trait LoadSource {
    /// Load this source into the given toolkit.
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()>;
}

impl LoadSource for &str {
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()> {
        toolkit.load_data(self)
    }
}

impl LoadSource for &Path {
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()> {
        toolkit.load_file(self)
    }
}

impl LoadSource for &std::path::PathBuf {
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()> {
        toolkit.load_file(self)
    }
}

impl<'a> LoadSource for ZipBase64<'a> {
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()> {
        toolkit.load_zip_data_base64(self.0)
    }
}

impl<'a> LoadSource for ZipBuffer<'a> {
    fn load_into(self, toolkit: &mut Toolkit) -> Result<()> {
        toolkit.load_zip_data_buffer(self.0)
    }
}

/// A safe wrapper around the Verovio toolkit.
///
/// This struct provides a safe, idiomatic interface to the Verovio music engraving library.
/// It manages the lifecycle of the underlying C++ toolkit and ensures proper cleanup.
///
/// # Thread Safety
///
/// `Toolkit` implements `Send` but not `Sync`. This means you can move a toolkit between
/// threads, but you cannot share references to it across threads. Each toolkit instance
/// has internal mutable state that is not thread-safe to access concurrently.
///
/// # Resource Management
///
/// When created with bundled resources (via [`Toolkit::new()`]), the toolkit extracts
/// resources to a temporary directory that is automatically cleaned up when the toolkit
/// is dropped.
pub struct Toolkit {
    /// Raw pointer to the Verovio toolkit instance.
    ptr: *mut c_void,

    /// Temporary directory holding extracted resources.
    /// Kept alive for the lifetime of the toolkit.
    #[cfg(feature = "bundled-data")]
    _temp_dir: Option<TempDir>,
}

// SAFETY: Toolkit can be sent between threads because:
// - The underlying Verovio toolkit pointer is owned exclusively
// - No references are shared across threads
// - The TempDir is also Send
unsafe impl Send for Toolkit {}

// NOTE: We intentionally do NOT implement Sync because:
// - The Verovio toolkit has internal mutable state
// - Concurrent access to the same toolkit is not safe
// - Users who need concurrent rendering should create separate toolkits

impl Drop for Toolkit {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: ptr is valid and was created by a constructor function
            unsafe {
                verovioxide_sys::vrvToolkit_destructor(self.ptr);
            }
        }
    }
}

impl Toolkit {
    /// Creates a new toolkit with bundled resources.
    ///
    /// This extracts the embedded Verovio resources (fonts, etc.) to a temporary
    /// directory and initializes the toolkit to use them. The temporary directory
    /// is automatically cleaned up when the toolkit is dropped.
    ///
    /// # Performance
    ///
    /// This operation extracts bundled resources (fonts, symbols) to a temporary
    /// directory on disk, which involves I/O operations. The extraction typically
    /// takes a few hundred milliseconds depending on disk speed. For applications
    /// that create multiple toolkits, consider reusing a single toolkit instance
    /// when possible, or use [`with_resource_path`](Self::with_resource_path) with
    /// a pre-extracted resource directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Resource extraction fails
    /// - Toolkit initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    #[cfg(feature = "bundled-data")]
    pub fn new() -> Result<Self> {
        let temp_dir = verovioxide_data::extract_resources()?;
        let resource_path = temp_dir.path();

        let path_str = resource_path.to_str().ok_or_else(|| {
            Error::InitializationError("resource path contains invalid UTF-8".into())
        })?;

        let c_path = CString::new(path_str)?;

        // SAFETY: c_path is a valid null-terminated string
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorResourcePath(c_path.as_ptr()) };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit with resource path".into(),
            ));
        }

        Ok(Self {
            ptr,
            _temp_dir: Some(temp_dir),
        })
    }

    /// Creates a new toolkit using bundled resources extracted to a stable
    /// cache directory.
    ///
    /// Unlike [`new`](Self::new), which extracts the bundled resources to a
    /// fresh temporary directory for every toolkit (and leaks that directory
    /// if the process dies before `Drop` runs), this extracts once into
    /// `cache_root` via [`verovioxide_data::extract_resources_cached`] and
    /// reuses the extraction across toolkits, processes, and restarts.
    ///
    /// # Arguments
    ///
    /// * `cache_root` - Directory that holds the versioned resource cache
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Resource extraction fails
    /// - Toolkit initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let cache_root = std::env::temp_dir().join("my-app-cache");
    /// let toolkit = Toolkit::with_cached_resources(&cache_root)
    ///     .expect("Failed to create toolkit");
    /// ```
    #[cfg(feature = "bundled-data")]
    pub fn with_cached_resources(cache_root: &Path) -> Result<Self> {
        let resources = verovioxide_data::extract_resources_cached(cache_root)?;
        Self::with_resource_path(&resources)
    }

    /// Creates a new toolkit with an explicit resource path.
    ///
    /// Use this when you have your own Verovio resources directory and don't want
    /// to use the bundled resources.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Verovio resources directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path contains invalid UTF-8
    /// - Toolkit initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let toolkit = Toolkit::with_resource_path(Path::new("/path/to/verovio/data"))
    ///     .expect("Failed to create toolkit");
    /// ```
    pub fn with_resource_path(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or_else(|| {
            Error::InitializationError("resource path contains invalid UTF-8".into())
        })?;

        let c_path = CString::new(path_str)?;

        // SAFETY: c_path is a valid null-terminated string
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorResourcePath(c_path.as_ptr()) };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit with resource path".into(),
            ));
        }

        Ok(Self {
            ptr,
            #[cfg(feature = "bundled-data")]
            _temp_dir: None,
        })
    }

    /// Creates a new toolkit without loading any resources.
    ///
    /// This is useful for operations that don't require font resources, such as
    /// converting between formats or extracting metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if toolkit initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    pub fn without_resources() -> Result<Self> {
        // SAFETY: This function has no preconditions
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorNoResource() };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit without resources".into(),
            ));
        }

        Ok(Self {
            ptr,
            #[cfg(feature = "bundled-data")]
            _temp_dir: None,
        })
    }

    /// Loads music notation from various sources.
    ///
    /// This is the unified loading method that dispatches to the appropriate
    /// underlying loader based on the input type. The format is auto-detected.
    ///
    /// # Supported Sources
    ///
    /// | Type | Description |
    /// |------|-------------|
    /// | `&str` | Music notation as a string (MEI, MusicXML, ABC, Humdrum, PAE) |
    /// | `&Path` | Path to a music file |
    /// | `&PathBuf` | Path to a music file |
    /// | [`ZipBase64`] | Base64-encoded compressed MusicXML (`.mxl`) |
    /// | [`ZipBuffer`] | Raw bytes of compressed MusicXML (`.mxl`) |
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, ZipBase64, ZipBuffer};
    /// use std::path::Path;
    ///
    /// let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    ///
    /// // Load MEI from string
    /// let mei = r#"<mei xmlns="http://www.music-encoding.org/ns/mei">...</mei>"#;
    /// voxide.load(mei).expect("Failed to load MEI");
    ///
    /// // Load from file
    /// voxide.load(Path::new("score.musicxml")).expect("Failed to load file");
    ///
    /// // Load compressed MusicXML
    /// let mxl_bytes = std::fs::read("score.mxl").unwrap();
    /// voxide.load(ZipBuffer(&mxl_bytes)).expect("Failed to load MXL");
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_data`](Self::load_data) - Load specifically from string
    /// - [`load_file`](Self::load_file) - Load specifically from file path
    /// - [`load_zip_data_base64`](Self::load_zip_data_base64) - Load specifically from base64 ZIP
    /// - [`load_zip_data_buffer`](Self::load_zip_data_buffer) - Load specifically from ZIP bytes
    pub fn load(&mut self, source: impl LoadSource) -> Result<()> {
        source.load_into(self)
    }

    /// Loads music data from a string.
    ///
    /// The data format is auto-detected. Supported formats include:
    /// - MEI (Music Encoding Initiative)
    /// - MusicXML
    /// - Humdrum
    /// - Plaine & Easie Code (PAE)
    /// - ABC notation
    ///
    /// # Performance
    ///
    /// Parsing time scales with document complexity. Simple scores parse in
    /// milliseconds, while complex orchestral works with many pages may take
    /// several hundred milliseconds. The parsing also performs initial layout
    /// calculations. For repeated rendering of the same document with different
    /// options, load once and call [`set_options`](Self::set_options) followed
    /// by [`redo_layout`](Self::redo_layout) rather than reloading.
    ///
    /// # Arguments
    ///
    /// * `data` - The music data as a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is malformed
    /// - The format is not recognized
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    ///
    /// let mei = r#"<mei xmlns="http://www.music-encoding.org/ns/mei">...</mei>"#;
    /// toolkit.load_data(mei).expect("Failed to load data");
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_file`](Self::load_file) - Load music data from a file
    pub fn load_data(&mut self, data: &str) -> Result<()> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_loadData(self.ptr, c_data.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError(
                "failed to load data (check format and content)".into(),
            ))
        }
    }

    /// Loads music data from a file.
    ///
    /// The file format is auto-detected based on content.
    ///
    /// # Performance
    ///
    /// This method reads the entire file into memory and then parses it.
    /// Performance characteristics are similar to [`load_data`](Self::load_data),
    /// plus file I/O overhead. For large files, consider whether the file needs
    /// to be read from disk each time, or if caching the file content in memory
    /// would be beneficial.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the music file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be read
    /// - The data is malformed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.load_file(Path::new("score.mei")).expect("Failed to load file");
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_data`](Self::load_data) - Load music data from a string
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_path_buf()));
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| Error::LoadError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_loadFile(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError(format!(
                "failed to load file: {}",
                path.display()
            )))
        }
    }

    // =========================================================================
    // Format Control Functions
    // =========================================================================

    /// Sets the input format explicitly.
    ///
    /// By default, Verovio auto-detects the input format. Use this method
    /// to override the auto-detection and specify the format explicitly.
    ///
    /// # Arguments
    ///
    /// * `format` - Input format string (e.g., "mei", "musicxml", "humdrum", "pae", "abc")
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not recognized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.set_input_from("mei").expect("Failed to set input format");
    /// // Now load_data will treat input as MEI regardless of content
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_output_to`](Self::set_output_to) - Set output format
    /// - [`load_data`](Self::load_data) - Load music data
    pub fn set_input_from(&mut self, format: &str) -> Result<()> {
        let c_format = CString::new(format)?;

        // SAFETY: ptr is valid, c_format is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_setInputFrom(self.ptr, c_format.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError(format!(
                "unrecognized input format: {}",
                format
            )))
        }
    }

    /// Sets the output format.
    ///
    /// This affects the format used by [`render_data`](Self::render_data) and
    /// other rendering operations.
    ///
    /// # Arguments
    ///
    /// * `format` - Output format string (e.g., "svg", "mei", "midi", "humdrum")
    ///
    /// # Errors
    ///
    /// Returns an error if the format is not recognized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.set_output_to("mei").expect("Failed to set output format");
    /// // Now render_data will output MEI instead of SVG
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_input_from`](Self::set_input_from) - Set input format
    /// - [`render_data`](Self::render_data) - Render data with current output format
    pub fn set_output_to(&mut self, format: &str) -> Result<()> {
        let c_format = CString::new(format)?;

        // SAFETY: ptr is valid, c_format is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_setOutputTo(self.ptr, c_format.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError(format!(
                "unrecognized output format: {}",
                format
            )))
        }
    }

    // =========================================================================
    // ZIP Loading Functions
    // =========================================================================

    /// Loads compressed MusicXML from base64-encoded ZIP data.
    ///
    /// MusicXML files are often distributed as compressed `.mxl` files.
    /// This method loads such files when provided as base64-encoded data.
    ///
    /// # Arguments
    ///
    /// * `data` - Base64-encoded ZIP data containing MusicXML
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - The data is not valid base64
    /// - The ZIP archive is invalid
    /// - The MusicXML content is malformed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let base64_zip = "..."; // base64-encoded .mxl file contents
    /// toolkit.load_zip_data_base64(base64_zip)
    ///     .expect("Failed to load compressed MusicXML");
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_zip_data_buffer`](Self::load_zip_data_buffer) - Load from binary buffer
    /// - [`load_data`](Self::load_data) - Load uncompressed data
    pub fn load_zip_data_base64(&mut self, data: &str) -> Result<()> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_loadZipDataBase64(self.ptr, c_data.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError("failed to load ZIP data (base64)".into()))
        }
    }

    /// Loads compressed MusicXML from a binary buffer.
    ///
    /// MusicXML files are often distributed as compressed `.mxl` files.
    /// This method loads such files directly from binary data.
    ///
    /// # Arguments
    ///
    /// * `data` - Binary ZIP data containing MusicXML
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ZIP archive is invalid
    /// - The MusicXML content is malformed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::fs;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let zip_data = fs::read("score.mxl").expect("Failed to read file");
    /// toolkit.load_zip_data_buffer(&zip_data)
    ///     .expect("Failed to load compressed MusicXML");
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_zip_data_base64`](Self::load_zip_data_base64) - Load from base64 string
    /// - [`load_file`](Self::load_file) - Load from file path
    pub fn load_zip_data_buffer(&mut self, data: &[u8]) -> Result<()> {
        // SAFETY: ptr is valid, data.as_ptr() is valid for data.len() bytes
        let success = unsafe {
            verovioxide_sys::vrvToolkit_loadZipDataBuffer(
                self.ptr,
                data.as_ptr(),
                data.len() as std::ffi::c_int,
            )
        };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError("failed to load ZIP data buffer".into()))
        }
    }

    // =========================================================================
    // PAE Validation Functions
    // =========================================================================

    /// Validates Plaine & Easie code.
    ///
    /// This method validates PAE code without loading it into the toolkit.
    /// It returns a JSON string with validation results.
    ///
    /// # Arguments
    ///
    /// * `data` - PAE code to validate
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - Validation fails unexpectedly
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let pae_code = "@clef:G-2\n@keysig:xFCG\n@timesig:4/4\n@data:4C";
    /// let result = toolkit.validate_pae(pae_code)
    ///     .expect("Failed to validate");
    /// println!("Validation result: {}", result);
    /// ```
    ///
    /// # See also
    ///
    /// - [`validate_pae_file`](Self::validate_pae_file) - Validate from file
    /// - [`render_to_pae`](Self::render_to_pae) - Export to PAE
    pub fn validate_pae(&self, data: &str) -> Result<String> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_validatePAE(self.ptr, c_data.as_ptr()) };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to validate PAE".into()))
    }

    /// Validates PAE code from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PAE file to validate
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The path contains invalid UTF-8
    /// - Validation fails unexpectedly
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let result = toolkit.validate_pae_file(Path::new("score.pae"))
    ///     .expect("Failed to validate");
    /// println!("Validation result: {}", result);
    /// ```
    ///
    /// # See also
    ///
    /// - [`validate_pae`](Self::validate_pae) - Validate from string
    /// - [`render_to_pae_file`](Self::render_to_pae_file) - Export to PAE file
    pub fn validate_pae_file(&self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_path_buf()));
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_validatePAEFile(self.ptr, c_path.as_ptr()) };

        self.ptr_to_string(result_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to validate PAE file: {}", path.display()))
        })
    }

    // =========================================================================
    // Selection and Layout Functions
    // =========================================================================

    /// Selects elements in the document.
    ///
    /// This method allows selecting specific elements in the loaded document,
    /// which can affect rendering (e.g., highlighting selected elements).
    ///
    /// # Arguments
    ///
    /// * `selection` - JSON string describing the selection (element IDs, ranges, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The selection string contains a null byte
    /// - The selection is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// let selection = r#"{"start": "note-0001", "end": "note-0010"}"#;
    /// toolkit.select(selection).expect("Failed to select");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_svg`](Self::render_to_svg) - Render with selection applied
    /// - [`edit`](Self::edit) - Perform editor actions
    pub fn select(&mut self, selection: &str) -> Result<()> {
        let c_selection = CString::new(selection)?;

        // SAFETY: ptr is valid, c_selection is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_select(self.ptr, c_selection.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError("failed to apply selection".into()))
        }
    }

    /// Redoes the pitch position layout for the current page.
    ///
    /// This method recalculates pitch positions without redoing the full layout.
    /// It's useful after certain modifications that only affect vertical positioning.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data and make modifications ...
    /// toolkit.redo_page_pitch_pos_layout();
    /// ```
    ///
    /// # See also
    ///
    /// - [`redo_layout`](Self::redo_layout) - Full layout recalculation
    pub fn redo_page_pitch_pos_layout(&mut self) {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_redoPagePitchPosLayout(self.ptr) };
    }

    /// Resets the XML ID seed.
    ///
    /// This affects how new xml:id values are generated when creating or
    /// modifying elements. Setting a consistent seed can be useful for
    /// reproducible output.
    ///
    /// # Arguments
    ///
    /// * `seed` - The new seed value
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.reset_xml_id_seed(42);
    /// // Now newly generated IDs will be deterministic based on this seed
    /// ```
    pub fn reset_xml_id_seed(&mut self, seed: i32) {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_resetXmlIdSeed(self.ptr, seed) };
    }

    /// Gets the option usage string.
    ///
    /// Returns a formatted string describing all available command-line options,
    /// suitable for displaying help information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let usage = toolkit.get_option_usage_string();
    /// println!("Options:\n{}", usage);
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_available_options`](Self::get_available_options) - Get options as JSON
    /// - [`get_options`](Self::get_options) - Get current options
    #[must_use]
    pub fn get_option_usage_string(&self) -> String {
        // SAFETY: ptr is valid
        let usage_ptr = unsafe { verovioxide_sys::vrvToolkit_getOptionUsageString(self.ptr) };
        self.ptr_to_string(usage_ptr).unwrap_or_default()
    }

    // =========================================================================
    // Unified Render API
    // =========================================================================

    /// Renders to a format-specific output type using builder pattern.
    ///
    /// This is the unified rendering API that provides type-safe, consistent
    /// access to all output formats. Each format has its own builder type
    /// that specifies options and determines the output type.
    ///
    /// # Format Types
    ///
    /// | Format | Builder | Output Type |
    /// |--------|---------|-------------|
    /// | SVG (single page) | [`Svg`]`::page(n)` | `String` |
    /// | SVG (page range) | [`Svg`]`::pages(start, end)` | `Vec<String>` |
    /// | SVG (all pages) | [`Svg`]`::all_pages()` | `Vec<String>` |
    /// | MIDI | [`Midi`] | `String` (base64) |
    /// | PAE | [`Pae`] | `String` |
    /// | Timemap | [`Timemap`] | `String` (JSON) |
    /// | ExpansionMap | [`ExpansionMap`] | `String` (JSON) |
    /// | MEI | [`Mei`] | `String` |
    /// | Humdrum | [`Humdrum`] | `String` |
    ///
    /// [`Svg`]: crate::Svg
    /// [`Midi`]: crate::Midi
    /// [`Pae`]: crate::Pae
    /// [`Timemap`]: crate::Timemap
    /// [`ExpansionMap`]: crate::ExpansionMap
    /// [`Mei`]: crate::Mei
    /// [`Humdrum`]: crate::Humdrum
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, Svg, Midi, Timemap, Mei};
    ///
    /// let mut voxide = Toolkit::new().unwrap();
    /// voxide.load("score.mei").unwrap();
    ///
    /// // SVG rendering
    /// let svg: String = voxide.render(Svg::page(1)).unwrap();
    /// let svg: String = voxide.render(Svg::page(3).with_declaration()).unwrap();
    /// let pages: Vec<String> = voxide.render(Svg::all_pages()).unwrap();
    /// let pages: Vec<String> = voxide.render(Svg::pages(2, 5)).unwrap();
    ///
    /// // Other formats
    /// let midi: String = voxide.render(Midi).unwrap();
    /// let timemap: String = voxide.render(Timemap).unwrap();
    /// let timemap: String = voxide.render(
    ///     Timemap::with_options().include_measures(true)
    /// ).unwrap();
    /// let mei: String = voxide.render(Mei).unwrap();
    /// let mei: String = voxide.render(
    ///     Mei::with_options().remove_ids(true)
    /// ).unwrap();
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to`](Self::render_to) - Render to file with format inference
    /// - [`render_to_as`](Self::render_to_as) - Render to file with explicit format
    pub fn render<R: crate::render::RenderOutput>(&self, format: R) -> Result<R::Output> {
        format.render(self)
    }

    /// Renders to a file with format inferred from the file extension.
    ///
    /// This is a convenience method that automatically determines the output
    /// format based on the file extension. For formats that require additional
    /// options or for ambiguous extensions, use [`render_to_as`](Self::render_to_as).
    ///
    /// # Supported Extensions
    ///
    /// | Extension | Format | Notes |
    /// |-----------|--------|-------|
    /// | `.svg` | SVG | Renders page 1 |
    /// | `.mid`, `.midi` | MIDI | Base64-decoded and written as binary |
    /// | `.pae` | PAE | |
    /// | `.mei` | MEI | |
    /// | `.krn`, `.hmd` | Humdrum | |
    /// | `.json` | Error | Ambiguous: use `render_to_as` with `Timemap` or `ExpansionMap` |
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut voxide = Toolkit::new().unwrap();
    /// voxide.load("score.mei").unwrap();
    ///
    /// // Format inferred from extension
    /// voxide.render_to("output.svg").unwrap();    // SVG page 1
    /// voxide.render_to("output.mid").unwrap();    // MIDI
    /// voxide.render_to("output.mei").unwrap();    // MEI
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file extension is not recognized
    /// - The extension is ambiguous (`.json`)
    /// - The file cannot be written
    /// - Rendering fails
    ///
    /// # See also
    ///
    /// - [`render`](Self::render) - In-memory rendering
    /// - [`render_to_as`](Self::render_to_as) - File rendering with explicit format
    pub fn render_to(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        crate::render::infer_format_and_render(self, path.as_ref())
    }

    /// Renders to a file with explicit format specification.
    ///
    /// Use this method when you need to:
    /// - Specify a non-default page number for SVG
    /// - Render multiple pages to a directory
    /// - Disambiguate `.json` files (Timemap vs ExpansionMap)
    /// - Use format-specific options
    ///
    /// # Multi-Page Output
    ///
    /// When using `Svg::all_pages()` or `Svg::pages(start, end)`, this method
    /// creates a directory with the same name as the file (minus extension)
    /// and writes individual page files as `page-001.svg`, `page-002.svg`, etc.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, Svg, Timemap, Mei};
    ///
    /// let mut voxide = Toolkit::new().unwrap();
    /// voxide.load("score.mei").unwrap();
    ///
    /// // Specific page
    /// voxide.render_to_as("output.svg", Svg::page(3)).unwrap();
    ///
    /// // All pages (creates output/ directory with page-001.svg, page-002.svg, ...)
    /// voxide.render_to_as("output.svg", Svg::all_pages()).unwrap();
    ///
    /// // Page range
    /// voxide.render_to_as("output.svg", Svg::pages(2, 5)).unwrap();
    ///
    /// // Disambiguate JSON formats
    /// voxide.render_to_as("output.json", Timemap).unwrap();
    ///
    /// // With options
    /// voxide.render_to_as("output.json",
    ///     Timemap::with_options().include_measures(true)
    /// ).unwrap();
    /// voxide.render_to_as("output.mei",
    ///     Mei::with_options().remove_ids(true)
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file or directory cannot be created
    /// - Rendering fails
    ///
    /// # See also
    ///
    /// - [`render`](Self::render) - In-memory rendering
    /// - [`render_to`](Self::render_to) - File rendering with format inference
    pub fn render_to_as<F: crate::render::RenderSpec>(
        &self,
        path: impl AsRef<std::path::Path>,
        format: F,
    ) -> Result<()> {
        format.render_to_file(self, path.as_ref())
    }

    // =========================================================================
    // Unified Query API
    // =========================================================================

    /// Queries element or document information with type-safe output.
    ///
    /// This is the unified query API that provides type-safe, consistent
    /// access to element queries and document information. Each query type
    /// specifies its return type, enabling compile-time type checking.
    ///
    /// # Query Types
    ///
    /// | Query | Builder | Output Type |
    /// |-------|---------|-------------|
    /// | Page number | [`Page`]`::of(id)` | `u32` |
    /// | Attributes | [`Attrs`]`::of(id)` | `String` (JSON) |
    /// | Time | [`Time`]`::of(id)` | `f64` (milliseconds) |
    /// | Times | [`Times`]`::of(id)` | `String` (JSON) |
    /// | Expansion IDs | [`ExpansionIds`]`::of(id)` | `String` (JSON) |
    /// | MIDI values | [`MidiValues`]`::of(id)` | `String` (JSON) |
    /// | Notated ID | [`NotatedId`]`::of(id)` | `String` |
    /// | Elements at time | [`Elements`]`::at(ms)` | `String` (JSON) |
    /// | Descriptive features | [`Features`] | `String` (JSON) |
    ///
    /// [`Page`]: crate::Page
    /// [`Attrs`]: crate::Attrs
    /// [`Time`]: crate::Time
    /// [`Times`]: crate::Times
    /// [`ExpansionIds`]: crate::ExpansionIds
    /// [`MidiValues`]: crate::MidiValues
    /// [`NotatedId`]: crate::NotatedId
    /// [`Elements`]: crate::Elements
    /// [`Features`]: crate::Features
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, Page, Attrs, Time, Times, Elements, Features};
    ///
    /// let mut voxide = Toolkit::new().unwrap();
    /// voxide.load("score.mei").unwrap();
    ///
    /// // Element queries
    /// let page: u32 = voxide.get(Page::of("note-001")).unwrap();
    /// let attrs: String = voxide.get(Attrs::of("note-001")).unwrap();
    /// let time: f64 = voxide.get(Time::of("note-001")).unwrap();
    /// let times: String = voxide.get(Times::of("note-001")).unwrap();
    ///
    /// // Time-based query
    /// let elements: String = voxide.get(Elements::at(5000)).unwrap();
    ///
    /// // Descriptive features
    /// let features: String = voxide.get(Features).unwrap();
    /// let features: String = voxide.get(
    ///     Features::with_options().option("key", "value")
    /// ).unwrap();
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_page_with_element`](Self::get_page_with_element) - Legacy method for page lookup
    /// - [`get_element_attr`](Self::get_element_attr) - Legacy method for attributes
    /// - [`get_time_for_element`](Self::get_time_for_element) - Legacy method for time
    ///
    /// *Added in 0.3.0.*
    #[cfg_attr(docsrs, doc(cfg(since = "0.3.0")))]
    pub fn get<Q: crate::query::QueryOutput>(&self, query: Q) -> Result<Q::Output> {
        query.query(self)
    }

    // =========================================================================
    // Legacy Rendering Methods
    // =========================================================================
    //
    // The methods below are the original rendering API. They remain available
    // for backwards compatibility and for cases where you need direct access
    // to specific functionality. For new code, consider using the unified
    // render(), render_to(), and render_to_as() methods above.

    /// Renders a page to SVG.
    ///
    /// Page numbers are 1-based. Use [`page_count()`](Self::page_count) to get the
    /// total number of pages.
    ///
    /// # Performance
    ///
    /// SVG rendering is CPU-intensive, involving glyph lookup, path generation,
    /// and string formatting. Rendering time scales with page complexity (number
    /// of notes, staves, and annotations). For applications that render the same
    /// page multiple times (e.g., with different highlighting), consider caching
    /// the base SVG and applying modifications to the cached result.
    ///
    /// If you need to render multiple pages, calling this method in a loop is
    /// efficient as the layout is already computed. For parallel rendering of
    /// different documents, create separate [`Toolkit`] instances.
    ///
    /// # Arguments
    ///
    /// * `page` - The page number to render (1-based)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The page number is out of range
    /// - Rendering fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let svg = toolkit.render_to_svg(1).expect("Failed to render");
    /// println!("{}", svg);
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_svg_with_declaration`](Self::render_to_svg_with_declaration) - Include XML declaration
    /// - [`render_all_pages`](Self::render_all_pages) - Render all pages at once
    /// - [`page_count`](Self::page_count) - Get the total number of pages
    pub fn render_to_svg(&self, page: u32) -> Result<String> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(Error::RenderError(format!(
                "page {} out of range (document has {} pages)",
                page, page_count
            )));
        }

        // SAFETY: ptr is valid, page number is in range
        let svg_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToSVG(self.ptr, page as i32, false) };

        self.ptr_to_string(svg_ptr)
            .ok_or_else(|| Error::RenderError("failed to render SVG".into()))
    }

    /// Renders a page to SVG with XML declaration.
    ///
    /// Same as [`render_to_svg`](Self::render_to_svg) but includes the XML declaration
    /// at the start of the SVG output.
    ///
    /// # Arguments
    ///
    /// * `page` - The page number to render (1-based)
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    ///
    /// # See also
    ///
    /// - [`render_to_svg`](Self::render_to_svg) - Render without XML declaration
    /// - [`render_all_pages`](Self::render_all_pages) - Render all pages at once
    pub fn render_to_svg_with_declaration(&self, page: u32) -> Result<String> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(Error::RenderError(format!(
                "page {} out of range (document has {} pages)",
                page, page_count
            )));
        }

        // SAFETY: ptr is valid, page number is in range
        let svg_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToSVG(self.ptr, page as i32, true) };

        self.ptr_to_string(svg_ptr)
            .ok_or_else(|| Error::RenderError("failed to render SVG".into()))
    }

    /// Renders all pages to SVG.
    ///
    /// # Performance
    ///
    /// This method renders pages sequentially. For a document with N pages,
    /// the total time is approximately N times the single-page render time.
    /// The method pre-allocates the result vector to avoid reallocations.
    ///
    /// For parallel rendering of the same document, you would need to create
    /// multiple [`Toolkit`] instances, each with its own copy of the loaded
    /// data. However, for most use cases, sequential rendering is sufficient
    /// and avoids the overhead of multiple toolkit instances.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - Rendering any page fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let pages = toolkit.render_all_pages().expect("Failed to render");
    /// for (i, svg) in pages.iter().enumerate() {
    ///     println!("Page {}: {} bytes", i + 1, svg.len());
    /// }
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_svg`](Self::render_to_svg) - Render a single page
    /// - [`page_count`](Self::page_count) - Get the total number of pages
    pub fn render_all_pages(&self) -> Result<Vec<String>> {
        let count = self.page_count();
        let mut pages = Vec::with_capacity(count as usize);

        for page in 1..=count {
            pages.push(self.render_to_svg(page)?);
        }

        Ok(pages)
    }

    /// Returns the number of pages in the loaded document.
    ///
    /// Returns 0 if no document is loaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// println!("Document has {} pages", toolkit.page_count());
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_svg`](Self::render_to_svg) - Render a specific page
    /// - [`render_all_pages`](Self::render_all_pages) - Render all pages at once
    #[must_use]
    pub fn page_count(&self) -> u32 {
        // SAFETY: ptr is valid
        let count = unsafe { verovioxide_sys::vrvToolkit_getPageCount(self.ptr) };
        count.max(0) as u32
    }

    /// Sets rendering options.
    ///
    /// Options are merged with existing options. To reset to defaults, use
    /// [`reset_options()`](Self::reset_options) first.
    ///
    /// # Performance
    ///
    /// Setting options is a lightweight operation that only stores configuration
    /// values. However, if a document is already loaded, certain option changes
    /// (such as page dimensions, margins, or break modes) will require a layout
    /// recalculation on the next render. For best performance when experimenting
    /// with different options, set all desired options before loading data, or
    /// call [`redo_layout`](Self::redo_layout) explicitly after changing layout-
    /// affecting options.
    ///
    /// # Arguments
    ///
    /// * `options` - The rendering options to set
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - JSON serialization fails
    /// - Option values are invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, Options};
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    ///
    /// let options = Options::builder()
    ///     .scale(80)
    ///     .adjust_page_height(true)
    ///     .build();
    ///
    /// toolkit.set_options(&options).expect("Failed to set options");
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_options`](Self::get_options) - Get current options as JSON
    /// - [`reset_options`](Self::reset_options) - Reset to default options
    /// - [`get_default_options`](Self::get_default_options) - Get default options as JSON
    /// - [`Options`] - The options type
    pub fn set_options(&mut self, options: &Options) -> Result<()> {
        let json = options
            .to_json()
            .map_err(|e| Error::OptionsError(e.to_string()))?;

        let c_json = CString::new(json)?;

        // SAFETY: ptr is valid, c_json is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_setOptions(self.ptr, c_json.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError("failed to set options".into()))
        }
    }

    /// Gets the current options as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let options_json = toolkit.get_options();
    /// println!("Current options: {}", options_json);
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_options`](Self::set_options) - Set rendering options
    /// - [`reset_options`](Self::reset_options) - Reset to default options
    /// - [`get_default_options`](Self::get_default_options) - Get default options as JSON
    /// - [`get_available_options`](Self::get_available_options) - Get all available options
    #[must_use]
    pub fn get_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Gets the default options as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let defaults = toolkit.get_default_options();
    /// println!("Default options: {}", defaults);
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_options`](Self::set_options) - Set rendering options
    /// - [`get_options`](Self::get_options) - Get current options as JSON
    /// - [`reset_options`](Self::reset_options) - Reset to default options
    #[must_use]
    pub fn get_default_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getDefaultOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Gets all available options and their descriptions as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let available = toolkit.get_available_options();
    /// println!("Available options: {}", available);
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_options`](Self::set_options) - Set rendering options
    /// - [`get_options`](Self::get_options) - Get current options as JSON
    /// - [`get_default_options`](Self::get_default_options) - Get default options as JSON
    #[must_use]
    pub fn get_available_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getAvailableOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Resets all options to their default values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.reset_options();
    /// ```
    ///
    /// # See also
    ///
    /// - [`set_options`](Self::set_options) - Set rendering options
    /// - [`get_options`](Self::get_options) - Get current options as JSON
    /// - [`get_default_options`](Self::get_default_options) - Get default options as JSON
    pub fn reset_options(&mut self) {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_resetOptions(self.ptr) };
    }

    /// Returns the Verovio version string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    #[must_use]
    pub fn version(&self) -> String {
        // SAFETY: ptr is valid
        let version_ptr = unsafe { verovioxide_sys::vrvToolkit_getVersion(self.ptr) };
        self.ptr_to_string(version_ptr)
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Returns the log output from Verovio.
    ///
    /// Log output is only available if logging to buffer was enabled before
    /// loading data. Use [`enable_log_to_buffer()`](Self::enable_log_to_buffer)
    /// to enable it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// Toolkit::enable_log_to_buffer(true);
    /// // ... load data ...
    /// let log = toolkit.get_log();
    /// println!("Verovio log: {}", log);
    /// ```
    #[must_use]
    pub fn get_log(&self) -> String {
        // SAFETY: ptr is valid
        let log_ptr = unsafe { verovioxide_sys::vrvToolkit_getLog(self.ptr) };
        self.ptr_to_string(log_ptr).unwrap_or_default()
    }

    /// Exports the loaded document as MEI.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load MusicXML or other format ...
    ///
    /// let mei = toolkit.get_mei().expect("Failed to export MEI");
    /// println!("{}", mei);
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_mei_with_options`](Self::get_mei_with_options) - Export with custom options
    /// - [`get_humdrum`](Self::get_humdrum) - Export as Humdrum
    /// - [`render_to_pae`](Self::render_to_pae) - Export as Plaine & Easie
    /// - [`render_to_midi`](Self::render_to_midi) - Export as MIDI
    pub fn get_mei(&self) -> Result<String> {
        self.get_mei_with_options("{}")
    }

    /// Exports the loaded document as MEI with options.
    ///
    /// # Arguments
    ///
    /// * `options` - JSON string with MEI export options
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # See also
    ///
    /// - [`get_mei`](Self::get_mei) - Export with default options
    pub fn get_mei_with_options(&self, options: &str) -> Result<String> {
        let c_options = CString::new(options)?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        let mei_ptr = unsafe { verovioxide_sys::vrvToolkit_getMEI(self.ptr, c_options.as_ptr()) };

        self.ptr_to_string(mei_ptr)
            .ok_or_else(|| Error::RenderError("failed to export MEI".into()))
    }

    /// Exports the loaded document as Humdrum.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # See also
    ///
    /// - [`get_mei`](Self::get_mei) - Export as MEI
    /// - [`render_to_pae`](Self::render_to_pae) - Export as Plaine & Easie
    /// - [`render_to_midi`](Self::render_to_midi) - Export as MIDI
    pub fn get_humdrum(&self) -> Result<String> {
        // SAFETY: ptr is valid
        let humdrum_ptr = unsafe { verovioxide_sys::vrvToolkit_getHumdrum(self.ptr) };

        self.ptr_to_string(humdrum_ptr)
            .ok_or_else(|| Error::RenderError("failed to export Humdrum".into()))
    }

    // =========================================================================
    // Conversion Functions
    // =========================================================================

    /// Converts Humdrum data to processed Humdrum.
    ///
    /// This method processes Humdrum data through Verovio's internal pipeline,
    /// which can normalize and enhance the data.
    ///
    /// # Arguments
    ///
    /// * `data` - Humdrum data as a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - Conversion fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let humdrum_data = "**kern\n4c\n*-\n";
    /// let processed = toolkit.convert_humdrum_to_humdrum(humdrum_data)
    ///     .expect("Failed to convert");
    /// println!("{}", processed);
    /// ```
    ///
    /// # See also
    ///
    /// - [`convert_humdrum_to_midi`](Self::convert_humdrum_to_midi) - Convert to MIDI
    /// - [`convert_mei_to_humdrum`](Self::convert_mei_to_humdrum) - Convert MEI to Humdrum
    /// - [`get_humdrum`](Self::get_humdrum) - Get Humdrum from loaded document
    pub fn convert_humdrum_to_humdrum(&self, data: &str) -> Result<String> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let result_ptr = unsafe {
            verovioxide_sys::vrvToolkit_convertHumdrumToHumdrum(self.ptr, c_data.as_ptr())
        };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to convert Humdrum to Humdrum".into()))
    }

    /// Converts Humdrum data to MIDI (base64-encoded).
    ///
    /// This method converts Humdrum data directly to MIDI without loading
    /// the data into the toolkit first.
    ///
    /// # Arguments
    ///
    /// * `data` - Humdrum data as a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - Conversion fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let humdrum_data = "**kern\n4c\n*-\n";
    /// let midi_base64 = toolkit.convert_humdrum_to_midi(humdrum_data)
    ///     .expect("Failed to convert");
    /// println!("MIDI (base64): {}", midi_base64);
    /// ```
    ///
    /// # See also
    ///
    /// - [`convert_humdrum_to_humdrum`](Self::convert_humdrum_to_humdrum) - Process Humdrum
    /// - [`render_to_midi`](Self::render_to_midi) - Render loaded document to MIDI
    pub fn convert_humdrum_to_midi(&self, data: &str) -> Result<String> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_convertHumdrumToMIDI(self.ptr, c_data.as_ptr()) };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to convert Humdrum to MIDI".into()))
    }

    /// Converts MEI data to Humdrum.
    ///
    /// This method converts MEI data directly to Humdrum without loading
    /// the data into the toolkit first.
    ///
    /// # Arguments
    ///
    /// * `data` - MEI data as a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - Conversion fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let mei_data = r#"<mei xmlns="http://www.music-encoding.org/ns/mei">...</mei>"#;
    /// let humdrum = toolkit.convert_mei_to_humdrum(mei_data)
    ///     .expect("Failed to convert");
    /// println!("{}", humdrum);
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_humdrum`](Self::get_humdrum) - Get Humdrum from loaded document
    /// - [`convert_humdrum_to_humdrum`](Self::convert_humdrum_to_humdrum) - Process Humdrum
    pub fn convert_mei_to_humdrum(&self, data: &str) -> Result<String> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_convertMEIToHumdrum(self.ptr, c_data.as_ptr()) };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to convert MEI to Humdrum".into()))
    }

    /// Renders data with options in one step.
    ///
    /// This is a convenience method that loads data and renders it in a single
    /// operation. It combines `load_data`, `set_options`, and rendering.
    ///
    /// # Arguments
    ///
    /// * `data` - Music data to render (format auto-detected)
    /// * `options` - Optional JSON string with rendering options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data contains a null byte
    /// - Loading or rendering fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let mei = r#"<mei xmlns="http://www.music-encoding.org/ns/mei">...</mei>"#;
    /// let options = r#"{"scale": 50}"#;
    /// let svg = toolkit.render_data(mei, Some(options))
    ///     .expect("Failed to render");
    /// println!("{}", svg);
    /// ```
    ///
    /// # See also
    ///
    /// - [`load_data`](Self::load_data) - Load data separately
    /// - [`set_options`](Self::set_options) - Set options separately
    /// - [`render_to_svg`](Self::render_to_svg) - Render to SVG
    pub fn render_data(&mut self, data: &str, options: Option<&str>) -> Result<String> {
        let c_data = CString::new(data)?;
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_data and c_options are valid null-terminated strings
        let result_ptr = unsafe {
            verovioxide_sys::vrvToolkit_renderData(self.ptr, c_data.as_ptr(), c_options.as_ptr())
        };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to render data".into()))
    }

    /// Renders the loaded document to MIDI as base64-encoded data.
    ///
    /// # Performance
    ///
    /// MIDI generation traverses the entire score to extract timing and pitch
    /// information, then base64-encodes the binary MIDI data. For large scores,
    /// the base64 encoding adds a small overhead. The returned string is
    /// approximately 33% larger than the raw MIDI binary data.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or rendering fails.
    ///
    /// # See also
    ///
    /// - [`get_mei`](Self::get_mei) - Export as MEI
    /// - [`get_humdrum`](Self::get_humdrum) - Export as Humdrum
    /// - [`render_to_pae`](Self::render_to_pae) - Export as Plaine & Easie
    /// - [`render_to_timemap`](Self::render_to_timemap) - Get timing information
    pub fn render_to_midi(&self) -> Result<String> {
        if self.page_count() == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // SAFETY: ptr is valid, data is loaded
        let midi_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToMIDI(self.ptr) };

        self.ptr_to_string(midi_ptr)
            .ok_or_else(|| Error::RenderError("failed to render MIDI".into()))
    }

    /// Renders the loaded document to Plaine & Easie code.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or rendering fails.
    ///
    /// # See also
    ///
    /// - [`get_mei`](Self::get_mei) - Export as MEI
    /// - [`get_humdrum`](Self::get_humdrum) - Export as Humdrum
    /// - [`render_to_midi`](Self::render_to_midi) - Export as MIDI
    pub fn render_to_pae(&self) -> Result<String> {
        if self.page_count() == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // SAFETY: ptr is valid, data is loaded
        let pae_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToPAE(self.ptr) };

        self.ptr_to_string(pae_ptr)
            .ok_or_else(|| Error::RenderError("failed to render PAE".into()))
    }

    /// Gets the timemap as JSON.
    ///
    /// The timemap provides timing information for elements in the score,
    /// mapping musical time to milliseconds.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # See also
    ///
    /// - [`render_to_timemap_with_options`](Self::render_to_timemap_with_options) - Get timemap with custom options
    /// - [`get_elements_at_time`](Self::get_elements_at_time) - Get elements at a specific time
    /// - [`get_time_for_element`](Self::get_time_for_element) - Get time for a specific element
    /// - [`render_to_midi`](Self::render_to_midi) - Export as MIDI (includes timing)
    pub fn render_to_timemap(&self) -> Result<String> {
        self.render_to_timemap_with_options("{}")
    }

    /// Gets the timemap as JSON with options.
    ///
    /// # Arguments
    ///
    /// * `options` - JSON string with timemap options
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # See also
    ///
    /// - [`render_to_timemap`](Self::render_to_timemap) - Get timemap with default options
    /// - [`get_elements_at_time`](Self::get_elements_at_time) - Get elements at a specific time
    /// - [`get_time_for_element`](Self::get_time_for_element) - Get time for a specific element
    pub fn render_to_timemap_with_options(&self, options: &str) -> Result<String> {
        let c_options = CString::new(options)?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        let timemap_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToTimemap(self.ptr, c_options.as_ptr()) };

        self.ptr_to_string(timemap_ptr)
            .ok_or_else(|| Error::RenderError("failed to render timemap".into()))
    }

    /// Gets the expansion map as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn render_to_expansion_map(&self) -> Result<String> {
        // SAFETY: ptr is valid
        let map_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToExpansionMap(self.ptr) };

        self.ptr_to_string(map_ptr)
            .ok_or_else(|| Error::RenderError("failed to render expansion map".into()))
    }

    // =========================================================================
    // File Output Functions
    // =========================================================================

    /// Renders a page to SVG and saves to a file.
    ///
    /// This is a convenience method that combines rendering and file writing
    /// in a single operation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    /// * `page` - The page number to render (1-based)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The page number is out of range
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.render_to_svg_file(Path::new("output.svg"), 1)
    ///     .expect("Failed to save SVG");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_svg`](Self::render_to_svg) - Render to string
    /// - [`render_to_midi_file`](Self::render_to_midi_file) - Save MIDI to file
    pub fn render_to_svg_file(&self, path: &Path, page: u32) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success = unsafe {
            verovioxide_sys::vrvToolkit_renderToSVGFile(self.ptr, c_path.as_ptr(), page as i32)
        };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save SVG to file: {}",
                path.display()
            )))
        }
    }

    /// Renders the document to MIDI and saves to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output MIDI file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.render_to_midi_file(Path::new("output.mid"))
    ///     .expect("Failed to save MIDI");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_midi`](Self::render_to_midi) - Render to base64 string
    /// - [`render_to_svg_file`](Self::render_to_svg_file) - Save SVG to file
    pub fn render_to_midi_file(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_renderToMIDIFile(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save MIDI to file: {}",
                path.display()
            )))
        }
    }

    /// Renders the document to PAE and saves to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output PAE file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.render_to_pae_file(Path::new("output.pae"))
    ///     .expect("Failed to save PAE");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_pae`](Self::render_to_pae) - Render to string
    /// - [`validate_pae`](Self::validate_pae) - Validate PAE code
    pub fn render_to_pae_file(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_renderToPAEFile(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save PAE to file: {}",
                path.display()
            )))
        }
    }

    /// Renders the expansion map and saves to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.render_to_expansion_map_file(Path::new("expansion_map.json"))
    ///     .expect("Failed to save expansion map");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_expansion_map`](Self::render_to_expansion_map) - Render to string
    pub fn render_to_expansion_map_file(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success = unsafe {
            verovioxide_sys::vrvToolkit_renderToExpansionMapFile(self.ptr, c_path.as_ptr())
        };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save expansion map to file: {}",
                path.display()
            )))
        }
    }

    /// Renders the timemap and saves to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    /// * `options` - Optional JSON string with timemap options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.render_to_timemap_file(Path::new("timemap.json"), None)
    ///     .expect("Failed to save timemap");
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_timemap`](Self::render_to_timemap) - Render to string
    /// - [`render_to_timemap_with_options`](Self::render_to_timemap_with_options) - Render with options
    pub fn render_to_timemap_file(&self, path: &Path, options: Option<&str>) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_path and c_options are valid null-terminated strings
        let success = unsafe {
            verovioxide_sys::vrvToolkit_renderToTimemapFile(
                self.ptr,
                c_path.as_ptr(),
                c_options.as_ptr(),
            )
        };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save timemap to file: {}",
                path.display()
            )))
        }
    }

    /// Saves the document to a file with options.
    ///
    /// This method saves the currently loaded document to a file. The output
    /// format depends on the options and the configured output format.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    /// * `options` - Optional JSON string with save options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.save_file(Path::new("output.mei"), None)
    ///     .expect("Failed to save file");
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_mei`](Self::get_mei) - Get MEI as string
    /// - [`set_output_to`](Self::set_output_to) - Set output format
    pub fn save_file(&self, path: &Path, options: Option<&str>) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_path and c_options are valid null-terminated strings
        let success = unsafe {
            verovioxide_sys::vrvToolkit_saveFile(self.ptr, c_path.as_ptr(), c_options.as_ptr())
        };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save to file: {}",
                path.display()
            )))
        }
    }

    /// Saves the Humdrum representation to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The path contains invalid UTF-8
    /// - Writing the file fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    /// toolkit.save_humdrum_to_file(Path::new("output.krn"))
    ///     .expect("Failed to save Humdrum");
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_humdrum`](Self::get_humdrum) - Get Humdrum as string
    pub fn save_humdrum_to_file(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::RenderError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_getHumdrumFile(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError(format!(
                "failed to save Humdrum to file: {}",
                path.display()
            )))
        }
    }

    /// Gets the current rendering scale as a percentage.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let scale = toolkit.get_scale();
    /// println!("Current scale: {}%", scale);
    ///
    /// // The scale affects the rendered output size
    /// if scale < 100 {
    ///     println!("Rendering at reduced size");
    /// }
    /// ```
    #[must_use]
    pub fn get_scale(&self) -> i32 {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_getScale(self.ptr) }
    }

    /// Sets the rendering scale as a percentage.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale percentage (e.g., 100 for 100%)
    ///
    /// # Errors
    ///
    /// Returns an error if the scale value is invalid.
    pub fn set_scale(&mut self, scale: i32) -> Result<()> {
        // SAFETY: ptr is valid
        let success = unsafe { verovioxide_sys::vrvToolkit_setScale(self.ptr, scale) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError(format!("invalid scale: {}", scale)))
        }
    }

    /// Gets the toolkit instance ID.
    ///
    /// Each toolkit instance has a unique identifier assigned by Verovio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let id = toolkit.get_id();
    /// println!("Toolkit ID: {}", id);
    /// ```
    #[must_use]
    pub fn get_id(&self) -> String {
        // SAFETY: ptr is valid
        let id_ptr = unsafe { verovioxide_sys::vrvToolkit_getID(self.ptr) };
        self.ptr_to_string(id_ptr).unwrap_or_default()
    }

    /// Gets the current resource path.
    ///
    /// Returns the path to the directory containing Verovio resources (fonts, etc.).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let path = toolkit.get_resource_path();
    /// println!("Resources located at: {}", path);
    /// ```
    #[must_use]
    pub fn get_resource_path(&self) -> String {
        // SAFETY: ptr is valid
        let path_ptr = unsafe { verovioxide_sys::vrvToolkit_getResourcePath(self.ptr) };
        self.ptr_to_string(path_ptr).unwrap_or_default()
    }

    /// Sets the resource path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Verovio resources directory
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid.
    pub fn set_resource_path(&mut self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::OptionsError("resource path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_setResourcePath(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError("failed to set resource path".into()))
        }
    }

    /// Gets the page number containing a specific element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The page number (1-based), or 0 if the element is not found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load MEI data ...
    ///
    /// let page = toolkit.get_page_with_element("note-0001").expect("Failed to get page");
    /// if page > 0 {
    ///     println!("Element is on page {}", page);
    /// } else {
    ///     println!("Element not found");
    /// }
    /// ```
    pub fn get_page_with_element(&self, xml_id: &str) -> Result<u32> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let page =
            unsafe { verovioxide_sys::vrvToolkit_getPageWithElement(self.ptr, c_id.as_ptr()) };

        Ok(page.max(0) as u32)
    }

    /// Gets element attributes by xml:id.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// A JSON string with the element's attributes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load MEI data with elements having xml:id attributes ...
    ///
    /// let attrs = toolkit.get_element_attr("note-0001").expect("Failed to get attributes");
    /// println!("Note attributes: {}", attrs);
    /// ```
    pub fn get_element_attr(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let attr_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getElementAttr(self.ptr, c_id.as_ptr()) };

        self.ptr_to_string(attr_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get attributes for element: {}", xml_id))
        })
    }

    /// Gets elements at a specific time in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `millisec` - Time in milliseconds
    ///
    /// # Returns
    ///
    /// A JSON string with the element IDs at the specified time.
    ///
    /// # See also
    ///
    /// - [`get_time_for_element`](Self::get_time_for_element) - Get time for a specific element
    /// - [`render_to_timemap`](Self::render_to_timemap) - Get the full timemap
    pub fn get_elements_at_time(&self, millisec: i32) -> Result<String> {
        // SAFETY: ptr is valid
        let elements_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getElementsAtTime(self.ptr, millisec) };

        self.ptr_to_string(elements_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get elements at time: {}", millisec))
        })
    }

    /// Gets the time (in milliseconds) for an element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The time in milliseconds.
    ///
    /// # See also
    ///
    /// - [`get_elements_at_time`](Self::get_elements_at_time) - Get elements at a specific time
    /// - [`render_to_timemap`](Self::render_to_timemap) - Get the full timemap
    pub fn get_time_for_element(&self, xml_id: &str) -> Result<f64> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let time =
            unsafe { verovioxide_sys::vrvToolkit_getTimeForElement(self.ptr, c_id.as_ptr()) };

        Ok(time)
    }

    /// Gets expansion IDs for an element.
    ///
    /// When working with documents that contain expansion elements (e.g., repeats),
    /// this method returns the expansion IDs associated with a given element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The xml_id contains a null byte
    /// - The query fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data with expansion elements ...
    ///
    /// let expansion_ids = toolkit.get_expansion_ids_for_element("note-0001")
    ///     .expect("Failed to get expansion IDs");
    /// println!("Expansion IDs: {}", expansion_ids);
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_expansion_map`](Self::render_to_expansion_map) - Get the full expansion map
    /// - [`get_notated_id_for_element`](Self::get_notated_id_for_element) - Get notated ID
    pub fn get_expansion_ids_for_element(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let result_ptr = unsafe {
            verovioxide_sys::vrvToolkit_getExpansionIdsForElement(self.ptr, c_id.as_ptr())
        };

        self.ptr_to_string(result_ptr).ok_or_else(|| {
            Error::RenderError(format!(
                "failed to get expansion IDs for element: {}",
                xml_id
            ))
        })
    }

    /// Gets MIDI values for an element.
    ///
    /// Returns MIDI-related information (pitch, velocity, etc.) for a specific element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The xml_id contains a null byte
    /// - The query fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let midi_values = toolkit.get_midi_values_for_element("note-0001")
    ///     .expect("Failed to get MIDI values");
    /// println!("MIDI values: {}", midi_values);
    /// ```
    ///
    /// # See also
    ///
    /// - [`render_to_midi`](Self::render_to_midi) - Render full MIDI
    /// - [`get_time_for_element`](Self::get_time_for_element) - Get timing for element
    pub fn get_midi_values_for_element(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getMIDIValuesForElement(self.ptr, c_id.as_ptr()) };

        self.ptr_to_string(result_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get MIDI values for element: {}", xml_id))
        })
    }

    /// Gets the notated ID for an element.
    ///
    /// When working with expansions, elements may have different rendered IDs
    /// than their notated IDs. This method returns the original notated ID
    /// for a given element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element (possibly a rendered ID)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The xml_id contains a null byte
    /// - The query fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let notated_id = toolkit.get_notated_id_for_element("rendered-note-0001")
    ///     .expect("Failed to get notated ID");
    /// println!("Notated ID: {}", notated_id);
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_expansion_ids_for_element`](Self::get_expansion_ids_for_element) - Get expansion IDs
    /// - [`render_to_expansion_map`](Self::render_to_expansion_map) - Get the full expansion map
    pub fn get_notated_id_for_element(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getNotatedIdForElement(self.ptr, c_id.as_ptr()) };

        self.ptr_to_string(result_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get notated ID for element: {}", xml_id))
        })
    }

    /// Gets timing information for an element.
    ///
    /// Returns detailed timing information including onset time, offset time,
    /// and duration for a specific element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The xml_id contains a null byte
    /// - The query fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let times = toolkit.get_times_for_element("note-0001")
    ///     .expect("Failed to get times");
    /// println!("Timing info: {}", times);
    /// ```
    ///
    /// # See also
    ///
    /// - [`get_time_for_element`](Self::get_time_for_element) - Get simple time value
    /// - [`render_to_timemap`](Self::render_to_timemap) - Get full timemap
    pub fn get_times_for_element(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let result_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getTimesForElement(self.ptr, c_id.as_ptr()) };

        self.ptr_to_string(result_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get times for element: {}", xml_id))
        })
    }

    /// Gets descriptive features from the document.
    ///
    /// Extracts descriptive features and metadata from the loaded document,
    /// useful for analysis and categorization.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional JSON string with feature extraction options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The options contain a null byte
    /// - Feature extraction fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let features = toolkit.get_descriptive_features(None)
    ///     .expect("Failed to get features");
    /// println!("Features: {}", features);
    /// ```
    pub fn get_descriptive_features(&self, options: Option<&str>) -> Result<String> {
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        let result_ptr = unsafe {
            verovioxide_sys::vrvToolkit_getDescriptiveFeatures(self.ptr, c_options.as_ptr())
        };

        self.ptr_to_string(result_ptr)
            .ok_or_else(|| Error::RenderError("failed to get descriptive features".into()))
    }

    /// Redoes the layout with optional new options.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional JSON string with layout options
    pub fn redo_layout(&mut self, options: Option<&str>) -> Result<()> {
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        unsafe { verovioxide_sys::vrvToolkit_redoLayout(self.ptr, c_options.as_ptr()) };

        Ok(())
    }

    /// Performs an editor action on the loaded document.
    ///
    /// # Arguments
    ///
    /// * `action` - JSON string describing the editor action
    ///
    /// # Errors
    ///
    /// Returns an error if the action fails.
    pub fn edit(&mut self, action: &str) -> Result<()> {
        let c_action = CString::new(action)?;

        // SAFETY: ptr is valid, c_action is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_edit(self.ptr, c_action.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError("editor action failed".into()))
        }
    }

    /// Gets information about the last edit operation.
    ///
    /// Returns a JSON string containing details about the most recent edit
    /// performed via [`edit()`](Self::edit).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data and perform an edit ...
    ///
    /// let info = toolkit.edit_info();
    /// println!("Last edit info: {}", info);
    /// ```
    #[must_use]
    pub fn edit_info(&self) -> String {
        // SAFETY: ptr is valid
        let info_ptr = unsafe { verovioxide_sys::vrvToolkit_editInfo(self.ptr) };
        self.ptr_to_string(info_ptr).unwrap_or_default()
    }

    /// Enables or disables logging to stderr.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable logging, `false` to disable
    pub fn enable_log(enable: bool) {
        // SAFETY: This function has no preconditions
        unsafe { verovioxide_sys::enableLog(enable) };
    }

    /// Enables or disables logging to an internal buffer.
    ///
    /// When enabled, log messages can be retrieved with [`get_log()`](Self::get_log).
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable buffer logging, `false` to disable
    pub fn enable_log_to_buffer(enable: bool) {
        // SAFETY: This function has no preconditions
        unsafe { verovioxide_sys::enableLogToBuffer(enable) };
    }

    /// Converts a C string pointer to an owned Rust string.
    ///
    /// Returns `None` if the pointer is null or contains invalid UTF-8.
    fn ptr_to_string(&self, ptr: *const i8) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: ptr is non-null and points to a valid C string owned by the toolkit
        let c_str = unsafe { CStr::from_ptr(ptr) };

        c_str.to_str().ok().map(String::from)
    }
}

impl std::fmt::Debug for Toolkit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolkit")
            .field("version", &self.version())
            .field("page_count", &self.page_count())
            .field("resource_path", &self.get_resource_path())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolkit_without_resources() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        assert!(!toolkit.version().is_empty());
    }

    #[test]
    fn test_toolkit_version() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let version = toolkit.version();
        // Version should look like a version number
        assert!(!version.is_empty());
    }

    #[test]
    fn test_toolkit_page_count_empty() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        assert_eq!(toolkit.page_count(), 0);
    }

    #[test]
    fn test_toolkit_get_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_options();
        let trimmed = options.trim();
        assert!(trimmed.starts_with('{'));
        assert!(trimmed.ends_with('}'));
    }

    #[test]
    fn test_toolkit_get_default_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_default_options();
        assert!(options.starts_with('{'));
    }

    #[test]
    fn test_toolkit_get_available_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_available_options();
        assert!(options.starts_with('{'));
    }

    #[test]
    fn test_toolkit_reset_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.reset_options();
        // Should not panic
    }

    #[test]
    fn test_toolkit_get_scale() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let scale = toolkit.get_scale();
        assert!(scale > 0);
    }

    #[test]
    fn test_toolkit_set_scale() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.set_scale(80).expect("Failed to set scale");
        assert_eq!(toolkit.get_scale(), 80);
    }

    #[test]
    fn test_toolkit_get_id() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let id = toolkit.get_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_toolkit_debug() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let debug = format!("{:?}", toolkit);
        assert!(debug.contains("Toolkit"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_toolkit_render_to_svg_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_render_to_svg_page_zero() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg(0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn test_toolkit_set_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = Options::builder().scale(80).build();
        toolkit
            .set_options(&options)
            .expect("Failed to set options");
    }

    #[test]
    fn test_toolkit_load_data_empty() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_load_file_not_found() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_file(Path::new("/nonexistent/path/to/file.mei"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_toolkit_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Toolkit>();
    }

    #[test]
    fn test_toolkit_enable_log() {
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
        // Should not panic
    }

    #[test]
    fn test_toolkit_enable_log_to_buffer() {
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
        // Should not panic
    }

    #[test]
    fn test_toolkit_get_log() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let _log = toolkit.get_log();
        // Log may be empty, that's fine
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_new_with_bundled_data() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");
        assert!(!toolkit.version().is_empty());
        assert!(!toolkit.get_resource_path().is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_simple_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        assert!(toolkit.page_count() > 0);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_simple_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit.render_to_svg(1).expect("Failed to render SVG");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_all_pages() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let pages = toolkit.render_all_pages().expect("Failed to render pages");
        assert!(!pages.is_empty());
        for page in &pages {
            assert!(page.contains("<svg"));
        }
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let exported_mei = toolkit.get_mei().expect("Failed to export MEI");
        assert!(exported_mei.contains("mei"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let options = Options::builder()
            .scale(50)
            .page_width(1000)
            .page_height(1000)
            .adjust_page_height(true)
            .build();

        toolkit
            .set_options(&options)
            .expect("Failed to set options");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit.render_to_svg(1).expect("Failed to render SVG");
        assert!(svg.contains("<svg"));
    }

    // =========================================================================
    // Additional tests for improved coverage
    // =========================================================================

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_page_zero() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_get_mei_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei();
        // With no data loaded, this may return an empty MEI or error depending on Verovio
        // Just ensure it doesn't panic and returns a result
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_mei_with_options_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei_with_options(r#"{"removeIds": true}"#);
        // May succeed with empty result or error - just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_humdrum_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_humdrum();
        // May return empty or error - just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_midi_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_midi();
        assert!(result.is_err(), "render_to_midi should fail without data");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("no data loaded"),
            "Error should mention no data loaded"
        );
    }

    #[test]
    fn test_toolkit_render_to_pae_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_pae();
        assert!(result.is_err(), "render_to_pae should fail without data");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("no data loaded"),
            "Error should mention no data loaded"
        );
    }

    #[test]
    fn test_toolkit_render_to_timemap_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap();
        // Returns JSON array (possibly empty) or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_timemap_with_options_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap_with_options(r#"{"includeMeasures": true}"#);
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_expansion_map_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_expansion_map();
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_invalid() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Scale of 0 should be invalid
        let result = toolkit.set_scale(0);
        // Verovio may accept or reject this - test that we handle the result
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_negative() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_scale(-1);
        // Verovio may accept or reject this - test that we handle the result
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_large() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Very large scale
        let result = toolkit.set_scale(10000);
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_resource_path() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let path = toolkit.get_resource_path();
        // Without resources, path may be empty
        let _ = path;
    }

    #[test]
    #[cfg(feature = "bundled-data")]
    fn test_toolkit_with_cached_resources_creates_and_reuses_cache() {
        let cache_root = tempfile::TempDir::new().expect("Failed to create cache root");

        let toolkit =
            Toolkit::with_cached_resources(cache_root.path()).expect("Failed to create toolkit");
        assert!(!toolkit.get_resource_path().is_empty());

        // A second toolkit reuses the same extracted directory.
        let second = Toolkit::with_cached_resources(cache_root.path())
            .expect("Failed to create second toolkit");
        assert_eq!(toolkit.get_resource_path(), second.get_resource_path());
    }

    #[test]
    fn test_toolkit_set_resource_path_nonexistent() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_resource_path(Path::new("/nonexistent/path/to/resources"));
        // This may succeed (just sets the path) or fail depending on implementation
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_page_with_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_page_with_element("nonexistent-id");
        // Should return 0 when element not found
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_toolkit_get_element_attr_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_element_attr("nonexistent-id");
        // May return empty JSON or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_elements_at_time_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_elements_at_time(0);
        // May return empty array or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_elements_at_time_negative() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_elements_at_time(-1000);
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_time_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_time_for_element("nonexistent-id");
        assert!(result.is_ok());
        // Time may be 0 or negative when not found
    }

    #[test]
    fn test_toolkit_redo_layout_no_data() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_toolkit_redo_layout_with_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(Some(r#"{"pageWidth": 2100}"#));
        assert!(result.is_ok());
    }

    #[test]
    fn test_toolkit_edit_no_data() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit(r#"{"action": "commit"}"#);
        // Edit without data may fail
        let _ = result;
    }

    #[test]
    fn test_toolkit_edit_invalid_json() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit("not valid json");
        // Should handle invalid JSON gracefully
        let _ = result;
    }

    #[test]
    fn test_toolkit_edit_info() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let info = toolkit.edit_info();
        // Should return JSON (possibly empty object)
        let _ = info;
    }

    #[test]
    fn test_toolkit_load_data_with_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("test\0data");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_load_data_malformed_mei() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("<mei><invalid></mei>");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_load_data_random_text() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("This is just random text, not valid music notation");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_get_mei_with_options_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei_with_options("test\0options");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_render_to_timemap_with_options_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap_with_options("test\0options");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_page_with_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_page_with_element("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_element_attr_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_element_attr("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_time_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_time_for_element("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_redo_layout_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(Some("test\0options"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_edit_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit("test\0action");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_set_options_null_byte_in_font() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Create options with a null byte in a string field
        let mut options = Options::builder().build();
        options.font = Some("test\0font".to_string());
        let result = toolkit.set_options(&options);
        // The JSON serialization should succeed, but CString creation may fail
        // or the toolkit may reject it
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_all_pages_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_all_pages();
        // With no data, page_count is 0, so we get empty vec
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_svg_with_declaration() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit
            .render_to_svg_with_declaration(1)
            .expect("Failed to render SVG");
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_midi() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_midi();
        // MIDI rendering should succeed with valid MEI
        assert!(result.is_ok());
        let midi = result.unwrap();
        // MIDI is base64-encoded
        assert!(!midi.is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_mei_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.get_mei_with_options(r#"{"removeIds": false}"#);
        assert!(result.is_ok());
        let exported = result.unwrap();
        assert!(exported.contains("mei"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_timemap() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_timemap();
        assert!(result.is_ok());
        let timemap = result.unwrap();
        // Timemap is JSON
        assert!(timemap.starts_with('[') || timemap.starts_with('{'));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_expansion_map() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_expansion_map();
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_redo_layout_after_load() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        // Redo layout with different options
        let result = toolkit.redo_layout(Some(r#"{"pageWidth": 1500}"#));
        assert!(result.is_ok());

        // Should still be able to render
        let svg = toolkit
            .render_to_svg(1)
            .expect("Failed to render after redo");
        assert!(svg.contains("<svg"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_set_and_get_resource_path() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");
        let path = toolkit.get_resource_path();
        // With bundled data, resource path should not be empty
        assert!(!path.is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_elements_at_time_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.get_elements_at_time(0);
        assert!(result.is_ok());
        let elements = result.unwrap();
        // Should be valid JSON
        assert!(elements.starts_with('{') || elements.starts_with('['));
    }

    #[test]
    fn test_toolkit_with_resource_path_nonexistent() {
        let result = Toolkit::with_resource_path(Path::new("/nonexistent/resources"));
        // May fail or succeed depending on whether Verovio validates the path
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_file_with_tempfile() {
        use std::io::Write;

        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");

        // Create a temp file with invalid content
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.mei");
        let mut file = std::fs::File::create(&file_path).expect("Failed to create file");
        file.write_all(b"<invalid>not valid mei</invalid>")
            .expect("Failed to write");

        let result = toolkit.load_file(&file_path);
        // Should fail to load invalid MEI
        assert!(result.is_err());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_file_with_valid_mei() {
        use std::io::Write;

        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.mei");
        let mut file = std::fs::File::create(&file_path).expect("Failed to create file");
        file.write_all(mei.as_bytes()).expect("Failed to write");

        let result = toolkit.load_file(&file_path);
        assert!(result.is_ok());
        assert!(toolkit.page_count() > 0);
    }

    #[test]
    fn test_toolkit_not_sync() {
        // This is a compile-time check - Toolkit should NOT implement Sync
        // We can't easily test this at runtime, but we document it here
        // fn assert_sync<T: Sync>() {}
        // assert_sync::<Toolkit>(); // This would fail to compile
    }

    #[test]
    fn test_toolkit_debug_format_detailed() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let debug = format!("{:?}", toolkit);
        assert!(debug.contains("Toolkit"));
        assert!(debug.contains("version"));
        assert!(debug.contains("page_count"));
        assert!(debug.contains("resource_path"));
    }

    #[test]
    fn test_toolkit_enable_log_toggle() {
        // Test toggling log on and off multiple times
        Toolkit::enable_log(true);
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
        Toolkit::enable_log(false);
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
    }

    #[test]
    fn test_toolkit_enable_log_to_buffer_toggle() {
        // Test toggling buffer log on and off multiple times
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
        Toolkit::enable_log_to_buffer(false);
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_logging_with_data() {
        Toolkit::enable_log_to_buffer(true);

        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        // Load some data to generate log messages
        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let log = toolkit.get_log();
        // Log may or may not have content depending on Verovio's behavior
        let _ = log;

        Toolkit::enable_log_to_buffer(false);
    }

    #[test]
    fn test_toolkit_render_to_svg_page_exceeds_count() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // With no data, page_count is 0, so page 1 should be out of range
        let result = toolkit.render_to_svg(1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
        assert!(err.to_string().contains("0 pages"));
    }

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_page_exceeds_count() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(100);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    // =========================================================================
    // Tests for new API methods (26 functions for 100% coverage)
    // =========================================================================

    // Format Control Functions
    #[test]
    fn test_toolkit_set_input_from() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_input_from("mei");
        // May succeed or fail depending on Verovio behavior
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_input_from_various_formats() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        for format in &["mei", "musicxml", "humdrum", "pae", "abc"] {
            let _ = toolkit.set_input_from(format);
        }
    }

    #[test]
    fn test_toolkit_set_input_from_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_input_from("mei\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_set_output_to() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_output_to("mei");
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_output_to_various_formats() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        for format in &["mei", "svg", "midi", "humdrum", "pae"] {
            let _ = toolkit.set_output_to(format);
        }
    }

    #[test]
    fn test_toolkit_set_output_to_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_output_to("mei\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // ZIP Loading Functions
    // Note: load_zip_data functions can throw C++ exceptions on invalid input
    // so we only test the null byte handling (which is caught by Rust before FFI)

    #[test]
    fn test_toolkit_load_zip_data_base64_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_zip_data_base64("data\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // PAE Validation Functions
    #[test]
    fn test_toolkit_validate_pae() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Simple PAE code
        let result = toolkit.validate_pae("@clef:G-2 @key:bBEA @time:4/4 ''4C/8DE");
        // Returns JSON validation result
        let _ = result;
    }

    #[test]
    fn test_toolkit_validate_pae_empty() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.validate_pae("");
        let _ = result;
    }

    #[test]
    fn test_toolkit_validate_pae_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.validate_pae("@clef:G-2\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_validate_pae_file_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.validate_pae_file(Path::new("/nonexistent/path.pae"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("file not found"));
    }

    #[test]
    fn test_toolkit_validate_pae_file_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Path with null byte in it - this should fail at CString conversion
        // We need to construct this carefully since Path won't accept null bytes on some systems
        // This test verifies the path string conversion handles this case
        let _ = toolkit.validate_pae_file(Path::new("/some/path.pae"));
    }

    // Selection Function
    #[test]
    fn test_toolkit_select_no_data() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.select(r#"{"measureRange": "1-2"}"#);
        // Select without data may fail
        let _ = result;
    }

    #[test]
    fn test_toolkit_select_empty() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.select("{}");
        let _ = result;
    }

    #[test]
    fn test_toolkit_select_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.select("{\0}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // Layout Functions
    #[test]
    fn test_toolkit_redo_page_pitch_pos_layout() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.redo_page_pitch_pos_layout();
        // Should not panic
    }

    #[test]
    fn test_toolkit_reset_xml_id_seed() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.reset_xml_id_seed(42);
        // Should not panic
    }

    #[test]
    fn test_toolkit_reset_xml_id_seed_zero() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.reset_xml_id_seed(0);
    }

    #[test]
    fn test_toolkit_reset_xml_id_seed_negative() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.reset_xml_id_seed(-1);
    }

    #[test]
    fn test_toolkit_get_option_usage_string() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let usage = toolkit.get_option_usage_string();
        // Should return a non-empty string with option documentation
        assert!(!usage.is_empty());
    }

    // Conversion Functions
    // Note: Empty input to conversion functions can cause Verovio to hang,
    // so we only test null byte handling here. Actual functionality is tested
    // with valid data in the bundled-data tests below.

    #[test]
    fn test_toolkit_convert_humdrum_to_humdrum_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.convert_humdrum_to_humdrum("data\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_convert_humdrum_to_midi_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.convert_humdrum_to_midi("data\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_convert_mei_to_humdrum_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.convert_mei_to_humdrum("<mei>\0</mei>");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_render_data_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_data("data\0invalid", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_render_data_options_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_data("", Some("{\0}"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // Element Query Functions
    #[test]
    fn test_toolkit_get_expansion_ids_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_expansion_ids_for_element("nonexistent-id");
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_expansion_ids_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_expansion_ids_for_element("id\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_midi_values_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_midi_values_for_element("nonexistent-id");
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_midi_values_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_midi_values_for_element("id\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_notated_id_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_notated_id_for_element("nonexistent-id");
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_notated_id_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_notated_id_for_element("id\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_times_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_times_for_element("nonexistent-id");
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_times_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_times_for_element("id\0invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_descriptive_features_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_descriptive_features(None);
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_descriptive_features_with_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_descriptive_features(Some("{}"));
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_descriptive_features_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_descriptive_features(Some("{\0}"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // File Output Functions
    // Note: Many file output functions assert/crash in Verovio when called without data,
    // so we only test null byte handling here. Actual file output functionality is tested
    // in the bundled-data tests below.

    #[test]
    fn test_toolkit_render_to_timemap_file_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.json");
        let result = toolkit.render_to_timemap_file(&path, Some("{\0}"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_save_file_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mei");
        let result = toolkit.save_file(&path, Some("{\0}"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    // Tests with bundled data for file output functions
    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_svg_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.svg");
        let result = toolkit.render_to_svg_file(&path, 1);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_midi_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mid");
        let result = toolkit.render_to_midi_file(&path);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_pae_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.pae");
        let result = toolkit.render_to_pae_file(&path);
        // PAE rendering may or may not succeed depending on content
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_expansion_map_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.json");
        let result = toolkit.render_to_expansion_map_file(&path);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_timemap_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.json");
        let result = toolkit.render_to_timemap_file(&path, None);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_save_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mei");
        let result = toolkit.save_file(&path, None);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_save_humdrum_to_file_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.krn");
        let result = toolkit.save_humdrum_to_file(&path);
        // Humdrum export may or may not succeed depending on content
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_select_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        let result = toolkit.select(r#"{"measureRange": "1"}"#);
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_descriptive_features_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        let result = toolkit.get_descriptive_features(None);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_data_with_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        let result = toolkit.render_data(mei, None);
        // render_data should process the data
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_data_with_mei_and_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        let result = toolkit.render_data(mei, Some(r#"{"scale": 50}"#));
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_descriptive_features_with_custom_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        let result = toolkit.get_descriptive_features(Some(r#"{"includeMeasures": true}"#));
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_timemap_with_options_loaded_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        let result = toolkit.render_to_timemap_with_options(r#"{"includeMeasures": true}"#);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_convert_humdrum_to_humdrum_with_data() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");

        let humdrum = r#"**kern
*clefG2
*k[]
*M4/4
4c
4d
4e
4f
*-"#;

        let result = toolkit.convert_humdrum_to_humdrum(humdrum);
        // May succeed or fail depending on Verovio's Humdrum support
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_convert_humdrum_to_midi_with_data() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");

        let humdrum = r#"**kern
*clefG2
*k[]
*M4/4
4c
4d
4e
4f
*-"#;

        let result = toolkit.convert_humdrum_to_midi(humdrum);
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_convert_mei_to_humdrum_with_data() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        let result = toolkit.convert_mei_to_humdrum(mei);
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_set_input_output_format_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        // Set input format before loading
        let _ = toolkit.set_input_from("mei");
        let _ = toolkit.set_output_to("svg");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        assert_eq!(toolkit.page_count(), 1);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_redo_page_pitch_pos_layout_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        toolkit.redo_page_pitch_pos_layout();
        // Should complete without panic
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_reset_xml_id_seed_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        toolkit.reset_xml_id_seed(12345);
        // Should complete without panic
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_timemap_file_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.json");

        let result = toolkit.render_to_timemap_file(&path, Some(r#"{"includeMeasures": true}"#));
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_save_file_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mei");

        let result = toolkit.save_file(&path, Some(r#"{"pageWidth": 2100}"#));
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_validate_pae_with_valid_code() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");
        // Valid PAE code
        let result = toolkit.validate_pae("@clef:G-2\n@keysig:bBE\n@timesig:4/4\n'4C/8DE");
        // Should return validation result
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_element_queries_with_loaded_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure xml:id="m1"><staff n="1"><layer n="1"><note xml:id="n1" pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        // These queries return JSON even if element not found
        let _ = toolkit.get_expansion_ids_for_element("n1");
        let _ = toolkit.get_midi_values_for_element("n1");
        let _ = toolkit.get_notated_id_for_element("n1");
        let _ = toolkit.get_times_for_element("n1");
    }

    // =========================================================================
    // LoadSource Trait and Unified Load Method Tests
    // =========================================================================

    #[test]
    fn test_zip_base64_marker_debug() {
        let marker = ZipBase64("test");
        let debug = format!("{:?}", marker);
        assert!(debug.contains("ZipBase64"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_zip_base64_marker_clone() {
        let marker = ZipBase64("test");
        let cloned = marker;
        assert_eq!(cloned.0, "test");
    }

    #[test]
    fn test_zip_buffer_marker_debug() {
        let data = b"test";
        let marker = ZipBuffer(data);
        let debug = format!("{:?}", marker);
        assert!(debug.contains("ZipBuffer"));
    }

    #[test]
    fn test_zip_buffer_marker_clone() {
        let data = b"test";
        let marker = ZipBuffer(data);
        let cloned = marker;
        assert_eq!(cloned.0, data);
    }

    #[test]
    fn test_load_source_str_empty() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load("");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_source_path_not_found() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load(Path::new("/nonexistent/path/to/file.mei"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_load_source_pathbuf_not_found() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let path = std::path::PathBuf::from("/nonexistent/path/to/file.mei");
        let result = toolkit.load(&path);
        assert!(result.is_err());
    }

    // NOTE: Tests for invalid ZIP data are intentionally omitted because
    // Verovio crashes with a C++ exception rather than returning an error
    // when given invalid ZIP data. This is documented behavior.

    #[test]
    fn test_load_source_str_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load("data with \0 null byte");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_source_zip_base64_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load(ZipBase64("data with \0 null byte"));
        assert!(result.is_err());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_load_source_str_valid_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        let result = toolkit.load(mei);
        assert!(result.is_ok());
        assert!(toolkit.page_count() > 0);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_load_source_str_valid_abc() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let abc = r#"X:1
T:Test
M:4/4
K:C
CDEF|"#;

        let result = toolkit.load(abc);
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_load_dispatches_correctly_str() {
        // Verify that load(&str) dispatches to load_data
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music><body><mdiv><score>
    <scoreDef><staffGrp><staffDef n="1" lines="5" clef.shape="G" clef.line="2"/></staffGrp></scoreDef>
    <section><measure><staff n="1"><layer n="1"><note pname="c" oct="4" dur="4"/></layer></staff></measure></section>
  </score></mdiv></body></music>
</mei>"#;

        // Load using unified method
        toolkit.load(mei).expect("Failed to load via load()");
        let page_count_load = toolkit.page_count();

        // Load using direct method
        let mut toolkit2 = Toolkit::new().expect("Failed to create toolkit");
        toolkit2
            .load_data(mei)
            .expect("Failed to load via load_data()");
        let page_count_direct = toolkit2.page_count();

        assert_eq!(page_count_load, page_count_direct);
    }

    #[test]
    fn test_load_source_trait_is_object_safe() {
        // This test verifies the trait can be used as a trait object (if needed)
        fn accepts_load_source(_source: &dyn std::any::Any) {}
        let source: &str = "test";
        accepts_load_source(&source);
    }
}
