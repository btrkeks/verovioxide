//! Rendering options for the Verovio toolkit.
//!
//! This module provides a builder-style API for configuring Verovio rendering options.
//! Options are serialized to JSON and passed to the underlying Verovio toolkit.
//!
//! # Example
//!
//! ```
//! use verovioxide::Options;
//!
//! let options = Options::builder()
//!     .scale(80)
//!     .page_width(2100)
//!     .page_height(2970)
//!     .adjust_page_height(true)
//!     .build();
//! ```
//!
//! # Performance
//!
//! Building and serializing options is lightweight. However, when options are applied
//! to a toolkit with [`Toolkit::set_options`](crate::Toolkit::set_options), certain
//! options may trigger a re-layout of any loaded document:
//!
//! - **Layout-affecting options**: `page_width`, `page_height`, `page_margin*`,
//!   `breaks`, `spacing_*`, `scale`, `adjust_page_height`
//! - **Rendering-only options**: `font`, `svg_*`, `midi_*` - these don't require
//!   re-layout
//!
//! For best performance, set layout-affecting options before loading data, or batch
//! option changes together to minimize layout recalculations.

use serde::{Deserialize, Serialize};

/// Break mode for page and system breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakMode {
    /// Automatic break placement.
    #[default]
    Auto,
    /// No automatic breaks.
    None,
    /// Use encoded breaks from the input file.
    Encoded,
    /// Break at each line.
    Line,
    /// Smart break placement.
    Smart,
}

/// Condense mode for dense layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CondenseMode {
    /// No condensing.
    #[default]
    None,
    /// Automatic condensing.
    Auto,
    /// Use encoded condensing.
    Encoded,
}

/// Footer display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FooterMode {
    /// No footer.
    #[default]
    None,
    /// Automatic footer.
    Auto,
    /// Use encoded footer.
    Encoded,
    /// Always show footer.
    Always,
}

/// Header display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaderMode {
    /// No header.
    #[default]
    None,
    /// Automatic header.
    Auto,
    /// Use encoded header.
    Encoded,
}

/// SMuFL text font to use for text rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextFont {
    /// Times font (default).
    #[default]
    Times,
    /// Custom font name.
    Custom(String),
}

impl TextFont {
    /// Returns the font name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Times => "Times",
            Self::Custom(name) => name,
        }
    }
}

/// Rendering options for the Verovio toolkit.
///
/// This struct provides a type-safe way to configure Verovio rendering options.
/// Use the builder pattern via [`Options::builder()`] to construct options.
///
/// All fields use `Option` to allow partial configuration. When serialized to JSON,
/// only set fields are included, letting Verovio use its defaults for unset values.
///
/// # See also
///
/// - [`OptionsBuilder`] - Builder for constructing options
/// - [`Toolkit::set_options`](crate::Toolkit::set_options) - Apply options to a toolkit
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    // =========================================================================
    // General Options
    // =========================================================================
    /// Rendering scale as a percentage (e.g., 100 for 100%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,

    /// Page width in MEI units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_width: Option<u32>,

    /// Page height in MEI units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_height: Option<u32>,

    /// Whether to adjust the page height to the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_page_height: Option<bool>,

    /// Whether to adjust the page width to the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_page_width: Option<bool>,

    /// Top page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_top: Option<u32>,

    /// Bottom page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_bottom: Option<u32>,

    /// Left page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_left: Option<u32>,

    /// Right page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_right: Option<u32>,

    // =========================================================================
    // Font Options
    // =========================================================================
    /// The SMuFL music font to use (e.g., "Leipzig", "Bravura").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,

    /// Lyric size as a percentage of the staff size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyric_size: Option<f64>,

    // =========================================================================
    // Layout Options
    // =========================================================================
    /// Break mode for page and system breaks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaks: Option<BreakMode>,

    /// Maximum number of systems placed on one page (`0` keeps Verovio's default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_max_per_page: Option<u32>,

    /// Whether grace-note groups align to preceding notes across all staves.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace_right_align: Option<bool>,

    /// Condense mode for dense layouts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense: Option<CondenseMode>,

    /// Whether to condense the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense_first_page: Option<bool>,

    /// Minimum width for condensed scores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense_tempo_pages: Option<bool>,

    /// Whether to even note spacing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub even_note_spacing: Option<bool>,

    /// The minimum measure width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_measure_width: Option<u32>,

    /// How frequently to place measure numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnum_interval: Option<u32>,

    /// Header display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HeaderMode>,

    /// Footer display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FooterMode>,

    // =========================================================================
    // SVG Output Options
    // =========================================================================
    /// Whether to include the XML declaration in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_xml_declaration: Option<bool>,

    /// Whether to include bounding boxes in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_bounding_boxes: Option<bool>,

    /// Whether to use viewBox attribute in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_view_box: Option<bool>,

    /// Whether to remove xlink namespace from SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_remove_xlink: Option<bool>,

    /// CSS stylesheet to embed in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_css: Option<String>,

    /// Whether to format SVG output with indentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_format_raw: Option<bool>,

    /// Whether to include font fallback in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_font_face_include: Option<bool>,

    // =========================================================================
    // MIDI Options
    // =========================================================================
    /// Default MIDI tempo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_tempo: Option<f64>,

    /// MIDI velocity for notes without dynamics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_velocity: Option<u8>,

    // =========================================================================
    // Input Options
    // =========================================================================
    /// Input format (auto, mei, musicxml, musicxml-compressed, humdrum, pae, abc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_from: Option<String>,

    // =========================================================================
    // Selection Options
    // =========================================================================
    /// Starting measure for selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdiv_x_path_query: Option<String>,

    /// Expansion to use from the MEI document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expansion: Option<String>,

    // =========================================================================
    // Transposition Options
    // =========================================================================
    /// Transposition interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose: Option<String>,

    /// Whether to transpose the written/sounding selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose_selected_only: Option<bool>,

    /// Whether to transpose to written or sounding pitch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose_to_sounding_pitch: Option<bool>,

    // =========================================================================
    // Spacing Options
    // =========================================================================
    /// Spacing between staff lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_staff: Option<u32>,

    /// Spacing between systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_system: Option<u32>,

    /// Linear spacing factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_linear: Option<f64>,

    /// Non-linear spacing factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_non_linear: Option<f64>,
}

impl Options {
    /// Creates a new builder for constructing options.
    ///
    /// # See also
    ///
    /// - [`OptionsBuilder`] - The builder type
    /// - [`OptionsBuilder::build`] - Finalize the builder into [`Options`]
    #[must_use]
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// Serializes the options to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes options from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Builder for constructing [`Options`].
///
/// Use [`Options::builder()`] to create a new builder.
///
/// # See also
///
/// - [`Options`] - The options type this builds
/// - [`Options::builder`] - Create a new builder
#[derive(Debug, Default, Clone)]
pub struct OptionsBuilder {
    options: Options,
}

impl OptionsBuilder {
    /// Sets the rendering scale as a percentage.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale percentage (e.g., 100 for 100%)
    #[must_use]
    pub fn scale(mut self, scale: u32) -> Self {
        self.options.scale = Some(scale);
        self
    }

    /// Sets the page width in MEI units.
    #[must_use]
    pub fn page_width(mut self, width: u32) -> Self {
        self.options.page_width = Some(width);
        self
    }

    /// Sets the page height in MEI units.
    #[must_use]
    pub fn page_height(mut self, height: u32) -> Self {
        self.options.page_height = Some(height);
        self
    }

    /// Sets whether to adjust the page height to the content.
    #[must_use]
    pub fn adjust_page_height(mut self, adjust: bool) -> Self {
        self.options.adjust_page_height = Some(adjust);
        self
    }

    /// Sets whether to adjust the page width to the content.
    #[must_use]
    pub fn adjust_page_width(mut self, adjust: bool) -> Self {
        self.options.adjust_page_width = Some(adjust);
        self
    }

    /// Sets the page margin for all sides in MEI units.
    ///
    /// # See also
    ///
    /// - [`page_margin_top`](Self::page_margin_top), [`page_margin_bottom`](Self::page_margin_bottom),
    ///   [`page_margin_left`](Self::page_margin_left), [`page_margin_right`](Self::page_margin_right) -
    ///   Set individual margins
    #[must_use]
    pub fn page_margin(mut self, margin: u32) -> Self {
        self.options.page_margin_top = Some(margin);
        self.options.page_margin_bottom = Some(margin);
        self.options.page_margin_left = Some(margin);
        self.options.page_margin_right = Some(margin);
        self
    }

    /// Sets the top page margin in MEI units.
    ///
    /// # See also
    ///
    /// - [`page_margin`](Self::page_margin) - Set all margins at once
    #[must_use]
    pub fn page_margin_top(mut self, margin: u32) -> Self {
        self.options.page_margin_top = Some(margin);
        self
    }

    /// Sets the bottom page margin in MEI units.
    ///
    /// # See also
    ///
    /// - [`page_margin`](Self::page_margin) - Set all margins at once
    #[must_use]
    pub fn page_margin_bottom(mut self, margin: u32) -> Self {
        self.options.page_margin_bottom = Some(margin);
        self
    }

    /// Sets the left page margin in MEI units.
    ///
    /// # See also
    ///
    /// - [`page_margin`](Self::page_margin) - Set all margins at once
    #[must_use]
    pub fn page_margin_left(mut self, margin: u32) -> Self {
        self.options.page_margin_left = Some(margin);
        self
    }

    /// Sets the right page margin in MEI units.
    ///
    /// # See also
    ///
    /// - [`page_margin`](Self::page_margin) - Set all margins at once
    #[must_use]
    pub fn page_margin_right(mut self, margin: u32) -> Self {
        self.options.page_margin_right = Some(margin);
        self
    }

    /// Sets the SMuFL music font to use.
    #[must_use]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.options.font = Some(font.into());
        self
    }

    /// Sets the lyric size as a percentage of staff size.
    #[must_use]
    pub fn lyric_size(mut self, size: f64) -> Self {
        self.options.lyric_size = Some(size);
        self
    }

    /// Sets the break mode for page and system breaks.
    #[must_use]
    pub fn breaks(mut self, mode: BreakMode) -> Self {
        self.options.breaks = Some(mode);
        self
    }

    /// Sets the maximum number of systems placed on one page.
    #[must_use]
    pub fn system_max_per_page(mut self, maximum: u32) -> Self {
        self.options.system_max_per_page = Some(maximum);
        self
    }

    /// Sets whether grace-note groups align to preceding notes across all staves.
    #[must_use]
    pub fn grace_right_align(mut self, align: bool) -> Self {
        self.options.grace_right_align = Some(align);
        self
    }

    /// Sets the condense mode for dense layouts.
    #[must_use]
    pub fn condense(mut self, mode: CondenseMode) -> Self {
        self.options.condense = Some(mode);
        self
    }

    /// Sets whether to condense the first page.
    #[must_use]
    pub fn condense_first_page(mut self, condense: bool) -> Self {
        self.options.condense_first_page = Some(condense);
        self
    }

    /// Sets whether to condense tempo pages.
    #[must_use]
    pub fn condense_tempo_pages(mut self, condense: bool) -> Self {
        self.options.condense_tempo_pages = Some(condense);
        self
    }

    /// Sets whether to use even note spacing.
    #[must_use]
    pub fn even_note_spacing(mut self, even: bool) -> Self {
        self.options.even_note_spacing = Some(even);
        self
    }

    /// Sets the minimum measure width.
    #[must_use]
    pub fn min_measure_width(mut self, width: u32) -> Self {
        self.options.min_measure_width = Some(width);
        self
    }

    /// Sets how frequently to place measure numbers.
    #[must_use]
    pub fn mnum_interval(mut self, interval: u32) -> Self {
        self.options.mnum_interval = Some(interval);
        self
    }

    /// Sets the header display mode.
    #[must_use]
    pub fn header(mut self, mode: HeaderMode) -> Self {
        self.options.header = Some(mode);
        self
    }

    /// Sets the footer display mode.
    #[must_use]
    pub fn footer(mut self, mode: FooterMode) -> Self {
        self.options.footer = Some(mode);
        self
    }

    /// Sets whether to include the XML declaration in SVG output.
    #[must_use]
    pub fn svg_xml_declaration(mut self, include: bool) -> Self {
        self.options.svg_xml_declaration = Some(include);
        self
    }

    /// Sets whether to include bounding boxes in SVG output.
    #[must_use]
    pub fn svg_bounding_boxes(mut self, include: bool) -> Self {
        self.options.svg_bounding_boxes = Some(include);
        self
    }

    /// Sets whether to use viewBox attribute in SVG output.
    #[must_use]
    pub fn svg_view_box(mut self, use_view_box: bool) -> Self {
        self.options.svg_view_box = Some(use_view_box);
        self
    }

    /// Sets whether to remove xlink namespace from SVG output.
    #[must_use]
    pub fn svg_remove_xlink(mut self, remove: bool) -> Self {
        self.options.svg_remove_xlink = Some(remove);
        self
    }

    /// Sets the CSS stylesheet to embed in SVG output.
    #[must_use]
    pub fn svg_css(mut self, css: impl Into<String>) -> Self {
        self.options.svg_css = Some(css.into());
        self
    }

    /// Sets whether to format SVG output with indentation.
    #[must_use]
    pub fn svg_format_raw(mut self, raw: bool) -> Self {
        self.options.svg_format_raw = Some(raw);
        self
    }

    /// Sets whether to include font fallback in SVG output.
    #[must_use]
    pub fn svg_font_face_include(mut self, include: bool) -> Self {
        self.options.svg_font_face_include = Some(include);
        self
    }

    /// Sets the default MIDI tempo.
    #[must_use]
    pub fn midi_tempo(mut self, tempo: f64) -> Self {
        self.options.midi_tempo = Some(tempo);
        self
    }

    /// Sets the MIDI velocity for notes without dynamics.
    #[must_use]
    pub fn midi_velocity(mut self, velocity: u8) -> Self {
        self.options.midi_velocity = Some(velocity);
        self
    }

    /// Sets the input format.
    #[must_use]
    pub fn input_from(mut self, format: impl Into<String>) -> Self {
        self.options.input_from = Some(format.into());
        self
    }

    /// Sets the mdiv XPath query.
    #[must_use]
    pub fn mdiv_x_path_query(mut self, query: impl Into<String>) -> Self {
        self.options.mdiv_x_path_query = Some(query.into());
        self
    }

    /// Sets the expansion to use from the MEI document.
    #[must_use]
    pub fn expansion(mut self, expansion: impl Into<String>) -> Self {
        self.options.expansion = Some(expansion.into());
        self
    }

    /// Sets the transposition interval.
    #[must_use]
    pub fn transpose(mut self, interval: impl Into<String>) -> Self {
        self.options.transpose = Some(interval.into());
        self
    }

    /// Sets whether to transpose only the selection.
    #[must_use]
    pub fn transpose_selected_only(mut self, selected: bool) -> Self {
        self.options.transpose_selected_only = Some(selected);
        self
    }

    /// Sets whether to transpose to sounding pitch.
    #[must_use]
    pub fn transpose_to_sounding_pitch(mut self, sounding: bool) -> Self {
        self.options.transpose_to_sounding_pitch = Some(sounding);
        self
    }

    /// Sets the spacing between staff lines.
    #[must_use]
    pub fn spacing_staff(mut self, spacing: u32) -> Self {
        self.options.spacing_staff = Some(spacing);
        self
    }

    /// Sets the spacing between systems.
    #[must_use]
    pub fn spacing_system(mut self, spacing: u32) -> Self {
        self.options.spacing_system = Some(spacing);
        self
    }

    /// Sets the linear spacing factor.
    #[must_use]
    pub fn spacing_linear(mut self, factor: f64) -> Self {
        self.options.spacing_linear = Some(factor);
        self
    }

    /// Sets the non-linear spacing factor.
    #[must_use]
    pub fn spacing_non_linear(mut self, factor: f64) -> Self {
        self.options.spacing_non_linear = Some(factor);
        self
    }

    /// Builds the options.
    ///
    /// # See also
    ///
    /// - [`Options`] - The resulting options type
    /// - [`Toolkit::set_options`](crate::Toolkit::set_options) - Apply options to a toolkit
    #[must_use]
    pub fn build(self) -> Options {
        self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_builder_scale() {
        let options = Options::builder().scale(80).build();
        assert_eq!(options.scale, Some(80));
    }

    #[test]
    fn test_options_builder_page_dimensions() {
        let options = Options::builder()
            .page_width(2100)
            .page_height(2970)
            .build();
        assert_eq!(options.page_width, Some(2100));
        assert_eq!(options.page_height, Some(2970));
    }

    #[test]
    fn test_options_builder_adjust_page_height() {
        let options = Options::builder().adjust_page_height(true).build();
        assert_eq!(options.adjust_page_height, Some(true));
    }

    #[test]
    fn test_options_builder_adjust_page_width() {
        let options = Options::builder().adjust_page_width(true).build();
        assert_eq!(options.adjust_page_width, Some(true));
    }

    #[test]
    fn test_options_builder_font() {
        let options = Options::builder().font("Bravura").build();
        assert_eq!(options.font, Some("Bravura".to_string()));
    }

    #[test]
    fn test_options_builder_breaks() {
        let options = Options::builder().breaks(BreakMode::Encoded).build();
        assert_eq!(options.breaks, Some(BreakMode::Encoded));
    }

    #[test]
    fn test_options_builder_system_max_per_page() {
        let options = Options::builder().system_max_per_page(1).build();
        assert_eq!(options.system_max_per_page, Some(1));
        assert!(
            options
                .to_json()
                .unwrap()
                .contains(r#""systemMaxPerPage":1"#)
        );
    }

    #[test]
    fn test_options_builder_grace_right_align() {
        let options = Options::builder().grace_right_align(true).build();
        assert_eq!(options.grace_right_align, Some(true));
        assert!(options.to_json().unwrap().contains(r#""graceRightAlign":true"#));
    }

    #[test]
    fn test_options_builder_header_footer() {
        let options = Options::builder()
            .header(HeaderMode::Auto)
            .footer(FooterMode::Always)
            .build();
        assert_eq!(options.header, Some(HeaderMode::Auto));
        assert_eq!(options.footer, Some(FooterMode::Always));
    }

    #[test]
    fn test_options_builder_chaining() {
        let options = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .adjust_page_height(true)
            .adjust_page_width(true)
            .font("Leipzig")
            .breaks(BreakMode::Auto)
            .build();

        assert_eq!(options.scale, Some(80));
        assert_eq!(options.page_width, Some(2100));
        assert_eq!(options.page_height, Some(2970));
        assert_eq!(options.adjust_page_height, Some(true));
        assert_eq!(options.adjust_page_width, Some(true));
        assert_eq!(options.font, Some("Leipzig".to_string()));
        assert_eq!(options.breaks, Some(BreakMode::Auto));
    }

    #[test]
    fn test_options_to_json() {
        let options = Options::builder().scale(80).page_width(2100).build();
        let json = options.to_json().unwrap();
        assert!(json.contains("\"scale\":80"));
        assert!(json.contains("\"pageWidth\":2100"));
    }

    #[test]
    fn test_options_adjust_page_width_json() {
        let options = Options::builder().adjust_page_width(true).build();
        let json = options.to_json().unwrap();
        assert!(json.contains("\"adjustPageWidth\":true"));

        let options = Options::from_json(r#"{"adjustPageWidth":true}"#).unwrap();
        assert_eq!(options.adjust_page_width, Some(true));
    }

    #[test]
    fn test_options_from_json() {
        let json = r#"{"scale":80,"pageWidth":2100}"#;
        let options = Options::from_json(json).unwrap();
        assert_eq!(options.scale, Some(80));
        assert_eq!(options.page_width, Some(2100));
    }

    #[test]
    fn test_options_skip_none_in_json() {
        let options = Options::builder().scale(80).build();
        let json = options.to_json().unwrap();
        assert!(json.contains("scale"));
        assert!(!json.contains("pageWidth"));
    }

    #[test]
    fn test_default_options_empty() {
        let options = Options::default();
        assert!(options.scale.is_none());
        assert!(options.page_width.is_none());
        assert!(options.page_height.is_none());
    }

    #[test]
    fn test_break_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&BreakMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::Encoded).unwrap(),
            r#""encoded""#
        );
    }

    #[test]
    fn test_break_mode_deserialize() {
        let mode: BreakMode = serde_json::from_str(r#""smart""#).unwrap();
        assert_eq!(mode, BreakMode::Smart);
    }

    #[test]
    fn test_header_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&HeaderMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&HeaderMode::Auto).unwrap(),
            r#""auto""#
        );
    }

    #[test]
    fn test_footer_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&FooterMode::Always).unwrap(),
            r#""always""#
        );
    }

    #[test]
    fn test_condense_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&CondenseMode::Auto).unwrap(),
            r#""auto""#
        );
    }

    #[test]
    fn test_text_font_default() {
        let font = TextFont::default();
        assert_eq!(font.as_str(), "Times");
    }

    #[test]
    fn test_text_font_custom() {
        let font = TextFont::Custom("Arial".to_string());
        assert_eq!(font.as_str(), "Arial");
    }

    #[test]
    fn test_options_builder_margins() {
        // Test that page_margin sets all four margins
        let options = Options::builder().page_margin(50).build();
        assert_eq!(options.page_margin_top, Some(50));
        assert_eq!(options.page_margin_bottom, Some(50));
        assert_eq!(options.page_margin_left, Some(50));
        assert_eq!(options.page_margin_right, Some(50));

        // Test that individual margins can override
        let options = Options::builder()
            .page_margin(50)
            .page_margin_top(100)
            .page_margin_bottom(100)
            .page_margin_left(75)
            .page_margin_right(75)
            .build();

        assert_eq!(options.page_margin_top, Some(100));
        assert_eq!(options.page_margin_bottom, Some(100));
        assert_eq!(options.page_margin_left, Some(75));
        assert_eq!(options.page_margin_right, Some(75));
    }

    #[test]
    fn test_options_builder_svg_options() {
        let options = Options::builder()
            .svg_xml_declaration(false)
            .svg_bounding_boxes(true)
            .svg_view_box(true)
            .svg_remove_xlink(true)
            .svg_css("svg { background: white; }")
            .svg_format_raw(false)
            .svg_font_face_include(true)
            .build();

        assert_eq!(options.svg_xml_declaration, Some(false));
        assert_eq!(options.svg_bounding_boxes, Some(true));
        assert_eq!(options.svg_view_box, Some(true));
        assert_eq!(options.svg_remove_xlink, Some(true));
        assert_eq!(
            options.svg_css,
            Some("svg { background: white; }".to_string())
        );
        assert_eq!(options.svg_format_raw, Some(false));
        assert_eq!(options.svg_font_face_include, Some(true));
    }

    #[test]
    fn test_options_builder_midi_options() {
        let options = Options::builder()
            .midi_tempo(120.0)
            .midi_velocity(80)
            .build();

        assert_eq!(options.midi_tempo, Some(120.0));
        assert_eq!(options.midi_velocity, Some(80));
    }

    #[test]
    fn test_options_builder_spacing_options() {
        let options = Options::builder()
            .spacing_staff(12)
            .spacing_system(6)
            .spacing_linear(0.25)
            .spacing_non_linear(0.6)
            .build();

        assert_eq!(options.spacing_staff, Some(12));
        assert_eq!(options.spacing_system, Some(6));
        assert_eq!(options.spacing_linear, Some(0.25));
        assert_eq!(options.spacing_non_linear, Some(0.6));
    }

    #[test]
    fn test_options_builder_transposition() {
        let options = Options::builder()
            .transpose("M2")
            .transpose_selected_only(true)
            .transpose_to_sounding_pitch(false)
            .build();

        assert_eq!(options.transpose, Some("M2".to_string()));
        assert_eq!(options.transpose_selected_only, Some(true));
        assert_eq!(options.transpose_to_sounding_pitch, Some(false));
    }

    #[test]
    fn test_options_round_trip_json() {
        let original = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .font("Leipzig")
            .breaks(BreakMode::Smart)
            .build();

        let json = original.to_json().unwrap();
        let parsed = Options::from_json(&json).unwrap();

        assert_eq!(original.scale, parsed.scale);
        assert_eq!(original.page_width, parsed.page_width);
        assert_eq!(original.page_height, parsed.page_height);
        assert_eq!(original.font, parsed.font);
        assert_eq!(original.breaks, parsed.breaks);
    }

    // =========================================================================
    // Additional tests for improved coverage
    // =========================================================================

    #[test]
    fn test_options_from_json_invalid() {
        let result = Options::from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_options_from_json_empty_object() {
        let result = Options::from_json("{}");
        assert!(result.is_ok());
        let options = result.unwrap();
        assert!(options.scale.is_none());
        assert!(options.page_width.is_none());
    }

    #[test]
    fn test_options_from_json_unknown_fields() {
        // serde should ignore unknown fields by default
        let result = Options::from_json(r#"{"unknownField": "value", "scale": 100}"#);
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.scale, Some(100));
    }

    #[test]
    fn test_options_from_json_wrong_type() {
        let result = Options::from_json(r#"{"scale": "not a number"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_break_mode_serialize_all_variants() {
        assert_eq!(
            serde_json::to_string(&BreakMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::Encoded).unwrap(),
            r#""encoded""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::Line).unwrap(),
            r#""line""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::Smart).unwrap(),
            r#""smart""#
        );
    }

    #[test]
    fn test_break_mode_deserialize_all_variants() {
        let auto: BreakMode = serde_json::from_str(r#""auto""#).unwrap();
        assert_eq!(auto, BreakMode::Auto);

        let none: BreakMode = serde_json::from_str(r#""none""#).unwrap();
        assert_eq!(none, BreakMode::None);

        let encoded: BreakMode = serde_json::from_str(r#""encoded""#).unwrap();
        assert_eq!(encoded, BreakMode::Encoded);

        let line: BreakMode = serde_json::from_str(r#""line""#).unwrap();
        assert_eq!(line, BreakMode::Line);

        let smart: BreakMode = serde_json::from_str(r#""smart""#).unwrap();
        assert_eq!(smart, BreakMode::Smart);
    }

    #[test]
    fn test_break_mode_default() {
        let mode = BreakMode::default();
        assert_eq!(mode, BreakMode::Auto);
    }

    #[test]
    fn test_condense_mode_serialize_all_variants() {
        assert_eq!(
            serde_json::to_string(&CondenseMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&CondenseMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&CondenseMode::Encoded).unwrap(),
            r#""encoded""#
        );
    }

    #[test]
    fn test_condense_mode_deserialize_all_variants() {
        let none: CondenseMode = serde_json::from_str(r#""none""#).unwrap();
        assert_eq!(none, CondenseMode::None);

        let auto: CondenseMode = serde_json::from_str(r#""auto""#).unwrap();
        assert_eq!(auto, CondenseMode::Auto);

        let encoded: CondenseMode = serde_json::from_str(r#""encoded""#).unwrap();
        assert_eq!(encoded, CondenseMode::Encoded);
    }

    #[test]
    fn test_condense_mode_default() {
        let mode = CondenseMode::default();
        assert_eq!(mode, CondenseMode::None);
    }

    #[test]
    fn test_footer_mode_serialize_all_variants() {
        assert_eq!(
            serde_json::to_string(&FooterMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&FooterMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&FooterMode::Encoded).unwrap(),
            r#""encoded""#
        );
        assert_eq!(
            serde_json::to_string(&FooterMode::Always).unwrap(),
            r#""always""#
        );
    }

    #[test]
    fn test_footer_mode_deserialize_all_variants() {
        let none: FooterMode = serde_json::from_str(r#""none""#).unwrap();
        assert_eq!(none, FooterMode::None);

        let auto: FooterMode = serde_json::from_str(r#""auto""#).unwrap();
        assert_eq!(auto, FooterMode::Auto);

        let encoded: FooterMode = serde_json::from_str(r#""encoded""#).unwrap();
        assert_eq!(encoded, FooterMode::Encoded);

        let always: FooterMode = serde_json::from_str(r#""always""#).unwrap();
        assert_eq!(always, FooterMode::Always);
    }

    #[test]
    fn test_footer_mode_default() {
        let mode = FooterMode::default();
        assert_eq!(mode, FooterMode::None);
    }

    #[test]
    fn test_header_mode_serialize_all_variants() {
        assert_eq!(
            serde_json::to_string(&HeaderMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&HeaderMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&HeaderMode::Encoded).unwrap(),
            r#""encoded""#
        );
    }

    #[test]
    fn test_header_mode_deserialize_all_variants() {
        let none: HeaderMode = serde_json::from_str(r#""none""#).unwrap();
        assert_eq!(none, HeaderMode::None);

        let auto: HeaderMode = serde_json::from_str(r#""auto""#).unwrap();
        assert_eq!(auto, HeaderMode::Auto);

        let encoded: HeaderMode = serde_json::from_str(r#""encoded""#).unwrap();
        assert_eq!(encoded, HeaderMode::Encoded);
    }

    #[test]
    fn test_header_mode_default() {
        let mode = HeaderMode::default();
        assert_eq!(mode, HeaderMode::None);
    }

    #[test]
    fn test_text_font_times() {
        let font = TextFont::Times;
        assert_eq!(font.as_str(), "Times");
    }

    #[test]
    fn test_text_font_custom_empty() {
        let font = TextFont::Custom(String::new());
        assert_eq!(font.as_str(), "");
    }

    #[test]
    fn test_text_font_serialize() {
        let times = TextFont::Times;
        let json = serde_json::to_string(&times).unwrap();
        assert_eq!(json, r#""Times""#);

        let custom = TextFont::Custom("Arial".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("Arial"));
    }

    #[test]
    fn test_text_font_deserialize() {
        let times: TextFont = serde_json::from_str(r#""Times""#).unwrap();
        assert_eq!(times, TextFont::Times);
    }

    #[test]
    fn test_text_font_clone() {
        let font = TextFont::Custom("MyFont".to_string());
        let cloned = font.clone();
        assert_eq!(font, cloned);
    }

    #[test]
    fn test_text_font_debug() {
        let font = TextFont::Times;
        let debug = format!("{:?}", font);
        assert!(debug.contains("Times"));

        let custom = TextFont::Custom("Arial".to_string());
        let debug = format!("{:?}", custom);
        assert!(debug.contains("Custom"));
        assert!(debug.contains("Arial"));
    }

    #[test]
    fn test_options_builder_lyric_size() {
        let options = Options::builder().lyric_size(0.8).build();
        assert_eq!(options.lyric_size, Some(0.8));
    }

    #[test]
    fn test_options_builder_condense() {
        let options = Options::builder().condense(CondenseMode::Auto).build();
        assert_eq!(options.condense, Some(CondenseMode::Auto));
    }

    #[test]
    fn test_options_builder_condense_first_page() {
        let options = Options::builder().condense_first_page(true).build();
        assert_eq!(options.condense_first_page, Some(true));
    }

    #[test]
    fn test_options_builder_condense_tempo_pages() {
        let options = Options::builder().condense_tempo_pages(false).build();
        assert_eq!(options.condense_tempo_pages, Some(false));
    }

    #[test]
    fn test_options_builder_even_note_spacing() {
        let options = Options::builder().even_note_spacing(true).build();
        assert_eq!(options.even_note_spacing, Some(true));
    }

    #[test]
    fn test_options_builder_min_measure_width() {
        let options = Options::builder().min_measure_width(150).build();
        assert_eq!(options.min_measure_width, Some(150));
    }

    #[test]
    fn test_options_builder_mnum_interval() {
        let options = Options::builder().mnum_interval(1).build();
        assert_eq!(options.mnum_interval, Some(1));
        assert!(options.to_json().unwrap().contains("\"mnumInterval\":1"));
    }

    #[test]
    fn test_options_builder_input_from() {
        let options = Options::builder().input_from("musicxml").build();
        assert_eq!(options.input_from, Some("musicxml".to_string()));
    }

    #[test]
    fn test_options_builder_mdiv_x_path_query() {
        let options = Options::builder()
            .mdiv_x_path_query("/score/mdiv[1]")
            .build();
        assert_eq!(
            options.mdiv_x_path_query,
            Some("/score/mdiv[1]".to_string())
        );
    }

    #[test]
    fn test_options_builder_expansion() {
        let options = Options::builder().expansion("expanded").build();
        assert_eq!(options.expansion, Some("expanded".to_string()));
    }

    #[test]
    fn test_options_clone() {
        let original = Options::builder().scale(80).font("Leipzig").build();
        let cloned = original.clone();
        assert_eq!(original.scale, cloned.scale);
        assert_eq!(original.font, cloned.font);
    }

    #[test]
    fn test_options_debug() {
        let options = Options::builder().scale(100).build();
        let debug = format!("{:?}", options);
        assert!(debug.contains("Options"));
        assert!(debug.contains("scale"));
        assert!(debug.contains("100"));
    }

    #[test]
    fn test_options_builder_clone() {
        let builder = Options::builder().scale(80);
        let cloned = builder.clone();
        let options1 = builder.page_width(2100).build();
        let options2 = cloned.page_height(2970).build();

        assert_eq!(options1.scale, Some(80));
        assert_eq!(options1.page_width, Some(2100));
        assert!(options1.page_height.is_none());

        assert_eq!(options2.scale, Some(80));
        assert!(options2.page_width.is_none());
        assert_eq!(options2.page_height, Some(2970));
    }

    #[test]
    fn test_options_builder_debug() {
        let builder = Options::builder().scale(100);
        let debug = format!("{:?}", builder);
        assert!(debug.contains("OptionsBuilder"));
    }

    #[test]
    fn test_options_builder_default() {
        let builder = OptionsBuilder::default();
        let options = builder.build();
        assert!(options.scale.is_none());
    }

    #[test]
    fn test_options_all_fields_in_json() {
        let options = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .adjust_page_height(true)
            .page_margin(50)
            .page_margin_top(100)
            .page_margin_bottom(100)
            .page_margin_left(75)
            .page_margin_right(75)
            .font("Leipzig")
            .lyric_size(0.8)
            .breaks(BreakMode::Smart)
            .condense(CondenseMode::Auto)
            .condense_first_page(true)
            .condense_tempo_pages(false)
            .even_note_spacing(true)
            .min_measure_width(150)
            .mnum_interval(1)
            .header(HeaderMode::Auto)
            .footer(FooterMode::Always)
            .svg_xml_declaration(true)
            .svg_bounding_boxes(false)
            .svg_view_box(true)
            .svg_remove_xlink(false)
            .svg_css("svg { background: white; }")
            .svg_format_raw(true)
            .svg_font_face_include(false)
            .midi_tempo(120.0)
            .midi_velocity(80)
            .input_from("mei")
            .mdiv_x_path_query("/score")
            .expansion("expanded")
            .transpose("M2")
            .transpose_selected_only(true)
            .transpose_to_sounding_pitch(false)
            .spacing_staff(12)
            .spacing_system(6)
            .spacing_linear(0.25)
            .spacing_non_linear(0.6)
            .build();

        let json = options.to_json().unwrap();

        // Verify camelCase conversion
        assert!(json.contains("pageWidth"));
        assert!(json.contains("pageHeight"));
        assert!(json.contains("adjustPageHeight"));
        assert!(json.contains("pageMarginTop"));
        assert!(json.contains("pageMarginBottom"));
        assert!(json.contains("pageMarginLeft"));
        assert!(json.contains("pageMarginRight"));
        assert!(json.contains("lyricSize"));
        assert!(json.contains("condenseFirstPage"));
        assert!(json.contains("evenNoteSpacing"));
        assert!(json.contains("minMeasureWidth"));
        assert!(json.contains("mnumInterval"));
        assert!(json.contains("svgXmlDeclaration"));
        assert!(json.contains("svgBoundingBoxes"));
        assert!(json.contains("svgViewBox"));
        assert!(json.contains("svgRemoveXlink"));
        assert!(json.contains("svgCss"));
        assert!(json.contains("svgFormatRaw"));
        assert!(json.contains("svgFontFaceInclude"));
        assert!(json.contains("midiTempo"));
        assert!(json.contains("midiVelocity"));
        assert!(json.contains("inputFrom"));
        assert!(json.contains("mdivXPathQuery"));
        assert!(json.contains("transposeSelectedOnly"));
        assert!(json.contains("transposeToSoundingPitch"));
        assert!(json.contains("spacingStaff"));
        assert!(json.contains("spacingSystem"));
        assert!(json.contains("spacingLinear"));
        assert!(json.contains("spacingNonLinear"));
    }

    #[test]
    fn test_options_round_trip_all_fields() {
        let original = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .adjust_page_height(true)
            .page_margin(50)
            .lyric_size(0.8)
            .breaks(BreakMode::Encoded)
            .condense(CondenseMode::Encoded)
            .header(HeaderMode::Encoded)
            .footer(FooterMode::Encoded)
            .midi_tempo(120.0)
            .midi_velocity(80)
            .spacing_linear(0.25)
            .spacing_non_linear(0.6)
            .build();

        let json = original.to_json().unwrap();
        let parsed = Options::from_json(&json).unwrap();

        assert_eq!(original.scale, parsed.scale);
        assert_eq!(original.page_width, parsed.page_width);
        assert_eq!(original.page_height, parsed.page_height);
        assert_eq!(original.adjust_page_height, parsed.adjust_page_height);
        assert_eq!(original.page_margin_top, parsed.page_margin_top);
        assert_eq!(original.page_margin_bottom, parsed.page_margin_bottom);
        assert_eq!(original.page_margin_left, parsed.page_margin_left);
        assert_eq!(original.page_margin_right, parsed.page_margin_right);
        assert_eq!(original.lyric_size, parsed.lyric_size);
        assert_eq!(original.breaks, parsed.breaks);
        assert_eq!(original.condense, parsed.condense);
        assert_eq!(original.header, parsed.header);
        assert_eq!(original.footer, parsed.footer);
        assert_eq!(original.midi_tempo, parsed.midi_tempo);
        assert_eq!(original.midi_velocity, parsed.midi_velocity);
        assert_eq!(original.spacing_linear, parsed.spacing_linear);
        assert_eq!(original.spacing_non_linear, parsed.spacing_non_linear);
    }

    #[test]
    fn test_break_mode_clone() {
        let mode = BreakMode::Smart;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_break_mode_copy() {
        let mode = BreakMode::Line;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_break_mode_debug() {
        let mode = BreakMode::Encoded;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("Encoded"));
    }

    #[test]
    fn test_condense_mode_clone() {
        let mode = CondenseMode::Auto;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_condense_mode_copy() {
        let mode = CondenseMode::Encoded;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_condense_mode_debug() {
        let mode = CondenseMode::Auto;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("Auto"));
    }

    #[test]
    fn test_footer_mode_clone() {
        let mode = FooterMode::Always;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_footer_mode_copy() {
        let mode = FooterMode::Auto;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_footer_mode_debug() {
        let mode = FooterMode::Encoded;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("Encoded"));
    }

    #[test]
    fn test_header_mode_clone() {
        let mode = HeaderMode::Auto;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_header_mode_copy() {
        let mode = HeaderMode::Encoded;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_header_mode_debug() {
        let mode = HeaderMode::None;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("None"));
    }

    #[test]
    fn test_options_json_with_enums() {
        let options = Options::builder()
            .breaks(BreakMode::None)
            .condense(CondenseMode::None)
            .header(HeaderMode::None)
            .footer(FooterMode::None)
            .build();

        let json = options.to_json().unwrap();
        assert!(json.contains(r#""breaks":"none""#));
        assert!(json.contains(r#""condense":"none""#));
        assert!(json.contains(r#""header":"none""#));
        assert!(json.contains(r#""footer":"none""#));
    }

    #[test]
    fn test_options_empty_json() {
        let options = Options::default();
        let json = options.to_json().unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_options_partial_json_deserialization() {
        // Test that we can deserialize JSON with only some fields set
        let json = r#"{"scale":100,"breaks":"smart"}"#;
        let options = Options::from_json(json).unwrap();
        assert_eq!(options.scale, Some(100));
        assert_eq!(options.breaks, Some(BreakMode::Smart));
        assert!(options.page_width.is_none());
        assert!(options.font.is_none());
    }

    #[test]
    fn test_break_mode_invalid_deserialize() {
        let result: std::result::Result<BreakMode, _> = serde_json::from_str(r#""invalid_mode""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_condense_mode_invalid_deserialize() {
        let result: std::result::Result<CondenseMode, _> =
            serde_json::from_str(r#""invalid_mode""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_mode_invalid_deserialize() {
        let result: std::result::Result<HeaderMode, _> = serde_json::from_str(r#""invalid_mode""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_footer_mode_invalid_deserialize() {
        let result: std::result::Result<FooterMode, _> = serde_json::from_str(r#""invalid_mode""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_options_builder_string_methods_with_string() {
        // Test that methods accepting impl Into<String> work with String
        let options = Options::builder()
            .font(String::from("Bravura"))
            .svg_css(String::from("svg {}"))
            .input_from(String::from("humdrum"))
            .mdiv_x_path_query(String::from("/"))
            .expansion(String::from("exp"))
            .transpose(String::from("P5"))
            .build();

        assert_eq!(options.font, Some("Bravura".to_string()));
        assert_eq!(options.svg_css, Some("svg {}".to_string()));
        assert_eq!(options.input_from, Some("humdrum".to_string()));
        assert_eq!(options.mdiv_x_path_query, Some("/".to_string()));
        assert_eq!(options.expansion, Some("exp".to_string()));
        assert_eq!(options.transpose, Some("P5".to_string()));
    }

    #[test]
    fn test_options_lyric_size_edge_values() {
        // Test with edge values for lyric size
        let options1 = Options::builder().lyric_size(0.0).build();
        assert_eq!(options1.lyric_size, Some(0.0));

        let options2 = Options::builder().lyric_size(1.0).build();
        assert_eq!(options2.lyric_size, Some(1.0));

        let options3 = Options::builder().lyric_size(2.5).build();
        assert_eq!(options3.lyric_size, Some(2.5));
    }

    #[test]
    fn test_options_midi_velocity_edge_values() {
        let options1 = Options::builder().midi_velocity(0).build();
        assert_eq!(options1.midi_velocity, Some(0));

        let options2 = Options::builder().midi_velocity(127).build();
        assert_eq!(options2.midi_velocity, Some(127));

        let options3 = Options::builder().midi_velocity(255).build();
        assert_eq!(options3.midi_velocity, Some(255));
    }

    #[test]
    fn test_options_scale_edge_values() {
        let options1 = Options::builder().scale(1).build();
        assert_eq!(options1.scale, Some(1));

        let options2 = Options::builder().scale(100).build();
        assert_eq!(options2.scale, Some(100));

        let options3 = Options::builder().scale(1000).build();
        assert_eq!(options3.scale, Some(1000));
    }

    #[test]
    fn test_options_spacing_factors_edge_values() {
        let options = Options::builder()
            .spacing_linear(0.0)
            .spacing_non_linear(0.0)
            .build();
        assert_eq!(options.spacing_linear, Some(0.0));
        assert_eq!(options.spacing_non_linear, Some(0.0));

        let options2 = Options::builder()
            .spacing_linear(1.0)
            .spacing_non_linear(1.0)
            .build();
        assert_eq!(options2.spacing_linear, Some(1.0));
        assert_eq!(options2.spacing_non_linear, Some(1.0));
    }
}
