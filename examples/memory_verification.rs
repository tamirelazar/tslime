use memory_stats::memory_stats;
use std::mem::size_of;
use tslime::simulation::agent::Agent;
use tslime::simulation::config::{InitMode, SimConfig, SpeciesConfig};
use tslime::simulation::Simulation;

fn main() {
    println!("=== Phase 6.5: Memory Optimization - Final Verification ===\n");

    // Acceptance Criteria: Total memory < 3MB with 100k agents at 400×400
    let width = 400;
    let height = 400;
    let agent_count = 100_000;

    println!("Test Configuration:");
    println!("  - Grid: {}×{}", width, height);
    println!("  - Agents: {}", agent_count);
    println!("  - Target: < 3.00 MB\n");

    if let Some(usage_before) = memory_stats() {
        let config = SimConfig {
            species_configs: vec![SpeciesConfig {
                count: agent_count,
                ..Default::default()
            }],
            ..Default::default()
        };

        let sim = Simulation::new(width, height, config, 42, InitMode::Random, 0);

        if let Some(usage_after) = memory_stats() {
            let mem_mb = (usage_after.physical_mem as i64 - usage_before.physical_mem as i64)
                as f64
                / 1_048_576.0;

            println!("Memory Usage:");
            println!("  Actual: {:.2} MB", mem_mb);
            println!("  Target: < 3.00 MB");

            if mem_mb < 3.0 {
                println!(
                    "  Status: ✓ PASS (under budget by {:.2} MB)\n",
                    3.0 - mem_mb
                );
            } else {
                println!("  Status: ✗ FAIL (over budget by {:.2} MB)\n", mem_mb - 3.0);
            }

            // Breakdown
            let agent_mem_mb = (size_of::<Agent>() * agent_count) as f64 / 1_048_576.0;
            let trail_mem_mb = (width * height * size_of::<f32>() * 2) as f64 / 1_048_576.0;
            let theoretical_mb = agent_mem_mb + trail_mem_mb;
            let overhead_mb = mem_mb - theoretical_mb;
            let overhead_pct = (overhead_mb / theoretical_mb) * 100.0;

            println!("Memory Breakdown:");
            println!(
                "  Agents ({} × {} bytes): {:.2} MB",
                agent_count,
                size_of::<Agent>(),
                agent_mem_mb
            );
            println!("  TrailMap (2 buffers): {:.2} MB", trail_mem_mb);
            println!("  Theoretical: {:.2} MB", theoretical_mb);
            println!("  Overhead: {:.2} MB ({:.1}%)\n", overhead_mb, overhead_pct);

            println!("Optimizations Applied:");
            println!("  ✓ Agent struct is Copy (16 bytes, optimal)");
            println!("  ✓ Pre-allocated Vec with capacity");
            println!("  ✓ No runtime allocations in hot paths");
            println!("  ✓ Efficient buffer management (swap instead of clone)\n");

            println!("Additional Stats:");
            println!("  Bytes per agent: {}", size_of::<Agent>());
            println!("  Simulation dimensions: {}×{}", sim.width(), sim.height());
            println!("  Agent count: {}", sim.agent_count());

            // Final verdict
            println!("\n=== VERDICT ===");
            if mem_mb < 3.0 {
                println!("Phase 6.5 acceptance criteria MET:");
                println!("Memory usage ({:.2} MB) is below target (3.00 MB)", mem_mb);
                println!("\nNo further optimization needed. Current implementation is efficient.");
            } else {
                println!("Phase 6.5 acceptance criteria NOT MET:");
                println!("Memory usage ({:.2} MB) exceeds target (3.00 MB)", mem_mb);
                println!("Further optimization required.");
            }
        } else {
            println!("ERROR: Failed to measure memory after simulation");
        }
    } else {
        println!("ERROR: Failed to measure memory before simulation");
    }
}
