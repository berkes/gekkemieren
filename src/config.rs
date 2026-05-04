//! Central configuration module for the ant simulation.
//!
//! This module holds all simulation constants in a `Config` struct for CPU use,
//! and a `GpuConfig` struct for GPU buffer compatibility.
//!
//! Configuration is loaded from a TOML file. All fields are required.

use std::path::Path;

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::color_scheme::Palette;

/// Main application configuration holding all simulation parameters.
/// Uses natural Rust types (e.g., usize for counts).
/// All fields are required and must be present in the config file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub decay_ratio: f32,
    pub deposit_ratio: f32,
    pub forager_randomness: f32,
    pub scout_randomness: f32,
    pub sensor_distance: f32,
    pub sensor_angle: f32,
    pub n_ants: usize,
    pub base_speed: f32,
    pub scout_ratio: f32,
    pub ratio_step: f32,
    pub food_source_radius: f32,
    pub window_width: u32,
    pub window_height: u32,
    pub palette: Palette,
}

impl Config {
    /// Load configuration from a TOML file.
    /// Fails if the file cannot be read or deserialized.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }
}

/// GPU-compatible configuration struct.
/// Must be #[repr(C)] and Pod/Zeroable for GPU buffer compatibility.
/// Must match the `GpuConfig` struct in all WGSL shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Serialize)]
pub struct GpuConfig {
    pub decay_ratio: f32,
    pub deposit_ratio: f32,
    pub forager_randomness: f32,
    pub scout_randomness: f32,
    pub sensor_distance: f32,
    pub sensor_angle: f32,
    pub n_ants: u32,
    pub base_speed: f32,
    pub scout_ratio: f32,
    pub ratio_step: f32,
    pub food_source_radius: f32,
    // Window and UI settings (not used by GPU shaders but included for simplicity)
    pub window_width: u32,
    pub window_height: u32,
}

impl From<&Config> for GpuConfig {
    fn from(config: &Config) -> Self {
        Self {
            decay_ratio: config.decay_ratio,
            deposit_ratio: config.deposit_ratio,
            forager_randomness: config.forager_randomness,
            scout_randomness: config.scout_randomness,
            sensor_distance: config.sensor_distance,
            sensor_angle: config.sensor_angle,
            n_ants: config.n_ants as u32,
            base_speed: config.base_speed,
            scout_ratio: config.scout_ratio,
            ratio_step: config.ratio_step,
            food_source_radius: config.food_source_radius,
            window_width: config.window_width,
            window_height: config.window_height,
        }
    }
}
