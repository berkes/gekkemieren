//! Central configuration module for the ant simulation.
//!
//! This module holds all simulation constants in a single `Config` struct
//! that is passed to both the GPU (via buffers) and used on the CPU.

use bytemuck::{Pod, Zeroable};

/// Simulation-wide constants that are passed to GPU shaders.
/// Must match the `SimConfig` struct in all WGSL shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SimConfig {
    pub decay_amount: u32,
    pub max_strength: u32,
    pub deposit_amount: u32,
    pub dot_radius: f32,
    pub collision_radius: f32,
    pub collision_angle_min: f32,
    pub collision_angle_max: f32,
    pub forager_randomness: f32,
    pub scout_randomness: f32,
    pub sensor_distance: f32,
    pub sensor_angle: f32,
    pub _pad: [u32; 1],
}

/// Main application configuration holding all simulation parameters.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub sim_config: SimConfig,
    pub n_ants: usize,
    pub base_speed: f32,
    pub initial_scout_ratio: f32,
    pub ratio_step: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sim_config: SimConfig {
                decay_amount: 1,
                max_strength: 1000,
                deposit_amount: 50,
                dot_radius: 0.001,
                collision_radius: 0.0001,
                collision_angle_min: 1.169_370_6, // 67 degrees
                collision_angle_max: 1.954_768_8, // 112 degrees
                forager_randomness: 0.1,
                scout_randomness: 0.1,
                sensor_distance: 0.0060,
                sensor_angle: 0.524, // ~30 degrees
                _pad: [0; 1],
            },
            n_ants: 15000,
            base_speed: 0.0015,
            initial_scout_ratio: 0.75,
            ratio_step: 0.05,
        }
    }
}

impl Config {
    /// Returns the GPU-compatible simulation config.
    pub fn sim_config(&self) -> SimConfig {
        self.sim_config
    }
}
