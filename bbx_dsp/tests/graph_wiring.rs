//! Graph wiring guarantees: input ports resolve by index, gaps read as silence,
//! duplicate connections are rejected, and buffers follow a later buffer size.

use bbx_dsp::{
    blocks::{EnvelopeBlock, GainBlock, OscillatorBlock, VcaBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};

const SAMPLE_RATE: f64 = 44_100.0;
const BUFFER_SIZE: usize = 256;

fn render(graph: &mut bbx_dsp::graph::Graph<f32>, length: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; length];
    let mut outputs: [&mut [f32]; 1] = [&mut output];
    graph.process_buffers(&mut outputs);
    output
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |peak, sample| peak.max(sample.abs()))
}

/// The VCA treats a missing audio input as silence and a missing control input
/// as unity, so wiring only one port tells us which port the block saw it on.
#[test]
fn a_lone_audio_connection_lands_on_port_zero() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let vca = builder.add(VcaBlock::new());
    builder.connect(oscillator, 0, vca, 0);
    let mut graph = builder.build();
    let output = render(&mut graph, BUFFER_SIZE);
    assert!(peak(&output) > 0.9, "audio on port 0 with no control passes at unity");
}

/// With connection-order mapping the envelope would land on port 0 and be
/// heard as audio; port-indexed mapping leaves port 0 empty, so silence.
#[test]
fn unconnected_ports_read_as_empty_slices() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let envelope = builder.add(EnvelopeBlock::new(0.001, 0.001, 1.0, 0.001));
    let vca = builder.add(VcaBlock::new());
    builder.connect(envelope, 0, vca, 1);
    let mut graph = builder.build();
    if let Some(bbx_dsp::block::BlockType::Envelope(block)) = graph.get_block_mut(envelope) {
        block.note_on();
    }
    render(&mut graph, BUFFER_SIZE);
    let output = render(&mut graph, BUFFER_SIZE);
    assert_eq!(
        peak(&output),
        0.0,
        "no audio input means silence, even with an open envelope"
    );
}

#[test]
#[should_panic(expected = "input 0")]
fn connecting_the_same_port_twice_fails_the_build() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let a = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let b = builder.add(OscillatorBlock::new(220.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(0.0, None));
    builder.connect(a, 0, gain, 0);
    builder.connect(b, 0, gain, 0);
    let _ = builder.build();
}

/// Re-preparing with a larger buffer size must grow every internal buffer,
/// otherwise the tail of each output stays silent.
#[test]
fn buffers_follow_a_later_buffer_size() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(0.0, None));
    builder.connect(oscillator, 0, gain, 0);
    let mut graph = builder.build();

    let larger = BUFFER_SIZE * 4;
    graph.prepare(SAMPLE_RATE, larger, 1);
    let output = render(&mut graph, larger);

    let tail = &output[BUFFER_SIZE..];
    assert!(
        peak(tail) > 0.5,
        "samples beyond the original buffer size must be rendered"
    );
}
