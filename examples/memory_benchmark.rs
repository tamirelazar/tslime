use memory_stats::memory_stats;
use std::mem::size_of;
use tslime::simulation::agent::Agent;
use tslime::simulation::config::{InitMode, SimConfig, SpeciesConfig};
use tslime::simulation::trail_map::TrailMap;
use tslime::simulation::Simulation;

fn main() {
    println!("=== Memory Benchmark ===\n");

    // Test 1: Agent struct size
    println!("Agent struct size: {} bytes", size_of::<Agent>());
    println!(
        "100k agents would use: {:.2} MB",
        (size_of::<Agent>() * 100_000) as f64 / 1_048_576.0
    );
    println!();

    // Test 2: TrailMap size
    let width = 400;
    let height = 400;
    let _trail_map = TrailMap::new(width, height);
    let trail_map_size = size_of::<TrailMap>()
        + (width * height * size_of::<f32>()) * 2 // current + scratch buffers
        + size_of::<f32>() * 25; // gaussian kernel

    println!(
        "TrailMap ({}x{}) estimated size: {:.2} MB",
        width,
        height,
        trail_map_size as f64 / 1_048_576.0
    );
    println!("  - Header: {} bytes", size_of::<TrailMap>());
    println!(
        "  - Current buffer: {:.2} MB",
        (width * height * size_of::<f32>()) as f64 / 1_048_576.0
    );
    println!(
        "  - Scratch buffer: {:.2} MB",
        (width * height * size_of::<f32>()) as f64 / 1_048_576.0
    );
    println!("  - Gaussian kernel: {} bytes", size_of::<f32>() * 25);
    println!();

    // Test 3: Full simulation memory usage
    println!("Testing full simulation memory usage...");

    if let Some(usage_before) = memory_stats() {
        println!(
            "Memory before simulation: {:.2} MB",
            usage_before.physical_mem as f64 / 1_048_576.0
        );

        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: 100_000,
                ..Default::default()
            }],
            ..Default::default()
        };

        let sim = Simulation::new(width, height, config, 42, InitMode::Random, 0);

        if let Some(usage_after) = memory_stats() {
            println!(
                "Memory after simulation: {:.2} MB",
                usage_after.physical_mem as f64 / 1_048_576.0
            );
            let delta = (usage_after.physical_mem as i64 - usage_before.physical_mem as i64) as f64
                / 1_048_576.0;
            println!("Memory increase: {:.2} MB", delta);
            println!();

            // Theoretical memory usage
            let agents_mem = size_of::<Agent>() * 100_000;
            let trail_mem = trail_map_size;
            let theoretical = (agents_mem + trail_mem) as f64 / 1_048_576.0;

            println!("Theoretical minimum: {:.2} MB", theoretical);
            println!(
                "  - Agents (100k): {:.2} MB",
                agents_mem as f64 / 1_048_576.0
            );
            println!("  - TrailMap: {:.2} MB", trail_mem as f64 / 1_048_576.0);
            println!();

            let overhead = delta - theoretical;
            println!(
                "Overhead: {:.2} MB ({:.1}%)",
                overhead,
                (overhead / theoretical) * 100.0
            );

            // Keep sim alive
            println!("\nSimulation stats:");
            println!("  - Agent count: {}", sim.agent_count());
            println!("  - Dimensions: {}x{}", sim.width(), sim.height());
        }
    } else {
        println!("Failed to get memory stats");
    }

    println!("\n=== Analysis ===");
    println!(
        "Agent struct is already optimal at {} bytes (16 bytes = 4 floats + 1 byte + padding)",
        size_of::<Agent>()
    );
    println!("Main memory consumers:");
    println!(
        "  1. TrailMap f32 buffers: 2 × {}×{} × 4 bytes = {:.2} MB",
        width,
        height,
        (width * height * 2 * 4) as f64 / 1_048_576.0
    );
    println!(
        "  2. Agents Vec: 100k × {} bytes = {:.2} MB",
        size_of::<Agent>(),
        (100_000 * size_of::<Agent>()) as f64 / 1_048_576.0
    );
    println!("\nOptimization opportunities:");
    println!("  - Use f16 (half precision) for trail maps where precision allows");
    println!("  - Consider arena allocator for agents (reduce allocation overhead)");
    println!("  - Profile actual runtime to find hidden allocations");
}
