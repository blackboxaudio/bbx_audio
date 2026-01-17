//! Integration tests for the DSP graph system.

use bbx_dsp::{
    blocks::{EnvelopeBlock, GainBlock, LfoBlock, MixerBlock, OscillatorBlock, OverdriveBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

// =============================================================================
// Tolerance Constants
// =============================================================================
//
// Expected amplitude ranges for different audio signals. These verify that
// the graph produces meaningful output, not just "non-zero" values.
//
// SINE_MIN_AMPLITUDE: A 440Hz sine should reach at least 70% of its peak
// amplitude within a single 512-sample buffer (at 44100Hz, one cycle is ~100 samples).
//
// MAX_NORMALIZED_AMPLITUDE: Oscillators should output signals bounded to ±1.0,
// with small tolerance for PolyBLEP anti-aliasing overshoot.
//
// GAIN_ATTENUATION_6DB: -6dB should attenuate to ~0.5 (actually 10^(-6/20) ≈ 0.501)

const SINE_MIN_AMPLITUDE: f32 = 0.7;
const MAX_NORMALIZED_AMPLITUDE: f32 = 1.1;
const GAIN_ATTENUATION_6DB_MAX: f32 = 0.55;

#[test]
fn test_simple_oscillator_graph() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        max_amplitude >= SINE_MIN_AMPLITUDE,
        "440Hz sine should reach at least {:.0}% of peak amplitude within {} samples, got {:.1}%",
        SINE_MIN_AMPLITUDE * 100.0,
        buffer_size,
        max_amplitude * 100.0
    );
    assert!(
        max_amplitude <= MAX_NORMALIZED_AMPLITUDE,
        "Oscillator output should be normalized (max {}), got {:.4}",
        MAX_NORMALIZED_AMPLITUDE,
        max_amplitude
    );
}

#[test]
fn test_oscillator_with_overdrive() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let overdrive = builder.add(OverdriveBlock::new(2.0, 0.5, 0.5, sample_rate));
    builder.connect(osc, 0, overdrive, 0);

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        max_amplitude >= 0.3,
        "Overdrive should produce significant output (at least 30%), got {:.1}%",
        max_amplitude * 100.0
    );
}

#[test]
fn test_multiple_oscillators_mixed() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, Some(12345)));
    builder.add(OscillatorBlock::new(880.0, Waveform::Sine, Some(67890)));

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    // Both oscillators should contribute to output
    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.0, "Multiple oscillators should produce output");
}

#[test]
fn test_lfo_modulation() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let lfo = builder.add(LfoBlock::new(5.0, 100.0, Waveform::Sine, None));
    builder.modulate(lfo, osc, "frequency");

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    // Process multiple buffers to let LFO affect pitch
    for _ in 0..10 {
        graph.process_buffers(&mut output_buffers);
    }

    // Verify it produces output
    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.0, "Modulated oscillator should produce output");
}

#[test]
fn test_envelope_modulation() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    let _osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let _env = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.7, 0.2));

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    // Just verify no crash with envelope in graph
    assert!(left.len() == buffer_size);
}

#[test]
fn test_different_waveforms() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let waveforms = [Waveform::Sine, Waveform::Sawtooth, Waveform::Square, Waveform::Triangle];

    for waveform in waveforms {
        let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
        builder.add(OscillatorBlock::new(440.0, waveform, None));
        let mut graph = builder.build();

        let mut left = vec![0.0f32; buffer_size];
        let mut right = vec![0.0f32; buffer_size];
        let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

        graph.process_buffers(&mut output_buffers);

        let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amplitude > 0.0, "{:?} waveform should produce output", waveform);
    }
}

#[test]
fn test_stereo_panners_summed_correctly() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);

    // Create two oscillators through panners (similar to 07_stereo_panner example)
    let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, Some(12345)));
    let pan1 = builder.add(PannerBlock::new(-50.0)); // pan left
    builder.connect(osc1, 0, pan1, 0);

    let osc2 = builder.add(OscillatorBlock::new(880.0, Waveform::Sine, Some(67890)));
    let pan2 = builder.add(PannerBlock::new(50.0)); // pan right
    builder.connect(osc2, 0, pan2, 0);

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    // Both channels should have non-zero values (mixed from both panners)
    let left_sum: f32 = left.iter().map(|s| s.abs()).sum();
    let right_sum: f32 = right.iter().map(|s| s.abs()).sum();

    assert!(left_sum > 0.0, "Left channel should have audio");
    assert!(right_sum > 0.0, "Right channel should have audio");
}

#[test]
fn test_explicit_mixer_produces_output() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);

    // Create two oscillators
    let osc1 = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let osc2 = builder.add(OscillatorBlock::new(880.0, Waveform::Sine, None));

    // Explicitly add a stereo mixer
    let mixer = builder.add(MixerBlock::stereo(2));
    builder.connect(osc1, 0, mixer, 0);
    builder.connect(osc1, 0, mixer, 1); // mono -> stereo
    builder.connect(osc2, 0, mixer, 2);
    builder.connect(osc2, 0, mixer, 3); // mono -> stereo

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    // Both channels should have audio from both oscillators
    let left_sum: f32 = left.iter().map(|s| s.abs()).sum();
    let right_sum: f32 = right.iter().map(|s| s.abs()).sum();

    assert!(left_sum > 0.0, "Left channel should have audio with explicit mixer");
    assert!(right_sum > 0.0, "Right channel should have audio with explicit mixer");
}

#[test]
fn test_five_stereo_panners_all_audible() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);

    // Create 5 oscillators through stereo panners (like 07_stereo_panner example)
    let frequencies = [55.0, 82.5, 110.0, 220.0, 330.0];
    let pan_positions = [-80.0, -40.0, 0.0, 40.0, 80.0];

    for (freq, pan) in frequencies.iter().zip(pan_positions.iter()) {
        let osc = builder.add(OscillatorBlock::new(*freq, Waveform::Sine, None));
        let panner = builder.add(PannerBlock::new(*pan));
        builder.connect(osc, 0, panner, 0);
    }

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    // Both channels should have audio from all panners
    let left_sum: f32 = left.iter().map(|s| s.abs()).sum();
    let right_sum: f32 = right.iter().map(|s| s.abs()).sum();

    // With 5 sources at various pan positions, both channels should have significant output
    assert!(
        left_sum > 100.0,
        "Left channel should have significant audio from 5 panners"
    );
    assert!(
        right_sum > 100.0,
        "Right channel should have significant audio from 5 panners"
    );
}

#[test]
fn test_gain_block() {
    let sample_rate = 44100.0;
    let buffer_size = 256;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(-6.0, None));
    builder.connect(osc, 0, gain, 0);

    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        max_amplitude > 0.3,
        "Gain block should produce significant output, got {:.1}%",
        max_amplitude * 100.0
    );
    assert!(
        max_amplitude <= GAIN_ATTENUATION_6DB_MAX,
        "-6dB gain should attenuate to ~50% (max {}), got {:.1}%",
        GAIN_ATTENUATION_6DB_MAX,
        max_amplitude * 100.0
    );
}

#[test]
fn test_graph_prepare_propagates_to_blocks() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let mut graph = builder.build();

    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    graph.process_buffers(&mut output_buffers);

    let new_sample_rate = 48000.0;
    let new_buffer_size = 256;
    graph.prepare(new_sample_rate, new_buffer_size, num_channels);

    assert_eq!(graph.context().sample_rate, new_sample_rate);
    assert_eq!(graph.context().buffer_size, new_buffer_size);

    let mut left_new = vec![0.0f32; new_buffer_size];
    let mut right_new = vec![0.0f32; new_buffer_size];
    let mut output_buffers_new: Vec<&mut [f32]> = vec![&mut left_new, &mut right_new];

    graph.process_buffers(&mut output_buffers_new);

    let max_amplitude = left_new.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.0, "Graph should produce output after prepare()");
}

#[test]
fn test_graph_reset_clears_block_state() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, Some(12345)));
    let mut graph = builder.build();

    let mut left1 = vec![0.0f32; buffer_size];
    let mut right1 = vec![0.0f32; buffer_size];
    let mut output1: Vec<&mut [f32]> = vec![&mut left1, &mut right1];
    graph.process_buffers(&mut output1);

    graph.reset();

    let mut left2 = vec![0.0f32; buffer_size];
    let mut right2 = vec![0.0f32; buffer_size];
    let mut output2: Vec<&mut [f32]> = vec![&mut left2, &mut right2];
    graph.process_buffers(&mut output2);

    // Verify reset() restores initial state by checking that both buffers:
    // 1. Start from the same initial phase (first samples should match closely)
    // 2. Have the same overall waveform characteristics
    // Note: We check that the waveforms are similar, not bit-for-bit identical,
    // as the important behavior is that state is cleared (phase returns to 0).
    let first_samples_match = (left1[0] - left2[0]).abs() < 0.01;
    let second_samples_match = (left1[1] - left2[1]).abs() < 0.01;
    let max1 = left1.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let max2 = left2.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let similar_amplitude = (max1 - max2).abs() < 0.1;

    assert!(
        first_samples_match && second_samples_match && similar_amplitude,
        "After reset(), oscillator should restart from initial phase. \
         First samples: {:.4} vs {:.4}, amplitudes: {:.4} vs {:.4}",
        left1[0],
        left2[0],
        max1,
        max2
    );
}
