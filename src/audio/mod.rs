//! Choir-mode audio engine.
//!
//! Sonification adapted from Miranda, E. R., Adamatzky, A., & Jones, J.
//! (2011). "Sounds Synthesis with Slime Mould of Physarum Polycephalum."
//! Journal of Bionic Engineering, 8(2), 107-113.
//! doi:10.1016/S1672-6529(11)60016-4
//!
//! We follow the paper's Section 5 (simulation variant): K virtual
//! electrodes sample the multi-agent state every few scheduler ticks; each
//! value drives the frequency of one partial in an additive synth.
//!
//! Per the paper §4, partials are pure sinewaves and the synth is
//! granular — short ~30–80 ms bursts windowed and stitched. We follow
//! that: each voice plays two overlapping Hann-windowed grains; at each
//! grain-start the voice's target frequency/gain is resampled (with
//! ±5-cent pitch jitter for chorus depth on melody voices). 8 melody
//! voices spread across a pentatonic scale and stereo field follow trail
//! intensity at 8 fixed grid points; 2 additional bass voices (root +
//! perfect fifth, longer grains, center pan, no jitter) follow the
//! aggregate trail signal so a low foundation grounds the sporadic
//! upper texture. All inter-thread state lives in lock-free atomics;
//! the audio thread never allocates.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Number of melody (choir) voices — paper's K virtual electrodes.
pub const VOICES: usize = 8;

/// Number of bass voices added below the choir (root + perfect fifth).
pub const BASS_VOICES: usize = 2;

/// Internal total voices (melody + bass).
const TOTAL_VOICES: usize = VOICES + BASS_VOICES;

/// Default grain length for melody voices (seconds). Long grains +
/// 50% overlap give a sustained, droning crossfade rather than a
/// stuttering granular texture.
const MELODY_GRAIN_SECONDS: f32 = 0.30;

/// Default grain length for bass voices (seconds). Longer = slower
/// crossfade, steadier low foundation.
const BASS_GRAIN_SECONDS: f32 = 0.40;

/// Pitch jitter for melody voices (cents). 0 for bass.
const MELODY_JITTER_CENTS: f32 = 5.0;

/// How many consecutive grains a melody voice holds the same pitch before
/// re-sampling its target. With 300 ms grains and 50% overlap, 4 holds
/// ≈ 750 ms of held pitch.
const MELODY_HOLD_GRAINS: u8 = 4;

/// Bass voices use long grains; re-sample every grain.
const BASS_HOLD_GRAINS: u8 = 1;

/// Master lowpass cutoff (Hz). Rolls off harsh upper partials/aliasing.
const MASTER_LPF_HZ: f32 = 1800.0;

/// Master lowpass Q (Butterworth ≈ 0.707).
const MASTER_LPF_Q: f32 = 0.707;

/// Stereo delay length (seconds). Long enough to feel like reverb wash
/// without pushing onset clarity into the next phrase.
const DELAY_SECONDS: f32 = 0.6;

/// Feedback coefficient inside the cross-coupled delay. Below 1.0 keeps
/// the line stable; closer to 1 = longer tail, more wash.
const DELAY_FEEDBACK: f32 = 0.65;

/// Wet/dry mix for the master delay (0 = dry, 1 = all wet).
const DELAY_WET: f32 = 0.40;

/// Cutoff of the 1-pole LPF inside the delay feedback loop. Darker
/// feedback prevents shimmery resonant build-up.
const DELAY_FEEDBACK_LPF_HZ: f32 = 1500.0;

/// Per-sample gain glide time constant (seconds). Smooths transitions
/// across the gate so voices don't pop on/off.
const GAIN_GLIDE_SECONDS: f32 = 0.5;

struct VoiceParam {
    freq_bits: AtomicU32,
    gain_bits: AtomicU32,
}

impl VoiceParam {
    const fn new() -> Self {
        Self {
            freq_bits: AtomicU32::new(0),
            gain_bits: AtomicU32::new(0),
        }
    }

    fn set_freq(&self, hz: f32) {
        self.freq_bits.store(hz.to_bits(), Ordering::Relaxed);
    }

    fn set_gain(&self, g: f32) {
        self.gain_bits.store(g.to_bits(), Ordering::Relaxed);
    }

    fn freq(&self) -> f32 {
        f32::from_bits(self.freq_bits.load(Ordering::Relaxed))
    }

    fn gain(&self) -> f32 {
        f32::from_bits(self.gain_bits.load(Ordering::Relaxed))
    }
}

#[derive(Clone, Copy)]
struct Grain {
    active: bool,
    phase: f32,
    freq: f32,
    gain: f32,
    env: f32,
}

impl Grain {
    const fn new() -> Self {
        Self {
            active: false,
            phase: 0.0,
            freq: 0.0,
            gain: 0.0,
            env: 0.0,
        }
    }
}

/// Per-voice runtime state.
struct VoiceState {
    grains: [Grain; 2],
    last_started: usize,
    pan_l: f32,
    pan_r: f32,
    /// Per-sample envelope advance (1 / (grain_seconds * sample_rate)).
    env_step: f32,
    /// Pitch jitter at grain start, cents. 0 disables.
    jitter_cents: f32,
    /// Frequency currently held across multiple grains; re-sampled when
    /// `holds_remaining` hits 0.
    held_freq: f32,
    /// Number of grain triggers remaining before re-sampling target.
    holds_remaining: u8,
    /// How many grains to hold a freq before picking a new target.
    max_hold: u8,
    /// Smoothed voice gain (lerped toward target each sample). Sampled
    /// at grain start so attacks/releases avoid clicks at the gate.
    cur_gain: f32,
}

/// Cross-coupled stereo feedback delay with dark feedback path. Used as
/// a low-cost reverb / drone-tail effect. Single-tap per channel, with
/// L's feedback fed into R's input and vice-versa.
struct StereoDelay {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write_idx: usize,
    feedback: f32,
    wet: f32,
    /// 1-pole LPF state in the feedback path.
    fb_state_l: f32,
    fb_state_r: f32,
    /// 1-pole LPF coefficient (0..1).
    fb_lpf_alpha: f32,
}

impl StereoDelay {
    fn new(
        delay_seconds: f32,
        sample_rate: f32,
        feedback: f32,
        wet: f32,
        fb_cutoff_hz: f32,
    ) -> Self {
        let len = ((delay_seconds * sample_rate).round() as usize).max(1);
        // 1-pole LPF: alpha = 1 - exp(-2π * fc / sr)
        let fb_lpf_alpha = 1.0 - (-std::f32::consts::TAU * fb_cutoff_hz / sample_rate).exp();
        Self {
            buf_l: vec![0.0; len],
            buf_r: vec![0.0; len],
            write_idx: 0,
            feedback,
            wet,
            fb_state_l: 0.0,
            fb_state_r: 0.0,
            fb_lpf_alpha,
        }
    }

    #[inline]
    fn step(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let tap_l = self.buf_l[self.write_idx];
        let tap_r = self.buf_r[self.write_idx];

        // 1-pole LPF on feedback (darker = warmer tail, no resonant peaks).
        self.fb_state_l += self.fb_lpf_alpha * (tap_l - self.fb_state_l);
        self.fb_state_r += self.fb_lpf_alpha * (tap_r - self.fb_state_r);

        // Cross-coupled write: each side's feedback feeds the other.
        self.buf_l[self.write_idx] = in_l + self.fb_state_r * self.feedback;
        self.buf_r[self.write_idx] = in_r + self.fb_state_l * self.feedback;

        self.write_idx += 1;
        if self.write_idx >= self.buf_l.len() {
            self.write_idx = 0;
        }

        // Wet/dry mix.
        let dry = 1.0 - self.wet;
        (in_l * dry + tap_l * self.wet, in_r * dry + tap_r * self.wet)
    }
}

/// Biquad lowpass filter coefficients (RBJ cookbook, normalized by a0).
#[derive(Clone, Copy)]
struct LowPass {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl LowPass {
    fn new(cutoff_hz: f32, q: f32, sample_rate: f32) -> Self {
        let omega = std::f32::consts::TAU * cutoff_hz / sample_rate;
        let cos_w = omega.cos();
        let sin_w = omega.sin();
        let alpha = sin_w / (2.0 * q);
        let a0 = 1.0 + alpha;
        let inv_a0 = 1.0 / a0;
        Self {
            b0: ((1.0 - cos_w) * 0.5) * inv_a0,
            b1: (1.0 - cos_w) * inv_a0,
            b2: ((1.0 - cos_w) * 0.5) * inv_a0,
            a1: (-2.0 * cos_w) * inv_a0,
            a2: (1.0 - alpha) * inv_a0,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    #[inline]
    fn step(&mut self, x: f32, c: &LowPass) -> f32 {
        let y = c.b0 * x + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Choir-mode audio engine. Holds the active output stream until dropped.
pub struct Choir {
    /// Indices [0, VOICES) = melody, [VOICES, TOTAL_VOICES) = bass.
    params: Arc<[VoiceParam; TOTAL_VOICES]>,
    _stream: cpal::Stream,
}

impl Choir {
    /// Open the default output device and begin playback.
    pub fn try_new(master_gain: f32) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "no default output device".to_string())?;
        let supported = device
            .default_output_config()
            .map_err(|e| format!("default_output_config: {e}"))?;
        let sample_format = supported.sample_format();
        let stream_config: cpal::StreamConfig = supported.into();
        let sample_rate = stream_config.sample_rate.0 as f32;
        let channels = stream_config.channels as usize;

        let params: Arc<[VoiceParam; TOTAL_VOICES]> =
            Arc::new(std::array::from_fn(|_| VoiceParam::new()));

        let mut voices: [VoiceState; TOTAL_VOICES] = std::array::from_fn(|i| {
            if i < VOICES {
                // Melody voice: pan ladder L→R, short grain, jittered, holds.
                let pan = if VOICES > 1 {
                    (i as f32 / (VOICES as f32 - 1.0)) * 2.0 - 1.0
                } else {
                    0.0
                };
                let pan_angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
                VoiceState {
                    grains: [Grain::new(); 2],
                    last_started: 0,
                    pan_l: pan_angle.cos(),
                    pan_r: pan_angle.sin(),
                    env_step: 1.0 / (MELODY_GRAIN_SECONDS * sample_rate),
                    jitter_cents: MELODY_JITTER_CENTS,
                    held_freq: 0.0,
                    holds_remaining: 0,
                    max_hold: MELODY_HOLD_GRAINS,
                    cur_gain: 0.0,
                }
            } else {
                // Bass voice: long grain, center pan (equal-power center
                // = cos(π/4) = sin(π/4) ≈ 0.7071), no jitter, no hold.
                let center = std::f32::consts::FRAC_1_SQRT_2;
                VoiceState {
                    grains: [Grain::new(); 2],
                    last_started: 0,
                    pan_l: center,
                    pan_r: center,
                    env_step: 1.0 / (BASS_GRAIN_SECONDS * sample_rate),
                    jitter_cents: 0.0,
                    held_freq: 0.0,
                    holds_remaining: 0,
                    max_hold: BASS_HOLD_GRAINS,
                    cur_gain: 0.0,
                }
            }
        });

        let lpf_coeffs = LowPass::new(MASTER_LPF_HZ, MASTER_LPF_Q, sample_rate);
        let mut lpf_l = BiquadState::default();
        let mut lpf_r = BiquadState::default();
        let mut delay = StereoDelay::new(
            DELAY_SECONDS,
            sample_rate,
            DELAY_FEEDBACK,
            DELAY_WET,
            DELAY_FEEDBACK_LPF_HZ,
        );
        let gain_glide = (1.0 / (GAIN_GLIDE_SECONDS * sample_rate)).min(1.0);

        let inv_sr = 1.0 / sample_rate;
        let overlap_trigger = 0.5_f32;
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0x5151_70DE_FEC8_C0DE);

        let params_for_cb = Arc::clone(&params);
        let err_fn = |err| eprintln!("choir audio stream error: {err}");

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    render(
                        data,
                        channels,
                        &params_for_cb,
                        &mut voices,
                        overlap_trigger,
                        inv_sr,
                        master_gain,
                        gain_glide,
                        &mut rng,
                        &lpf_coeffs,
                        &mut lpf_l,
                        &mut lpf_r,
                        &mut delay,
                    );
                },
                err_fn,
                None,
            ),
            other => return Err(format!("unsupported sample format: {other:?}")),
        }
        .map_err(|e| format!("build_output_stream: {e}"))?;

        stream.play().map_err(|e| format!("stream play: {e}"))?;

        Ok(Self {
            params,
            _stream: stream,
        })
    }

    /// Push melody voice parameters from the simulation thread (lock-free).
    pub fn set_voice(&self, i: usize, freq_hz: f32, gain: f32) {
        if i < VOICES {
            self.params[i].set_freq(freq_hz);
            self.params[i].set_gain(gain);
        }
    }

    /// Push bass voice parameters (i ∈ [0, BASS_VOICES)).
    pub fn set_bass(&self, i: usize, freq_hz: f32, gain: f32) {
        if i < BASS_VOICES {
            self.params[VOICES + i].set_freq(freq_hz);
            self.params[VOICES + i].set_gain(gain);
        }
    }

    /// Mute all voices (melody + bass).
    pub fn silence_all(&self) {
        for v in self.params.iter() {
            v.set_gain(0.0);
        }
    }
}

#[inline]
fn hann(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        0.0
    } else {
        0.5 - 0.5 * (std::f32::consts::TAU * t).cos()
    }
}

/// Reuse the voice's currently held frequency, or sample a fresh one
/// from `target_freq` and reset the hold counter. Always decrements the
/// hold counter so consecutive grains drain it.
fn pick_held_freq(v: &mut VoiceState, target_freq: f32) -> f32 {
    if v.holds_remaining == 0 {
        v.held_freq = target_freq;
        v.holds_remaining = v.max_hold;
    }
    let f = v.held_freq;
    v.holds_remaining = v.holds_remaining.saturating_sub(1);
    f
}

fn trigger_grain(
    g: &mut Grain,
    target_freq: f32,
    target_gain: f32,
    jitter_cents: f32,
    rng: &mut Xoshiro256PlusPlus,
) {
    use rand::Rng;
    let factor = if jitter_cents > 0.0 {
        let cents: f32 = rng.gen_range(-jitter_cents..jitter_cents);
        2.0_f32.powf(cents / 1200.0)
    } else {
        1.0
    };
    g.active = true;
    g.phase = 0.0;
    g.freq = target_freq * factor;
    g.gain = target_gain;
    g.env = 0.0;
}

#[allow(clippy::too_many_arguments)]
fn render(
    data: &mut [f32],
    channels: usize,
    params: &[VoiceParam; TOTAL_VOICES],
    voices: &mut [VoiceState; TOTAL_VOICES],
    overlap_trigger: f32,
    inv_sr: f32,
    master_gain: f32,
    gain_glide: f32,
    rng: &mut Xoshiro256PlusPlus,
    lpf_coeffs: &LowPass,
    lpf_l: &mut BiquadState,
    lpf_r: &mut BiquadState,
    delay: &mut StereoDelay,
) {
    let two_pi = std::f32::consts::TAU;

    for frame in data.chunks_mut(channels.max(1)) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for idx in 0..TOTAL_VOICES {
            let target_freq = params[idx].freq();
            let target_gain = params[idx].gain();
            let v = &mut voices[idx];

            // Smooth voice gain toward target each sample (slow ramp;
            // grain trigger uses cur_gain rather than target).
            v.cur_gain += (target_gain - v.cur_gain) * gain_glide;

            let live = v.last_started;
            let other = 1 - live;
            if v.grains[live].active
                && v.grains[live].env >= overlap_trigger
                && !v.grains[other].active
            {
                let freq = pick_held_freq(v, target_freq);
                trigger_grain(&mut v.grains[other], freq, v.cur_gain, v.jitter_cents, rng);
                v.last_started = other;
            }

            if !v.grains[0].active && !v.grains[1].active && v.cur_gain > 1e-4 {
                // Cold start: clear hold to ensure fresh pitch pick.
                v.holds_remaining = 0;
                let freq = pick_held_freq(v, target_freq);
                trigger_grain(&mut v.grains[0], freq, v.cur_gain, v.jitter_cents, rng);
                v.last_started = 0;
            }

            let mut voice_sample = 0.0f32;
            for g in v.grains.iter_mut() {
                if !g.active {
                    continue;
                }
                let s = g.phase.sin() * hann(g.env) * g.gain;
                voice_sample += s;
                g.phase += two_pi * g.freq * inv_sr;
                if g.phase > two_pi {
                    g.phase -= two_pi;
                }
                g.env += v.env_step;
                if g.env >= 1.0 {
                    g.active = false;
                }
            }

            left += voice_sample * v.pan_l;
            right += voice_sample * v.pan_r;
        }

        // Master chain: lowpass → cross-coupled feedback delay → soft sat.
        let lpf_l_out = lpf_l.step(left * master_gain, lpf_coeffs);
        let lpf_r_out = lpf_r.step(right * master_gain, lpf_coeffs);
        let (wet_l, wet_r) = delay.step(lpf_l_out, lpf_r_out);
        // Softsign (x / (1 + |x|)) soft-clips peaks.
        let l = wet_l / (1.0 + wet_l.abs());
        let r = wet_r / (1.0 + wet_r.abs());

        match channels {
            1 => frame[0] = (l + r) * 0.5,
            2 => {
                frame[0] = l;
                frame[1] = r;
            }
            _ => {
                if !frame.is_empty() {
                    frame[0] = l;
                }
                if frame.len() > 1 {
                    frame[1] = r;
                }
                for s in frame.iter_mut().skip(2) {
                    *s = 0.0;
                }
            }
        }
    }
}

/// 8 fixed sample positions on a 3×3 grid (centre omitted to avoid
/// clustering when sims start centered).
pub fn electrode_positions(width: usize, height: usize) -> [(usize, usize); VOICES] {
    let xs = [width / 4, width / 2, (3 * width) / 4];
    let ys = [height / 4, height / 2, (3 * height) / 4];
    [
        (xs[0], ys[0]),
        (xs[1], ys[0]),
        (xs[2], ys[0]),
        (xs[0], ys[1]),
        (xs[2], ys[1]),
        (xs[0], ys[2]),
        (xs[1], ys[2]),
        (xs[2], ys[2]),
    ]
}

/// Quantize a normalized [0, 1] intensity to a melody pentatonic frequency.
///
/// Voice index detunes by a chord interval so all voices on identical
/// values still produce a chord rather than unison.
pub fn quantize_to_scale(value: f32, voice_index: usize) -> f32 {
    const SCALE: [i32; 5] = [0, 3, 5, 7, 10];
    let v = value.clamp(0.0, 1.0);
    let steps = (v * (SCALE.len() as f32 * 2.0 - 1.0)).round() as usize;
    let octave = (steps / SCALE.len()) as i32;
    let semitone = SCALE[steps % SCALE.len()] + octave * 12;
    let detune = SCALE[voice_index % SCALE.len()];
    let total = semitone + detune;
    220.0_f32 * 2.0_f32.powf(total as f32 / 12.0)
}

/// Quantize aggregate signal to a low pentatonic register for bass.
///
/// `voice_index == 0` is the root; `1` is a perfect fifth above. Roots
/// sweep 55–98 Hz (A1–G2) and the fifth voice reaches ~147 Hz — low
/// enough to ground the texture without rumble.
pub fn quantize_to_bass(value: f32, voice_index: usize) -> f32 {
    const SCALE: [i32; 5] = [0, 3, 5, 7, 10];
    let v = value.clamp(0.0, 1.0);
    // Single-octave sweep so the bass moves slowly and stays low.
    let step = (v * (SCALE.len() as f32 - 1.0)).round() as usize;
    let semitone = SCALE[step.min(SCALE.len() - 1)];
    let fifth = if voice_index % 2 == 1 { 7 } else { 0 };
    // A1 = 55 Hz.
    55.0_f32 * 2.0_f32.powf((semitone + fifth) as f32 / 12.0)
}

/// Sample the trail map at K positions, push freq/gain to voice atomics.
/// Also computes the mean signal across electrodes and drives bass voices.
pub fn update_voices_from_trail(
    choir: &Choir,
    trail: &[f32],
    width: usize,
    height: usize,
    max_brightness: f32,
) {
    if trail.len() != width * height || max_brightness <= 0.0 {
        return;
    }
    let inv_max = 1.0 / max_brightness;
    let positions = electrode_positions(width, height);

    let mut sum = 0.0f32;
    for (i, (x, y)) in positions.iter().enumerate() {
        let idx = y * width + x;
        let v = (trail[idx] * inv_max).clamp(0.0, 1.0);
        sum += v;
        let freq = quantize_to_scale(v, i);
        // Low gate: most voices stay alive most of the time so the
        // choir feels like a sustained drone rather than discrete
        // events. Smoothing happens via per-voice gain glide.
        let gain = ((v - 0.05) / 0.40).clamp(0.0, 1.0) * 0.18;
        choir.set_voice(i, freq, gain);
    }

    // Aggregate signal drives bass: smoother, less reactive than
    // per-electrode taps, so bass evolves slowly under the texture.
    let mean = sum / VOICES as f32;
    for b in 0..BASS_VOICES {
        let freq = quantize_to_bass(mean, b);
        let gain = ((mean - 0.05) / 0.40).clamp(0.0, 1.0) * 0.28;
        choir.set_bass(b, freq, gain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn electrodes_in_bounds() {
        let pts = electrode_positions(400, 400);
        for (x, y) in pts {
            assert!(x < 400 && y < 400);
        }
    }

    #[test]
    fn scale_in_audible_range() {
        for i in 0..VOICES {
            for v in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let f = quantize_to_scale(v, i);
                assert!((40.0..=4000.0).contains(&f), "freq {f} out of range");
            }
        }
    }

    #[test]
    fn scale_low_value_picks_low_pitch() {
        let lo = quantize_to_scale(0.0, 0);
        let hi = quantize_to_scale(1.0, 0);
        assert!(hi > lo);
    }

    #[test]
    fn bass_in_low_register() {
        for i in 0..BASS_VOICES {
            for v in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let f = quantize_to_bass(v, i);
                assert!((40.0..=200.0).contains(&f), "bass freq {f} out of range");
            }
        }
    }

    #[test]
    fn bass_voice_one_higher_than_voice_zero_at_same_value() {
        // Voice 1 = root + perfect fifth above voice 0.
        let v0 = quantize_to_bass(0.5, 0);
        let v1 = quantize_to_bass(0.5, 1);
        assert!(v1 > v0);
        let ratio = v1 / v0;
        let expected = 2.0_f32.powf(7.0 / 12.0); // perfect fifth
        assert!(
            (ratio - expected).abs() < 1e-3,
            "ratio {ratio}, expected {expected}"
        );
    }

    #[test]
    fn hann_endpoints_zero_center_one() {
        assert_eq!(hann(0.0), 0.0);
        assert_eq!(hann(1.0), 0.0);
        assert!((hann(0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hann_symmetric() {
        for t in [0.1, 0.2, 0.3, 0.4] {
            assert!((hann(t) - hann(1.0 - t)).abs() < 1e-6);
        }
    }

    #[test]
    fn trigger_grain_with_jitter_close_to_target() {
        let mut g = Grain::new();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
        trigger_grain(&mut g, 440.0, 0.5, MELODY_JITTER_CENTS, &mut rng);
        assert!(g.active);
        assert_eq!(g.gain, 0.5);
        assert_eq!(g.env, 0.0);
        let lo = 440.0 * 2.0_f32.powf(-MELODY_JITTER_CENTS / 1200.0);
        let hi = 440.0 * 2.0_f32.powf(MELODY_JITTER_CENTS / 1200.0);
        assert!(
            g.freq >= lo && g.freq <= hi,
            "freq {} outside jitter range",
            g.freq
        );
    }

    #[test]
    fn trigger_grain_no_jitter_exact() {
        let mut g = Grain::new();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
        trigger_grain(&mut g, 110.0, 0.3, 0.0, &mut rng);
        assert_eq!(g.freq, 110.0);
    }

    #[test]
    fn pick_held_freq_holds_then_resamples() {
        let mut v = VoiceState {
            grains: [Grain::new(); 2],
            last_started: 0,
            pan_l: 1.0,
            pan_r: 0.0,
            env_step: 1.0,
            jitter_cents: 0.0,
            held_freq: 0.0,
            holds_remaining: 0,
            max_hold: 3,
            cur_gain: 0.0,
        };
        let f1 = pick_held_freq(&mut v, 220.0); // first call → 220, hold 3 (decremented to 2)
        let f2 = pick_held_freq(&mut v, 440.0); // hold = 1, reuse 220
        let f3 = pick_held_freq(&mut v, 880.0); // hold = 0, reuse 220
        let f4 = pick_held_freq(&mut v, 110.0); // expired → 110
        assert_eq!(f1, 220.0);
        assert_eq!(f2, 220.0);
        assert_eq!(f3, 220.0);
        assert_eq!(f4, 110.0);
    }

    #[test]
    fn delay_produces_decaying_tail() {
        // Single-sample impulse should bleed through the delay tap and
        // the cross-coupled feedback should produce a slowly decaying
        // response that's still audible after several round-trips.
        let mut d = StereoDelay::new(0.05, 48_000.0, 0.6, 0.5, 2_000.0);
        let mut tail = 0.0_f32;
        // Impulse on left input.
        let (_, _) = d.step(1.0, 0.0);
        for _ in 0..(0.05 * 48_000.0) as usize * 4 {
            let (l, r) = d.step(0.0, 0.0);
            tail = tail.max(l.abs()).max(r.abs());
        }
        assert!(
            tail > 0.05,
            "no audible delay tail after 4 round-trips: {tail}"
        );
    }

    #[test]
    fn lowpass_attenuates_above_cutoff() {
        // Drive a single biquad LPF with a high-freq sine; output level
        // should be much lower than the input level.
        let coeffs = LowPass::new(1000.0, 0.707, 48000.0);
        let mut state = BiquadState::default();
        // Warm up with high-freq tone (10 kHz, well above cutoff).
        let mut max_high = 0.0f32;
        let two_pi = std::f32::consts::TAU;
        for n in 0..2048 {
            let x = (two_pi * 10_000.0 * (n as f32) / 48_000.0).sin();
            let y = state.step(x, &coeffs);
            if n > 1024 {
                max_high = max_high.max(y.abs());
            }
        }
        // Reset; pass low-freq tone (200 Hz, well below cutoff).
        state = BiquadState::default();
        let mut max_low = 0.0f32;
        for n in 0..2048 {
            let x = (two_pi * 200.0 * (n as f32) / 48_000.0).sin();
            let y = state.step(x, &coeffs);
            if n > 1024 {
                max_low = max_low.max(y.abs());
            }
        }
        assert!(
            max_low > 0.5 && max_high < 0.2,
            "LPF should pass low (got {max_low:.3}), block high (got {max_high:.3})"
        );
    }
}
