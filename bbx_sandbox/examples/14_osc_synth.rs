//! Network-controlled synthesizer via OSC (TouchOSC compatible).
//!
//! A simple synthesizer that responds to OSC messages for real-time parameter control.
//! Connect TouchOSC or any OSC-capable app to control the synth.
//!
//! Signal chain: Oscillator → LowPassFilter → Gain → Panner → Output
//!
//! # OSC Address Format
//!
//! All addresses follow the pattern `/blocks/param/<name>`:
//! - `/blocks/param/freq` - Oscillator frequency (0.0-1.0 → 20-2000 Hz)
//! - `/blocks/param/cutoff` - Filter cutoff (0.0-1.0 → 100-5000 Hz)
//! - `/blocks/param/gain` - Output gain (0.0-1.0 → -60 to 0 dB)
//!
//! # TouchOSC Setup
//!
//! 1. Set OSC host to your computer's IP address
//! 2. Set OSC port to 9000
//! 3. Create faders with addresses `/blocks/param/freq`, `/blocks/param/cutoff`, `/blocks/param/gain`
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --example 14_osc_synth -p bbx_sandbox
//! ```

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::{
    block::BlockId,
    blocks::{GainBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_net::{
    NetBufferConsumer, NetMessageType, hash_param_name, net_buffer,
    osc::{OscServer, OscServerConfig},
};
use rodio::{OutputStream, Source};

const NET_BUFFER_CAPACITY: usize = 256;

struct ParamHashes {
    freq: u32,
    cutoff: u32,
    gain: u32,
}

impl ParamHashes {
    fn new() -> Self {
        Self {
            freq: hash_param_name("freq"),
            cutoff: hash_param_name("cutoff"),
            gain: hash_param_name("gain"),
        }
    }
}

struct OscSynth {
    graph: Graph<f32>,
    output_buffers: Vec<AudioBuffer<f32>>,
    net_consumer: NetBufferConsumer,
    oscillator_id: BlockId,
    filter_id: BlockId,
    gain_id: BlockId,
    param_hashes: ParamHashes,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
}

impl OscSynth {
    fn new(net_consumer: NetBufferConsumer) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let num_channels = 2;

        let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

        let oscillator_id = builder.add(OscillatorBlock::new(220.0, Waveform::Sawtooth, None));
        let filter_id = builder.add(LowPassFilterBlock::new(1000.0, 2.0));
        let gain_id = builder.add(GainBlock::new(-12.0, None));
        let panner_id = builder.add(PannerBlock::new(0.0));

        builder
            .connect(oscillator_id, 0, filter_id, 0)
            .connect(filter_id, 0, gain_id, 0)
            .connect(gain_id, 0, panner_id, 0);

        let graph = builder.build();

        let mut output_buffers = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            output_buffers,
            net_consumer,
            oscillator_id,
            filter_id,
            gain_id,
            param_hashes: ParamHashes::new(),
            sample_rate: sample_rate as u32,
            num_channels,
            buffer_size,
            channel_index: 0,
            sample_index: 0,
        }
    }

    fn process_net_events(&mut self) {
        let events = self.net_consumer.drain_into_stack();

        for msg in events {
            if msg.message_type != NetMessageType::ParameterChange {
                continue;
            }

            if msg.param_hash == self.param_hashes.freq {
                let freq = scale_frequency(msg.value);
                self.set_oscillator_frequency(freq);
            } else if msg.param_hash == self.param_hashes.cutoff {
                let cutoff = scale_cutoff(msg.value);
                self.set_filter_cutoff(cutoff);
            } else if msg.param_hash == self.param_hashes.gain {
                let gain_db = scale_gain(msg.value);
                self.set_gain(gain_db);
            }
        }
    }

    fn set_oscillator_frequency(&mut self, frequency: f32) {
        if let Some(bbx_dsp::block::BlockType::Oscillator(osc)) =
            self.graph.get_block_mut(self.oscillator_id)
        {
            osc.set_midi_frequency(frequency);
        }
    }

    fn set_filter_cutoff(&mut self, cutoff: f32) {
        if let Some(bbx_dsp::block::BlockType::LowPassFilter(filter)) =
            self.graph.get_block_mut(self.filter_id)
        {
            filter.cutoff = bbx_dsp::parameter::Parameter::Constant(cutoff);
        }
    }

    fn set_gain(&mut self, gain_db: f32) {
        if let Some(bbx_dsp::block::BlockType::Gain(gain)) = self.graph.get_block_mut(self.gain_id)
        {
            gain.level_db = bbx_dsp::parameter::Parameter::Constant(gain_db);
        }
    }

    fn process(&mut self) -> f32 {
        if self.channel_index == 0 && self.sample_index == 0 {
            self.process_net_events();

            let mut output_refs: Vec<&mut [f32]> =
                self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
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

impl Iterator for OscSynth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for OscSynth {
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

fn scale_frequency(normalized: f32) -> f32 {
    let min_freq = 20.0_f32;
    let max_freq = 2000.0_f32;
    min_freq * (max_freq / min_freq).powf(normalized.clamp(0.0, 1.0))
}

fn scale_cutoff(normalized: f32) -> f32 {
    let min_cutoff = 100.0_f32;
    let max_cutoff = 5000.0_f32;
    min_cutoff * (max_cutoff / min_cutoff).powf(normalized.clamp(0.0, 1.0))
}

fn scale_gain(normalized: f32) -> f32 {
    let min_db = -60.0_f32;
    let max_db = 0.0_f32;
    min_db + (max_db - min_db) * normalized.clamp(0.0, 1.0)
}

fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

fn main() {
    println!("BBX OSC Synthesizer");
    println!("===================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (producer, consumer) = net_buffer(NET_BUFFER_CAPACITY);

    let config = OscServerConfig {
        bind_addr: "0.0.0.0:9000".parse().unwrap(),
        ..Default::default()
    };

    let osc_server = OscServer::new(config, producer);
    let _server_handle = osc_server.spawn();

    if let Some(ip) = get_local_ip() {
        println!("OSC server listening on {ip}:9000");
    } else {
        println!("OSC server listening on port 9000");
    }

    println!("\nOSC Address Mappings:");
    println!("  /blocks/param/freq   - Oscillator frequency (0.0-1.0)");
    println!("  /blocks/param/cutoff - Filter cutoff (0.0-1.0)");
    println!("  /blocks/param/gain   - Output gain (0.0-1.0)");
    println!("\nPress Ctrl+C to exit.\n");

    let synth = OscSynth::new(consumer);

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
