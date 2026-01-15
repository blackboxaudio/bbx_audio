//! Generative quartal harmony ambient piece.
//!
//! Creates a unique Polygonia-style drone each run using randomized quartal
//! voicings (stacks of perfect 4ths). Each voice passes through a low-pass
//! filter with slow LFO modulation, then through a stereo panner with its
//! own spatial movement. Output is written to WAV.
//!
//! Signal chain per voice:
//!   Oscillator → LowPassFilter → Gain → Panner → Mixer → FileOutput
//!                      ↑                   ↑
//!                  FilterLFO            PanLFO

use bbx_dsp::{
    blocks::{FileOutputBlock, GainBlock, LfoBlock, LowPassFilterBlock, MixerBlock, OscillatorBlock, PannerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_file::writers::wav::WavFileWriter;
use bbx_sandbox::player::Player;
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

const MIN_VOICES: usize = 4;
const MAX_VOICES: usize = 6;
const MIN_DURATION_SECS: usize = 30;
const MAX_DURATION_SECS: usize = 90;

const FILTER_LFO_RATE_MIN: f64 = 0.01;
const FILTER_LFO_RATE_MAX: f64 = 0.05;
const PAN_LFO_RATE_MIN: f64 = 0.005;
const PAN_LFO_RATE_MAX: f64 = 0.02;

const FILTER_LFO_DEPTH_MIN: f64 = 200.0;
const FILTER_LFO_DEPTH_MAX: f64 = 600.0;
const PAN_LFO_DEPTH_MIN: f64 = 30.0;
const PAN_LFO_DEPTH_MAX: f64 = 80.0;

const FILTER_CUTOFF_MIN: f64 = 400.0;
const FILTER_CUTOFF_MAX: f64 = 1200.0;
const FILTER_Q_MIN: f64 = 0.707;
const FILTER_Q_MAX: f64 = 1.5;

fn quartal_frequency(root_hz: f64, stack_position: usize) -> f64 {
    root_hz * PERFECT_FOURTH.powi(stack_position as i32)
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

fn voice_gain_db(position: usize, num_voices: usize) -> f64 {
    let normalized = position as f64 / (num_voices - 1).max(1) as f64;
    -6.0 - (normalized * 12.0)
}

fn distribute_pan_position(rng: &mut impl Rng, position: usize, num_voices: usize) -> f64 {
    let base = -60.0 + (120.0 * position as f64 / (num_voices - 1).max(1) as f64);
    (base + rng.gen_range(-20.0..20.0)).clamp(-100.0, 100.0)
}

fn create_graph(rng: &mut impl Rng) -> (Graph<f32>, usize) {
    let root_hz = ROOTS[rng.gen_range(0..ROOTS.len())];
    let num_voices = rng.gen_range(MIN_VOICES..=MAX_VOICES);
    let duration = rng.gen_range(MIN_DURATION_SECS..=MAX_DURATION_SECS);

    println!(
        "Generating: {num_voices} voices, root={root_hz:.1}Hz, duration={duration}s"
    );

    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mixer = builder.add(MixerBlock::stereo(num_voices));

    let mut file_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/bbx_sandbox/examples/05_output_wav_file.wav");

    let writer = WavFileWriter::new(file_path.as_str(), DEFAULT_SAMPLE_RATE, 2).unwrap();
    let file_output = builder.add(FileOutputBlock::new(Box::new(writer)));

    builder.connect(mixer, 0, file_output, 0);
    builder.connect(mixer, 1, file_output, 1);

    for voice_idx in 0..num_voices {
        let freq = quartal_frequency(root_hz, voice_idx);
        let waveform = random_waveform(rng);
        let gain_db = voice_gain_db(voice_idx, num_voices);
        let pan_position = distribute_pan_position(rng, voice_idx, num_voices);

        println!(
            "  Voice {voice_idx}: {freq:.1}Hz, {waveform:?}, {gain_db:.1}dB, pan={pan_position:.0}"
        );

        let osc = builder.add(OscillatorBlock::new(freq, waveform, Some(rng.next_u64())));

        let filter_cutoff = rng.gen_range(FILTER_CUTOFF_MIN..FILTER_CUTOFF_MAX);
        let filter_q = rng.gen_range(FILTER_Q_MIN..FILTER_Q_MAX);
        let lpf = builder.add(LowPassFilterBlock::new(filter_cutoff, filter_q));

        let filter_lfo_rate = rng.gen_range(FILTER_LFO_RATE_MIN..FILTER_LFO_RATE_MAX);
        let filter_lfo_depth = rng.gen_range(FILTER_LFO_DEPTH_MIN..FILTER_LFO_DEPTH_MAX);
        let filter_lfo = builder.add(LfoBlock::new(
            filter_lfo_rate,
            filter_lfo_depth,
            Waveform::Sine,
            Some(rng.next_u64()),
        ));

        builder.connect(osc, 0, lpf, 0);
        builder.modulate(filter_lfo, lpf, "cutoff");

        let gain = builder.add(GainBlock::new(gain_db, None));
        builder.connect(lpf, 0, gain, 0);

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
        builder.connect(panner, 0, mixer, mixer_input_l);
        builder.connect(panner, 1, mixer, mixer_input_r);
    }

    (builder.build(), duration)
}

fn main() {
    let mut rng = rand::thread_rng();
    let seed: u64 = rng.r#gen();
    println!("Quartal Harmony Generator - Seed: {seed}");

    let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(seed);

    let (graph, duration) = create_graph(&mut seeded_rng);

    println!("Rendering to 05_output_wav_file.wav...");
    let player = Player::from_graph(graph);
    player.play(Some(duration));

    println!("Done! Generated {duration} seconds of quartal harmony.");
}
