# Synthesizer Support Plan

This document outlines the roadmap for adding synthesizer capabilities to bbx_dsp, enabling the creation of virtual instruments (VSTi/AUi) in addition to effect plugins.

## Overview

Synthesizer support requires several new components:
1. **ADSR Envelope** - Attack/Decay/Sustain/Release envelope generator
2. **Voice Management** - Polyphonic voice allocation and stealing
3. **MIDI Processing** - Note events, velocity, pitch bend, CC
4. **Oscillator Enhancements** - Phase sync, unison, detune
5. **Additional Modulators** - Envelope followers, sample & hold

## Architecture

### Voice-Based Processing

Unlike effect plugins that process audio continuously, synthesizers generate audio in response to MIDI events. Each voice is a complete signal chain:

```
┌─────────────────────────────────────────────────────────────┐
│ Voice                                                       │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐ │
│  │Oscillator│──▶│  Filter  │──▶│   Amp    │──▶│  Output  │ │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘ │
│       ▲              ▲              ▲                       │
│       │              │              │                       │
│  ┌────┴────┐    ┌────┴────┐   ┌────┴────┐                  │
│  │Pitch Env│    │Filter Env│  │ Amp Env │                  │
│  └─────────┘    └─────────┘   └─────────┘                  │
│                                                             │
│  MIDI Note → Frequency                                      │
│  Velocity → Envelope depth/filter cutoff                    │
└─────────────────────────────────────────────────────────────┘
```

### PolySynth Block

The `PolySynthBlock` manages multiple voices and routes MIDI to them:

```
┌─────────────────────────────────────────────────────────────┐
│ PolySynthBlock                                              │
│                                                             │
│  MIDI In ──┬──▶ Voice 0 ──┐                                │
│            ├──▶ Voice 1 ──┤                                │
│            ├──▶ Voice 2 ──┼──▶ Mix ──▶ Output              │
│            ├──▶ Voice 3 ──┤                                │
│            └──▶ ...    ───┘                                │
│                                                             │
│  Voice Allocator (round-robin, steal oldest, etc.)          │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. ADSR Envelope

```rust
/// Envelope state machine
pub enum EnvelopeState {
    Idle,
    Attack { start_time: u64 },
    Decay { start_time: u64 },
    Sustain,
    Release { start_time: u64, release_level: f64 },
}

/// ADSR envelope generator
pub struct AdsrBlock<S: Sample> {
    /// Attack time in seconds
    pub attack: ModulatableParam<S>,
    /// Decay time in seconds
    pub decay: ModulatableParam<S>,
    /// Sustain level (0.0 - 1.0)
    pub sustain: ModulatableParam<S>,
    /// Release time in seconds
    pub release: ModulatableParam<S>,

    /// Current state
    state: EnvelopeState,
    /// Current output level
    current_level: f64,
    /// Sample counter for timing
    sample_counter: u64,
}

impl<S: Sample> AdsrBlock<S> {
    pub fn new(attack: S, decay: S, sustain: S, release: S) -> Self {
        Self {
            attack: ModulatableParam::new(attack),
            decay: ModulatableParam::new(decay),
            sustain: ModulatableParam::new(sustain),
            release: ModulatableParam::new(release),
            state: EnvelopeState::Idle,
            current_level: 0.0,
            sample_counter: 0,
        }
    }

    /// Trigger envelope (note on)
    pub fn trigger(&mut self, velocity: f64) {
        self.state = EnvelopeState::Attack {
            start_time: self.sample_counter,
        };
        // Optional: scale envelope by velocity
    }

    /// Release envelope (note off)
    pub fn release_note(&mut self) {
        if !matches!(self.state, EnvelopeState::Idle) {
            self.state = EnvelopeState::Release {
                start_time: self.sample_counter,
                release_level: self.current_level,
            };
        }
    }

    /// Check if envelope has finished
    pub fn is_idle(&self) -> bool {
        matches!(self.state, EnvelopeState::Idle)
    }

    /// Generate next sample
    fn next_sample(&mut self, sample_rate: f64, modulation_values: &[S]) -> f64 {
        let attack_time = self.attack.evaluate(modulation_values).to_f64();
        let decay_time = self.decay.evaluate(modulation_values).to_f64();
        let sustain_level = self.sustain.evaluate(modulation_values).to_f64();
        let release_time = self.release.evaluate(modulation_values).to_f64();

        let attack_samples = (attack_time * sample_rate) as u64;
        let decay_samples = (decay_time * sample_rate) as u64;
        let release_samples = (release_time * sample_rate) as u64;

        self.current_level = match &self.state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack { start_time } => {
                let elapsed = self.sample_counter - start_time;
                if elapsed >= attack_samples {
                    self.state = EnvelopeState::Decay {
                        start_time: self.sample_counter,
                    };
                    1.0
                } else if attack_samples == 0 {
                    1.0
                } else {
                    elapsed as f64 / attack_samples as f64
                }
            }

            EnvelopeState::Decay { start_time } => {
                let elapsed = self.sample_counter - start_time;
                if elapsed >= decay_samples {
                    self.state = EnvelopeState::Sustain;
                    sustain_level
                } else if decay_samples == 0 {
                    sustain_level
                } else {
                    let t = elapsed as f64 / decay_samples as f64;
                    1.0 - t * (1.0 - sustain_level)
                }
            }

            EnvelopeState::Sustain => sustain_level,

            EnvelopeState::Release { start_time, release_level } => {
                let elapsed = self.sample_counter - start_time;
                if elapsed >= release_samples {
                    self.state = EnvelopeState::Idle;
                    0.0
                } else if release_samples == 0 {
                    0.0
                } else {
                    let t = elapsed as f64 / release_samples as f64;
                    release_level * (1.0 - t)
                }
            }
        };

        self.sample_counter += 1;
        self.current_level
    }
}
```

### 2. Voice Structure

```rust
/// A single synthesizer voice
pub struct Voice<S: Sample> {
    /// Audio generation
    pub oscillator: OscillatorBlock<S>,
    /// Optional second oscillator for detune/unison
    pub oscillator2: Option<OscillatorBlock<S>>,
    /// Filter (optional)
    pub filter: Option<FilterBlock<S>>,
    /// Amplitude envelope
    pub amp_env: AdsrBlock<S>,
    /// Filter envelope (optional)
    pub filter_env: Option<AdsrBlock<S>>,
    /// Pitch envelope (optional)
    pub pitch_env: Option<AdsrBlock<S>>,
    /// LFO for vibrato (optional)
    pub lfo: Option<LfoBlock<S>>,

    /// Currently playing note (None if voice is free)
    pub note: Option<u8>,
    /// Note velocity (0.0 - 1.0)
    pub velocity: f64,
    /// Note start time (for voice stealing priority)
    pub start_time: u64,
    /// Pitch bend amount (-1.0 to 1.0)
    pub pitch_bend: f64,
}

impl<S: Sample> Voice<S> {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            oscillator: OscillatorBlock::new(440.0, Waveform::Sawtooth),
            oscillator2: None,
            filter: Some(FilterBlock::new(8000.0, 0.7, FilterMode::LowPass)),
            amp_env: AdsrBlock::new(
                S::from_f64(0.01),  // 10ms attack
                S::from_f64(0.1),   // 100ms decay
                S::from_f64(0.7),   // 70% sustain
                S::from_f64(0.3),   // 300ms release
            ),
            filter_env: None,
            pitch_env: None,
            lfo: None,
            note: None,
            velocity: 0.0,
            start_time: 0,
            pitch_bend: 0.0,
        }
    }

    /// Start playing a note
    pub fn note_on(&mut self, note: u8, velocity: u8, time: u64) {
        self.note = Some(note);
        self.velocity = velocity as f64 / 127.0;
        self.start_time = time;

        // Convert MIDI note to frequency
        let freq = midi_to_frequency(note);
        self.oscillator.set_frequency(S::from_f64(freq));
        if let Some(ref mut osc2) = self.oscillator2 {
            osc2.set_frequency(S::from_f64(freq));
        }

        // Trigger envelopes
        self.amp_env.trigger(self.velocity);
        if let Some(ref mut env) = self.filter_env {
            env.trigger(self.velocity);
        }
        if let Some(ref mut env) = self.pitch_env {
            env.trigger(self.velocity);
        }

        // Reset oscillator phase for consistent attack
        self.oscillator.reset();
        if let Some(ref mut osc2) = self.oscillator2 {
            osc2.reset();
        }
    }

    /// Release the note
    pub fn note_off(&mut self) {
        self.amp_env.release_note();
        if let Some(ref mut env) = self.filter_env {
            env.release_note();
        }
        if let Some(ref mut env) = self.pitch_env {
            env.release_note();
        }
    }

    /// Check if voice is available for new note
    pub fn is_free(&self) -> bool {
        self.note.is_none() && self.amp_env.is_idle()
    }

    /// Check if voice is releasing (can be stolen with lower priority)
    pub fn is_releasing(&self) -> bool {
        self.note.is_none() && !self.amp_env.is_idle()
    }

    /// Process audio for this voice
    pub fn process(
        &mut self,
        output: &mut [S],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        if self.is_free() {
            return;
        }

        let buffer_size = output.len();
        let mut osc_output = vec![S::ZERO; buffer_size];

        // Generate oscillator output
        self.oscillator.generate(&mut osc_output, modulation_values, context);

        // Mix second oscillator if present
        if let Some(ref mut osc2) = self.oscillator2 {
            let mut osc2_output = vec![S::ZERO; buffer_size];
            osc2.generate(&mut osc2_output, modulation_values, context);
            for (out, osc2) in osc_output.iter_mut().zip(osc2_output.iter()) {
                *out = (*out + *osc2) * S::from_f64(0.5);
            }
        }

        // Apply filter
        if let Some(ref mut filter) = self.filter {
            let mut filtered = vec![S::ZERO; buffer_size];
            filter.process_mono(&osc_output, &mut filtered, modulation_values, context);
            osc_output = filtered;
        }

        // Apply amplitude envelope
        for (i, sample) in osc_output.iter().enumerate() {
            let env_value = self.amp_env.next_sample(context.sample_rate, modulation_values);
            output[i] = output[i] + *sample * S::from_f64(env_value * self.velocity);
        }

        // Check if voice has finished
        if self.amp_env.is_idle() {
            self.note = None;
        }
    }
}

/// Convert MIDI note number to frequency in Hz
pub fn midi_to_frequency(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0)
}
```

### 3. Voice Allocation

```rust
/// Voice stealing strategy
#[derive(Clone, Copy, Debug)]
pub enum VoiceStealingMode {
    /// Ignore new notes when all voices are busy
    None,
    /// Steal the oldest note
    OldestFirst,
    /// Steal the quietest note (lowest velocity or releasing)
    QuietestFirst,
    /// Round-robin allocation
    RoundRobin,
}

/// Polyphonic synthesizer block
pub struct PolySynthBlock<S: Sample, const VOICES: usize> {
    /// Voice pool
    voices: [Voice<S>; VOICES],
    /// Voice stealing strategy
    stealing_mode: VoiceStealingMode,
    /// Global pitch bend (from MIDI)
    pitch_bend: f64,
    /// Pitch bend range in semitones
    pitch_bend_range: f64,
    /// Last allocated voice (for round-robin)
    last_voice: usize,
    /// Sample counter
    sample_counter: u64,
}

impl<S: Sample, const VOICES: usize> PolySynthBlock<S, VOICES> {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::new(sample_rate)),
            stealing_mode: VoiceStealingMode::OldestFirst,
            pitch_bend: 0.0,
            pitch_bend_range: 2.0,  // ±2 semitones
            last_voice: 0,
            sample_counter: 0,
        }
    }

    /// Find a free voice or steal one
    fn allocate_voice(&mut self) -> Option<usize> {
        // First, look for completely free voice
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.is_free() {
                return Some(i);
            }
        }

        // Then, look for releasing voice
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.is_releasing() {
                return Some(i);
            }
        }

        // Finally, apply stealing strategy
        match self.stealing_mode {
            VoiceStealingMode::None => None,

            VoiceStealingMode::OldestFirst => {
                let oldest = self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.start_time)
                    .map(|(i, _)| i);
                oldest
            }

            VoiceStealingMode::QuietestFirst => {
                let quietest = self.voices
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.velocity.partial_cmp(&b.velocity).unwrap()
                    })
                    .map(|(i, _)| i);
                quietest
            }

            VoiceStealingMode::RoundRobin => {
                let voice = (self.last_voice + 1) % VOICES;
                self.last_voice = voice;
                Some(voice)
            }
        }
    }

    /// Handle MIDI note on
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        if velocity == 0 {
            // Note on with velocity 0 is note off
            self.note_off(note);
            return;
        }

        if let Some(voice_idx) = self.allocate_voice() {
            self.voices[voice_idx].note_on(note, velocity, self.sample_counter);
        }
    }

    /// Handle MIDI note off
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note == Some(note) {
                voice.note_off();
            }
        }
    }

    /// Handle MIDI pitch bend
    pub fn set_pitch_bend(&mut self, value: i16) {
        // value is -8192 to 8191
        self.pitch_bend = value as f64 / 8192.0;
        // Apply to all voices
        for voice in &mut self.voices {
            voice.pitch_bend = self.pitch_bend;
        }
    }

    /// Handle MIDI control change
    pub fn control_change(&mut self, cc: u8, value: u8) {
        match cc {
            1 => {
                // Mod wheel - could control LFO depth
            }
            64 => {
                // Sustain pedal
                if value < 64 {
                    // Pedal up - release all held notes
                    // (Implement sustain pedal logic)
                }
            }
            123 => {
                // All notes off
                for voice in &mut self.voices {
                    voice.note_off();
                }
            }
            _ => {}
        }
    }
}

impl<S: Sample, const VOICES: usize> Block<S> for PolySynthBlock<S, VOICES> {
    fn process(
        &mut self,
        _inputs: &[&[S]],  // Synths don't use audio input
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        let buffer_size = outputs[0].len();

        // Clear output buffers
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = S::ZERO;
            }
        }

        // Process each voice and mix
        let mut voice_output = vec![S::ZERO; buffer_size];
        for voice in &mut self.voices {
            if !voice.is_free() {
                voice_output.fill(S::ZERO);
                voice.process(&mut voice_output, modulation_values, context);

                // Mix into output (mono for now)
                for (out, voice_sample) in outputs[0].iter_mut().zip(voice_output.iter()) {
                    *out = *out + *voice_sample;
                }
            }
        }

        // Copy left to right for stereo
        if outputs.len() > 1 {
            outputs[1].copy_from_slice(&outputs[0]);
        }

        self.sample_counter += buffer_size as u64;
    }

    fn input_count(&self) -> usize { 0 }  // Synths don't have audio input
    fn output_count(&self) -> usize { 2 }
    fn modulation_outputs(&self) -> &[ModulationOutput] { &[] }
}
```

### 4. MIDI Event Processing

```rust
/// MIDI event for FFI
#[repr(C)]
pub struct MidiEvent {
    /// Sample offset within buffer (0 = start of buffer)
    pub sample_offset: u32,
    /// MIDI status byte
    pub status: u8,
    /// MIDI data byte 1
    pub data1: u8,
    /// MIDI data byte 2
    pub data2: u8,
}

impl MidiEvent {
    /// Get MIDI channel (0-15)
    pub fn channel(&self) -> u8 {
        self.status & 0x0F
    }

    /// Get message type
    pub fn message_type(&self) -> MidiMessageType {
        match self.status & 0xF0 {
            0x80 => MidiMessageType::NoteOff,
            0x90 => MidiMessageType::NoteOn,
            0xA0 => MidiMessageType::Aftertouch,
            0xB0 => MidiMessageType::ControlChange,
            0xC0 => MidiMessageType::ProgramChange,
            0xD0 => MidiMessageType::ChannelPressure,
            0xE0 => MidiMessageType::PitchBend,
            _ => MidiMessageType::Unknown,
        }
    }

    /// Get pitch bend value (-8192 to 8191)
    pub fn pitch_bend_value(&self) -> i16 {
        let value = ((self.data2 as u16) << 7) | (self.data1 as u16);
        (value as i16) - 8192
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MidiMessageType {
    NoteOff,
    NoteOn,
    Aftertouch,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    Unknown,
}
```

### 5. FFI Extensions for MIDI

```rust
/// Process audio with MIDI events (for synthesizers)
#[no_mangle]
pub extern "C" fn bbx_process_with_midi(
    engine: *mut DspEngine,
    output_left: *mut f32,
    output_right: *mut f32,
    num_samples: usize,
    midi_events: *const MidiEvent,
    num_events: usize,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    let events = if midi_events.is_null() || num_events == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(midi_events, num_events) }
    };

    unsafe {
        let mut outputs = [
            std::slice::from_raw_parts_mut(output_left, num_samples),
            std::slice::from_raw_parts_mut(output_right, num_samples),
        ];

        engine.graph.process_with_midi(&[], &mut outputs, events);
    }
}

/// Send MIDI event immediately (for real-time processing)
#[no_mangle]
pub extern "C" fn bbx_send_midi(
    engine: *mut DspEngine,
    status: u8,
    data1: u8,
    data2: u8,
) {
    let engine = match unsafe { engine.as_mut() } {
        Some(e) => e,
        None => return,
    };

    engine.graph.send_midi(MidiEvent {
        sample_offset: 0,
        status,
        data1,
        data2,
    });
}
```

### 6. Graph Extensions

```rust
impl<S: Sample> Graph<S> {
    /// Process with MIDI events (sample-accurate timing)
    pub fn process_with_midi(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        midi_events: &[MidiEvent],
    ) {
        let buffer_size = outputs[0].len();

        // Sort events by sample offset
        let mut sorted_events: Vec<_> = midi_events.iter().collect();
        sorted_events.sort_by_key(|e| e.sample_offset);

        // Process in chunks between MIDI events for sample-accurate timing
        let mut current_sample = 0;

        for event in sorted_events {
            let event_sample = (event.sample_offset as usize).min(buffer_size);

            // Process audio up to this event
            if event_sample > current_sample {
                self.process_chunk(inputs, outputs, current_sample, event_sample);
            }

            // Dispatch MIDI event
            self.dispatch_midi(event);
            current_sample = event_sample;
        }

        // Process remaining samples
        if current_sample < buffer_size {
            self.process_chunk(inputs, outputs, current_sample, buffer_size);
        }
    }

    fn dispatch_midi(&mut self, event: &MidiEvent) {
        // Find PolySynth blocks and dispatch to them
        for block in &mut self.blocks {
            if let BlockType::PolySynth(synth) = block {
                match event.message_type() {
                    MidiMessageType::NoteOn => {
                        synth.note_on(event.data1, event.data2);
                    }
                    MidiMessageType::NoteOff => {
                        synth.note_off(event.data1);
                    }
                    MidiMessageType::PitchBend => {
                        synth.set_pitch_bend(event.pitch_bend_value());
                    }
                    MidiMessageType::ControlChange => {
                        synth.control_change(event.data1, event.data2);
                    }
                    _ => {}
                }
            }
        }
    }
}
```

## Configuration Schema

### Synth Voice Configuration

```json
{
    "voice": {
        "oscillator1": {
            "waveform": "sawtooth",
            "detune_cents": 0
        },
        "oscillator2": {
            "enabled": true,
            "waveform": "square",
            "detune_cents": 7,
            "mix": 0.5
        },
        "filter": {
            "enabled": true,
            "mode": "lowpass",
            "cutoff": 4000,
            "resonance": 0.5,
            "env_depth": 2000,
            "key_tracking": 0.5
        },
        "amp_env": {
            "attack": 0.01,
            "decay": 0.2,
            "sustain": 0.7,
            "release": 0.5
        },
        "filter_env": {
            "attack": 0.05,
            "decay": 0.3,
            "sustain": 0.3,
            "release": 0.5
        },
        "lfo": {
            "enabled": true,
            "frequency": 5.0,
            "waveform": "sine",
            "destinations": [
                { "target": "pitch", "depth": 0.1 },
                { "target": "filter_cutoff", "depth": 500 }
            ]
        }
    },
    "polyphony": {
        "max_voices": 16,
        "stealing_mode": "oldest_first"
    },
    "pitch_bend_range": 2
}
```

### Full Synth Graph Configuration

```json
{
    "type": "synthesizer",
    "blocks": [
        {
            "id": 0,
            "type": "poly_synth",
            "name": "main_synth",
            "config": {
                "voices": 8,
                "stealing_mode": "oldest_first"
            }
        },
        {
            "id": 1,
            "type": "filter",
            "name": "master_filter",
            "params": {
                "mode": "lowpass",
                "cutoff": 20000,
                "resonance": 0.0
            }
        },
        {
            "id": 2,
            "type": "gain",
            "name": "master_gain",
            "params": {
                "level": 0.0
            }
        },
        {
            "id": 3,
            "type": "output",
            "name": "audio_out"
        }
    ],
    "connections": [
        { "from": [0, 0], "to": [1, 0] },
        { "from": [0, 1], "to": [1, 1] },
        { "from": [1, 0], "to": [2, 0] },
        { "from": [1, 1], "to": [2, 1] },
        { "from": [2, 0], "to": [3, 0] },
        { "from": [2, 1], "to": [3, 1] }
    ],
    "parameter_bindings": {
        "MASTER_VOLUME": { "block": 2, "param": "level" },
        "MASTER_CUTOFF": { "block": 1, "param": "cutoff" }
    }
}
```

## C++ Integration for Synthesizers

### processor.cpp (Synthesizer)

```cpp
void SynthAudioProcessor::processBlock(
    juce::AudioBuffer<float>& buffer,
    juce::MidiBuffer& midiMessages
)
{
    juce::ScopedNoDenormals noDenormals;

    auto* left = buffer.getWritePointer(0);
    auto* right = buffer.getWritePointer(1);
    auto numSamples = static_cast<size_t>(buffer.getNumSamples());

    // Convert JUCE MidiBuffer to bbx_dsp format
    std::vector<MidiEvent> events;
    events.reserve(midiMessages.getNumEvents());

    for (const auto metadata : midiMessages) {
        auto msg = metadata.getMessage();
        events.push_back(MidiEvent {
            .sample_offset = static_cast<uint32_t>(metadata.samplePosition),
            .status = static_cast<uint8_t>(msg.getRawData()[0]),
            .data1 = msg.getRawDataSize() > 1 ? msg.getRawData()[1] : 0,
            .data2 = msg.getRawDataSize() > 2 ? msg.getRawData()[2] : 0,
        });
    }

    // Process with MIDI
    bbx_process_with_midi(
        m_dsp,
        left, right,
        numSamples,
        events.data(),
        events.size()
    );
}
```

## Implementation Order

### Phase 1: Core Envelope
1. Implement `AdsrBlock` with state machine
2. Add as `BlockType::Adsr` variant
3. Add modulation output for envelope value
4. Test with simple gain modulation

### Phase 2: Voice Structure
1. Create basic `Voice` struct with oscillator + envelope
2. Implement `note_on`, `note_off`, `process`
3. Add MIDI frequency conversion
4. Test monophonic synthesis

### Phase 3: Polyphony
1. Implement `PolySynthBlock` with voice pool
2. Add voice allocation strategies
3. Implement voice stealing
4. Test polyphonic playback

### Phase 4: MIDI Integration
1. Add `MidiEvent` FFI type
2. Implement `bbx_process_with_midi`
3. Add sample-accurate MIDI timing
4. Test with JUCE MidiBuffer

### Phase 5: Voice Features
1. Add filter to voice chain
2. Implement filter envelope
3. Add pitch envelope
4. Add per-voice LFO

### Phase 6: Advanced Features
1. Unison/detune (multiple oscillators per voice)
2. Portamento/glide
3. MPE (MIDI Polyphonic Expression) support
4. Velocity curves

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_adsr_envelope() {
    let mut env = AdsrBlock::<f32>::new(0.01, 0.1, 0.5, 0.3);
    env.trigger(1.0);

    // Should ramp up during attack
    let sample1 = env.next_sample(44100.0, &[]);
    assert!(sample1 > 0.0);

    // Fast-forward to sustain
    for _ in 0..10000 {
        env.next_sample(44100.0, &[]);
    }
    let sustain = env.next_sample(44100.0, &[]);
    assert!((sustain - 0.5).abs() < 0.01);

    // Release
    env.release_note();
    for _ in 0..20000 {
        env.next_sample(44100.0, &[]);
    }
    assert!(env.is_idle());
}

#[test]
fn test_voice_allocation() {
    let mut synth = PolySynthBlock::<f32, 4>::new(44100.0);

    // Play 4 notes
    synth.note_on(60, 100);
    synth.note_on(64, 100);
    synth.note_on(67, 100);
    synth.note_on(72, 100);

    // All voices should be allocated
    for voice in &synth.voices {
        assert!(voice.note.is_some());
    }

    // 5th note should steal oldest
    synth.note_on(76, 100);
    assert!(synth.voices.iter().any(|v| v.note == Some(76)));
    assert!(!synth.voices.iter().any(|v| v.note == Some(60)));
}
```

### Integration Tests

```rust
#[test]
fn test_synth_audio_output() {
    let mut graph = Graph::<f32>::new(44100.0, 512, 2);
    // Add synth block
    // Send note on
    // Process
    // Verify non-silent output
}
```

## Future Considerations

### MPE Support
- Per-note pitch bend
- Per-note pressure (aftertouch)
- Per-note slide (CC 74)

### Modulation Matrix
- Flexible source → destination routing
- Multiple sources per destination
- Scaling and curves

### Wavetable Synthesis
- Multiple wavetables per oscillator
- Wavetable morphing
- Import from WAV files

### FM Synthesis
- Operator blocks
- Feedback routing
- Algorithm presets
