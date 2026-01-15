//! CPU-intensive offline rendering demonstration.
//!
//! Renders a 60-second ambient piece using the OfflineRenderer at maximum CPU
//! speed (faster than realtime). Features thick unison synthesis with multiple
//! detuned oscillators per voice, each with individual filter modulation.
//!
//! The dense graph (~35+ oscillators, ~35+ filters, numerous LFOs) showcases
//! the OfflineRenderer's ability to process complex DSP chains efficiently.
//!
//! Signal chain per voice:
//!   UnisonOsc[0] → LPF → ─┐
//!   UnisonOsc[1] → LPF → ─┤
//!   UnisonOsc[2] → LPF → ─┼→ VoiceMixer → Gain → Panner → MainMixer → Output
//!   UnisonOsc[3] → LPF → ─┤
//!   UnisonOsc[4] → LPF → ─┘
//!          ↑                                 ↑
//!      FilterLFO                          PanLFO

use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, LowPassFilterBlock, MixerBlock, OscillatorBlock, PannerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::GraphBuilder,
    waveform::Waveform,
};
use bbx_file::{OfflineRenderer, RenderDuration, writers::wav::WavFileWriter};
use rand::prelude::*;

const ROOTS: [f64; 6] = [
    55.0, // A1
    65.4, // C2
    73.4, // D2
    82.4, // E2
    87.3, // F2
    98.0, // G2
];

const PERFECT_FOURTH: f64 = 1.334839854; // 2^(5/12)

const NUM_VOICES: usize = 5;
const UNISON_COUNT: usize = 7;
const DURATION_SECS: usize = 60;

const DETUNE_CENTS_MAX: f64 = 12.0;

const FILTER_LFO_RATE_MIN: f64 = 0.01;
const FILTER_LFO_RATE_MAX: f64 = 0.05;
const PAN_LFO_RATE_MIN: f64 = 0.005;
const PAN_LFO_RATE_MAX: f64 = 0.02;

const FILTER_LFO_DEPTH_MIN: f64 = 200.0;
const FILTER_LFO_DEPTH_MAX: f64 = 600.0;
const PAN_LFO_DEPTH_MIN: f64 = 30.0;
const PAN_LFO_DEPTH_MAX: f64 = 80.0;

const FILTER_CUTOFF_MIN: f64 = 300.0;
const FILTER_CUTOFF_MAX: f64 = 1000.0;
const FILTER_Q_MIN: f64 = 0.707;
const FILTER_Q_MAX: f64 = 2.0;

fn quartal_frequency(root_hz: f64, stack_position: usize) -> f64 {
    root_hz * PERFECT_FOURTH.powi(stack_position as i32)
}

fn detune_frequency(base_hz: f64, cents: f64) -> f64 {
    base_hz * 2.0_f64.powf(cents / 1200.0)
}

fn unison_detune_cents(unison_index: usize, unison_count: usize, max_cents: f64) -> f64 {
    if unison_count == 1 {
        return 0.0;
    }
    let normalized = unison_index as f64 / (unison_count - 1) as f64;
    (normalized * 2.0 - 1.0) * max_cents
}

fn random_waveform(rng: &mut impl Rng) -> Waveform {
    let roll: f64 = rng.r#gen();
    if roll < 0.50 {
        Waveform::Sine
    } else if roll < 0.85 {
        Waveform::Triangle
    } else {
        Waveform::Sawtooth
    }
}

fn voice_gain_db(position: usize, num_voices: usize, unison_count: usize) -> f64 {
    let normalized = position as f64 / (num_voices - 1).max(1) as f64;
    let base_gain = -6.0 - (normalized * 12.0);
    let unison_compensation = -3.0 * (unison_count as f64).log2();
    base_gain + unison_compensation
}

fn distribute_pan_position(rng: &mut impl Rng, position: usize, num_voices: usize) -> f64 {
    let base = -60.0 + (120.0 * position as f64 / (num_voices - 1).max(1) as f64);
    (base + rng.gen_range(-20.0..20.0)).clamp(-100.0, 100.0)
}

fn main() {
    let mut rng = rand::thread_rng();
    let seed: u64 = rng.r#gen();
    println!("Unison Quartal Harmony - Offline Renderer Demo");
    println!("Seed: {seed}");

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let root_hz = ROOTS[rng.gen_range(0..ROOTS.len())];
    let total_oscillators = NUM_VOICES * UNISON_COUNT;

    println!("Config: {NUM_VOICES} voices x {UNISON_COUNT} unison = {total_oscillators} oscillators");
    println!("Root: {root_hz:.1}Hz, Duration: {DURATION_SECS}s");
    println!();

    let mut builder = GraphBuilder::<f32>::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let main_mixer = builder.add(MixerBlock::stereo(NUM_VOICES));

    for voice_idx in 0..NUM_VOICES {
        let base_freq = quartal_frequency(root_hz, voice_idx);
        let waveform = random_waveform(&mut rng);
        let gain_db = voice_gain_db(voice_idx, NUM_VOICES, UNISON_COUNT);
        let pan_position = distribute_pan_position(&mut rng, voice_idx, NUM_VOICES);

        println!("Voice {voice_idx}: {base_freq:.1}Hz, {waveform:?}, {gain_db:.1}dB, pan={pan_position:.0}");

        let voice_mixer = builder.add(MixerBlock::new(UNISON_COUNT, 1));

        let filter_lfo_rate = rng.gen_range(FILTER_LFO_RATE_MIN..FILTER_LFO_RATE_MAX);
        let filter_lfo_depth = rng.gen_range(FILTER_LFO_DEPTH_MIN..FILTER_LFO_DEPTH_MAX);
        let filter_lfo = builder.add(LfoBlock::new(
            filter_lfo_rate,
            filter_lfo_depth,
            Waveform::Sine,
            Some(rng.next_u64()),
        ));

        for unison_idx in 0..UNISON_COUNT {
            let detune_cents = unison_detune_cents(unison_idx, UNISON_COUNT, DETUNE_CENTS_MAX);
            let freq = detune_frequency(base_freq, detune_cents);

            let osc = builder.add(OscillatorBlock::new(freq, waveform, Some(rng.next_u64())));

            let filter_cutoff = rng.gen_range(FILTER_CUTOFF_MIN..FILTER_CUTOFF_MAX);
            let filter_q = rng.gen_range(FILTER_Q_MIN..FILTER_Q_MAX);
            let lpf = builder.add(LowPassFilterBlock::new(filter_cutoff, filter_q));

            builder.connect(osc, 0, lpf, 0);
            builder.modulate(filter_lfo, lpf, "cutoff");
            builder.connect(lpf, 0, voice_mixer, unison_idx);
        }

        let gain = builder.add(GainBlock::new(gain_db, None));
        builder.connect(voice_mixer, 0, gain, 0);

        let panner = builder.add(PannerBlock::new(pan_position));

        let pan_lfo_rate = rng.gen_range(PAN_LFO_RATE_MIN..PAN_LFO_RATE_MAX);
        let pan_lfo_depth = rng.gen_range(PAN_LFO_DEPTH_MIN..PAN_LFO_DEPTH_MAX);
        let pan_lfo = builder.add(LfoBlock::new(
            pan_lfo_rate,
            pan_lfo_depth,
            Waveform::Sine,
            Some(rng.next_u64()),
        ));

        builder.connect(gain, 0, panner, 0);
        builder.modulate(pan_lfo, panner, "position");

        let mixer_input_l = voice_idx * 2;
        let mixer_input_r = voice_idx * 2 + 1;
        builder.connect(panner, 0, main_mixer, mixer_input_l);
        builder.connect(panner, 1, main_mixer, mixer_input_r);
    }

    let graph = builder.build();

    let mut file_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/bbx_sandbox/examples/16_offline_rendering.wav");

    let writer = WavFileWriter::new(file_path.as_str(), DEFAULT_SAMPLE_RATE, 2).unwrap();
    let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

    println!();
    println!("Rendering to 16_offline_rendering.wav...");

    let stats = renderer.render(RenderDuration::Duration(DURATION_SECS)).unwrap();

    println!();
    println!(
        "Done! Rendered {:.1}s of audio in {:.2}s ({:.1}x realtime)",
        stats.duration_seconds, stats.render_time_seconds, stats.speedup
    );
}
