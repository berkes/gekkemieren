mod app;
mod shader;
mod wgpu_setup;

use app::run;

fn main() {
    run().unwrap();
}
