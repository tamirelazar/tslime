//! Parameter explanations and help text for the application.
//!
//! This module provides detailed explanations of simulation parameters
//! that can be displayed to users via the `--explain` CLI flag.

/// Prints detailed explanations of all simulation parameters to stdout.
///
/// This output is intended for the `--explain` flag, providing users with
/// context on how each parameter affects the simulation behavior.
pub fn print_parameter_explanations() {
    println!("\n╔═══════════════════════════════════════════════════════════════════════╗");
    println!("║                    TSLIME PARAMETER REFERENCE                         ║");
    println!("╚═══════════════════════════════════════════════════════════════════════╝\n");

    println!("AGENT BEHAVIOR PARAMETERS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --sensor-angle <DEG> (default: 22.5)");
    println!("    Angle between left/right sensors in degrees.");
    println!("    • Smaller values (5-15°): Agents form tight, organized networks");
    println!("    • Medium values (20-30°): Balanced exploration and pattern formation");
    println!("    • Larger values (45-90°): Chaotic, exploratory behavior");
    println!("    Range: 5.0-90.0");

    println!("\n  --sensor-distance <FLOAT> (default: 9.0)");
    println!("    How far ahead agents can sense pheromones.");
    println!("    • Shorter distance (1-5): Agents follow trails closely, fine details");
    println!("    • Medium distance (6-12): Good balance of following and exploring");
    println!("    • Longer distance (15-50): Agents react to distant trails, broader patterns");
    println!("    Range: 1.0-50.0");

    println!("\n  --rotation-angle <DEG> (default: 45.0)");
    println!("    Maximum turn amount per step when changing direction.");
    println!("    • Small angles (5-20°): Smooth, flowing curves and tendrils");
    println!("    • Medium angles (30-50°): Mix of curves and sharp turns");
    println!("    • Large angles (60-90°): Sharp, angular patterns and quick direction changes");
    println!("    Range: 5.0-90.0");

    println!("\n  --step-size <FLOAT> (default: 1.0)");
    println!("    Distance agents move per simulation step.");
    println!("    • Slower movement (0.5-0.8): Dense, intricate patterns");
    println!("    • Normal movement (1.0): Balanced pattern development");
    println!("    • Faster movement (1.5-5.0): Loose, expansive patterns");
    println!("    Range: 0.5-5.0");

    println!("\n  --deposit <FLOAT> (default: 5.0)");
    println!("    Amount of pheromone deposited by agents per step.");
    println!("    • Low deposit (1-3): Faint trails, requires more agents to see patterns");
    println!("    • Medium deposit (4-6): Clear trails, good visibility");
    println!("    • High deposit (7-20): Very bright, intense trails");
    println!("    Range: 1.0-20.0");

    println!("\n\nTRAIL MAP PARAMETERS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --decay <FLOAT> (default: 0.5)");
    println!("    Trail persistence multiplier (applied each frame).");
    println!("    • Fast decay (0.5-0.7): Trails fade quickly, dynamic patterns");
    println!("    • Medium decay (0.8-0.9): Balanced trail persistence");
    println!("    • Slow decay (0.95-0.99): Trails persist long, accumulate over time");
    println!("    Range: 0.5-0.99");

    println!("\n  --diffusion-kernel <TYPE>");
    println!("    Algorithm used for pheromone spreading.");
    println!("    • mean3x3: Simple 3×3 averaging, sharp patterns");
    println!("    • gaussian: Smooth Gaussian blur, organic patterns");

    println!("\n  --diffusion-sigma <FLOAT>");
    println!("    Smoothness of Gaussian blur (only for gaussian kernel).");
    println!("    • Lower sigma (0.5-0.8): Less spreading, sharper details");
    println!("    • Higher sigma (1.0-2.0): More spreading, softer, blurred trails");
    println!("    Range: 0.5-2.0");

    println!("\n  --max-brightness <FLOAT> (default: 100.0)");
    println!("    Fixed maximum brightness for normalization.");
    println!("    • Lower values (1-10): High contrast, prevents flickering");
    println!("    • Medium values (20-50): Balanced brightness");
    println!("    • Higher values (75-100): More dynamic range, may flicker");
    println!("    Range: 1.0-100.0");

    println!("\n\nPOPULATION & INITIALIZATION");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  -n, --population <INT> (default: 50000)");
    println!("    Number of agents in the simulation.");
    println!("    • Small population (1k-20k): Sparse, individual trails visible");
    println!("    • Medium population (30k-70k): Good pattern density");
    println!("    • Large population (80k-200k): Dense, complex patterns");
    println!("    Range: 1000-200000");

    println!("\n  --init <MODE> (default: food)");
    println!("    Agent initialization pattern.");
    println!("    • random: Scattered throughout canvas");
    println!("    • central: Burst from center point");
    println!("    • circle: Ring around center");
    println!("    • gradient: Linear gradient distribution");
    println!("    • wave: Wave front from edge");
    println!("    • spiral: Spiral distribution");
    println!("    • clusters: Multiple random clusters");
    println!("    • food: Load from image (see --food)");

    #[cfg(feature = "multi-species")]
    {
        println!("\n  --species <SPEC>");
        println!("    Define multiple species with different behaviors.");
        println!("    Format: 'name:count@sensor_angle,rotation_angle,step_size,deposit:color'");
        println!(
            "    Example: --species 'red:20k@22.5,45,1.0,5.0:ff0000,blue:30k@30,60,1.5,3.0:0000ff'"
        );
        println!("    Enables multi-species simulations with distinct movement patterns.");
    }

    println!("\n\nENVIRONMENTAL FORCES");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --attract <X,Y,STRENGTH>");
    println!("    Add point attractors that pull/push agents.");
    println!("    • Positive strength: Attracts agents toward point");
    println!("    • Negative strength: Repels agents away from point");
    println!("    Example: --attract 200,200,1.0 --attract 400,300,-0.5");
    println!("    Range: -10.0 to 10.0 for strength");

    println!("\n  --wind <DX,DY>");
    println!("    Apply constant wind force.");
    println!("    Example: --wind 0.5,0.0 (rightward wind)");
    println!("    Each component range: -1.0 to 1.0");

    println!("\n  --obstacle <TYPE:PARAMS>");
    println!("    Add obstacles that agents bounce off.");
    println!("    • circle:x,y,radius - Circular obstacle");
    println!("    • rect:x,y,width,height - Rectangular obstacle");
    println!("    • image:path,x,y,w,h,invert,threshold - Image-based obstacle");

    println!("\n  --terrain <TYPE> (default: none)");
    println!("    Organic terrain effects on agent movement.");
    println!("    • none: No terrain effects");
    println!("    • smooth: Gentle Perlin noise flow fields");
    println!("    • turbulent: Chaotic turbulent patterns");
    println!("    • mixed: Combination of smooth and turbulent");

    println!("\n  --terrain-strength <FLOAT> (default: 1.0)");
    println!("    Intensity of terrain influence on movement.");
    println!("    Range: 0.1-5.0");

    println!("\n\nRENDERING & DISPLAY");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --palette <NAME> (default: moss)");
    println!("    Color scheme for rendering trails.");
    println!("    Available: organic, heat, ocean, mono, forest, neon, warm, vibrant,");
    println!("               legiblemono, slime, mold, fungus, swamp, moss, cosmic, ethereal");
    println!("    Or custom: \"#rrggbb,#rrggbb,...\" (2-11 colors)");

    println!("\n  --colors <MODE> (default: true)");
    println!("    Terminal color mode.");
    println!("    • 8: 8 colors (basic compatibility)");
    println!("    • 16: 16 colors");
    println!("    • 256: 256 colors");
    println!("    • true: 24-bit true color (16.7M colors)");

    println!("\n  --ascii, --braille, --quadrant");
    println!("    Character set for rendering.");
    println!("    • --ascii: ASCII characters only (widest compatibility)");
    println!("    • --braille: Braille Unicode characters (2× vertical resolution)");
    println!("    • --quadrant: Quadrant blocks (2×2 subpixels per cell)");

    println!("\n  --resolution <WxH> (default: 400x200)");
    println!("    Internal simulation grid size.");
    println!("    • Smaller (200×100): Faster, less detail");
    println!("    • Default (400×200): Good balance");
    println!("    • Larger (800×400): Slower, more detail");

    println!("\n  --dither-mode <MODE> (default: none) [dev-only]");
    println!("    Dithering algorithm for color quantization.");
    println!("    • none: No dithering");
    println!("    • ordered: Bayer matrix ordered dithering");
    println!("    • error-diffusion: Floyd-Steinberg error diffusion");
    println!("    • hybrid: Combination of ordered and error diffusion");

    println!("\n\nPERFORMANCE & TIMING");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --fps <INT> (default: 30)");
    println!("    Target frames per second for animation.");
    println!("    • Lower FPS (10-20): Slower animation, lower CPU usage");
    println!("    • Normal FPS (25-30): Smooth animation");
    println!("    • High FPS (60+): Very smooth, requires fast hardware");

    println!("\n  --time-scale <FLOAT> (default: 1.0)");
    println!("    Speed multiplier for simulation time.");
    println!("    • Slower (0.1-0.5): Slow-motion effect");
    println!("    • Normal (1.0): Real-time");
    println!("    • Faster (1.5-10.0): Fast-forward effect");
    println!("    Range: 0.1-10.0");

    println!("\n  --simd-off");
    println!("    Disable SIMD acceleration for diffusion.");
    println!("    Use scalar fallback (slower but more compatible).");

    println!("\n\nPRESETS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --preset <NAME>");
    println!("    Use pre-configured parameter combinations.");
    println!("    • network: Tight, organized networks with sharp edges");
    println!("    • exploratory: Chaotic, wide-ranging exploration");
    println!("    • tendrils: Long, flowing tendril-like patterns");
    println!("    • organic: Balanced, natural-looking patterns");
    println!("    • minimal: Sparse, minimalist aesthetic");
    println!("    • moss: Dense, moss-like growth patterns");
    println!("    • cosmic: Nebula-like, ethereal patterns");
    println!("    • fire: Intense, flame-like movement");
    println!("    • zen: Calm, meditative patterns");
    println!("    • storm: Turbulent, stormy patterns with wind");
    println!("    • river: Flowing, river-like patterns with directional wind");
    println!("    • ethereal: Delicate, ghostly patterns");

    println!("\n\nMODES OF OPERATION");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  -l, --live");
    println!("    Continuous animation mode (interactive).");

    println!("\n  -S, --screensaver");
    println!("    Screensaver mode - exits on any keypress.");

    println!("\n  -p, --print");
    println!("    Print a single frame and exit (non-interactive).");

    println!("\n  --export-gif <PATH>");
    println!("    Export animation to GIF file.");

    println!("\n  --export-webm <PATH>");
    println!("    Export animation to WebM video (requires FFmpeg).");

    println!("\n\nEXAMPLES");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  # Fast, tight networks");
    println!("  tslime --sensor-angle 15 --rotation-angle 30 --decay 0.85");

    println!("\n  # Slow, flowing tendrils");
    println!("  tslime --sensor-angle 30 --step-size 2.0 --decay 0.90");

    println!("\n  # Chaotic exploration");
    println!("  tslime --sensor-angle 45 --rotation-angle 60 --sensor-distance 15");

    #[cfg(feature = "multi-species")]
    {
        println!("\n  # Multi-species competition");
        println!("  tslime --species 'red:20k:ff0000,blue:20k:0000ff' --separate-species-trails");
    }

    println!("\n  # Wind-driven river pattern");
    println!("  tslime --preset river --wind 0.3,0.0");

    println!("\n  # High-res export");
    println!("  tslime --resolution 800x400 --export-gif output.gif --export-frames 100");

    println!("\n\nFor more information, visit: https://github.com/tamirelazar/tslime");
    println!();
}
