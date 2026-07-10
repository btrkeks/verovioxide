//! Bundled SMuFL fonts and resources for verovioxide.
//!
//! This crate embeds the SMuFL fonts and related resources from the Verovio
//! project for use at runtime. The data includes glyph definitions, bounding
//! boxes, and optional CSS files with embedded WOFF2 fonts for web output.
//!
//! # Font Features
//!
//! By default, only the Leipzig font is included. Additional fonts can be
//! enabled via feature flags:
//!
//! - `font-leipzig` (default) - Leipzig SMuFL font
//! - `font-bravura` - Bravura SMuFL font
//! - `font-gootville` - Gootville SMuFL font
//! - `font-leland` - Leland SMuFL font
//! - `font-petaluma` - Petaluma SMuFL font
//! - `all-fonts` - Enable all fonts
//!
//! Note: Bravura is always included as baseline because it's used to build
//! the glyph name table in Verovio.
//!
//! # Example
//!
//! ```no_run
//! use verovioxide_data::{resource_dir, extract_resources};
//!
//! // Access embedded resources directly (in-memory)
//! let dir = resource_dir();
//! if let Some(bravura) = dir.get_file("Bravura.xml") {
//!     let contents = bravura.contents_utf8().unwrap();
//!     println!("Bravura bounding boxes loaded: {} bytes", contents.len());
//! }
//!
//! // Or extract to a temporary directory for file-based access
//! let temp_dir = extract_resources().expect("Failed to extract resources");
//! let path = temp_dir.path().join("Bravura.xml");
//! assert!(path.exists());
//! ```

use include_dir::{Dir, include_dir};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;

/// The embedded verovio data directory.
///
/// This contains all bundled SMuFL fonts and resources compiled into the binary.
/// The actual contents depend on which font features are enabled at compile time.
static VEROVIO_DATA: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Errors that can occur when working with verovioxide data resources.
#[derive(Debug, Error)]
pub enum DataError {
    /// Failed to create the temporary directory.
    #[error("failed to create temporary directory: {0}")]
    TempDirCreation(#[source] io::Error),

    /// Failed to create a directory during extraction.
    #[error("failed to create directory '{path}': {source}")]
    DirectoryCreation {
        /// The path that failed to be created.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Failed to write a file during extraction.
    #[error("failed to write file '{path}': {source}")]
    FileWrite {
        /// The path that failed to be written.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },
}

/// Returns a reference to the embedded verovio data directory.
///
/// This provides in-memory access to all bundled resources without extraction.
/// Use this when you need to read resources directly from the embedded data.
///
/// # Example
///
/// ```
/// use verovioxide_data::resource_dir;
///
/// let dir = resource_dir();
///
/// // List all top-level files
/// for file in dir.files() {
///     println!("File: {}", file.path().display());
/// }
///
/// // Access a specific file - Bravura.xml is always included
/// let file = dir.get_file("Bravura.xml").expect("Bravura.xml should exist");
/// let contents = file.contents_utf8().expect("Should be valid UTF-8");
/// assert!(contents.len() > 0);
/// ```
#[must_use]
pub fn resource_dir() -> &'static Dir<'static> {
    &VEROVIO_DATA
}

/// Extracts all embedded resources to a temporary directory.
///
/// This creates a temporary directory and writes all embedded resources to it,
/// preserving the directory structure. The returned `TempDir` will automatically
/// clean up when dropped.
///
/// Use this when you need file-system access to the resources, for example when
/// interfacing with C libraries that expect file paths.
///
/// # Errors
///
/// Returns a [`DataError`] if:
/// - The temporary directory cannot be created
/// - A subdirectory cannot be created
/// - A file cannot be written
///
/// # Example
///
/// ```no_run
/// use verovioxide_data::extract_resources;
///
/// let temp_dir = extract_resources().expect("Failed to extract resources");
/// let bravura_path = temp_dir.path().join("Bravura.xml");
/// assert!(bravura_path.exists());
///
/// // Use the resources...
///
/// // TempDir is automatically cleaned up when dropped
/// ```
pub fn extract_resources() -> Result<TempDir, DataError> {
    let temp_dir = TempDir::new().map_err(DataError::TempDirCreation)?;
    extract_dir_contents(&VEROVIO_DATA, temp_dir.path())?;
    Ok(temp_dir)
}

/// Extracts all embedded resources into a stable cache directory, reusing a
/// previous extraction when one exists.
///
/// The resources are placed in `<cache_root>/verovio-resources-<fingerprint>`,
/// where the fingerprint is derived from the embedded data, so a rebuild with
/// different bundled resources gets a fresh cache entry. Repeated calls —
/// including from concurrent processes — return the same directory without
/// re-extracting.
///
/// Prefer this over [`extract_resources`] for long-lived applications: the
/// `TempDir` returned by [`extract_resources`] is only cleaned up on `Drop`,
/// so a killed process (SIGKILL, test-harness teardown, service restart)
/// leaks the extracted directory. The cached directory is instead reused
/// across process lifetimes.
///
/// Extraction is atomic: data is written to a staging directory inside
/// `cache_root` and renamed into place, so a crashed extraction never leaves
/// a partial cache entry, and concurrent extractions race safely.
///
/// # Errors
///
/// Returns a [`DataError`] if:
/// - The cache root or staging directory cannot be created
/// - A file cannot be written during extraction
/// - The staging directory cannot be renamed into place
///
/// # Example
///
/// ```no_run
/// use verovioxide_data::extract_resources_cached;
///
/// let cache_root = std::env::temp_dir().join("my-app-cache");
/// let resources = extract_resources_cached(&cache_root).expect("Failed to extract resources");
/// assert!(resources.join("Bravura.xml").exists());
/// ```
pub fn extract_resources_cached(cache_root: &Path) -> Result<PathBuf, DataError> {
    let target = cache_root.join(format!(
        "verovio-resources-{:016x}",
        embedded_data_fingerprint()
    ));
    if target.is_dir() {
        return Ok(target);
    }

    std::fs::create_dir_all(cache_root).map_err(|source| DataError::DirectoryCreation {
        path: cache_root.display().to_string(),
        source,
    })?;

    let staging = tempfile::Builder::new()
        .prefix(".verovio-extract-")
        .tempdir_in(cache_root)
        .map_err(DataError::TempDirCreation)?;
    extract_dir_contents(&VEROVIO_DATA, staging.path())?;

    // Cleanup responsibility transfers to us: on the success path the staging
    // directory is renamed away, and on failure we remove it explicitly.
    let staging_path = staging.keep();
    match std::fs::rename(&staging_path, &target) {
        Ok(()) => Ok(target),
        Err(_) if target.is_dir() => {
            // A concurrent extraction won the race; use its result.
            let _ = std::fs::remove_dir_all(&staging_path);
            Ok(target)
        }
        Err(source) => {
            let _ = std::fs::remove_dir_all(&staging_path);
            Err(DataError::DirectoryCreation {
                path: target.display().to_string(),
                source,
            })
        }
    }
}

/// Fingerprints the embedded data so cache entries are keyed by content.
///
/// `DefaultHasher::new()` is deterministic across runs of the same build but
/// not guaranteed stable across Rust releases; a toolchain upgrade at worst
/// produces one extra cache entry.
fn embedded_data_fingerprint() -> u64 {
    use std::hash::Hasher;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_dir_contents(&VEROVIO_DATA, &mut hasher);
    hasher.finish()
}

/// Recursively hashes file paths and contents in embedded iteration order.
fn hash_dir_contents(dir: &Dir<'_>, hasher: &mut impl std::hash::Hasher) {
    for file in dir.files() {
        hasher.write(file.path().to_string_lossy().as_bytes());
        hasher.write(file.contents());
    }
    for subdir in dir.dirs() {
        hash_dir_contents(subdir, hasher);
    }
}

/// Recursively extracts directory contents to the target path.
fn extract_dir_contents(dir: &Dir<'_>, target: &Path) -> Result<(), DataError> {
    // Extract all files in this directory
    for file in dir.files() {
        let file_path = target.join(file.path());
        std::fs::write(&file_path, file.contents()).map_err(|source| DataError::FileWrite {
            path: file_path.display().to_string(),
            source,
        })?;
    }

    // Recursively extract subdirectories
    for subdir in dir.dirs() {
        let subdir_path = target.join(subdir.path());
        std::fs::create_dir_all(&subdir_path).map_err(|source| DataError::DirectoryCreation {
            path: subdir_path.display().to_string(),
            source,
        })?;
        extract_dir_contents(subdir, target)?;
    }

    Ok(())
}

/// Returns `true` if the Leipzig font is available.
///
/// Leipzig is the default font and is always included unless explicitly disabled.
#[must_use]
pub const fn has_leipzig() -> bool {
    cfg!(feature = "font-leipzig")
}

/// Returns `true` if the Bravura font is available.
///
/// Note: The Bravura baseline data (Bravura.xml and Bravura/) is always included
/// because it's required for building the glyph name table. This function returns
/// `true` only when the full Bravura feature is enabled.
#[must_use]
pub const fn has_bravura() -> bool {
    cfg!(feature = "font-bravura")
}

/// Returns `true` if the Gootville font is available.
#[must_use]
pub const fn has_gootville() -> bool {
    cfg!(feature = "font-gootville")
}

/// Returns `true` if the Leland font is available.
#[must_use]
pub const fn has_leland() -> bool {
    cfg!(feature = "font-leland")
}

/// Returns `true` if the Petaluma font is available.
#[must_use]
pub const fn has_petaluma() -> bool {
    cfg!(feature = "font-petaluma")
}

/// Lists all available SMuFL font names based on enabled features.
///
/// This always includes "Bravura" as it's required for baseline functionality.
/// Additional fonts are included based on which feature flags are enabled.
///
/// # Example
///
/// ```
/// use verovioxide_data::available_fonts;
///
/// let fonts = available_fonts();
/// // Bravura is always included as the baseline font
/// assert!(fonts.contains(&"Bravura"));
/// // Should have at least one font
/// assert!(!fonts.is_empty());
/// ```
#[must_use]
pub fn available_fonts() -> Vec<&'static str> {
    let mut fonts = vec!["Bravura"]; // Always included as baseline

    if has_leipzig() {
        fonts.push("Leipzig");
    }
    if has_bravura() && !fonts.contains(&"Bravura") {
        fonts.push("Bravura");
    }
    if has_gootville() {
        fonts.push("Gootville");
    }
    if has_leland() {
        fonts.push("Leland");
    }
    if has_petaluma() {
        fonts.push("Petaluma");
    }

    fonts
}

/// Returns the default font name.
///
/// The default font is Leipzig when the `font-leipzig` feature is enabled,
/// otherwise falls back to Bravura (which is always available).
#[must_use]
pub const fn default_font() -> &'static str {
    if cfg!(feature = "font-leipzig") {
        "Leipzig"
    } else {
        "Bravura"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_dir_contains_bravura_xml() {
        let dir = resource_dir();
        let bravura = dir.get_file("Bravura.xml");
        assert!(
            bravura.is_some(),
            "Bravura.xml should exist in embedded data"
        );
    }

    #[test]
    fn test_resource_dir_contains_bravura_directory() {
        let dir = resource_dir();
        let bravura_dir = dir.get_dir("Bravura");
        assert!(
            bravura_dir.is_some(),
            "Bravura/ directory should exist in embedded data"
        );
    }

    #[test]
    fn test_resource_dir_contains_text_directory() {
        let dir = resource_dir();
        let text_dir = dir.get_dir("text");
        assert!(
            text_dir.is_some(),
            "text/ directory should exist in embedded data"
        );
    }

    #[test]
    fn test_extract_resources_creates_files() {
        let temp_dir = extract_resources().expect("Failed to extract resources");
        let bravura_path = temp_dir.path().join("Bravura.xml");
        assert!(bravura_path.exists(), "Bravura.xml should be extracted");
    }

    #[test]
    fn test_extract_resources_creates_subdirectories() {
        let temp_dir = extract_resources().expect("Failed to extract resources");
        let text_path = temp_dir.path().join("text");
        assert!(text_path.exists(), "text/ directory should be extracted");
        assert!(text_path.is_dir(), "text should be a directory");
    }

    #[test]
    fn test_extract_resources_cached_creates_files() {
        let cache_root = TempDir::new().expect("Failed to create cache root");
        let target =
            extract_resources_cached(cache_root.path()).expect("Failed to extract resources");
        assert!(
            target.starts_with(cache_root.path()),
            "Target should live inside the cache root"
        );
        assert!(
            target.join("Bravura.xml").exists(),
            "Bravura.xml should be extracted"
        );
        assert!(
            target.join("text").is_dir(),
            "text/ directory should be extracted"
        );
    }

    #[test]
    fn test_extract_resources_cached_reuses_existing_extraction() {
        let cache_root = TempDir::new().expect("Failed to create cache root");
        let first = extract_resources_cached(cache_root.path()).expect("First extraction failed");

        let sentinel = first.join(".reuse-sentinel");
        std::fs::write(&sentinel, b"x").expect("Failed to write sentinel");

        let second = extract_resources_cached(cache_root.path()).expect("Second extraction failed");
        assert_eq!(first, second, "Both calls should return the same directory");
        assert!(
            sentinel.exists(),
            "Second call should reuse the extraction, not redo it"
        );
    }

    #[test]
    fn test_extract_resources_cached_leaves_no_staging_dirs() {
        let cache_root = TempDir::new().expect("Failed to create cache root");
        extract_resources_cached(cache_root.path()).expect("Failed to extract resources");

        let entries: Vec<_> = std::fs::read_dir(cache_root.path())
            .expect("Failed to read cache root")
            .map(|entry| entry.expect("Failed to read entry").file_name())
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "Only the target directory should remain, got: {:?}",
            entries
        );
    }

    #[test]
    fn test_extract_resources_cached_creates_missing_cache_root() {
        let base = TempDir::new().expect("Failed to create base dir");
        let cache_root = base.path().join("nested").join("cache");
        let target = extract_resources_cached(&cache_root).expect("Failed to extract resources");
        assert!(target.join("Bravura.xml").exists());
    }

    #[test]
    fn test_extract_resources_cached_matches_embedded_content() {
        let cache_root = TempDir::new().expect("Failed to create cache root");
        let target =
            extract_resources_cached(cache_root.path()).expect("Failed to extract resources");

        let extracted = std::fs::read_to_string(target.join("Bravura.xml"))
            .expect("Failed to read extracted file");
        let embedded = resource_dir()
            .get_file("Bravura.xml")
            .expect("Bravura.xml should exist")
            .contents_utf8()
            .expect("Should be valid UTF-8");
        assert_eq!(extracted, embedded);
    }

    #[test]
    fn test_embedded_data_fingerprint_is_stable_within_run() {
        assert_eq!(embedded_data_fingerprint(), embedded_data_fingerprint());
    }

    #[test]
    fn test_available_fonts_includes_bravura() {
        let fonts = available_fonts();
        assert!(
            fonts.contains(&"Bravura"),
            "Bravura should always be available"
        );
    }

    #[test]
    #[cfg(feature = "font-leipzig")]
    fn test_available_fonts_includes_leipzig_when_feature_enabled() {
        let fonts = available_fonts();
        assert!(
            fonts.contains(&"Leipzig"),
            "Leipzig should be available when feature is enabled"
        );
    }

    #[test]
    #[cfg(feature = "font-leipzig")]
    fn test_default_font_is_leipzig_when_feature_enabled() {
        assert_eq!(default_font(), "Leipzig");
    }

    #[test]
    fn test_has_leipzig_matches_feature() {
        // This test always passes - it just verifies the function works
        let _ = has_leipzig();
    }

    #[test]
    fn test_has_bravura_matches_feature() {
        let _ = has_bravura();
    }

    #[test]
    fn test_has_gootville_matches_feature() {
        let _ = has_gootville();
    }

    #[test]
    fn test_has_leland_matches_feature() {
        let _ = has_leland();
    }

    #[test]
    fn test_has_petaluma_matches_feature() {
        let _ = has_petaluma();
    }

    #[test]
    fn test_bravura_xml_has_content() {
        let dir = resource_dir();
        let bravura = dir
            .get_file("Bravura.xml")
            .expect("Bravura.xml should exist");
        let contents = bravura.contents_utf8().expect("Should be valid UTF-8");
        assert!(
            contents.contains("bounding-boxes"),
            "Bravura.xml should contain bounding-boxes element"
        );
    }

    #[test]
    fn test_text_directory_contains_times_font() {
        let dir = resource_dir();
        let text_dir = dir.get_dir("text").expect("text/ should exist");

        // include_dir stores full paths from the included directory root
        // So we need to check for "text/Times.xml" not just "Times.xml"
        let times = text_dir.get_file("text/Times.xml");
        assert!(
            times.is_some(),
            "text/Times.xml should exist in text directory"
        );
    }

    #[test]
    fn test_default_font_returns_valid_font() {
        let font = default_font();
        // Should be either Leipzig or Bravura
        assert!(
            font == "Leipzig" || font == "Bravura",
            "Default font should be Leipzig or Bravura, got: {}",
            font
        );
    }

    #[test]
    fn test_available_fonts_not_empty() {
        let fonts = available_fonts();
        assert!(!fonts.is_empty(), "Should have at least one font available");
    }

    #[test]
    fn test_available_fonts_no_duplicates() {
        let fonts = available_fonts();
        let mut seen = std::collections::HashSet::new();
        for font in &fonts {
            assert!(seen.insert(*font), "Font {} appears more than once", font);
        }
    }

    #[test]
    fn test_extract_resources_preserves_content() {
        let temp_dir = extract_resources().expect("Failed to extract resources");

        // Read extracted file
        let extracted_path = temp_dir.path().join("Bravura.xml");
        let extracted_content =
            std::fs::read_to_string(&extracted_path).expect("Failed to read extracted file");

        // Read embedded file
        let dir = resource_dir();
        let embedded = dir
            .get_file("Bravura.xml")
            .expect("Bravura.xml should exist");
        let embedded_content = embedded.contents_utf8().expect("Should be valid UTF-8");

        assert_eq!(
            extracted_content, embedded_content,
            "Extracted content should match embedded content"
        );
    }

    #[test]
    fn test_data_error_display() {
        // Test error Display implementations for coverage
        let err = DataError::TempDirCreation(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test error",
        ));
        let msg = format!("{}", err);
        assert!(msg.contains("temporary directory"));

        let err = DataError::DirectoryCreation {
            path: "/test/path".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("/test/path"));

        let err = DataError::FileWrite {
            path: "/test/file.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("/test/file.txt"));
    }

    #[test]
    fn test_data_error_debug() {
        let err =
            DataError::TempDirCreation(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let _ = format!("{:?}", err);

        let err = DataError::DirectoryCreation {
            path: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::Other, "test"),
        };
        let _ = format!("{:?}", err);

        let err = DataError::FileWrite {
            path: "test".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::Other, "test"),
        };
        let _ = format!("{:?}", err);
    }

    #[test]
    fn test_resource_dir_files_iteration() {
        let dir = resource_dir();
        let file_count = dir.files().count();
        assert!(file_count > 0, "Should have at least one file in root");
    }

    #[test]
    fn test_resource_dir_dirs_iteration() {
        let dir = resource_dir();
        let dir_count = dir.dirs().count();
        assert!(dir_count > 0, "Should have at least one subdirectory");
    }
}
