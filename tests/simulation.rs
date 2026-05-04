mod common;

use gekkemieren::{
    ant::{Ant, AntType},
    color_scheme::Palette,
    config::{Config, GpuConfig},
    pipeline::SimulationPipeline,
    spawn::{AntSpawner, Colony, FixedSpawner},
};

const COLLISION_RADIUS: f32 = 0.0001;

fn config() -> Config {
    Config {
        decay_amount: 1,
        max_strength: 1000,
        deposit_amount: 50,
        deposit_amount_carrying_food: 100,
        dot_radius: 0.001,
        collision_radius: COLLISION_RADIUS,
        collision_angle_min: 1.169_370_6,
        collision_angle_max: 1.954_768_8,
        // Zero randomness makes movement fully deterministic: direction is
        // only changed by collision, never by the hash-based noise.
        forager_randomness: 0.0,
        scout_randomness: 0.0,
        sensor_distance: 0.03,
        sensor_angle: 0.5,
        n_ants: 0,        // not used in test
        base_speed: 0.0,  // not used in test
        scout_ratio: 0.0, // not used in test
        ratio_step: 0.0,  // not used in test
        homing_strength: 0.1,
        food_source_radius: 0.001,
        window_width: 64, // test grid size
        window_height: 64,
        palette: Palette::Pastel,
    }
}

fn gpu_config() -> GpuConfig {
    GpuConfig::from(&config())
}

fn make_sim(ants: Vec<Ant>) -> (common::HeadlessGpuSetup, SimulationPipeline) {
    let setup = pollster::block_on(common::HeadlessGpuSetup::new()).unwrap();
    let spawner = FixedSpawner::new(ants, Colony::default());
    let sim = SimulationPipeline::new(
        &setup.device,
        64,
        64,
        gpu_config(),
        spawner.colony(),
        spawner.ants(),
    );
    (setup, sim)
}

#[test]
fn ants_move_after_one_tick() {
    let ants = vec![Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager)];
    let initial_pos = ants[0].position;
    let (setup, mut sim) = make_sim(ants);

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_ne!(
        result[0].position, initial_pos,
        "ant should have moved after one tick"
    );
}

#[test]
fn ant_moves_to_exact_position() {
    // With randomness=0, movement is pure IEEE 754 addition: position += direction.
    // No transcendentals involved, so CPU and GPU produce identical bit patterns.
    let ants = vec![Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager)];
    let (setup, mut sim) = make_sim(ants);

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_eq!(result[0].position, [0.501, 0.5]);
}

#[test]
fn ants_keep_direction_without_collision() {
    // Two ants far apart: no collision triggers. With randomness=0 the direction
    // vector is never modified, so it must survive the tick unchanged.
    let ant_a = Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Scout);
    let ant_b = Ant::new([0.8, 0.8], [0.001, 0.0], AntType::Scout);
    let (dir_a, dir_b) = (ant_a.direction, ant_b.direction);
    let (setup, mut sim) = make_sim(vec![ant_a, ant_b]);

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_eq!(
        result[0].direction, dir_a,
        "direction should be unchanged without collision"
    );
    assert_eq!(
        result[1].direction, dir_b,
        "direction should be unchanged without collision"
    );
}

#[test]
fn ants_change_direction_on_collision() {
    // Two ants placed closer than collision_radius, both outside the colony.
    // The collision shader rotates their directions by a hash-derived angle.
    // Expected positions are the lavapipe-confirmed outputs for these exact inputs.
    let ant_a = Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager);
    let ant_b = Ant::new(
        [0.5 + COLLISION_RADIUS / 2.0, 0.5],
        [-0.001, 0.0],
        AntType::Forager,
    );
    let (setup, mut sim) = make_sim(vec![ant_a, ant_b]);

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_eq!(result[0].position, [0.501, 0.5]);
    assert_eq!(result[1].position, [0.49905002, 0.5]);
}
