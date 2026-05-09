use serde::{Deserialize, Serialize};

/// sRGB to linear conversion constants
///
/// sRGB uses a piecewise function:
/// - Below SRGB_THRESHOLD: linear = srgb / SRGB_SCALE_BELOW
/// - Above SRGB_THRESHOLD: linear = ((srgb + SRGB_ALPHA) / SRGB_BETA) ^ SRGB_GAMMA
const SRGB_THRESHOLD: f32 = 0.04045;
const SRGB_SCALE_BELOW: f32 = 12.92;
const SRGB_ALPHA: f32 = 0.055;
const SRGB_BETA: f32 = 1.055;
const SRGB_GAMMA: f32 = 2.4;

const DEFAULT_ALPHA: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorScheme {
    pub background: [f32; 4],
    pub forager: [f32; 4],
    pub scout: [f32; 4],
    pub homing_pheromone: [f32; 4],
    pub food_pheromone: [f32; 4],
    pub food: [f32; 4],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    Pastel,
    Light,
    Debug,
}

/// Convert sRGB value (0.0-1.0) to linear RGB
///
/// Formula:
/// - If c <= 0.04045: c / 12.92
/// - Otherwise: ((c + 0.055) / 1.055) ^ 2.4
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= SRGB_THRESHOLD {
        c / SRGB_SCALE_BELOW
    } else {
        ((c + SRGB_ALPHA) / SRGB_BETA).powf(SRGB_GAMMA)
    }
}

/// Parse hex color string (e.g., "#4c054d" or "4c054d") to linear RGB [f32; 3]
fn hex_to_linear_rgb(hex: &str) -> [f32; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

    [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b)]
}

/// Create a ColorScheme color (linear RGB + alpha) from hex string
fn color_from_hex(hex: &str) -> [f32; 4] {
    let rgb = hex_to_linear_rgb(hex);
    [rgb[0], rgb[1], rgb[2], DEFAULT_ALPHA]
}

impl ColorScheme {
    pub fn from_palette(palette: Palette) -> Self {
        match palette {
            Palette::Pastel => Self {
                //  #ff99c8, #fcf6bd, #d0f4de, #a9def9, #e4c1f9
                background: color_from_hex("#0c090d"),
                forager: color_from_hex("#d0f4de"),
                scout: color_from_hex("#a9def9"),
                homing_pheromone: color_from_hex("#e4c1f9"),
                food_pheromone: color_from_hex("#fcf6bd"),
                food: color_from_hex("#fcf6bd"),
            },
            Palette::Light => Self {
                // #e7ecef, #274c77, #6096ba, #a3cef1
                background: color_from_hex("#e7ecef"),
                forager: color_from_hex("#0c090d"),
                scout: color_from_hex("#274c77"),
                homing_pheromone: color_from_hex("#6096ba"),
                food_pheromone: color_from_hex("#a3cef1"),
                food: color_from_hex("#a3cef1"),
            },
            Palette::Debug => Self {
                background: [1.0, 1.0, 1.0, 1.0],
                forager: [0.1, 0.0, 0.0, 1.0],
                scout: [0.0, 0.1, 0.0, 1.0],
                homing_pheromone: [0.0, 1.0, 1.0, 1.0],
                food_pheromone: [1.0, 1.0, 0.0, 1.0],
                food: [0.0, 0.0, 1.0, 1.0],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_linear_black() {
        let result = hex_to_linear_rgb("#000000");
        assert!((result[0] - 0.0).abs() < 0.0001);
        assert!((result[1] - 0.0).abs() < 0.0001);
        assert!((result[2] - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_hex_to_linear_white() {
        let result = hex_to_linear_rgb("#ffffff");
        // White in linear is still [1.0, 1.0, 1.0]
        assert!((result[0] - 1.0).abs() < 0.0001);
        assert!((result[1] - 1.0).abs() < 0.0001);
        assert!((result[2] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_hex_to_linear_gray_50() {
        // #808080 = 128/255 ≈ 0.50196 sRGB
        // Linear: ((0.50196 + 0.055) / 1.055) ^ 2.4 ≈ (0.55696/1.055)^2.4 ≈ 0.527^2.4 ≈ 0.225
        let result = hex_to_linear_rgb("#808080");
        let expected = ((0.50196 + SRGB_ALPHA) / SRGB_BETA).powf(SRGB_GAMMA);
        assert!((result[0] - expected).abs() < 0.0001);
        assert!((result[1] - expected).abs() < 0.0001);
        assert!((result[2] - expected).abs() < 0.0001);
    }

    #[test]
    fn test_hex_to_linear_red() {
        // #ff0000 = pure red
        let result = hex_to_linear_rgb("#ff0000");
        assert!((result[0] - 1.0).abs() < 0.0001);
        assert!((result[1] - 0.0).abs() < 0.0001);
        assert!((result[2] - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_hex_without_hash() {
        let with_hash = hex_to_linear_rgb("#4c054d");
        let without_hash = hex_to_linear_rgb("4c054d");
        assert!((with_hash[0] - without_hash[0]).abs() < 0.0001);
        assert!((with_hash[1] - without_hash[1]).abs() < 0.0001);
        assert!((with_hash[2] - without_hash[2]).abs() < 0.0001);
    }

    #[test]
    fn test_color_from_hex() {
        let color = color_from_hex("#ffffff");
        assert!((color[0] - 1.0).abs() < 0.0001);
        assert!((color[1] - 1.0).abs() < 0.0001);
        assert!((color[2] - 1.0).abs() < 0.0001);
        assert!((color[3] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_palette_pastel() {
        let scheme = ColorScheme::from_palette(Palette::Pastel);

        // Verify all alphas are 1.0
        assert_eq!(scheme.background[3], 1.0);
        assert_eq!(scheme.forager[3], 1.0);
        assert_eq!(scheme.scout[3], 1.0);
        assert_eq!(scheme.homing_pheromone[3], 1.0);
        assert_eq!(scheme.food_pheromone[3], 1.0);
        assert_eq!(scheme.food[3], 1.0);

        // Verify values are in valid range [0, 1]
        for field in [
            scheme.background,
            scheme.forager,
            scheme.scout,
            scheme.homing_pheromone,
            scheme.food_pheromone,
            scheme.food,
        ] {
            assert!(field[0] >= 0.0 && field[0] <= 1.0);
            assert!(field[1] >= 0.0 && field[1] <= 1.0);
            assert!(field[2] >= 0.0 && field[2] <= 1.0);
        }
    }
}
