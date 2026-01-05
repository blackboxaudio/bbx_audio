//! Integration tests for the DSP graph system.

use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

#[test]
fn test_simple_oscillator_graph() {
    let sample_rate = 44100.0;
    let buffer_size = 512;
    let num_channels = 2;

    let mut builder = GraphBuilder::<f32>::new(sample_rate, buffer_size, num_channels);
    builder.add_oscillator(440.0, Waveform::Sine, None);
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
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let overdrive = builder.add_overdrive(2.0, 0.5, 0.5, sample_rate);
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
    builder.add_oscillator(440.0, Waveform::Sine, Some(12345));
    builder.add_oscillator(880.0, Waveform::Sine, Some(67890));

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
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let lfo = builder.add_lfo(5.0, 100.0, None);
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
    let _osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let _env = builder.add_envelope(0.01, 0.1, 0.7, 0.2);

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
        builder.add_oscillator(440.0, waveform, None);
        let mut graph = builder.build();

        let mut left = vec![0.0f32; buffer_size];
        let mut right = vec![0.0f32; buffer_size];
        let mut output_buffers: Vec<&mut [f32]> = vec![&mut left, &mut right];

        graph.process_buffers(&mut output_buffers);

        let max_amplitude = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_amplitude > 0.0, "{:?} waveform should produce output", waveform);
    }
}
