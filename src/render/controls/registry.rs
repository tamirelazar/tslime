//! Parameter kinds and descriptors for the Controls Instrument UI — the set of
//! parameters shown and edited in the Tuner and Console surfaces.

/// The kind of control parameter — determines how it's displayed and interacted with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamKind {
    /// Numeric parameter with a continuous range.
    Numeric,
    /// Enumerated parameter with discrete choices.
    Enum,
    /// Boolean toggle parameter.
    Toggle,
    /// Action parameter (button/command).
    Action,
    /// Read-only parameter from CLI args.
    CliReadonly,
    /// Display-only parameter (not directly editable).
    Display,
}

/// Unique identifier for each parameter in the simulation and rendering system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamId {
    /// Sensor angle.
    SensorAngle,
    /// Sensor distance.
    SensorDistance,
    /// Turn angle.
    TurnAngle,
    /// Step size.
    StepSize,
    /// Trail decay factor.
    Decay,
    /// Deposit amount.
    Deposit,
    /// Time scale.
    TimeScale,
    /// Diffusion kernel.
    DiffusionKernel,
    /// Gaussian diffusion sigma.
    DiffusionSigma,
    /// Wind direction.
    Wind,
    /// Terrain type.
    TerrainType,
    /// Terrain strength.
    TerrainStrength,
    /// Attractor strength.
    Attractor,

    /// Mouse interaction mode.
    MouseMode,
    /// Mouse idle timeout.
    MouseTimeout,
    /// UI theme.
    Theme,
    /// Color palette.
    Palette,
    /// Glyph charset.
    Charset,
    /// Color anti-aliasing.
    ColorAa,
    /// Palette shift speed.
    PaletteShift,
    /// Invert palette.
    Invert,
    /// Reverse palette.
    Reverse,
    /// Always-on status line (ambient base row) toggle.
    StatusLine,
    /// Window frame style (None / Accented / Glow / Frame).
    WindowFrame,
    /// Window chrome mode (Minimal / Expanded / Fullscreen).
    Chrome,
    /// Dither mode (dev-only).
    Dither,
    /// Intensity (tone) mapping curve.
    IntensityMapping,
    /// Auto-normalize brightness.
    AutoNormalize,
    /// Motion blur.
    MotionBlur,
    /// Brightness.
    Brightness,
    /// Trail-age coloring.
    TrailAge,
    /// Trail-delta coloring.
    TrailDelta,
    /// Edge glow.
    EdgeGlow,
    /// Fast mode.
    FastMode,

    /// Agent population (CLI-only, read-only at runtime).
    Population,
    /// Save current frame to PNG.
    SaveFrame,
    /// Reset to defaults.
    Reset,
    /// Randomize parameters.
    Randomize,
}

/// Descriptor for a parameter: metadata required for UI rendering and interaction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParamDesc {
    /// Unique identifier for this parameter.
    pub id: ParamId,
    /// Keyboard hint (e.g., "A/a", "─").
    pub key_hint: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// The kind of parameter (determines how it's rendered and interacted with).
    pub kind: ParamKind,
}

/// Context needed to determine which parameters are currently visible.
/// Passed to [`visible_params`] and [`locate`] instead of the full `RuntimeState`
/// to keep the registry testable in isolation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RegistryCtx {
    /// Whether the Gaussian diffusion kernel is active (shows Diff Sigma row when true).
    pub diffusion_gaussian: bool,
    /// Whether mouse interaction is enabled (shows Mouse Timeout row when true).
    pub mouse_enabled: bool,
}

/// Short names for each control category, matching the tab strip in the overlay.
pub const CATEGORY_NAMES: [&str; 6] = ["SIM", "ENV", "APP", "PST", "PRF", "SYS"];

/// Returns the ordered list of visible parameters for the given category (0..6).
///
/// Category indices: 0=SIM, 1=ENV, 2=APP, 3=PST, 4=PRF, 5=SYS.
/// Conditional rows: `DiffusionSigma` appears only when `ctx.diffusion_gaussian` is true;
/// `MouseTimeout` appears only when `ctx.mouse_enabled` is true.
pub fn visible_params(category: usize, ctx: &RegistryCtx) -> Vec<ParamDesc> {
    match category {
        // ── Category 0: SIM — Simulation Core ────────────────────────────────
        0 => vec![
            ParamDesc {
                id: ParamId::SensorAngle,
                key_hint: "A/a",
                label: "Sensor Angle",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::SensorDistance,
                key_hint: "J/j",
                label: "Sensor Dist",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::TurnAngle,
                key_hint: "T/t",
                label: "Turn Angle",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::StepSize,
                key_hint: "S/s",
                label: "Step Size",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::Decay,
                key_hint: "E/e",
                label: "Decay Factor",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::Deposit,
                key_hint: "I/i",
                label: "Deposit Amt",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::TimeScale,
                key_hint: "+/-",
                label: "Time Scale",
                kind: ParamKind::Numeric,
            },
        ],
        // ── Category 1: ENV — Forces & Environment ───────────────────────────
        1 => {
            let mut v = vec![ParamDesc {
                id: ParamId::DiffusionKernel,
                key_hint: "K",
                label: "Diffusion",
                kind: ParamKind::Enum,
            }];
            if ctx.diffusion_gaussian {
                v.push(ParamDesc {
                    id: ParamId::DiffusionSigma,
                    key_hint: ";/:",
                    label: "Diff Sigma",
                    kind: ParamKind::Numeric,
                });
            }
            v.push(ParamDesc {
                id: ParamId::Wind,
                key_hint: "W",
                label: "Wind",
                kind: ParamKind::Enum,
            });
            v.push(ParamDesc {
                id: ParamId::TerrainType,
                key_hint: "U",
                label: "Terrain Type",
                kind: ParamKind::Enum,
            });
            v.push(ParamDesc {
                id: ParamId::TerrainStrength,
                key_hint: "Y/y",
                label: "Terrain Str",
                kind: ParamKind::Numeric,
            });
            v.push(ParamDesc {
                id: ParamId::Attractor,
                key_hint: "L/l",
                label: "Attractor",
                kind: ParamKind::Numeric,
            });
            v.push(ParamDesc {
                id: ParamId::MouseMode,
                key_hint: ",",
                label: "Mouse Mode",
                kind: ParamKind::Enum,
            });
            if ctx.mouse_enabled {
                v.push(ParamDesc {
                    id: ParamId::MouseTimeout,
                    key_hint: "─",
                    label: "Mouse Timeout",
                    kind: ParamKind::Display,
                });
            }
            v
        }
        // ── Category 2: APP — Appearance ─────────────────────────────────────
        2 => vec![
            ParamDesc {
                id: ParamId::Theme,
                key_hint: "9/*",
                label: "Theme",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::Palette,
                key_hint: "c/C",
                label: "Palette",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::Charset,
                key_hint: "`/~",
                label: "Charset",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::ColorAa,
                key_hint: "\"",
                label: "Color AA",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::PaletteShift,
                key_hint: "O",
                label: "Palette Shift",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::Invert,
                key_hint: "X",
                label: "Invert",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::Reverse,
                key_hint: "Z",
                label: "Reverse",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::StatusLine,
                key_hint: "←→",
                label: "Status Line",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::WindowFrame,
                key_hint: ")/(",
                label: "Frame",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::Chrome,
                key_hint: "F10",
                label: "Chrome",
                kind: ParamKind::Enum,
            },
        ],
        // ── Category 3: PST — Post-Processing ────────────────────────────────
        3 => vec![
            ParamDesc {
                id: ParamId::IntensityMapping,
                key_hint: "m/M",
                label: "Intensity",
                kind: ParamKind::Enum,
            },
            ParamDesc {
                id: ParamId::Dither,
                key_hint: "d/D",
                label: "Dither (dev)",
                // Display (inert, muted): matches the old "(dev)" row semantics.
                // It is technically live-adjustable via {/} when dither is
                // unlocked, but is surfaced as a read-only display value here —
                // not CLI-readonly (which would falsely read "restart to change").
                kind: ParamKind::Display,
            },
            ParamDesc {
                id: ParamId::AutoNormalize,
                key_hint: "B",
                label: "Auto Normalize",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::MotionBlur,
                key_hint: "V",
                label: "Motion Blur",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::Brightness,
                key_hint: "N/n",
                label: "Brightness",
                kind: ParamKind::Numeric,
            },
            ParamDesc {
                id: ParamId::TrailAge,
                key_hint: "'",
                label: "Trail Age",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::TrailDelta,
                key_hint: ".",
                label: "Trail Delta",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::EdgeGlow,
                key_hint: ">",
                label: "Edge Glow",
                kind: ParamKind::Toggle,
            },
        ],
        // ── Category 4: PRF — Performance ────────────────────────────────────
        4 => vec![
            ParamDesc {
                id: ParamId::FastMode,
                key_hint: "F",
                label: "Fast Mode",
                kind: ParamKind::Toggle,
            },
            ParamDesc {
                id: ParamId::Population,
                key_hint: "─",
                label: "Population",
                kind: ParamKind::CliReadonly,
            },
        ],
        // ── Category 5: SYS — System ─────────────────────────────────────────
        5 => vec![
            ParamDesc {
                id: ParamId::SaveFrame,
                key_hint: "G",
                label: "Save Frame",
                kind: ParamKind::Action,
            },
            ParamDesc {
                id: ParamId::Reset,
                key_hint: "0",
                label: "Reset",
                kind: ParamKind::Action,
            },
            ParamDesc {
                id: ParamId::Randomize,
                key_hint: "8",
                label: "Randomize",
                kind: ParamKind::Action,
            },
        ],
        _ => vec![],
    }
}

/// Maps an editable parameter to the real [`ControlAction`] that an arrow-key
/// adjustment should dispatch.
///
/// `sign` is `+1.0` for a forward/right adjustment and `-1.0` for a
/// backward/left one. Numeric parameters scale their per-key delta by `sign`,
/// with the exact step copied from the matching hotkey arm in
/// `crate::terminal::input::handle_key_event` so arrow steps equal hotkey
/// steps. Enum parameters cycle (using the reverse variant for `sign < 0.0`
/// where one exists); toggles are sign-independent.
///
/// Returns `None` for parameters that have no adjust action — display-only
/// rows ([`ParamId::MouseTimeout`], [`ParamId::Dither`]), CLI-readonly rows
/// ([`ParamId::Population`]), and action rows ([`ParamId::SaveFrame`],
/// [`ParamId::Reset`], [`ParamId::Randomize`], handled via activation instead).
///
/// [`ControlAction`]: crate::terminal::state::ControlAction
pub fn action_for(id: ParamId, sign: f32) -> Option<crate::terminal::state::ControlAction> {
    use crate::terminal::state::ControlAction as A;
    let reverse = sign < 0.0;
    Some(match id {
        // ── Numeric (delta copied from input.rs hotkey arms) ─────────────────
        ParamId::SensorAngle => A::AdjustSensorAngle(1.0 * sign),
        ParamId::SensorDistance => A::AdjustSensorDistance(1.0 * sign),
        ParamId::TurnAngle => A::AdjustTurnAngle(1.0 * sign),
        ParamId::StepSize => A::AdjustStepSize(0.1 * sign),
        ParamId::Decay => A::AdjustDecay(0.01 * sign),
        ParamId::Deposit => A::AdjustDeposit(1.0 * sign),
        ParamId::TimeScale => A::AdjustTimeScale(0.5 * sign),
        ParamId::DiffusionSigma => A::AdjustDiffusionSigma(0.1 * sign),
        ParamId::TerrainStrength => A::AdjustTerrainStrength(0.5 * sign),
        ParamId::Attractor => A::AdjustAttractorStrength(0.5 * sign),
        // Brightness: the user-facing control reads as gain (right = brighter)
        // but the engine stores a white-point divisor it divides by, so a
        // brighter image is a *lower* value. Hotkey 'n' (un-shifted) brightens
        // with delta −5.0, so a forward adjust (sign > 0) must also be −5.0.
        ParamId::Brightness => A::AdjustMaxBrightness(-5.0 * sign),
        // MotionBlur is numeric in the UI but only a cycle action exists.
        ParamId::MotionBlur => A::CycleMotionBlur,
        // ── Enum (use the reverse variant for sign < 0 where one exists) ─────
        ParamId::DiffusionKernel => A::CycleDiffusionKernel,
        ParamId::Wind => {
            if reverse {
                A::CycleWindDirectionReverse
            } else {
                A::CycleWindDirection
            }
        }
        ParamId::TerrainType => A::CycleTerrainType,
        ParamId::MouseMode => A::CycleMouseMode,
        ParamId::Theme => {
            if reverse {
                A::CycleThemeReverse
            } else {
                A::CycleTheme
            }
        }
        ParamId::Palette => {
            if reverse {
                A::CyclePaletteReverse
            } else {
                A::CyclePalette
            }
        }
        ParamId::Charset => {
            if reverse {
                A::CycleCharsetReverse
            } else {
                A::CycleCharset
            }
        }
        ParamId::ColorAa => A::CycleColorAa,
        ParamId::PaletteShift => A::CyclePaletteShiftSpeed,
        ParamId::IntensityMapping => {
            if reverse {
                A::CycleIntensityMappingReverse
            } else {
                A::CycleIntensityMapping
            }
        }
        ParamId::WindowFrame => {
            if reverse {
                A::CycleWindowFrameReverse
            } else {
                A::CycleWindowFrame
            }
        }
        // Chrome cycles forward-only (3 states; wraps). No reverse action exists.
        ParamId::Chrome => A::CycleChrome,
        // ── Toggle (sign-independent) ────────────────────────────────────────
        ParamId::Invert => A::ToggleInvertPalette,
        ParamId::Reverse => A::ToggleReversePalette,
        ParamId::StatusLine => A::ToggleStatusBar,
        ParamId::AutoNormalize => A::ToggleAutoNormalize,
        ParamId::TrailAge => A::ToggleTrailAge,
        ParamId::TrailDelta => A::ToggleTrailDelta,
        ParamId::EdgeGlow => A::ToggleGradientMagnitude,
        ParamId::FastMode => A::ToggleFastMode,
        // ── No adjust action ─────────────────────────────────────────────────
        // Display / CLI-readonly rows.
        ParamId::MouseTimeout | ParamId::Population | ParamId::Dither => return None,
        // Action rows are activated via `ControlsActivateFocused`, not adjusted.
        ParamId::SaveFrame | ParamId::Reset | ParamId::Randomize => return None,
    })
}

/// Maps an action-kind parameter to the [`ControlAction`] that activating it
/// (e.g. pressing Enter on the focused row) should dispatch.
///
/// Returns `None` for any parameter that is not an action row.
///
/// [`ControlAction`]: crate::terminal::state::ControlAction
pub fn activate_action_for(id: ParamId) -> Option<crate::terminal::state::ControlAction> {
    use crate::terminal::state::ControlAction as A;
    Some(match id {
        ParamId::SaveFrame => A::SaveFrameToPng,
        ParamId::Reset => A::ResetToDefaults,
        ParamId::Randomize => A::RandomizeParams,
        _ => return None,
    })
}

/// Reverse mapping: the [`ParamId`] a per-param hotkey [`ControlAction`]
/// belongs to, used for in-category focus retargeting when a hotkey fires.
///
/// Returns `None` for actions that do not correspond to a single registry
/// parameter (overlay/global actions, presets, etc.).
///
/// [`ControlAction`]: crate::terminal::state::ControlAction
pub fn param_id_for_action(action: &crate::terminal::state::ControlAction) -> Option<ParamId> {
    use crate::terminal::state::ControlAction as A;
    Some(match action {
        A::AdjustSensorAngle(_) => ParamId::SensorAngle,
        A::AdjustSensorDistance(_) => ParamId::SensorDistance,
        A::AdjustTurnAngle(_) => ParamId::TurnAngle,
        A::AdjustStepSize(_) => ParamId::StepSize,
        A::AdjustDecay(_) => ParamId::Decay,
        A::AdjustDeposit(_) => ParamId::Deposit,
        A::AdjustTimeScale(_) => ParamId::TimeScale,
        A::CycleDiffusionKernel => ParamId::DiffusionKernel,
        A::AdjustDiffusionSigma(_) => ParamId::DiffusionSigma,
        A::CycleWindDirection | A::CycleWindDirectionReverse => ParamId::Wind,
        A::CycleTerrainType => ParamId::TerrainType,
        A::AdjustTerrainStrength(_) => ParamId::TerrainStrength,
        A::AdjustAttractorStrength(_) => ParamId::Attractor,
        A::CycleMouseMode => ParamId::MouseMode,
        A::CycleTheme | A::CycleThemeReverse => ParamId::Theme,
        A::CyclePalette | A::CyclePaletteReverse => ParamId::Palette,
        A::CycleCharset | A::CycleCharsetReverse => ParamId::Charset,
        A::CycleColorAa => ParamId::ColorAa,
        A::CyclePaletteShiftSpeed => ParamId::PaletteShift,
        A::CycleIntensityMapping | A::CycleIntensityMappingReverse => ParamId::IntensityMapping,
        A::CycleWindowFrame | A::CycleWindowFrameReverse => ParamId::WindowFrame,
        A::CycleChrome => ParamId::Chrome,
        A::ToggleInvertPalette => ParamId::Invert,
        A::ToggleReversePalette => ParamId::Reverse,
        A::ToggleStatusBar => ParamId::StatusLine,
        A::ToggleAutoNormalize => ParamId::AutoNormalize,
        A::CycleMotionBlur => ParamId::MotionBlur,
        A::AdjustMaxBrightness(_) => ParamId::Brightness,
        A::ToggleTrailAge => ParamId::TrailAge,
        A::ToggleTrailDelta => ParamId::TrailDelta,
        A::ToggleGradientMagnitude => ParamId::EdgeGlow,
        A::ToggleFastMode => ParamId::FastMode,
        A::SaveFrameToPng => ParamId::SaveFrame,
        A::ResetToDefaults => ParamId::Reset,
        A::RandomizeParams => ParamId::Randomize,
        _ => return None,
    })
}

/// Returns `(category, visible_index)` for a given parameter id, or `None` if
/// the parameter is not currently visible (e.g. conditional row is hidden).
pub fn locate(id: ParamId, ctx: &RegistryCtx) -> Option<(usize, usize)> {
    for cat in 0..CATEGORY_NAMES.len() {
        let params = visible_params(cat, ctx);
        if let Some(idx) = params.iter().position(|p| p.id == id) {
            return Some((cat, idx));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_carries_kind_and_label() {
        let d = ParamDesc {
            id: ParamId::SensorAngle,
            key_hint: "A/a",
            label: "Sensor Angle",
            kind: ParamKind::Numeric,
        };
        assert_eq!(d.kind, ParamKind::Numeric);
        assert_eq!(d.label, "Sensor Angle");
    }

    #[test]
    fn mouse_timeout_is_display_kind() {
        // Mouse Timeout has no ControlAction; it must be Display, not CliReadonly.
        let d = ParamDesc {
            id: ParamId::MouseTimeout,
            key_hint: "─",
            label: "Mouse Timeout",
            kind: ParamKind::Display,
        };
        assert_eq!(d.kind, ParamKind::Display);
    }

    #[test]
    fn sim_category_has_seven_params() {
        let ctx = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        };
        let v = visible_params(0, &ctx);
        assert_eq!(v.len(), 7);
        assert_eq!(v[0].id, ParamId::SensorAngle);
        assert!(v.iter().all(|p| matches!(p.kind, ParamKind::Numeric)));
    }

    #[test]
    fn env_diff_sigma_only_when_gaussian() {
        let off = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        };
        let on = RegistryCtx {
            diffusion_gaussian: true,
            mouse_enabled: false,
        };
        assert!(!visible_params(1, &off)
            .iter()
            .any(|p| p.id == ParamId::DiffusionSigma));
        assert!(visible_params(1, &on)
            .iter()
            .any(|p| p.id == ParamId::DiffusionSigma));
    }

    #[test]
    fn env_mouse_timeout_only_when_mouse_enabled() {
        let off = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        };
        let on = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: true,
        };
        assert!(!visible_params(1, &off)
            .iter()
            .any(|p| p.id == ParamId::MouseTimeout));
        assert!(visible_params(1, &on)
            .iter()
            .any(|p| p.id == ParamId::MouseTimeout));
    }

    #[test]
    fn locate_returns_category_and_index() {
        let ctx = RegistryCtx {
            diffusion_gaussian: false,
            mouse_enabled: false,
        };
        // SIM order: SensorAngle(0), SensorDistance(1), TurnAngle(2), StepSize(3), ...
        assert_eq!(locate(ParamId::StepSize, &ctx), Some((0, 3)));
        // DiffusionSigma not visible when diffusion_gaussian=false
        assert_eq!(locate(ParamId::DiffusionSigma, &ctx), None);
    }

    #[test]
    fn action_for_numeric_uses_sign() {
        use crate::terminal::state::ControlAction;
        assert_eq!(
            action_for(ParamId::SensorAngle, -1.0),
            Some(ControlAction::AdjustSensorAngle(-1.0))
        );
        assert_eq!(
            action_for(ParamId::SensorAngle, 1.0),
            Some(ControlAction::AdjustSensorAngle(1.0))
        );
        // Display / CLI-readonly / action rows have no adjust action.
        assert_eq!(action_for(ParamId::MouseTimeout, 1.0), None);
        assert_eq!(action_for(ParamId::Population, 1.0), None);
        assert_eq!(action_for(ParamId::Reset, 1.0), None);
    }

    #[test]
    fn action_for_enum_uses_reverse_for_negative_sign() {
        use crate::terminal::state::ControlAction;
        assert_eq!(
            action_for(ParamId::Palette, 1.0),
            Some(ControlAction::CyclePalette)
        );
        assert_eq!(
            action_for(ParamId::Palette, -1.0),
            Some(ControlAction::CyclePaletteReverse)
        );
        // Enum with no reverse variant maps both signs to the same cycle.
        assert_eq!(
            action_for(ParamId::DiffusionKernel, 1.0),
            Some(ControlAction::CycleDiffusionKernel)
        );
        assert_eq!(
            action_for(ParamId::DiffusionKernel, -1.0),
            Some(ControlAction::CycleDiffusionKernel)
        );
    }

    #[test]
    fn action_for_brightness_forward_brightens() {
        use crate::terminal::state::ControlAction;
        // Forward (sign > 0) must lower the white-point divisor (brighter):
        // matches the un-shifted 'n' hotkey delta of −5.0.
        assert_eq!(
            action_for(ParamId::Brightness, 1.0),
            Some(ControlAction::AdjustMaxBrightness(-5.0))
        );
        assert_eq!(
            action_for(ParamId::Brightness, -1.0),
            Some(ControlAction::AdjustMaxBrightness(5.0))
        );
    }

    #[test]
    fn activate_action_for_maps_action_rows() {
        use crate::terminal::state::ControlAction;
        assert_eq!(
            activate_action_for(ParamId::SaveFrame),
            Some(ControlAction::SaveFrameToPng)
        );
        assert_eq!(
            activate_action_for(ParamId::Reset),
            Some(ControlAction::ResetToDefaults)
        );
        assert_eq!(
            activate_action_for(ParamId::Randomize),
            Some(ControlAction::RandomizeParams)
        );
        assert_eq!(activate_action_for(ParamId::SensorAngle), None);
    }

    #[test]
    fn orphan_enum_params_are_now_tunable() {
        use crate::terminal::state::ControlAction as A;
        // Intensity mapping, window frame, and chrome were notification-only;
        // they now map to actions (so a keybind surfaces a tuner) and round-trip
        // back to their ParamId.
        assert_eq!(
            action_for(ParamId::IntensityMapping, 1.0),
            Some(A::CycleIntensityMapping)
        );
        assert_eq!(
            action_for(ParamId::IntensityMapping, -1.0),
            Some(A::CycleIntensityMappingReverse)
        );
        assert_eq!(
            action_for(ParamId::WindowFrame, 1.0),
            Some(A::CycleWindowFrame)
        );
        assert_eq!(
            action_for(ParamId::WindowFrame, -1.0),
            Some(A::CycleWindowFrameReverse)
        );
        // Chrome cycles forward-only (no reverse action).
        assert_eq!(action_for(ParamId::Chrome, 1.0), Some(A::CycleChrome));
        assert_eq!(action_for(ParamId::Chrome, -1.0), Some(A::CycleChrome));

        // Reverse direction: actions resolve back to the ParamId so in-category
        // focus retargeting + TUNE surfacing fire.
        assert_eq!(
            param_id_for_action(&A::CycleIntensityMapping),
            Some(ParamId::IntensityMapping)
        );
        assert_eq!(
            param_id_for_action(&A::CycleWindowFrame),
            Some(ParamId::WindowFrame)
        );
        assert_eq!(param_id_for_action(&A::CycleChrome), Some(ParamId::Chrome));
    }

    #[test]
    fn locate_diffusion_sigma_when_gaussian_true() {
        let ctx_gaussian_true = RegistryCtx {
            diffusion_gaussian: true,
            mouse_enabled: false,
        };
        // ENV category (1) order: DiffusionKernel(0), DiffusionSigma(1), Wind(2), TerrainType(3), ...
        assert_eq!(
            locate(ParamId::DiffusionSigma, &ctx_gaussian_true),
            Some((1, 1))
        );
    }
}
