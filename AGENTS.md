# Gekke Mieren

A GPU-accelerated Ant simulation written in Rust using WGPU compute shaders. The simulation models emergent behavior of millions of agents following simple rules, creating complex organic patterns. 

Running the application will visualize moving Ants that use pheromone trails to
find and retrieve food and avoid obstructions.

## Common Development Commands

### Building and Running
- **Development build**: `cargo run`
- **Test**: `cargo test`
- **Lint and check**: `cargo check`
- **Format**: `cargo fmt`

## Architecture Overview

### Core Application Structure

- **App** (`src/app.rs`): Main application state and event loop using winit/ApplicationHandler
- **GPU setup** (`src/wgpu_setup.rs`): Boilerplate for WGPU
- **Rendering** (`src/pipeline.rs`): Pipeline and buffer setup

## Development Notes

### Iterate

Develop in small iterations. Each iteration:

* change code
* run tests
* run lint and format to determine warnings and errors
* repeat these steps until no warnings or errors exist in tests, linting and formatting
* run the app and asks human for confirmation. Explain what the human should look for:
  * When refactoring: ask that everything works as before
  * When adding, changing or removing features: summarize what should appear and ask human if this appears

### Guidelines

* Prefer KISS over clever but complex setups
* Don't add comments in code that explain *what* code does. Make the code self-explanatory intead
* Keep code modular and separate concerns: one responsibility per unit
* Never add inline comments to disable linting or formatting errors. Fix the issue instead
* Complex functions or behaviour need inline tests. There are no integration tests

### Performance Considerations

- Agent count directly impacts GPU workload
- Texture size limited by GPU capabilities
- Frame limiting available for power management

### Shader Development
- All shaders in `src/shaders/` directory
- Uses WGSL (WebGPU Shading Language)
- Compute shaders handle simulation logic
- Render shaders handle visualization
