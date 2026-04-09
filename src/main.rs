mod ant;
mod app;
mod pipeline;
mod wgpu_setup;

use app::run;

fn main() {
    run().unwrap();
}
