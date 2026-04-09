struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    emerged: u32,
}

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;

const COLLISION_RADIUS: f32 = 0.0001;
const COLLISION_ANGLE_MIN: f32 = 1.16937059884; // 67deg
const COLLISION_ANGLE_MAX: f32 = 1.95476876223; // 112deg

fn hash(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn random_collision_angle(pos: vec2<f32>) -> f32 {
    return COLLISION_ANGLE_MIN + hash(pos) * (COLLISION_ANGLE_MAX - COLLISION_ANGLE_MIN);
}

struct Colony {
    center: vec2<f32>,
    half_size: f32,
    _pad: f32,
}

@group(0) @binding(1) var<uniform> colony: Colony;

fn in_colony(pos: vec2<f32>) -> bool {
    let d = abs(pos - colony.center);
    return d.x < colony.half_size && d.y < colony.half_size;
}

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

@compute
@workgroup_size(64)
fn collision_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let count = arrayLength(&ants);
    if index >= count {
        return;
    }

    let ant = ants[index];
    let pos = ant.position;

    if in_colony(pos) {
        return;
    }

    for (var i: u32 = 0u; i < count; i++) {
        if i == index { continue; }
        if distance(pos, ants[i].position) < COLLISION_RADIUS {
            ants[index].direction = rotate(ant.direction, random_collision_angle(pos));
            break;
        }
    }
}

@compute
@workgroup_size(64)
fn movement_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= arrayLength(&ants) {
        return;
    }

    var ant = ants[index];
    let next = ant.position + ant.direction;
    if next.x < 0.0 || next.x > 1.0 { ant.direction.x = -ant.direction.x; }
    if next.y < 0.0 || next.y > 1.0 { ant.direction.y = -ant.direction.y; }

    ant.position = clamp(next, vec2<f32>(0.0), vec2<f32>(1.0));

    ants[index] = ant;
}
