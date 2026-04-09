mod ant;
mod app;
mod pheromone;
mod pipeline;
mod spawn;
mod wgpu_setup;

use app::run;

fn main() {
    run().unwrap();
}
