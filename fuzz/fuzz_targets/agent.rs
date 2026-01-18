#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tslime::simulation::agent::Agent;

#[derive(Arbitrary, Debug)]
struct AgentInput {
    x: f32,
    y: f32,
    heading: f32,
    step_size: f32,
    sensor_angle: f32,
    sensor_distance: f32,
    rotation_angle: f32,
    map_width: u8,
    map_height: u8,
    trail_values: Vec<f32>,
    rng_seed: u64,
}

fuzz_target!(|data: AgentInput| {
    // Sanitize inputs
    let width = (data.map_width as usize % 64) + 10;
    let height = (data.map_height as usize % 64) + 10;

    // Ensure x, y are within some reasonable bound or even out of bounds to test boundary logic
    let x = if data.x.is_finite() { data.x } else { 0.0 };
    let y = if data.y.is_finite() { data.y } else { 0.0 };
    let heading = if data.heading.is_finite() {
        data.heading
    } else {
        0.0
    };

    let mut agent = Agent::new(x, y, heading, 0);

    // Setup trail map
    let mut trail = vec![0.0; width * height];
    for (i, &val) in data.trail_values.iter().take(width * height).enumerate() {
        if val.is_finite() {
            trail[i] = val;
        }
    }

    let sensor_angle = if data.sensor_angle.is_finite() {
        data.sensor_angle
    } else {
        45.0
    };
    let sensor_distance = if data.sensor_distance.is_finite() {
        data.sensor_distance
    } else {
        10.0
    };

    // 1. Sense
    let (left, center, right) = agent.sense(&trail, width, height, sensor_angle, sensor_distance);

    // Check sense results are finite
    assert!(left.is_finite());
    assert!(center.is_finite());
    assert!(right.is_finite());

    // 2. Rotate
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(data.rng_seed);
    let rotation_angle = if data.rotation_angle.is_finite() {
        data.rotation_angle
    } else {
        45.0
    };

    agent.rotate(left, center, right, rotation_angle, &mut rng);
    assert!(agent.heading.is_finite());

    // 3. Move
    let step_size = if data.step_size.is_finite() {
        data.step_size
    } else {
        1.0
    };
    // Pass empty obstacles for now as constructing them via Arbitrary is more complex and Agent::move_forward handles them safely
    agent.move_forward(step_size, width, height, &[], &[]);

    assert!(agent.x.is_finite());
    assert!(agent.y.is_finite());
    assert!(agent.heading.is_finite());

    // Verify boundary constraints
    assert!(agent.x >= 0.0 && agent.x <= (width - 1) as f32);
    assert!(agent.y >= 0.0 && agent.y <= (height - 1) as f32);

    // 4. Deposit
    agent.deposit(&mut trail, width, height, 1.0);
    // Just ensure no panic
});
