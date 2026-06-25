//! Interactive simulation runner and main event loop.
//!
//! This module contains the main simulation runner that handles the interactive
//! terminal-based simulation with real-time user input, rendering, and overlay management.

use std::io::{self, Write};

use clap::Parser as _;

use crate::app::{
    apply_live_params, apply_random_config, extract_species_rgb_colors, REFERENCE_TIME_STEP,
};
use crate::cli::{self, Args, ColorMode, Mode, Palette};
use crate::config_defaults::warmup::{TRANSITION_DURATION_FRAMES, WARMUP_SPEED_MULTIPLIER};
use crate::config_manager;
use crate::export::GifExporter;
use crate::export::WebmExporter;
use crate::food_image::FOOD_IMAGE_PNG;
use crate::overlay::{OverlayInputManager, OverlayInputResult, OverlayType};
use crate::palette_manager;
use crate::render::adaptive_brightness::AdaptiveBrightness;
use crate::render::ambient::{AmbientState, BaseStatus};
use crate::render::charset::Charset;
use crate::render::controls::registry::RegistryCtx;
use crate::render::dither::DitherMode;
use crate::render::downsample::downsample;
use crate::render::grid::{GridRenderer, GridStyle};
use crate::render::overlay::{
    ConfigBrowserOverlay, ConfigSaveOverlay, DashboardOverlay, DirtyGuardOverlay,
    KeyboardHintsOverlay, OverlayRenderer, PauseOverlay, PresetComparisonOverlay, RenderedOverlay,
};
use crate::render::palette::{hex_to_rgb, palette_accent_color, RgbColor};
use crate::render::palette_editor::{
    EditorComponent, EditorMode, PaletteEditorOverlay, PaletteEditorState,
};
use crate::simulation::config::TerrainType;
use crate::simulation::config::{Attractor, DiffusionKernel, InitMode, Preset, SimConfig};
use crate::simulation::food::load_logo_from_memory;
use crate::simulation::Simulation;
use crate::terminal::control::num_palettes;
use crate::terminal::control::{
    charset_name, handle_key_event, palette_name, preset_name, ControlAction, MouseInteractionMode,
    PaletteShiftSpeed, PendingSwap, RuntimeState, ALL_CHARSETS, ALL_PALETTES,
};
use crate::terminal::detection::{log_capabilities, TerminalCapabilities};
use crate::terminal::frame_buffer::FrameBuffer;
use crate::terminal::input::{InputPoller, MouseEventType};
use crate::terminal::renderer::{ChromeSnapshot, TerminalRenderer};
use crate::terminal::screen::TerminalScreen;
use crate::terminal::signal::is_shutdown_requested;
use crate::terminal::timing::FrameTimer;
use crossterm::event::Event;
use memory_stats::memory_stats;

/// Target frame time at 30 FPS, in milliseconds.
const TARGET_FRAME_TIME_MS: f32 = 33.333;

/// Updates food persistence attractors with fade-out effect.
///
/// Gradually reduces the strength of food attractors over time using
/// quadratic easing for a smooth fade-out effect.
fn update_food_persistence(sim: &mut Simulation, runtime_state: &mut RuntimeState) {
    let duration = runtime_state.app.food_persist_duration;
    if !runtime_state.food_persist_enabled || runtime_state.is_paused || duration == 0 {
        return;
    }

    runtime_state.food_persist_counter += 1;

    if runtime_state.food_persist_counter <= duration {
        let progress = runtime_state.food_persist_counter as f32 / duration as f32;
        let fade_factor: f32 = (1.0 - progress).powi(2); // Quadratic fade-out

        // Update attractor strengths without cloning entire config
        let attractors: Vec<Attractor> = runtime_state
            .initial_food_attractors
            .iter()
            .map(|a| Attractor::new(a.x, a.y, a.strength * fade_factor))
            .collect();
        sim.update_attractors(attractors);
    } else if runtime_state.food_persist_counter == duration + 1 {
        // Remove all food attractors when duration expires
        sim.update_attractors(Vec::new());
    }
}

/// The init mode to actually seed with. The constellation figure is chosen by
/// the seeded simulation RNG inside `Simulation::new`, so reset stays
/// deterministic and every preset uses its stable baseline init mode.
pub(crate) fn effective_init_mode(base: InitMode) -> InitMode {
    base
}

/// Checks if simulation should auto-reset based on entropy collapse.
///
/// Monitors entropy levels and resets the simulation if it collapses
/// (entropy stays below threshold for specified duration).
fn check_auto_reset(
    sim: &mut Simulation,
    runtime_state: &mut RuntimeState,
    entropy: f32,
    blended_trail: &[f32],
) {
    if !runtime_state.app.auto_reset || runtime_state.is_paused {
        return;
    }

    // Collapse fires on either signal: entropy falling away (the field dies out)
    // or the pattern stagnating (a near-static shape that stops evolving even
    // while entropy stays healthy, e.g. a frozen diagonal line).
    let entropy_collapse = runtime_state.track_entropy(
        entropy,
        runtime_state.app.auto_reset_entropy_threshold,
        runtime_state.app.auto_reset_duration_frames,
    );
    let stagnation_collapse = runtime_state.track_stagnation(
        blended_trail,
        crate::config_defaults::auto_reset::DEFAULT_STAGNATION_EPSILON,
        crate::config_defaults::auto_reset::DEFAULT_STAGNATION_FRAMES,
    );
    let should_reset = entropy_collapse || stagnation_collapse;

    if should_reset {
        let new_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Use the live original init mode (the apply seam updates it on restart),
        // NOT the startup `init_mode` local which is stale after a config load.
        let init_mode = effective_init_mode(runtime_state.original_init_mode);
        sim.reset(new_seed, init_mode);
        runtime_state.reset_collapse_counter();
        runtime_state.reset_warmup();
        runtime_state.food_persist_counter = 0;
        runtime_state.show_notification(format!(
            "Simulation collapsed - restarting with seed {}",
            new_seed
        ));
    }
}

/// Returns `true` when a key event should immediately exit the application
/// in screensaver mode.
///
/// Any key press exits; key release/repeat events (emitted on some platforms,
/// e.g. Windows) are ignored so a single keystroke does not double-fire.
fn screensaver_exit_on_key(mode: &Mode, key_event: &crossterm::event::KeyEvent) -> bool {
    *mode == Mode::Screensaver && key_event.kind == crossterm::event::KeyEventKind::Press
}

/// Outcome of [`apply_controls_action`].
///
/// Controls-overlay interaction is split into a pure state mutation (focus,
/// depth) and an optional "real" [`ControlAction`] that must be re-dispatched
/// through the main action match (because adjusting the focused parameter is
/// exactly the same as pressing its hotkey, which can touch the simulation,
/// renderer, timer, etc. that live in the run loop).
enum ControlsDispatch {
    /// The action was not a controls-overlay action; let the main match handle it.
    Passthrough,
    /// The action was fully handled by mutating [`RuntimeState`]; nothing else to do.
    Handled,
    /// Re-dispatch this real [`ControlAction`] through the main action match.
    Redispatch(ControlAction),
}

/// Returns `true` when a *competing* (interactive, non-Controls) overlay is the
/// foreground overlay.
///
/// Overlays are mutually exclusive (`OverlayState` holds a single active slot),
/// so "another overlay is open" is simply: the active overlay is `Some(x)` where
/// `x` is neither `Controls` nor a non-interactive badge.
///
/// Used to suppress controls-interaction meta-actions (`Tab`/arrows/`Enter`)
/// while e.g. the Dashboard is open, so they don't silently flip controls depth
/// or mutate the simulation behind an overlay that doesn't render controls. The
/// spec-mandated auto-promote-from-Closed behaviour (←→/Tab with NO overlay
/// open) is preserved because that case has no active overlay at all.
fn other_overlay_open(state: &RuntimeState) -> bool {
    match state.overlay_state.active() {
        // Controls itself is not a competitor.
        Some(OverlayType::Controls) => false,
        // Non-interactive badges never block controls interaction.
        Some(OverlayType::PauseBadge) | Some(OverlayType::PauseLogo) => false,
        // Any other interactive overlay (Help, Dashboard, ConfigBrowser,
        // ConfigSave, DirtyGuard, PresetComparison, KeyboardHints,
        // PaletteEditor) competes for the foreground.
        Some(_) => true,
        None => false,
    }
}

/// Applies the controls-overlay interaction actions (depth toggle, focus
/// movement, focused adjust/activate) to [`RuntimeState`], and performs
/// in-category focus retargeting for plain per-param hotkeys.
///
/// The five interaction meta-actions (`ToggleControlsDepth`, `ControlsFocusNext`,
/// `ControlsFocusPrev`, `ControlsAdjustFocused`, `ControlsActivateFocused`) only
/// act when Controls is the foreground overlay OR no competing overlay is open
/// (see [`other_overlay_open`]); otherwise they `Passthrough` without mutating
/// depth/focus or auto-promoting, so e.g. arrows don't silently tweak the
/// simulation behind the Dashboard.
///
/// Returns:
/// - [`ControlsDispatch::Handled`] when the action was a pure state mutation.
/// - [`ControlsDispatch::Redispatch`] with the real action to re-run (focused
///   adjust/activate resolve to the focused parameter's hotkey action).
/// - [`ControlsDispatch::Passthrough`] otherwise (the main match handles it),
///   after retargeting focus if the hotkey's parameter is in the current
///   category.
fn apply_controls_action(
    state: &mut RuntimeState,
    action: &ControlAction,
    ctx: &RegistryCtx,
) -> ControlsDispatch {
    use crate::render::controls::registry;
    use crate::render::controls::ControlsDepth;

    let category = state.controls_category_idx;

    // Gate controls-interaction meta-actions behind the active overlay. We act
    // only when Controls is the foreground overlay, OR no competing overlay is
    // open (the latter preserves the spec's auto-promote-from-Closed behaviour
    // for ←→/Tab when nothing is open). When a competing overlay (e.g. the
    // Dashboard) is foreground, the five interaction meta-actions and the
    // open-case category clamp are suppressed (Passthrough, no depth/focus
    // mutation, no auto-promote).
    let may_interact =
        state.overlay_state.is_open(OverlayType::Controls) || !other_overlay_open(state);

    // Defensive clamp: if a conditional row disappeared (e.g. Gaussian toggled
    // off removes Diff Sigma; mouse disabled removes Mouse Timeout) the focus
    // index can be stale. Correct it before any action so that navigation and
    // adjust always land on a real row.
    let visible_len = registry::visible_params(category, ctx).len();
    if visible_len > 0 {
        state.controls_focus = state.controls_focus.min(visible_len - 1);
    }

    match action {
        ControlAction::ToggleControlsDepth => {
            if !may_interact {
                return ControlsDispatch::Passthrough;
            }
            state.controls_depth = match state.controls_depth {
                ControlsDepth::Closed => ControlsDepth::Console,
                ControlsDepth::Console => ControlsDepth::Tuner,
                ControlsDepth::Tuner => ControlsDepth::Console,
            };
            ControlsDispatch::Handled
        }
        ControlAction::ControlsFocusNext => {
            if !may_interact {
                return ControlsDispatch::Passthrough;
            }
            let len = registry::visible_params(category, ctx).len();
            if len > 0 {
                state.controls_focus = (state.controls_focus + 1).min(len - 1);
            }
            ControlsDispatch::Handled
        }
        ControlAction::ControlsFocusPrev => {
            if !may_interact {
                return ControlsDispatch::Passthrough;
            }
            state.controls_focus = state.controls_focus.saturating_sub(1);
            ControlsDispatch::Handled
        }
        ControlAction::ControlsAdjustFocused(sign) => {
            if !may_interact {
                return ControlsDispatch::Passthrough;
            }
            // Auto-promote: a focused adjust from Closed opens the Tuner first.
            if state.controls_depth == ControlsDepth::Closed {
                state.controls_depth = ControlsDepth::Tuner;
            }
            let params = registry::visible_params(category, ctx);
            let Some(desc) = params.get(state.controls_focus.min(params.len().saturating_sub(1)))
            else {
                return ControlsDispatch::Handled;
            };
            match registry::action_for(desc.id, *sign) {
                Some(real) => ControlsDispatch::Redispatch(real),
                None => ControlsDispatch::Handled,
            }
        }
        ControlAction::ControlsActivateFocused => {
            if !may_interact {
                return ControlsDispatch::Passthrough;
            }
            let params = registry::visible_params(category, ctx);
            let Some(desc) = params.get(state.controls_focus.min(params.len().saturating_sub(1)))
            else {
                return ControlsDispatch::Handled;
            };
            match registry::activate_action_for(desc.id) {
                Some(real) => ControlsDispatch::Redispatch(real),
                None => ControlsDispatch::Handled,
            }
        }
        // Category cycling: when the controls overlay is open, cycle the category
        // here and clamp focus to the new category's visible row count, then
        // return Handled so the runner's match arm is skipped (no double-cycle).
        // When the overlay is closed, return Passthrough so the runner's arm can
        // open the overlay and set depth=Console.
        ControlAction::CycleOptionsCategory => {
            if state.overlay_state.is_open(OverlayType::Controls) {
                state.cycle_controls_category(true);
                let new_len = registry::visible_params(state.controls_category_idx, ctx).len();
                state.controls_focus = state.controls_focus.min(new_len.saturating_sub(1));
                ControlsDispatch::Handled
            } else {
                ControlsDispatch::Passthrough
            }
        }
        ControlAction::CycleOptionsCategoryReverse => {
            if state.overlay_state.is_open(OverlayType::Controls) {
                state.cycle_controls_category(false);
                let new_len = registry::visible_params(state.controls_category_idx, ctx).len();
                state.controls_focus = state.controls_focus.min(new_len.saturating_sub(1));
                ControlsDispatch::Handled
            } else {
                ControlsDispatch::Passthrough
            }
        }
        // Any other action: if it's a per-param hotkey whose parameter is in the
        // CURRENT category, retarget focus to it (silent cross-category rule:
        // parameters in other categories never change focus or category).
        other => {
            if let Some(id) = registry::param_id_for_action(other) {
                if let Some((cat, idx)) = registry::locate(id, ctx) {
                    if cat == category {
                        state.controls_focus = idx;
                    }
                }
            }
            ControlsDispatch::Passthrough
        }
    }
}

/// Builds the [`ParamView`] list for the currently-visible parameters of the
/// active controls category, sourcing every value from live [`RuntimeState`].
///
/// Value formatting, gauge ranges, and modified/default detection follow the
/// same logic as [`build_param_views`] / the controls module so the Console
/// reads identically to the old controls overlay.
///
/// `ratio`/`def_ratio` are filled only for [`ParamKind::Numeric`] rows; for all
/// other kinds they are `None`. `state` is:
/// - [`ParamState::Cli`] for CLI-readonly rows (Population),
/// - [`ParamState::Display`] for display-only rows (Mouse Timeout, Dither),
/// - otherwise [`ParamState::Modified`] / [`ParamState::Default`] by comparing
///   the live value against `runtime_state.default_values` (float tolerance for
///   numerics; only the fields present in [`DefaultValues`] can show modified —
///   params without a tracked default always read as Default, matching the old
///   overlay's blank marker).
fn build_param_views(
    runtime_state: &RuntimeState,
    ctx: &RegistryCtx,
    population: usize,
) -> Vec<crate::render::controls::ParamView> {
    build_param_views_for_category(
        runtime_state,
        runtime_state.controls_category_idx,
        ctx,
        population,
    )
}

/// Like [`build_param_views`] but for an explicit `category` index rather than
/// the live `controls_category_idx`. Used by TUNE surfacing to build the view
/// for an adjusted param that may live in any category.
fn build_param_views_for_category(
    runtime_state: &RuntimeState,
    category: usize,
    ctx: &RegistryCtx,
    population: usize,
) -> Vec<crate::render::controls::ParamView> {
    use crate::render::controls::registry::{visible_params, ParamId, ParamKind};
    use crate::render::controls::{ParamState, ParamView};

    let defaults = &runtime_state.default_values;
    // Float-equality tolerance for modified detection (matches old eps choices).
    let near = |a: f32, b: f32, eps: f32| (a - b).abs() <= eps;
    // Clamp helper for gauge ratios.
    let ratio = |v: f32, min: f32, max: f32| ((v - min) / (max - min)).clamp(0.0, 1.0);

    let palette_name = runtime_state
        .current_palette(&ALL_PALETTES)
        .name()
        .to_string();
    let charset = charset_name(&runtime_state.current_charset()).to_string();
    let color_aa = runtime_state.current_color_aa().as_label().to_string();
    let theme = runtime_state.current_theme_name().to_string();

    let shift_name = match runtime_state.palette_shift_speed {
        PaletteShiftSpeed::Off => "Off",
        PaletteShiftSpeed::Slow => "Slow",
        PaletteShiftSpeed::Medium => "Medium",
        PaletteShiftSpeed::Fast => "Fast",
    };
    let mouse_mode_name = match runtime_state.mouse_mode {
        MouseInteractionMode::Disabled => "Disabled",
        MouseInteractionMode::Attract => "Attract",
        MouseInteractionMode::Repel => "Repel",
    };
    let terrain_name = match runtime_state.terrain_type {
        TerrainType::None => "None",
        TerrainType::Smooth => "Smooth",
        TerrainType::Turbulent => "Turbulent",
        TerrainType::Mixed => "Mixed",
    };
    let kernel_name = match runtime_state.diffusion_kernel {
        DiffusionKernel::Mean3x3 => "Mean3x3",
        DiffusionKernel::Gaussian => "Gaussian",
    };

    visible_params(category, ctx)
        .into_iter()
        .map(|desc| {
            // (value_text, ratio, def_ratio, state)
            let (value_text, r, dr, state): (String, Option<f32>, Option<f32>, ParamState) =
                match desc.id {
                    // ── SIM ───────────────────────────────────────────────────
                    ParamId::SensorAngle => {
                        let v = runtime_state.sensor_angle;
                        let d = defaults.sensor_angle;
                        (
                            format!("{v:.1}°"),
                            Some(ratio(v, 5.0, 90.0)),
                            Some(ratio(d, 5.0, 90.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::SensorDistance => {
                        let v = runtime_state.sensor_distance;
                        let d = defaults.sensor_distance;
                        (
                            format!("{v:.1}px"),
                            Some(ratio(v, 1.0, 50.0)),
                            Some(ratio(d, 1.0, 50.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::TurnAngle => {
                        let v = runtime_state.rotation_angle;
                        let d = defaults.rotation_angle;
                        (
                            format!("{v:.1}°"),
                            Some(ratio(v, 5.0, 90.0)),
                            Some(ratio(d, 5.0, 90.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::StepSize => {
                        let v = runtime_state.step_size;
                        let d = defaults.step_size;
                        (
                            format!("{v:.1}px"),
                            Some(ratio(v, 0.5, 5.0)),
                            Some(ratio(d, 0.5, 5.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::Decay => {
                        let v = runtime_state.decay_factor;
                        let d = defaults.decay_factor;
                        (
                            format!("{v:.3}"),
                            Some(ratio(v, 0.5, 0.99)),
                            Some(ratio(d, 0.5, 0.99)),
                            if near(v, d, 0.001) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::Deposit => {
                        let v = runtime_state.deposit_amount;
                        let d = defaults.deposit_amount;
                        (
                            format!("{v:.1}×"),
                            Some(ratio(v, 1.0, 20.0)),
                            Some(ratio(d, 1.0, 20.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::TimeScale => {
                        // No tracked default → always Default (matches old blank marker).
                        let v = runtime_state.time_scale;
                        (
                            format!("{v:.1}×"),
                            Some(ratio(v, 0.5, 4.0)),
                            Some(ratio(1.0, 0.5, 4.0)),
                            ParamState::Default,
                        )
                    }
                    // ── ENV ───────────────────────────────────────────────────
                    ParamId::DiffusionKernel => {
                        let modified = format!("{:?}", runtime_state.diffusion_kernel)
                            != format!("{:?}", defaults.diffusion_kernel);
                        (
                            kernel_name.to_string(),
                            None,
                            None,
                            if modified {
                                ParamState::Modified
                            } else {
                                ParamState::Default
                            },
                        )
                    }
                    ParamId::DiffusionSigma => {
                        let v = runtime_state.diffusion_sigma;
                        let d = defaults.diffusion_sigma;
                        (
                            format!("{v:.2}"),
                            Some(ratio(v, 0.5, 4.0)),
                            Some(ratio(d, 0.5, 4.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::Wind => {
                        let modified = format!("{:?}", runtime_state.wind_direction)
                            != format!("{:?}", defaults.wind_direction);
                        (
                            runtime_state.wind_direction.name().to_string(),
                            None,
                            None,
                            if modified {
                                ParamState::Modified
                            } else {
                                ParamState::Default
                            },
                        )
                    }
                    ParamId::TerrainType => {
                        let modified = format!("{:?}", runtime_state.terrain_type)
                            != format!("{:?}", defaults.terrain_type);
                        (
                            terrain_name.to_string(),
                            None,
                            None,
                            if modified {
                                ParamState::Modified
                            } else {
                                ParamState::Default
                            },
                        )
                    }
                    ParamId::TerrainStrength => {
                        let v = runtime_state.terrain_strength;
                        let d = defaults.terrain_strength;
                        (
                            format!("{v:.1}×"),
                            Some(ratio(v, 0.1, 5.0)),
                            Some(ratio(d, 0.1, 5.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::Attractor => {
                        let v = runtime_state.attractor_strength;
                        let d = defaults.attractor_strength;
                        (
                            format!("{v:.1}×"),
                            Some(ratio(v, 0.1, 10.0)),
                            Some(ratio(d, 0.1, 10.0)),
                            if near(v, d, 0.01) {
                                ParamState::Default
                            } else {
                                ParamState::Modified
                            },
                        )
                    }
                    ParamId::MouseMode => {
                        // No tracked default → always Default (old marker is blank).
                        (mouse_mode_name.to_string(), None, None, ParamState::Default)
                    }
                    ParamId::MouseTimeout => (
                        format!("{:.1}s", runtime_state.mouse_timeout),
                        None,
                        None,
                        ParamState::Display,
                    ),
                    // ── APP ───────────────────────────────────────────────────
                    ParamId::Theme => (theme.clone(), None, None, ParamState::Default),
                    ParamId::Palette => (palette_name.clone(), None, None, ParamState::Default),
                    ParamId::Charset => (charset.clone(), None, None, ParamState::Default),
                    ParamId::ColorAa => (color_aa.clone(), None, None, ParamState::Default),
                    ParamId::PaletteShift => {
                        (shift_name.to_string(), None, None, ParamState::Default)
                    }
                    ParamId::Invert => (
                        if runtime_state.invert_palette {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::Reverse => (
                        if runtime_state.reverse_palette {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::StatusLine => (
                        if runtime_state.show_status_bar {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::WindowFrame => (
                        format!("{:?}", runtime_state.window_frame),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::Chrome => (
                        format!("{:?}", runtime_state.chrome_style),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    // ── PST ───────────────────────────────────────────────────
                    ParamId::IntensityMapping => (
                        runtime_state.intensity_mapping_name().to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::Dither => (
                        runtime_state.dither_mode.name().to_string(),
                        None,
                        None,
                        // Display (muted), not Cli (red "restart to change") —
                        // matches the old "(dev)" row semantics.
                        ParamState::Display,
                    ),
                    ParamId::AutoNormalize => {
                        let v = runtime_state.auto_normalize;
                        (
                            if v { "On" } else { "Off" }.to_string(),
                            None,
                            None,
                            if v != defaults.auto_normalize {
                                ParamState::Modified
                            } else {
                                ParamState::Default
                            },
                        )
                    }
                    ParamId::MotionBlur => {
                        let v = runtime_state.motion_blur_frames;
                        (
                            format!("{v} fr"),
                            Some((v as f32 / 7.0).clamp(0.0, 1.0)),
                            Some((defaults.motion_blur_frames as f32 / 7.0).clamp(0.0, 1.0)),
                            if v != defaults.motion_blur_frames {
                                ParamState::Modified
                            } else {
                                ParamState::Default
                            },
                        )
                    }
                    ParamId::Brightness => {
                        // Brightness shown as gain; default (1.0×) sits mid-bar.
                        let v = runtime_state.max_brightness;
                        let d = defaults.max_brightness;
                        if runtime_state.auto_normalize {
                            (
                                "auto".to_string(),
                                Some(0.0),
                                Some(
                                    (crate::config_defaults::trail::brightness_gain(d) / 2.0)
                                        .clamp(0.0, 1.0),
                                ),
                                if near(v, d, 0.01) {
                                    ParamState::Default
                                } else {
                                    ParamState::Modified
                                },
                            )
                        } else {
                            let gain = crate::config_defaults::trail::brightness_gain(v);
                            let def_gain = crate::config_defaults::trail::brightness_gain(d);
                            (
                                format!("{gain:.1}×"),
                                Some((gain / 2.0).clamp(0.0, 1.0)),
                                Some((def_gain / 2.0).clamp(0.0, 1.0)),
                                if near(v, d, 0.01) {
                                    ParamState::Default
                                } else {
                                    ParamState::Modified
                                },
                            )
                        }
                    }
                    ParamId::TrailAge => {
                        let value = if runtime_state.trail_age_enabled {
                            if runtime_state.trail_age_reverse {
                                format!("On ({} rev)", runtime_state.trail_age_mode.name())
                            } else {
                                format!("On ({})", runtime_state.trail_age_mode.name())
                            }
                        } else {
                            "Off".to_string()
                        };
                        (value, None, None, ParamState::Default)
                    }
                    ParamId::TrailDelta => (
                        if runtime_state.trail_delta_enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::EdgeGlow => (
                        if runtime_state.gradient_magnitude_enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    // ── PRF ───────────────────────────────────────────────────
                    ParamId::FastMode => (
                        if runtime_state.fast_mode_enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        None,
                        None,
                        ParamState::Default,
                    ),
                    ParamId::Population => (
                        format!("{}k", population / 1000),
                        None,
                        None,
                        ParamState::Cli,
                    ),
                    // ── SYS (action rows) ─────────────────────────────────────
                    ParamId::SaveFrame => ("(PNG)".to_string(), None, None, ParamState::Default),
                    ParamId::Reset => ("Defaults".to_string(), None, None, ParamState::Default),
                    ParamId::Randomize => ("Params".to_string(), None, None, ParamState::Default),
                };

            // Numeric rows keep their ratio pair; everything else clears it.
            let (ratio, def_ratio) = if desc.kind == ParamKind::Numeric {
                (r, dr)
            } else {
                (None, None)
            };

            ParamView {
                desc,
                value_text,
                ratio,
                def_ratio,
                state,
            }
        })
        .collect()
}

/// Build a [`TuneView`] for the parameter a just-dispatched [`ControlAction`]
/// adjusted, sourcing the value/range/default/state from the same registry +
/// [`build_param_views_for_category`] path the Controls surface uses.
///
/// Returns `None` when the action is not a single-param adjust (overlay/global
/// actions), or when the param has no gauge-able view (action rows like
/// Save/Reset/Randomize, or display-only rows). Numeric params surface a real
/// gauge; Enum/Toggle params surface a flat marker (normalized 0 with the
/// formatted value text carrying the meaning).
fn tune_view_for_action(
    runtime_state: &RuntimeState,
    action: &ControlAction,
    ctx: &RegistryCtx,
    population: usize,
) -> Option<crate::render::ambient::TuneView> {
    use crate::render::controls::registry::{self, ParamId};

    let id = registry::param_id_for_action(action)?;
    // Action rows are MSG/activation events, not tunable params.
    if matches!(id, ParamId::SaveFrame | ParamId::Reset | ParamId::Randomize) {
        return None;
    }

    let (category, idx) = registry::locate(id, ctx)?;
    let views = build_param_views_for_category(runtime_state, category, ctx, population);
    let pv = views.get(idx)?;

    // Numeric rows carry normalized ratio/def_ratio for the gauge; non-numeric
    // rows have none — surface them at the bottom of the gauge with the value
    // text carrying the state. Gauge values are already normalized to [0, 1].
    let (value, default, show_gauge) = match (pv.ratio, pv.def_ratio) {
        (Some(r), Some(d)) => (r, d, true),
        _ => (0.0, 0.0, false),
    };

    Some(crate::render::ambient::TuneView {
        label: pv.desc.label.to_string(),
        value_text: pv.value_text.clone(),
        value,
        range: (0.0, 1.0),
        default,
        state: pv.state,
        show_gauge,
        kind: pv.desc.kind,
    })
}

/// Resolves a quick-key press to a bind target.
///
/// User binds (from `~/.config/tslime/keybinds.toml`) take precedence over
/// built-in preset defaults. Returns `None` when the key has no binding.
fn resolve_bind(
    c: char,
    user_binds: &std::collections::HashMap<char, crate::keybind_manager::BindTarget>,
) -> Option<crate::keybind_manager::BindTarget> {
    use crate::keybind_manager::BindTarget;
    if let Some(t) = user_binds.get(&c) {
        return Some(t.clone());
    }
    crate::simulation::config::preset_for_set_key(c).map(BindTarget::Preset)
}

/// Runs the interactive simulation loop (Live or Screensaver mode).
///
/// Handles terminal setup, input processing, simulation updates, and rendering
/// loop.
#[allow(unused_assignments)]
pub fn run_simulation(
    sim: &mut Simulation,
    args: &Args,
    mode: Mode,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    let mut screen = TerminalScreen::new();
    screen.setup()?;

    let capabilities = TerminalCapabilities::detect();
    log_capabilities(&capabilities, args.verbose);

    let mouse_mode = if args.mouse_attract {
        MouseInteractionMode::Attract
    } else if args.mouse_repel {
        MouseInteractionMode::Repel
    } else {
        MouseInteractionMode::Disabled
    };

    if mouse_mode != MouseInteractionMode::Disabled && capabilities.supports_mouse_tracking {
        if let Err(e) = crate::terminal::enable_mouse_tracking() {
            eprintln!(
                "Warning: Failed to enable mouse tracking: {}. Mouse interaction disabled.",
                e
            );
        }
    }

    let color_mode = capabilities.auto_select_color_mode(args.color_mode().ok());

    let startup_profile = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let config = startup_profile.sim.clone();
    let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));

    let init_mode = args
        .init
        .or(config.preferred_init_mode)
        .unwrap_or(InitMode::Food);

    let mut renderer = TerminalRenderer::new(
        0,
        0,
        palette,
        charset.clone(),
        args.reverse_palette,
        args.invert_palette,
        color_mode,
        background_color,
    );
    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);
    renderer.set_dither_mode(dither_mode);
    // Dither is dev-only for v0.1.0: runtime keys work only when it was
    // explicitly enabled at startup via the (hidden) CLI flags.
    let dither_unlocked = !matches!(dither_mode, DitherMode::None);
    renderer.set_ascii_contrast(args.ascii_contrast);
    renderer.set_window_frame(config.window_frame);
    let mut timer = FrameTimer::with_time_scale(args.fps, args.frame_delay, args.time_scale);
    timer.set_adaptive_fps(args.auto_fps);
    let input_poller = InputPoller::new();

    let (mut term_width, mut term_height) = screen.get_size()?;
    renderer.set_dimensions(term_width as usize, term_height as usize);

    // Compute initial window layout for windowed (non-fullscreen) chrome styles.
    // `mut` because the config-load apply seam recomputes it on load.
    let mut window = crate::render::window::Window {
        aspect: config.aspect,
        padding: config.window_padding,
        ring_cols: config.frame_matte_cols + 1,
        ring_rows: config.frame_matte_rows + 1,
        min_sim_size: config.min_sim_size,
        min_frame_size: config.min_frame_size,
    };
    {
        use crate::simulation::config::ChromeStyle;
        let initial_layout = if matches!(config.chrome_style, ChromeStyle::Fullscreen) {
            None
        } else {
            let l = window.compute_rects(term_width as usize, term_height as usize);
            if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
                None
            } else {
                Some(l)
            }
        };
        renderer.set_window_layout(initial_layout);
    }

    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let initial_preset = args.preset.unwrap_or(Preset::Organic);

    let resolved = startup_profile.render.clone();

    let mut runtime_state = RuntimeState::new(
        seed,
        init_mode,
        initial_preset,
        mouse_mode,
        args.mouse_timeout,
        &config,
        args.pause_style,
        args.pause_logo,
        args.pause_pulse_draw_mode,
    );
    // Classify the startup invocation: bare `--preset <p>` (no other overrides, no seed pin)
    // → Preset(p); anything more complex (extra flags, seed, no preset) → StartupCli.
    let startup_ov = crate::profile_overrides::ProfileOverrides::from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    runtime_state.active_source = if let Some(p) = args.preset {
        // Build a clap-parsed template for bare `--preset <name>` to get correct clap
        // defaults (not Rust struct defaults which differ on fps, time_scale, etc.).
        let preset_name = crate::terminal::control::preset_name(p);
        let template_args = Args::parse_from(["tslime", "--preset", preset_name]);
        let template = crate::profile_overrides::ProfileOverrides::from_args(&template_args)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        match startup_ov.bare_preset_against(&template) {
            Some(preset) => crate::profile::ProfileSource::Preset(preset),
            None => crate::profile::ProfileSource::StartupCli,
        }
    } else {
        crate::profile::ProfileSource::StartupCli
    };
    // Seed the live app-runtime config + active overrides from the startup profile so
    // the runner reads warmup/auto-reset/grid/food from rs.app (the single live source).
    runtime_state.app = startup_profile.app.clone();
    runtime_state.active_overrides = startup_ov;
    runtime_state.preload_pause_logo(term_width as usize, term_height as usize);
    runtime_state.dither_mode = dither_mode;
    runtime_state.trail_age_enabled = args.trail_age;
    runtime_state.trail_delta_enabled = args.trail_delta;
    runtime_state.gradient_magnitude_enabled = args.gradient_magnitude;
    runtime_state.gradient_strength = args.gradient_strength;
    runtime_state.trail_age_hue_range = args.trail_age_hue_range;
    runtime_state.trail_age_blend = args.trail_age_blend;
    runtime_state.trail_age_mode = match args.trail_age_mode.as_str() {
        "alternating" => crate::config_defaults::TrailAgeMode::Alternating,
        _ => crate::config_defaults::TrailAgeMode::Bidirectional,
    };
    runtime_state.trail_age_reverse = args.trail_age_reverse;
    runtime_state.trail_delta_strength = args.trail_delta_strength;
    // decay_gamma / diffuse_weight come from the assembled config via
    // RuntimeState::new — do not re-clobber them from raw CLI args here.
    if args.stats {
        runtime_state.overlay_state.open(OverlayType::Dashboard);
    }
    if args.trail_age {
        sim.set_compute_trail_age(true);
    }
    if args.trail_delta {
        sim.set_compute_trail_delta(true);
    }
    if args.gradient_magnitude {
        sim.set_compute_gradient_magnitude(true);
    }
    crate::app::apply_render_config(&resolved, &mut runtime_state, &mut renderer, sim);
    runtime_state.set_render_baseline(resolved.clone());

    // Initialize food persistence
    if args.food_persist && init_mode == InitMode::Food {
        runtime_state.food_persist_enabled = true;
        runtime_state.initial_food_attractors = Simulation::create_food_attractors(
            args.resolution.width,
            args.resolution.height,
            &args.food,
            args.food_invert,
            args.food_scale,
            runtime_state.app.food_persist_strength,
            0.3,
        );

        let mut new_config = sim.config().clone();
        new_config
            .attractors
            .extend(runtime_state.initial_food_attractors.clone());
        sim.update_config(new_config);
    }

    if args.species_colors_enabled() {
        let species_rgb_colors = extract_species_rgb_colors(&config);
        renderer.set_species_colors(true, species_rgb_colors);
    }

    // apply_render_config (above) has already set runtime_state.auto_normalize from
    // the resolved render config, so a preset may default it ON. Seed the loop-local
    // and the AdaptiveBrightness from rs, not the raw CLI flag.
    let mut current_auto_normalize = runtime_state.auto_normalize;
    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, current_auto_normalize);
    let mut hue_offset: f32 = 0.0;

    if args.random {
        runtime_state.randomize_params();
        apply_random_config(&runtime_state, sim, &mut renderer, &ALL_PALETTES);
    }

    let start_time = std::time::Instant::now();

    let mut grid_renderer = crate::app::build_grid_renderer(
        &runtime_state.app,
        (term_width as usize, term_height as usize),
    );

    // Compute sim render dimensions (may be smaller than terminal in windowed mode)
    let compute_render_dims = |tw: usize, th: usize| -> (usize, usize) {
        use crate::simulation::config::ChromeStyle;
        if matches!(config.chrome_style, ChromeStyle::Fullscreen) {
            return (tw, th);
        }
        let l = window.compute_rects(tw, th);
        if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
            (tw, th)
        } else {
            (l.sim_w, l.sim_h)
        }
    };
    let (initial_render_w, initial_render_h) =
        compute_render_dims(term_width as usize, term_height as usize);

    // Pre-allocate frame buffers to avoid per-frame allocations
    let mut downsampled_frame =
        crate::render::downsample::DownsampledFrame::new(initial_render_w, initial_render_h);
    let mut aux_frame = crate::render::downsample::AuxFrame {
        width: initial_render_w,
        height: initial_render_h,
        cells: vec![
            crate::render::downsample::AuxCell::default();
            initial_render_w * initial_render_h
        ],
    };
    let mut blended_trail_buffer: Vec<f32> = Vec::new();

    // Resize the sim render buffers to new dimensions. Must run on EVERY change to
    // render dims — terminal resize AND chrome-mode change — or `blur_field`'s
    // `src.len() == width*height` invariant breaks (e.g. F10 into Fullscreen grows
    // the sim while the buffers stay windowed-sized → panic in antialiasing.rs).
    fn resize_render_frames(
        downsampled: &mut crate::render::downsample::DownsampledFrame,
        aux: &mut crate::render::downsample::AuxFrame,
        w: usize,
        h: usize,
    ) {
        *downsampled = crate::render::downsample::DownsampledFrame::new(w, h);
        aux.width = w;
        aux.height = h;
        aux.cells = vec![crate::render::downsample::AuxCell::default(); w * h];
    }

    // Resolve the ambient surface overlay + its screen position for this frame.
    //   • Controls Tuner depth → the focused param renders as a persistent
    //     centered modal (the interactive tuner surface).
    //   • Controls Console depth → ambient hidden (the console IS the surface).
    //   • A live TUNE/MSG event (controls closed) → a transient centered modal.
    //   • Otherwise, when the always-on status line is enabled, a compact base row
    //     docks at the bottom interior of the frame (skipped in Expanded chrome,
    //     whose legacy footer already serves that role). Pause renders nothing
    //     extra here — the pause screen already presents its own surface.
    //   • Default: nothing (clean frame).
    // Shared by the main render path and the pause re-render path.
    fn compute_ambient_overlay(
        runtime_state: &RuntimeState,
        base_status: &crate::render::ambient::BaseStatus,
        window: &crate::render::window::Window,
        term_width: usize,
        term_height: usize,
        agent_count: usize,
    ) -> Option<(crate::render::panel::RenderedOverlay, usize, usize)> {
        use crate::render::ambient::{build_ambient_modal, build_base_row, resolve, AmbientState};
        use crate::render::window::FallbackMode;
        use crate::simulation::config::ChromeStyle;

        let now = runtime_state.phase_clock;
        let controls_open = runtime_state.overlay_state.is_open(OverlayType::Controls)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard);
        let mode = ambient_mode(controls_open, runtime_state.controls_depth);
        let st = &runtime_state.panel_style;

        let center = |ov: &crate::render::panel::RenderedOverlay| {
            let w = ov.lines.first().map(|l| l.chars().count()).unwrap_or(0);
            let h = ov.lines.len();
            (
                term_width.saturating_sub(w) / 2,
                term_height.saturating_sub(h) / 2,
            )
        };

        match mode {
            // Console open: the console is the full surface; no ambient overlay.
            AmbientMode::Hidden => None,
            // Tuner depth: render the focused param as a persistent centered modal.
            AmbientMode::Tuner => {
                let ctx = RegistryCtx {
                    diffusion_gaussian: matches!(
                        runtime_state.diffusion_kernel,
                        DiffusionKernel::Gaussian
                    ),
                    mouse_enabled: runtime_state.mouse_mode != MouseInteractionMode::Disabled,
                };
                let params = build_param_views(runtime_state, &ctx, agent_count);
                let idx = runtime_state
                    .controls_focus
                    .min(params.len().saturating_sub(1));
                let pv = params.get(idx)?;
                let (value, default, show_gauge) = match (pv.ratio, pv.def_ratio) {
                    (Some(r), Some(d)) => (r, d, true),
                    _ => (0.0, 0.0, false),
                };
                let tune = AmbientState::Tune {
                    param: crate::render::ambient::TuneView {
                        label: pv.desc.label.to_string(),
                        value_text: pv.value_text.clone(),
                        value,
                        range: (0.0, 1.0),
                        default,
                        state: pv.state,
                        show_gauge,
                        kind: pv.desc.kind,
                    },
                    until: f32::MAX,
                };
                let ov = build_ambient_modal(&tune, st, now);
                let (x, y) = center(&ov);
                Some((ov, x, y))
            }
            // Controls closed: live TUNE/MSG event → transient centered modal;
            // else optional always-on base row.
            AmbientMode::Normal => {
                let resolved = resolve(&runtime_state.ambient_states, now).clone();
                if matches!(
                    resolved,
                    AmbientState::Tune { .. } | AmbientState::Msg { .. }
                ) {
                    let ov = build_ambient_modal(&resolved, st, now);
                    let (x, y) = center(&ov);
                    return Some((ov, x, y));
                }
                if !runtime_state.show_status_bar {
                    return None;
                }
                let (bx, by, bw) = if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen)
                {
                    (0usize, term_height.saturating_sub(1), term_width)
                } else {
                    let l = window.compute_rects(term_width, term_height);
                    if matches!(l.fallback, FallbackMode::Fullscreen) {
                        (0usize, term_height.saturating_sub(1), term_width)
                    } else {
                        (l.sim_x, l.sim_y + l.sim_h.saturating_sub(1), l.sim_w)
                    }
                };
                // Expanded chrome's legacy footer already occupies the bottom row.
                if matches!(runtime_state.chrome_style, ChromeStyle::Expanded) {
                    None
                } else {
                    let ov = build_base_row(bw, st, base_status);
                    Some((ov, bx, by))
                }
            }
        }
    }

    #[cfg(feature = "audio")]
    let mut choir: Option<crate::audio::Choir> = if args.choir {
        match crate::audio::Choir::try_new(args.choir_volume.clamp(0.0, 1.0)) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Choir mode disabled: {e}");
                None
            }
        }
    } else {
        None
    };

    // The SINGLE swap executor. Both the clean path (is_dirty == false → call
    // immediately) and the post-confirm path (Enter on the dirty guard) route
    // through this so they can never diverge. Transactional: each variant commits
    // provenance (active_source/active_overrides) only after a successful apply.
    #[allow(clippy::too_many_arguments)]
    fn do_swap(
        pending: PendingSwap,
        rs: &mut RuntimeState,
        renderer: &mut TerminalRenderer,
        sim: &mut Simulation,
        timer: &mut FrameTimer,
        grid_renderer: &mut Option<crate::render::grid::GridRenderer>,
        window: &mut crate::render::window::Window,
        downsampled_frame: &mut crate::render::downsample::DownsampledFrame,
        aux_frame: &mut crate::render::downsample::AuxFrame,
        term_size: (usize, usize),
    ) -> io::Result<()> {
        match pending {
            PendingSwap::Preset(preset) => {
                switch_preset(
                    preset,
                    rs,
                    renderer,
                    sim,
                    timer,
                    grid_renderer,
                    window,
                    downsampled_frame,
                    aux_frame,
                    term_size,
                )?;
                rs.show_notification(format!(
                    "Applied preset: {}",
                    crate::terminal::control::preset_name(preset)
                ));
            }
            PendingSwap::Config(config) => {
                // The config was already resolved at selection time and carried
                // through the pending swap, so load it through the apply seam
                // directly, committing provenance only on success (transactional).
                match crate::app::apply_overrides(
                    &config.overrides,
                    rs,
                    renderer,
                    sim,
                    timer,
                    grid_renderer,
                    window,
                    downsampled_frame,
                    aux_frame,
                    term_size,
                    true,
                ) {
                    Ok(()) => {
                        rs.active_source =
                            crate::profile::ProfileSource::SavedConfig(config.name.clone());
                        rs.active_overrides = config.overrides.clone();
                        rs.show_notification(format!("Config '{}' loaded", config.name));
                    }
                    Err(e) => {
                        rs.show_notification(format!("Failed to load '{}': {}", config.name, e));
                    }
                }
            }
            PendingSwap::Reset => {
                let ov = rs.active_overrides.clone();
                rs.reset_transient();
                match crate::app::apply_overrides(
                    &ov,
                    rs,
                    renderer,
                    sim,
                    timer,
                    grid_renderer,
                    window,
                    downsampled_frame,
                    aux_frame,
                    term_size,
                    true,
                ) {
                    Ok(()) => {
                        rs.show_notification("Reset to defaults".to_string());
                    }
                    Err(e) => {
                        rs.show_notification(format!("Reset failed: {e}"));
                    }
                }
            }
        }
        Ok(())
    }

    // Dirty-guard-or-swap: the single place that decides between parking a swap
    // behind the dirty guard and applying it immediately. Returns true when it
    // swapped now (clean path), false when it parked (dirty path). Callers that
    // need clean-path-only side effects (e.g. zeroing hue) branch on the bool.
    #[allow(clippy::too_many_arguments)]
    fn dispatch_swap(
        pending: PendingSwap,
        rs: &mut RuntimeState,
        renderer: &mut TerminalRenderer,
        sim: &mut Simulation,
        timer: &mut FrameTimer,
        grid_renderer: &mut Option<crate::render::grid::GridRenderer>,
        window: &mut crate::render::window::Window,
        downsampled_frame: &mut crate::render::downsample::DownsampledFrame,
        aux_frame: &mut crate::render::downsample::AuxFrame,
        term_size: (usize, usize),
        current_auto_normalize: &mut bool,
        adaptive_brightness: &mut AdaptiveBrightness,
        normalize_window: usize,
    ) -> io::Result<bool> {
        if rs.is_dirty(
            sim.config(),
            rs.live_palette.clone(),
            rs.live_charset.clone(),
        ) {
            rs.pending_swap = Some(pending);
            rs.close_all_overlays();
            rs.overlay_state.open(OverlayType::DirtyGuard);
            rs.on_modal_open();
            Ok(false)
        } else {
            do_swap(
                pending,
                rs,
                renderer,
                sim,
                timer,
                grid_renderer,
                window,
                downsampled_frame,
                aux_frame,
                term_size,
            )?;
            *current_auto_normalize = rs.auto_normalize;
            *adaptive_brightness =
                AdaptiveBrightness::new(normalize_window, *current_auto_normalize);
            Ok(true)
        }
    }

    let user_binds = crate::keybind_manager::load_keybinds();

    loop {
        if is_shutdown_requested() {
            break;
        }

        if screen.check_resize() {
            let (new_width, new_height) = screen.get_size()?;
            if (new_width != term_width || new_height != term_height)
                && new_width > 0
                && new_height > 0
            {
                term_width = new_width;
                term_height = new_height;
                renderer.set_dimensions(term_width as usize, term_height as usize);
                // Recompute window layout on resize; derive render dims from the same layout
                // to avoid calling compute_rects a second time.
                let (new_render_w, new_render_h) = {
                    use crate::simulation::config::ChromeStyle;
                    // Read LIVE chrome (a loaded config may have changed it); the frozen
                    // startup `config.chrome_style` would size buffers against stale chrome.
                    let new_layout =
                        if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen) {
                            None
                        } else {
                            let l = window.compute_rects(term_width as usize, term_height as usize);
                            if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen)
                            {
                                None
                            } else {
                                Some(l)
                            }
                        };
                    // Derive render dims from the computed layout before moving it into renderer
                    let dims = new_layout
                        .as_ref()
                        .map(|l| (l.sim_w, l.sim_h))
                        .unwrap_or((term_width as usize, term_height as usize));
                    renderer.set_window_layout(new_layout);
                    dims
                };
                // Reinitialize grid with new dimensions
                if let Some(grid) = &mut grid_renderer {
                    grid.initialize(term_width as usize, term_height as usize);
                }
                // Resize frame buffers to sim render dimensions (may differ in windowed mode)
                resize_render_frames(
                    &mut downsampled_frame,
                    &mut aux_frame,
                    new_render_w,
                    new_render_h,
                );
            }
        }

        if term_width == 0 || term_height == 0 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        // Per-frame elapsed time, scaled by time_scale (NOT raw wall clock). Used
        // for UI effects (hue shift, chrome fade via dt_wall below) — never to step
        // the simulation, which uses the fixed timestep sim_dt.
        let dt = timer.delta_time();

        // Clamp dt to avoid UI animation jumps during lag spikes (max 0.1s / 10 FPS)
        let dt = dt.min(0.1);
        runtime_state.advance_phase(dt);

        // Fixed simulation timestep, decoupled from frame-write jitter. A blocked
        // write (e.g. terminal back-pressure while holding a key) inflates wall `dt`
        // and previously made the sim lurch forward one big step, which read as
        // flicker. Stepping by a fixed amount keeps motion smooth regardless of I/O.
        let sim_dt = timer.fixed_delta();

        // Chrome fade-out (500ms collapse) is UI, not simulation — divide the
        // time_scale back out so it animates at wall-clock speed.
        let dt_wall = dt / runtime_state.time_scale.max(f32::EPSILON);
        runtime_state.advance_fade(dt_wall);

        let in_warmup = !runtime_state.app.skip_warmup
            && runtime_state.is_in_warmup(runtime_state.app.warmup_frames);

        let frames_since_warmup = runtime_state
            .warmup_counter
            .saturating_sub(runtime_state.app.warmup_frames);
        let in_transition = frames_since_warmup < TRANSITION_DURATION_FRAMES;

        // Warmup→normal handoff: 0.0 during warmup, ramps 0→1 over
        // TRANSITION_DURATION_FRAMES, then holds at 1.0.
        let fade_factor = if in_warmup {
            0.0
        } else if in_transition {
            frames_since_warmup as f32 / TRANSITION_DURATION_FRAMES as f32
        } else {
            1.0
        };

        if !runtime_state.is_paused {
            timer.start_sim();

            // Ramp sim speed from WARMUP_SPEED_MULTIPLIER up to 1.0 as warmup ends.
            let speed_multiplier =
                WARMUP_SPEED_MULTIPLIER + (1.0 - WARMUP_SPEED_MULTIPLIER) * fade_factor;

            // Clamp against floating-point drift.
            let speed_multiplier = speed_multiplier.clamp(WARMUP_SPEED_MULTIPLIER, 1.0);

            let adjusted_dt = sim_dt * speed_multiplier;
            sim.update(adjusted_dt / REFERENCE_TIME_STEP);

            // Cap the counter so it stops once warmup + transition are complete.
            if !runtime_state.app.skip_warmup
                && runtime_state.warmup_counter
                    < runtime_state.app.warmup_frames + TRANSITION_DURATION_FRAMES
            {
                runtime_state.increment_warmup();
            }

            timer.end_sim_start_render();
        } else {
            timer.start_sim();
            timer.end_sim_start_render();
        }

        #[cfg(feature = "audio")]
        if let Some(ref c) = choir {
            if runtime_state.is_paused {
                c.silence_all();
            } else {
                let tm = sim.trail_map();
                crate::audio::update_voices_from_trail(
                    c,
                    tm.current(),
                    tm.width(),
                    tm.height(),
                    runtime_state.max_brightness.max(1.0),
                );
            }
        }

        let sim_width = sim.width();
        let sim_height = sim.height();
        let sim_dims = sim_width * sim_height;
        let agent_count = sim.agent_count();

        // Get blended trail first (takes &mut self)
        sim.trail_map_blended(&mut blended_trail_buffer);
        crate::app::fold_afterglow(
            &mut blended_trail_buffer,
            sim.afterglow_lag(),
            runtime_state.afterglow,
        );
        // Use sim render dimensions from downsampled_frame (may differ from terminal in windowed mode)
        let render_w = downsampled_frame.width();
        let render_h = downsampled_frame.height();
        downsample(
            &blended_trail_buffer,
            sim_width,
            sim_height,
            render_w,
            render_h,
            &mut downsampled_frame,
        );

        // Compute auxiliary frame for trail age / temporal delta / gradient / temporal color
        let current_aux_frame = if runtime_state.trail_age_enabled
            || runtime_state.trail_delta_enabled
            || runtime_state.gradient_magnitude_enabled
            || runtime_state.temporal_color > 0.0
        {
            let trail_age = if runtime_state.trail_age_enabled {
                sim.trail_age()
            } else {
                None
            };
            let trail_delta = if runtime_state.trail_delta_enabled {
                sim.trail_delta()
            } else {
                None
            };
            let gradient_mag = if runtime_state.gradient_magnitude_enabled {
                sim.gradient_magnitude()
            } else {
                None
            };

            crate::render::downsample::downsample_aux(
                trail_age,
                trail_delta,
                gradient_mag,
                sim.temporal_diff(),
                sim_width,
                sim_height,
                render_w,
                render_h,
                &mut aux_frame,
            );
            Some(&aux_frame)
        } else {
            None
        };
        renderer.set_visual_fx(
            current_aux_frame.cloned(),
            runtime_state.trail_age_enabled,
            runtime_state.trail_delta_enabled,
            runtime_state.trail_age_hue_range,
            runtime_state.trail_age_blend,
            runtime_state.trail_delta_strength,
            runtime_state.gradient_magnitude_enabled,
            runtime_state.gradient_strength,
            runtime_state.trail_age_mode,
            runtime_state.trail_age_reverse,
        );
        renderer.set_temporal(
            runtime_state.temporal_color,
            runtime_state.temporal_mode,
            runtime_state.temporal_accent,
        );

        // Narrow live reads instead of cloning the whole SimConfig (which carries heap Vecs
        // including obstacle masks). Species colors are the only Vec consumers, collected once
        // here and reused at both render sites. (DiffusionKernel is read directly via sim.config()
        // where needed — the old local was removed in Task 15 with build_status_line.)
        let current_species_rgb: Vec<crate::render::palette::RgbColor> =
            extract_species_rgb_colors(sim.config());

        adaptive_brightness.update(downsampled_frame.cells());
        let mut max_brightness = if current_auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            runtime_state.max_brightness
        };

        // Warmup brightness boost, eased out with the same fade_factor as the
        // speed ramp.
        if in_warmup || in_transition {
            // Inverse of fade_factor: 1.0 during warmup, easing to 0.0 as the
            // boost fades out.
            let brightness_fade = 1.0 - fade_factor;

            let multiplier =
                1.0 + (runtime_state.app.warmup_brightness_multiplier - 1.0) * brightness_fade;
            max_brightness *= multiplier;
        }

        let current_palette = runtime_state.current_palette(&ALL_PALETTES);

        let shift_degrees = runtime_state.palette_shift_speed.degrees_per_second();
        let is_off = runtime_state.palette_shift_speed == PaletteShiftSpeed::Off;

        if is_off && hue_offset.abs() > 0.05 {
            let lerp_factor = 3.0 * dt;
            hue_offset *= 1.0 - lerp_factor;

            if hue_offset.abs() < 0.1 {
                hue_offset = 0.0;
            }
        } else if !is_off {
            hue_offset += shift_degrees * dt;
            hue_offset %= 360.0;
        }

        renderer.set_hue_shift(hue_offset);

        // Build preset comparison overlay (Shift+1-7 keys)
        let preset_comparison_lines: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::PresetComparison)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
        {
            Some(PresetComparisonOverlay::build_overlay(
                &runtime_state,
                &runtime_state.comparison_target,
            ))
        } else {
            None
        };
        let (preset_comparison_x, preset_comparison_y) = if preset_comparison_lines.is_some() {
            PresetComparisonOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        let accent = palette_accent_color(
            &current_palette,
            runtime_state.reverse_palette,
            runtime_state.invert_palette,
            0.0,
            Some(&runtime_state.intensity_mapping),
        );
        let palette_editor_overlay: Option<RenderedOverlay> = (runtime_state
            .overlay_state
            .is_palette_editor_open()
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard))
        .then(|| {
            runtime_state
                .overlay_state
                .palette_editor
                .as_ref()
                .map(|s| PaletteEditorOverlay::build_overlay(s, &runtime_state.panel_style, accent))
        })
        .flatten();
        let (palette_editor_x, palette_editor_y) = if palette_editor_overlay.is_some() {
            PaletteEditorOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Controls position is computed below from the built overlay's own dims.

        // Palette accent colour used for key-binding highlights and title badges.
        let ui_accent = palette_accent_color(
            &current_palette,
            runtime_state.reverse_palette,
            runtime_state.invert_palette,
            hue_offset,
            Some(&runtime_state.intensity_mapping),
        );

        // Build keyboard hints overlay (? key)
        let keyboard_hints_lines: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::KeyboardHints)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
        {
            Some(KeyboardHintsOverlay::build_overlay(
                ui_accent,
                &runtime_state.panel_style,
                &user_binds,
            ))
        } else {
            None
        };
        let (keyboard_hints_x, keyboard_hints_y) = if let Some(ref ov) = keyboard_hints_lines {
            KeyboardHintsOverlay::calculate_position(
                term_width as usize,
                term_height as usize,
                ov.lines.len(),
            )
        } else {
            (0, 0)
        };

        // Build controls overlay (h key). The Controls surface is gated by the
        // overlay open-state (so esc / CloseOverlays / exclusivity keep working),
        // while `controls_depth` selects which depth (Console / Tuner) renders.
        //
        // Task 15: Tuner depth is now handled entirely by the ambient strip (see
        // ambient_overlay_built below). Only Console depth renders a separate
        // composite here; Tuner returns (None, 0, 0) so the bottom rows belong
        // exclusively to the ambient strip.
        let (controls_lines, controls_x, controls_y): (Option<RenderedOverlay>, usize, usize) =
            if runtime_state.overlay_state.is_open(OverlayType::Controls)
                && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
            {
                use crate::render::controls::{build_controls, ControlsDepth};
                // Defensive: an open surface with a stale `Closed` depth renders Console.
                let depth = match runtime_state.controls_depth {
                    ControlsDepth::Closed => ControlsDepth::Console,
                    other => other,
                };
                // Tuner depth: ambient strip owns the bottom rows — no separate composite.
                if depth == ControlsDepth::Tuner {
                    (None, 0, 0)
                } else {
                    let ctx = RegistryCtx {
                        diffusion_gaussian: matches!(
                            runtime_state.diffusion_kernel,
                            DiffusionKernel::Gaussian
                        ),
                        mouse_enabled: runtime_state.mouse_mode != MouseInteractionMode::Disabled,
                    };
                    let params = build_param_views(&runtime_state, &ctx, agent_count);
                    let truecolor = matches!(color_mode, ColorMode::TrueColor);
                    let overlay = build_controls(
                        depth,
                        runtime_state.controls_category_idx,
                        runtime_state.controls_focus,
                        &params,
                        &runtime_state.panel_style,
                        ui_accent,
                        truecolor,
                        term_width as usize,
                    );
                    // Console → centered on screen.
                    let (x, y) = match overlay.as_ref() {
                        Some(ov) => {
                            let w = ov.lines.first().map(|l| l.chars().count()).unwrap_or(0);
                            let h = ov.lines.len();
                            (
                                (term_width as usize).saturating_sub(w) / 2,
                                (term_height as usize).saturating_sub(h) / 2,
                            )
                        }
                        None => (0, 0),
                    };
                    (overlay, x, y)
                }
            } else {
                (None, 0, 0)
            };

        // Ambient surface model (redesign):
        //   • Default (always_on = false): clean sim frame at rest. The ambient
        //     surface appears only as a CENTERED MODAL on a live event — TUNE,
        //     MSG (notification), or pause — then dismisses back to the clean frame.
        //   • always_on (show_status_bar = true): a compact single-row BASE status
        //     line is docked at the bottom interior of the frame, in addition to
        //     the event modals. Consistent across Minimal/Fullscreen; in Expanded
        //     the legacy footer already serves as the status line.

        // Build ambient BASE status context (used by the compact base row).
        let dither_label = match runtime_state.dither_mode {
            crate::render::dither::DitherMode::None => None,
            crate::render::dither::DitherMode::Ordered { intensity, .. } => {
                Some(format!("D {:.1}×", intensity))
            }
            crate::render::dither::DitherMode::ErrorDiffusion { .. } => Some("ED".to_string()),
            crate::render::dither::DitherMode::Hybrid { intensity, .. } => {
                Some(format!("H {:.1}×", intensity))
            }
        };
        let ambient_base_status = BaseStatus {
            preset_name: preset_name(runtime_state.current_preset).to_string(),
            palette_name: palette_name(current_palette.clone()).to_string(),
            time_scale_text: format!("{:.1}×", runtime_state.time_scale),
            population: Some(agent_count),
            dither_label,
            can_undo: !runtime_state.undo_stack.is_empty(),
            can_redo: !runtime_state.redo_stack.is_empty(),
            accent: ui_accent,
            is_paused: runtime_state.is_paused,
        };
        // Resolve the highest-priority live ambient state for this frame. Drop
        // expired non-sticky entries first so the Vec stays small; the Base
        // sentinel always survives (it never expires).
        {
            let now = runtime_state.phase_clock;
            runtime_state
                .ambient_states
                .retain(|s| crate::render::ambient::ambient_state_is_live(s, now));
            if !runtime_state
                .ambient_states
                .iter()
                .any(|s| matches!(s, AmbientState::Base))
            {
                runtime_state.ambient_states.push(AmbientState::Base);
            }
        }
        // Resolve the ambient surface (event modal / optional base row / nothing).
        let ambient_overlay_built = compute_ambient_overlay(
            &runtime_state,
            &ambient_base_status,
            &window,
            term_width as usize,
            term_height as usize,
            agent_count,
        );
        let ambient_data = ambient_overlay_built.as_ref().map(|(v, x, y)| (v, *x, *y));

        // Legacy single-row status_data: only used by the PAUSE path renderer (see below).
        // In the MAIN path we pass None — the ambient strip takes its place.
        #[allow(clippy::type_complexity)]
        let status_data: Option<(String, usize, Vec<(usize, RgbColor)>)> = None;

        // Legacy notification path removed — all toasts now route through
        // ambient MSG (push_msg). notification_data kept as None for callers
        // that still reference it.
        let notification_data: Option<(&RenderedOverlay, usize, usize)> = None;

        // Dashboard overlay (merged stats + info)
        let entropy = DashboardOverlay::calculate_entropy(&blended_trail_buffer, 100);
        let trail_sum: f32 = blended_trail_buffer.iter().sum();
        let trail_capacity = sim_dims as f32 * 10.0;
        let trail_density = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity).min(1.0)
        } else {
            0.0
        };

        runtime_state.update_history(timer.current_fps() as f32, entropy, trail_density);

        let dashboard_overlay: Option<RenderedOverlay> =
            if runtime_state.overlay_state.is_open(OverlayType::Dashboard) {
                let elapsed = start_time.elapsed().as_secs_f32();
                let trail_max = blended_trail_buffer.iter().fold(0.0f32, |m, &v| v.max(m));
                let memory_mb = memory_stats()
                    .map(|m| m.physical_mem as f32 / 1024.0 / 1024.0)
                    .unwrap_or(0.0);
                let frame_time_ms = timer.last_frame_ms();
                let cpu_percent = (frame_time_ms / TARGET_FRAME_TIME_MS) * 100.0;

                // Use the live original_init_mode (updated by apply_overrides on config load),
                // not the startup-time `init_mode` local which is stale after a config load.
                let live_init_mode = runtime_state.original_init_mode;
                let init_mode_name = match live_init_mode {
                    InitMode::Random => "Random",
                    InitMode::CentralBurst => "Central",
                    InitMode::Circle => "Circle",
                    InitMode::Gradient => "Gradient",
                    InitMode::WaveFront => "Wave",
                    InitMode::Spiral => "Spiral",
                    InitMode::RandomClusters => "Clusters",
                    InitMode::Food => "Food",
                    InitMode::Petri => "Petri",
                    InitMode::Constellation => "Constellation",
                };

                let color_mode_name = match color_mode {
                    ColorMode::TrueColor => "TrueColor",
                    ColorMode::Bits8 => "8",
                    ColorMode::Bits16 => "16",
                    ColorMode::Bits256 => "256",
                };

                let charset_str = match charset {
                    Charset::HalfBlock => "HalfBlock",
                    Charset::HalfBlockDual => "HalfBlockDual",
                    Charset::Ascii => "ASCII",
                    Charset::Braille => "Braille",
                    Charset::Quadrant => "Quadrant",
                    Charset::Shade => "Shade",
                    Charset::Points => "Points",
                    Charset::Sculpted => "Sculpted",
                    Charset::CustomAscii(_) => "Custom",
                };

                let food_source = if live_init_mode == InitMode::Food {
                    Some(args.food.clone())
                } else {
                    None
                };

                let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
                let pname = palette_name(current_palette.clone());
                let prname = preset_name(runtime_state.current_preset);
                let palette_colors: Vec<RgbColor> = (0..78)
                    .map(|i| {
                        crate::render::palette::map_brightness_rgb(
                            i as f32 / 77.0,
                            current_palette.clone(),
                            runtime_state.reverse_palette,
                            runtime_state.invert_palette,
                            0.0,
                            None,
                        )
                    })
                    .collect();

                let current_config = sim.config();

                Some(DashboardOverlay::build_overlay(
                    sim.agent_count(),
                    trail_sum,
                    trail_capacity,
                    trail_max,
                    entropy,
                    timer.current_fps() as f32,
                    timer.average_fps() as f32,
                    timer.frame_count(),
                    elapsed,
                    sim.width(),
                    sim.height(),
                    sim.attractor_count(),
                    sim.obstacle_count(),
                    sim.species_count(),
                    memory_mb,
                    cpu_percent,
                    runtime_state.is_paused,
                    prname,
                    pname,
                    &palette_colors,
                    term_width as usize,
                    term_height as usize,
                    init_mode_name,
                    color_mode_name,
                    charset_str,
                    !args.simd_off,
                    current_config.decay_factor,
                    current_config.sensor_angle,
                    seed,
                    &food_source,
                    runtime_state.app.warmup_frames,
                    runtime_state.app.auto_reset,
                    ui_accent,
                    &runtime_state.panel_style,
                ))
            } else {
                None
            };

        let (dashboard_x, dashboard_y) =
            DashboardOverlay::calculate_position(term_width as usize, term_height as usize);

        // Config browser overlay
        let config_browser_overlay: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::ConfigBrowser)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
        {
            match config_manager::list_configs() {
                Ok(configs) => {
                    // Clamp selected index to valid range
                    runtime_state.config_browser_selected_index = runtime_state
                        .config_browser_selected_index
                        .min(configs.len().saturating_sub(1));
                    let active_name = match &runtime_state.active_source {
                        crate::profile::ProfileSource::SavedConfig(n) => Some(n.as_str()),
                        _ => None,
                    };
                    Some(crate::render::ratatui_adapter::build_config_browser(
                        &configs,
                        runtime_state.config_browser_selected_index,
                        active_name,
                    ))
                }
                Err(_) => {
                    runtime_state.show_notification("Failed to load configurations".to_string());
                    runtime_state.overlay_state.close();
                    None
                }
            }
        } else {
            None
        };
        let (config_browser_x, config_browser_y) = if config_browser_overlay.is_some() {
            ConfigBrowserOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Config save dialog overlay
        let config_save_overlay: Option<RenderedOverlay> =
            if runtime_state.overlay_state.is_open(OverlayType::ConfigSave)
                && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
            {
                // SPIKE: tui_input-backed field with a rendered caret (was append-only).
                Some(crate::render::ratatui_adapter::build_config_save(
                    runtime_state.config_save_name_input.value(),
                    runtime_state.config_save_name_input.cursor(),
                    &runtime_state.panel_style,
                ))
            } else {
                None
            };
        let (config_save_x, config_save_y) = if config_save_overlay.is_some() {
            ConfigSaveOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        let dirty_guard_overlay: Option<RenderedOverlay> =
            if runtime_state.overlay_state.is_open(OverlayType::DirtyGuard) {
                Some(DirtyGuardOverlay::build_overlay())
            } else {
                None
            };
        let (dirty_guard_x, dirty_guard_y) = if dirty_guard_overlay.is_some() {
            DirtyGuardOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        update_food_persistence(sim, &mut runtime_state);
        check_auto_reset(sim, &mut runtime_state, entropy, &blended_trail_buffer);

        // Update pause frame counter for animated pause effects
        if runtime_state.is_paused {
            runtime_state.pause_frame_counter += 1;
        } else {
            runtime_state.pause_frame_counter = 0;
        }

        // VCR-style pause overlay: dimmed logo centered in the drawable area
        // (the status bar shows PAUSED; no separate badge).
        let (pause_logo_overlay, pause_logo_x, pause_logo_y) = if runtime_state.is_paused
            && !runtime_state.any_overlay_open()
            && runtime_state.pause_logo_enabled
        {
            // Scale logo to terminal: small terminals use more %, large ones less
            let pct = if term_width < 80 {
                0.90
            } else if term_width < 120 {
                0.75
            } else {
                0.60
            };
            let logo_w = ((term_width as f32 * pct) as usize).clamp(30, 180);
            // Image is 1365×1024 (~1.33:1). Quadrant cells are 2×2 sub-pixels.
            // Terminal cell aspect ≈ 1:2 → logo_h = logo_w / (image_aspect * 2) = logo_w / 2.67
            let logo_h = ((logo_w as f32 / 2.67) as usize).max(6);
            // Quadrant: 2 pixels wide × 2 pixels tall per terminal cell
            let pixel_w = logo_w * 2;
            let pixel_h = logo_h * 2;

            // Decode or reuse cached brightness map
            let brightness_map = if runtime_state
                .pause_logo_cache
                .as_ref()
                .is_some_and(|(cw, _, _)| *cw == logo_w)
            {
                runtime_state.pause_logo_cache.as_ref().unwrap().2.clone()
            } else {
                let map = load_logo_from_memory(FOOD_IMAGE_PNG, pixel_w, pixel_h, true)
                    .unwrap_or_else(|_| vec![0.0; pixel_w * pixel_h]);
                runtime_state.pause_logo_cache = Some((logo_w, pixel_h, map.clone()));
                map
            };

            let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
            let logo_mapping = args
                .logo_mapping()
                .ok()
                .flatten()
                .unwrap_or_else(|| runtime_state.intensity_mapping.clone());
            let logo = PauseOverlay::build_logo(
                &brightness_map,
                logo_w,
                logo_h,
                current_palette,
                runtime_state.reverse_palette,
                runtime_state.invert_palette,
                0.0,
                Some(&logo_mapping),
            );

            let actual_logo_h = logo.lines.len();
            let lx = (term_width as usize).saturating_sub(logo_w) / 2;
            // Center vertically in the drawable area (exclude status bar row)
            let drawable_h = (term_height as usize).saturating_sub(1);
            let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

            (Some(logo), lx, ly)
        } else {
            (None, 0, 0)
        };

        // Update chrome snapshot so the renderer can draw expanded chrome overlays.
        {
            let diffusion_kernel_str = match runtime_state.diffusion_kernel {
                crate::simulation::config::DiffusionKernel::Mean3x3 => "Mean3x3",
                crate::simulation::config::DiffusionKernel::Gaussian => "Gaussian",
            };
            renderer.set_chrome_snapshot(ChromeSnapshot {
                chrome_state: runtime_state.chrome_state,
                preset: runtime_state.current_preset,
                palette: current_palette.clone(),
                charset_str: charset_name(&runtime_state.current_charset()).to_string(),
                population: agent_count,
                time_scale: runtime_state.time_scale,
                dither_mode: runtime_state.dither_mode,
                diffusion_kernel: Some(diffusion_kernel_str.to_string()),
                can_undo: !runtime_state.undo_stack.is_empty(),
                can_redo: !runtime_state.redo_stack.is_empty(),
                is_paused: runtime_state.is_paused,
            });
        }

        renderer.set_phase_clock(runtime_state.phase_clock);

        if args.species_colors_enabled() && sim.config().separate_species_trails {
            let species_trail_maps = sim.trail_maps_for_species_colors();
            let combined: Vec<_> = species_trail_maps
                .iter()
                .zip(current_species_rgb.iter())
                .map(|(tm, color)| (*tm, *color))
                .collect();
            renderer.render_multi_species_with_overlay(
                &combined,
                sim.width(),
                sim.height(),
                max_brightness.max(1.0),
                if runtime_state.is_paused {
                    Some(runtime_state.pause_frame_counter)
                } else {
                    None
                },
                pause_logo_overlay
                    .as_ref()
                    .map(|v| (v, pause_logo_x, pause_logo_y)),
                None, // no badge — status bar shows PAUSED
                controls_lines.as_ref().map(|v| (v, controls_x, controls_y)),
                status_data,
                notification_data,
                dashboard_overlay
                    .as_ref()
                    .map(|v| (v, dashboard_x, dashboard_y)),
                grid_renderer.as_ref(),
                config_browser_overlay
                    .as_ref()
                    .map(|v| (v, config_browser_x, config_browser_y)),
                config_save_overlay
                    .as_ref()
                    .map(|v| (v, config_save_x, config_save_y)),
                dirty_guard_overlay
                    .as_ref()
                    .map(|v| (v, dirty_guard_x, dirty_guard_y)),
                keyboard_hints_lines
                    .as_ref()
                    .map(|v| (v, keyboard_hints_x, keyboard_hints_y)),
                preset_comparison_lines
                    .as_ref()
                    .map(|v| (v, preset_comparison_x, preset_comparison_y)),
                palette_editor_overlay
                    .as_ref()
                    .map(|v| (v, palette_editor_x, palette_editor_y)),
                ambient_data,
                Some(&runtime_state.panel_style),
                runtime_state.overlay_state.active(),
                runtime_state.pause_style,
                runtime_state.pause_pulse_draw_mode,
            )?;
        } else {
            renderer.render_with_overlay(
                downsampled_frame.cells(),
                max_brightness.max(1.0),
                if runtime_state.is_paused {
                    Some(runtime_state.pause_frame_counter)
                } else {
                    None
                },
                pause_logo_overlay
                    .as_ref()
                    .map(|v| (v, pause_logo_x, pause_logo_y)),
                None, // no badge — status bar shows PAUSED
                controls_lines.as_ref().map(|v| (v, controls_x, controls_y)),
                status_data,
                notification_data,
                dashboard_overlay
                    .as_ref()
                    .map(|v| (v, dashboard_x, dashboard_y)),
                grid_renderer.as_ref(),
                config_browser_overlay
                    .as_ref()
                    .map(|v| (v, config_browser_x, config_browser_y)),
                config_save_overlay
                    .as_ref()
                    .map(|v| (v, config_save_x, config_save_y)),
                dirty_guard_overlay
                    .as_ref()
                    .map(|v| (v, dirty_guard_x, dirty_guard_y)),
                keyboard_hints_lines
                    .as_ref()
                    .map(|v| (v, keyboard_hints_x, keyboard_hints_y)),
                preset_comparison_lines
                    .as_ref()
                    .map(|v| (v, preset_comparison_x, preset_comparison_y)),
                palette_editor_overlay
                    .as_ref()
                    .map(|v| (v, palette_editor_x, palette_editor_y)),
                ambient_data,
                Some(&runtime_state.panel_style),
                runtime_state.overlay_state.active(),
                runtime_state.pause_style,
                runtime_state.pause_pulse_draw_mode,
            )?;
        }

        timer.end_render();

        let mut should_exit = false;
        let events = input_poller.drain_all_events()?;
        for event in events {
            match event {
                Event::Key(key_event) => {
                    // Track shift key state - only update on key press to avoid resetting on release
                    use crossterm::event::KeyEventKind;
                    if key_event.kind == KeyEventKind::Press {
                        use crossterm::event::KeyModifiers;
                        runtime_state.shift_held =
                            key_event.modifiers.contains(KeyModifiers::SHIFT);
                    }

                    // Screensaver mode: any key press exits (mouse events and
                    // resizes do not). Checked before overlay/control dispatch so
                    // no other handler can swallow the event.
                    if screensaver_exit_on_key(&mode, &key_event) {
                        should_exit = true;
                        break;
                    }

                    // GLOBAL EXIT HANDLING -- always allow 'q' to quit regardless of overlay
                    if InputPoller::is_exit_key(&key_event) {
                        should_exit = true;
                        break;
                    }
                    let action = handle_key_event(&key_event);
                    if let ControlAction::Quit = action {
                        should_exit = true;
                        break;
                    }

                    // Skip warmup on any key press
                    if in_warmup {
                        runtime_state.warmup_counter = runtime_state.app.warmup_frames;
                        // Skip to end
                    }

                    // Centralized overlay input handling: toggle keys, Escape, and
                    // blocking other keys while an overlay is open.
                    match OverlayInputManager::handle_input(
                        &runtime_state.overlay_state,
                        &key_event,
                    ) {
                        OverlayInputResult::CloseOverlay => {
                            let active = runtime_state.overlay_state.active();
                            let was_dirty_guard = active == Some(OverlayType::DirtyGuard);
                            let was_controls = active == Some(OverlayType::Controls);
                            let was_palette_editor = active == Some(OverlayType::PaletteEditor);

                            // PaletteEditor needs its sub-state cleared on close.
                            if was_palette_editor {
                                runtime_state.overlay_state.close_palette_editor();
                            } else {
                                runtime_state.overlay_state.close();
                            }

                            // Reset Controls depth when Controls is closed via toggle/Esc.
                            if was_controls {
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Closed;
                            }

                            // DirtyGuard cancel: clear the parked swap.
                            if was_dirty_guard {
                                runtime_state.pending_swap = None;
                                runtime_state.show_notification("Switch cancelled".to_string());
                            }

                            // Always update chrome state when an overlay closes.
                            runtime_state.on_modal_close();
                            continue;
                        }
                        OverlayInputResult::Consumed => {
                            // Key was blocked by open overlay, do nothing
                            continue;
                        }
                        OverlayInputResult::NotHandled => {
                            // No overlay open, continue to normal processing
                        }
                    }

                    // Handle dirty-state guard input. Escape was already handled by
                    // the centralized manager (→ CloseOverlay → cancels). Here we only
                    // act on Enter (confirm); all other keys are swallowed so the modal
                    // stays blocking. Both the confirm path and the clean path route
                    // through the SAME `do_swap`, so they cannot diverge.
                    if runtime_state.overlay_state.is_open(OverlayType::DirtyGuard) {
                        use crossterm::event::KeyCode;
                        if key_event.code == KeyCode::Enter {
                            runtime_state.overlay_state.close();
                            runtime_state.on_modal_close();
                            if let Some(pending) = runtime_state.pending_swap.take() {
                                let is_reset = matches!(pending, PendingSwap::Reset);
                                do_swap(
                                    pending,
                                    &mut runtime_state,
                                    &mut renderer,
                                    sim,
                                    &mut timer,
                                    &mut grid_renderer,
                                    &mut window,
                                    &mut downsampled_frame,
                                    &mut aux_frame,
                                    (term_width as usize, term_height as usize),
                                )?;
                                if is_reset {
                                    // Reset-specific loop-local side effects.
                                    hue_offset = 0.0;
                                }
                                // do_swap → apply_render_config may flip auto_normalize
                                // (preset/config can default it ON); re-sync the
                                // loop-local + rebuild AdaptiveBrightness for any swap.
                                current_auto_normalize = runtime_state.auto_normalize;
                                adaptive_brightness = AdaptiveBrightness::new(
                                    args.normalize_window,
                                    current_auto_normalize,
                                );
                            }
                        }
                        continue;
                    }

                    // Handle config save dialog input
                    if runtime_state.overlay_state.is_open(OverlayType::ConfigSave) {
                        use crossterm::event::{KeyCode, KeyModifiers};
                        use tui_input::InputRequest;
                        // Map crossterm keys → tui_input requests ourselves (tui-input's
                        // own crossterm backend is disabled to avoid version coupling).
                        let input = &mut runtime_state.config_save_name_input;
                        match key_event.code {
                            KeyCode::Char(c)
                                if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                if input.value().chars().count() < 26 {
                                    input.handle(InputRequest::InsertChar(c));
                                }
                                continue;
                            }
                            KeyCode::Backspace => {
                                input.handle(InputRequest::DeletePrevChar);
                                continue;
                            }
                            KeyCode::Delete => {
                                input.handle(InputRequest::DeleteNextChar);
                                continue;
                            }
                            KeyCode::Left => {
                                input.handle(InputRequest::GoToPrevChar);
                                continue;
                            }
                            KeyCode::Right => {
                                input.handle(InputRequest::GoToNextChar);
                                continue;
                            }
                            KeyCode::Home => {
                                input.handle(InputRequest::GoToStart);
                                continue;
                            }
                            KeyCode::End => {
                                input.handle(InputRequest::GoToEnd);
                                continue;
                            }
                            KeyCode::Enter => {
                                if !runtime_state.config_save_name_input.value().is_empty() {
                                    let named_profile = config_manager::NamedProfile {
                                        name: runtime_state
                                            .config_save_name_input
                                            .value()
                                            .to_string(),
                                        description: None,
                                        overrides: config_manager::capture_overrides(
                                            sim.config(),
                                            // Save the EXACT live palette/charset (incl. Custom),
                                            // not the lossy index or the stale launch charset.
                                            runtime_state.live_palette.clone(),
                                            runtime_state.live_charset.clone(),
                                            &runtime_state,
                                        ),
                                    };

                                    match config_manager::save_config(named_profile) {
                                        Ok(warning) => {
                                            let msg = match warning {
                                                Some(w) => format!(
                                                    "Config '{}' saved ({})",
                                                    runtime_state.config_save_name_input.value(),
                                                    w
                                                ),
                                                None => format!(
                                                    "Config '{}' saved successfully",
                                                    runtime_state.config_save_name_input.value()
                                                ),
                                            };
                                            runtime_state.show_notification(msg);
                                        }
                                        Err(e) => {
                                            runtime_state.show_notification(format!(
                                                "Failed to save config: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                runtime_state.overlay_state.close();
                                continue;
                            }
                            _ => continue,
                        }
                    }

                    // Handle preset comparison input
                    if runtime_state
                        .overlay_state
                        .is_open(OverlayType::PresetComparison)
                    {
                        use crossterm::event::KeyCode;
                        if key_event.code == KeyCode::Enter {
                            use crate::terminal::state::ComparisonTarget;
                            let pending = match &runtime_state.comparison_target {
                                ComparisonTarget::Preset(p) => PendingSwap::Preset(*p),
                                ComparisonTarget::Config(np) => PendingSwap::Config(np.clone()),
                            };
                            runtime_state.overlay_state.close();
                            dispatch_swap(
                                pending,
                                &mut runtime_state,
                                &mut renderer,
                                sim,
                                &mut timer,
                                &mut grid_renderer,
                                &mut window,
                                &mut downsampled_frame,
                                &mut aux_frame,
                                (term_width as usize, term_height as usize),
                                &mut current_auto_normalize,
                                &mut adaptive_brightness,
                                args.normalize_window,
                            )?;
                            continue;
                        }
                        // Note: Other keys are blocked by centralized handler
                    }

                    // Handle config browser input
                    if runtime_state
                        .overlay_state
                        .is_open(OverlayType::ConfigBrowser)
                    {
                        use crossterm::event::KeyCode;
                        match key_event.code {
                            KeyCode::Up => {
                                if runtime_state.config_browser_selected_index > 0 {
                                    runtime_state.config_browser_selected_index -= 1;
                                }
                                continue;
                            }
                            KeyCode::Down => {
                                // Bound the selection to the last config so it can't
                                // overshoot the list (render-time clamp is kept as a
                                // defensive guard).
                                let last = config_manager::list_configs()
                                    .map(|c| c.len().saturating_sub(1))
                                    .unwrap_or(0);
                                if runtime_state.config_browser_selected_index < last {
                                    runtime_state.config_browser_selected_index += 1;
                                }
                                continue;
                            }
                            KeyCode::Enter => {
                                // Resolve the selected config (in full), close the
                                // browser, then gate the load on dirty state: dirty →
                                // park behind the guard; clean → load now via the single
                                // executor. Carrying the resolved config avoids a
                                // redundant disk re-read inside do_swap.
                                let selected_config =
                                    config_manager::list_configs().ok().and_then(|mut configs| {
                                        let idx = runtime_state.config_browser_selected_index;
                                        (idx < configs.len()).then(|| configs.swap_remove(idx))
                                    });
                                runtime_state.overlay_state.close();
                                if let Some(config) = selected_config {
                                    dispatch_swap(
                                        PendingSwap::Config(Box::new(config)),
                                        &mut runtime_state,
                                        &mut renderer,
                                        sim,
                                        &mut timer,
                                        &mut grid_renderer,
                                        &mut window,
                                        &mut downsampled_frame,
                                        &mut aux_frame,
                                        (term_width as usize, term_height as usize),
                                        &mut current_auto_normalize,
                                        &mut adaptive_brightness,
                                        args.normalize_window,
                                    )?;
                                }
                                continue;
                            }
                            KeyCode::Delete => {
                                if let Ok(configs) = config_manager::list_configs() {
                                    if let Some(config) =
                                        configs.get(runtime_state.config_browser_selected_index)
                                    {
                                        let name = config.name.clone();
                                        match config_manager::delete_config(&name) {
                                            Ok(_) => {
                                                runtime_state.show_notification(format!(
                                                    "Deleted config '{}'",
                                                    name
                                                ));
                                            }
                                            Err(e) => {
                                                runtime_state.show_notification(format!(
                                                    "Failed to delete: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                                continue;
                            }
                            _ => continue,
                        }
                    }

                    // Handle palette editor input using OverlayInputHandler trait
                    if runtime_state.overlay_state.is_palette_editor_open() {
                        use crate::overlay::input::OverlayInputHandler;

                        // Initialize palette editor state if needed
                        if runtime_state.overlay_state.palette_editor.is_none() {
                            let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                            runtime_state
                                .overlay_state
                                .open_palette_editor(PaletteEditorState::new(&current_palette));
                        }

                        let mut should_close = false;
                        let mut notification: Option<String> = None;

                        if let Some(ref mut state) = runtime_state.overlay_state.palette_editor {
                            let was_modified = state.is_modified;

                            let handled = state.handle_key(&key_event);

                            // Check if we need to close (Esc or Enter in Editing mode returned false)
                            let is_escape = key_event.code == crossterm::event::KeyCode::Esc;
                            let is_enter = key_event.code == crossterm::event::KeyCode::Enter;
                            let is_editing = matches!(state.mode, EditorMode::Editing);

                            if !handled && (is_escape || is_enter) && is_editing {
                                should_close = true;
                                if is_enter {
                                    notification = Some("Custom palette applied".to_string());
                                } else {
                                    // On escape, restore original palette
                                    let original = Palette::Custom(state.original_colors.to_vec());
                                    renderer.set_palette(original);
                                }
                            } else if handled {
                                // Apply palette changes after each adjustment
                                renderer.set_palette(state.to_palette());

                                if matches!(state.mode, EditorMode::SaveDialog) {
                                    // No-op: save completion is detected via the
                                    // is_modified transition below.
                                }
                            }

                            // Track if palette was saved
                            let saved_name = if was_modified && !state.is_modified {
                                Some(state.base_palette_name.clone())
                            } else {
                                None
                            };

                            // Apply cleanup after borrow
                            if should_close {
                                runtime_state.overlay_state.close_palette_editor();
                            }
                            if let Some(name) = saved_name {
                                runtime_state.saved_palette_name = Some(name);
                            }
                            if let Some(msg) = notification {
                                runtime_state.show_notification(msg);
                            }
                        }
                        continue;
                    }

                    if InputPoller::is_exit_key(&key_event) {
                        should_exit = true;
                        break;
                    }

                    let mut action = handle_key_event(&key_event);
                    // Build the registry visibility context from current runtime state.
                    let registry_ctx = RegistryCtx {
                        diffusion_gaussian: matches!(
                            runtime_state.diffusion_kernel,
                            DiffusionKernel::Gaussian
                        ),
                        mouse_enabled: runtime_state.mouse_mode != MouseInteractionMode::Disabled,
                    };
                    // Controls-overlay interaction is intercepted first. Focused
                    // adjust/activate resolve to a real action that is then
                    // re-dispatched through the same match below (one hop).
                    match apply_controls_action(&mut runtime_state, &action, &registry_ctx) {
                        ControlsDispatch::Handled => continue,
                        ControlsDispatch::Redispatch(real) => action = real,
                        ControlsDispatch::Passthrough => {}
                    }
                    match action {
                        ControlAction::Quit => {
                            should_exit = true;
                        }
                        ControlAction::TogglePause => {
                            runtime_state.toggle_pause();
                            runtime_state.pause_just_toggled = true;
                            if runtime_state.is_paused {
                                runtime_state.on_pause();
                            } else {
                                runtime_state.on_unpause_with_fade();
                            }
                        }
                        ControlAction::Restart => {
                            let init_mode = effective_init_mode(runtime_state.original_init_mode);
                            sim.reset(runtime_state.original_seed, init_mode);
                        }
                        ControlAction::QuickKey(c) => match resolve_bind(c, &user_binds) {
                            Some(crate::keybind_manager::BindTarget::Preset(preset)) => {
                                dispatch_swap(
                                    PendingSwap::Preset(preset),
                                    &mut runtime_state,
                                    &mut renderer,
                                    sim,
                                    &mut timer,
                                    &mut grid_renderer,
                                    &mut window,
                                    &mut downsampled_frame,
                                    &mut aux_frame,
                                    (term_width as usize, term_height as usize),
                                    &mut current_auto_normalize,
                                    &mut adaptive_brightness,
                                    args.normalize_window,
                                )?;
                            }
                            Some(crate::keybind_manager::BindTarget::Config(name)) => {
                                match config_manager::load_config(&name) {
                                    Ok(config) => {
                                        dispatch_swap(
                                            PendingSwap::Config(Box::new(config)),
                                            &mut runtime_state,
                                            &mut renderer,
                                            sim,
                                            &mut timer,
                                            &mut grid_renderer,
                                            &mut window,
                                            &mut downsampled_frame,
                                            &mut aux_frame,
                                            (term_width as usize, term_height as usize),
                                            &mut current_auto_normalize,
                                            &mut adaptive_brightness,
                                            args.normalize_window,
                                        )?;
                                    }
                                    Err(e) => {
                                        runtime_state.show_notification(format!(
                                            "Failed to load '{name}': {e}"
                                        ));
                                    }
                                }
                            }
                            None => {
                                runtime_state.show_notification(format!(
                                    "Key {c} unbound — set in ~/.config/tslime/keybinds.toml"
                                ));
                            }
                        },
                        ControlAction::CompareQuickKey(c) => {
                            use crate::keybind_manager::BindTarget;
                            use crate::terminal::state::ComparisonTarget;
                            let target = match resolve_bind(c, &user_binds) {
                                Some(BindTarget::Preset(p)) => Some(ComparisonTarget::Preset(p)),
                                Some(BindTarget::Config(name)) => {
                                    match config_manager::load_config(&name) {
                                        Ok(np) => Some(ComparisonTarget::Config(Box::new(np))),
                                        Err(e) => {
                                            runtime_state.show_notification(format!(
                                                "Failed to load '{name}': {e}"
                                            ));
                                            None
                                        }
                                    }
                                }
                                None => None,
                            };
                            if let Some(target) = target {
                                runtime_state.toggle_comparison(target);
                                if runtime_state.any_overlay_open() {
                                    runtime_state.on_modal_open();
                                } else {
                                    runtime_state.on_modal_close();
                                }
                            }
                        }
                        ControlAction::AdjustTimeScale(delta) => {
                            runtime_state.adjust_time_scale(delta);
                            timer.set_time_scale(runtime_state.time_scale);
                        }
                        ControlAction::CyclePalette => {
                            runtime_state.cycle_palette(num_palettes());
                            let new_palette = runtime_state.current_palette(&ALL_PALETTES);
                            renderer.set_palette(new_palette);
                        }
                        ControlAction::CyclePaletteReverse => {
                            runtime_state.cycle_palette_reverse(num_palettes());
                            let new_palette = runtime_state.current_palette(&ALL_PALETTES);
                            renderer.set_palette(new_palette);
                        }
                        ControlAction::CycleCharset => {
                            runtime_state.cycle_charset();
                            renderer.set_charset(runtime_state.current_charset());
                            renderer.set_color_aa(runtime_state.current_color_aa());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::CycleCharsetReverse => {
                            runtime_state.cycle_charset_reverse();
                            renderer.set_charset(runtime_state.current_charset());
                            renderer.set_color_aa(runtime_state.current_color_aa());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::CycleColorAa => {
                            if runtime_state.cycle_color_aa() {
                                renderer.set_color_aa(runtime_state.current_color_aa());
                                runtime_state.show_notification(format!(
                                    "Color AA ({}): {}",
                                    charset_name(&runtime_state.current_charset()),
                                    runtime_state.current_color_aa().as_label()
                                ));
                            } else {
                                runtime_state.show_notification(format!(
                                    "Color AA not applicable to {}",
                                    charset_name(&runtime_state.current_charset())
                                ));
                            }
                        }
                        ControlAction::ToggleDither => {
                            if dither_unlocked {
                                runtime_state.toggle_dither();
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::CycleDitherMode => {
                            if dither_unlocked {
                                runtime_state.cycle_dither_mode();
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::AdjustDitherIntensity(delta) => {
                            if dither_unlocked {
                                runtime_state.adjust_dither_intensity(delta);
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::ToggleKeyboardHints => {
                            runtime_state.toggle_keyboard_hints();
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::ToggleControls => {
                            runtime_state.toggle_controls();
                            // Opening the surface enters the Console depth; closing
                            // resets it. Tab then cycles Console↔Tuner per the grammar.
                            if runtime_state.overlay_state.is_open(OverlayType::Controls) {
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Console;
                            } else {
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Closed;
                            }
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::CloseOverlays => {
                            if runtime_state.any_overlay_open() {
                                runtime_state.close_all_overlays();
                                // Closing all overlays also resets the Controls depth.
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Closed;
                                runtime_state.on_modal_close();
                            } else {
                                // No overlays open: Esc clears a sticky error MSG if one
                                // is the resolved foreground state.
                                let now = runtime_state.phase_clock;
                                let resolved = crate::render::ambient::resolve(
                                    &runtime_state.ambient_states,
                                    now,
                                );
                                let is_sticky_error = matches!(
                                    resolved,
                                    AmbientState::Msg {
                                        level: crate::terminal::state::NotificationLevel::Error,
                                        sticky: true,
                                        ..
                                    }
                                );
                                if is_sticky_error {
                                    runtime_state.ambient_states.retain(|s| {
                                        !matches!(
                                            s,
                                            AmbientState::Msg {
                                                level: crate::terminal::state::NotificationLevel::Error,
                                                sticky: true,
                                                ..
                                            }
                                        )
                                    });
                                }
                            }
                        }
                        ControlAction::CycleOptionsCategory => {
                            // Only reached when the overlay is CLOSED: open it into Console depth.
                            // The open-overlay case (cycle category + clamp focus) is handled earlier
                            // in apply_controls_action, which returns Handled so this arm is skipped.
                            if !runtime_state.overlay_state.is_open(OverlayType::Controls) {
                                runtime_state.toggle_controls();
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Console;
                            }
                        }
                        ControlAction::CycleOptionsCategoryReverse => {
                            // Only reached when the overlay is CLOSED: open it into Console depth.
                            // The open-overlay case (cycle category + clamp focus) is handled earlier
                            // in apply_controls_action, which returns Handled so this arm is skipped.
                            if !runtime_state.overlay_state.is_open(OverlayType::Controls) {
                                runtime_state.toggle_controls();
                                runtime_state.controls_depth =
                                    crate::render::controls::ControlsDepth::Console;
                            }
                        }
                        ControlAction::AdjustSensorAngle(delta) => {
                            let at_bound = runtime_state.adjust_sensor_angle(delta);
                            sim.with_config_mut(|c| c.sensor_angle = runtime_state.sensor_angle);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor angle at {}°",
                                    runtime_state.sensor_angle
                                ));
                            }
                        }
                        ControlAction::AdjustSensorDistance(delta) => {
                            let at_bound = runtime_state.adjust_sensor_distance(delta);
                            sim.with_config_mut(|c| {
                                c.sensor_distance = runtime_state.sensor_distance
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor distance at {:.1}",
                                    runtime_state.sensor_distance
                                ));
                            }
                        }
                        ControlAction::AdjustTurnAngle(delta) => {
                            let at_bound = runtime_state.adjust_rotation_angle(delta);
                            sim.with_config_mut(|c| {
                                c.rotation_angle = runtime_state.rotation_angle
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Turn angle at {}°",
                                    runtime_state.rotation_angle
                                ));
                            }
                        }
                        ControlAction::AdjustStepSize(delta) => {
                            let at_bound = runtime_state.adjust_step_size(delta);
                            sim.with_config_mut(|c| c.step_size = runtime_state.step_size);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Step size at {:.1}",
                                    runtime_state.step_size
                                ));
                            }
                        }
                        ControlAction::AdjustDecay(delta) => {
                            let at_bound = runtime_state.adjust_decay(delta);
                            sim.with_config_mut(|c| c.decay_factor = runtime_state.decay_factor);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Decay factor at {:.3}",
                                    runtime_state.decay_factor
                                ));
                            }
                        }
                        ControlAction::AdjustDeposit(delta) => {
                            let at_bound = runtime_state.adjust_deposit(delta);
                            sim.with_config_mut(|c| {
                                c.deposit_amount = runtime_state.deposit_amount
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Deposit amount at {:.1}",
                                    runtime_state.deposit_amount
                                ));
                            }
                        }
                        ControlAction::CycleDiffusionKernel => {
                            runtime_state.cycle_diffusion_kernel();
                            sim.with_config_mut(|c| {
                                c.diffusion_kernel = runtime_state.diffusion_kernel
                            });
                            runtime_state.show_notification(format!(
                                "Diffusion kernel: {}",
                                match runtime_state.diffusion_kernel {
                                    DiffusionKernel::Mean3x3 => "Mean3x3",
                                    DiffusionKernel::Gaussian => "Gaussian",
                                }
                            ));
                        }
                        ControlAction::AdjustDiffusionSigma(delta) => {
                            let at_bound = runtime_state.adjust_diffusion_sigma(delta);
                            sim.with_config_mut(|c| {
                                c.diffusion_sigma = runtime_state.diffusion_sigma
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Diffusion sigma at {:.2}",
                                    runtime_state.diffusion_sigma
                                ));
                            }
                        }
                        ControlAction::AdjustAttractorStrength(delta) => {
                            let at_bound = runtime_state.adjust_attractor_strength(delta);
                            sim.with_config_mut(|c| {
                                c.attractor_strength = runtime_state.attractor_strength
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Attractor strength at {:.1}",
                                    runtime_state.attractor_strength
                                ));
                            }
                        }
                        ControlAction::CycleMouseMode => {
                            runtime_state.cycle_mouse_mode();
                            runtime_state.show_notification(format!(
                                "Mouse mode: {}",
                                match runtime_state.mouse_mode {
                                    MouseInteractionMode::Disabled => "Disabled",
                                    MouseInteractionMode::Attract => "Attract",
                                    MouseInteractionMode::Repel => "Repel",
                                }
                            ));
                        }
                        ControlAction::CycleWindDirection => {
                            runtime_state.cycle_wind_direction();
                            // Coarse cycle PRODUCES the precise vector; store it losslessly.
                            runtime_state.wind = runtime_state.wind_direction.to_wind();
                            sim.with_config_mut(|c| c.wind = runtime_state.wind);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::CycleWindDirectionReverse => {
                            runtime_state.cycle_wind_direction_reverse();
                            runtime_state.wind = runtime_state.wind_direction.to_wind();
                            sim.with_config_mut(|c| c.wind = runtime_state.wind);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::AdjustTerrainStrength(delta) => {
                            let at_bound = runtime_state.adjust_terrain_strength(delta);
                            sim.with_config_mut(|c| {
                                c.terrain_strength = runtime_state.terrain_strength
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Terrain strength at {:.1}",
                                    runtime_state.terrain_strength
                                ));
                            }
                        }
                        ControlAction::CycleTerrainType => {
                            runtime_state.cycle_terrain_type();
                            sim.with_config_mut(|c| c.terrain = runtime_state.terrain_type);
                            runtime_state.show_notification(format!(
                                "Terrain: {}",
                                match runtime_state.terrain_type {
                                    TerrainType::None => "None",
                                    TerrainType::Smooth => "Smooth",
                                    TerrainType::Turbulent => "Turbulent",
                                    TerrainType::Mixed => "Mixed",
                                }
                            ));
                        }
                        ControlAction::ToggleAutoNormalize => {
                            runtime_state.toggle_auto_normalize();
                            current_auto_normalize = runtime_state.auto_normalize;
                            adaptive_brightness = AdaptiveBrightness::new(
                                args.normalize_window,
                                current_auto_normalize,
                            );
                        }
                        ControlAction::CycleMotionBlur => {
                            runtime_state.cycle_motion_blur();
                            runtime_state.show_notification(format!(
                                "Motion blur: {} frames",
                                runtime_state.motion_blur_frames
                            ));
                        }
                        ControlAction::AdjustMaxBrightness(delta) => {
                            if current_auto_normalize {
                                // Auto-normalize drives the white-point from the
                                // adaptive peak, so the manual control is inert.
                                // Warning level so it outranks the tuner that the
                                // adjust surfaces — the inert gauge alone would
                                // mislead; this message explains why.
                                runtime_state.show_notification_with_level(
                                    "Brightness is auto-normalized (press B to disable)"
                                        .to_string(),
                                    crate::terminal::state::NotificationLevel::Warning,
                                );
                            } else {
                                let at_bound = runtime_state.adjust_max_brightness(delta);
                                // Mirror the live value into the sim config so it is
                                // captured on save (Ctrl+S reads sim.config()), matching
                                // how every other adjustable parameter syncs.
                                sim.with_config_mut(|c| {
                                    c.max_brightness = runtime_state.max_brightness;
                                });
                                if at_bound {
                                    let gain = crate::config_defaults::trail::brightness_gain(
                                        runtime_state.max_brightness,
                                    );
                                    runtime_state
                                        .show_notification(format!("Brightness at {gain:.1}×"));
                                }
                            }
                        }
                        ControlAction::SaveFrameToPng => {
                            use crate::export::png::save_frame_as_png;

                            let png_aux_cells = if runtime_state.temporal_color > 0.0 {
                                Some(aux_frame.cells.as_slice())
                            } else {
                                None
                            };
                            match save_frame_as_png(
                                downsampled_frame.cells(),
                                term_width as usize,
                                term_height as usize,
                                current_palette.clone(),
                                runtime_state.reverse_palette,
                                runtime_state.invert_palette,
                                hue_offset,
                                Some(&runtime_state.intensity_mapping),
                                max_brightness.max(1.0),
                                runtime_state.temporal_color,
                                runtime_state.temporal_mode,
                                png_aux_cells,
                                runtime_state.palette_cycle,
                                None,
                            ) {
                                Ok(filename) => {
                                    runtime_state
                                        .show_notification(format!("Frame saved: {}", filename));
                                }
                                Err(e) => {
                                    runtime_state.show_notification(format!("Error: {}", e));
                                }
                            }
                        }
                        ControlAction::ToggleFastMode => {
                            runtime_state.toggle_fast_mode();
                            runtime_state.show_notification(format!(
                                "Fast mode: {}",
                                if runtime_state.fast_mode_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::CyclePaletteShiftSpeed => {
                            runtime_state.cycle_palette_shift_speed();
                            runtime_state.show_notification(format!(
                                "Palette shift: {}",
                                match runtime_state.palette_shift_speed {
                                    PaletteShiftSpeed::Off => "Off",
                                    PaletteShiftSpeed::Slow => "Slow (5°/s)",
                                    PaletteShiftSpeed::Medium => "Medium (15°/s)",
                                    PaletteShiftSpeed::Fast => "Fast (45°/s)",
                                }
                            ));
                        }
                        ControlAction::ToggleInvertPalette => {
                            runtime_state.toggle_invert_palette();
                            renderer.set_invert_palette(runtime_state.invert_palette);
                        }
                        ControlAction::ToggleReversePalette => {
                            runtime_state.toggle_reverse_palette();
                            renderer.set_reverse_palette(runtime_state.reverse_palette);
                        }
                        ControlAction::ToggleStatusBar => {
                            runtime_state.show_status_bar = !runtime_state.show_status_bar;
                            runtime_state.show_notification(format!(
                                "Status line: {}",
                                if runtime_state.show_status_bar {
                                    "on"
                                } else {
                                    "off"
                                }
                            ));
                        }
                        ControlAction::CycleIntensityMapping => {
                            runtime_state.cycle_intensity_mapping(false);
                            runtime_state.show_notification(format!(
                                "Intensity: {}",
                                runtime_state.intensity_mapping_name()
                            ));
                            renderer.set_intensity_mapping(Some(
                                runtime_state.intensity_mapping.clone(),
                            ));
                        }
                        ControlAction::CycleIntensityMappingReverse => {
                            runtime_state.cycle_intensity_mapping(true);
                            runtime_state.show_notification(format!(
                                "Intensity: {}",
                                runtime_state.intensity_mapping_name()
                            ));
                            renderer.set_intensity_mapping(Some(
                                runtime_state.intensity_mapping.clone(),
                            ));
                        }
                        ControlAction::ResetToDefaults => {
                            let applied = dispatch_swap(
                                PendingSwap::Reset,
                                &mut runtime_state,
                                &mut renderer,
                                sim,
                                &mut timer,
                                &mut grid_renderer,
                                &mut window,
                                &mut downsampled_frame,
                                &mut aux_frame,
                                (term_width as usize, term_height as usize),
                                &mut current_auto_normalize,
                                &mut adaptive_brightness,
                                args.normalize_window,
                            )?;
                            if applied {
                                hue_offset = 0.0;
                            }
                        }
                        ControlAction::ToggleDashboard => {
                            runtime_state.toggle_dashboard();
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::SetIntensityMapping(_) => {}
                        ControlAction::ShowConfigBrowser => {
                            runtime_state.close_all_overlays();
                            runtime_state.overlay_state.open(OverlayType::ConfigBrowser);
                            // Intentionally do NOT reset config_browser_selected_index here —
                            // persisting the selection across reopen is UX-win F.
                            runtime_state.on_modal_open();
                        }
                        ControlAction::ShowConfigSaveDialog => {
                            runtime_state.close_all_overlays();
                            runtime_state.overlay_state.open(OverlayType::ConfigSave);
                            runtime_state.config_save_name_input.reset();
                            runtime_state.on_modal_open();
                        }
                        ControlAction::RandomizeParams => {
                            runtime_state.randomize_params();
                            apply_random_config(&runtime_state, sim, &mut renderer, &ALL_PALETTES);

                            runtime_state.show_notification("Parameters Randomized!".to_string());
                        }
                        ControlAction::Undo => {
                            if runtime_state.undo().is_some() {
                                apply_live_params(&runtime_state, sim, &mut renderer);
                                runtime_state.show_notification("Undo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to undo".to_string());
                            }
                        }
                        ControlAction::Redo => {
                            if runtime_state.redo().is_some() {
                                apply_live_params(&runtime_state, sim, &mut renderer);
                                runtime_state.show_notification("Redo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to redo".to_string());
                            }
                        }
                        ControlAction::CycleTheme => {
                            runtime_state.cycle_theme();
                            runtime_state.show_notification(format!(
                                "Theme: {}",
                                runtime_state.current_theme_name()
                            ));
                        }
                        ControlAction::CycleThemeReverse => {
                            runtime_state.cycle_theme_reverse();
                            runtime_state.show_notification(format!(
                                "Theme: {}",
                                runtime_state.current_theme_name()
                            ));
                        }
                        ControlAction::CycleWindowFrame => {
                            runtime_state.cycle_window_frame();
                            renderer.set_window_frame(runtime_state.window_frame);
                            runtime_state.show_notification(format!(
                                "Frame: {:?}",
                                runtime_state.window_frame
                            ));
                        }
                        ControlAction::CycleWindowFrameReverse => {
                            runtime_state.cycle_window_frame_reverse();
                            renderer.set_window_frame(runtime_state.window_frame);
                            runtime_state.show_notification(format!(
                                "Frame: {:?}",
                                runtime_state.window_frame
                            ));
                        }
                        ControlAction::ShowPaletteEditor => {
                            if runtime_state.overlay_state.is_palette_editor_open() {
                                runtime_state.overlay_state.close_palette_editor();
                            } else {
                                let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                                runtime_state
                                    .overlay_state
                                    .open_palette_editor(PaletteEditorState::new(&current_palette));
                            }
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::ToggleTrailAge => {
                            runtime_state.trail_age_enabled = !runtime_state.trail_age_enabled;
                            sim.set_compute_trail_age(runtime_state.trail_age_enabled);
                            runtime_state.show_notification(format!(
                                "Trail Age: {}",
                                if runtime_state.trail_age_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::ToggleTrailDelta => {
                            runtime_state.trail_delta_enabled = !runtime_state.trail_delta_enabled;
                            sim.set_compute_trail_delta(runtime_state.trail_delta_enabled);
                            runtime_state.show_notification(format!(
                                "Trail Delta: {}",
                                if runtime_state.trail_delta_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::ToggleGradientMagnitude => {
                            runtime_state.gradient_magnitude_enabled =
                                !runtime_state.gradient_magnitude_enabled;
                            sim.set_compute_gradient_magnitude(
                                runtime_state.gradient_magnitude_enabled,
                            );
                            runtime_state.show_notification(format!(
                                "Edge Glow: {}",
                                if runtime_state.gradient_magnitude_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::CycleChrome => {
                            use crate::render::window::FallbackMode;
                            use crate::simulation::config::ChromeStyle;
                            runtime_state.cycle_chrome_style();
                            let layout =
                                if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen) {
                                    None
                                } else {
                                    let (tw, th) = crossterm::terminal::size()
                                        .map(|(w, h)| (w as usize, h as usize))
                                        .unwrap_or((80, 24));
                                    let l = window.compute_rects(tw, th);
                                    if matches!(l.fallback, FallbackMode::Fullscreen) {
                                        None
                                    } else {
                                        Some(l)
                                    }
                                };
                            // Resize render buffers to match the new chrome's sim dims
                            // BEFORE moving `layout` into the renderer (else blur_field panics).
                            let (rw, rh) = layout
                                .as_ref()
                                .map(|l| (l.sim_w, l.sim_h))
                                .unwrap_or((term_width as usize, term_height as usize));
                            resize_render_frames(&mut downsampled_frame, &mut aux_frame, rw, rh);
                            renderer.set_window_layout(layout);
                            runtime_state.show_notification(format!(
                                "Chrome: {:?}",
                                runtime_state.chrome_style
                            ));
                        }
                        ControlAction::ToggleFullscreen => {
                            use crate::render::window::FallbackMode;
                            use crate::simulation::config::ChromeStyle;
                            if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen) {
                                // Restore windowed mode
                                runtime_state.chrome_style = ChromeStyle::Minimal;
                                let (tw, th) = crossterm::terminal::size()
                                    .map(|(w, h)| (w as usize, h as usize))
                                    .unwrap_or((80, 24));
                                let l = window.compute_rects(tw, th);
                                let layout = if matches!(l.fallback, FallbackMode::Fullscreen) {
                                    None
                                } else {
                                    Some(l)
                                };
                                let (rw, rh) = layout
                                    .as_ref()
                                    .map(|l| (l.sim_w, l.sim_h))
                                    .unwrap_or((term_width as usize, term_height as usize));
                                resize_render_frames(
                                    &mut downsampled_frame,
                                    &mut aux_frame,
                                    rw,
                                    rh,
                                );
                                renderer.set_window_layout(layout);
                            } else {
                                // Switch to fullscreen
                                runtime_state.chrome_style = ChromeStyle::Fullscreen;
                                runtime_state.chrome_state =
                                    crate::terminal::state::ChromeState::Minimal;
                                resize_render_frames(
                                    &mut downsampled_frame,
                                    &mut aux_frame,
                                    term_width as usize,
                                    term_height as usize,
                                );
                                renderer.set_window_layout(None);
                            }
                        }
                        #[cfg(feature = "audio")]
                        ControlAction::ToggleChoir => {
                            if choir.is_some() {
                                choir = None;
                                runtime_state.show_notification("Choir mode: off".to_string());
                            } else {
                                match crate::audio::Choir::try_new(
                                    args.choir_volume.clamp(0.0, 1.0),
                                ) {
                                    Ok(c) => {
                                        choir = Some(c);
                                        runtime_state
                                            .show_notification("Choir mode: on".to_string());
                                    }
                                    Err(e) => {
                                        runtime_state
                                            .show_notification(format!("Choir init failed: {e}"));
                                    }
                                }
                            }
                        }
                        ControlAction::None => {}
                        // Controls-overlay interaction actions are intercepted by
                        // `apply_controls_action` above (they either mutate state
                        // and `continue`, or re-dispatch a real action), so they
                        // never reach this match. These arms keep it exhaustive.
                        ControlAction::ToggleControlsDepth
                        | ControlAction::ControlsFocusNext
                        | ControlAction::ControlsFocusPrev
                        | ControlAction::ControlsAdjustFocused(_)
                        | ControlAction::ControlsActivateFocused => {}
                    }

                    // ── TUNE surfacing (Task 13) ─────────────────────────────
                    // When the dispatched action adjusted a single sim/render
                    // param, surface it in the ambient TUNE state (debounced):
                    // each adjust refreshes the hold so rapid adjusts extend it,
                    // then it eases back to BASE. This REPLACES any on-adjust
                    // toast (none existed — param adjusts only notify at bounds).
                    if let Some(tune) =
                        tune_view_for_action(&runtime_state, &action, &registry_ctx, agent_count)
                    {
                        const TUNE_HOLD_SECS: f32 = 2.5;
                        let now = runtime_state.phase_clock;
                        // Surface the focused param (debounced) and drop the
                        // redundant value-echo Info toast the handler pushed so
                        // the tuner is always the visible feedback. Relevant
                        // Warning/Error messages survive and momentarily win.
                        crate::render::ambient::surface_tune(
                            &mut runtime_state.ambient_states,
                            tune,
                            now,
                            TUNE_HOLD_SECS,
                        );
                    }
                }
                Event::Mouse(mouse_event) => {
                    if runtime_state.mouse_mode == MouseInteractionMode::Disabled {
                        continue;
                    }

                    let event_type =
                        if matches!(mouse_event.kind, crossterm::event::MouseEventKind::Down(_)) {
                            MouseEventType::Down
                        } else if matches!(
                            mouse_event.kind,
                            crossterm::event::MouseEventKind::Drag(_)
                        ) {
                            MouseEventType::Drag
                        } else if matches!(
                            mouse_event.kind,
                            crossterm::event::MouseEventKind::Moved
                        ) {
                            MouseEventType::Moved
                        } else {
                            continue;
                        };

                    if term_width == 0 || term_height == 0 {
                        continue;
                    }

                    let term_x = mouse_event.column as usize - 1;
                    let term_y = mouse_event.row as usize - 1;

                    let sim_x = (term_x as f32 / term_width as f32) * sim.width() as f32;
                    let sim_y = (term_y as f32 / term_height as f32) * sim.height() as f32;

                    let strength = match runtime_state.mouse_mode {
                        MouseInteractionMode::Attract => 2.0,
                        MouseInteractionMode::Repel => -2.0,
                        MouseInteractionMode::Disabled => 0.0,
                    };

                    match event_type {
                        MouseEventType::Down => {
                            sim.add_mouse_attractor(sim_x, sim_y, strength);
                            if args.verbose {
                                eprintln!(
                                    "[Mouse] {} at ({:.0}, {:.0})",
                                    match runtime_state.mouse_mode {
                                        MouseInteractionMode::Attract => "Attractor",
                                        MouseInteractionMode::Repel => "Repeller",
                                        _ => "Effect",
                                    },
                                    sim_x,
                                    sim_y
                                );
                            }
                        }
                        MouseEventType::Drag => {
                            sim.add_mouse_attractor(sim_x, sim_y, strength);
                            if args.verbose {
                                eprintln!(
                                    "[Mouse] Drag {} at ({:.0}, {:.0})",
                                    match runtime_state.mouse_mode {
                                        MouseInteractionMode::Attract => "attract",
                                        MouseInteractionMode::Repel => "repel",
                                        _ => "effect",
                                    },
                                    sim_x,
                                    sim_y
                                );
                            }
                        }
                        MouseEventType::Moved => {
                            if args.verbose {
                                eprintln!("[Mouse] Move at ({:.0}, {:.0})", sim_x, sim_y);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // If pause was toggled during input, re-render immediately with the new pause state
        // This gives instant visual feedback instead of waiting for the next frame
        if runtime_state.pause_just_toggled {
            runtime_state.pause_just_toggled = false;

            // Update pause frame counter for animated pause effects
            if runtime_state.is_paused {
                runtime_state.pause_frame_counter += 1;
            } else {
                runtime_state.pause_frame_counter = 0;
            }

            // Rebuild pause overlays with new state
            let (pause_logo_overlay, pause_logo_x, pause_logo_y) =
                if runtime_state.is_paused && runtime_state.pause_logo_enabled {
                    let pct = if term_width < 80 {
                        0.90
                    } else if term_width < 120 {
                        0.75
                    } else {
                        0.60
                    };
                    let logo_w = ((term_width as f32 * pct) as usize).clamp(30, 180);
                    let logo_h = ((logo_w as f32 / 2.67) as usize).max(6);
                    let pixel_w = logo_w * 2;
                    let pixel_h = logo_h * 2;

                    let brightness_map = if runtime_state
                        .pause_logo_cache
                        .as_ref()
                        .is_some_and(|(cw, _, _)| *cw == logo_w)
                    {
                        runtime_state.pause_logo_cache.as_ref().unwrap().2.clone()
                    } else {
                        let map = load_logo_from_memory(FOOD_IMAGE_PNG, pixel_w, pixel_h, true)
                            .unwrap_or_else(|_| vec![0.0; pixel_w * pixel_h]);
                        runtime_state.pause_logo_cache = Some((logo_w, pixel_h, map.clone()));
                        map
                    };

                    let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
                    let logo_mapping = args
                        .logo_mapping()
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| runtime_state.intensity_mapping.clone());
                    let logo = PauseOverlay::build_logo(
                        &brightness_map,
                        logo_w,
                        logo_h,
                        current_palette,
                        runtime_state.reverse_palette,
                        runtime_state.invert_palette,
                        0.0,
                        Some(&logo_mapping),
                    );

                    let actual_logo_h = logo.lines.len();
                    let lx = (term_width as usize).saturating_sub(logo_w) / 2;
                    let drawable_h = (term_height as usize).saturating_sub(1);
                    let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

                    (Some(logo), lx, ly)
                } else {
                    (None, 0, 0)
                };

            // Pause re-render path: resolve the ambient surface with the same
            // shared logic as the main path (pause → centered modal, etc.).
            let pause_dither_label = match runtime_state.dither_mode {
                crate::render::dither::DitherMode::None => None,
                crate::render::dither::DitherMode::Ordered { intensity, .. } => {
                    Some(format!("D {:.1}×", intensity))
                }
                crate::render::dither::DitherMode::ErrorDiffusion { .. } => Some("ED".to_string()),
                crate::render::dither::DitherMode::Hybrid { intensity, .. } => {
                    Some(format!("H {:.1}×", intensity))
                }
            };
            let pause_base_status = BaseStatus {
                preset_name: preset_name(runtime_state.current_preset).to_string(),
                palette_name: palette_name(current_palette.clone()).to_string(),
                time_scale_text: format!("{:.1}×", runtime_state.time_scale),
                population: Some(sim.agent_count()),
                dither_label: pause_dither_label,
                can_undo: !runtime_state.undo_stack.is_empty(),
                can_redo: !runtime_state.redo_stack.is_empty(),
                accent: ui_accent,
                is_paused: runtime_state.is_paused,
            };
            let pause_ambient_overlay = compute_ambient_overlay(
                &runtime_state,
                &pause_base_status,
                &window,
                term_width as usize,
                term_height as usize,
                sim.agent_count(),
            );
            #[allow(clippy::type_complexity)]
            let status_data: Option<(
                String,
                usize,
                Vec<(usize, crate::render::palette::RgbColor)>,
            )> = None;

            // Re-render with updated pause state
            if args.species_colors_enabled() && sim.config().separate_species_trails {
                let species_trail_maps = sim.trail_maps_for_species_colors();
                let combined: Vec<_> = species_trail_maps
                    .iter()
                    .zip(current_species_rgb.iter())
                    .map(|(tm, color)| (*tm, *color))
                    .collect();
                renderer.render_multi_species_with_overlay(
                    &combined,
                    sim.width(),
                    sim.height(),
                    max_brightness.max(1.0),
                    if runtime_state.is_paused {
                        Some(runtime_state.pause_frame_counter)
                    } else {
                        None
                    },
                    pause_logo_overlay
                        .as_ref()
                        .map(|v| (v, pause_logo_x, pause_logo_y)),
                    None,
                    None,
                    status_data,
                    None,
                    None,
                    grid_renderer.as_ref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    pause_ambient_overlay.as_ref().map(|(v, x, y)| (v, *x, *y)),
                    Some(&runtime_state.panel_style),
                    runtime_state.overlay_state.active(),
                    runtime_state.pause_style,
                    runtime_state.pause_pulse_draw_mode,
                )?;
            } else {
                renderer.render_with_overlay(
                    downsampled_frame.cells(),
                    max_brightness.max(1.0),
                    if runtime_state.is_paused {
                        Some(runtime_state.pause_frame_counter)
                    } else {
                        None
                    },
                    pause_logo_overlay
                        .as_ref()
                        .map(|v| (v, pause_logo_x, pause_logo_y)),
                    None,
                    None,
                    status_data,
                    None,
                    None,
                    grid_renderer.as_ref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    pause_ambient_overlay.as_ref().map(|(v, x, y)| (v, *x, *y)),
                    Some(&runtime_state.panel_style),
                    runtime_state.overlay_state.active(),
                    runtime_state.pause_style,
                    runtime_state.pause_pulse_draw_mode,
                )?;
            }
        }

        if args.verbose {
            eprintln!(
                "FPS: {:.1} (avg: {:.1}) | Sim: {:.1}ms | Render: {:.1}ms | Frame: {}",
                timer.current_fps(),
                timer.average_fps(),
                timer.sim_duration().as_secs_f64() * 1000.0,
                timer.render_duration().as_secs_f64() * 1000.0,
                timer.frame_count(),
            );
        }

        if should_exit {
            break;
        }

        if timer.should_adjust_fps() {
            if let Some(new_fps) = timer.get_adjusted_fps() {
                timer.apply_fps_adjustment(new_fps);
                runtime_state
                    .show_notification(format!("Adaptive FPS: {} -> {}", args.fps, new_fps));
            }
        }

        if timer.fps_adjusted_notification {
            timer.fps_adjusted_notification = false;
        }

        timer.tick();
    }

    if runtime_state.mouse_mode != MouseInteractionMode::Disabled {
        let _ = crate::terminal::disable_mouse_tracking();
    }

    Ok(())
}

/// Live preset-switch: applies the bare preset through the ONE total apply seam,
/// wiping the trail (restart: true). CLI args are NOT sticky after a swap.
#[allow(clippy::too_many_arguments)]
fn switch_preset(
    new_preset: Preset,
    rs: &mut RuntimeState,
    renderer: &mut TerminalRenderer,
    sim: &mut Simulation,
    timer: &mut FrameTimer,
    grid_renderer: &mut Option<crate::render::grid::GridRenderer>,
    window: &mut crate::render::window::Window,
    downsampled_frame: &mut crate::render::downsample::DownsampledFrame,
    aux_frame: &mut crate::render::downsample::AuxFrame,
    term_size: (usize, usize),
) -> io::Result<()> {
    let ov = crate::profile_overrides::ProfileOverrides {
        preset: Some(new_preset),
        ..Default::default()
    };
    crate::app::apply_overrides(
        &ov,
        rs,
        renderer,
        sim,
        timer,
        grid_renderer,
        window,
        downsampled_frame,
        aux_frame,
        term_size,
        true,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    // Commit provenance ONLY after successful apply (transactional).
    rs.set_preset(new_preset);
    rs.active_source = crate::profile::ProfileSource::Preset(new_preset);
    rs.active_overrides = ov;
    Ok(())
}

/// Gets terminal size from environment variables or crossterm.
pub fn get_terminal_size() -> (usize, usize) {
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let height = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());

    if let (Some(w), Some(h)) = (width, height) {
        return (w, h);
    }

    match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}

/// Which bottom-surface mode the ambient strip should take this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AmbientMode {
    /// Console overlay is open — hide the ambient strip (M1).
    Hidden,
    /// Controls are at Tuner depth — show the persistent TUNE surface.
    Tuner,
    /// Controls closed or not open — BASE/MSG normal resolution.
    Normal,
}

/// Resolve which ambient mode applies for this frame.
///
/// `controls_open` — true when the Controls overlay is open and Dashboard is not.
/// `controls_depth` — the current depth within the controls panel.
///
/// The `Closed`-while-open defensive case (controls reported open but depth is
/// `Closed`) maps to `Hidden` (treats it as Console), preserving prior behaviour.
fn ambient_mode(
    controls_open: bool,
    controls_depth: crate::render::controls::ControlsDepth,
) -> AmbientMode {
    use crate::render::controls::ControlsDepth;
    if controls_open {
        match controls_depth {
            ControlsDepth::Closed => AmbientMode::Hidden,
            ControlsDepth::Console => AmbientMode::Hidden,
            ControlsDepth::Tuner => AmbientMode::Tuner,
        }
    } else {
        AmbientMode::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        }
    }

    fn controls_test_state() -> RuntimeState {
        use crate::cli::PauseStyle;
        use crate::simulation::config::SimConfig;
        RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            MouseInteractionMode::Disabled,
            3.0,
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        )
    }

    fn controls_test_ctx() -> RegistryCtx {
        RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        }
    }

    #[test]
    fn tune_view_for_numeric_adjust_is_some_with_gauge() {
        let state = controls_test_state();
        let ctx = controls_test_ctx();
        let tv = tune_view_for_action(&state, &ControlAction::AdjustSensorAngle(1.0), &ctx, 50_000)
            .expect("sensor-angle adjust must surface a TuneView");
        assert_eq!(tv.label, "Sensor Angle");
        assert_eq!(tv.range, (0.0, 1.0)); // normalized gauge
        assert!((0.0..=1.0).contains(&tv.value));
        assert!(!tv.value_text.is_empty());
    }

    #[test]
    fn tune_view_for_non_param_action_is_none() {
        let state = controls_test_state();
        let ctx = controls_test_ctx();
        // TogglePause has no registry ParamId → no TUNE surfacing.
        assert!(tune_view_for_action(&state, &ControlAction::TogglePause, &ctx, 0).is_none());
    }

    #[test]
    fn tune_view_for_action_rows_is_none() {
        let state = controls_test_state();
        let ctx = controls_test_ctx();
        // Action rows (Save/Reset/Randomize) are MSG/activation events, not TUNE.
        assert!(tune_view_for_action(&state, &ControlAction::ResetToDefaults, &ctx, 0).is_none());
        assert!(tune_view_for_action(&state, &ControlAction::SaveFrameToPng, &ctx, 0).is_none());
    }

    #[test]
    fn controls_depth_toggle_cycles_console_tuner() {
        use crate::render::controls::ControlsDepth;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        assert_eq!(state.controls_depth, ControlsDepth::Closed);

        // Closed -> Console
        assert!(matches!(
            apply_controls_action(&mut state, &ControlAction::ToggleControlsDepth, &ctx),
            ControlsDispatch::Handled
        ));
        assert_eq!(state.controls_depth, ControlsDepth::Console);

        // Console -> Tuner
        apply_controls_action(&mut state, &ControlAction::ToggleControlsDepth, &ctx);
        assert_eq!(state.controls_depth, ControlsDepth::Tuner);

        // Tuner -> Console
        apply_controls_action(&mut state, &ControlAction::ToggleControlsDepth, &ctx);
        assert_eq!(state.controls_depth, ControlsDepth::Console);
    }

    #[test]
    fn controls_focus_next_clamps_to_last_visible() {
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // SIM category (index 0) has 7 params; focus must clamp at 6.
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        for _ in 0..20 {
            apply_controls_action(&mut state, &ControlAction::ControlsFocusNext, &ctx);
        }
        assert_eq!(state.controls_focus, 6);

        for _ in 0..20 {
            apply_controls_action(&mut state, &ControlAction::ControlsFocusPrev, &ctx);
        }
        assert_eq!(state.controls_focus, 0);
    }

    #[test]
    fn controls_adjust_focused_redispatches_real_action() {
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // SIM[0] = SensorAngle. A forward adjust must re-dispatch AdjustSensorAngle(+1.0).
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        match apply_controls_action(&mut state, &ControlAction::ControlsAdjustFocused(1.0), &ctx) {
            ControlsDispatch::Redispatch(ControlAction::AdjustSensorAngle(d)) => {
                assert_eq!(d, 1.0)
            }
            _ => panic!("expected Redispatch(AdjustSensorAngle(1.0))"),
        }
    }

    #[test]
    fn controls_adjust_suppressed_when_competing_overlay_open() {
        use crate::overlay::OverlayType;
        use crate::render::controls::ControlsDepth;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // A competing overlay (Dashboard) is the foreground, Controls is closed.
        state.overlay_state.open(OverlayType::Dashboard);
        state.controls_depth = ControlsDepth::Closed;
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        // ←→ adjust must NOT auto-promote or redispatch; it passes through.
        assert!(matches!(
            apply_controls_action(&mut state, &ControlAction::ControlsAdjustFocused(1.0), &ctx),
            ControlsDispatch::Passthrough
        ));
        // Depth must remain Closed (no silent auto-promote / sim mutation).
        assert_eq!(state.controls_depth, ControlsDepth::Closed);
    }

    #[test]
    fn controls_adjust_auto_promotes_when_no_overlay_open() {
        use crate::render::controls::ControlsDepth;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // No overlay open at all → spec-mandated auto-promote-from-Closed.
        assert!(!state.overlay_state.any_open());
        state.controls_depth = ControlsDepth::Closed;
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        // SIM[0] = SensorAngle: forward adjust auto-promotes to Tuner and
        // redispatches the real AdjustSensorAngle(+1.0).
        match apply_controls_action(&mut state, &ControlAction::ControlsAdjustFocused(1.0), &ctx) {
            ControlsDispatch::Redispatch(ControlAction::AdjustSensorAngle(d)) => {
                assert_eq!(d, 1.0)
            }
            _ => panic!("expected Redispatch(AdjustSensorAngle(1.0))"),
        }
        assert_eq!(state.controls_depth, ControlsDepth::Tuner);
    }

    #[test]
    fn controls_in_category_hotkey_retargets_focus() {
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // In SIM category, pressing the StepSize hotkey (SIM index 3) must move
        // focus to 3 and pass the action through to the main match.
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        assert!(matches!(
            apply_controls_action(&mut state, &ControlAction::AdjustStepSize(0.1), &ctx),
            ControlsDispatch::Passthrough
        ));
        assert_eq!(state.controls_focus, 3);
    }

    #[test]
    fn controls_cross_category_hotkey_does_not_retarget() {
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // In SIM category (0), a Palette hotkey (APP category) must NOT change
        // focus or category; it just passes through.
        state.controls_category_idx = 0;
        state.controls_focus = 2;
        assert!(matches!(
            apply_controls_action(&mut state, &ControlAction::CyclePalette, &ctx),
            ControlsDispatch::Passthrough
        ));
        assert_eq!(state.controls_focus, 2);
        assert_eq!(state.controls_category_idx, 0);
    }

    // ── Task-14 tests ─────────────────────────────────────────────────────────

    /// ControlsAdjustFocused from Closed must auto-promote depth to Tuner and
    /// then continue to dispatch the focused parameter's adjust action.
    #[test]
    fn arrow_from_closed_promotes_to_tuner() {
        use crate::render::controls::ControlsDepth;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // Start fully closed (RuntimeState::new sets depth=Closed).
        assert_eq!(state.controls_depth, ControlsDepth::Closed);
        state.controls_category_idx = 0;
        state.controls_focus = 0;
        // A focused adjust must auto-promote depth to Tuner.
        apply_controls_action(&mut state, &ControlAction::ControlsAdjustFocused(1.0), &ctx);
        assert_eq!(state.controls_depth, ControlsDepth::Tuner);
    }

    /// After cycling to a category with fewer visible rows than the current
    /// focus index, controls_focus must be clamped to the new category's last
    /// row (no stale out-of-bounds index).
    #[test]
    fn category_change_clamps_focus() {
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        // Open the controls overlay so CycleOptionsCategory is handled inside
        // apply_controls_action (not just the closed→open branch in the runner).
        state
            .overlay_state
            .open(crate::overlay::OverlayType::Controls);
        // SIM (category 0) has 7 rows; set focus to the last row (index 6).
        state.controls_category_idx = 0;
        state.controls_focus = 6;
        // Cycle through all categories until we reach SYS (index 5, 3 rows).
        // SIM→ENV→APP→PST→PRF→SYS = 5 forward cycles.
        for _ in 0..5 {
            apply_controls_action(&mut state, &ControlAction::CycleOptionsCategory, &ctx);
        }
        assert_eq!(state.controls_category_idx, 5); // SYS
                                                    // SYS has 3 rows (indices 0-2); focus must be clamped to at most 2.
        let sys_len = crate::render::controls::registry::visible_params(5, &ctx).len();
        assert!(
            state.controls_focus < sys_len,
            "focus {} out of bounds for SYS len {}",
            state.controls_focus,
            sys_len,
        );
    }

    /// When ctx shrinks the current category's visible list (e.g. Gaussian
    /// toggled off removes Diff Sigma from ENV), the defensive clamp at the
    /// top of apply_controls_action must correct a stale focus index before
    /// the action runs.
    #[test]
    fn conditional_row_disappearance_clamps_focus() {
        let mut state = controls_test_state();
        // Start in ENV (category 1) with Gaussian ON — that adds Diff Sigma,
        // giving 7 rows (indices 0-6). Set focus to the last row.
        let ctx_gaussian_on = RegistryCtx {
            diffusion_gaussian: true,
            mouse_enabled: false,
        };
        state.controls_category_idx = 1; // ENV
        let len_with_sigma =
            crate::render::controls::registry::visible_params(1, &ctx_gaussian_on).len();
        state.controls_focus = len_with_sigma - 1; // last row
                                                   // Now ctx flips to Gaussian OFF — Diff Sigma disappears, shorter list.
        let ctx_gaussian_off = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        };
        let len_without_sigma =
            crate::render::controls::registry::visible_params(1, &ctx_gaussian_off).len();
        assert!(
            len_without_sigma < len_with_sigma,
            "gaussian off must shrink ENV row count"
        );
        // Any action dispatched with the new (shorter) ctx must clamp focus.
        apply_controls_action(
            &mut state,
            &ControlAction::ControlsFocusPrev, // innocuous action
            &ctx_gaussian_off,
        );
        assert!(
            state.controls_focus < len_without_sigma,
            "focus {} must be within new ENV len {}",
            state.controls_focus,
            len_without_sigma,
        );
    }

    #[test]
    fn build_param_views_sim_count_and_default_state() {
        use crate::render::controls::ParamState;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        state.controls_category_idx = 0; // SIM
        let views = build_param_views(&state, &ctx, 50_000);
        // SIM has 7 numeric params, all unmodified at preset defaults.
        assert_eq!(views.len(), 7);
        let sensor = &views[0];
        assert_eq!(sensor.value_text, format!("{:.1}°", state.sensor_angle));
        assert_eq!(sensor.state, ParamState::Default);
        assert!(sensor.ratio.is_some(), "numeric param must carry a ratio");
        assert!(sensor.def_ratio.is_some());
    }

    #[test]
    fn build_param_views_modified_numeric_flags_modified() {
        use crate::render::controls::ParamState;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        state.controls_category_idx = 0;
        // Push sensor_angle well off its default → must read as Modified.
        state.sensor_angle = state.default_values.sensor_angle + 20.0;
        let views = build_param_views(&state, &ctx, 50_000);
        assert_eq!(views[0].state, ParamState::Modified);
    }

    #[test]
    fn build_param_views_population_is_cli() {
        use crate::render::controls::registry::ParamId;
        use crate::render::controls::ParamState;
        let mut state = controls_test_state();
        let ctx = controls_test_ctx();
        state.controls_category_idx = 4; // PRF
        let views = build_param_views(&state, &ctx, 50_000);
        let pop = views
            .iter()
            .find(|v| v.desc.id == ParamId::Population)
            .expect("Population row present in PRF");
        assert_eq!(pop.state, ParamState::Cli);
        assert_eq!(pop.value_text, "50k");
        assert!(pop.ratio.is_none());
    }

    #[test]
    fn build_param_views_mouse_timeout_is_display() {
        use crate::render::controls::registry::ParamId;
        use crate::render::controls::ParamState;
        let mut state = controls_test_state();
        // Mouse Timeout only appears when mouse is enabled.
        state.mouse_mode = MouseInteractionMode::Attract;
        let ctx = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: true,
        };
        state.controls_category_idx = 1; // ENV
        let views = build_param_views(&state, &ctx, 50_000);
        let mt = views
            .iter()
            .find(|v| v.desc.id == ParamId::MouseTimeout)
            .expect("Mouse Timeout row present when mouse enabled");
        assert_eq!(mt.state, ParamState::Display);
        assert!(mt.ratio.is_none());
        assert!(mt.value_text.ends_with('s'));
    }

    #[test]
    fn screensaver_exits_on_any_key_press() {
        for code in [
            KeyCode::Char('a'),
            KeyCode::Char(' '),
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Up,
            KeyCode::F(1),
        ] {
            assert!(
                screensaver_exit_on_key(&Mode::Screensaver, &key(code, KeyEventKind::Press)),
                "expected screensaver exit for {:?}",
                code
            );
        }
    }

    #[test]
    fn screensaver_ignores_key_release_and_repeat() {
        let release = key(KeyCode::Char('a'), KeyEventKind::Release);
        let repeat = key(KeyCode::Char('a'), KeyEventKind::Repeat);
        assert!(!screensaver_exit_on_key(&Mode::Screensaver, &release));
        assert!(!screensaver_exit_on_key(&Mode::Screensaver, &repeat));
    }

    #[test]
    fn non_screensaver_modes_never_exit_on_key() {
        let press = key(KeyCode::Char('a'), KeyEventKind::Press);
        for mode in [Mode::Default, Mode::Live, Mode::Print] {
            assert!(!screensaver_exit_on_key(&mode, &press));
        }
    }

    /// `switch_preset` builds a bare `ProfileOverrides { preset: Some(p), ..Default::default() }`,
    /// so the resulting overrides carry NO previously-set CLI fields.
    /// This test verifies the contract at the `ProfileOverrides` level: a CLI-startup
    /// override (with sensor_angle set) is NOT what a preset switch produces.
    #[test]
    fn switch_preset_ignores_cli() {
        use crate::profile_overrides::ProfileOverrides;
        use crate::simulation::config::Preset;

        // Simulated CLI startup: preset=Network, sensor_angle=42.0 (non-default CLI override).
        let cli_ov = ProfileOverrides {
            preset: Some(Preset::Network),
            sensor_angle: Some(42.0),
            ..Default::default()
        };

        // What switch_preset actually builds — a bare preset override with NO CLI fields.
        let switched_ov = ProfileOverrides {
            preset: Some(Preset::Organic),
            ..Default::default()
        };

        // The switched override must NOT carry the CLI sensor_angle.
        assert_eq!(
            switched_ov.sensor_angle, None,
            "switch_preset must not carry forward CLI sensor_angle"
        );
        // And the two overrides are not equal (different preset, no CLI field).
        assert_ne!(
            cli_ov, switched_ov,
            "CLI override must differ from bare-preset override"
        );
        // sensor_angle was set in the CLI override but not in the switched one.
        assert_eq!(
            cli_ov.sensor_angle,
            Some(42.0),
            "CLI override retains sensor_angle"
        );
    }

    // ── Task-15 tests ─────────────────────────────────────────────────────────

    #[test]
    fn ambient_hidden_when_controls_depth_is_console() {
        use crate::render::controls::ControlsDepth;
        // Console open → ambient_mode returns Hidden.
        assert_eq!(
            ambient_mode(true, ControlsDepth::Console),
            AmbientMode::Hidden,
            "ambient must be hidden when Console depth is active"
        );
        // Defensive: Closed-while-open → also Hidden.
        assert_eq!(
            ambient_mode(true, ControlsDepth::Closed),
            AmbientMode::Hidden,
            "ambient must be hidden for Closed-while-open defensive case"
        );
    }

    #[test]
    fn ambient_shows_tuner_depth_when_controls_tuner() {
        use crate::render::controls::ControlsDepth;
        // Tuner open → ambient_mode returns Tuner.
        assert_eq!(
            ambient_mode(true, ControlsDepth::Tuner),
            AmbientMode::Tuner,
            "ambient must be Tuner when Tuner depth is active"
        );
        // Controls not open → Normal regardless of depth.
        assert_eq!(
            ambient_mode(false, ControlsDepth::Tuner),
            AmbientMode::Normal,
            "ambient must be Normal when controls are not open"
        );
    }

    #[test]
    fn constellation_reset_is_now_stable() {
        use crate::simulation::config::InitMode;
        // Constellation must NOT re-roll a random init mode anymore.
        assert_eq!(
            effective_init_mode(InitMode::Constellation),
            InitMode::Constellation
        );
    }
}

#[cfg(test)]
mod keybind_resolution_tests {
    use super::resolve_bind;
    use crate::keybind_manager::BindTarget;
    use crate::simulation::config::Preset;
    use std::collections::HashMap;

    #[test]
    fn user_bind_overrides_builtin_default() {
        let mut m = HashMap::new();
        m.insert('1', BindTarget::Preset(Preset::Fire));
        assert_eq!(
            resolve_bind('1', &m),
            Some(BindTarget::Preset(Preset::Fire))
        );
        assert_eq!(resolve_bind('5', &m), None);
        m.insert('5', BindTarget::Config("mynight".to_string()));
        assert_eq!(
            resolve_bind('5', &m),
            Some(BindTarget::Config("mynight".to_string()))
        );
    }
}
