mod common;

use gekkemieren::{
    ant::{Ant, AntType},
    color_scheme::Palette,
    config::{Config, GpuConfig},
    pipeline::SimulationPipeline,
    spawn::{AntSpawner, Colony, FixedSpawner},
};

fn config() -> Config {
    Config {
        decay_ratio: 0.1,
        diffusion_ratio: 0.0,
        deposit_ratio: 0.001,
        carrying_deposit_boost: 100.0,
        // Zero randomness makes movement fully deterministic: direction is
        forager_randomness: 0.0,
        scout_randomness: 0.0,
        sensor_distance: 0.03,
        sensor_angle: 0.5,
        compass_turn_rate: 0.55,
        n_ants: 0,           // not used in test
        ant_lifetime: 10000, // drives the colony food economy: hungry at 50%
        base_speed: 0.0,     // not used in test
        scout_ratio: 0.0,    // not used in test
        ratio_step: 0.0,     // not used in test
        food_source_center: [0.9, 0.0],
        food_source_radius: 0.001,
        colony_center: [0.5, 0.5],
        colony_radius: 0.01,
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
    // Create config with colony settings matching the spawner's colony
    let mut config = gpu_config();
    config.colony_center = [0.5, 0.5];
    config.colony_radius = 0.01;
    let colony = Colony::new(config.colony_center, config.colony_radius);
    let spawner = FixedSpawner::new(ants, colony);

    let sim = SimulationPipeline::new(&setup.device, 64, 64, config, spawner.ants());
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

    assert_eq!(result[0].position, [0.5 + 0.001, 0.5]);
}

#[test]
fn carrying_ant_deposits_food_units_into_colony_store() {
    let mut ant = Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager);
    ant.carries_food = 1;
    let (setup, mut sim) = make_sim(vec![ant]);

    sim.update(&setup.device, &setup.queue);

    assert_eq!(
        sim.read_colony_food(&setup.device, &setup.queue),
        10,
        "a delivered food pixel should add 10 units to the colony store"
    );
}

#[test]
fn hungry_ant_waits_in_colony_and_leaks_no_food() {
    let mut ant = Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager);
    ant.lifetime = 8000;
    let (setup, mut sim) = make_sim(vec![ant]);

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_eq!(
        result[0].position,
        [0.5, 0.5],
        "a hungry ant without food in the colony should stay put"
    );
    assert_eq!(
        sim.read_colony_food(&setup.device, &setup.queue),
        0,
        "a failed meal attempt must not leak food units"
    );
}

#[test]
fn dying_ant_eats_from_colony_store_and_revives() {
    let mut ant = Ant::new([0.25, 0.25], [0.001, 0.0], AntType::Forager);
    ant.lifetime = 15000;
    let (setup, mut sim) = make_sim(vec![ant]);
    setup
        .queue
        .write_buffer(&sim.colony_food_buffer, 0, bytemuck::bytes_of(&5i32));

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    let dx = result[0].position[0] - 0.5;
    let dy = result[0].position[1] - 0.5;
    assert!(
        (dx * dx + dy * dy).sqrt() < 0.012,
        "an exhausted ant should be fed in the colony, not shuttling out"
    );
    assert_eq!(
        result[0].lifetime, 0,
        "a dying ant with colony food must eat and reset its lifetime"
    );
    assert_eq!(
        sim.read_colony_food(&setup.device, &setup.queue),
        4,
        "the revived ant should have consumed exactly one meal"
    );
}

#[test]
fn dying_carrier_drops_food_at_death_spot() {
    let mut ant = Ant::new([0.25, 0.25], [0.001, 0.0], AntType::Forager);
    ant.carries_food = 1;
    ant.lifetime = 15000;
    let (setup, mut sim) = make_sim(vec![ant]);

    sim.update(&setup.device, &setup.queue);
    let food = sim.read_food_state(&setup.device, &setup.queue);

    let cell = (0.25 * 64.0) as usize;
    assert_eq!(
        food[cell * 64 + cell],
        1,
        "a dying carrier should drop its food pixel where it died"
    );
    assert_eq!(
        sim.read_colony_food(&setup.device, &setup.queue),
        0,
        "dropped food must not enter the colony store"
    );
}
