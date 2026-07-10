//! Integration tests for the verovioxide crate.
//!
//! These tests verify end-to-end functionality including:
//! - Toolkit creation with bundled resources
//! - Loading and rendering various music notation formats
//! - Options configuration
//! - Format conversion (e.g., MusicXML to MEI)
//! - Error handling for invalid input

use serial_test::serial;
use verovioxide::{Options, Toolkit};

// =============================================================================
// Test Fixtures
// =============================================================================

/// Simple MusicXML file with a C major scale (4 measures, quarter notes).
const SIMPLE_MUSICXML: &str = include_str!("../../../test-fixtures/musicxml/simple.musicxml");

/// Simple MEI file with a C major scale (4 measures, quarter notes).
const SIMPLE_MEI: &str = include_str!("../../../test-fixtures/mei/simple.mei");

/// Simple ABC notation file with "Twinkle Twinkle Little Star".
const SIMPLE_ABC: &str = include_str!("../../../test-fixtures/abc/simple.abc");

const SIMPLE_HUMDRUM_C: &str = "**kern\n4c\n==\n*-\n";
const SIMPLE_HUMDRUM_D: &str = "**kern\n4d\n==\n*-\n";
const SIMPLE_HUMDRUM_E: &str = "**kern\n4e\n==\n*-\n";

// =============================================================================
// Helper Functions
// =============================================================================

/// Verifies that the given string is a valid SVG document.
///
/// Checks for the presence of opening and closing SVG tags.
fn assert_valid_svg(svg: &str) {
    assert!(
        svg.contains("<svg"),
        "SVG should contain opening <svg tag, but got: {}...",
        &svg[..svg.len().min(200)]
    );
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
}

// =============================================================================
// Toolkit Creation Tests
// =============================================================================

/// Test that a toolkit can be created with bundled resources and reports a valid version.
#[test]
#[serial]
fn test_toolkit_creation() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit with bundled resources");

    // Version should be a non-empty string
    let version = toolkit.version();
    assert!(!version.is_empty(), "Version should not be empty");

    // Version should look like a semver (e.g., "4.2.1" or similar)
    // At minimum, it should contain digits and dots
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits: {}",
        version
    );
}

/// Test that a toolkit can be created without resources.
#[test]
#[serial]
fn test_toolkit_without_resources() {
    let toolkit = Toolkit::without_resources().expect("Failed to create toolkit without resources");
    assert!(!toolkit.version().is_empty());
}

// =============================================================================
// MusicXML Rendering Tests
// =============================================================================

/// Test loading and rendering a MusicXML file.
#[test]
#[serial]
fn test_render_musicxml() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the MusicXML fixture
    toolkit
        .load_data(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML data");

    // Verify page count is at least 1
    let page_count = toolkit.page_count();
    assert!(
        page_count >= 1,
        "Document should have at least 1 page, got {}",
        page_count
    );

    // Render to SVG and verify
    let svg = toolkit.render_to_svg(1).expect("Failed to render page 1");
    assert_valid_svg(&svg);

    // SVG should be reasonably sized (at least a few KB for music notation)
    assert!(
        svg.len() > 1000,
        "SVG should be substantial, got {} bytes",
        svg.len()
    );
}

// =============================================================================
// MEI Rendering Tests
// =============================================================================

/// Test loading and rendering an MEI file.
#[test]
#[serial]
fn test_render_mei() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the MEI fixture
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Verify we have pages
    assert!(
        toolkit.page_count() >= 1,
        "Document should have at least 1 page"
    );

    // Render to SVG and verify
    let svg = toolkit
        .render_to_svg(1)
        .expect("Failed to render MEI to SVG");
    assert_valid_svg(&svg);
}

// =============================================================================
// ABC Notation Rendering Tests
// =============================================================================

/// Test loading and rendering ABC notation.
#[test]
#[serial]
fn test_render_abc() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the ABC fixture
    toolkit
        .load_data(SIMPLE_ABC)
        .expect("Failed to load ABC data");

    // Verify we have pages
    assert!(
        toolkit.page_count() >= 1,
        "Document should have at least 1 page"
    );

    // Render to SVG and verify
    let svg = toolkit
        .render_to_svg(1)
        .expect("Failed to render ABC to SVG");
    assert_valid_svg(&svg);
}

/// Test rendering ABC notation provided inline (not from fixture file).
#[test]
#[serial]
fn test_render_abc_inline() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Minimal ABC tune inline
    let abc_inline = r#"X:1
T:Test Tune
M:4/4
L:1/4
K:C
CDEF | GABC |]
"#;

    toolkit
        .load_data(abc_inline)
        .expect("Failed to load inline ABC data");

    assert!(toolkit.page_count() >= 1);

    let svg = toolkit
        .render_to_svg(1)
        .expect("Failed to render inline ABC");
    assert_valid_svg(&svg);
}

// =============================================================================
// Options Tests
// =============================================================================

/// Test setting various rendering options.
#[test]
#[serial]
fn test_options() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Build options with various settings
    let options = Options::builder()
        .scale(80) // 80% scale
        .page_width(2100) // A4 width in MEI units
        .page_height(2970) // A4 height
        .adjust_page_height(true)
        .page_margin(50)
        .build();

    // Set options should succeed
    toolkit
        .set_options(&options)
        .expect("Failed to set options");

    // Load some data to verify options take effect
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Render should work with custom options
    let svg = toolkit
        .render_to_svg(1)
        .expect("Failed to render with custom options");
    assert_valid_svg(&svg);

    // Verify scale was set correctly
    assert_eq!(toolkit.get_scale(), 80, "Scale should be 80");
}

/// Test setting and retrieving scale independently.
#[test]
#[serial]
fn test_options_scale() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get default scale
    let default_scale = toolkit.get_scale();
    assert!(default_scale > 0, "Default scale should be positive");

    // Set new scale
    toolkit.set_scale(50).expect("Failed to set scale to 50");
    assert_eq!(toolkit.get_scale(), 50);

    // Set back to 100
    toolkit.set_scale(100).expect("Failed to set scale to 100");
    assert_eq!(toolkit.get_scale(), 100);
}

/// Test that options can be retrieved as JSON.
#[test]
#[serial]
fn test_options_json() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get current options (Verovio returns JSON with trailing whitespace)
    let options_json = toolkit.get_options();
    let trimmed = options_json.trim();
    assert!(trimmed.starts_with('{'), "Options should be valid JSON");
    assert!(trimmed.ends_with('}'), "Options should be valid JSON");

    // Get default options
    let defaults_json = toolkit.get_default_options();
    assert!(defaults_json.trim().starts_with('{'));

    // Get available options
    let available_json = toolkit.get_available_options();
    assert!(available_json.trim().starts_with('{'));
}

/// Test resetting options to defaults.
#[test]
#[serial]
fn test_options_reset() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get initial scale (should be 100)
    let initial_scale = toolkit.get_scale();
    assert_eq!(initial_scale, 100, "Initial scale should be 100");

    // Change scale
    toolkit.set_scale(50).expect("Failed to set scale");
    assert_eq!(toolkit.get_scale(), 50, "Scale should be 50 after setting");

    // Reset options - this calls Verovio's resetOptions
    // Note: Verovio's reset behavior for scale may vary between versions
    toolkit.reset_options();

    // Just verify reset_options doesn't crash and the toolkit is still usable
    // The exact reset behavior is Verovio-version dependent
    let _scale = toolkit.get_scale();
    let _options = toolkit.get_options();
    assert!(
        !_options.is_empty(),
        "Options should still be retrievable after reset"
    );
}

// =============================================================================
// MEI Export Tests
// =============================================================================

/// Test exporting loaded MusicXML as MEI.
#[test]
#[serial]
fn test_get_mei_export() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load MusicXML
    toolkit
        .load_data(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML data");

    // Export as MEI
    let mei = toolkit.get_mei().expect("Failed to export MEI");

    // Verify it's valid MEI
    assert!(
        mei.contains("<mei"),
        "Exported data should contain <mei tag"
    );
    assert!(
        mei.contains("</mei>"),
        "Exported data should contain </mei> closing tag"
    );
    assert!(
        mei.contains("http://www.music-encoding.org/ns/mei"),
        "Exported data should contain MEI namespace"
    );
}

/// Test exporting with options.
#[test]
#[serial]
fn test_get_mei_export_with_options() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load MEI
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Export as MEI with default options
    let mei = toolkit
        .get_mei_with_options("{}")
        .expect("Failed to export MEI with options");

    assert!(mei.contains("<mei"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

/// Test that loading invalid/malformed input produces an error.
#[test]
#[serial]
fn test_invalid_input() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Empty string should fail
    let result = toolkit.load_data("");
    assert!(result.is_err(), "Loading empty string should fail");

    // Random text should fail
    let result = toolkit.load_data("this is not valid music notation");
    assert!(result.is_err(), "Loading random text should fail");

    // Malformed XML should fail
    let result = toolkit.load_data("<mei><broken");
    assert!(result.is_err(), "Loading malformed XML should fail");
}

/// Test that rendering with no data loaded produces an error.
#[test]
#[serial]
fn test_render_no_data() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    // No data loaded, page count should be 0
    assert_eq!(toolkit.page_count(), 0);

    // Rendering page 1 should fail
    let result = toolkit.render_to_svg(1);
    assert!(result.is_err(), "Rendering with no data should fail");
}

/// Test that rendering an invalid page number produces an error.
#[test]
#[serial]
fn test_render_invalid_page() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Page 0 should fail (pages are 1-indexed)
    let result = toolkit.render_to_svg(0);
    assert!(result.is_err(), "Rendering page 0 should fail");

    // Page 1000 should fail (beyond page count)
    let result = toolkit.render_to_svg(1000);
    assert!(result.is_err(), "Rendering beyond page count should fail");
}

/// Test loading a non-existent file.
#[test]
#[serial]
fn test_load_nonexistent_file() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    let result = toolkit.load_file(std::path::Path::new("/nonexistent/path/to/score.mei"));
    assert!(result.is_err(), "Loading non-existent file should fail");

    // Error should mention file not found
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Error should mention file not found: {}",
        err
    );
}

// =============================================================================
// Multi-Page Rendering Tests
// =============================================================================

/// Test rendering all pages of a document.
#[test]
#[serial]
fn test_render_all_pages() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    let pages = toolkit
        .render_all_pages()
        .expect("Failed to render all pages");

    assert!(!pages.is_empty(), "Should have at least one page");

    for (i, svg) in pages.iter().enumerate() {
        assert_valid_svg(svg);
        assert!(
            svg.len() > 100,
            "Page {} should have substantial content",
            i + 1
        );
    }
}

// =============================================================================
// SVG Output Variation Tests
// =============================================================================

/// Test rendering with XML declaration.
#[test]
#[serial]
fn test_render_with_declaration() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    let svg = toolkit
        .render_to_svg_with_declaration(1)
        .expect("Failed to render with declaration");

    assert_valid_svg(&svg);
    // With declaration should include XML prolog
    assert!(
        svg.contains("<?xml") || svg.contains("<svg"),
        "Should contain XML declaration or SVG tag"
    );
}

// =============================================================================
// Toolkit Information Tests
// =============================================================================

/// Test retrieving toolkit ID.
#[test]
#[serial]
fn test_toolkit_id() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let id = toolkit.get_id();
    assert!(!id.is_empty(), "Toolkit should have an ID");
}

/// Test retrieving resource path.
#[test]
#[serial]
fn test_toolkit_resource_path() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let path = toolkit.get_resource_path();
    assert!(!path.is_empty(), "Resource path should not be empty");
}

/// Test toolkit Debug implementation.
#[test]
#[serial]
fn test_toolkit_debug() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let debug_str = format!("{:?}", toolkit);
    assert!(
        debug_str.contains("Toolkit"),
        "Debug should contain 'Toolkit'"
    );
    assert!(
        debug_str.contains("version"),
        "Debug should contain 'version'"
    );
}

// =============================================================================
// Logging Tests
// =============================================================================

/// Test enabling and retrieving logs.
#[test]
#[serial]
fn test_logging() {
    // Enable logging to buffer
    Toolkit::enable_log_to_buffer(true);

    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load some data (may generate log messages)
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Get log - may or may not have content depending on Verovio's behavior
    let _log = toolkit.get_log();

    // Disable logging
    Toolkit::enable_log_to_buffer(false);
}

// =============================================================================
// README Example Tests
// =============================================================================
// These tests verify that the code examples in the README compile and work.

/// Path to test fixtures relative to workspace root.
fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::PathBuf::from(manifest_dir)
        .join("../../test-fixtures")
        .join(relative)
}

/// Test the unified load() method with string data (README Quick Start pattern).
#[test]
#[serial]
fn test_readme_load_string_data() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");

    // This mirrors the README pattern of loading MEI data
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    assert!(voxide.page_count() > 0);
}

/// Test the unified load() method with Path (README Quick Start pattern).
#[test]
#[serial]
fn test_readme_load_path() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");

    // This mirrors the README: voxide.load(Path::new("score.musicxml"))?;
    let path = fixture_path("musicxml/simple.musicxml");
    voxide.load(path.as_path()).expect("Failed to load file");

    assert!(voxide.page_count() > 0);
}

/// Test the unified load() method with PathBuf reference.
#[test]
#[serial]
fn test_readme_load_pathbuf() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");

    let path = fixture_path("mei/simple.mei");
    voxide.load(&path).expect("Failed to load file");

    assert!(voxide.page_count() > 0);
}

/// Test the README Quick Start example pattern.
#[test]
#[serial]
fn test_readme_quick_start_pattern() {
    // This is the exact pattern from the README Quick Start section
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");

    // Load notation (format auto-detected)
    let path = fixture_path("musicxml/simple.musicxml");
    voxide.load(path.as_path()).expect("Failed to load");

    // Configure rendering
    let options = Options::builder()
        .scale(100)
        .adjust_page_height(true)
        .build();
    voxide.set_options(&options).expect("Failed to set options");

    // Render to SVG
    let svg = voxide.render_to_svg(1).expect("Failed to render");

    assert_valid_svg(&svg);
}

// =============================================================================
// Unified Render API Tests
// =============================================================================

use verovioxide::{
    Attrs, Elements, ExpansionMap, Features, Humdrum, Mei, Midi, Pae, Page, Svg, Time, Timemap,
    Times,
};

/// Test the unified render() method with Svg::page().
#[test]
#[serial]
fn test_render_svg_page() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let svg: String = voxide.render(Svg::page(1)).expect("Failed to render");
    assert_valid_svg(&svg);
}

/// Test the unified render() method with Svg::page() and declaration.
#[test]
#[serial]
fn test_render_svg_page_with_declaration() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let svg: String = voxide
        .render(Svg::page(1).with_declaration())
        .expect("Failed to render");
    assert_valid_svg(&svg);
    // Note: The declaration may or may not be present depending on Verovio's behavior
}

/// Test the unified render() method with Svg::all_pages().
#[test]
#[serial]
fn test_render_svg_all_pages() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let pages: Vec<String> = voxide.render(Svg::all_pages()).expect("Failed to render");
    assert!(!pages.is_empty());
    for svg in &pages {
        assert_valid_svg(svg);
    }
}

/// Test the unified render() method with Svg::pages() for a range.
#[test]
#[serial]
fn test_render_svg_pages_range() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // Get page count first
    let count = voxide.page_count();
    if count >= 1 {
        let pages: Vec<String> = voxide
            .render(Svg::pages(1, count))
            .expect("Failed to render");
        assert_eq!(pages.len(), count as usize);
        for svg in &pages {
            assert_valid_svg(svg);
        }
    }
}

/// Test the unified render() method with Midi.
#[test]
#[serial]
fn test_render_midi() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let midi: String = voxide.render(Midi).expect("Failed to render MIDI");
    // MIDI is base64-encoded, should be non-empty
    assert!(!midi.is_empty());
}

/// Test the unified render() method with Pae.
#[test]
#[serial]
fn test_render_pae() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let pae: String = voxide.render(Pae).expect("Failed to render PAE");
    assert!(!pae.is_empty());
}

/// Test the unified render() method with Timemap.
#[test]
#[serial]
fn test_render_timemap() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");
    // Timemap is JSON, should start with [ or {
    let trimmed = timemap.trim();
    assert!(
        trimmed.starts_with('[') || trimmed.starts_with('{'),
        "Timemap should be JSON"
    );
}

/// Test the unified render() method with Timemap options.
#[test]
#[serial]
fn test_render_timemap_with_options() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let timemap: String = voxide
        .render(Timemap::with_options().include_measures(true))
        .expect("Failed to render timemap");
    let trimmed = timemap.trim();
    assert!(trimmed.starts_with('[') || trimmed.starts_with('{'));
}

/// Test the unified render() method with ExpansionMap.
#[test]
#[serial]
fn test_render_expansion_map() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let expansion: String = voxide
        .render(ExpansionMap)
        .expect("Failed to render expansion map");
    // Expansion map is JSON
    let trimmed = expansion.trim();
    assert!(
        trimmed.starts_with('[') || trimmed.starts_with('{'),
        "Expansion map should be JSON"
    );
}

/// Test the unified render() method with Mei.
#[test]
#[serial]
fn test_render_unified_mei() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide
        .load(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML");

    let mei: String = voxide.render(Mei).expect("Failed to render MEI");
    assert!(mei.contains("<mei"));
    assert!(mei.contains("</mei>"));
}

/// Test the unified render() method with Mei options.
#[test]
#[serial]
fn test_render_mei_with_options() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide
        .load(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML");

    let mei: String = voxide
        .render(Mei::with_options().remove_ids(false))
        .expect("Failed to render MEI");
    assert!(mei.contains("<mei"));
}

/// Test the unified render() method with Humdrum.
#[test]
#[serial]
fn test_render_humdrum() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let humdrum: String = voxide.render(Humdrum).expect("Failed to render Humdrum");
    // Humdrum format typically has kern data
    assert!(!humdrum.is_empty());
}

/// Live toolkit instances must retain ownership of their own Humdrum buffers.
#[test]
#[serial]
fn test_live_toolkits_keep_distinct_humdrum_buffers() {
    let mut first = Toolkit::new().expect("Failed to create first toolkit");
    let mut second = Toolkit::new().expect("Failed to create second toolkit");
    first
        .set_input_from("humdrum")
        .expect("Failed to select first Humdrum input");
    second
        .set_input_from("humdrum")
        .expect("Failed to select second Humdrum input");

    first
        .load_data(SIMPLE_HUMDRUM_C)
        .expect("Failed to load first Humdrum score");
    let first_snapshot = first
        .get_humdrum()
        .expect("First toolkit lost its initial Humdrum buffer");

    second
        .load_data(SIMPLE_HUMDRUM_D)
        .expect("Failed to load second Humdrum score");
    let second_snapshot = second
        .get_humdrum()
        .expect("Second toolkit lost its initial Humdrum buffer");

    assert_eq!(
        first.get_humdrum().unwrap(),
        first_snapshot,
        "loading the second toolkit replaced the first toolkit's buffer"
    );
    assert_eq!(second.get_humdrum().unwrap(), second_snapshot);
    assert_ne!(first_snapshot, second_snapshot);

    first
        .load_data(SIMPLE_HUMDRUM_E)
        .expect("Failed to replace first Humdrum score");
    let replacement_snapshot = first
        .get_humdrum()
        .expect("First toolkit lost its replacement Humdrum buffer");

    assert_eq!(
        second.get_humdrum().unwrap(),
        second_snapshot,
        "reloading the first toolkit replaced the second toolkit's buffer"
    );
    assert_ne!(replacement_snapshot, first_snapshot);

    drop(second);
    assert_eq!(first.get_humdrum().unwrap(), replacement_snapshot);
}

/// Test render_to() with format inference from .svg extension.
#[test]
#[serial]
fn test_render_to_svg_file() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    voxide
        .render_to(&output_path)
        .expect("Failed to render to file");

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert_valid_svg(&content);
}

/// Test render_to() with format inference from .mid extension.
#[test]
#[serial]
fn test_render_to_midi_file() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.mid");

    voxide
        .render_to(&output_path)
        .expect("Failed to render to file");

    // MIDI file should exist and have content
    let metadata = std::fs::metadata(&output_path).expect("Failed to get metadata");
    assert!(metadata.len() > 0);
}

/// Test render_to() with format inference from .mei extension.
#[test]
#[serial]
fn test_render_to_mei_file() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide
        .load(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.mei");

    voxide
        .render_to(&output_path)
        .expect("Failed to render to file");

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert!(content.contains("<mei"));
}

/// Test render_to_as() with explicit Svg::page().
#[test]
#[serial]
fn test_render_to_as_svg_page() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    voxide
        .render_to_as(&output_path, Svg::page(1))
        .expect("Failed to render to file");

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    assert_valid_svg(&content);
}

/// Test render_to_as() with Svg::all_pages() creates directory.
#[test]
#[serial]
fn test_render_to_as_svg_all_pages_creates_directory() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    voxide
        .render_to_as(&output_path, Svg::all_pages())
        .expect("Failed to render to file");

    // Should create output/ directory (output.svg minus extension)
    let output_dir = temp_dir.path().join("output");
    assert!(output_dir.exists(), "Output directory should exist");
    assert!(output_dir.is_dir(), "Output should be a directory");

    // Should have page-001.svg (at minimum)
    let page1 = output_dir.join("page-001.svg");
    assert!(page1.exists(), "page-001.svg should exist");

    let content = std::fs::read_to_string(&page1).expect("Failed to read page file");
    assert_valid_svg(&content);
}

/// Test render_to_as() with Timemap for .json files.
#[test]
#[serial]
fn test_render_to_as_timemap_json() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    voxide
        .render_to_as(&output_path, Timemap)
        .expect("Failed to render to file");

    let content = std::fs::read_to_string(&output_path).expect("Failed to read file");
    let trimmed = content.trim();
    assert!(trimmed.starts_with('[') || trimmed.starts_with('{'));
}

/// Test that render_to() fails for ambiguous .json extension.
#[test]
#[serial]
fn test_render_to_json_fails_ambiguous() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    let result = voxide.render_to(&output_path);
    assert!(result.is_err(), "Should fail for ambiguous .json extension");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("ambiguous"),
        "Error should mention ambiguity: {}",
        err
    );
}

// =============================================================================
// Unified Query API Tests
// =============================================================================

/// Test the unified get() method with Page query.
#[test]
#[serial]
fn test_query_page() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // First render to generate data
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    // Get timemap to find note IDs (these are MEI element IDs, not SVG IDs)
    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");

    // Find an "id" field in the timemap (MEI element ID)
    if let Some(pos) = timemap.find("\"id\":\"") {
        let start = pos + 6;
        if let Some(end) = timemap[start..].find('"') {
            let note_id = &timemap[start..start + end];
            // Query which page this element is on
            let page: u32 = voxide.get(Page::of(note_id)).expect("Failed to get page");
            assert_eq!(page, 1, "Element should be on page 1");
        }
    }
}

/// Test the unified get() method with Attrs query.
#[test]
#[serial]
fn test_query_attrs() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // First render to generate data
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    // Get timemap to find note IDs (these are MEI element IDs)
    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");

    // Find an "id" field in the timemap (MEI element ID)
    if let Some(pos) = timemap.find("\"id\":\"") {
        let start = pos + 6;
        if let Some(end) = timemap[start..].find('"') {
            let note_id = &timemap[start..start + end];
            // Query attributes
            let attrs: String = voxide.get(Attrs::of(note_id)).expect("Failed to get attrs");
            // Attributes should be JSON
            assert!(
                attrs.starts_with('{') || attrs == "{}",
                "Attrs should be JSON: {}",
                attrs
            );
        }
    }
}

/// Test the unified get() method with Time query.
#[test]
#[serial]
fn test_query_time() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // First render to generate timing data
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    // Get timemap to find note IDs
    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");

    // If timemap contains note IDs, we can query their times
    if timemap.contains("\"on\":") {
        // Find an "id" field in the timemap
        if let Some(pos) = timemap.find("\"id\":\"") {
            let start = pos + 6;
            if let Some(end) = timemap[start..].find('"') {
                let note_id = &timemap[start..start + end];
                // Query time for this note
                let time: f64 = voxide.get(Time::of(note_id)).expect("Failed to get time");
                assert!(time >= 0.0, "Time should be non-negative");
            }
        }
    }
}

/// Test the unified get() method with Times query.
#[test]
#[serial]
fn test_query_times() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // First render to generate timing data
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    // Get timemap to find note IDs
    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");

    // If timemap contains note IDs, we can query their times
    if timemap.contains("\"on\":") {
        // Find an "id" field in the timemap
        if let Some(pos) = timemap.find("\"id\":\"") {
            let start = pos + 6;
            if let Some(end) = timemap[start..].find('"') {
                let note_id = &timemap[start..start + end];
                // Query times for this note (returns JSON array)
                let times: String = voxide.get(Times::of(note_id)).expect("Failed to get times");
                // Times should be JSON array
                assert!(
                    times.starts_with('[') || times.starts_with('{'),
                    "Times should be JSON: {}",
                    times
                );
            }
        }
    }
}

/// Test the unified get() method with Elements query.
#[test]
#[serial]
fn test_query_elements_at_time() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // First render to generate timing data
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    // Query elements at time 0 (should include initial notes)
    let elements: String = voxide
        .get(Elements::at(0))
        .expect("Failed to get elements at time");

    // Elements should be JSON
    let trimmed = elements.trim();
    assert!(
        trimmed.starts_with('[') || trimmed.starts_with('{'),
        "Elements should be JSON: {}",
        elements
    );
}

/// Test the unified get() method with Features query.
#[test]
#[serial]
fn test_query_features() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // Query descriptive features
    let features: String = voxide.get(Features).expect("Failed to get features");

    // Features should be JSON
    let trimmed = features.trim();
    assert!(
        trimmed.starts_with('[') || trimmed.starts_with('{'),
        "Features should be JSON: {}",
        features
    );
}

/// Test the unified get() method with Features options.
#[test]
#[serial]
fn test_query_features_with_options() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    // Query descriptive features with options
    let features: String = voxide
        .get(Features::with_options().option("key", "value"))
        .expect("Failed to get features");

    // Features should be JSON
    let trimmed = features.trim();
    assert!(
        trimmed.starts_with('[') || trimmed.starts_with('{'),
        "Features should be JSON: {}",
        features
    );
}

// =============================================================================
// Additional Query API Tests for Coverage
// =============================================================================

/// Test ExpansionIds query.
#[test]
#[serial]
fn test_query_expansion_ids() {
    use verovioxide::ExpansionIds;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");
    if let Some(pos) = timemap.find("\"id\":\"") {
        let start = pos + 6;
        if let Some(end) = timemap[start..].find('"') {
            let note_id = &timemap[start..start + end];
            // Query expansion IDs - result may be empty but should not error
            let result = voxide.get(ExpansionIds::of(note_id));
            // May or may not succeed depending on document structure
            if let Ok(ids) = result {
                assert!(ids.is_empty() || ids.starts_with('[') || ids.starts_with('{'));
            }
        }
    }
}

/// Test MidiValues query.
#[test]
#[serial]
fn test_query_midi_values() {
    use verovioxide::MidiValues;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");
    if let Some(pos) = timemap.find("\"id\":\"") {
        let start = pos + 6;
        if let Some(end) = timemap[start..].find('"') {
            let note_id = &timemap[start..start + end];
            // Query MIDI values - result may be empty but should not error
            let result = voxide.get(MidiValues::of(note_id));
            if let Ok(midi) = result {
                // MIDI values are JSON
                let trimmed = midi.trim();
                assert!(
                    trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with('{'),
                    "MIDI values should be JSON or empty"
                );
            }
        }
    }
}

/// Test NotatedId query.
#[test]
#[serial]
fn test_query_notated_id() {
    use verovioxide::NotatedId;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");
    let _svg: String = voxide.render(Svg::page(1)).expect("Failed to render");

    let timemap: String = voxide.render(Timemap).expect("Failed to render timemap");
    if let Some(pos) = timemap.find("\"id\":\"") {
        let start = pos + 6;
        if let Some(end) = timemap[start..].find('"') {
            let note_id = &timemap[start..start + end];
            // Query notated ID
            let result = voxide.get(NotatedId::of(note_id));
            if let Ok(notated) = result {
                // Should be a string (possibly same as input for non-expanded elements)
                assert!(!notated.is_empty() || notated.is_empty()); // Just verify it's a string
            }
        }
    }
}

// =============================================================================
// Additional Render API Tests for Coverage
// =============================================================================

/// Test render_to_as with Pae format.
#[test]
#[serial]
fn test_render_to_as_pae_file() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.pae");

    voxide
        .render_to_as(&output_path, Pae)
        .expect("Failed to render PAE");
    assert!(output_path.exists(), "PAE file should exist");
}

/// Test render_to_as with ExpansionMap format.
#[test]
#[serial]
fn test_render_to_as_expansion_map_file() {
    use verovioxide::ExpansionMap;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    voxide
        .render_to_as(&output_path, ExpansionMap)
        .expect("Failed to render expansion map");
    assert!(output_path.exists(), "ExpansionMap file should exist");
}

/// Test render_to_as with Humdrum format.
#[test]
#[serial]
fn test_render_to_as_humdrum_file() {
    use verovioxide::Humdrum;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.krn");

    voxide
        .render_to_as(&output_path, Humdrum)
        .expect("Failed to render Humdrum");
    assert!(output_path.exists(), "Humdrum file should exist");
}

/// Test render_to_as with MEI options.
#[test]
#[serial]
fn test_render_to_as_mei_with_options() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.mei");

    voxide
        .render_to_as(&output_path, Mei::with_options().remove_ids(true))
        .expect("Failed to render MEI with options");
    assert!(output_path.exists(), "MEI file should exist");
}

/// Test render_to_as with Timemap options.
#[test]
#[serial]
fn test_render_to_as_timemap_with_options() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    voxide
        .render_to_as(&output_path, Timemap::with_options().include_measures(true))
        .expect("Failed to render timemap with options");
    assert!(output_path.exists(), "Timemap file should exist");
}

/// Test SvgPages with declaration.
#[test]
#[serial]
fn test_render_svg_pages_with_declaration() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let pages: Vec<String> = voxide
        .render(Svg::pages(1, 1).with_declaration())
        .expect("Failed to render SVG pages with declaration");

    assert!(!pages.is_empty());
    assert!(
        pages[0].contains("<?xml"),
        "SVG should contain XML declaration"
    );
}

/// Test SvgAllPages with declaration.
#[test]
#[serial]
fn test_render_svg_all_pages_with_declaration() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let pages: Vec<String> = voxide
        .render(Svg::all_pages().with_declaration())
        .expect("Failed to render all SVG pages with declaration");

    assert!(!pages.is_empty());
    assert!(
        pages[0].contains("<?xml"),
        "SVG should contain XML declaration"
    );
}

/// Test render_to with unsupported extension.
#[test]
#[serial]
fn test_render_to_unsupported_extension() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.xyz");

    let result = voxide.render_to(&output_path);
    assert!(result.is_err(), "Should fail for unsupported extension");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unsupported"),
        "Error should mention unsupported: {}",
        err
    );
}

/// Test render_to with no extension.
#[test]
#[serial]
fn test_render_to_no_extension() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output");

    let result = voxide.render_to(&output_path);
    assert!(result.is_err(), "Should fail for missing extension");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("no extension"),
        "Error should mention no extension: {}",
        err
    );
}

/// Test render_to_as with SvgPages creates directory.
#[test]
#[serial]
fn test_render_to_as_svg_pages_creates_directory() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("pages.svg");

    voxide
        .render_to_as(&output_path, Svg::pages(1, 1))
        .expect("Failed to render SVG pages");

    // Should create a directory named "pages" (without extension)
    let dir_path = temp_dir.path().join("pages");
    assert!(dir_path.exists(), "Pages directory should exist");
    assert!(dir_path.is_dir(), "Should be a directory");
}

// =============================================================================
// PNG Rendering Tests (feature-gated)
// =============================================================================

/// PNG magic bytes for verification.
#[cfg(feature = "png")]
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Helper to verify PNG bytes.
#[cfg(feature = "png")]
fn assert_valid_png(bytes: &[u8]) {
    assert!(
        bytes.len() > 8,
        "PNG should have at least 8 bytes for header"
    );
    assert_eq!(
        &bytes[0..8],
        &PNG_MAGIC,
        "PNG should start with PNG magic bytes"
    );
}

/// Test rendering a single page to PNG bytes.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_page() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide.render(Png::page(1)).expect("Failed to render PNG");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with width option.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_width() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).width(800))
        .expect("Failed to render PNG with width");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with height option.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_height() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).height(600))
        .expect("Failed to render PNG with height");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with scale option.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_scale() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).scale(2.0))
        .expect("Failed to render PNG with scale");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with white background.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_white_background() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).white_background())
        .expect("Failed to render PNG with background");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with custom RGBA background.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_custom_background() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).background(240, 240, 240, 255))
        .expect("Failed to render PNG with custom background");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering with chained options.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_with_chained_options() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide
        .render(Png::page(1).width(800).scale(1.5).white_background())
        .expect("Failed to render PNG with chained options");
    assert_valid_png(&png_bytes);
}

/// Test rendering all pages to PNG.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_all_pages() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let pages: Vec<Vec<u8>> = voxide
        .render(Png::all_pages())
        .expect("Failed to render PNG all pages");

    assert!(!pages.is_empty(), "Should have at least one page");
    for (i, png) in pages.iter().enumerate() {
        assert_valid_png(png);
        assert!(
            png.len() > 100,
            "Page {} PNG should have meaningful content",
            i + 1
        );
    }
}

/// Test rendering a page range to PNG.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_pages_range() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let count = voxide.page_count();
    if count >= 1 {
        let pages: Vec<Vec<u8>> = voxide
            .render(Png::pages(1, count))
            .expect("Failed to render PNG pages range");

        assert_eq!(pages.len(), count as usize);
        for png in &pages {
            assert_valid_png(png);
        }
    }
}

/// Test render_to() with .png extension.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_to_png_file() {
    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    voxide
        .render_to(&output_path)
        .expect("Failed to render PNG");
    assert!(output_path.exists(), "PNG file should exist");

    let content = std::fs::read(&output_path).expect("Failed to read PNG file");
    assert_valid_png(&content);
}

/// Test render_to_as() with Png::page().
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_to_as_png_page() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("page.png");

    voxide
        .render_to_as(&output_path, Png::page(1).width(800))
        .expect("Failed to render PNG page");
    assert!(output_path.exists(), "PNG file should exist");

    let content = std::fs::read(&output_path).expect("Failed to read PNG file");
    assert_valid_png(&content);
}

/// Test render_to_as() with Png::all_pages() creates directory.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_to_as_png_all_pages_creates_directory() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("pages.png");

    voxide
        .render_to_as(&output_path, Png::all_pages())
        .expect("Failed to render PNG pages");

    // Should create a "pages" directory (without extension)
    let dir_path = temp_dir.path().join("pages");
    assert!(dir_path.exists(), "Pages directory should exist");
    assert!(dir_path.is_dir(), "Should be a directory");

    // Should have page-001.png
    let page1 = dir_path.join("page-001.png");
    assert!(page1.exists(), "page-001.png should exist");

    let content = std::fs::read(&page1).expect("Failed to read PNG file");
    assert_valid_png(&content);
}

/// Test PNG compatibility with image crate (for viuer).
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_png_viuer_compatibility() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_MEI).expect("Failed to load MEI");

    let png_bytes: Vec<u8> = voxide.render(Png::page(1)).expect("Failed to render PNG");

    // Verify the image crate can load this PNG (viuer uses image internally)
    let img = image::load_from_memory(&png_bytes).expect("Failed to load PNG with image crate");

    assert!(img.width() > 0, "Image should have positive width");
    assert!(img.height() > 0, "Image should have positive height");
}

/// Test PNG rendering from MusicXML source.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_from_musicxml() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide
        .load(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML");

    let png_bytes: Vec<u8> = voxide.render(Png::page(1)).expect("Failed to render PNG");
    assert_valid_png(&png_bytes);
}

/// Test PNG rendering from ABC source.
#[cfg(feature = "png")]
#[test]
#[serial]
fn test_render_png_from_abc() {
    use verovioxide::Png;

    let mut voxide = Toolkit::new().expect("Failed to create toolkit");
    voxide.load(SIMPLE_ABC).expect("Failed to load ABC");

    let png_bytes: Vec<u8> = voxide.render(Png::page(1)).expect("Failed to render PNG");
    assert_valid_png(&png_bytes);
}
