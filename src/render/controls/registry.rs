//! Parameter kinds and descriptors for the Controls Instrument UI.
//! Defines the type system for all parameters that can be displayed and manipulated
//! in the Tuner and Console surfaces.

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
    /// Sensor angle parameter.
    SensorAngle,
    /// Sensor distance parameter.
    SensorDistance,
    /// Turn angle parameter.
    TurnAngle,
    /// Step size parameter.
    StepSize,
    /// Decay parameter.
    Decay,
    /// Deposit parameter.
    Deposit,
    /// Time scale parameter.
    TimeScale,
    /// Diffusion kernel parameter.
    DiffusionKernel,
    /// Diffusion sigma parameter.
    DiffusionSigma,
    /// Wind parameter.
    Wind,
    /// Terrain type parameter.
    TerrainType,
    /// Terrain strength parameter.
    TerrainStrength,
    /// Attractor parameter.
    Attractor,

    /// Mouse mode parameter.
    MouseMode,
    /// Mouse timeout parameter.
    MouseTimeout,
    /// Theme parameter.
    Theme,
    /// Palette parameter.
    Palette,
    /// Charset parameter.
    Charset,
    /// Color anti-aliasing parameter.
    ColorAa,
    /// Palette shift parameter.
    PaletteShift,
    /// Invert parameter.
    Invert,
    /// Reverse parameter.
    Reverse,
    /// Dither parameter.
    Dither,
    /// Auto-normalize parameter.
    AutoNormalize,
    /// Motion blur parameter.
    MotionBlur,
    /// Brightness parameter.
    Brightness,
    /// Trail age parameter.
    TrailAge,
    /// Trail delta parameter.
    TrailDelta,
    /// Edge glow parameter.
    EdgeGlow,
    /// Fast mode parameter.
    FastMode,

    /// Population parameter.
    Population,
    /// Save frame parameter.
    SaveFrame,
    /// Reset parameter.
    Reset,
    /// Randomize parameter.
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
        ],
        // ── Category 3: PST — Post-Processing ────────────────────────────────
        3 => vec![
            ParamDesc {
                id: ParamId::Dither,
                key_hint: "d/D",
                label: "Dither (dev)",
                kind: ParamKind::CliReadonly,
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
