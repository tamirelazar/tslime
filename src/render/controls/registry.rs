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
    /// Keyboard hint (e.g., "A/a", "—").
    pub key_hint: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// The kind of parameter (determines how it's rendered and interacted with).
    pub kind: ParamKind,
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
            key_hint: "—",
            label: "Mouse Timeout",
            kind: ParamKind::Display,
        };
        assert_eq!(d.kind, ParamKind::Display);
    }
}
