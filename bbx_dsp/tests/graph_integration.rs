//! Integration tests for the DSP graph system.

use bbx_dsp::{
    blocks::{EnvelopeBlock, GainBlock, LfoBlock, MixerBlock, OscillatorBlock, OverdriveBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

#[test]
fn test_simple_oscillator_graph() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let mut graph = builder.build();

    // Prepare output buffers
    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

    // Process one buffer
    graph.process_buffers(&mut output_buffers);

    // Output should be non-zero (oscillator is generating audio)
    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.0, "Oscillator should produce non-zero output");
    assert!(max_amplitude <= 1.0, "Output should be normalized");
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

    // Output should be non-zero
    let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_amplitude > 0.0, "Overdrive should produce output");
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
    assert!(max_amplitude > 0.0, "Gain block should produce output");
    assert!(max_amplitude < 1.0, "Gain at -6dB should attenuate signal");
}
