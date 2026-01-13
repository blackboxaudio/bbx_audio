//! Terminal MIDI synthesizer example.
//!
//! A monophonic subtractive synthesizer that responds to real-time MIDI input.
//! Connect a USB MIDI keyboard and play notes to hear the synthesizer.
//!
//! Signal chain: Oscillator → VCA ← Envelope → LowPassFilter → Output
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --example 06_midi_synth -p bbx_sandbox
//! ```

use std::{
    io::{Write, stdin, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::{
    block::BlockId,
    blocks::{EnvelopeBlock, GainBlock, LowPassFilterBlock, OscillatorBlock, VcaBlock},
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_midi::{MidiBufferConsumer, MidiBufferProducer, MidiMessage, MidiMessageStatus, midi_buffer};
use midir::{Ignore, MidiInput, MidiInputConnection};
use rodio::{OutputStream, Source};

const MIDI_BUFFER_CAPACITY: usize = 256;

struct VoiceState {
    current_note: Option<u8>,
}

impl VoiceState {
    fn new() -> Self {
        Self { current_note: None }
    }

    fn note_on(&mut self, note: u8) {
        self.current_note = Some(note);
    }

    fn note_off(&mut self, note: u8) -> bool {
        if self.current_note == Some(note) {
            self.current_note = None;
            true
        } else {
            false
        }
    }
}

struct MidiSynth {
    graph: Graph<f32>,
    output_buffers: Vec<AudioBuffer<f32>>,
    voice_state: VoiceState,
    oscillator_id: BlockId,
    envelope_id: BlockId,
    midi_consumer: MidiBufferConsumer,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
}

impl MidiSynth {
    fn new(midi_consumer: MidiBufferConsumer) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let num_channels = 2;

        let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

        let oscillator_id = builder.add(OscillatorBlock::new(440.0, Waveform::Sawtooth, None));
        let envelope_id = builder.add(EnvelopeBlock::new(0.01, 0.1, 0.7, 0.3));
        let vca_id = builder.add(VcaBlock::new());
        let filter_id = builder.add(LowPassFilterBlock::new(1000.0, 1.5));
        let gain_id = builder.add(GainBlock::new(-6.0, None));

        builder
            .connect(oscillator_id, 0, vca_id, 0)
            .connect(envelope_id, 0, vca_id, 1)
            .connect(vca_id, 0, filter_id, 0)
            .connect(filter_id, 0, gain_id, 0);

        let graph = builder.build();

        let mut output_buffers = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            output_buffers,
            voice_state: VoiceState::new(),
            oscillator_id,
            envelope_id,
            midi_consumer,
            sample_rate: sample_rate as u32,
            num_channels,
            buffer_size,
            channel_index: 0,
            sample_index: 0,
        }
    }

    fn process_midi_events(&mut self) {
        while let Some(msg) = self.midi_consumer.try_pop() {
            match msg.get_status() {
                MidiMessageStatus::NoteOn => {
                    let velocity = msg.get_velocity().unwrap_or(0);
                    if velocity > 0 {
                        if let Some(note) = msg.get_note_number() {
                            self.voice_state.note_on(note);

                            if let Some(freq) = msg.get_note_frequency() {
                                self.set_oscillator_frequency(freq);
                            }
                            self.trigger_envelope();
                        }
                    } else {
                        self.handle_note_off(&msg);
                    }
                }
                MidiMessageStatus::NoteOff => {
                    self.handle_note_off(&msg);
                }
                _ => {}
            }
        }
    }

    fn handle_note_off(&mut self, msg: &MidiMessage) {
        if let Some(note) = msg.get_note_number()
            && self.voice_state.note_off(note)
        {
            self.release_envelope();
        }
    }

    fn set_oscillator_frequency(&mut self, frequency: f32) {
        if let Some(bbx_dsp::block::BlockType::Oscillator(osc)) = self.graph.get_block_mut(self.oscillator_id) {
            osc.set_midi_frequency(frequency);
        }
    }

    fn trigger_envelope(&mut self) {
        if let Some(bbx_dsp::block::BlockType::Envelope(env)) = self.graph.get_block_mut(self.envelope_id) {
            env.note_on();
        }
    }

    fn release_envelope(&mut self) {
        if let Some(bbx_dsp::block::BlockType::Envelope(env)) = self.graph.get_block_mut(self.envelope_id) {
            env.note_off();
        }
    }

    fn process(&mut self) -> f32 {
        if self.channel_index == 0 && self.sample_index == 0 {
            self.process_midi_events();

            let mut output_refs: Vec<&mut [f32]> = self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
            self.graph.process_buffers(&mut output_refs);
        }

        let sample = self.output_buffers[self.channel_index][self.sample_index];

        self.channel_index += 1;
        if self.channel_index >= self.num_channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl Iterator for MidiSynth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for MidiSynth {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.num_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn setup_midi_input(mut producer: MidiBufferProducer) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("MIDI Synth Input").ok()?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => {
            println!("No MIDI input ports found.");
            return None;
        }
        1 => {
            let port_name = midi_in.port_name(&in_ports[0]).unwrap_or_default();
            println!("Using MIDI port: {port_name}");
            in_ports[0].clone()
        }
        _ => {
            println!("\nAvailable MIDI input ports:");
            for (idx, port) in in_ports.iter().enumerate() {
                let name = midi_in.port_name(port).unwrap_or_default();
                println!("  {idx}: {name}");
            }
            print!("\nSelect port: ");
            stdout().flush().ok()?;

            let mut input = String::new();
            stdin().read_line(&mut input).ok()?;
            let idx: usize = input.trim().parse().ok()?;

            if idx >= in_ports.len() {
                println!("Invalid port selection.");
                return None;
            }

            let port_name = midi_in.port_name(&in_ports[idx]).unwrap_or_default();
            println!("Using MIDI port: {port_name}");
            in_ports[idx].clone()
        }
    };

    let connection = midi_in
        .connect(
            &in_port,
            "midi-synth-input",
            move |_timestamp, message_bytes, _| {
                let message = MidiMessage::from(message_bytes);
                let _ = producer.try_send(message);
            },
            (),
        )
        .ok()?;

    Some(connection)
}

fn main() {
    println!("BBX MIDI Synthesizer");
    println!("====================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (producer, consumer) = midi_buffer(MIDI_BUFFER_CAPACITY);

    let _midi_connection = match setup_midi_input(producer) {
        Some(conn) => conn,
        None => {
            println!("Failed to set up MIDI input. Exiting.");
            return;
        }
    };

    println!("\nSynth ready! Play notes on your MIDI keyboard.");
    println!("Press Ctrl+C to exit.\n");

    let synth = MidiSynth::new(consumer);

    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to open audio output: {e}");
            return;
        }
    };

    if let Err(e) = stream_handle.play_raw(synth.convert_samples()) {
        println!("Failed to start audio playback: {e}");
        return;
    }

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nShutting down...");
}
