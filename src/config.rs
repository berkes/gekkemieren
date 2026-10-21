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
    pub diffusion_ratio: f32,
    pub deposit_ratio: f32,
    pub forager_randomness: f32,
    pub scout_randomness: f32,
    pub sensor_distance: f32,
    pub sensor_angle: f32,
    pub n_ants: usize,
    pub ant_lifetime: u32,
    pub base_speed: f32,
    pub scout_ratio: f32,
    pub ratio_step: f32,
    pub food_source_center: [f32; 2],
    pub food_source_radius: f32,
    pub window_width: u32,
    pub window_height: u32,
    pub palette: Palette,
    pub colony_center: [f32; 2],
    pub colony_radius: f32,
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
    pub scout_ratio: f32,
    pub ant_lifetime: u32,
    pub food_source_center: [f32; 2],
    pub food_source_radius: f32,
    // Occupies the former _pad1 slot: keeps colony_center 8-byte aligned for
    // WGSL, where vec2<f32> members require 8-byte alignment.
    pub diffusion_ratio: f32,
    pub colony_center: [f32; 2],
    pub colony_radius: f32,
    pub _pad2: u32,
    pub _pad3: u32,
    pub _pad4: u32,
}

/// GPU-compatible simulation state struct.
/// Holds runtime information like frame counter that changes each frame.
/// Must be #[repr(C)] and Pod/Zeroable for GPU buffer compatibility.
/// Must match the `SimulationInfo` struct in WGSL shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct SimulationInfo {
    pub frame_counter: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

impl SimulationInfo {
    pub fn new() -> Self {
        Self::default()
    }
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
            scout_ratio: config.scout_ratio,
            ant_lifetime: config.ant_lifetime,
            food_source_center: config.food_source_center,
            food_source_radius: config.food_source_radius,
            diffusion_ratio: config.diffusion_ratio,
            colony_center: config.colony_center,
            colony_radius: config.colony_radius,
            _pad2: 0,
            _pad3: 0,
            _pad4: 0,
        }
    }
}
