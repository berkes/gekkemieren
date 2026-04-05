mod app;
mod camera;
mod shader;
mod texture;
mod wgpu_setup;

use app::run;

fn main() {
    run().unwrap();
}
