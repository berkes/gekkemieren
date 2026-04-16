#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorScheme {
    pub background: [f32; 4],
    pub forager: [f32; 4],
    pub scout: [f32; 4],
    pub pheromone: [f32; 4],
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    BoldHues,
    WarmEarth,
    OceanSunsetVibes,
    Debug
}

impl ColorScheme {
    pub fn from_palette(palette: Palette) -> Self {
        match palette {
            Palette::BoldHues => Self {
                // https://coolors.co/palette/f72585-7209b7-3a0ca3-4361ee-4cc9f0
                // background: [1.000, 1.000, 1.000, 1.0],
                background: [0.0, 0.0, 0.0, 1.0],
                forager: [0.227, 0.047, 0.639, 1.0],   // #3a0ca3
                scout: [0.263, 0.380, 0.933, 1.0],     // #4361ee
                pheromone: [0.447, 0.035, 0.718, 1.0], // #7209b7
            },
            Palette::WarmEarth => Self {
                // https://coolors.co/palette/c9cba3-ffe1a8-e26d5c
                background: [1.000, 1.000, 1.000, 1.0],
                forager: [0.886, 0.427, 0.361, 1.0],   // #e26d5c
                scout: [0.788, 0.796, 0.639, 1.0],     // #c9cba3
                pheromone: [1.000, 0.882, 0.659, 1.0], // #ffe1a8
            },
            Palette::OceanSunsetVibes => Self {
                // https://coolors.co/palette/26547c-ef476f-ffd166-06d6a0
                background: [0.149, 0.325, 0.478, 1.0], // #26547C Dusk Blue background
                pheromone: [0.988, 0.824, 0.412, 1.0],     // #FFD166 Royal Gold
                scout: [0.922, 0.298, 0.443, 1.0],   // #EF476F Bubblegum Pink
                forager: [0.027, 0.851, 0.635, 1.0], // #06D6A0 Emerald
            },
            Palette::Debug => Self {
                background: [1.0, 1.0, 1.0, 1.0],
                forager: [0.0, 0.0, 0.0, 1.0],
                scout: [0.0, 0.0, 1.0, 1.0],
                pheromone: [0.0, 1.0, 0.0, 1.0],
            },
        }
    }
}
