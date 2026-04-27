//! Choir-mode audio engine.
//!
//! Sonification adapted from Miranda, Adamatzky, Jones (2011),
//! "Sounds Synthesis with Slime Mould of Physarum Polycephalum",
//! *Journal of Bionic Engineering* 8: 107–113. Section 5 (simulation
//! variant): K virtual electrodes sample the multi-agent state every few
//! scheduler ticks; each value drives the frequency of one partial in an
//! additive synth, with stereo split between odd/even electrodes.
//!
//! tslime adaptation: trail-map intensity at 8 fixed grid points (the
//! trail map already integrates recent agent presence + diffusion +
//! decay, which matches the role of plasmodium voltage in the paper).
//! Values are quantized to a pentatonic scale for choir-pad aesthetics
//! and rendered as 8 sine voices with shared 5 Hz vibrato via cpal.
//!
//! All inter-thread state lives in lock-free atomics; the audio thread
//! never allocates.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Number of voices (paper's K virtual electrodes).
pub const VOICES: usize = 8;

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

/// Choir-mode audio engine. Holds the active output stream until dropped.
pub struct Choir {
    params: Arc<[VoiceParam; VOICES]>,
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

        let params: Arc<[VoiceParam; VOICES]> =
            Arc::new(std::array::from_fn(|_| VoiceParam::new()));

        // Audio-thread state retained across callback invocations.
        let mut phases = [0.0f32; VOICES];
        let mut cur_freq = [0.0f32; VOICES];
        let mut cur_gain = [0.0f32; VOICES];
        let mut vibrato_phase = 0.0f32;

        let two_pi = std::f32::consts::TAU;
        let inv_sr = 1.0 / sample_rate;
        // Per-sample lerp toward target — ~5 ms time constant.
        let glide = (1.0 / (0.005 * sample_rate)).min(1.0);

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
                        &mut phases,
                        &mut cur_freq,
                        &mut cur_gain,
                        &mut vibrato_phase,
                        glide,
                        inv_sr,
                        two_pi,
                        master_gain,
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

    /// Push voice parameters from the simulation thread (lock-free).
    pub fn set_voice(&self, i: usize, freq_hz: f32, gain: f32) {
        if i < VOICES {
            self.params[i].set_freq(freq_hz);
            self.params[i].set_gain(gain);
        }
    }

    /// Mute all voices (e.g. on pause).
    pub fn silence_all(&self) {
        for v in self.params.iter() {
            v.set_gain(0.0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    data: &mut [f32],
    channels: usize,
    params: &[VoiceParam; VOICES],
    phases: &mut [f32; VOICES],
    cur_freq: &mut [f32; VOICES],
    cur_gain: &mut [f32; VOICES],
    vibrato_phase: &mut f32,
    glide: f32,
    inv_sr: f32,
    two_pi: f32,
    master_gain: f32,
) {
    let vibrato_rate = 5.0_f32;
    let vibrato_depth = 0.004_f32;

    for frame in data.chunks_mut(channels.max(1)) {
        *vibrato_phase += two_pi * vibrato_rate * inv_sr;
        if *vibrato_phase > two_pi {
            *vibrato_phase -= two_pi;
        }
        let vib = 1.0 + vibrato_depth * vibrato_phase.sin();

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for i in 0..VOICES {
            let target_freq = params[i].freq();
            let target_gain = params[i].gain();
            cur_freq[i] += (target_freq - cur_freq[i]) * glide;
            cur_gain[i] += (target_gain - cur_gain[i]) * glide;

            let freq = cur_freq[i] * vib;
            phases[i] += two_pi * freq * inv_sr;
            if phases[i] > two_pi {
                phases[i] -= two_pi;
            }
            let s = phases[i].sin() * cur_gain[i];
            // Stereo split: even -> L, odd -> R (paper, §4).
            if i & 1 == 0 {
                left += s;
            } else {
                right += s;
            }
        }

        let l = left * master_gain;
        let r = right * master_gain;

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

/// Quantize a normalized [0, 1] intensity to a pentatonic scale frequency.
///
/// Voice index detunes by a chord interval so all voices on identical
/// values still produce a chord rather than unison.
pub fn quantize_to_scale(value: f32, voice_index: usize) -> f32 {
    // C minor pentatonic semitones.
    const SCALE: [i32; 5] = [0, 3, 5, 7, 10];
    let v = value.clamp(0.0, 1.0);
    let steps = (v * (SCALE.len() as f32 * 2.0 - 1.0)).round() as usize;
    let octave = (steps / SCALE.len()) as i32;
    let semitone = SCALE[steps % SCALE.len()] + octave * 12;
    let detune = SCALE[voice_index % SCALE.len()];
    let total = semitone + detune;
    // Base A3 = 220 Hz.
    220.0_f32 * 2.0_f32.powf(total as f32 / 12.0)
}

/// Sample the trail map at K positions, push freq/gain to voice atomics.
///
/// `max_brightness` normalizes raw trail values to [0, 1].
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
    for (i, (x, y)) in positions.iter().enumerate() {
        let idx = y * width + x;
        let v = (trail[idx] * inv_max).clamp(0.0, 1.0);
        let freq = quantize_to_scale(v, i);
        // Soft noise gate; full at v >= 0.5. 0.18 keeps headroom for 8 voices.
        let gain = ((v - 0.05) / 0.45).clamp(0.0, 1.0) * 0.18;
        choir.set_voice(i, freq, gain);
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
}
