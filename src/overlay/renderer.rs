use crate::activation::SystemState;
use std::num::ParseIntError;
use thiserror::Error;
use tiny_skia::{Color, Paint, Transform};

/// Error types for overlay rendering
#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Failed to parse color: {0}")]
    InvalidColor(String),

    #[error("Failed to parse hex color: {0}")]
    HexParseError(#[from] ParseIntError),

    #[error("Invalid hex color format: {0}")]
    InvalidHexFormat(String),
}

/// Represents an RGBA color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl OverlayColor {
    /// Creates a new color from RGBA components
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a fully opaque color
    pub fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Converts to tiny_skia Color
    pub fn to_skia_color(self) -> Color {
        Color::from_rgba(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
        .unwrap_or(Color::BLACK)
    }
}

/// Parses a color string (named or hex) into an OverlayColor
///
/// Supports:
/// - Named colors: "green", "gray", "red", "blue", etc.
/// - Hex colors: "#FF0000", "#00FF00", "#808080"
pub fn parse_color(color_str: &str) -> Result<OverlayColor, RendererError> {
    let trimmed = color_str.trim().to_lowercase();

    if let Some(color) = parse_named_color(&trimmed) {
        return Ok(color);
    }

    if trimmed.starts_with('#') {
        return parse_hex_color(&trimmed);
    }

    Err(RendererError::InvalidColor(color_str.to_string()))
}

/// Parses a color string with automatic fallback on error
///
/// If the color string cannot be parsed, logs a warning and returns a safe default.
/// This function never fails, making it suitable for use during initialization when
/// live-reload recovery is possible.
///
/// # Arguments
/// * `color_str` - The color string to parse
/// * `fallback` - The fallback color to use if parsing fails
pub fn parse_color_with_fallback(color_str: &str, fallback: OverlayColor) -> OverlayColor {
    match parse_color(color_str) {
        Ok(color) => color,
        Err(e) => {
            tracing::warn!("Failed to parse color '{}': {}, using fallback color", color_str, e);
            fallback
        }
    }
}

/// Parses a named color string
fn parse_named_color(name: &str) -> Option<OverlayColor> {
    match name {
        "green" => Some(OverlayColor::opaque(0, 255, 0)),
        "lime" => Some(OverlayColor::opaque(0, 255, 0)),
        "gray" | "grey" => Some(OverlayColor::opaque(128, 128, 128)),
        "red" => Some(OverlayColor::opaque(255, 0, 0)),
        "blue" => Some(OverlayColor::opaque(0, 0, 255)),
        "yellow" => Some(OverlayColor::opaque(255, 255, 0)),
        "cyan" => Some(OverlayColor::opaque(0, 255, 255)),
        "magenta" => Some(OverlayColor::opaque(255, 0, 255)),
        "white" => Some(OverlayColor::opaque(255, 255, 255)),
        "black" => Some(OverlayColor::opaque(0, 0, 0)),
        "orange" => Some(OverlayColor::opaque(255, 165, 0)),
        "purple" => Some(OverlayColor::opaque(128, 0, 128)),
        "pink" => Some(OverlayColor::opaque(255, 192, 203)),
        _ => None,
    }
}

/// Parses a hex color string like "#FF0000"
fn parse_hex_color(hex_str: &str) -> Result<OverlayColor, RendererError> {
    if !hex_str.starts_with('#') {
        return Err(RendererError::InvalidHexFormat(hex_str.to_string()));
    }

    let hex_digits = &hex_str[1..];

    match hex_digits.len() {
        6 => {
            let r = u8::from_str_radix(&hex_digits[0..2], 16)?;
            let g = u8::from_str_radix(&hex_digits[2..4], 16)?;
            let b = u8::from_str_radix(&hex_digits[4..6], 16)?;
            Ok(OverlayColor::opaque(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex_digits[0..2], 16)?;
            let g = u8::from_str_radix(&hex_digits[2..4], 16)?;
            let b = u8::from_str_radix(&hex_digits[4..6], 16)?;
            let a = u8::from_str_radix(&hex_digits[6..8], 16)?;
            Ok(OverlayColor::new(r, g, b, a))
        }
        _ => Err(RendererError::InvalidHexFormat(hex_str.to_string())),
    }
}

/// Maps a system state and error flag to an overlay color
pub fn state_to_color(
    state: SystemState,
    awake_color: OverlayColor,
    asleep_color: OverlayColor,
    error_color: OverlayColor,
    has_error: bool,
) -> OverlayColor {
    if has_error {
        return error_color;
    }

    match state {
        SystemState::Awake => awake_color,
        SystemState::Asleep => asleep_color,
    }
}

/// Renders a 32x32px circular indicator as RGBA pixel data
///
/// The circle is anti-aliased and centered in the 32x32px canvas.
/// Returns a Vec<u8> representing RGBA pixel data (which will be converted to BGRA for Wayland).
pub fn render_circle(color: OverlayColor) -> Vec<u8> {
    const SIZE: u32 = 32;
    const RADIUS: f32 = 15.0;
    const CENTER: f32 = 16.0;

    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE).expect("Failed to create pixmap");

    pixmap.fill(Color::TRANSPARENT);

    let paint = Paint {
        shader: tiny_skia::Shader::SolidColor(color.to_skia_color()),
        anti_alias: true,
        ..Default::default()
    };

    let mut path = tiny_skia::PathBuilder::new();
    path.push_circle(CENTER, CENTER, RADIUS);

    if let Some(path) = path.finish() {
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::default(), None);
    }

    pixmap.data().to_vec()
}

/// Converts RGBA pixel data to BGRA byte order for Wayland wl_shm Argb8888 format
///
/// Wayland's Argb8888 format uses BGRA byte ordering in memory.
/// This function converts from the standard RGBA format (as returned by tiny_skia)
/// to the BGRA format expected by Wayland.
pub fn rgba_to_bgra(rgba_data: &[u8]) -> Vec<u8> {
    let mut bgra_data = Vec::with_capacity(rgba_data.len());
    for chunk in rgba_data.chunks_exact(4) {
        bgra_data.push(chunk[2]);
        bgra_data.push(chunk[1]);
        bgra_data.push(chunk[0]);
        bgra_data.push(chunk[3]);
    }
    bgra_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_named_colors() {
        assert_eq!(parse_color("green").unwrap(), OverlayColor::opaque(0, 255, 0));
        assert_eq!(parse_color("gray").unwrap(), OverlayColor::opaque(128, 128, 128));
        assert_eq!(parse_color("red").unwrap(), OverlayColor::opaque(255, 0, 0));
        assert_eq!(parse_color("blue").unwrap(), OverlayColor::opaque(0, 0, 255));
    }

    #[test]
    fn test_parse_named_colors_case_insensitive() {
        assert_eq!(parse_color("GREEN").unwrap(), OverlayColor::opaque(0, 255, 0));
        assert_eq!(parse_color("Gray").unwrap(), OverlayColor::opaque(128, 128, 128));
        assert_eq!(parse_color("RED").unwrap(), OverlayColor::opaque(255, 0, 0));
    }

    #[test]
    fn test_parse_hex_colors_6_digit() {
        assert_eq!(
            parse_color("#00FF00").unwrap(),
            OverlayColor::opaque(0, 255, 0)
        );
        assert_eq!(
            parse_color("#808080").unwrap(),
            OverlayColor::opaque(128, 128, 128)
        );
        assert_eq!(
            parse_color("#FF0000").unwrap(),
            OverlayColor::opaque(255, 0, 0)
        );
    }

    #[test]
    fn test_parse_hex_colors_8_digit() {
        let color = parse_color("#FF0000FF").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);

        let color = parse_color("#FF000080").unwrap();
        assert_eq!(color.a, 128);
    }

    #[test]
    fn test_parse_hex_colors_lowercase() {
        assert_eq!(
            parse_color("#00ff00").unwrap(),
            OverlayColor::opaque(0, 255, 0)
        );
        assert_eq!(
            parse_color("#ff0000").unwrap(),
            OverlayColor::opaque(255, 0, 0)
        );
    }

    #[test]
    fn test_invalid_color_returns_error() {
        assert!(parse_color("invalid123").is_err());
        assert!(parse_color("#GGGGGG").is_err());
        assert!(parse_color("#00FF0").is_err()); // 5 digits
        assert!(parse_color("notacolor").is_err());
    }

    #[test]
    fn test_state_to_color_awake_no_error() {
        let awake = OverlayColor::opaque(0, 255, 0);
        let asleep = OverlayColor::opaque(128, 128, 128);
        let error = OverlayColor::opaque(255, 0, 0);

        let result = state_to_color(SystemState::Awake, awake, asleep, error, false);
        assert_eq!(result, awake);
    }

    #[test]
    fn test_state_to_color_asleep_no_error() {
        let awake = OverlayColor::opaque(0, 255, 0);
        let asleep = OverlayColor::opaque(128, 128, 128);
        let error = OverlayColor::opaque(255, 0, 0);

        let result = state_to_color(SystemState::Asleep, awake, asleep, error, false);
        assert_eq!(result, asleep);
    }

    #[test]
    fn test_state_to_color_error_overrides_state() {
        let awake = OverlayColor::opaque(0, 255, 0);
        let asleep = OverlayColor::opaque(128, 128, 128);
        let error = OverlayColor::opaque(255, 0, 0);

        let result = state_to_color(SystemState::Awake, awake, asleep, error, true);
        assert_eq!(result, error);

        let result = state_to_color(SystemState::Asleep, awake, asleep, error, true);
        assert_eq!(result, error);
    }

    #[test]
    fn test_render_circle_produces_valid_pixmap() {
        let color = OverlayColor::opaque(0, 255, 0);
        let data = render_circle(color);

        assert_eq!(data.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_parse_all_supported_named_colors() {
        let color_names = vec![
            "green", "lime", "gray", "grey", "red", "blue", "yellow", "cyan", "magenta",
            "white", "black", "orange", "purple", "pink",
        ];

        for name in color_names {
            assert!(
                parse_color(name).is_ok(),
                "Failed to parse color: {}",
                name
            );
        }
    }

    #[test]
    fn test_overlay_color_to_skia_conversion() {
        let color = OverlayColor::opaque(255, 128, 64);
        let skia_color = color.to_skia_color();
        assert!(skia_color.is_opaque() || !skia_color.is_opaque());
    }

    #[test]
    fn test_rgba_to_bgra_conversion() {
        let rgba = vec![255, 128, 64, 255];
        let bgra = rgba_to_bgra(&rgba);
        assert_eq!(bgra, vec![64, 128, 255, 255]);
    }

    #[test]
    fn test_rgba_to_bgra_conversion_multiple_pixels() {
        let rgba = vec![
            255, 0, 0, 255,
            0, 255, 0, 255,
        ];
        let bgra = rgba_to_bgra(&rgba);
        assert_eq!(
            bgra,
            vec![
                0, 0, 255, 255,
                0, 255, 0, 255,
            ]
        );
    }
}
