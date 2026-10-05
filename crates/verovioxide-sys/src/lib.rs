//! Raw FFI bindings to the Verovio C++ library.
//!
//! This crate provides unsafe, low-level bindings to the Verovio music engraving library
//! via its C wrapper API. For a safe, idiomatic Rust API, use the `verovioxide` crate instead.
//!
//! # Safety
//!
//! All functions in this crate are unsafe and require careful handling:
//!
//! - Toolkit pointers must be valid and non-null (obtained from constructor functions)
//! - String pointers must be valid null-terminated C strings
//! - Returned string pointers are owned by the toolkit and may be invalidated by subsequent calls
//! - The toolkit must be properly destroyed with `vrvToolkit_destructor` to avoid memory leaks
//!
//! # Example
//!
//! ```no_run
//! use std::ffi::{CStr, CString};
//! use verovioxide_sys::*;
//!
//! unsafe {
//!     // Create a toolkit instance
//!     let toolkit = vrvToolkit_constructorNoResource();
//!     assert!(!toolkit.is_null());
//!
//!     // Get the version
//!     let version_ptr = vrvToolkit_getVersion(toolkit);
//!     let version = CStr::from_ptr(version_ptr).to_string_lossy();
//!     println!("Verovio version: {}", version);
//!
//!     // Clean up
//!     vrvToolkit_destructor(toolkit);
//! }
//! ```
//!
//! # Memory Management
//!
//! - String values returned by Verovio are stored internally in the toolkit instance
//! - These strings remain valid until the next call that returns a string, or until
//!   the toolkit is destroyed
//! - Callers should copy returned strings if they need to persist beyond the next API call

#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_double, c_int, c_uchar, c_void};

// Re-export bindings module if it exists
mod bindings;
pub use bindings::*;

unsafe extern "C" {
    // =========================================================================
    // Global Functions
    // =========================================================================

    /// Enable or disable logging output.
    ///
    /// When enabled, Verovio will output log messages to stderr.
    ///
    /// # Arguments
    ///
    /// * `value` - `true` to enable logging, `false` to disable
    pub fn enableLog(value: bool);

    /// Enable or disable logging to an internal buffer.
    ///
    /// When enabled, log messages are stored in a buffer that can be retrieved
    /// with `vrvToolkit_getLog`.
    ///
    /// # Arguments
    ///
    /// * `value` - `true` to enable buffer logging, `false` to disable
    pub fn enableLogToBuffer(value: bool);

    // =========================================================================
    // Constructor / Destructor
    // =========================================================================

    /// Create a new Verovio toolkit instance with default resource path.
    ///
    /// The default resource path is set to "/data".
    ///
    /// # Returns
    ///
    /// A pointer to the newly created toolkit instance, or null on failure.
    pub fn vrvToolkit_constructor() -> *mut c_void;

    /// Create a new Verovio toolkit instance with a custom resource path.
    ///
    /// # Arguments
    ///
    /// * `resourcePath` - Path to the Verovio resources directory (null-terminated C string)
    ///
    /// # Returns
    ///
    /// A pointer to the newly created toolkit instance, or null on failure.
    pub fn vrvToolkit_constructorResourcePath(resourcePath: *const c_char) -> *mut c_void;

    /// Create a new Verovio toolkit instance without loading resources.
    ///
    /// This is useful for operations that don't require font resources.
    ///
    /// # Returns
    ///
    /// A pointer to the newly created toolkit instance, or null on failure.
    pub fn vrvToolkit_constructorNoResource() -> *mut c_void;

    /// Destroy a Verovio toolkit instance and free its resources.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance to destroy
    ///
    /// # Safety
    ///
    /// The toolkit pointer must be valid and must not be used after this call.
    pub fn vrvToolkit_destructor(tkPtr: *mut c_void);

    // =========================================================================
    // Editing Functions
    // =========================================================================

    /// Perform an editor action on the loaded document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `editorAction` - JSON string describing the editor action
    ///
    /// # Returns
    ///
    /// `true` if the action was successful, `false` otherwise.
    pub fn vrvToolkit_edit(tkPtr: *mut c_void, editorAction: *const c_char) -> bool;

    /// Get the status of the last edit operation.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// A JSON string with the edit status. The pointer is valid until the next
    /// API call that returns a string.
    pub fn vrvToolkit_editStatus(tkPtr: *mut c_void) -> *const c_char;

    /// Get the editor response of the last edit operation (experimental upstream).
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The editor response as a string. The pointer is valid until the next
    /// API call that returns a string.
    pub fn vrvToolkit_editResponse(tkPtr: *mut c_void) -> *const c_char;

    // =========================================================================
    // Options Functions
    // =========================================================================

    /// Get all available options and their descriptions.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// A JSON string describing all available options.
    pub fn vrvToolkit_getAvailableOptions(tkPtr: *mut c_void) -> *const c_char;

    /// Get the default values for all options.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// A JSON string with default option values.
    pub fn vrvToolkit_getDefaultOptions(tkPtr: *mut c_void) -> *const c_char;

    /// Get the current option values.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// A JSON string with current option values.
    pub fn vrvToolkit_getOptions(tkPtr: *mut c_void) -> *const c_char;

    /// Get a usage string for command-line options.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// A string describing command-line option usage.
    pub fn vrvToolkit_getOptionUsageString(tkPtr: *mut c_void) -> *const c_char;

    /// Set toolkit options from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `options` - JSON string containing option key-value pairs
    ///
    /// # Returns
    ///
    /// `true` if options were set successfully, `false` otherwise.
    pub fn vrvToolkit_setOptions(tkPtr: *mut c_void, options: *const c_char) -> bool;

    /// Reset all options to their default values.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    pub fn vrvToolkit_resetOptions(tkPtr: *mut c_void);

    // =========================================================================
    // Loading Functions
    // =========================================================================

    /// Load music data from a string.
    ///
    /// The data format is auto-detected (MEI, MusicXML, Humdrum, PAE, ABC).
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `data` - The music data as a null-terminated string
    ///
    /// # Returns
    ///
    /// `true` if the data was loaded successfully, `false` otherwise.
    pub fn vrvToolkit_loadData(tkPtr: *mut c_void, data: *const c_char) -> bool;

    /// Load music data from a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Path to the file to load
    ///
    /// # Returns
    ///
    /// `true` if the file was loaded successfully, `false` otherwise.
    pub fn vrvToolkit_loadFile(tkPtr: *mut c_void, filename: *const c_char) -> bool;

    /// Load compressed MusicXML data from a base64-encoded string.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `data` - Base64-encoded ZIP data
    ///
    /// # Returns
    ///
    /// `true` if the data was loaded successfully, `false` otherwise.
    pub fn vrvToolkit_loadZipDataBase64(tkPtr: *mut c_void, data: *const c_char) -> bool;

    /// Load compressed MusicXML data from a binary buffer.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `data` - Pointer to the ZIP data buffer
    /// * `length` - Length of the buffer in bytes
    ///
    /// # Returns
    ///
    /// `true` if the data was loaded successfully, `false` otherwise.
    pub fn vrvToolkit_loadZipDataBuffer(
        tkPtr: *mut c_void,
        data: *const c_uchar,
        length: c_int,
    ) -> bool;

    // =========================================================================
    // Input/Output Format Functions
    // =========================================================================

    /// Set the input format explicitly.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `inputFrom` - Input format string (e.g., "mei", "musicxml", "humdrum")
    ///
    /// # Returns
    ///
    /// `true` if the format was set successfully, `false` otherwise.
    pub fn vrvToolkit_setInputFrom(tkPtr: *mut c_void, inputFrom: *const c_char) -> bool;

    /// Set the output format.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `outputTo` - Output format string
    ///
    /// # Returns
    ///
    /// `true` if the format was set successfully, `false` otherwise.
    pub fn vrvToolkit_setOutputTo(tkPtr: *mut c_void, outputTo: *const c_char) -> bool;

    // =========================================================================
    // Rendering Functions
    // =========================================================================

    /// Render the loaded data and return output in one step.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `data` - Music data to render
    /// * `options` - JSON string with rendering options
    ///
    /// # Returns
    ///
    /// The rendered output (format depends on outputTo setting).
    pub fn vrvToolkit_renderData(
        tkPtr: *mut c_void,
        data: *const c_char,
        options: *const c_char,
    ) -> *const c_char;

    /// Render a page to SVG.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `page_no` - Page number to render (1-based)
    /// * `xmlDeclaration` - Whether to include the XML declaration
    ///
    /// # Returns
    ///
    /// SVG content as a string.
    pub fn vrvToolkit_renderToSVG(
        tkPtr: *mut c_void,
        page_no: c_int,
        xmlDeclaration: bool,
    ) -> *const c_char;

    /// Render a page to SVG and save to a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    /// * `pageNo` - Page number to render (1-based)
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_renderToSVGFile(
        tkPtr: *mut c_void,
        filename: *const c_char,
        pageNo: c_int,
    ) -> bool;

    /// Render the loaded document to MIDI (base64-encoded).
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// Base64-encoded MIDI data.
    pub fn vrvToolkit_renderToMIDI(tkPtr: *mut c_void) -> *const c_char;

    /// Render the loaded document to a MIDI file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_renderToMIDIFile(tkPtr: *mut c_void, filename: *const c_char) -> bool;

    /// Render the loaded document to Plaine & Easie code.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// PAE code as a string.
    pub fn vrvToolkit_renderToPAE(tkPtr: *mut c_void) -> *const c_char;

    /// Render the loaded document to a PAE file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_renderToPAEFile(tkPtr: *mut c_void, filename: *const c_char) -> bool;

    /// Render the expansion map for the document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// JSON string with the expansion map.
    pub fn vrvToolkit_renderToExpansionMap(tkPtr: *mut c_void) -> *const c_char;

    /// Render the expansion map to a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_renderToExpansionMapFile(tkPtr: *mut c_void, filename: *const c_char)
    -> bool;

    /// Render the timemap for the document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `c_options` - JSON options string
    ///
    /// # Returns
    ///
    /// JSON string with the timemap.
    pub fn vrvToolkit_renderToTimemap(
        tkPtr: *mut c_void,
        c_options: *const c_char,
    ) -> *const c_char;

    /// Render the timemap to a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    /// * `c_options` - JSON options string
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_renderToTimemapFile(
        tkPtr: *mut c_void,
        filename: *const c_char,
        c_options: *const c_char,
    ) -> bool;

    // =========================================================================
    // MEI Functions
    // =========================================================================

    /// Get the MEI representation of the loaded document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `options` - JSON options string (e.g., for page range selection)
    ///
    /// # Returns
    ///
    /// MEI XML content as a string.
    pub fn vrvToolkit_getMEI(tkPtr: *mut c_void, options: *const c_char) -> *const c_char;

    /// Save the document to a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    /// * `c_options` - JSON options string
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_saveFile(
        tkPtr: *mut c_void,
        filename: *const c_char,
        c_options: *const c_char,
    ) -> bool;

    // =========================================================================
    // Humdrum Functions
    // =========================================================================

    /// Get the Humdrum representation of the loaded document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// Humdrum content as a string.
    pub fn vrvToolkit_getHumdrum(tkPtr: *mut c_void) -> *const c_char;

    /// Save the Humdrum representation to a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Output file path
    ///
    /// # Returns
    ///
    /// `true` if the file was saved successfully, `false` otherwise.
    pub fn vrvToolkit_getHumdrumFile(tkPtr: *mut c_void, filename: *const c_char) -> bool;

    /// Convert Humdrum data to processed Humdrum.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `humdrumData` - Input Humdrum data
    ///
    /// # Returns
    ///
    /// Processed Humdrum content.
    pub fn vrvToolkit_convertHumdrumToHumdrum(
        tkPtr: *mut c_void,
        humdrumData: *const c_char,
    ) -> *const c_char;

    /// Convert Humdrum data to MIDI.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `humdrumData` - Input Humdrum data
    ///
    /// # Returns
    ///
    /// Base64-encoded MIDI data.
    pub fn vrvToolkit_convertHumdrumToMIDI(
        tkPtr: *mut c_void,
        humdrumData: *const c_char,
    ) -> *const c_char;

    /// Convert MEI data to Humdrum.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `meiData` - Input MEI data
    ///
    /// # Returns
    ///
    /// Humdrum content.
    pub fn vrvToolkit_convertMEIToHumdrum(
        tkPtr: *mut c_void,
        meiData: *const c_char,
    ) -> *const c_char;

    // =========================================================================
    // Page Information Functions
    // =========================================================================

    /// Get the total number of pages in the loaded document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The number of pages.
    pub fn vrvToolkit_getPageCount(tkPtr: *mut c_void) -> c_int;

    /// Get the page number containing a specific element.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The page number (1-based), or 0 if not found.
    pub fn vrvToolkit_getPageWithElement(tkPtr: *mut c_void, xmlId: *const c_char) -> c_int;

    // =========================================================================
    // Scale Functions
    // =========================================================================

    /// Get the current rendering scale.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The scale as a percentage (e.g., 100 for 100%).
    pub fn vrvToolkit_getScale(tkPtr: *mut c_void) -> c_int;

    /// Set the rendering scale.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `scale` - The scale as a percentage
    ///
    /// # Returns
    ///
    /// `true` if the scale was set successfully, `false` otherwise.
    pub fn vrvToolkit_setScale(tkPtr: *mut c_void, scale: c_int) -> bool;

    // =========================================================================
    // Element Query Functions
    // =========================================================================

    /// Get attributes of an element by its xml:id.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// JSON string with element attributes.
    pub fn vrvToolkit_getElementAttr(tkPtr: *mut c_void, xmlId: *const c_char) -> *const c_char;

    /// Get elements at a specific time in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `millisec` - Time in milliseconds
    ///
    /// # Returns
    ///
    /// JSON string with element IDs at the specified time.
    pub fn vrvToolkit_getElementsAtTime(tkPtr: *mut c_void, millisec: c_int) -> *const c_char;

    /// Get expansion IDs for an element.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// JSON string with expansion IDs.
    pub fn vrvToolkit_getExpansionIdsForElement(
        tkPtr: *mut c_void,
        xmlId: *const c_char,
    ) -> *const c_char;

    /// Get MIDI values for an element.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// JSON string with MIDI values.
    pub fn vrvToolkit_getMIDIValuesForElement(
        tkPtr: *mut c_void,
        xmlId: *const c_char,
    ) -> *const c_char;

    /// Get the notated ID for an element.
    ///
    /// This is useful when working with expansions where elements may have
    /// different rendered IDs than their notated IDs.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The notated ID as a string.
    pub fn vrvToolkit_getNotatedIdForElement(
        tkPtr: *mut c_void,
        xmlId: *const c_char,
    ) -> *const c_char;

    /// Get the time (in milliseconds) for an element.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The time in milliseconds.
    pub fn vrvToolkit_getTimeForElement(tkPtr: *mut c_void, xmlId: *const c_char) -> c_double;

    /// Get timing information for an element.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `xmlId` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// JSON string with timing information.
    pub fn vrvToolkit_getTimesForElement(tkPtr: *mut c_void, xmlId: *const c_char)
    -> *const c_char;

    // =========================================================================
    // Feature Extraction
    // =========================================================================

    /// Get descriptive features from the loaded document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `options` - JSON options string
    ///
    /// # Returns
    ///
    /// JSON string with descriptive features.
    pub fn vrvToolkit_getDescriptiveFeatures(
        tkPtr: *mut c_void,
        options: *const c_char,
    ) -> *const c_char;

    // =========================================================================
    // Resource Path Functions
    // =========================================================================

    /// Get the current resource path.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The resource path as a string.
    pub fn vrvToolkit_getResourcePath(tkPtr: *mut c_void) -> *const c_char;

    /// Set the resource path.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `path` - The new resource path
    ///
    /// # Returns
    ///
    /// `true` if the path was set successfully, `false` otherwise.
    pub fn vrvToolkit_setResourcePath(tkPtr: *mut c_void, path: *const c_char) -> bool;

    // =========================================================================
    // Layout Functions
    // =========================================================================

    /// Redo the layout with optional new options.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `c_options` - JSON options string (can be empty)
    pub fn vrvToolkit_redoLayout(tkPtr: *mut c_void, c_options: *const c_char);

    /// Redo the pitch position layout for the current page.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    pub fn vrvToolkit_redoPagePitchPosLayout(tkPtr: *mut c_void);

    // =========================================================================
    // Selection Functions
    // =========================================================================

    /// Select elements in the document.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `selection` - JSON string describing the selection
    ///
    /// # Returns
    ///
    /// `true` if the selection was applied successfully, `false` otherwise.
    pub fn vrvToolkit_select(tkPtr: *mut c_void, selection: *const c_char) -> bool;

    // =========================================================================
    // Utility Functions
    // =========================================================================

    /// Get the Verovio version string.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The version string.
    pub fn vrvToolkit_getVersion(tkPtr: *mut c_void) -> *const c_char;

    /// Get the toolkit instance ID.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The instance ID string.
    pub fn vrvToolkit_getID(tkPtr: *mut c_void) -> *const c_char;

    /// Get the log buffer contents.
    ///
    /// Requires `enableLogToBuffer(true)` to have been called.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    ///
    /// # Returns
    ///
    /// The log contents as a string.
    pub fn vrvToolkit_getLog(tkPtr: *mut c_void) -> *const c_char;

    /// Reset the XML ID seed.
    ///
    /// This affects how new xml:id values are generated.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `seed` - The new seed value
    pub fn vrvToolkit_resetXmlIdSeed(tkPtr: *mut c_void, seed: c_int);

    // =========================================================================
    // PAE Validation Functions
    // =========================================================================

    /// Validate Plaine & Easie code.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `data` - PAE code to validate
    ///
    /// # Returns
    ///
    /// JSON string with validation results.
    pub fn vrvToolkit_validatePAE(tkPtr: *mut c_void, data: *const c_char) -> *const c_char;

    /// Validate Plaine & Easie code from a file.
    ///
    /// # Arguments
    ///
    /// * `tkPtr` - Pointer to the toolkit instance
    /// * `filename` - Path to the PAE file
    ///
    /// # Returns
    ///
    /// JSON string with validation results.
    pub fn vrvToolkit_validatePAEFile(tkPtr: *mut c_void, filename: *const c_char)
    -> *const c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_constructor_destructor() {
        unsafe {
            let toolkit = vrvToolkit_constructorNoResource();
            assert!(!toolkit.is_null());
            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_get_version() {
        unsafe {
            let toolkit = vrvToolkit_constructorNoResource();
            assert!(!toolkit.is_null());

            let version_ptr = vrvToolkit_getVersion(toolkit);
            assert!(!version_ptr.is_null());

            let version = CStr::from_ptr(version_ptr).to_string_lossy();
            // Version should be non-empty and look like a version number
            assert!(!version.is_empty());

            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_page_count_empty() {
        unsafe {
            let toolkit = vrvToolkit_constructorNoResource();
            assert!(!toolkit.is_null());

            // No document loaded, so page count should be 0
            let page_count = vrvToolkit_getPageCount(toolkit);
            assert_eq!(page_count, 0);

            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_get_options() {
        unsafe {
            let toolkit = vrvToolkit_constructorNoResource();
            assert!(!toolkit.is_null());

            let options_ptr = vrvToolkit_getOptions(toolkit);
            assert!(!options_ptr.is_null());

            let options = CStr::from_ptr(options_ptr).to_string_lossy();
            // Options should be a JSON object (may have trailing whitespace)
            let trimmed = options.trim();
            assert!(trimmed.starts_with('{'));
            assert!(trimmed.ends_with('}'));

            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_logging() {
        unsafe {
            // Enable logging to buffer
            enableLogToBuffer(true);
            enableLog(true);

            let toolkit = vrvToolkit_constructorNoResource();
            assert!(!toolkit.is_null());

            // Get log - should work without crashing
            let log_ptr = vrvToolkit_getLog(toolkit);
            assert!(!log_ptr.is_null());

            // Disable logging
            enableLog(false);
            enableLogToBuffer(false);

            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_is_valid_toolkit() {
        use crate::bindings::{NULL_TOOLKIT, ToolkitPtr, is_valid_toolkit};

        // Null pointer should be invalid
        assert!(!is_valid_toolkit(NULL_TOOLKIT));

        // Non-null pointer should be valid
        unsafe {
            let toolkit: ToolkitPtr = vrvToolkit_constructorNoResource();
            assert!(is_valid_toolkit(toolkit));
            vrvToolkit_destructor(toolkit);
        }
    }

    #[test]
    fn test_null_toolkit_constant() {
        use crate::bindings::NULL_TOOLKIT;

        assert!(NULL_TOOLKIT.is_null());
    }

    #[test]
    fn test_toolkit_ptr_type() {
        use crate::bindings::ToolkitPtr;

        // Verify ToolkitPtr is the correct type
        let ptr: ToolkitPtr = std::ptr::null_mut();
        assert!(ptr.is_null());
    }
}
