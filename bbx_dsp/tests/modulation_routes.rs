//! Graph-level tests for summed, depth-scaled modulation routes.

use bbx_dsp::{
    block::BlockId,
    blocks::{GainBlock, LfoBlock, OscillatorBlock},
    graph::{GraphBuilder, GraphError},
    parameter::{MAX_MODULATION_ROUTES, ModulationRoute, ModulationSource, ParameterError},
    waveform::Waveform,
};

const SAMPLE_RATE: f64 = 44_100.0;
const BUFFER_SIZE: usize = 512;

/// An LFO at this frequency has advanced exactly a quarter cycle after one
/// buffer, so the first sample of the second buffer is `depth · sin(π/2) = depth`.
const QUARTER_CYCLE_PER_BUFFER_HZ: f64 = SAMPLE_RATE / (4.0 * BUFFER_SIZE as f64);

fn quarter_cycle_lfo(depth: f64) -> LfoBlock<f32> {
    LfoBlock::new(QUARTER_CYCLE_PER_BUFFER_HZ, depth, Waveform::Sine, None)
}

fn render_two_buffers(graph: &mut bbx_dsp::graph::Graph<f32>) {
    let mut output = vec![0.0f32; BUFFER_SIZE];
    for _ in 0..2 {
        let mut outputs: [&mut [f32]; 1] = [&mut output];
        graph.process_buffers(&mut outputs);
    }
}

#[test]
fn two_sources_sum_into_one_parameter_with_per_route_depth() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(0.0, None));
    let lfo_a = builder.add(quarter_cycle_lfo(10.0));
    let lfo_b = builder.add(quarter_cycle_lfo(5.0));
    builder.connect(oscillator, 0, gain, 0);
    builder.modulate(lfo_a, gain, "level_db").unwrap();
    builder
        .modulate_with(
            ModulationRoute::new(ModulationSource::first_output(lfo_b), 2.0),
            gain,
            "level",
        )
        .unwrap();

    let mut graph = builder.build();
    render_two_buffers(&mut graph);

    let level = graph
        .get_block(gain)
        .unwrap()
        .parameter("level_db")
        .unwrap()
        .value(&graph.modulation_values());
    assert!((level - 20.0).abs() < 1e-2, "expected 0 + 10 + 2·5 dB, got {level}");
}

#[test]
fn unmodulated_parameter_reads_its_base() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(-6.0, None));
    builder.connect(oscillator, 0, gain, 0);
    let mut graph = builder.build();
    render_two_buffers(&mut graph);

    let parameter = graph.get_block(gain).unwrap().parameter("level_db").unwrap();
    assert!(!parameter.has_routes());
    assert_eq!(parameter.value(&graph.modulation_values()), -6.0);
}

#[test]
fn parameter_names_are_case_insensitive_and_accept_aliases() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let lfo = builder.add(quarter_cycle_lfo(1.0));
    assert!(builder.modulate(lfo, oscillator, "Frequency").is_ok());
    let gain = builder.add(GainBlock::new(0.0, None));
    assert!(builder.modulate(lfo, gain, "level").is_ok());
}

#[test]
fn unknown_parameter_is_a_typed_error() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let lfo = builder.add(quarter_cycle_lfo(1.0));
    let error = builder.modulate(lfo, oscillator, "cutoff").err().unwrap();
    assert_eq!(
        error,
        GraphError::UnknownParameter {
            block: oscillator,
            name: "cutoff".into(),
        }
    );
}

#[test]
fn missing_source_block_is_a_typed_error() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let missing = BlockId(99);
    let error = builder.modulate(missing, oscillator, "frequency").err().unwrap();
    assert_eq!(error, GraphError::BlockNotFound(missing));
}

#[test]
fn source_output_beyond_declared_outputs_is_a_typed_error() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let lfo = builder.add(quarter_cycle_lfo(1.0));
    let route = ModulationRoute::new(ModulationSource::new(lfo, 1), 1.0);
    let error = builder.modulate_with(route, oscillator, "frequency").err().unwrap();
    assert_eq!(
        error,
        GraphError::InvalidModulationOutput {
            block: lfo,
            output: 1,
            available: 1,
        }
    );
}

#[test]
fn a_non_modulator_source_has_no_outputs_to_route() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(0.0, None));
    let error = builder.modulate(oscillator, gain, "level_db").err().unwrap();
    assert!(matches!(
        error,
        GraphError::InvalidModulationOutput { available: 0, .. }
    ));
}

#[test]
fn route_capacity_is_enforced() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let gain = builder.add(GainBlock::new(0.0, None));
    let lfo = builder.add(quarter_cycle_lfo(1.0));
    for _ in 0..MAX_MODULATION_ROUTES {
        builder.modulate(lfo, gain, "level_db").unwrap();
    }
    let error = builder.modulate(lfo, gain, "level_db").err().unwrap();
    assert_eq!(
        error,
        GraphError::Parameter {
            block: gain,
            name: "level_db".into(),
            error: ParameterError::RoutesFull {
                capacity: MAX_MODULATION_ROUTES,
            },
        }
    );
}

#[test]
fn topology_snapshot_lists_every_route_with_its_depth() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let gain = builder.add(GainBlock::new(0.0, None));
    let lfo_a = builder.add(quarter_cycle_lfo(1.0));
    let lfo_b = builder.add(quarter_cycle_lfo(1.0));
    builder.modulate(lfo_a, gain, "level_db").unwrap();
    builder
        .modulate_with(
            ModulationRoute::new(ModulationSource::first_output(lfo_b), 0.5),
            gain,
            "level_db",
        )
        .unwrap();

    let snapshot = builder.capture_topology();
    let mut routes: Vec<(usize, f64)> = snapshot
        .modulation_connections
        .iter()
        .filter(|connection| connection.to_block == gain.0 && connection.parameter_name == "level_db")
        .map(|connection| (connection.from_block, connection.depth))
        .collect();
    routes.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(routes, vec![(lfo_a.0, 1.0), (lfo_b.0, 0.5)]);
}

#[test]
fn midi_frequency_sets_the_base_and_routes_still_add() {
    let mut builder = GraphBuilder::<f32>::new(SAMPLE_RATE, BUFFER_SIZE, 1);
    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let lfo = builder.add(quarter_cycle_lfo(3.0));
    builder.modulate(lfo, oscillator, "frequency").unwrap();
    let mut graph = builder.build();

    if let Some(bbx_dsp::block::BlockType::Oscillator(block)) = graph.get_block_mut(oscillator) {
        block.set_midi_frequency(880.0);
    }
    render_two_buffers(&mut graph);

    let modulation = graph.modulation_values();
    let frequency = graph
        .get_block(oscillator)
        .unwrap()
        .parameter("frequency")
        .unwrap()
        .value_with_base(880.0, &modulation);
    assert!((frequency - 883.0).abs() < 1e-2, "expected 880 + 3, got {frequency}");
}
