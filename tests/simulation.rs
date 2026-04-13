mod common;

use gekkemieren::{
    ant::{Ant, AntType},
    pheromone::SimConfig,
    pipeline::SimulationPipeline,
    spawn::{AntSpawner, Colony, FixedSpawner},
};

fn sim_config() -> SimConfig {
    SimConfig {
        decay_amount: 1,
        max_strength: 1000,
        deposit_amount: 50,
        dot_radius: 0.001,
        collision_radius: 0.0001,
        collision_angle_min: 1.169_370_6,
        collision_angle_max: 1.954_768_8,
        forager_randomness: 0.0,
        scout_randomness: 0.0,
        _pad: [0; 3],
    }
}

#[test]
fn ants_move_after_one_tick() {
    let setup = pollster::block_on(common::HeadlessGpuSetup::new()).unwrap();

    let colony = Colony::default();
    let ants = vec![Ant::new([0.5, 0.5], [0.001, 0.0], AntType::Forager)];
    let initial_pos = ants[0].position;

    let spawner = FixedSpawner::new(ants.clone(), colony);
    let mut sim = SimulationPipeline::new(
        &setup.device,
        64,
        64,
        sim_config(),
        spawner.colony(),
        spawner.ants(),
    );

    sim.update(&setup.device, &setup.queue);
    let result = sim.read_ant_state(&setup.device, &setup.queue);

    assert_ne!(
        result[0].position, initial_pos,
        "ant should have moved after one tick"
    );
}
