//! Signal level and clipping tests.
//!
//! These tests systematically isolate each component of the synth signal chain
//! to verify amplitude bounds at each processing stage.
//!
//! # Tolerance Constants
//!
//! - POLYBLEP_TOLERANCE (1.05): PolyBLEP/PolyBLAMP anti-aliasing can overshoot by ~5% near waveform discontinuities.
//!   This is expected behavior for band-limited synthesis.
//!
//! - TRIANGLE_TOLERANCE (1.15): Triangle waves have additional overshoot at high frequencies due to PolyBLAMP's 8x
//!   scaling factor for the BLAMP correction.
//!
//! - HIGH_Q_FILTER_TOLERANCE (2.05): Resonant filters with Q > 1 can boost frequencies near cutoff. With 2/Q
//!   compensation, peak gain is limited to ~2x.

use bbx_dsp::{
    block::{Block, BlockType},
    blocks::{EnvelopeBlock, GainBlock, LowPassFilterBlock, OscillatorBlock, VcaBlock},
    channel::ChannelLayout,
    context::DspContext,
    graph::GraphBuilder,
    waveform::Waveform,
};

const POLYBLEP_TOLERANCE: f64 = 1.05;
const TRIANGLE_TOLERANCE: f64 = 1.15;
const HIGH_Q_FILTER_TOLERANCE: f64 = 2.05;

fn make_context(sample_rate: f64, buffer_size: usize) -> DspContext {
    DspContext {
        sample_rate,
        num_channels: 1,
        buffer_size,
        current_sample: 0,
        channel_layout: ChannelLayout::default(),
    }
}

// =============================================================================
// Phase 1: Isolated Component Testing
// =============================================================================

/// Test peak amplitude of each waveform in isolation.
/// All waveforms should output ±1.0 with at most ~5% overshoot from PolyBLEP.
#[test]
fn test_oscillator_peak_amplitude_by_waveform() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let frequencies = [100.0, 440.0, 1000.0, 5000.0, 10000.0];
    let waveforms = [
        (Waveform::Sine, "Sine"),
        (Waveform::Triangle, "Triangle"),
        (Waveform::Sawtooth, "Sawtooth"),
        (Waveform::Square, "Square"),
    ];

    for (waveform, name) in waveforms {
        for freq in frequencies {
            let mut osc = OscillatorBlock::<f64>::new(freq, waveform, Some(42));
            let context = make_context(sample_rate, buffer_size);

            let mut max_amplitude = 0.0f64;
            let num_buffers = 100;

            for _ in 0..num_buffers {
                let mut output = vec![0.0f64; buffer_size];
                let mut outputs: [&mut [f64]; 1] = [&mut output];
                osc.process(&[], &mut outputs, &[], &context);

                for sample in output {
                    max_amplitude = max_amplitude.max(sample.abs());
                }
            }

            let tolerance = if matches!(waveform, Waveform::Triangle) {
                TRIANGLE_TOLERANCE
            } else {
                POLYBLEP_TOLERANCE
            };
            assert!(
                max_amplitude <= tolerance,
                "{} at {}Hz: peak amplitude {:.6} exceeds {} tolerance",
                name,
                freq,
                max_amplitude,
                tolerance
            );
        }
    }
}

/// Verify envelope never exceeds 0-1 range.
#[test]
fn test_envelope_output_range() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let mut env = EnvelopeBlock::<f64>::new(0.01, 0.1, 0.7, 0.2);
    let context = make_context(sample_rate, buffer_size);

    env.note_on();

    let mut max_output = 0.0f64;
    let mut min_output = 1.0f64;

    // Process through attack/decay/sustain
    for _ in 0..200 {
        let mut output = vec![0.0f64; buffer_size];
        let mut outputs: [&mut [f64]; 1] = [&mut output];
        env.process(&[], &mut outputs, &[], &context);

        for sample in &output {
            max_output = max_output.max(*sample);
            min_output = min_output.min(*sample);
        }
    }

    // Trigger release
    env.note_off();
    for _ in 0..100 {
        let mut output = vec![0.0f64; buffer_size];
        let mut outputs: [&mut [f64]; 1] = [&mut output];
        env.process(&[], &mut outputs, &[], &context);

        for sample in &output {
            max_output = max_output.max(*sample);
            min_output = min_output.min(*sample);
        }
    }

    assert!(max_output <= 1.0, "Envelope max {:.6} exceeds 1.0", max_output);
    assert!(min_output >= 0.0, "Envelope min {:.6} below 0.0", min_output);
}

/// Test filter gain at various resonance levels.
/// With 2/Q compensation, resonance peak is limited to ~2.0 while preserving passband.
#[test]
fn test_filter_resonance_gain() {
    let sample_rate = 44100.0;
    let buffer_size = 4096;
    let cutoff = 1000.0;
    let q_values = [0.5, 0.707, 1.0, 2.0, 5.0, 10.0];

    for q in q_values {
        let mut filter = LowPassFilterBlock::<f64>::new(cutoff, q);

        let context = make_context(sample_rate, buffer_size);
        let test_freq = cutoff;
        let mut max_output = 0.0f64;

        for buffer_idx in 0..50 {
            let mut input = vec![0.0f64; buffer_size];
            let mut output = vec![0.0f64; buffer_size];

            for i in 0..buffer_size {
                let t = (buffer_idx * buffer_size + i) as f64 / sample_rate;
                input[i] = (2.0 * std::f64::consts::PI * test_freq * t).sin();
            }

            let inputs: [&[f64]; 1] = [&input];
            let mut outputs: [&mut [f64]; 1] = [&mut output];
            filter.process(&inputs, &mut outputs, &[], &context);

            if buffer_idx > 5 {
                for sample in output {
                    max_output = max_output.max(sample.abs());
                }
            }
        }

        assert!(
            max_output <= HIGH_Q_FILTER_TOLERANCE,
            "Q={} should not exceed {:.2}, got {:.4}",
            q,
            HIGH_Q_FILTER_TOLERANCE,
            max_output
        );
    }
}

// =============================================================================
// Phase 2: Incremental Signal Chain Testing
// =============================================================================

/// Test oscillator only - should be bounded to ±1.0 (with small PolyBLEP tolerance).
#[test]
fn test_chain_oscillator_only() {
    let sample_rate = 44100.0;
    let buffer_size = 512;

    for waveform in [Waveform::Sawtooth, Waveform::Square, Waveform::Sine, Waveform::Triangle] {
        let mut builder = GraphBuilder::<f64>::new(sample_rate, buffer_size, 2);
        builder.add(OscillatorBlock::new(440.0, waveform, Some(42)));
        let mut graph = builder.build();

        let mut max_amplitude = 0.0f64;
        for _ in 0..100 {
            let mut left = vec![0.0f64; buffer_size];
            let mut right = vec![0.0f64; buffer_size];
            {
                let mut output_buffers: Vec<&mut [f64]> = vec![&mut left, &mut right];
                graph.process_buffers(&mut output_buffers);
            }
            for &s in &left {
                max_amplitude = max_amplitude.max(s.abs());
            }
        }

        let tolerance = if matches!(waveform, Waveform::Triangle) {
            TRIANGLE_TOLERANCE
        } else {
            POLYBLEP_TOLERANCE
        };
        assert!(
            max_amplitude <= tolerance,
            "Oscillator {:?} exceeds bounds ({}): {:.6}",
            waveform,
            tolerance,
            max_amplitude
        );
    }
}

/// Test oscillator + VCA with envelope - should still be bounded (envelope attenuates).
#[test]
fn test_chain_oscillator_plus_vca() {
    let sample_rate = 44100.0;
    let buffer_size = 512;

    let mut builder = GraphBuilder::<f64>::new(sample_rate, buffer_size, 2);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sawtooth, Some(42)));
    let env = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.8, 0.2));
    let vca = builder.add(VcaBlock::new());

    builder.connect(osc, 0, vca, 0);
    builder.connect(env, 0, vca, 1);

    let mut graph = builder.build();

    if let Some(BlockType::Envelope(envelope)) = graph.get_block_mut(env) {
        envelope.note_on();
    }

    let mut max_amplitude = 0.0f64;
    for _ in 0..100 {
        let mut left = vec![0.0f64; buffer_size];
        let mut right = vec![0.0f64; buffer_size];
        {
            let mut output_buffers: Vec<&mut [f64]> = vec![&mut left, &mut right];
            graph.process_buffers(&mut output_buffers);
        }
        for &s in &left {
            max_amplitude = max_amplitude.max(s.abs());
        }
    }

    assert!(
        max_amplitude <= POLYBLEP_TOLERANCE,
        "Osc+VCA exceeds bounds ({}): {:.6}",
        POLYBLEP_TOLERANCE,
        max_amplitude
    );
}

/// Test oscillator + VCA + filter at various Q values.
/// With 2/Q compensation, even high Q filters should stay within bounds.
#[test]
fn test_chain_oscillator_vca_filter() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let q_values = [0.707, 2.0, 5.0];

    for q in q_values {
        let mut builder = GraphBuilder::<f64>::new(sample_rate, buffer_size, 2);
        let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sawtooth, Some(42)));
        let env = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.8, 0.2));
        let vca = builder.add(VcaBlock::new());
        let filter = builder.add(LowPassFilterBlock::new(1000.0, q));

        builder.connect(osc, 0, vca, 0);
        builder.connect(env, 0, vca, 1);
        builder.connect(vca, 0, filter, 0);

        let mut graph = builder.build();
        if let Some(BlockType::Envelope(envelope)) = graph.get_block_mut(env) {
            envelope.note_on();
        }

        let mut max_amplitude = 0.0f64;
        for _ in 0..100 {
            let mut left = vec![0.0f64; buffer_size];
            let mut right = vec![0.0f64; buffer_size];
            {
                let mut output_buffers: Vec<&mut [f64]> = vec![&mut left, &mut right];
                graph.process_buffers(&mut output_buffers);
            }
            for &s in &left {
                max_amplitude = max_amplitude.max(s.abs());
            }
        }

        assert!(
            max_amplitude <= HIGH_Q_FILTER_TOLERANCE,
            "Osc+VCA+Filter (Q={}) exceeds bounds ({}): {:.6}",
            q,
            HIGH_Q_FILTER_TOLERANCE,
            max_amplitude
        );
    }
}

/// Test full synth chain with gain stage.
/// With 2/Q compensation, even high Q (5.0) should stay within bounds.
#[test]
fn test_full_synth_chain() {
    let sample_rate = 44100.0;
    let buffer_size = 512;

    let mut builder = GraphBuilder::<f64>::new(sample_rate, buffer_size, 2);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sawtooth, Some(42)));
    let env = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.8, 0.2));
    let vca = builder.add(VcaBlock::new());
    let filter = builder.add(LowPassFilterBlock::new(1000.0, 5.0));
    let gain = builder.add(GainBlock::new(0.0, None));

    builder.connect(osc, 0, vca, 0);
    builder.connect(env, 0, vca, 1);
    builder.connect(vca, 0, filter, 0);
    builder.connect(filter, 0, gain, 0);

    let mut graph = builder.build();
    if let Some(BlockType::Envelope(envelope)) = graph.get_block_mut(env) {
        envelope.note_on();
    }

    let mut max_amplitude = 0.0f64;
    for _ in 0..100 {
        let mut left = vec![0.0f64; buffer_size];
        let mut right = vec![0.0f64; buffer_size];
        {
            let mut output_buffers: Vec<&mut [f64]> = vec![&mut left, &mut right];
            graph.process_buffers(&mut output_buffers);
        }
        for &s in &left {
            max_amplitude = max_amplitude.max(s.abs());
        }
    }

    assert!(
        max_amplitude <= HIGH_Q_FILTER_TOLERANCE,
        "Full synth chain (Q=5.0) exceeds bounds ({}): {:.6}",
        HIGH_Q_FILTER_TOLERANCE,
        max_amplitude
    );
}

/// Test filter at high cutoff (20kHz) with low Q values.
/// The g-factor compensation should prevent clipping near Nyquist.
#[test]
fn test_filter_high_cutoff_low_q() {
    let sample_rate = 44100.0;
    let buffer_size = 4096;
    let cutoff = 20000.0;
    let q_values = [0.707, 1.0, 1.1, 1.2];

    for q in q_values {
        let mut filter = LowPassFilterBlock::<f64>::new(cutoff, q);

        let context = make_context(sample_rate, buffer_size);
        let test_freq = 18000.0;
        let mut max_output = 0.0f64;

        for buffer_idx in 0..100 {
            let mut input = vec![0.0f64; buffer_size];
            let mut output = vec![0.0f64; buffer_size];

            for i in 0..buffer_size {
                let t = (buffer_idx * buffer_size + i) as f64 / sample_rate;
                input[i] = (2.0 * std::f64::consts::PI * test_freq * t).sin();
            }

            let inputs: [&[f64]; 1] = [&input];
            let mut outputs: [&mut [f64]; 1] = [&mut output];
            filter.process(&inputs, &mut outputs, &[], &context);

            if buffer_idx > 10 {
                for sample in output {
                    max_output = max_output.max(sample.abs());
                }
            }
        }

        assert!(
            max_output <= POLYBLEP_TOLERANCE,
            "High cutoff (20kHz) Q={} should not exceed {}, got {:.4}",
            q,
            POLYBLEP_TOLERANCE,
            max_output
        );
    }
}
